use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, AddressingMode, ImmediateType,
    InstructionToken,
};
use nom::error::{ErrorKind, FromExternalError};
use nom::sequence::tuple;
use nom::{branch::alt, combinator::map_res};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, Instruction, InstructionData,
};

fn tag_to_instruction(tag: String) -> Instruction {
    match tag.as_str() {
        "ADDI" => Instruction::AddImmediate,
        "ADCI" => Instruction::AddImmediateWithCarry,
        "SUBI" => Instruction::SubtractImmediate,
        "SBCI" => Instruction::SubtractImmediateWithCarry,
        "ANDI" => Instruction::AndImmediate,
        "ORRI" => Instruction::OrImmediate,
        "XORI" => Instruction::XorImmediate,
        "LSLI" => Instruction::LogicalShiftLeftImmediate,
        "LSRI" => Instruction::LogicalShiftRightImmediate,
        "ASLI" => Instruction::ArithmeticShiftLeftImmediate,
        "ASRI" => Instruction::ArithmeticShiftRightImmediate,
        "RTLI" => Instruction::RotateLeftImmediate,
        "RTRI" => Instruction::RotateRightImmediate,
        "CMPI" => Instruction::CompareImmediate,

        _ => panic!("No tag mapping for instruction [{}]", tag),
    }
}

use super::super::shared::AsmResult;

///
/// Parses immediate arithmetic/logic opcodes
///
/// ```
/// use toolchain::parsers::opcodes::arithmetic_immediate::arithmetic_immediate;
/// use toolchain::parsers::instruction::InstructionToken;
/// use peripheral_cpu::instructions::definitions::{ConditionFlags, Instruction, InstructionData, ImmediateInstructionData};
///
///
/// let (_, parsed_instruction) = arithmetic_immediate("ADDI|!= r2, #123").unwrap();
/// let (op_code, register, value, condition_flag, additional_flags) = match parsed_instruction.instruction {
///     InstructionData::Immediate(inner) => (inner.op_code, inner.register, inner.value, inner.condition_flag, inner.additional_flags),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// // TODO: Make a helper function or something to make these asserts smaller
/// assert_eq!(op_code, Instruction::AddImmediate);
/// assert_eq!(register, 1);
/// assert_eq!(value, 123);
/// assert_eq!(additional_flags, 0x0);
/// assert_eq!(condition_flag, ConditionFlags::NotEqual);
///
/// ```
pub fn arithmetic_immediate(i: &str) -> AsmResult<InstructionToken> {
    let instructions = alt((
        parse_instruction_tag("ADDI"),
        parse_instruction_tag("ADCI"),
        parse_instruction_tag("SUBI"),
        parse_instruction_tag("SBCI"),
        parse_instruction_tag("ANDI"),
        parse_instruction_tag("ORRI"),
        parse_instruction_tag("XORI"),
        parse_instruction_tag("LSLI"),
        parse_instruction_tag("LSRI"),
        parse_instruction_tag("ASLI"),
        parse_instruction_tag("ASRI"),
        parse_instruction_tag("RTLI"),
        parse_instruction_tag("RTRI"),
        parse_instruction_tag("CMPI"),
        // parse_instruction_tag("SJMP"),
        // parse_instruction_tag("SJSR"),
    ));

    map_res(
        tuple((instructions, parse_instruction_operands)),
        |((tag, condition_flag), operands)| match operands.as_slice() {
            [AddressingMode::DirectRegister(dest_register), AddressingMode::Immediate(immediate_type)] => {
                match immediate_type {
                    ImmediateType::Value(value) => Ok(InstructionToken {
                        instruction: InstructionData::Immediate(ImmediateInstructionData {
                            op_code: tag_to_instruction(tag),
                            register: dest_register.to_register_index(),
                            value: *value,
                            condition_flag,
                            additional_flags: 0x00,
                        }),
                        symbol_ref: None,
                    }),
                    _ => {
                        let error_string = format!(
                            "The [{}] opcode does not support symbol refs at this time",
                            tag
                        );
                        Err(nom::Err::Failure(ErrorTree::from_external_error(
                            i.to_owned(),
                            ErrorKind::Fail,
                            error_string.as_str(),
                        )))
                    }
                }
            }
            _ => {
                let error_string = format!("The [{}] opcode only supports immediate->register addressing mode (e.g. ADDI y1, #1)", tag);
                Err(nom::Err::Failure(ErrorTree::from_external_error(
                    i.to_owned(),
                    ErrorKind::Fail,
                    error_string.as_str(),
                )))
            }
        },
    )(i)
}
