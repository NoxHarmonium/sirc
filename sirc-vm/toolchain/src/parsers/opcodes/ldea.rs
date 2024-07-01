use crate::{
    parsers::{
        data::override_ref_token_type_if_implied,
        instruction::{
            parse_instruction_operands1, parse_instruction_tag, AddressingMode, ImmediateType,
            InstructionToken,
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
    registers::AddressRegisterName,
};

use super::super::shared::AsmResult;
pub fn ldea(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();
    let (i, ((_, condition_flag), operands)) =
        tuple((parse_instruction_tag("LDEA"), parse_instruction_operands1))(i)?;

    let construct_load_effective_address_instruction =
        |value: u16,
         dest_register: &AddressRegisterName,
         address_register: &AddressRegisterName| {
            InstructionData::Immediate(ImmediateInstructionData {
                op_code: Instruction::LoadEffectiveAddressFromIndirectImmediate,
                register: dest_register.to_register_index(),
                value,
                condition_flag,
                additional_flags: address_register.to_register_index(),
            })
        };

    match operands.as_slice() {
        [AddressingMode::DirectAddressRegister(dest_register), AddressingMode::IndirectImmediateDisplacement(offset, address_register)] =>
        {
            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: construct_load_effective_address_instruction(
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
                        instruction: construct_load_effective_address_instruction(
                            0x0, // Will be replaced by linker
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
                        instruction: construct_load_effective_address_instruction(
                            0x0, // Will be replaced by linker
                            dest_register,
                            address_register,
                        ),
                        placeholder_name: Some(placeholder_name.clone()),
                        ..Default::default()
                    },
                )),
            }
        }
        [AddressingMode::DirectAddressRegister(dest_register), AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register)] =>
        {
            Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadEffectiveAddressFromIndirectRegister,
                        r1: dest_register.to_register_index(),
                        r2: displacement_register.to_register_index(),
                        r3: 0x0, // Unused
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
        [AddressingMode::DirectAddressRegister(dest_register), AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register), AddressingMode::ShiftDefinition(shift_definition_data)] =>
        {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadEffectiveAddressFromIndirectRegister,
                        r1: dest_register.to_register_index(),
                        r2: displacement_register.to_register_index(),
                        r3: 0x0, // Unused
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
        modes => {
            let error_string = format!("Invalid addressing mode for LDEA: ({modes:?})");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}

//
