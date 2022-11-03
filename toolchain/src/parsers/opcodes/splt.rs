// TODO: Undo this circular dependency
use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, AddressingMode, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    Instruction, RegisterInstructionData, SplitWordInstructionData,
};

pub fn splt(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("SPLT"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::DirectRegister(dest_register_high), AddressingMode::DirectRegister(dest_register_low), AddressingMode::DirectRegister(src_register)] => {
                InstructionToken {
                    instruction: Instruction::SplitWord(SplitWordInstructionData {
                        data: RegisterInstructionData {
                            r1: dest_register_high.to_register_index(),
                            r2: dest_register_low.to_register_index(),
                            r3: src_register.to_register_index(),
                            condition_flag,
                            additional_flags: 0x00,
                        },
                    }),
                    symbol_ref: None,
                }
            }
            _ => panic!(
                "SPLT opcode only supports direct register addressing mode (e.g. ADDR y1, z3)"
            ),
        },
    )(i)
}
