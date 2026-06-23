use super::super::shared::AsmResult;
use crate::parsers::instruction::{
    parse_instruction_operands0, parse_instruction_tag, AddressingMode, ImmediateType,
};
use crate::types::instruction::InstructionToken;
use nom::branch::alt;
use nom::error::{ErrorKind, FromExternalError};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData, ShiftOperand,
    ShiftType,
};

fn reject_status_update<'a>(input: &'a str, tag: &str) -> AsmResult<'a, 'a, InstructionToken> {
    let error_string = format!(
        "The [{tag}] opcode does not support an explicit status register update source. Coprocessor calls preserve the status register."
    );
    Err(nom::Err::Failure(ErrorTree::from_external_error(
        input,
        ErrorKind::Fail,
        error_string.as_str(),
    )))
}

fn reject_hidden_destination<'a>(
    input: &'a str,
    tag: &str,
    public_syntax: &str,
) -> AsmResult<'a, 'a, InstructionToken> {
    let error_string = format!(
        "The [{tag}] opcode does not support a destination register field. Use {public_syntax} instead."
    );
    Err(nom::Err::Failure(ErrorTree::from_external_error(
        input,
        ErrorKind::Fail,
        error_string.as_str(),
    )))
}

fn reject_shift<'a>(input: &'a str, tag: &str) -> AsmResult<'a, 'a, InstructionToken> {
    let error_string = format!("The [{tag}] opcode does not support shift syntax.");
    Err(nom::Err::Failure(ErrorTree::from_external_error(
        input,
        ErrorKind::Fail,
        error_string.as_str(),
    )))
}

pub fn coprocessor(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();
    let mut instructions = alt((parse_instruction_tag("COPI"), parse_instruction_tag("COPR")));

    let (i_after_instruction, (tag, condition_flag, status_register_update_source)) =
        instructions(i)?;

    if status_register_update_source.is_some() {
        return reject_status_update(i, tag.as_str());
    }

    let (i, operands) = parse_instruction_operands0(i_after_instruction)?;

    match (tag.as_str(), operands.as_slice()) {
        ("COPI", [AddressingMode::Immediate(immediate_type)]) => match immediate_type {
            ImmediateType::Value(value) => Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Immediate(ImmediateInstructionData {
                        op_code: Instruction::CoprocessorCallImmediate,
                        register: 0x0,
                        value: *value,
                        condition_flag,
                        additional_flags: 0x0,
                    }),
                    ..Default::default()
                },
            )),
            ImmediateType::PlaceHolder(placeholder_name) => Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Immediate(ImmediateInstructionData {
                        op_code: Instruction::CoprocessorCallImmediate,
                        register: 0x0,
                        value: 0x0,
                        condition_flag,
                        additional_flags: 0x0,
                    }),
                    placeholder_name: Some(placeholder_name.clone()),
                    ..Default::default()
                },
            )),
            ImmediateType::SymbolRef(_) => {
                let error_string =
                    format!("The [{tag}] opcode does not support symbol refs at this time");
                Err(nom::Err::Failure(ErrorTree::from_external_error(
                    i_after_instruction,
                    ErrorKind::Fail,
                    error_string.as_str(),
                )))
            }
        },
        ("COPI", [AddressingMode::Immediate(_), AddressingMode::ShiftDefinition(_)])
        | ("COPR", [AddressingMode::DirectRegister(_), AddressingMode::ShiftDefinition(_)]) => {
            reject_shift(i_after_instruction, tag.as_str())
        }
        ("COPI", [AddressingMode::DirectRegister(_), ..]) => {
            reject_hidden_destination(i_after_instruction, tag.as_str(), "COPI #value")
        }
        ("COPR", [AddressingMode::DirectRegister(src_register)]) => Ok((
            i,
            InstructionToken {
                input_length,
                instruction: InstructionData::Register(RegisterInstructionData {
                    op_code: Instruction::CoprocessorCallRegister,
                    r1: 0x0,
                    r2: 0x0,
                    r3: src_register.to_register_index(),
                    shift_operand: ShiftOperand::Immediate,
                    shift_type: ShiftType::None,
                    shift_count: 0,
                    condition_flag,
                    additional_flags: 0x0,
                }),
                ..Default::default()
            },
        )),
        ("COPR", [AddressingMode::DirectRegister(_), AddressingMode::DirectRegister(_), ..]) => {
            reject_hidden_destination(i_after_instruction, tag.as_str(), "COPR rS")
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
