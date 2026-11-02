use super::super::shared::AsmResult;
use crate::parsers::{
    data::override_ref_token_type_if_implied,
    instruction::{
        parse_instruction_operands0, parse_instruction_tag, AddressingMode, ImmediateType,
    },
};
use crate::types::instruction::InstructionToken;
use crate::types::numeric::{NumericField, NumericPlaceholder};
use crate::types::object::RefType;
use nom::branch::alt;
use nom::error::{ErrorKind, FromExternalError};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData, ShiftOperand,
    ShiftType, StatusRegisterUpdateSource,
};

///
/// Parses the ALU move instructions: `MOVI` loads an immediate value into a register and `MOVR`
/// copies one register to another.
///
/// Like the other ALU instructions they update the status register from the ALU result (`[A]`)
/// unless `[N]` is given. `[S]` is rejected because there is no shifted operand.
///
/// ```
/// use toolchain::parsers::opcodes::mov::mov;
/// use peripheral_cpu::coprocessors::processing_unit::definitions::{ConditionFlags, Instruction, InstructionData};
///
/// let (_, parsed_instruction) = mov("MOVI|== r2, #0x1234\n").unwrap();
/// let (op_code, register, value, condition_flag) = match parsed_instruction.instruction {
///     InstructionData::Immediate(inner) => (inner.op_code, inner.register, inner.value, inner.condition_flag),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// assert_eq!(op_code, Instruction::MoveRegisterFromImmediate);
/// assert_eq!(register, 1); // r2
/// assert_eq!(value, 0x1234);
/// assert_eq!(condition_flag, ConditionFlags::Equal);
///
/// let (_, parsed_instruction) = mov("MOVI[N] r2, #0x1234\n").unwrap();
/// let additional_flags = match parsed_instruction.instruction {
///     InstructionData::Immediate(inner) => inner.additional_flags,
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// assert_eq!(additional_flags, 0x0); // [N] preserves the status register
///
/// let (_, parsed_instruction) = mov("MOVR r1, r3\n").unwrap();
/// let (op_code, r1, r3) = match parsed_instruction.instruction {
///     InstructionData::Register(inner) => (inner.op_code, inner.r1, inner.r3),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// assert_eq!(op_code, Instruction::MoveRegisterFromRegister);
/// assert_eq!(r1, 0); // r1
/// assert_eq!(r3, 2); // r3
/// ```
pub fn mov(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();

    let mut instructions = alt((parse_instruction_tag("MOVI"), parse_instruction_tag("MOVR")));

    let (i_after_instruction, (tag, condition_flag, status_register_update_source)) =
        instructions(i)?;

    // There is no shifted operand to take flags from, so only [A] and [N] are meaningful
    if status_register_update_source == Some(StatusRegisterUpdateSource::Shift) {
        let error_string = format!(
            "The [{tag}] opcode does not support the [S] status register update source because it has no shift operand. Use [A] or [N]."
        );
        return Err(nom::Err::Failure(ErrorTree::from_external_error(
            i_after_instruction,
            ErrorKind::Fail,
            error_string.as_str(),
        )));
    }

    let (i, operands) = parse_instruction_operands0(i_after_instruction)?;

    let additional_flags = status_register_update_source
        .unwrap_or(StatusRegisterUpdateSource::Alu)
        .to_flags();

    let construct_immediate_instruction = |value: u16, dest_register_index: u8| {
        InstructionData::Immediate(ImmediateInstructionData {
            op_code: Instruction::MoveRegisterFromImmediate,
            register: dest_register_index,
            value,
            condition_flag,
            additional_flags,
        })
    };

    // NOTE: No shifting with immediate operands because there is no short immediate representation of MOVI
    match (tag.as_str(), operands.as_slice()) {
        // MOVI r1, #0
        (
            "MOVI",
            [AddressingMode::DirectRegister(dest_register), AddressingMode::Immediate(offset)],
        ) => {
            let dest_register_index = dest_register.to_register_index();
            match offset {
                ImmediateType::Value(value) => Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: construct_immediate_instruction(*value, dest_register_index),
                        ..Default::default()
                    },
                )),
                ImmediateType::SymbolRef(ref_token) => Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: construct_immediate_instruction(0x0, dest_register_index),
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
                        instruction: construct_immediate_instruction(0x0, dest_register_index),
                        numeric_placeholders: vec![NumericPlaceholder::new(
                            placeholder_name.clone(),
                            NumericField::Immediate16,
                        )],
                        ..Default::default()
                    },
                )),
            }
        }
        // MOVR r1, r2
        (
            "MOVR",
            [AddressingMode::DirectRegister(dest_register), AddressingMode::DirectRegister(src_register)],
        ) => {
            Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::MoveRegisterFromRegister,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: src_register.to_register_index(),
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                        condition_flag,
                        additional_flags,
                    }),
                    ..Default::default()
                },
            ))
        }
        // MOVR r1, r2, ASL #1
        (
            "MOVR",
            [AddressingMode::DirectRegister(_), AddressingMode::DirectRegister(_), AddressingMode::ShiftDefinition(_)],
        ) => {
            // NOTE: No shifting with direct register -> direct register because of the way the CPU architecture works - use SHFT instead
            // MOVR r1, r2, ASL #1 would translate directly to SHFT r1, r2, ASL #1.
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i_after_instruction,
                ErrorKind::Fail,
                "Invalid addressing mode for MOVR:. Cannot use a shift with direct register -> direct register moves. Use SHFT instruction instead.",
            )))
        }
        (tag, modes) => {
            let error_string = format!("Invalid addressing mode for {tag}: ({modes:?})");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i_after_instruction,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}
