use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, InstructionToken,
};
use nom::IResult;
use nom::{combinator::map, sequence::tuple};
use peripheral_cpu::instructions::definitions::{
    HaltInstructionData, ImpliedInstructionData, Instruction,
};

use super::super::shared::AsmResult;
pub fn halt(i: &str) -> AsmResult<InstructionToken> {
    map(
        tuple((parse_instruction_tag("HALT"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [] => InstructionToken {
                instruction: Instruction::Halt(HaltInstructionData {
                    data: ImpliedInstructionData { condition_flag },
                }),
                symbol_ref: None,
            },
            // TODO: Replace all these panics with proper error handler via the parser
            _ => panic!("HALT opcode only supports implied addressing mode (e.g. HALT)"),
        },
    )(i)
}
