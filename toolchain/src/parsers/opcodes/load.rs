use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, AddressingMode, ImmediateType,
    InstructionToken, LabelToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, Instruction, LoadRegisterFromImmediateData,
    LoadRegisterFromIndirectImmediateData, LoadRegisterFromIndirectRegisterData,
    LoadRegisterFromRegisterData, RegisterInstructionData,
};

pub fn load(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("LOAD"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::DirectRegister(dest_register), AddressingMode::Immediate(offset)] => {
                match offset {
                    ImmediateType::Value(offset) => InstructionToken {
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
                    ImmediateType::SymbolRef(symbol_name) => InstructionToken {
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
                        symbol_ref: Some(LabelToken {
                            name: String::from(symbol_name),
                        }),
                    },
                }
            }
            [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register)] => {
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
                }
            }
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
                                    additional_flags: *address_register as u8,
                                },
                            },
                        ),
                        symbol_ref: None,
                    },
                    ImmediateType::SymbolRef(symbol_name) => InstructionToken {
                        instruction: Instruction::LoadRegisterFromIndirectImmediate(
                            LoadRegisterFromIndirectImmediateData {
                                data: ImmediateInstructionData {
                                    register: dest_register.to_register_index(),
                                    value: 0x0, // placeholder
                                    condition_flag,
                                    additional_flags: *address_register as u8,
                                },
                            },
                        ),
                        symbol_ref: Some(LabelToken {
                            name: String::from(symbol_name),
                        }),
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
                            r3: 0x0,
                            condition_flag,
                            // TODO: Clamp/validate additional_flags to 10 bits
                            additional_flags: *address_register as u8,
                        },
                    },
                ),
                symbol_ref: None,
            },

            // TODO: Better error message without being too verbose?
            _ => panic!("Invalid addressing mode for LOAD"),
        },
    )(i)
}
