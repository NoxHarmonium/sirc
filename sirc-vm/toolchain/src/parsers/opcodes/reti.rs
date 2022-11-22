use crate::parsers::instruction::{parse_instruction_tag, InstructionToken};
use nom::combinator::map;
use peripheral_cpu::instructions::definitions::{
    ImpliedInstructionData, Instruction, ReturnFromInterruptData,
};

use super::super::shared::AsmResult;
pub fn reti(i: &str) -> AsmResult<InstructionToken> {
    map(parse_instruction_tag("RETI"), |condition_flag| {
        InstructionToken {
            instruction: Instruction::ReturnFromInterrupt(ReturnFromInterruptData {
                data: ImpliedInstructionData { condition_flag },
            }),
            symbol_ref: None,
        }
    })(i)
}
