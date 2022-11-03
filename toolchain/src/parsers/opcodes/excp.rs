use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, AddressingMode, ImmediateType,
    InstructionToken,
};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, Instruction, TriggerSoftwareInterruptData,
};

pub fn excp(i: &str) -> IResult<&str, InstructionToken> {
    map(
        tuple((parse_instruction_tag("EXCP"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::Immediate(offset)] => match offset {
                ImmediateType::Value(offset) => InstructionToken {
                    instruction: Instruction::TriggerSoftwareInterrupt(
                        TriggerSoftwareInterruptData {
                            data: ImmediateInstructionData {
                                register: 0x0, // unused
                                value: offset.to_owned(),
                                condition_flag,
                                additional_flags: 0x0,
                            },
                        },
                    ),
                    symbol_ref: None,
                },
                ImmediateType::SymbolRef(_) => {
                    // TODO: Proper error handling that isn't a panic
                    // TODO: Validate that value is within range (128?)
                    panic!("EXCP only supports a standard value (no symbol references)")
                }
            },
            _ => panic!("EXCP opcode only supports immediate addressing mode (e.g. EXCP)"),
        },
    )(i)
}
