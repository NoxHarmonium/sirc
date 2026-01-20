use crate::parsers::instruction::{
    parse_instruction_operands1, parse_instruction_tag, AddressingMode, ImmediateType,
};
use crate::parsers::shared::split_shift_definition_data;
use crate::types::instruction::InstructionToken;
use nom::branch::alt;
use nom::error::{ErrorKind, FromExternalError};
use nom::sequence::tuple;
use nom_supreme::error::ErrorTree;
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ImmediateInstructionData, Instruction, InstructionData, ShiftOperand, ShiftType,
    ShortImmediateInstructionData, StatusRegisterUpdateSource,
};
use peripheral_cpu::registers::RegisterName;

fn tag_to_instruction_long(tag: &String) -> Instruction {
    match tag.as_str() {
        // SHFI is a "meta instruction" i.e. it does not have its own opcode - It is just AddImmediate but status register is updated from shift not ALU
        "ADDI" | "SHFI" => Instruction::AddImmediate,
        "ADCI" => Instruction::AddImmediateWithCarry,
        "SUBI" => Instruction::SubtractImmediate,
        "SBCI" => Instruction::SubtractImmediateWithCarry,
        "ANDI" => Instruction::AndImmediate,
        "ORRI" => Instruction::OrImmediate,
        "XORI" => Instruction::XorImmediate,
        "CMPI" => Instruction::CompareImmediate,
        "TSAI" => Instruction::TestAndImmediate,
        "TSXI" => Instruction::TestXorImmediate,
        "COPI" => Instruction::CoprocessorCallImmediate,

        _ => panic!("No tag mapping for instruction [{tag}]"),
    }
}

fn tag_to_instruction_short(tag: &String) -> Instruction {
    match tag.as_str() {
        // SHFI is a "meta instruction" i.e. it does not have its own opcode - It is just AddShortImmediate but status register is updated from shift not ALU
        "ADDI" | "SHFI" => Instruction::AddShortImmediate,
        "ADCI" => Instruction::AddShortImmediateWithCarry,
        "SUBI" => Instruction::SubtractShortImmediate,
        "SBCI" => Instruction::SubtractShortImmediateWithCarry,
        "ANDI" => Instruction::AndShortImmediate,
        "ORRI" => Instruction::OrShortImmediate,
        "XORI" => Instruction::XorShortImmediate,
        "CMPI" => Instruction::CompareShortImmediate,
        "TSAI" => Instruction::TestAndShortImmediate,
        "TSXI" => Instruction::TestXorShortImmediate,
        "COPI" => Instruction::CoprocessorCallShortImmediate,

        _ => panic!("No tag mapping for instruction [{tag}]"),
    }
}

use super::super::shared::AsmResult;

///
/// Parses immediate arithmetic/logic opcodes
///
/// ```
/// use toolchain::parsers::opcodes::arithmetic_immediate::arithmetic_immediate;
/// use toolchain::types::instruction::InstructionToken;
/// use peripheral_cpu::coprocessors::processing_unit::definitions::{ConditionFlags, Instruction, InstructionData, ImmediateInstructionData, ShiftType};
///
///
/// let (_, parsed_instruction) = arithmetic_immediate("ADDI|!= r2, #123, LSL #2\n").unwrap();
/// let (op_code, register, value, shift_type, shift_count, condition_flag, additional_flags) = match parsed_instruction.instruction {
///     InstructionData::ShortImmediate(inner) => (inner.op_code, inner.register, inner.value, inner.shift_type, inner.shift_count, inner.condition_flag, inner.additional_flags),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// assert_eq!(op_code, Instruction::AddShortImmediate);
/// assert_eq!(register, 2);
/// assert_eq!(value, 123);
/// assert_eq!(shift_type, ShiftType::LogicalLeftShift);
/// assert_eq!(shift_count, 0x2);
/// assert_eq!(additional_flags, 0x1);
/// assert_eq!(condition_flag, ConditionFlags::NotEqual);
///
/// ```
pub fn arithmetic_immediate(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();
    let instructions = alt((
        parse_instruction_tag("ADDI"),
        parse_instruction_tag("ADCI"),
        parse_instruction_tag("SUBI"),
        parse_instruction_tag("SBCI"),
        parse_instruction_tag("ANDI"),
        parse_instruction_tag("ORRI"),
        parse_instruction_tag("XORI"),
        parse_instruction_tag("CMPI"),
        parse_instruction_tag("TSAI"),
        parse_instruction_tag("TSXI"),
        parse_instruction_tag("COPI"),
        // Meta instruction
        parse_instruction_tag("SHFI"),
    ));

    let (i, ((tag, condition_flag), operands)) =
        tuple((instructions, parse_instruction_operands1))(i)?;

    let construct_immediate_instruction = |value: u16, dest_register: &RegisterName| {
        InstructionData::Immediate(ImmediateInstructionData {
            op_code: tag_to_instruction_long(&tag),
            register: dest_register.to_register_index(),
            value,
            condition_flag,
            additional_flags: if &tag == "SHFI" {
                StatusRegisterUpdateSource::Shift.to_flags()
            } else {
                StatusRegisterUpdateSource::Alu.to_flags()
            },
        })
    };

    let construct_short_immediate_instruction =
        |value: u8,
         dest_register: &RegisterName,
         shift_operand: ShiftOperand,
         shift_type: ShiftType,
         shift_count: u8| {
            InstructionData::ShortImmediate(ShortImmediateInstructionData {
                op_code: tag_to_instruction_short(&tag),
                register: dest_register.to_register_index(),
                value,
                condition_flag,
                additional_flags: if &tag == "SHFI" {
                    StatusRegisterUpdateSource::Shift.to_flags()
                } else {
                    StatusRegisterUpdateSource::Alu.to_flags()
                },
                shift_operand,
                shift_type,
                shift_count,
            })
        };

    match operands.as_slice() {
        [AddressingMode::DirectRegister(dest_register), AddressingMode::Immediate(immediate_type)] => {
            match immediate_type {
                ImmediateType::Value(value) => Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: construct_immediate_instruction(
                            value.to_owned(),
                            dest_register,
                        ),
                        ..Default::default()
                    },
                )),
                ImmediateType::PlaceHolder(placeholder_name) => Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: construct_immediate_instruction(0x0, dest_register),
                        placeholder_name: Some(placeholder_name.clone()),
                        ..Default::default()
                    },
                )),
                ImmediateType::SymbolRef(_) => {
                    let error_string =
                        format!("The [{tag}] opcode does not support symbol refs at this time");
                    Err(nom::Err::Failure(ErrorTree::from_external_error(
                        i,
                        ErrorKind::Fail,
                        error_string.as_str(),
                    )))
                }
            }
        }
        [AddressingMode::DirectRegister(dest_register), AddressingMode::Immediate(immediate_type), AddressingMode::ShiftDefinition(shift_definition_data)] =>
        {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            match immediate_type {
                ImmediateType::Value(value) => {
                    let short_value: Result<u8, _> = (*value).try_into();
                    short_value.map_or_else(|_| {
                          let error_string = format!("Immediate values must fit into 8 bits when using a shift definition ({value} > 0xFF)");
                        Err(nom::Err::Failure(ErrorTree::from_external_error(
                            i,
                            ErrorKind::Fail,
                            error_string.as_str(),
                        )))
                    },|short_value| {
                        Ok((
                            i,
                            InstructionToken {
                                input_length,
                                instruction: construct_short_immediate_instruction(
                                    short_value,
                                    dest_register,
                                    shift_operand,
                                    shift_type,
                                    shift_count,
                                ),
                                symbol_ref: None,
                                ..Default::default()
                            },
                        ))
                    })
                }
                ImmediateType::PlaceHolder(placeholder_name) => Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: construct_short_immediate_instruction(
                            0x0,
                            dest_register,
                            shift_operand,
                            shift_type,
                            shift_count,
                        ),
                        placeholder_name: Some(placeholder_name.clone()),
                        ..Default::default()
                    },
                )),
                ImmediateType::SymbolRef(_) => {
                    let error_string =
                        format!("The [{tag}] opcode does not support symbol refs at this time");
                    Err(nom::Err::Failure(ErrorTree::from_external_error(
                        i,
                        ErrorKind::Fail,
                        error_string.as_str(),
                    )))
                }
            }
        }
        _ => {
            let error_string = format!("The [{tag}] opcode only supports immediate->register addressing mode (e.g. ADDI y1, #1)");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}
