use crate::{
    parsers::{
        instruction::{
            override_ref_token_type_if_implied, parse_instruction_operands1, parse_instruction_tag,
            AddressingMode, ImmediateType, InstructionToken,
        },
        shared::split_shift_definition_data,
    },
    types::object::RefType,
};
use nom::{branch::alt, error::ErrorKind};
use nom::{error::FromExternalError, sequence::tuple};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::{
    instructions::definitions::{
        ImmediateInstructionData, Instruction, InstructionData, ShortImmediateInstructionData,
    },
    registers::AddressRegisterName,
};

fn tag_to_instruction_long(tag: &str) -> Instruction {
    match tag {
        "BRAN" => Instruction::BranchImmediate,
        "BRSR" => Instruction::BranchToSubroutineImmediate,
        "SJMP" => Instruction::ShortJumpImmediate,
        "SJSR" => Instruction::ShortJumpToSubroutineImmediate,
        _ => panic!("No tag mapping for instruction [{tag}]"),
    }
}

fn tag_to_instruction_short(tag: &str) -> Instruction {
    match tag {
        "BRAN" => Instruction::BranchShortImmediate,
        "BRSR" => Instruction::BranchToSubroutineShortImmediate,
        "SJMP" => Instruction::ShortJumpShortImmediate,
        "SJSR" => Instruction::ShortJumpToSubroutineShortImmediate,
        _ => panic!("No tag mapping for instruction [{tag}]"),
    }
}

use super::super::shared::AsmResult;
pub fn branching(i: &str) -> AsmResult<InstructionToken> {
    let instructions = alt((
        parse_instruction_tag("BRAN"),
        parse_instruction_tag("BRSR"),
        parse_instruction_tag("SJMP"),
        parse_instruction_tag("SJSR"),
    ));

    let (i, ((tag, condition_flag), operands)) =
        tuple((instructions, parse_instruction_operands1))(i)?;

    match operands.as_slice() {
        [AddressingMode::Immediate(offset)] => match offset {
            ImmediateType::Value(offset) => Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Immediate(ImmediateInstructionData {
                        op_code: tag_to_instruction_long(tag.as_str()),
                        register: 0x0, // unused
                        value: offset.to_owned(),
                        condition_flag,
                        additional_flags: AddressRegisterName::ProgramCounter.to_register_index(),
                    }),
                    symbol_ref: None,
                },
            )),
            ImmediateType::SymbolRef(ref_token) => Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Immediate(ImmediateInstructionData {
                        op_code: tag_to_instruction_long(tag.as_str()),
                        register: 0x0, // unused
                        value: 0x0,    // Placeholder
                        condition_flag,
                        additional_flags: AddressRegisterName::ProgramCounter.to_register_index(),
                    }),
                    symbol_ref: Some(override_ref_token_type_if_implied(
                        ref_token,
                        RefType::Offset,
                    )),
                },
            )),
        },
        [AddressingMode::Immediate(offset), AddressingMode::ShiftDefinition(shift_definition_data)] =>
        {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            match offset {
                ImmediateType::Value(offset) => {
                    if offset > &0xFF {
                        let error_string = format!("Immediate values can only be up to 8 bits when using a shift definition ({offset} > 0xFF)");
                        Err(nom::Err::Failure(ErrorTree::from_external_error(
                            i,
                            ErrorKind::Fail,
                            error_string.as_str(),
                        )))
                    } else {
                        Ok((
                            i,
                            InstructionToken {
                                instruction: InstructionData::ShortImmediate(
                                    ShortImmediateInstructionData {
                                        op_code: tag_to_instruction_short(tag.as_str()),
                                        register: 0x0, // unused
                                        value: offset.to_owned().try_into().expect(
                                            "Offset should fit into 0xFF as it checked above",
                                        ),
                                        shift_operand,
                                        shift_type,
                                        shift_count,
                                        condition_flag,
                                        additional_flags: AddressRegisterName::ProgramCounter
                                            .to_register_index(),
                                    },
                                ),
                                symbol_ref: None,
                            },
                        ))
                    }
                }
                ImmediateType::SymbolRef(ref_token) => Ok((
                    i,
                    InstructionToken {
                        instruction: InstructionData::ShortImmediate(
                            ShortImmediateInstructionData {
                                op_code: tag_to_instruction_short(tag.as_str()),
                                register: 0x0, // unused
                                value: 0x0,    // Placeholder
                                shift_operand,
                                shift_type,
                                shift_count,
                                condition_flag,
                                additional_flags: AddressRegisterName::ProgramCounter
                                    .to_register_index(),
                            },
                        ),
                        symbol_ref: Some(override_ref_token_type_if_implied(
                            ref_token,
                            RefType::Offset,
                        )),
                    },
                )),
            }
        }
        _ => {
            let error_string = format!(
                "The [{tag}] opcode only supports immediate addressing mode (e.g. BRAN #-3)"
            );
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}
