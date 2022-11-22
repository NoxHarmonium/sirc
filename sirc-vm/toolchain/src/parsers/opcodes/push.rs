use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, AddressingMode, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    Instruction, PushInstructionData, RegisterInstructionData,
};

use super::super::shared::AsmResult;
pub fn push(i: &str) -> AsmResult<InstructionToken> {
    map(
        tuple((parse_instruction_tag("PUSH"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::DirectRegister(dest_register)] => {
                InstructionToken {
                    instruction: Instruction::Push(PushInstructionData {
                        data: RegisterInstructionData {
                            r1: dest_register.to_register_index(),
                            r2: 0x0, // unused
                            r3: 0x0, // unused
                            condition_flag,
                            additional_flags: 0x00,
                        },
                    }),
                    symbol_ref: None,
                }
            }
            _ => panic!(
                "PUSH opcode only supports single direct register addressing mode (e.g. PUSH y1)"
            ),
        },
    )(i)
}
