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
use nom::error::FromExternalError;
use nom::{error::ErrorKind, sequence::tuple};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData,
        ShiftOperand, ShiftType,
    },
    registers::{AddressRegisterName, RegisterName},
};

use super::super::shared::AsmResult;
pub fn stor(i: &str) -> AsmResult<InstructionToken> {
    let (i, ((_, condition_flag), operands)) =
        tuple((parse_instruction_tag("STOR"), parse_instruction_operands1))(i)?;

    let construct_indirect_immediate_instruction =
        |offset: u16, src_register: &RegisterName, address_register: &AddressRegisterName| {
            InstructionData::Immediate(ImmediateInstructionData {
                op_code: Instruction::StoreRegisterToIndirectImmediate,
                register: src_register.to_register_index(),
                value: offset,
                condition_flag,
                additional_flags: address_register.to_register_index(),
            })
        };

    let construct_indirect_immediate_pre_increment_instruction =
        |offset: u16, src_register: &RegisterName, address_register: &AddressRegisterName| {
            InstructionData::Immediate(ImmediateInstructionData {
                op_code: Instruction::StoreRegisterToIndirectImmediatePreDecrement,
                register: src_register.to_register_index(),
                value: offset,
                condition_flag,
                additional_flags: address_register.to_register_index(),
            })
        };

    match operands.as_slice() {
        [AddressingMode::IndirectImmediateDisplacement(offset, address_register), AddressingMode::DirectRegister(src_register)] => {
            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        instruction: construct_indirect_immediate_instruction(
                            offset.to_owned(),
                            src_register,
                            address_register,
                        ),
                        ..Default::default()
                    },
                )),
                ImmediateType::SymbolRef(ref_token) => Ok((
                    i,
                    InstructionToken {
                        instruction: construct_indirect_immediate_instruction(
                            0x0,
                            src_register,
                            address_register,
                        ),
                        symbol_ref: Some(override_ref_token_type_if_implied(
                            ref_token,
                            RefType::LowerWord,
                        )),
                        ..Default::default()
                    },
                )),
                ImmediateType::PlaceHolder(placeholder_name) => Ok((
                    i,
                    InstructionToken {
                        instruction: construct_indirect_immediate_instruction(
                            0x0,
                            src_register,
                            address_register,
                        ),
                        placeholder_name: Some(placeholder_name.clone()),
                        ..Default::default()
                    },
                )),
            }
        }
        [AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register), AddressingMode::DirectRegister(src_register)] =>
        {
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::StoreRegisterToIndirectRegister,
                        r1: 0x0, // unused
                        r2: src_register.to_register_index(),
                        r3: displacement_register.to_register_index(),
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                        condition_flag,
                        additional_flags: address_register.to_register_index(),
                    }),
                    ..Default::default()
                },
            ))
        }
        [AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register), AddressingMode::DirectRegister(src_register), AddressingMode::ShiftDefinition(shift_definition_data)] =>
        {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::StoreRegisterToIndirectRegister,
                        r1: 0x0, // unused
                        r2: src_register.to_register_index(),
                        r3: displacement_register.to_register_index(),
                        shift_operand,
                        shift_type,
                        shift_count,
                        condition_flag,
                        additional_flags: address_register.to_register_index(),
                    }),
                    ..Default::default()
                },
            ))
        }
        [AddressingMode::IndirectImmediateDisplacementPreDecrement(offset, address_register), AddressingMode::DirectRegister(src_register)] => {
            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        instruction: construct_indirect_immediate_pre_increment_instruction(
                            offset.to_owned(),
                            src_register,
                            address_register,
                        ),
                        ..Default::default()
                    },
                )),
                ImmediateType::SymbolRef(ref_token) => Ok((
                    i,
                    InstructionToken {
                        instruction: construct_indirect_immediate_pre_increment_instruction(
                            0x0,
                            src_register,
                            address_register,
                        ),
                        symbol_ref: Some(override_ref_token_type_if_implied(
                            ref_token,
                            RefType::LowerWord,
                        )),
                        ..Default::default()
                    },
                )),
                ImmediateType::PlaceHolder(placeholder_name) => Ok((
                    i,
                    InstructionToken {
                        instruction: construct_indirect_immediate_pre_increment_instruction(
                            0x0,
                            src_register,
                            address_register,
                        ),
                        placeholder_name: Some(placeholder_name.clone()),
                        ..Default::default()
                    },
                )),
            }
        }
        [AddressingMode::IndirectRegisterDisplacementPreDecrement(
            displacement_register,
            address_register,
        ), AddressingMode::DirectRegister(src_register)] => {
            Ok((i, {
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::StoreRegisterToIndirectRegisterPreDecrement,
                        r1: 0x0, //Unused
                        r2: src_register.to_register_index(),
                        r3: displacement_register.to_register_index(), //Unused
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                        condition_flag,
                        additional_flags: address_register.to_register_index(),
                    }),
                    ..Default::default()
                }
            }))
        }
        [AddressingMode::IndirectRegisterDisplacementPreDecrement(
            displacement_register,
            address_register,
        ), AddressingMode::DirectRegister(src_register), AddressingMode::ShiftDefinition(shift_definition_data)] =>
        {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            Ok((i, {
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::StoreRegisterToIndirectRegisterPreDecrement,
                        r1: 0x0, //Unused
                        r2: src_register.to_register_index(),
                        r3: displacement_register.to_register_index(), //Unused
                        shift_operand,
                        shift_type,
                        shift_count,
                        condition_flag,
                        additional_flags: address_register.to_register_index(),
                    }),
                    ..Default::default()
                }
            }))
        }
        // TODO: Better error message without being too verbose?
        modes => {
            let error_string = format!("Invalid addressing mode for STOR: ({modes:?})");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}
