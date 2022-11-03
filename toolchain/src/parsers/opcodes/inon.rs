use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    EnableInterruptsData, ImpliedInstructionData, Instruction,
};

pub fn inon(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("INON"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [] => InstructionToken {
                instruction: Instruction::EnableInterrupts(EnableInterruptsData {
                    data: ImpliedInstructionData { condition_flag },
                }),
                symbol_ref: None,
            },
            // TODO: Replace all these panics with proper error handler via the parser
            _ => panic!("INON opcode only supports implied addressing mode (e.g. INON)"),
        },
    )(i)
}
