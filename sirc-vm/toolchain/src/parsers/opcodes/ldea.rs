use crate::{
    parsers::instruction::{
        override_ref_token_type_if_implied, parse_instruction_operands, parse_instruction_tag,
        AddressingMode, ImmediateType, InstructionToken,
    },
    types::object::RefType,
};
use nom::combinator::map;
use nom::sequence::tuple;
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, Instruction, LoadRegisterFromIndirectImmediateData,
    LoadRegisterFromIndirectRegisterData, RegisterInstructionData,
};

use super::super::shared::AsmResult;
pub fn ldea(i: &str) -> AsmResult<InstructionToken> {
    map(
        tuple((parse_instruction_tag("LDEA"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectImmediateDisplacement(offset, address_register)] =>
            {
                match offset {
                    ImmediateType::Value(offset) => InstructionToken {
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
                    ImmediateType::SymbolRef(ref_token) => InstructionToken {
                        instruction: Instruction::LoadRegisterFromIndirectImmediate(
                            LoadRegisterFromIndirectImmediateData {
                                data: ImmediateInstructionData {
                                    register: dest_register.to_register_index(),
                                    value: 0x0, // placeholder
                                    condition_flag,
                                    // TODO: Clamp/validate additional_flags to two bits
                                    additional_flags: address_register.to_register_index(),
                                },
                            },
                        ),
                        symbol_ref: Some(override_ref_token_type_if_implied(
                            ref_token,
                            RefType::LowerByte,
                        )),
                    },
                }
            }
            [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectRegisterDisplacement(
                displacement_register,
                address_register,
            )] => InstructionToken {
                instruction: Instruction::LoadRegisterFromIndirectRegister(
                    LoadRegisterFromIndirectRegisterData {
                        data: RegisterInstructionData {
                            r1: dest_register.to_register_index(),
                            r2: displacement_register.to_register_index(),
                            r3: address_register.to_register_index(),
                            condition_flag,
                            additional_flags: 0x0,
                        },
                    },
                ),
                symbol_ref: None,
            },

            // TODO: Better error message without being too verbose?
            modes => panic!("Invalid addressing mode for LDEA: ({:?})", modes),
        },
    )(i)
}
