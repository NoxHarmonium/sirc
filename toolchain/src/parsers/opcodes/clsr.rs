use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    ClearAluStatusInstructionData, ImpliedInstructionData, Instruction,
};

pub fn clsr(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("CLSR"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [] => InstructionToken {
                instruction: Instruction::ClearAluStatus(ClearAluStatusInstructionData {
                    data: ImpliedInstructionData { condition_flag },
                }),
                symbol_ref: None,
            },
            _ => panic!("CLSR opcode only supports implied addressing mode (e.g. LJMP)"),
        },
    )(i)
}
