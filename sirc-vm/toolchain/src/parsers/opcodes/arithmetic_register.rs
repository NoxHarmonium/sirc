use crate::parsers::instruction::{
    parse_instruction_operands1, parse_instruction_tag, AddressingMode, InstructionToken,
};
use crate::parsers::shared::split_shift_definition_data;
use nom::branch::alt;
use nom::error::{ErrorKind, FromExternalError};
use nom::sequence::tuple;
use nom_supreme::error::ErrorTree;
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    Instruction, InstructionData, RegisterInstructionData, ShiftOperand, ShiftType,
    StatusRegisterUpdateSource,
};

fn tag_to_instruction(tag: &str) -> Instruction {
    match tag {
        "ADDR" => Instruction::AddRegister,
        "ADCR" => Instruction::AddRegisterWithCarry,
        "SUBR" => Instruction::SubtractRegister,
        "SBCR" => Instruction::SubtractRegisterWithCarry,
        "ANDR" => Instruction::AndRegister,
        "ORRR" => Instruction::OrRegister,
        "XORR" => Instruction::XorRegister,
        "CMPR" => Instruction::CompareRegister,
        "TSAR" => Instruction::TestAndRegister,
        "TSXR" => Instruction::TestXorRegister,
        "COPR" => Instruction::CoprocessorCallRegister,
        _ => panic!("No tag mapping for instruction [{tag}]"),
    }
}

use super::super::shared::AsmResult;

///
/// Parses register arithmetic/logic opcodes
///
/// ```
/// use toolchain::parsers::opcodes::arithmetic_register::arithmetic_register;
/// use toolchain::parsers::instruction::InstructionToken;
/// use peripheral_cpu::coprocessors::processing_unit::definitions::{ConditionFlags, Instruction, InstructionData, RegisterInstructionData, ShiftType};
///
/// let (_, parsed_instruction) = arithmetic_register("ADDR|>= r1, r3, RTL #4\n").unwrap();
/// let (op_code, r1, r2, r3, shift_type, shift_count, condition_flag, additional_flags) = match parsed_instruction.instruction {
///     InstructionData::Register(inner) => (inner.op_code, inner.r1, inner.r2, inner.r3, inner.shift_type, inner.shift_count, inner.condition_flag, inner.additional_flags),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// // TODO: Make a helper function or something to make these asserts smaller
/// assert_eq!(op_code, Instruction::AddRegister);
/// assert_eq!(r1, 1);
/// assert_eq!(r2, 1);
/// assert_eq!(r3, 3);
/// assert_eq!(shift_type, ShiftType::RotateLeft);
/// assert_eq!(shift_count, 4);
/// assert_eq!(additional_flags, 0x1);
/// assert_eq!(condition_flag, ConditionFlags::GreaterOrEqual);
///
/// let (_, parsed_instruction) = arithmetic_register("ADDR|>= r1, r2, r3\n").unwrap();
/// let (op_code, r1, r2, r3, condition_flag, additional_flags) = match parsed_instruction.instruction {
///     InstructionData::Register(inner) => (inner.op_code, inner.r1, inner.r2, inner.r3, inner.condition_flag, inner.additional_flags),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// // TODO: Make a helper function or something to make these asserts smaller
/// assert_eq!(op_code, Instruction::AddRegister);
/// assert_eq!(r1, 1);
/// assert_eq!(r2, 2);
/// assert_eq!(r3, 3);
/// assert_eq!(additional_flags, 0x1);
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
        parse_instruction_tag("CMPR"),
        parse_instruction_tag("TSAR"),
        parse_instruction_tag("TSXR"),
        parse_instruction_tag("COPR"),
    ));

    let (i, ((tag, condition_flag), operands)) =
        tuple((instructions, parse_instruction_operands1))(i)?;

    match operands.as_slice() {
        // The CPU does not support register arithmetic instructions with two register operands
        // If third register is omitted, it is implied that the first operand is the destination register and the assembler fills that in
        [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register)] => {
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: tag_to_instruction(tag.as_str()),
                        r1: dest_register.to_register_index(),
                        r2: dest_register.to_register_index(),
                        r3: src_register.to_register_index(),
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                        condition_flag,
                        additional_flags: if &tag == "SHFR" {
                            StatusRegisterUpdateSource::Shift.to_flags()
                        } else {
                            StatusRegisterUpdateSource::Alu.to_flags()
                        },
                    }),
                    ..Default::default()
                },
            ))
        }
        [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register1), AddressingMode::DirectRegister(src_register2)] => {
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: tag_to_instruction(tag.as_str()),
                        r1: dest_register.to_register_index(),
                        r2: src_register1.to_register_index(),
                        r3: src_register2.to_register_index(),
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                        condition_flag,
                        additional_flags: if &tag == "SHFR" {
                            StatusRegisterUpdateSource::Shift.to_flags()
                        } else {
                            StatusRegisterUpdateSource::Alu.to_flags()
                        },
                    }),
                    ..Default::default()
                },
            ))
        }
        // Same as above but with shift definitions
        [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register), AddressingMode::ShiftDefinition(shift_defintion_data)] =>
        {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_defintion_data);

            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: tag_to_instruction(tag.as_str()),
                        r1: dest_register.to_register_index(),
                        r2: dest_register.to_register_index(),
                        r3: src_register.to_register_index(),
                        shift_operand,
                        shift_type,
                        shift_count,
                        condition_flag,
                        additional_flags: if &tag == "SHFR" {
                            StatusRegisterUpdateSource::Shift.to_flags()
                        } else {
                            StatusRegisterUpdateSource::Alu.to_flags()
                        },
                    }),
                    ..Default::default()
                },
            ))
        }
        [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register1), AddressingMode::DirectRegister(src_register2), AddressingMode::ShiftDefinition(shift_definition_data)] =>
        {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);

            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: tag_to_instruction(tag.as_str()),
                        r1: dest_register.to_register_index(),
                        r2: src_register1.to_register_index(),
                        r3: src_register2.to_register_index(),
                        shift_operand,
                        shift_type,
                        shift_count,
                        condition_flag,
                        additional_flags: if &tag == "SHFR" {
                            StatusRegisterUpdateSource::Shift.to_flags()
                        } else {
                            StatusRegisterUpdateSource::Alu.to_flags()
                        },
                    }),
                    ..Default::default()
                },
            ))
        }
        _ => {
            let error_string = format!("The [{tag}] opcode only supports register->register or register->register->register addressing mode (e.g. ADDR y1, z2, a2 or SUBR y1, a2)");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}
