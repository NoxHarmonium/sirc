use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    ImpliedInstructionData, Instruction, ReturnFromSubroutineData,
};

pub fn rets(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("RETS"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [] => InstructionToken {
                instruction: Instruction::ReturnFromSubroutine(ReturnFromSubroutineData {
                    data: ImpliedInstructionData { condition_flag },
                }),
                symbol_ref: None,
            },
            _ => panic!("RETS opcode only supports implied addressing mode (e.g. RETS)"),
        },
    )(i)
}
