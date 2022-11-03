use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    ImpliedInstructionData, Instruction, ShortJumpInstructionData,
};

pub fn sjmp(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("SJMP"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [] => InstructionToken {
                instruction: Instruction::ShortJump(ShortJumpInstructionData {
                    data: ImpliedInstructionData { condition_flag },
                }),
                symbol_ref: None,
            },
            _ => panic!("SJMP opcode only supports implied addressing mode (e.g. SJMP)"),
        },
    )(i)
}
