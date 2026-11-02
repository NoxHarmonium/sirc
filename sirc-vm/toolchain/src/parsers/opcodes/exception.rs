use super::super::shared::AsmResult;
use super::numeric_field;
use crate::parsers::instruction::{
    parse_instruction_operands0, parse_instruction_tag, AddressingMode,
};
use crate::types::instruction::InstructionToken;
use crate::types::numeric::NumericField;
use nom::branch::alt;
use nom::error::{ErrorKind, FromExternalError};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ImmediateInstructionData, Instruction, InstructionData,
};
use peripheral_cpu::registers::{AddressRegisterName, RegisterName};

///
/// Parses meta instructions that encode into exception co-processor COP instructions
///
/// ```
/// use toolchain::parsers::opcodes::exception::exception;
/// use toolchain::types::instruction::InstructionToken;
/// use peripheral_cpu::coprocessors::processing_unit::definitions::{ConditionFlags, Instruction, InstructionData, RegisterInstructionData};
/// use nom_supreme::error::ErrorTree;
/// use nom_supreme::final_parser::{final_parser, Location};
///
/// let parsed_instruction = match final_parser::<&str, InstructionToken, ErrorTree<&str>, ErrorTree<Location>>(exception)("RSET|==") {
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
pub fn exception(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();
    let mut instructions = alt((
        parse_instruction_tag("EXCP"),
        parse_instruction_tag("WAIT"),
        parse_instruction_tag("RETE"),
        parse_instruction_tag("RSET"),
        parse_instruction_tag("ETFR"),
        parse_instruction_tag("ETTR"),
    ));

    let (i_after_instruction, (tag, condition_flag, status_register_update_source)) =
        instructions(i)?;

    if status_register_update_source.is_some() {
        let error_string =
            format!("The [{tag}] opcode does not support an explicit status register update source. Only ALU instructions can update the status register as a side-effect.");
        return Err(nom::Err::Failure(ErrorTree::from_external_error(
            i,
            ErrorKind::Fail,
            error_string.as_str(),
        )));
    }

    let (i, operands) = parse_instruction_operands0(i_after_instruction)?;

    let (base, numeric) = match (tag.as_str(), operands.as_slice()) {
        ("EXCP", [AddressingMode::Immediate(value)]) => {
            (0x1100, Some((value, NumericField::ExceptionVector)))
        }
        ("WAIT", []) => (0x1900, None),
        ("RETE", []) => (0x1a00, None),
        ("RSET", []) => (0x1b00, None),
        ("ETFR", [AddressingMode::Immediate(value)]) => {
            (0x1c30, Some((value, NumericField::ExceptionLinkIndex)))
        }
        ("ETTR", [AddressingMode::Immediate(value)]) => {
            (0x1d30, Some((value, NumericField::ExceptionLinkIndex)))
        }
        (
            "ETFR",
            [AddressingMode::DirectRegister(RegisterName::R7), AddressingMode::Immediate(value)],
        ) => (0x1c20, Some((value, NumericField::ExceptionLinkIndex))),
        (
            "ETTR",
            [AddressingMode::Immediate(value), AddressingMode::DirectRegister(RegisterName::R7)],
        ) => (0x1d20, Some((value, NumericField::ExceptionLinkIndex))),
        (
            "ETFR",
            [AddressingMode::DirectAddressRegister(AddressRegisterName::Address), AddressingMode::Immediate(value)],
        ) => (0x1c10, Some((value, NumericField::ExceptionLinkIndex))),
        (
            "ETTR",
            [AddressingMode::Immediate(value), AddressingMode::DirectAddressRegister(AddressRegisterName::Address)],
        ) => (0x1d10, Some((value, NumericField::ExceptionLinkIndex))),
        modes => {
            let error_string = format!("Invalid addressing mode for {tag}: ({modes:?})");
            return Err(nom::Err::Failure(ErrorTree::from_external_error(
                i_after_instruction,
                ErrorKind::Fail,
                error_string,
            )));
        }
    };
    let (value, numeric_placeholders) = numeric.map_or_else(
        || Ok((0, Vec::new())),
        |(operand, field)| numeric_field(i_after_instruction, operand, field),
    )?;
    Ok((
        i,
        InstructionToken {
            input_length,
            instruction: InstructionData::Immediate(ImmediateInstructionData {
                op_code: Instruction::CoprocessorCallImmediate,
                register: 0,
                value: base | value,
                additional_flags: 0,
                condition_flag,
            }),
            numeric_placeholders,
            ..Default::default()
        },
    ))
}
