use super::super::shared::AsmResult;
use crate::parsers::shared::split_shift_definition_data;
use crate::types::instruction::InstructionToken;
use crate::{
    parsers::{
        data::override_ref_token_type_if_implied,
        instruction::{
            AddressingMode, ImmediateType, parse_instruction_operands0, parse_instruction_tag,
        },
    },
    types::object::RefType,
};
use nom::error::{ErrorKind, FromExternalError};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData,
        ShiftOperand, ShiftType,
    },
    registers::{AddressRegisterName, RegisterName},
};
pub fn load(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();

    let (i_after_instruction, (_, condition_flag, status_register_update_source)) =
        parse_instruction_tag("LOAD")(i)?;

    if status_register_update_source.is_some() {
        let error_string = "The [LOAD] opcode does not support an explicit status register update source. Only ALU instructions can update the status register as a side-effect.";
        return Err(nom::Err::Failure(ErrorTree::from_external_error(
            i_after_instruction,
            ErrorKind::Fail,
            error_string,
        )));
    }

    let (i, operands) = parse_instruction_operands0(i_after_instruction)?;

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

    // NOTE: No shifting with immediate operands because there is no short immediate representation of LOAD
    match operands.as_slice() {
        // LOAD r1, #0
        [
            AddressingMode::DirectRegister(dest_register),
            AddressingMode::Immediate(offset),
        ] => match offset {
            ImmediateType::Value(value) => Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: construct_immediate_instruction(value.to_owned(), dest_register),
                    ..Default::default()
                },
            )),
            ImmediateType::SymbolRef(ref_token) => Ok((
                i,
                InstructionToken {
                    input_length,
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
                    input_length,
                    instruction: construct_immediate_instruction(0x0, dest_register),
                    placeholder_name: Some(placeholder_name.clone()),
                    ..Default::default()
                },
            )),
        },
        // LOAD r1, r2
        [
            AddressingMode::DirectRegister(dest_register),
            AddressingMode::DirectRegister(src_register),
        ] => {
            Ok((
                i,
                InstructionToken {
                    input_length,
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
        // LOAD r1, r2, ASL #1
        [
            AddressingMode::DirectRegister(_),
            AddressingMode::DirectRegister(_),
            AddressingMode::ShiftDefinition(_),
        ] => {
            // NOTE: No shifting with direct register -> direct register because of the way the CPU architecture works - use SHFT instead
            // LOAD r1, r2, ASL #1 would translate directly to SHFT r1, r2, ASL #1.
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i_after_instruction,
                ErrorKind::Fail,
                "Invalid addressing mode for LOAD:. Cannot use a shift with direct register -> direct register loads. Use SHFT instruction instead.",
            )))
        }
        // LOAD r1, (#0, a)
        [
            AddressingMode::DirectRegister(dest_register),
            AddressingMode::IndirectImmediateDisplacement(offset, address_register),
        ] => match offset {
            ImmediateType::Value(offset) => Ok((
                i,
                InstructionToken {
                    input_length,
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
                    input_length,
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
                    input_length,
                    instruction: construct_indirect_immediate_instruction(
                        0x0,
                        dest_register,
                        address_register,
                    ),
                    placeholder_name: Some(placeholder_name.clone()),
                    ..Default::default()
                },
            )),
        },
        // LOAD r1, (r2, a)
        [
            AddressingMode::DirectRegister(dest_register),
            AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register),
        ] => {
            Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromIndirectRegister,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
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
        // LOAD r1, (r2, a), ASL #1
        [
            AddressingMode::DirectRegister(dest_register),
            AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register),
            AddressingMode::ShiftDefinition(shift_definition_data),
        ] => {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromIndirectRegister,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
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
        // LOAD r1, (#0, a)+
        [
            AddressingMode::DirectRegister(dest_register),
            AddressingMode::IndirectImmediateDisplacementPostIncrement(offset, address_register),
        ] => match offset {
            ImmediateType::Value(offset) => Ok((
                i,
                InstructionToken {
                    input_length,
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
                    input_length,
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
                    input_length,
                    instruction: construct_indirect_immediate_post_increment_instruction(
                        0x0,
                        dest_register,
                        address_register,
                    ),
                    placeholder_name: Some(placeholder_name.clone()),
                    ..Default::default()
                },
            )),
        },
        // LOAD r1, (r2, a)+
        [
            AddressingMode::DirectRegister(dest_register),
            AddressingMode::IndirectRegisterDisplacementPostIncrement(
                displacement_register,
                address_register,
            ),
            AddressingMode::ShiftDefinition(shift_definition_data),
        ] => {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromIndirectRegisterPostIncrement,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
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
        // LOAD r1, (r2, a)+, ASL #1
        [
            AddressingMode::DirectRegister(dest_register),
            AddressingMode::IndirectRegisterDisplacementPostIncrement(
                displacement_register,
                address_register,
            ),
        ] => {
            Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromIndirectRegisterPostIncrement,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
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
        modes => {
            let error_string = format!("Invalid addressing mode for LOAD: ({modes:?})");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i_after_instruction,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}

//
