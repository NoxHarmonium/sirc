use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    DisableInterruptsData, ImpliedInstructionData, Instruction,
};

pub fn inof(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("INOF"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [] => InstructionToken {
                instruction: Instruction::DisableInterrupts(DisableInterruptsData {
                    data: ImpliedInstructionData { condition_flag },
                }),
                symbol_ref: None,
            },
            // TODO: Replace all these panics with proper error handler via the parser
            _ => panic!("INOF opcode only supports implied addressing mode (e.g. HALT)"),
        },
    )(i)
}
