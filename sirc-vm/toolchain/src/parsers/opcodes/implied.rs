use crate::parsers::instruction::{parse_instruction_tag, InstructionToken};
use nom::branch::alt;
use nom::character::complete::one_of;
use nom::combinator::peek;
use nom::error::{ErrorKind, FromExternalError};
use nom_supreme::error::ErrorTree;
use nom_supreme::ParserExt;
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, ImpliedInstructionData, Instruction, InstructionData,
};
use peripheral_cpu::registers::AddressRegisterName;

fn tag_to_instruction(tag: String) -> Instruction {
    match tag.as_str() {
        "RETS" => Instruction::ReturnFromSubroutine,
        "NOOP" => Instruction::NoOperation,
        "WAIT" => Instruction::WaitForException,
        "RETE" => Instruction::ReturnFromException,
        _ => panic!("No tag mapping for instruction [{}]", tag),
    }
}

use super::super::shared::AsmResult;

///
/// Parses immediate arithmetic/logic opcodes
///
/// ```
/// use toolchain::parsers::opcodes::implied::implied;
/// use toolchain::parsers::instruction::InstructionToken;
/// use peripheral_cpu::instructions::definitions::{ConditionFlags, Instruction, InstructionData, RegisterInstructionData};
/// use nom_supreme::error::ErrorTree;
/// use nom_supreme::final_parser::{final_parser, Location};
///
/// let parsed_instruction = match final_parser::<&str, InstructionToken, ErrorTree<&str>, ErrorTree<Location>>(implied)("WAIT|==") {
///   Ok(tokens) => tokens,
///   Err(error) => panic!("Error parsing instruction:\n{}", error),
/// };
/// let (op_code, condition_flag) = match parsed_instruction.instruction {
///     InstructionData::Implied(inner) => (inner.op_code, inner.condition_flag),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// // TODO: Make a helper function or something to make these asserts smaller
/// assert_eq!(op_code, Instruction::WaitForException);
/// assert_eq!(condition_flag, ConditionFlags::Equal);
/// ```
pub fn implied(i: &str) -> AsmResult<InstructionToken> {
    let mut instructions = alt((
        parse_instruction_tag("RETS").context("RETS"),
        parse_instruction_tag("NOOP").context("NOOP"),
        parse_instruction_tag("WAIT").context("WAIT"),
        parse_instruction_tag("RETE").context("RETE"),
    ));

    let (i, (tag, condition_flag)) = instructions(i)?;

    match peek(one_of::<&str, &str, ErrorTree<&str>>("\r\n"))(i) {
        Ok(_) => {}
        Err(_) => {
            let error_string = format!(
                "The [{}] does not support any addressing modes (e.g. NOOP or RETE)",
                tag
            );
            return Err(nom::Err::Failure(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )));
        }
    }

    match tag_to_instruction(tag) {
        // Special case, RETS doesn't have any arguments, but the instruction format encodes a long jump
        // to the link register (TODO: should this be moved out to a different parser or something?)
        Instruction::ReturnFromSubroutine => Ok((
            i,
            InstructionToken {
                instruction: InstructionData::Immediate(ImmediateInstructionData {
                    op_code: Instruction::ReturnFromSubroutine,
                    register: AddressRegisterName::Address.to_register_index(),
                    value: 0x0,
                    additional_flags: AddressRegisterName::LinkRegister.to_register_index(),
                    condition_flag,
                }),
                symbol_ref: None,
            },
        )),
        other => Ok((
            i,
            InstructionToken {
                instruction: InstructionData::Implied(ImpliedInstructionData {
                    op_code: other,
                    condition_flag,
                }),
                symbol_ref: None,
            },
        )),
    }
}
