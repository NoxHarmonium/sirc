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
        ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData,
        ShiftOperand, ShiftType,
    },
    registers::{AddressRegisterName, RegisterName},
};

fn tag_to_instruction_long_immediate(tag: &str) -> Instruction {
    match tag {
        "BRAN" => Instruction::BranchWithImmediateDisplacement,
        "BRSR" => Instruction::BranchToSubroutineWithImmediateDisplacement,
        _ => panic!("No tag mapping for instruction [{tag}]"),
    }
}

fn tag_to_instruction_long_register(tag: &str) -> Instruction {
    match tag {
        "BRAN" => Instruction::BranchWithRegisterDisplacement,
        "BRSR" => Instruction::BranchToSubroutineWithRegisterDisplacement,
        _ => panic!("No tag mapping for instruction [{tag}]"),
    }
}

use super::super::shared::AsmResult;
pub fn branching(i: &str) -> AsmResult<InstructionToken> {
    let instructions = alt((parse_instruction_tag("BRAN"), parse_instruction_tag("BRSR")));

    let (i, ((tag, condition_flag), operands)) =
        tuple((instructions, parse_instruction_operands1))(i)?;

    match operands.as_slice() {
        [AddressingMode::Immediate(offset)] => match offset {
            // Shorthand, always immediate offset to PC. Can also use other address registers with full syntax below
            ImmediateType::Value(offset) => Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Immediate(ImmediateInstructionData {
                        op_code: tag_to_instruction_long_immediate(tag.as_str()),
                        register: RegisterName::Pl.to_register_index(),
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
                        op_code: tag_to_instruction_long_immediate(tag.as_str()),
                        register: RegisterName::Pl.to_register_index(),
                        value: 0x0, // Placeholder
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
        [AddressingMode::IndirectImmediateDisplacement(offset, address_register)] => {
            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        instruction: InstructionData::Immediate(ImmediateInstructionData {
                            op_code: tag_to_instruction_long_immediate(tag.as_str()),
                            register: RegisterName::Pl.to_register_index(),
                            value: offset.to_owned(),
                            condition_flag,
                            additional_flags: address_register.to_register_index(),
                        }),
                        symbol_ref: None,
                    },
                )),
                ImmediateType::SymbolRef(ref_token) => Ok((
                    i,
                    InstructionToken {
                        instruction: InstructionData::Immediate(ImmediateInstructionData {
                            op_code: tag_to_instruction_long_immediate(tag.as_str()),
                            register: RegisterName::Pl.to_register_index(),
                            value: 0x0, // Placeholder
                            condition_flag,
                            additional_flags: address_register.to_register_index(),
                        }),
                        symbol_ref: Some(override_ref_token_type_if_implied(
                            ref_token,
                            RefType::LowerWord,
                        )),
                    },
                )),
            }
        }
        [AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register)] => {
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: tag_to_instruction_long_register(tag.as_str()),
                        r1: RegisterName::Pl.to_register_index(),
                        r2: displacement_register.to_register_index(),
                        r3: 0x0, // Unused
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                        condition_flag,
                        // TODO: Clamp/validate additional_flags to 10 bits
                        additional_flags: address_register.to_register_index(),
                    }),
                    symbol_ref: None,
                },
            ))
        }
        [AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register), AddressingMode::ShiftDefinition(shift_definition_data)] =>
        {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: tag_to_instruction_long_register(tag.as_str()),
                        r1: RegisterName::Pl.to_register_index(),
                        r2: displacement_register.to_register_index(),
                        r3: 0x0, // Unused
                        shift_operand,
                        shift_type,
                        shift_count,
                        condition_flag,
                        // TODO: Clamp/validate additional_flags to 10 bits
                        additional_flags: address_register.to_register_index(),
                    }),
                    symbol_ref: None,
                },
            ))
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
