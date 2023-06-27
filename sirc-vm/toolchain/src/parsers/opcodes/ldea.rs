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
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData, ShiftOperand,
    ShiftType,
};

use super::super::shared::AsmResult;
pub fn ldea(i: &str) -> AsmResult<InstructionToken> {
    let (i, ((_, condition_flag), operands)) =
        tuple((parse_instruction_tag("LDEA"), parse_instruction_operands1))(i)?;

    match operands.as_slice() {
        [AddressingMode::DirectAddressRegister(dest_register), AddressingMode::IndirectImmediateDisplacement(offset, address_register)] =>
        {
            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        instruction: InstructionData::Immediate(ImmediateInstructionData {
                            op_code: Instruction::LoadEffectiveAddressFromIndirectImmediate,
                            register: dest_register.to_register_index(),
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
                            op_code: Instruction::LoadEffectiveAddressFromIndirectImmediate,
                            register: dest_register.to_register_index(),
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
        [AddressingMode::DirectAddressRegister(dest_register), AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register)] =>
        {
            Ok((
                i,
                InstructionToken {
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
                    symbol_ref: None,
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
                    symbol_ref: None,
                },
            ))
        }
        modes => {
            let error_string = format!("Invalid addressing mode for LOAD: ({:?})", modes);
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}

//
