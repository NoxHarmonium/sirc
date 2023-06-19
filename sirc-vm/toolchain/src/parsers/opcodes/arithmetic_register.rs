use crate::parsers::instruction::{
    parse_instruction_operands1, parse_instruction_tag, AddressingMode, InstructionToken,
};
use nom::error::{ErrorKind, FromExternalError};
use nom::sequence::tuple;
use nom::{branch::alt, combinator::map_res};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::instructions::definitions::{
    Instruction, InstructionData, RegisterInstructionData,
};

fn tag_to_instruction(tag: String) -> Instruction {
    match tag.as_str() {
        "ADDR" => Instruction::AddRegister,
        "ADCR" => Instruction::AddRegisterWithCarry,
        "SUBR" => Instruction::SubtractRegister,
        "SBCR" => Instruction::SubtractRegisterWithCarry,
        "ANDR" => Instruction::AndRegister,
        "ORRR" => Instruction::OrRegister,
        "XORR" => Instruction::XorRegister,
        "LSLR" => Instruction::LogicalShiftLeftRegister,
        "LSRR" => Instruction::LogicalShiftRightRegister,
        "ASLR" => Instruction::ArithmeticShiftLeftRegister,
        "ASRR" => Instruction::ArithmeticShiftRightRegister,
        "RTLR" => Instruction::RotateLeftRegister,
        "RTRR" => Instruction::RotateRightRegister,
        "CMPR" => Instruction::CompareRegister,

        _ => panic!("No tag mapping for instruction [{}]", tag),
    }
}

use super::super::shared::AsmResult;

///
/// Parses register arithmetic/logic opcodes
///
/// ```
/// use toolchain::parsers::opcodes::arithmetic_register::arithmetic_register;
/// use toolchain::parsers::instruction::InstructionToken;
/// use peripheral_cpu::instructions::definitions::{ConditionFlags, Instruction, InstructionData, RegisterInstructionData};
///
/// let (_, parsed_instruction) = arithmetic_register("ADDR|>= r1, r3").unwrap();
/// let (op_code, r1, r2, r3, condition_flag, additional_flags) = match parsed_instruction.instruction {
///     InstructionData::Register(inner) => (inner.op_code, inner.r1, inner.r2, inner.r3, inner.condition_flag, inner.additional_flags),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// // TODO: Make a helper function or something to make these asserts smaller
/// assert_eq!(op_code, Instruction::AddRegister);
/// assert_eq!(r1, 0);
/// assert_eq!(r2, 0);
/// assert_eq!(r3, 2);
/// assert_eq!(additional_flags, 0x0);
/// assert_eq!(condition_flag, ConditionFlags::GreaterOrEqual);
///
/// let (_, parsed_instruction) = arithmetic_register("ADDR|>= r1, r2, r3").unwrap();
/// let (op_code, r1, r2, r3, condition_flag, additional_flags) = match parsed_instruction.instruction {
///     InstructionData::Register(inner) => (inner.op_code, inner.r1, inner.r2, inner.r3, inner.condition_flag, inner.additional_flags),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// // TODO: Make a helper function or something to make these asserts smaller
/// assert_eq!(op_code, Instruction::AddRegister);
/// assert_eq!(r1, 0);
/// assert_eq!(r2, 1);
/// assert_eq!(r3, 2);
/// assert_eq!(additional_flags, 0x0);
/// assert_eq!(condition_flag, ConditionFlags::GreaterOrEqual);
/// ```
pub fn arithmetic_register(i: &str) -> AsmResult<InstructionToken> {
    let instructions = alt((
        parse_instruction_tag("ADDR"),
        parse_instruction_tag("ADCR"),
        parse_instruction_tag("SUBR"),
        parse_instruction_tag("SBCR"),
        parse_instruction_tag("ANDR"),
        parse_instruction_tag("ORRR"),
        parse_instruction_tag("XORR"),
        parse_instruction_tag("LSLR"),
        parse_instruction_tag("LSRR"),
        parse_instruction_tag("ASLR"),
        parse_instruction_tag("ASRR"),
        parse_instruction_tag("RTLR"),
        parse_instruction_tag("RTRR"),
        parse_instruction_tag("CMPR"),
    ));

    map_res(
        tuple((instructions, parse_instruction_operands1)),
        |((tag, condition_flag), operands)| {
            match operands.as_slice() {
                // The CPU does not supportregister arithmetic instructions with two register operands
                // If third register is omitted, it is implied that the first operand is the destination register and the assembler fills that in
                [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register)] => {
                    Ok(InstructionToken {
                        instruction: InstructionData::Register(RegisterInstructionData {
                            op_code: tag_to_instruction(tag),
                            r1: dest_register.to_register_index(),
                            r2: dest_register.to_register_index(),
                            r3: src_register.to_register_index(),
                            condition_flag,
                            additional_flags: 0x00,
                        }),
                        symbol_ref: None,
                    })
                }
                [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register1), AddressingMode::DirectRegister(src_register2)] => {
                    Ok(InstructionToken {
                        instruction: InstructionData::Register(RegisterInstructionData {
                            op_code: tag_to_instruction(tag),
                            r1: dest_register.to_register_index(),
                            r2: src_register1.to_register_index(),
                            r3: src_register2.to_register_index(),
                            condition_flag,
                            additional_flags: 0x00,
                        }),
                        symbol_ref: None,
                    })
                }
                _ => {
                    let error_string = format!("The [{}] opcode only supports register->register or register->register->register addressing mode (e.g. ADDR y1, z2, a2 or SUBR y1, a2)", tag);
                    Err(nom::Err::Failure(ErrorTree::from_external_error(
                        i.to_owned(),
                        ErrorKind::Fail,
                        error_string.as_str(),
                    )))
                }
            }
        },
    )(i)
}
