use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, AddressingMode, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use peripheral_cpu::instructions::definitions::{
    DivideInstructionData, Instruction, RegisterInstructionData,
};

use super::super::shared::AsmResult;
pub fn divr(i: &str) -> AsmResult<InstructionToken> {
    map(
        tuple((parse_instruction_tag("DIVR"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register)] => {
                InstructionToken {
                    instruction: Instruction::Divide(DivideInstructionData {
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
                "DIVR opcode only supports direct register addressing mode (e.g. DIVR y1, z3)"
            ),
        },
    )(i)
}
