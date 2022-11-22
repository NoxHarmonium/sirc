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
    ImmediateInstructionData, Instruction, LoadRegisterFromImmediateData,
    LoadRegisterFromIndirectImmediateData, LoadRegisterFromIndirectRegisterData,
    LoadRegisterFromRegisterData, RegisterInstructionData,
};

use super::super::shared::AsmResult;
pub fn load(i: &str) -> AsmResult<InstructionToken> {
    let (i, condition_flag) = parse_instruction_tag("LOAD")(i)?;
    let (i, operands) = parse_instruction_operands(i)?;

    match operands.as_slice() {
        [AddressingMode::DirectRegister(dest_register), AddressingMode::Immediate(offset)] => {
            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        instruction: Instruction::LoadRegisterFromImmediate(
                            LoadRegisterFromImmediateData {
                                data: ImmediateInstructionData {
                                    register: dest_register.to_register_index(),
                                    value: offset.to_owned(),
                                    condition_flag,
                                    additional_flags: 0x0,
                                },
                            },
                        ),
                        symbol_ref: None,
                    },
                )),
                ImmediateType::SymbolRef(ref_token) => Ok((
                    i,
                    InstructionToken {
                        instruction: Instruction::LoadRegisterFromImmediate(
                            LoadRegisterFromImmediateData {
                                data: ImmediateInstructionData {
                                    register: dest_register.to_register_index(),
                                    value: 0x0, // placeholder
                                    condition_flag,
                                    additional_flags: 0x0,
                                },
                            },
                        ),
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
                    instruction: Instruction::LoadRegisterFromRegister(
                        LoadRegisterFromRegisterData {
                            data: RegisterInstructionData {
                                r1: dest_register.to_register_index(),
                                r2: src_register.to_register_index(),
                                r3: 0x00,
                                condition_flag,
                                additional_flags: 0x00,
                            },
                        },
                    ),
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
                        instruction: Instruction::LoadRegisterFromIndirectImmediate(
                            LoadRegisterFromIndirectImmediateData {
                                data: ImmediateInstructionData {
                                    register: dest_register.to_register_index(),
                                    value: offset.to_owned(),
                                    condition_flag,
                                    // TODO: Clamp/validate additional_flags to two bits
                                    additional_flags: address_register.to_register_index(),
                                },
                            },
                        ),
                        symbol_ref: None,
                    },
                )),
                ImmediateType::SymbolRef(ref_token) => Ok((
                    i,
                    InstructionToken {
                        instruction: Instruction::LoadRegisterFromIndirectImmediate(
                            LoadRegisterFromIndirectImmediateData {
                                data: ImmediateInstructionData {
                                    register: dest_register.to_register_index(),
                                    value: 0x0, // placeholder
                                    condition_flag,
                                    additional_flags: address_register.to_register_index(),
                                },
                            },
                        ),
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
                    instruction: Instruction::LoadRegisterFromIndirectRegister(
                        LoadRegisterFromIndirectRegisterData {
                            data: RegisterInstructionData {
                                r1: dest_register.to_register_index(),
                                r2: displacement_register.to_register_index(),
                                r3: address_register.to_register_index(),
                                condition_flag,
                                // TODO: Clamp/validate additional_flags to 10 bits
                                additional_flags: 0x0,
                            },
                        },
                    ),
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
