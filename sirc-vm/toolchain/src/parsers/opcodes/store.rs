use super::super::shared::AsmResult;
use crate::types::instruction::InstructionToken;
use crate::{
    parsers::{
        data::override_ref_token_type_if_implied,
        instruction::{
            AddressingMode, ImmediateType, parse_instruction_operands0, parse_instruction_tag,
        },
        shared::split_shift_definition_data,
    },
    types::object::RefType,
};
use nom::error::ErrorKind;
use nom::error::FromExternalError;
use nom_supreme::error::ErrorTree;
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData,
        ShiftOperand, ShiftType,
    },
    registers::{AddressRegisterName, RegisterName},
};
pub fn stor(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();
    let (i_after_instruction, (_, condition_flag, status_register_update_source)) =
        parse_instruction_tag("STOR")(i)?;

    let (i, operands) = parse_instruction_operands0(i_after_instruction)?;

    if status_register_update_source.is_some() {
        let error_string = "The [STOR] opcode does not support an explicit status register update source. Only ALU instructions can update the status register as a side-effect.";
        return Err(nom::Err::Failure(ErrorTree::from_external_error(
            i_after_instruction,
            ErrorKind::Fail,
            error_string,
        )));
    }

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

    // NOTE: No shifting with immediate operands because there is no short immediate representation of STOR
    match operands.as_slice() {
        // STOR (#0, a), r1
        [
            AddressingMode::IndirectImmediateDisplacement(offset, address_register),
            AddressingMode::DirectRegister(src_register),
        ] => match offset {
            ImmediateType::Value(offset) => Ok((
                i,
                InstructionToken {
                    input_length,
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
                    input_length,
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
                    input_length,
                    instruction: construct_indirect_immediate_instruction(
                        0x0,
                        src_register,
                        address_register,
                    ),
                    placeholder_name: Some(placeholder_name.clone()),
                    ..Default::default()
                },
            )),
        },
        // STOR (r1, a), r1
        [
            AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register),
            AddressingMode::DirectRegister(src_register),
        ] => {
            Ok((
                i,
                InstructionToken {
                    input_length,
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
        // STOR (r1, a), r1, ASL #1
        [
            AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register),
            AddressingMode::DirectRegister(src_register),
            AddressingMode::ShiftDefinition(shift_definition_data),
        ] => {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            Ok((
                i,
                InstructionToken {
                    input_length,
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
        // STOR -(#0, a), r1
        [
            AddressingMode::IndirectImmediateDisplacementPreDecrement(offset, address_register),
            AddressingMode::DirectRegister(src_register),
        ] => match offset {
            ImmediateType::Value(offset) => Ok((
                i,
                InstructionToken {
                    input_length,
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
                    input_length,
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
                    input_length,
                    instruction: construct_indirect_immediate_pre_increment_instruction(
                        0x0,
                        src_register,
                        address_register,
                    ),
                    placeholder_name: Some(placeholder_name.clone()),
                    ..Default::default()
                },
            )),
        },
        // STOR -(r1, a), r1
        [
            AddressingMode::IndirectRegisterDisplacementPreDecrement(
                displacement_register,
                address_register,
            ),
            AddressingMode::DirectRegister(src_register),
        ] => {
            Ok((i, {
                InstructionToken {
                    input_length,
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
        // STOR -(r1, a), r1, ASL #1
        [
            AddressingMode::IndirectRegisterDisplacementPreDecrement(
                displacement_register,
                address_register,
            ),
            AddressingMode::DirectRegister(src_register),
            AddressingMode::ShiftDefinition(shift_definition_data),
        ] => {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            Ok((i, {
                InstructionToken {
                    input_length,
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
        // TODO: Investigate ways of improving parse failure error messages
        // category=Toolchain
        // Better error message without being too verbose?
        modes => {
            let error_string = format!("Invalid addressing mode for STOR: ({modes:?})");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i_after_instruction,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}
