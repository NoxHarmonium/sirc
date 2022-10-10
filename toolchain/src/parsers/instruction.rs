use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::space0;
use nom::character::streaming::space1;
use nom::combinator::{eof, map};
use nom::multi::many1;
use nom::sequence::{separated_pair, terminated};
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    AddInstructionData, AddressInstructionData, CopyInstructionData, DivideInstructionData,
    ImmediateInstructionData, Instruction, IsEqualInstructionData,
    IsGreaterOrEqualThanInstructionData, IsGreaterThanInstructionData,
    IsLessOrEqualThanInstructionData, IsLessThanInstructionData, IsNotEqualInstructionData,
    JumpIfInstructionData, JumpIfNotInstructionData, JumpInstructionData, LoadOffsetImmediateData,
    LoadOffsetRegisterData, MultiplyInstructionData, NullInstructionData, RegisterInstructionData,
    SetAddressInstructionData, SetInstructionData, StoreOffsetImmediateData,
    StoreOffsetRegisterData, SubtractInstructionData,
};
use peripheral_cpu::registers::register_name_to_index;

use crate::types::object::SymbolDefinition;

use super::shared::{lexeme, parse_comma_sep, parse_label, parse_number, parse_symbol_reference};

pub struct LabelToken {
    pub name: String,
}

pub struct InstructionToken {
    pub instruction: Instruction,
    pub symbol_ref: Option<SymbolDefinition>,
}

pub enum Address {
    Value(u32),
    SymbolRef(String),
}

pub enum Token {
    Label(LabelToken),
    Instruction(InstructionToken),
}

pub fn extract_address_arguments(address: Address) -> (u32, Option<SymbolDefinition>) {
    match address {
        Address::SymbolRef(name) => (
            0x0,
            Some(SymbolDefinition {
                name,
                // First argument after instruction ID
                offset: 1,
            }),
        ),
        Address::Value(value) => (value, None),
    }
}

// Instruction Arguments Parsers

pub fn parse_address_argument(i: &str) -> IResult<&str, Address> {
    alt((
        map(parse_number, Address::Value),
        map(parse_symbol_reference, |symbol_name| {
            Address::SymbolRef(String::from(symbol_name))
        }),
    ))(i)
}

fn parse_rv_instruction(
    instruction_tag: &str,
) -> impl FnMut(&str) -> IResult<&str, (u8, u16)> + '_ {
    move |i: &str| {
        let (i, _) = terminated(tag(instruction_tag), space1)(i)?;
        let (i, (register_name, value)) =
            separated_pair(parse_register, parse_comma_sep, parse_number)(i)?;
        let register = register_name_to_index(register_name);
        // TODO: Does this truncate or what? Maybe we need to do a fallible convert and throw
        // if the value doesn't fit into u16 (values can only be u16, unlike addresses)
        Ok((i, (register, value as u16)))
    }
}

fn parse_rr_instruction(instruction_tag: &str) -> impl FnMut(&str) -> IResult<&str, (u8, u8)> + '_ {
    move |i: &str| {
        let (i, _) = terminated(tag(instruction_tag), space1)(i)?;
        let (i, (r1_name, r2_name)) =
            separated_pair(parse_register, parse_comma_sep, parse_register)(i)?;
        let (r1, r2) = (
            register_name_to_index(r1_name),
            register_name_to_index(r2_name),
        );
        Ok((i, (r1, r2)))
    }
}

fn parse_address_instruction(
    instruction_tag: &str,
) -> impl FnMut(&str) -> IResult<&str, Address> + '_ {
    move |i: &str| {
        let (i, _) = terminated(tag(instruction_tag), space1)(i)?;

        let (i, address) = parse_address_argument(i)?;
        Ok((i, address))
    }
}

// Instruction Parsers

pub fn parse_halt_instruction(i: &str) -> IResult<&str, InstructionToken> {
    let (i, _) = terminated(tag("HALT"), space0)(i)?;
    Ok((
        i,
        InstructionToken {
            instruction: Instruction::Halt(NullInstructionData {}),
            symbol_ref: None,
        },
    ))
}

// OR should these refer to some constant in the shared crate?
// what happens if the order changes or things are added?
// TODO: use a function to build this up
pub fn parse_instruction_token_(i: &str) -> IResult<&str, Token> {
    let (i, instruction_token) = alt((
        parse_halt_instruction,
        map(parse_rv_instruction("SETR"), |(register, value)| {
            InstructionToken {
                instruction: Instruction::Set(SetInstructionData {
                    data: ImmediateInstructionData { register, value },
                }),
                symbol_ref: None,
            }
        }),
        map(parse_address_instruction("SETA"), |address_enum| {
            let (address, symbol_ref) = extract_address_arguments(address_enum);
            InstructionToken {
                instruction: Instruction::SetAddress(SetAddressInstructionData {
                    data: AddressInstructionData { address },
                }),
                symbol_ref,
            }
        }),
        map(
            parse_rr_instruction("COPY"),
            |(src_register, dest_register)| InstructionToken {
                instruction: Instruction::Copy(CopyInstructionData {
                    data: RegisterInstructionData {
                        r1: src_register,
                        r2: dest_register,
                        r3: 0x00,
                    },
                }),
                symbol_ref: None,
            },
        ),
        map(
            parse_rr_instruction("ADDR"),
            |(src_register, dest_register)| InstructionToken {
                instruction: Instruction::Add(AddInstructionData {
                    data: RegisterInstructionData {
                        r1: src_register,
                        r2: dest_register,
                        r3: 0x00,
                    },
                }),
                symbol_ref: None,
            },
        ),
        map(
            parse_rr_instruction("SUBR"),
            |(src_register, dest_register)| InstructionToken {
                instruction: Instruction::Subtract(SubtractInstructionData {
                    data: RegisterInstructionData {
                        r1: src_register,
                        r2: dest_register,
                        r3: 0x00,
                    },
                }),
                symbol_ref: None,
            },
        ),
        map(
            parse_rr_instruction("MULR"),
            |(src_register, dest_register)| InstructionToken {
                instruction: Instruction::Multiply(MultiplyInstructionData {
                    data: RegisterInstructionData {
                        r1: src_register,
                        r2: dest_register,
                        r3: 0x00,
                    },
                }),
                symbol_ref: None,
            },
        ),
        map(
            parse_rr_instruction("DIVR"),
            |(src_register, dest_register)| InstructionToken {
                instruction: Instruction::Divide(DivideInstructionData {
                    data: RegisterInstructionData {
                        r1: src_register,
                        r2: dest_register,
                        r3: 0x00,
                    },
                }),
                symbol_ref: None,
            },
        ),
        map(
            parse_rr_instruction("CPEQ"),
            |(src_register, dest_register)| InstructionToken {
                instruction: Instruction::IsEqual(IsEqualInstructionData {
                    data: RegisterInstructionData {
                        r1: src_register,
                        r2: dest_register,
                        r3: 0x00,
                    },
                }),
                symbol_ref: None,
            },
        ),
        map(
            parse_rr_instruction("CPNE"),
            |(src_register, dest_register)| InstructionToken {
                instruction: Instruction::IsNotEqual(IsNotEqualInstructionData {
                    data: RegisterInstructionData {
                        r1: src_register,
                        r2: dest_register,
                        r3: 0x00,
                    },
                }),
                symbol_ref: None,
            },
        ),
        map(
            parse_rr_instruction("CPLT"),
            |(src_register, dest_register)| InstructionToken {
                instruction: Instruction::IsLessThan(IsLessThanInstructionData {
                    data: RegisterInstructionData {
                        r1: src_register,
                        r2: dest_register,
                        r3: 0x00,
                    },
                }),
                symbol_ref: None,
            },
        ),
        map(
            parse_rr_instruction("CPGT"),
            |(src_register, dest_register)| InstructionToken {
                instruction: Instruction::IsGreaterThan(IsGreaterThanInstructionData {
                    data: RegisterInstructionData {
                        r1: src_register,
                        r2: dest_register,
                        r3: 0x00,
                    },
                }),
                symbol_ref: None,
            },
        ),
        map(
            parse_rr_instruction("CPLT"),
            |(src_register, dest_register)| InstructionToken {
                instruction: Instruction::IsLessOrEqualThan(IsLessOrEqualThanInstructionData {
                    data: RegisterInstructionData {
                        r1: src_register,
                        r2: dest_register,
                        r3: 0x00,
                    },
                }),
                symbol_ref: None,
            },
        ),
        map(
            parse_rr_instruction("CPGT"),
            |(src_register, dest_register)| InstructionToken {
                instruction: Instruction::IsGreaterOrEqualThan(
                    IsGreaterOrEqualThanInstructionData {
                        data: RegisterInstructionData {
                            r1: src_register,
                            r2: dest_register,
                            r3: 0x00,
                        },
                    },
                ),
                symbol_ref: None,
            },
        ),
        map(parse_address_instruction("JUMP"), |address_enum| {
            let (address, symbol_ref) = extract_address_arguments(address_enum);
            InstructionToken {
                instruction: Instruction::Jump(JumpInstructionData {
                    data: AddressInstructionData { address },
                }),
                symbol_ref,
            }
        }),
        map(parse_address_instruction("JPEQ"), |address_enum| {
            let (address, symbol_ref) = extract_address_arguments(address_enum);
            InstructionToken {
                instruction: Instruction::JumpIf(JumpIfInstructionData {
                    data: AddressInstructionData { address },
                }),
                symbol_ref,
            }
        }),
        map(parse_address_instruction("JPNE"), |address_enum| {
            let (address, symbol_ref) = extract_address_arguments(address_enum);
            InstructionToken {
                instruction: Instruction::JumpIfNot(JumpIfNotInstructionData {
                    data: AddressInstructionData { address },
                }),
                symbol_ref,
            }
        }),
        map(
            parse_rr_instruction("LOAD"),
            |(src_register, dest_register)| InstructionToken {
                instruction: Instruction::LoadOffsetRegister(LoadOffsetRegisterData {
                    data: RegisterInstructionData {
                        r1: src_register,
                        r2: dest_register,
                        r3: 0x00,
                    },
                }),
                symbol_ref: None,
            },
        ),
        map(
            parse_rr_instruction("STOR"),
            |(src_register, dest_register)| InstructionToken {
                instruction: Instruction::StoreOffsetRegister(StoreOffsetRegisterData {
                    data: RegisterInstructionData {
                        r1: src_register,
                        r2: dest_register,
                        r3: 0x00,
                    },
                }),
                symbol_ref: None,
            },
        ),
        map(parse_rv_instruction("LOAD"), |(register, value)| {
            InstructionToken {
                instruction: Instruction::LoadOffsetImmediate(LoadOffsetImmediateData {
                    data: ImmediateInstructionData { register, value },
                }),
                symbol_ref: None,
            }
        }),
        map(parse_rv_instruction("STOR"), |(register, value)| {
            InstructionToken {
                instruction: Instruction::StoreOffsetImmediate(StoreOffsetImmediateData {
                    data: ImmediateInstructionData { register, value },
                }),
                symbol_ref: None,
            }
        }),
    ))(i)?;

    Ok((i, Token::Instruction(instruction_token)))
}

fn parse_register_(i: &str) -> IResult<&str, &str> {
    alt((
        tag("x1"),
        tag("y1"),
        tag("z1"),
        tag("x2"),
        tag("y2"),
        tag("z2"),
        tag("x3"),
        tag("y3"),
        tag("z3"),
        tag("ah"),
        tag("al"),
        tag("pc"),
        tag("sr"),
        tag("sp"),
    ))(i)
}

fn parse_register(i: &str) -> IResult<&str, &str> {
    lexeme(parse_register_)(i)
}

fn parse_label_token(i: &str) -> IResult<&str, Token> {
    let (i, name) = parse_label(i)?;
    Ok((
        i,
        Token::Label(LabelToken {
            name: String::from(name),
        }),
    ))
}

fn parse_instruction_token(i: &str) -> IResult<&str, Token> {
    lexeme(parse_instruction_token_)(i)
}

// TODO: Create object file struct and serialize with serde
// Addresses are replaced with indexes to object table and resolved by linker
pub fn parse_tokens(i: &str) -> IResult<&str, Vec<Token>> {
    let (i, tokens) = many1(alt((parse_instruction_token, parse_label_token)))(i)?;
    let (i, _) = eof(i)?;

    Ok((i, tokens))
}
