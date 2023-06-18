use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, InstructionToken,
};
use nom::error::{ErrorKind, FromExternalError};
use nom::sequence::tuple;
use nom::{branch::alt, combinator::map_res};
use nom_supreme::error::ErrorTree;
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
    let instructions = alt((
        parse_instruction_tag("RETS"),
        parse_instruction_tag("NOOP"),
        parse_instruction_tag("WAIT"),
        parse_instruction_tag("RETE"),
    ));

    map_res(
        tuple((instructions, parse_instruction_operands)),
        |((tag, condition_flag), operands)| match operands.as_slice() {
            [] => match tag_to_instruction(tag) {
                // Special case, RETS doesn't have any arguments, but the instruction format encodes a long jump
                // to the link register (TODO: should this be moved out to a different parser or something?)
                Instruction::ReturnFromSubroutine => Ok(InstructionToken {
                    instruction: InstructionData::Immediate(ImmediateInstructionData {
                        op_code: Instruction::ReturnFromSubroutine,
                        register: AddressRegisterName::Address.to_register_index(),
                        value: 0x0,
                        additional_flags: AddressRegisterName::LinkRegister.to_register_index(),
                        condition_flag,
                    }),
                    symbol_ref: None,
                }),
                other => Ok(InstructionToken {
                    instruction: InstructionData::Implied(ImpliedInstructionData {
                        op_code: other,
                        condition_flag,
                    }),
                    symbol_ref: None,
                }),
            },

            _ => {
                // TODO: Homogenise the error messages for bad addressing modes across all the opcode parsers
                let error_string = format!(
                    "The [{}] opcode only supports implied addressing mode (e.g. WAIT)",
                    tag
                );
                Err(nom::Err::Failure(ErrorTree::from_external_error(
                    i.to_owned(),
                    ErrorKind::Fail,
                    error_string.as_str(),
                )))
            }
        },
    )(i)
}
