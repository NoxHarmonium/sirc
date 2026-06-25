use super::super::shared::AsmResult;
use crate::parsers::instruction::{
    parse_instruction_operands0, parse_instruction_tag, AddressingMode, ImmediateType,
};
use crate::types::instruction::InstructionToken;
use nom::branch::alt;
use nom::error::{ErrorKind, FromExternalError};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ImmediateInstructionData, Instruction, InstructionData,
};

const DMA_COPROCESSOR_ID: u16 = 0x2;
const MATHS_COPROCESSOR_ID: u16 = 0x3;

fn command(coprocessor_id: u16, operation: u16, operand: u16) -> u16 {
    (coprocessor_id << 12) | (operation << 8) | operand
}

fn reject_status_update<'a>(input: &'a str, tag: &str) -> AsmResult<'a, 'a, InstructionToken> {
    let error_string = format!(
        "The [{tag}] meta instruction does not support an explicit status register update source. Coprocessor calls preserve the status register."
    );
    Err(nom::Err::Failure(ErrorTree::from_external_error(
        input,
        ErrorKind::Fail,
        error_string.as_str(),
    )))
}

fn reject_symbol_operand<'a>(input: &'a str, tag: &str) -> AsmResult<'a, 'a, InstructionToken> {
    let error_string =
        format!("The [{tag}] meta instruction only supports literal immediate operands");
    Err(nom::Err::Failure(ErrorTree::from_external_error(
        input,
        ErrorKind::Fail,
        error_string.as_str(),
    )))
}

fn reject_count<'a>(
    input: &'a str,
    tag: &str,
    count: u16,
    max_count: u16,
) -> AsmResult<'a, 'a, InstructionToken> {
    let error_string = format!(
        "The [{tag}] meta instruction only supports counts in the range 0-{max_count}, got {count}"
    );
    Err(nom::Err::Failure(ErrorTree::from_external_error(
        input,
        ErrorKind::Fail,
        error_string.as_str(),
    )))
}

pub fn coprocessor_meta(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();
    let mut instructions = alt((
        parse_instruction_tag("DMAR"),
        parse_instruction_tag("DMAW"),
        parse_instruction_tag("DMAT"),
        parse_instruction_tag("MULU"),
        parse_instruction_tag("MULS"),
        parse_instruction_tag("DIVU"),
        parse_instruction_tag("DIVS"),
    ));

    let (i_after_instruction, (tag, condition_flag, status_register_update_source)) =
        instructions(i)?;

    if status_register_update_source.is_some() {
        return reject_status_update(i, tag.as_str());
    }

    let (i, operands) = parse_instruction_operands0(i_after_instruction)?;

    let construct_instruction = |value: u16| InstructionToken {
        input_length,
        instruction: InstructionData::Immediate(ImmediateInstructionData {
            op_code: Instruction::CoprocessorCallImmediate,
            register: 0x0,
            value,
            additional_flags: 0x0,
            condition_flag,
        }),
        ..Default::default()
    };

    let parse_count = |immediate_type: &ImmediateType| -> Result<u16, nom::Err<ErrorTree<&str>>> {
        if let ImmediateType::Value(value) = immediate_type {
            Ok(*value)
        } else {
            let error_string =
                format!("The [{tag}] meta instruction only supports literal immediate operands");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i_after_instruction,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    };

    match (tag.as_str(), operands.as_slice()) {
        ("DMAR" | "DMAW", [AddressingMode::Immediate(immediate_type)]) => {
            let count = parse_count(immediate_type)?;
            if count > 7 {
                return reject_count(i_after_instruction, tag.as_str(), count, 7);
            }

            let operation = if tag == "DMAR" { 0x8 } else { 0x9 };
            Ok((
                i,
                construct_instruction(command(DMA_COPROCESSOR_ID, operation, count)),
            ))
        }
        ("DMAT", [AddressingMode::Immediate(immediate_type)]) => {
            let count = parse_count(immediate_type)?;
            if count > 0xFF {
                return reject_count(i_after_instruction, tag.as_str(), count, 0xFF);
            }

            Ok((
                i,
                construct_instruction(command(DMA_COPROCESSOR_ID, 0xA, count)),
            ))
        }
        ("MULU", []) => Ok((
            i,
            construct_instruction(command(MATHS_COPROCESSOR_ID, 0x0, 0x0)),
        )),
        ("MULS", []) => Ok((
            i,
            construct_instruction(command(MATHS_COPROCESSOR_ID, 0x1, 0x0)),
        )),
        ("DIVU", []) => Ok((
            i,
            construct_instruction(command(MATHS_COPROCESSOR_ID, 0x2, 0x0)),
        )),
        ("DIVS", []) => Ok((
            i,
            construct_instruction(command(MATHS_COPROCESSOR_ID, 0x3, 0x0)),
        )),
        (_, [AddressingMode::Immediate(immediate_type)])
            if !matches!(immediate_type, ImmediateType::Value(_)) =>
        {
            reject_symbol_operand(i_after_instruction, tag.as_str())
        }
        modes => {
            let error_string = format!("Invalid addressing mode for {tag}: ({modes:?})");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i_after_instruction,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}
