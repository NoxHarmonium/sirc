use nom::combinator::map;
use nom::sequence::tuple;
use peripheral_cpu::instructions::definitions::{
    CompareInstructionData, Instruction, RegisterInstructionData,
};

use super::super::shared::AsmResult;
use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, AddressingMode, InstructionToken,
};
pub fn comp(i: &str) -> AsmResult<InstructionToken> {
    map(
        tuple((parse_instruction_tag("COMP"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register)] => {
                InstructionToken {
                    instruction: Instruction::Compare(CompareInstructionData {
                        data: RegisterInstructionData {
                            r1: dest_register.to_register_index(),
                            r2: src_register.to_register_index(),
                            r3: 0x00,
                            condition_flag,
                            additional_flags: 0x00,
                        },
                    }),
                    symbol_ref: None,
                }
            }
            _ => panic!(
                "COMP opcode only supports direct register addressing mode (e.g. COMP y1, z3)"
            ),
        },
    )(i)
}
