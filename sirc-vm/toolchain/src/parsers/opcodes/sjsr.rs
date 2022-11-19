use crate::{
    parsers::instruction::{
        override_ref_token_type_if_implied, parse_instruction_operands, parse_instruction_tag,
        AddressingMode, ImmediateType, InstructionToken,
    },
    types::object::RefType,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, Instruction, ShortJumpToSubroutineWithImmediateData,
};

pub fn sjsr(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("SJSR"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::Immediate(offset)] => match offset {
                ImmediateType::Value(offset) => InstructionToken {
                    instruction: Instruction::ShortJumpToSubroutineWithImmediate(
                        ShortJumpToSubroutineWithImmediateData {
                            data: ImmediateInstructionData {
                                register: 0x0, // unused
                                value: offset.to_owned(),
                                condition_flag,
                                additional_flags: 0x0,
                            },
                        },
                    ),
                    symbol_ref: None,
                },
                ImmediateType::SymbolRef(ref_token) => InstructionToken {
                    instruction: Instruction::ShortJumpToSubroutineWithImmediate(
                        ShortJumpToSubroutineWithImmediateData {
                            data: ImmediateInstructionData {
                                register: 0x0, // unused
                                value: 0x0,    // placeholder
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
            },
            _ => panic!("SJSR opcode only supports immediate addressing mode (e.g. SJMP #-3)"),
        },
    )(i)
}
