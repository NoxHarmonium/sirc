use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    ImpliedInstructionData, Instruction, NoOperationInstructionData,
};

use super::super::shared::AsmResult;
pub fn noop(i: &str) -> AsmResult<InstructionToken> {
    map(
        tuple((parse_instruction_tag("NOOP"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [] => InstructionToken {
                instruction: Instruction::NoOperation(NoOperationInstructionData {
                    data: ImpliedInstructionData { condition_flag },
                }),
                symbol_ref: None,
            },
            _ => panic!("NOOP opcode only supports implied addressing mode (e.g. LJMP)"),
        },
    )(i)
}
