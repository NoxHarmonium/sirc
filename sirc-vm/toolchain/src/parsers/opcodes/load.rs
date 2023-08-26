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
use nom::{
    error::{ErrorKind, FromExternalError},
    sequence::tuple,
};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData,
        ShiftOperand, ShiftType,
    },
    registers::{AddressRegisterName, RegisterName},
};

use super::super::shared::AsmResult;
pub fn load(i: &str) -> AsmResult<InstructionToken> {
    let (i, ((_, condition_flag), operands)) =
        tuple((parse_instruction_tag("LOAD"), parse_instruction_operands1))(i)?;

    let construct_immediate_instruction = |value: u16, dest_register: &RegisterName| {
        InstructionData::Immediate(ImmediateInstructionData {
            op_code: Instruction::LoadRegisterFromImmediate,
            register: dest_register.to_register_index(),
            value: value.to_owned(),
            condition_flag,
            additional_flags: 0x0,
        })
    };

    let construct_indirect_immediate_instruction =
        |offset: u16, dest_register: &RegisterName, address_register: &AddressRegisterName| {
            InstructionData::Immediate(ImmediateInstructionData {
                op_code: Instruction::LoadRegisterFromIndirectImmediate,
                register: dest_register.to_register_index(),
                value: offset.to_owned(),
                condition_flag,
                additional_flags: address_register.to_register_index(),
            })
        };

    let construct_indirect_immediate_post_increment_instruction =
        |offset: u16, dest_register: &RegisterName, address_register: &AddressRegisterName| {
            InstructionData::Immediate(ImmediateInstructionData {
                op_code: Instruction::LoadRegisterFromIndirectImmediatePostIncrement,
                register: dest_register.to_register_index(),
                value: offset,
                condition_flag,
                additional_flags: address_register.to_register_index(),
            })
        };

    match operands.as_slice() {
        [AddressingMode::DirectRegister(dest_register), AddressingMode::Immediate(offset)] => {
            match offset {
                ImmediateType::Value(value) => Ok((
                    i,
                    InstructionToken {
                        instruction: construct_immediate_instruction(
                            value.to_owned(),
                            dest_register,
                        ),
                        ..Default::default()
                    },
                )),
                ImmediateType::SymbolRef(ref_token) => Ok((
                    i,
                    InstructionToken {
                        instruction: construct_immediate_instruction(0x0, dest_register),
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
                        instruction: construct_immediate_instruction(0x0, dest_register),
                        placeholder_name: Some(placeholder_name.clone()),
                        ..Default::default()
                    },
                )),
            }
        }
        [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register)] =>
        {
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromRegister,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: src_register.to_register_index(),
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                        condition_flag,
                        additional_flags: 0x00,
                    }),
                    ..Default::default()
                },
            ))
        }
        [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectImmediateDisplacement(offset, address_register)] => {
            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        instruction: construct_indirect_immediate_instruction(
                            offset.to_owned(),
                            dest_register,
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
                            dest_register,
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
                            dest_register,
                            address_register,
                        ),
                        placeholder_name: Some(placeholder_name.clone()),
                        ..Default::default()
                    },
                )),
            }
        }
        [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register)] =>
        {
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromIndirectRegister,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: displacement_register.to_register_index(),
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                        condition_flag,
                        // TODO: Clamp/validate additional_flags to 10 bits
                        additional_flags: address_register.to_register_index(),
                    }),
                    ..Default::default()
                },
            ))
        }
        [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register), AddressingMode::ShiftDefinition(shift_definition_data)] =>
        {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromIndirectRegister,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: displacement_register.to_register_index(),
                        shift_operand,
                        shift_type,
                        shift_count,
                        condition_flag,
                        // TODO: Clamp/validate additional_flags to 10 bits
                        additional_flags: address_register.to_register_index(),
                    }),
                    ..Default::default()
                },
            ))
        }
        [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectRegisterDisplacementPostIncrement(
            displacement_register,
            address_register,
        )] => {
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromIndirectRegisterPostIncrement,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: displacement_register.to_register_index(),
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                        condition_flag,
                        // TODO: Clamp/validate additional_flags to 10 bits
                        additional_flags: address_register.to_register_index(),
                    }),
                    ..Default::default()
                },
            ))
        }
        [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectRegisterDisplacementPostIncrement(
            displacement_register,
            address_register,
        ), AddressingMode::ShiftDefinition(shift_definition_data)] => {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromIndirectRegisterPostIncrement,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: displacement_register.to_register_index(),
                        shift_operand,
                        shift_type,
                        shift_count,
                        condition_flag,
                        // TODO: Clamp/validate additional_flags to 10 bits
                        additional_flags: address_register.to_register_index(),
                    }),
                    ..Default::default()
                },
            ))
        }
        [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectImmediateDisplacementPostIncrement(offset, address_register)] => {
            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        instruction: construct_indirect_immediate_post_increment_instruction(
                            offset.to_owned(),
                            dest_register,
                            address_register,
                        ),
                        ..Default::default()
                    },
                )),
                ImmediateType::SymbolRef(ref_token) => Ok((
                    i,
                    InstructionToken {
                        instruction: construct_indirect_immediate_post_increment_instruction(
                            0x0,
                            dest_register,
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
                        instruction: construct_indirect_immediate_post_increment_instruction(
                            0x0,
                            dest_register,
                            address_register,
                        ),
                        placeholder_name: Some(placeholder_name.clone()),
                        ..Default::default()
                    },
                )),
            }
        }
        modes => {
            let error_string = format!("Invalid addressing mode for LOAD: ({modes:?})");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}

//
