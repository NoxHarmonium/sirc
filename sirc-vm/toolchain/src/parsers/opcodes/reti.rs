use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    ImpliedInstructionData, Instruction, ReturnFromInterruptData,
};

pub fn reti(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("RETI"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [] => InstructionToken {
                instruction: Instruction::ReturnFromInterrupt(ReturnFromInterruptData {
                    data: ImpliedInstructionData { condition_flag },
                }),
                symbol_ref: None,
            },
            // TODO: Replace all these panics with proper error handler via the parser
            _ => panic!("RETI opcode only supports implied addressing mode (e.g. RETI)"),
        },
    )(i)
}