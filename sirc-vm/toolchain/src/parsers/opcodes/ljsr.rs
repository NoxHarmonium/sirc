use super::super::shared::AsmResult;
use crate::parsers::data::override_ref_token_type_if_implied;
use crate::parsers::instruction::{
    parse_instruction_operands1, parse_instruction_tag, AddressingMode, ImmediateType,
};
use crate::parsers::shared::split_shift_definition_data;
use crate::types::instruction::InstructionToken;
use crate::types::object::RefType;
use nom::error::{ErrorKind, FromExternalError};
use nom::sequence::tuple;
use nom_supreme::error::ErrorTree;
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData, ShiftOperand,
    ShiftType,
};
use peripheral_cpu::registers::AddressRegisterName;

///
/// Parses a long jump to subroutine instruction
///
/// ```
/// use toolchain::parsers::opcodes::ljsr::ljsr;
/// use toolchain::types::instruction::InstructionToken;
/// use peripheral_cpu::coprocessors::processing_unit::definitions::{ConditionFlags, Instruction, InstructionData, RegisterInstructionData};
/// use nom_supreme::error::ErrorTree;
/// use nom_supreme::final_parser::{final_parser, Location};
///
/// let parsed_instruction = match final_parser::<&str, InstructionToken, ErrorTree<&str>, ErrorTree<Location>>(ljsr)("LJSR|!= (#-4, a)\n") {
///   Ok(tokens) => tokens,
///   Err(error) => panic!("Error parsing instruction:\n{}", error),
/// };
/// let (op_code, register, value, condition_flag, additional_flags) = match parsed_instruction.instruction {
///     InstructionData::Immediate(inner) => (inner.op_code, inner.register, inner.value, inner.condition_flag, inner.additional_flags),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// assert_eq!(op_code, Instruction::LongJumpToSubroutineWithImmediateDisplacement);
/// assert_eq!(register, 0x0);
/// assert_eq!(value, 0xFFFC);
/// assert_eq!(condition_flag, ConditionFlags::NotEqual);
/// assert_eq!(additional_flags, 1);
/// ```
pub fn ljsr(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();
    let (i, ((_, condition_flag), operands)) =
        tuple((parse_instruction_tag("LJSR"), parse_instruction_operands1))(i)?;

    let construct_immediate_instruction = |offset: u16, address_register: &AddressRegisterName| {
        InstructionData::Immediate(ImmediateInstructionData {
            op_code: Instruction::LongJumpToSubroutineWithImmediateDisplacement,
            register: 0x0, // unused
            value: offset,
            condition_flag,
            additional_flags: address_register.to_register_index(),
        })
    };

    match operands.as_slice() {
        [AddressingMode::IndirectImmediateDisplacement(offset, address_register)] => match offset {
            ImmediateType::Value(offset) => Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: construct_immediate_instruction(
                        offset.to_owned(),
                        address_register,
                    ),
                    ..Default::default()
                },
            )),
            ImmediateType::SymbolRef(ref_token) => Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: construct_immediate_instruction(0x0, address_register),
                    symbol_ref: Some(override_ref_token_type_if_implied(
                        ref_token,
                        RefType::LowerWord,
                    )),
                    ..Default::default()
                },
            )),
            ImmediateType::PlaceHolder(placeholder_name) => Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: construct_immediate_instruction(0x0, address_register),
                    placeholder_name: Some(placeholder_name.clone()),
                    ..Default::default()
                },
            )),
        },
        [AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register)] => {
            Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LongJumpToSubroutineWithRegisterDisplacement,
                        r1: displacement_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: 0x0, // Unused
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                        condition_flag,
                        additional_flags: address_register.to_register_index(),
                    }),
                    ..Default::default()
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
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LongJumpToSubroutineWithRegisterDisplacement,
                        r1: displacement_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: 0x0, // Unused
                        shift_operand,
                        shift_type,
                        shift_count,
                        condition_flag,
                        additional_flags: address_register.to_register_index(),
                    }),
                    ..Default::default()
                },
            ))
        }
        modes => {
            let error_string = format!("Invalid addressing mode for LJSR: ({modes:?})");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}
