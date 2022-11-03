use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, AddressingMode, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    Instruction, JoinWordInstructionData, RegisterInstructionData,
};

pub fn join(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("JOIN"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register_high), AddressingMode::DirectRegister(src_register_low)] => {
                InstructionToken {
                    instruction: Instruction::JoinWord(JoinWordInstructionData {
                        data: RegisterInstructionData {
                            r1: dest_register.to_register_index(),
                            r2: src_register_high.to_register_index(),
                            r3: src_register_low.to_register_index(),
                            condition_flag,
                            additional_flags: 0x00,
                        },
                    }),
                    symbol_ref: None,
                }
            }
            _ => panic!(
                "JOIN opcode only supports direct register addressing mode (e.g. ADDR y1, z3)"
            ),
        },
    )(i)
}
