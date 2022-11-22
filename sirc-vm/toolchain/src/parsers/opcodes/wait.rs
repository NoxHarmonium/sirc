use crate::parsers::instruction::{parse_instruction_tag, InstructionToken};
use nom::combinator::map;
use peripheral_cpu::instructions::definitions::{
    ImpliedInstructionData, Instruction, WaitForInterruptInstructionData,
};

use super::super::shared::AsmResult;
pub fn wait(i: &str) -> AsmResult<InstructionToken> {
    map(parse_instruction_tag("WAIT"), |condition_flag| {
        InstructionToken {
            instruction: Instruction::WaitForInterrupt(WaitForInterruptInstructionData {
                data: ImpliedInstructionData { condition_flag },
            }),
            symbol_ref: None,
        }
    })(i)
}
