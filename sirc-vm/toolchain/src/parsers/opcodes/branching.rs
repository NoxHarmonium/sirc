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
use nom::{branch::alt, combinator::map_res, error::ErrorKind};
use nom::{error::FromExternalError, sequence::tuple};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, Instruction, InstructionData, ShortImmediateInstructionData,
};

fn tag_to_instruction_long(tag: String) -> Instruction {
    match tag.as_str() {
        "BRAN" => Instruction::BranchImmediate,
        "BRSR" => Instruction::BranchToSubroutineImmediate,
        "SJMP" => Instruction::ShortJumpImmediate,
        "SJSR" => Instruction::ShortJumpToSubroutineImmediate,
        _ => panic!("No tag mapping for instruction [{}]", tag),
    }
}

fn tag_to_instruction_short(tag: String) -> Instruction {
    match tag.as_str() {
        "BRAN" => Instruction::BranchShortImmediate,
        "BRSR" => Instruction::BranchToSubroutineShortImmediate,
        "SJMP" => Instruction::ShortJumpShortImmediate,
        "SJSR" => Instruction::ShortJumpToSubroutineShortImmediate,
        _ => panic!("No tag mapping for instruction [{}]", tag),
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

    map_res(
        tuple((instructions, parse_instruction_operands1)),
        |((tag, condition_flag), operands)| match operands.as_slice() {
            [AddressingMode::Immediate(offset)] => match offset {
                ImmediateType::Value(offset) => Ok(InstructionToken {
                    instruction: InstructionData::Immediate(ImmediateInstructionData {
                        op_code: tag_to_instruction_long(tag),
                        register: 0x0, // unused
                        value: offset.to_owned(),
                        condition_flag,
                        additional_flags: 0x0,
                    }),
                    symbol_ref: None,
                }),
                ImmediateType::SymbolRef(ref_token) => Ok(InstructionToken {
                    instruction: InstructionData::Immediate(ImmediateInstructionData {
                        op_code: tag_to_instruction_long(tag),
                        register: 0x0, // unused
                        value: 0x0,    // Placeholder
                        condition_flag,
                        additional_flags: 0x0,
                    }),
                    symbol_ref: Some(override_ref_token_type_if_implied(
                        &ref_token,
                        RefType::Offset,
                    )),
                }),
            },
            [AddressingMode::Immediate(offset), AddressingMode::ShiftDefinition(shift_definition_data)] =>
            {
                let (shift_operand, shift_type, shift_count) =
                    split_shift_definition_data(shift_definition_data);
                match offset {
                    ImmediateType::Value(offset) => {
                        if offset > &0xFF {
                            let error_string = format!("Immediate values can only be up to 8 bits when using a shift definition ({} > 0xFF)", offset);
                            Err(nom::Err::Failure(ErrorTree::from_external_error(
                                i.to_owned(),
                                ErrorKind::Fail,
                                error_string.as_str(),
                            )))
                        } else {
                            Ok(InstructionToken {
                                instruction: InstructionData::ShortImmediate(
                                    ShortImmediateInstructionData {
                                        op_code: tag_to_instruction_short(tag),
                                        register: 0x0, // unused
                                        value: offset.to_owned() as u8,
                                        shift_operand,
                                        shift_type,
                                        shift_count,
                                        condition_flag,
                                        additional_flags: 0x0,
                                    },
                                ),
                                symbol_ref: None,
                            })
                        }
                    }
                    ImmediateType::SymbolRef(ref_token) => Ok(InstructionToken {
                        instruction: InstructionData::ShortImmediate(
                            ShortImmediateInstructionData {
                                op_code: tag_to_instruction_short(tag),
                                register: 0x0, // unused
                                value: 0x0,    // Placeholder
                                shift_operand,
                                shift_type,
                                shift_count,
                                condition_flag,
                                additional_flags: 0x0,
                            },
                        ),
                        symbol_ref: Some(override_ref_token_type_if_implied(
                            &ref_token,
                            RefType::Offset,
                        )),
                    }),
                }
            }
            _ => {
                let error_string = format!(
                    "The [{}] opcode only supports immediate addressing mode (e.g. BRAN #-3)",
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
