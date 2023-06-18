use crate::{
    parsers::instruction::{
        override_ref_token_type_if_implied, parse_instruction_operands, parse_instruction_tag,
        AddressingMode, ImmediateType, InstructionToken,
    },
    types::object::RefType,
};
use nom::{combinator::map_res, error::FromExternalError};
use nom::{error::ErrorKind, sequence::tuple};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData,
};

use super::super::shared::AsmResult;
pub fn stor(i: &str) -> AsmResult<InstructionToken> {
    map_res(
        tuple((parse_instruction_tag("STOR"), parse_instruction_operands)),
        |((_, condition_flag), operands)| {
            match operands.as_slice() {
                [AddressingMode::IndirectImmediateDisplacement(offset, address_register), AddressingMode::DirectRegister(dest_register)] =>
                {
                    Ok(match offset {
                        ImmediateType::Value(offset) => InstructionToken {
                            instruction: InstructionData::Immediate(ImmediateInstructionData {
                                op_code: Instruction::StoreRegisterToIndirectImmediate,
                                register: dest_register.to_register_index(),
                                value: offset.to_owned(),
                                condition_flag,
                                additional_flags: address_register.to_register_index(),
                            }),
                            symbol_ref: None,
                        },
                        ImmediateType::SymbolRef(ref_token) => InstructionToken {
                            instruction: InstructionData::Immediate(ImmediateInstructionData {
                                op_code: Instruction::StoreRegisterToIndirectImmediate,
                                register: dest_register.to_register_index(),
                                value: 0x0, // placeholder
                                condition_flag,
                                additional_flags: address_register.to_register_index(),
                            }),
                            symbol_ref: Some(override_ref_token_type_if_implied(
                                ref_token,
                                RefType::LowerByte,
                            )),
                        },
                    })
                }
                [AddressingMode::IndirectRegisterDisplacement(
                    displacement_register,
                    address_register,
                ), AddressingMode::DirectRegister(dest_register)] => Ok(InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::StoreRegisterToIndirectRegister,
                        r1: dest_register.to_register_index(),
                        r2: displacement_register.to_register_index(),
                        r3: address_register.to_register_index(),
                        condition_flag,
                        additional_flags: 0x0,
                    }),
                    symbol_ref: None,
                }),

                [AddressingMode::IndirectPreDecrement(address_register), AddressingMode::DirectRegister(dest_register)] =>
                {
                    Ok({
                        InstructionToken {
                            instruction: InstructionData::Register(RegisterInstructionData {
                                op_code: Instruction::StoreRegisterToIndirectRegisterPreDecrement,
                                r1: dest_register.to_register_index(),
                                r2: 0x0, //Unused
                                r3: address_register.to_register_index(),
                                condition_flag,
                                additional_flags: 0x0,
                            }),
                            symbol_ref: None,
                        }
                    })
                }

                // TODO: Better error message without being too verbose?
                modes => {
                    let error_string = format!("Invalid addressing mode for STOR: ({:?})", modes);
                    Err(nom::Err::Failure(ErrorTree::from_external_error(
                        i.to_owned(),
                        ErrorKind::Fail,
                        error_string.as_str(),
                    )))
                }
            }
        },
    )(i)
}
