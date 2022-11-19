use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    ImpliedInstructionData, Instruction, WaitForInterruptInstructionData,
};

pub fn wait(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("WAIT"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [] => InstructionToken {
                instruction: Instruction::WaitForInterrupt(WaitForInterruptInstructionData {
                    data: ImpliedInstructionData { condition_flag },
                }),
                symbol_ref: None,
            },
            // TODO: Replace all these panics with proper error handler via the parser
            _ => panic!("WAIT opcode only supports implied addressing mode (e.g. WAIT)"),
        },
    )(i)
}
