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
use peripheral_cpu::registers::AddressRegisterName;

const DMA_COPROCESSOR_ID: u16 = 0x2;
const MATHS_COPROCESSOR_ID: u16 = 0x3;
const DMA_REGISTER_DIRECTION_BIT: u16 = 0b0010_0000;

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
    count: i16,
    range: &str,
) -> AsmResult<'a, 'a, InstructionToken> {
    let error_string = format!(
        "The [{tag}] meta instruction only supports counts in the range {range}, got {count}"
    );
    Err(nom::Err::Failure(ErrorTree::from_external_error(
        input,
        ErrorKind::Fail,
        error_string.as_str(),
    )))
}

fn reject_dma_address_register<'a>(
    input: &'a str,
    tag: &str,
    address_register: &AddressRegisterName,
) -> AsmResult<'a, 'a, InstructionToken> {
    let error_string = format!(
        "The [{tag}] meta instruction only supports address registers a, l, or s; got {address_register:?}"
    );
    Err(nom::Err::Failure(ErrorTree::from_external_error(
        input,
        ErrorKind::Fail,
        error_string.as_str(),
    )))
}

fn reject_dmat_register_pair(input: &str) -> AsmResult<'_, '_, InstructionToken> {
    Err(nom::Err::Failure(ErrorTree::from_external_error(
        input,
        ErrorKind::Fail,
        "The [DMAT] meta instruction only supports transfers from a to l",
    )))
}

fn dma_address_register_operand(
    address_register: &AddressRegisterName,
) -> Result<u16, AddressRegisterName> {
    match address_register {
        AddressRegisterName::Address => Ok(0b00 << 6),
        AddressRegisterName::LinkRegister => Ok(0b01 << 6),
        AddressRegisterName::StackPointer => Ok(0b10 << 6),
        AddressRegisterName::ProgramCounter => Err(AddressRegisterName::ProgramCounter),
    }
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

    let parse_dma_register_count =
        |immediate_type: &ImmediateType| -> Result<u16, nom::Err<ErrorTree<&str>>> {
            let raw_count = parse_count(immediate_type)?;
            let signed_count = raw_count.cast_signed();
            if !(-7..=7).contains(&signed_count) {
                let error_string = format!(
                    "The [{tag}] meta instruction only supports counts in the range -7..7, got {signed_count}"
                );
                return Err(nom::Err::Failure(ErrorTree::from_external_error(
                    i_after_instruction,
                    ErrorKind::Fail,
                    error_string.as_str(),
                )));
            }

            let magnitude = signed_count.unsigned_abs();
            let direction = if signed_count < 0 {
                DMA_REGISTER_DIRECTION_BIT
            } else {
                0
            };

            Ok(direction | magnitude)
        };

    match (tag.as_str(), operands.as_slice()) {
        (
            "DMAR" | "DMAW",
            [
                AddressingMode::DirectAddressRegister(address_register),
                AddressingMode::Immediate(immediate_type),
            ],
        ) => {
          let Ok(address_operand) = dma_address_register_operand(address_register) else {
                return reject_dma_address_register(
                    i_after_instruction,
                    tag.as_str(),
                    address_register,
                );
            };
            let count_operand = parse_dma_register_count(immediate_type)?;

            let operation = if tag == "DMAR" { 0x8 } else { 0x9 };
            Ok((i, construct_instruction(command(
                DMA_COPROCESSOR_ID,
                operation,
                address_operand | count_operand,
            ))))
        }
        ("DMAR" | "DMAW", [AddressingMode::Immediate(_)]) => {
            let error_string = format!(
                "The [{tag}] meta instruction requires an explicit address register operand (a, l, or s)"
            );
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i_after_instruction,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
        (
            "DMAT",
            [
                AddressingMode::DirectAddressRegister(source_register),
                AddressingMode::DirectAddressRegister(dest_register),
                AddressingMode::Immediate(immediate_type),
            ],
        ) => {
            if source_register != &AddressRegisterName::Address
                || dest_register != &AddressRegisterName::LinkRegister
            {
                return reject_dmat_register_pair(i_after_instruction);
            }

            let count = parse_count(immediate_type)?;
            if count > 0xFF {
                return reject_count(i_after_instruction, tag.as_str(), count.cast_signed(), "0-255");
            }

            Ok((
                i,
                construct_instruction(command(DMA_COPROCESSOR_ID, 0xA, count)),
            ))
        }
        ("DMAT", [AddressingMode::DirectAddressRegister(_), AddressingMode::DirectAddressRegister(_), _]) => {
            reject_symbol_operand(i_after_instruction, tag.as_str())
        }
        ("DMAT", [AddressingMode::Immediate(_)]) => {
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i_after_instruction,
                ErrorKind::Fail,
                "The [DMAT] meta instruction requires explicit source and destination address registers: DMAT a, l, #n",
            )))
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
