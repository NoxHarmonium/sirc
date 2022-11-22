use crate::parsers::instruction::{parse_instruction_tag, InstructionToken};
use nom::combinator::map;
use peripheral_cpu::instructions::definitions::{
    HaltInstructionData, ImpliedInstructionData, Instruction,
};

use super::super::shared::AsmResult;
pub fn halt(i: &str) -> AsmResult<InstructionToken> {
    map(parse_instruction_tag("HALT"), |condition_flag| {
        InstructionToken {
            instruction: Instruction::Halt(HaltInstructionData {
                data: ImpliedInstructionData { condition_flag },
            }),
            symbol_ref: None,
        }
    })(i)
}
