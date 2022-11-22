use crate::parsers::instruction::{parse_instruction_tag, InstructionToken};
use nom::combinator::map;
use peripheral_cpu::instructions::definitions::{
    ImpliedInstructionData, Instruction, NoOperationInstructionData,
};

use super::super::shared::AsmResult;
pub fn noop(i: &str) -> AsmResult<InstructionToken> {
    map(parse_instruction_tag("NOOP"), |condition_flag| {
        InstructionToken {
            instruction: Instruction::NoOperation(NoOperationInstructionData {
                data: ImpliedInstructionData { condition_flag },
            }),
            symbol_ref: None,
        }
    })(i)
}
