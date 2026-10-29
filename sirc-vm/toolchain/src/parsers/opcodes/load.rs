use super::super::shared::AsmResult;
use super::{direct_register_aliases_address_register, reject_aliased_address_register_write};
use crate::parsers::shared::split_shift_definition_data;
use crate::types::instruction::InstructionToken;
use crate::{
    parsers::{
        data::override_ref_token_type_if_implied,
        instruction::{
            parse_instruction_operands0, parse_instruction_tag, AddressingMode, ImmediateType,
        },
    },
    types::object::RefType,
};
use nom::error::{ErrorKind, FromExternalError};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData,
        ShiftOperand, ShiftType,
    },
    registers::{AddressRegisterName, RegisterName},
};
pub fn load(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();

    let (i_after_instruction, (_, condition_flag, status_register_update_source)) =
        parse_instruction_tag("LOAD")(i)?;

    if status_register_update_source.is_some() {
        let error_string =
            "The [LOAD] opcode does not support an explicit status register update source. Only ALU instructions can update the status register as a side-effect.";
        return Err(nom::Err::Failure(ErrorTree::from_external_error(
            i_after_instruction,
            ErrorKind::Fail,
            error_string,
        )));
    }

    let (i, operands) = parse_instruction_operands0(i_after_instruction)?;

    let construct_indirect_immediate_instruction =
        |offset: u16, dest_register: &RegisterName, address_register: &AddressRegisterName| {
            InstructionData::Immediate(ImmediateInstructionData {
                op_code: Instruction::LoadRegisterFromIndirectImmediate,
                register: dest_register.to_register_index(),
                value: offset.to_owned(),
                condition_flag,
                additional_flags: address_register.to_register_index(),
            })
        };

    let construct_indirect_immediate_post_increment_instruction =
        |offset: u16, dest_register: &RegisterName, address_register: &AddressRegisterName| {
            InstructionData::Immediate(ImmediateInstructionData {
                op_code: Instruction::LoadRegisterFromIndirectImmediatePostIncrement,
                register: dest_register.to_register_index(),
                value: offset,
                condition_flag,
                additional_flags: address_register.to_register_index(),
            })
        };

    match operands.as_slice() {
        // LOAD r1, (#0, a)
        [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectImmediateDisplacement(offset, address_register)] => {
            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: construct_indirect_immediate_instruction(
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
                        instruction: construct_indirect_immediate_instruction(
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
                        instruction: construct_indirect_immediate_instruction(
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
        // LOAD r1, (r2, a)
        [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register)] =>
        {
            Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromIndirectRegister,
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
        // LOAD r1, (r2, a), ASL #1
        [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectRegisterDisplacement(displacement_register, address_register), AddressingMode::ShiftDefinition(shift_definition_data)] =>
        {
            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromIndirectRegister,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: displacement_register.to_register_index(),
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
        // LOAD r1, (#0, a)+
        [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectImmediateDisplacementPostIncrement(offset, address_register)] =>
        {
            if direct_register_aliases_address_register(dest_register, address_register) {
                reject_aliased_address_register_write(
                    i_after_instruction,
                    "LOAD",
                    "destination register overlaps the post-increment address register",
                )?;
            }

            match offset {
                ImmediateType::Value(offset) => Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: construct_indirect_immediate_post_increment_instruction(
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
                        instruction: construct_indirect_immediate_post_increment_instruction(
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
                        instruction: construct_indirect_immediate_post_increment_instruction(
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
        // LOAD r1, (r2, a)+
        [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectRegisterDisplacementPostIncrement(
            displacement_register,
            address_register,
        ), AddressingMode::ShiftDefinition(shift_definition_data)] => {
            if direct_register_aliases_address_register(dest_register, address_register) {
                reject_aliased_address_register_write(
                    i_after_instruction,
                    "LOAD",
                    "destination register overlaps the post-increment address register",
                )?;
            }

            let (shift_operand, shift_type, shift_count) =
                split_shift_definition_data(shift_definition_data);
            Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromIndirectRegisterPostIncrement,
                        r1: dest_register.to_register_index(),
                        r2: 0x0, // Unused
                        r3: displacement_register.to_register_index(),
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
        // LOAD r1, (r2, a)+, ASL #1
        [AddressingMode::DirectRegister(dest_register), AddressingMode::IndirectRegisterDisplacementPostIncrement(
            displacement_register,
            address_register,
        )] => {
            if direct_register_aliases_address_register(dest_register, address_register) {
                reject_aliased_address_register_write(
                    i_after_instruction,
                    "LOAD",
                    "destination register overlaps the post-increment address register",
                )?;
            }

            Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromIndirectRegisterPostIncrement,
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
        // LOAD only accesses memory. Loading an immediate or copying a register is done with MOVI/MOVR
        [AddressingMode::DirectRegister(_), AddressingMode::Immediate(_)] => Err(nom::Err::Failure(
            ErrorTree::from_external_error(
                i_after_instruction,
                ErrorKind::Fail,
                "Invalid addressing mode for LOAD: LOAD only reads memory. Use MOVI to load an immediate value into a register.",
            ),
        )),
        [AddressingMode::DirectRegister(_), AddressingMode::DirectRegister(_)] => Err(nom::Err::Failure(
            ErrorTree::from_external_error(
                i_after_instruction,
                ErrorKind::Fail,
                "Invalid addressing mode for LOAD: LOAD only reads memory. Use MOVR to copy one register to another.",
            ),
        )),
        modes => {
            let error_string = format!("Invalid addressing mode for LOAD: ({modes:?})");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i_after_instruction,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}

//
