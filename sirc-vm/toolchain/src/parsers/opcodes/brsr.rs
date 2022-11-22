use crate::{
    parsers::instruction::{
        override_ref_token_type_if_implied, parse_instruction_operands, parse_instruction_tag,
        AddressingMode, ImmediateType, InstructionToken,
    },
    types::object::RefType,
};
use nom::combinator::map;
use nom::sequence::tuple;
use peripheral_cpu::instructions::definitions::{
    BranchToSubroutineData, ImmediateInstructionData, Instruction,
};

use super::super::shared::AsmResult;
pub fn brsr(i: &str) -> AsmResult<InstructionToken> {
    map(
        tuple((parse_instruction_tag("BRSR"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::Immediate(offset)] => match offset {
                ImmediateType::Value(offset) => InstructionToken {
                    instruction: Instruction::BranchToSubroutine(BranchToSubroutineData {
                        data: ImmediateInstructionData {
                            register: 0x0, // unused
                            value: offset.to_owned(),
                            condition_flag,
                            additional_flags: 0x0,
                        },
                    }),
                    symbol_ref: None,
                },
                ImmediateType::SymbolRef(ref_token) => InstructionToken {
                    instruction: Instruction::BranchToSubroutine(BranchToSubroutineData {
                        data: ImmediateInstructionData {
                            register: 0x0, // unused
                            value: 0x0,    // placeholder
                            condition_flag,
                            additional_flags: 0x0,
                        },
                    }),
                    symbol_ref: Some(override_ref_token_type_if_implied(
                        ref_token,
                        RefType::Offset,
                    )),
                },
            },
            _ => panic!("BRSR opcode only supports immediate addressing mode (e.g. BRSR #-3)"),
        },
    )(i)
}
