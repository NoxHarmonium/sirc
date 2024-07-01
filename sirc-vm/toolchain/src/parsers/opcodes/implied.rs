use super::super::shared::AsmResult;
use crate::parsers::instruction::parse_instruction_tag;
use crate::types::instruction::InstructionToken;
use nom::branch::alt;
use nom::character::complete::one_of;
use nom::error::{ErrorKind, FromExternalError};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ImmediateInstructionData, Instruction, InstructionData,
};
use peripheral_cpu::registers::AddressRegisterName;

///
/// Parses immediate arithmetic/logic opcodes
///
/// ```
/// use toolchain::parsers::opcodes::implied::implied;
/// use toolchain::parsers::instruction::InstructionToken;
/// use peripheral_cpu::coprocessors::processing_unit::definitions::{ConditionFlags, Instruction, InstructionData, RegisterInstructionData};
/// use nom_supreme::error::ErrorTree;
/// use nom_supreme::final_parser::{final_parser, Location};
///
/// let parsed_instruction = match final_parser::<&str, InstructionToken, ErrorTree<&str>, ErrorTree<Location>>(implied)("WAIT|==\n") {
///   Ok(tokens) => tokens,
///   Err(error) => panic!("Error parsing instruction:\n{}", error),
/// };
/// let (op_code, condition_flag) = match parsed_instruction.instruction {
///     InstructionData::Immediate(inner) => (inner.op_code, inner.condition_flag),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// assert_eq!(op_code, Instruction::CoprocessorCallImmediate);
/// assert_eq!(condition_flag, ConditionFlags::Equal);
/// ```
pub fn implied(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();
    let mut instructions = alt((
        parse_instruction_tag("RETS"),
        parse_instruction_tag("NOOP"),
        parse_instruction_tag("WAIT"),
        parse_instruction_tag("RETE"),
    ));

    let (i, (tag, condition_flag)) = instructions(i)?;
    let (i, _) = one_of::<&str, &str, ErrorTree<&str>>("\r\n")(i).map_err(|_| {
        let error_string =
            format!("The [{tag}] does not support any addressing modes (e.g. NOOP or RETE)");
        nom::Err::Failure(ErrorTree::from_external_error(
            i,
            ErrorKind::Fail,
            error_string.as_str(),
        ))
    })?;

    match tag.as_str() {
        // Returning from a subroutine is just loading the link register into the PC again
        // TODO: Double check that RETS instruction doesn't have security flaw
        // category=Hardware
        // Double check this doesn't provide a way for code running in non system mode to change the program segment (ph)
        "RETS" => Ok((
            i,
            InstructionToken {
                input_length,
                instruction: InstructionData::Immediate(ImmediateInstructionData {
                    op_code: Instruction::LoadEffectiveAddressFromIndirectImmediate,
                    register: AddressRegisterName::ProgramCounter.to_register_index(),
                    value: 0x0,
                    condition_flag,
                    additional_flags: AddressRegisterName::LinkRegister.to_register_index(),
                }),
                ..Default::default()
            },
        )),
        // Pseudo instruction - add zero to register 1 which does nothing
        // I guess any instruction with condition flag set to "NEVER" would also work
        "NOOP" => Ok((
            i,
            InstructionToken {
                input_length,
                instruction: InstructionData::Immediate(ImmediateInstructionData {
                    op_code: Instruction::AddImmediate,
                    register: 0x0,
                    value: 0x0,
                    additional_flags: 0x0, // No status register update
                    condition_flag,
                }),
                ..Default::default()
            },
        )),
        // Tell the exception module to halt the CPU until an event comes through by passing it 0xFFFF
        "WAIT" => Ok((
            i,
            InstructionToken {
                input_length,
                instruction: InstructionData::Immediate(ImmediateInstructionData {
                    op_code: Instruction::CoprocessorCallImmediate,
                    register: 0x0,
                    value: 0x01F00,
                    additional_flags: 0x0,
                    condition_flag,
                }),
                ..Default::default()
            },
        )),
        // Tell the exception module to return by passing it 0xFFFE
        "RETE" => Ok((
            i,
            InstructionToken {
                input_length,
                instruction: InstructionData::Immediate(ImmediateInstructionData {
                    op_code: Instruction::CoprocessorCallImmediate,
                    register: 0x0,
                    value: 0x1A00,
                    additional_flags: 0x0,
                    condition_flag,
                }),
                ..Default::default()
            },
        )),
        _ => {
            let error_string = format!("Mismatch between parser and handler for tag [{tag}] ");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}
