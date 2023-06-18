use crate::{
    parsers::instruction::{
        override_ref_token_type_if_implied, parse_instruction_operands, parse_instruction_tag,
        AddressingMode, ImmediateType, InstructionToken,
    },
    types::object::RefType,
};
use nom::error::{ErrorKind, FromExternalError};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData,
};

use super::super::shared::AsmResult;
pub fn load(i: &str) -> AsmResult<InstructionToken> {
    let (i, (_, condition_flag)) = parse_instruction_tag("LOAD")(i)?;
    let (i, operands) = parse_instruction_operands(i)?;

    match operands.as_slice() {
        [AddressingMode::DirectRegister(dest_register), AddressingMode::Immediate(offset)] => {
            match offset {
                ImmediateType::Value(value) => Ok((
                    i,
                    InstructionToken {
                        instruction: InstructionData::Immediate(ImmediateInstructionData {
                            op_code: Instruction::LoadRegisterFromImmediate,
                            register: dest_register.to_register_index(),
                            value: value.to_owned(),
                            condition_flag,
                            additional_flags: 0x0,
                        }),
                        symbol_ref: None,
                    },
                )),
                ImmediateType::SymbolRef(ref_token) => Ok((
                    i,
                    InstructionToken {
                        instruction: InstructionData::Immediate(ImmediateInstructionData {
                            op_code: Instruction::LoadRegisterFromImmediate,
                            register: dest_register.to_register_index(),
                            value: 0x0, // Placeholder
                            condition_flag,
                            additional_flags: 0x0,
                        }),
                        symbol_ref: Some(override_ref_token_type_if_implied(
                            ref_token,
                            RefType::LowerByte,
                        )),
                    },
                )),
            }
        }
        [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register)] => {
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromRegister,
                        r1: dest_register.to_register_index(),
                        r2: src_register.to_register_index(),
                        r3: 0x00,
                        condition_flag,
                        additional_flags: 0x00,
                    }),
                    symbol_ref: None,
                },
            ))
        }
        [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectImmediateDisplacement(offset, address_register)] =>
        {
            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        instruction: InstructionData::Immediate(ImmediateInstructionData {
                            op_code: Instruction::LoadRegisterFromIndirectImmediate,
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
                            op_code: Instruction::LoadRegisterFromIndirectImmediate,
                            register: dest_register.to_register_index(),
                            value: 0x0, // Placeholder
                            condition_flag,
                            additional_flags: address_register.to_register_index(),
                        }),
                        symbol_ref: Some(override_ref_token_type_if_implied(
                            ref_token,
                            RefType::LowerByte,
                        )),
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
                        r2: displacement_register.to_register_index(),
                        r3: 0x0, // Unused
                        condition_flag,
                        // TODO: Clamp/validate additional_flags to 10 bits
                        additional_flags: address_register.to_register_index(),
                    }),
                    symbol_ref: None,
                },
            ))
        }
        [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectPostIncrement(address_register)] =>
        {
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromIndirectRegisterPostIncrement,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: 0x0, // Unused
                        condition_flag,
                        // TODO: Clamp/validate additional_flags to 10 bits
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
