use crate::parsers::instruction::{
    override_ref_token_type_if_implied, parse_instruction_operands1, parse_instruction_tag,
    AddressingMode, ImmediateType, InstructionToken,
};
use crate::types::object::RefType;
use nom::error::{ErrorKind, FromExternalError};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData,
};

use super::super::shared::AsmResult;

///
/// Parses a long jump to subroutine instruction
///
/// ```
/// use toolchain::parsers::opcodes::ljsr::ljsr;
/// use toolchain::parsers::instruction::InstructionToken;
/// use peripheral_cpu::instructions::definitions::{ConditionFlags, Instruction, InstructionData, RegisterInstructionData};
/// use nom_supreme::error::ErrorTree;
/// use nom_supreme::final_parser::{final_parser, Location};
///
/// let parsed_instruction = match final_parser::<&str, InstructionToken, ErrorTree<&str>, ErrorTree<Location>>(ljsr)("LJSR|!= (#-4, a)") {
///   Ok(tokens) => tokens,
///   Err(error) => panic!("Error parsing instruction:\n{}", error),
/// };
/// let (op_code, register, value, condition_flag, additional_flags) = match parsed_instruction.instruction {
///     InstructionData::Immediate(inner) => (inner.op_code, inner.register, inner.value, inner.condition_flag, inner.additional_flags),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// // TODO: Make a helper function or something to make these asserts smaller
/// assert_eq!(op_code, Instruction::LongJumpToSubroutineWithImmediateDisplacement);
/// assert_eq!(register, 0x0);
/// assert_eq!(value, 0xFFFC);
/// assert_eq!(condition_flag, ConditionFlags::NotEqual);
/// assert_eq!(additional_flags, 1);
/// ```
pub fn ljsr(i: &str) -> AsmResult<InstructionToken> {
    let (i, (_, condition_flag)) = parse_instruction_tag("LJSR")(i)?;
    let (i, operands) = parse_instruction_operands1(i)?;

    match operands.as_slice() {
        [AddressingMode::IndirectImmediateDisplacement(offset, address_register)] => {
            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        instruction: InstructionData::Immediate(ImmediateInstructionData {
                            op_code: Instruction::LongJumpToSubroutineWithImmediateDisplacement,
                            register: 0x0, // unused
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
                            op_code: Instruction::LongJumpToSubroutineWithImmediateDisplacement,
                            register: 0x0, // unused
                            value: 0x0,    // Placeholder
                            condition_flag,
                            additional_flags: address_register.to_register_index(),
                        }),
                        symbol_ref: Some(override_ref_token_type_if_implied(
                            ref_token,
                            RefType::LowerByte,
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
                        op_code: Instruction::LongJumpToSubroutineWithRegisterDisplacement,
                        r1: displacement_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: 0x0, // Unused
                        condition_flag,
                        additional_flags: address_register.to_register_index(),
                    }),
                    symbol_ref: None,
                },
            ))
        }
        modes => {
            let error_string = format!("Invalid addressing mode for LJSR: ({:?})", modes);
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}
