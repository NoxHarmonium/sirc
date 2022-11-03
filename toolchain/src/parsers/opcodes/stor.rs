use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, AddressingMode, ImmediateType,
    InstructionToken, LabelToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, Instruction, RegisterInstructionData,
    StoreRegisterToIndirectImmediateData, StoreRegisterToIndirectRegisterData,
};

pub fn stor(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("STOR"), parse_instruction_operands)),
        |(condition_flag, operands)| {
            match operands.as_slice() {
                [AddressingMode::IndirectImmediateDisplacement(offset, address_register), AddressingMode::DirectRegister(dest_register)] =>
                {
                    match offset {
                        ImmediateType::Value(offset) => InstructionToken {
                            instruction: Instruction::StoreRegisterToIndirectImmediate(
                                StoreRegisterToIndirectImmediateData {
                                    data: ImmediateInstructionData {
                                        register: dest_register.to_register_index(),
                                        value: offset.to_owned(),
                                        condition_flag,
                                        additional_flags: *address_register as u8,
                                    },
                                },
                            ),
                            symbol_ref: None,
                        },
                        ImmediateType::SymbolRef(symbol_name) => InstructionToken {
                            instruction: Instruction::StoreRegisterToIndirectImmediate(
                                StoreRegisterToIndirectImmediateData {
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
                [AddressingMode::IndirectRegisterDisplacement(
                    displacement_register,
                    address_register,
                ), AddressingMode::DirectRegister(dest_register)] => InstructionToken {
                    instruction: Instruction::StoreRegisterToIndirectRegister(
                        StoreRegisterToIndirectRegisterData {
                            data: RegisterInstructionData {
                                r1: dest_register.to_register_index(),
                                r2: displacement_register.to_register_index(),
                                r3: 0x0,
                                condition_flag,
                                additional_flags: *address_register as u8,
                            },
                        },
                    ),
                    symbol_ref: None,
                },

                // TODO: Better error message without being too verbose?
                _ => panic!("Invalid addressing mode for STOR"),
            }
        },
    )(i)
}
