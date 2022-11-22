use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, AddressingMode, InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use peripheral_cpu::instructions::definitions::{
    Instruction, LongJumpWithAddressRegisterData, RegisterInstructionData,
};

use super::super::shared::AsmResult;
pub fn ljmp(i: &str) -> AsmResult<InstructionToken> {
    map(
        tuple((parse_instruction_tag("LJMP"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::DirectAddressRegister(address_register)] => InstructionToken {
                instruction: Instruction::LongJumpWithAddressRegister(
                    LongJumpWithAddressRegisterData {
                        data: RegisterInstructionData {
                            r1: 0x0,
                            r2: 0x0,
                            r3: 0x0,
                            condition_flag,
                            additional_flags: address_register.to_register_index(),
                        },
                    },
                ),
                symbol_ref: None,
            },
            // TODO: Replace all these panics with proper error handler via the parser
            _ => panic!("LJSR opcode only supports implied addressing mode (e.g. INON)"),
        },
    )(i)
}
