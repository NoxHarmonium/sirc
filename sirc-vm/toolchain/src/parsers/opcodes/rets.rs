use crate::parsers::instruction::{parse_instruction_tag, InstructionToken};
use nom::combinator::map;
use peripheral_cpu::instructions::definitions::{
    ImpliedInstructionData, Instruction, ReturnFromSubroutineData,
};

use super::super::shared::AsmResult;
pub fn rets(i: &str) -> AsmResult<InstructionToken> {
    map(parse_instruction_tag("RETS"), |condition_flag| {
        InstructionToken {
            instruction: Instruction::ReturnFromSubroutine(ReturnFromSubroutineData {
                data: ImpliedInstructionData { condition_flag },
            }),
            symbol_ref: None,
        }
    })(i)
}
