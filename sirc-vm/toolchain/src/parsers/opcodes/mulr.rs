use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, AddressingMode, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    Instruction, MultiplyInstructionData, RegisterInstructionData,
};

pub fn mulr(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("MULR"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register)] => {
                InstructionToken {
                    instruction: Instruction::Multiply(MultiplyInstructionData {
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
                "MULR opcode only supports direct register addressing mode (e.g. MULR y1, z3)"
            ),
        },
    )(i)
}