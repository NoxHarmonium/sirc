use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, AddressingMode, ImmediateType,
    InstructionToken, LabelToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    BranchInstructionData, ImmediateInstructionData, Instruction,
};

pub fn bran(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("BRAN"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::Immediate(offset)] => match offset {
                ImmediateType::Value(offset) => InstructionToken {
                    instruction: Instruction::Branch(BranchInstructionData {
                        data: ImmediateInstructionData {
                            register: 0x0, // unused
                            value: offset.to_owned(),
                            condition_flag,
                            additional_flags: 0x0,
                        },
                    }),
                    symbol_ref: None,
                },
                ImmediateType::SymbolRef(symbol_name) => InstructionToken {
                    instruction: Instruction::Branch(BranchInstructionData {
                        data: ImmediateInstructionData {
                            register: 0x0, // unused
                            value: 0x0,    // placeholder
                            condition_flag,
                            additional_flags: 0x0,
                        },
                    }),
                    symbol_ref: Some(LabelToken {
                        name: String::from(symbol_name),
                    }),
                },
            },
            _ => panic!("BRAN opcode only supports immediate addressing mode (e.g. BRAN #-3)"),
        },
    )(i)
}
