use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    ImpliedInstructionData, Instruction, LongJumpInstructionData,
};

pub fn ljmp(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("LJMP"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [] => InstructionToken {
                instruction: Instruction::LongJump(LongJumpInstructionData {
                    data: ImpliedInstructionData { condition_flag },
                }),
                symbol_ref: None,
            },
            _ => panic!("LJMP opcode only supports implied addressing mode (e.g. LJMP)"),
        },
    )(i)
}
