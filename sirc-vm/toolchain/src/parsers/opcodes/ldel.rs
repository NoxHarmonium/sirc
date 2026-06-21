use super::super::shared::AsmResult;
use crate::parsers::data::override_ref_token_type_if_implied;
use crate::parsers::instruction::{
    parse_instruction_operands0, parse_instruction_tag, AddressingMode, ImmediateType,
};
use crate::types::instruction::InstructionToken;
use crate::types::object::RefType;
use nom::error::{ErrorKind, FromExternalError};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData, ShiftOperand,
    ShiftType,
};
use peripheral_cpu::registers::AddressRegisterName;

pub fn ldel(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();
    let (i_after_instruction, (_, condition_flag, status_register_update_source)) =
        parse_instruction_tag("LDEL")(i)?;

    let (i, operands) = parse_instruction_operands0(i_after_instruction)?;

    if status_register_update_source.is_some() {
        let error_string =
            "The [LDEL] opcode does not support an explicit status register update source. Only ALU instructions can update the status register as a side-effect.";
        return Err(nom::Err::Failure(ErrorTree::from_external_error(
            i_after_instruction,
            ErrorKind::Fail,
            error_string,
        )));
    }

    let construct_immediate_instruction =
        |op_code: Instruction,
         offset: u16,
         dest_register: &AddressRegisterName,
         address_register: &AddressRegisterName| {
            InstructionData::Immediate(ImmediateInstructionData {
                op_code,
                register: dest_register.to_register_index(),
                value: offset,
                condition_flag,
                additional_flags: address_register.to_register_index(),
            })
        };

    match operands.as_slice() {
        [AddressingMode::DirectAddressRegister(dest_register), AddressingMode::IndirectImmediateDisplacement(offset, address_register)] => {
            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: construct_immediate_instruction(
                            Instruction::LongJumpToSubroutineWithImmediateDisplacement,
                            offset.to_owned(),
                            dest_register,
                            address_register,
                        ),
                        ..Default::default()
                    },
                )),
                ImmediateType::SymbolRef(ref_token) => Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: construct_immediate_instruction(
                            Instruction::LongJumpToSubroutineWithImmediateDisplacement,
                            0x0,
                            dest_register,
                            address_register,
                        ),
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
                        instruction: construct_immediate_instruction(
                            Instruction::LongJumpToSubroutineWithImmediateDisplacement,
                            0x0,
                            dest_register,
                            address_register,
                        ),
                        placeholder_name: Some(placeholder_name.clone()),
                        ..Default::default()
                    },
                )),
            }
        }
        [AddressingMode::DirectAddressRegister(dest_register), AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register)] =>
        {
            Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LongJumpToSubroutineWithRegisterDisplacement,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: displacement_register.to_register_index(),
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
        [AddressingMode::DirectAddressRegister(dest_register), AddressingMode::IndirectImmediateDisplacementPostIncrement(offset, address_register)] =>
        {
            // TODO: Reject aliased register writes once operand validation is centralised.
            // These forms are architecturally undefined.
            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: construct_immediate_instruction(
                            Instruction::BranchToSubroutineWithImmediateDisplacement,
                            offset.to_owned(),
                            dest_register,
                            address_register,
                        ),
                        ..Default::default()
                    },
                )),
                ImmediateType::SymbolRef(ref_token) => Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: construct_immediate_instruction(
                            Instruction::BranchToSubroutineWithImmediateDisplacement,
                            0x0,
                            dest_register,
                            address_register,
                        ),
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
                        instruction: construct_immediate_instruction(
                            Instruction::BranchToSubroutineWithImmediateDisplacement,
                            0x0,
                            dest_register,
                            address_register,
                        ),
                        placeholder_name: Some(placeholder_name.clone()),
                        ..Default::default()
                    },
                )),
            }
        }
        [AddressingMode::DirectAddressRegister(dest_register), AddressingMode::IndirectRegisterDisplacementPostIncrement(
            displacement_register,
            address_register,
        )] => {
            // TODO: Reject aliased register writes once operand validation is centralised.
            // These forms are architecturally undefined.
            Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::BranchToSubroutineWithRegisterDisplacement,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: displacement_register.to_register_index(),
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
        modes => {
            let error_string = format!("Invalid addressing mode for LDEL: ({modes:?})");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i_after_instruction,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}
