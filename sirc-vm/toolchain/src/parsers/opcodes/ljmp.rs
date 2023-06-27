use crate::parsers::instruction::{
    override_ref_token_type_if_implied, parse_instruction_operands1, parse_instruction_tag,
    AddressingMode, ImmediateType, InstructionToken,
};
use crate::parsers::shared::split_shift_definition_data;
use crate::types::object::RefType;
use nom::error::{ErrorKind, FromExternalError};
use nom::sequence::tuple;
use nom_supreme::error::ErrorTree;
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData, ShiftOperand,
    ShiftType,
};

use super::super::shared::AsmResult;

///
/// Parses a long jump instruction
///
/// ```
/// use toolchain::parsers::opcodes::ljmp::ljmp;
/// use toolchain::parsers::instruction::InstructionToken;
/// use peripheral_cpu::instructions::definitions::{ConditionFlags, Instruction, InstructionData, RegisterInstructionData, ShiftType};
/// use nom_supreme::error::ErrorTree;
/// use nom_supreme::final_parser::{final_parser, Location};
///
/// let parsed_instruction = match final_parser::<&str, InstructionToken, ErrorTree<&str>, ErrorTree<Location>>(ljmp)("LJMP|!= (r2, a), ASR #3 ") {
///   Ok(tokens) => tokens,
///   Err(error) => panic!("Error parsing instruction:\n{}", error),
/// };
/// let (op_code, r1, r2, r3, shift_type, shift_count, condition_flag, additional_flags) = match parsed_instruction.instruction {
///     InstructionData::Register(inner) => (inner.op_code, inner.r1, inner.r2, inner.r3, inner.shift_type, inner.shift_count, inner.condition_flag, inner.additional_flags),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// // TODO: Make a helper function or something to make these asserts smaller
/// assert_eq!(op_code, Instruction::LongJumpWithRegisterDisplacement);
/// assert_eq!(r1, 0x01);
/// assert_eq!(r2, 0x00);
/// assert_eq!(r3, 0x00);
/// assert_eq!(condition_flag, ConditionFlags::NotEqual);
/// assert_eq!(shift_type, ShiftType::ArithmeticRightShift);
/// assert_eq!(shift_count, 3);
/// assert_eq!(additional_flags, 1);
/// ```
pub fn ljmp(i: &str) -> AsmResult<InstructionToken> {
    let (i, ((_, condition_flag), operands)) =
        tuple((parse_instruction_tag("LJMP"), parse_instruction_operands1))(i)?;

    match operands.as_slice() {
        // TODO: It is confusing that a LJMP uses the indirect address syntax when it isn't indirect
        // It isn't fetching anything from memory, it is just using the value in the register which means its direct and
        // the brackets are confusing because they imply indirectness.
        // I'll let it slide for now in the interest of getting something working but I should revisit this
        [AddressingMode::IndirectImmediateDisplacement(offset, address_register)] => {
            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        instruction: InstructionData::Immediate(ImmediateInstructionData {
                            op_code: Instruction::LongJumpWithImmediateDisplacement,
                            register: 0x0, // Unused
                            value: offset.to_owned(),
                            condition_flag,
                            additional_flags: address_register.to_register_index(),
                        }),
                        symbol_ref: None,
                    },
                )),
                ImmediateType::SymbolRef(ref_token) => Ok((
                    i,
                    InstructionToken {
                        instruction: InstructionData::Immediate(ImmediateInstructionData {
                            op_code: Instruction::LongJumpWithImmediateDisplacement,
                            register: 0x0, // Unused
                            value: 0x0,    // Placeholder
                            condition_flag,
                            additional_flags: address_register.to_register_index(),
                        }),
                        symbol_ref: Some(override_ref_token_type_if_implied(
                            ref_token,
                            RefType::LowerWord,
                        )),
                    },
                )),
            }
        }
        [AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register)] => {
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LongJumpWithRegisterDisplacement,
                        r1: displacement_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: 0x0, // Unused
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                        condition_flag,
                        additional_flags: address_register.to_register_index(),
                    }),
                    symbol_ref: None,
                },
            ))
        }
        [AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register), AddressingMode::ShiftDefinition(shift_definition_data)] =>
        {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            Ok((
                i,
                InstructionToken {
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LongJumpWithRegisterDisplacement,
                        r1: displacement_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: 0x0, // Unused
                        shift_operand,
                        shift_type,
                        shift_count,
                        condition_flag,
                        additional_flags: address_register.to_register_index(),
                    }),
                    symbol_ref: None,
                },
            ))
        }
        modes => {
            let error_string = format!("Invalid addressing mode for LJMP: ({:?})", modes);
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}
