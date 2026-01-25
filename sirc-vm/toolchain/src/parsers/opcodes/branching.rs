use crate::types::instruction::InstructionToken;
use crate::{
    parsers::{
        data::override_ref_token_type_if_implied,
        instruction::{
            parse_instruction_operands1, parse_instruction_tag, AddressingMode, ImmediateType,
        },
        shared::split_shift_definition_data,
    },
    types::object::RefType,
};
use nom::{branch::alt, error::ErrorKind};
use nom::{error::FromExternalError, sequence::tuple};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData,
        ShiftOperand, ShiftType,
    },
    registers::AddressRegisterName,
};

fn tag_to_instruction_long_immediate(tag: &str) -> Instruction {
    match tag {
        "BRAN" => Instruction::BranchWithImmediateDisplacement,
        "BRSR" => Instruction::BranchToSubroutineWithImmediateDisplacement,
        _ => panic!("No tag mapping for instruction [{tag}]"),
    }
}

fn tag_to_instruction_long_register(tag: &str) -> Instruction {
    match tag {
        "BRAN" => Instruction::BranchWithRegisterDisplacement,
        "BRSR" => Instruction::BranchToSubroutineWithRegisterDisplacement,
        _ => panic!("No tag mapping for instruction [{tag}]"),
    }
}

use super::super::shared::AsmResult;

///
/// Parses a long jump meta instruction (LDEA with p implied as the destination)
///
/// ```
/// use toolchain::parsers::opcodes::branching::branching;
/// use toolchain::types::instruction::InstructionToken;
/// use peripheral_cpu::coprocessors::processing_unit::definitions::{ConditionFlags, Instruction, InstructionData, RegisterInstructionData, ShiftType};
/// use nom_supreme::error::ErrorTree;
/// use nom_supreme::final_parser::{final_parser, Location};
///
/// let parsed_instruction = match final_parser::<&str, InstructionToken, ErrorTree<&str>, ErrorTree<Location>>(branching)("BRAN|!= (r2, a), ASR #3\n") {
///   Ok(tokens) => tokens,
///   Err(error) => panic!("Error parsing instruction:\n{}", error),
/// };
/// let (op_code, r1, r2, r3, shift_type, shift_count, condition_flag, additional_flags) = match parsed_instruction.instruction {
///     InstructionData::Register(inner) => (inner.op_code, inner.r1, inner.r2, inner.r3, inner.shift_type, inner.shift_count, inner.condition_flag, inner.additional_flags),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// assert_eq!(op_code, Instruction::BranchWithRegisterDisplacement);
/// assert_eq!(r1, 0x03);
/// assert_eq!(r2, 0x03);
/// assert_eq!(r3, 0x02);
/// assert_eq!(condition_flag, ConditionFlags::NotEqual);
/// assert_eq!(shift_type, ShiftType::ArithmeticRightShift);
/// assert_eq!(shift_count, 3);
/// assert_eq!(additional_flags, 1);
/// ```
pub fn branching(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();
    let instructions = alt((parse_instruction_tag("BRAN"), parse_instruction_tag("BRSR")));

    let (i, ((tag, condition_flag, status_register_update_source), operands)) =
        tuple((instructions, parse_instruction_operands1))(i)?;

    if status_register_update_source.is_some() {
        let error_string =
            format!("The [{tag}] opcode does not support an explicit status register update source. Only ALU instructions can update the status register as a side-effect.");
        return Err(nom::Err::Failure(ErrorTree::from_external_error(
            i,
            ErrorKind::Fail,
            error_string.as_str(),
        )));
    }

    let construct_branch_immediate_instruction = |value: u16| {
        InstructionData::Immediate(ImmediateInstructionData {
            op_code: tag_to_instruction_long_immediate(tag.as_str()),
            register: AddressRegisterName::ProgramCounter.to_register_index(),
            value,
            condition_flag,
            additional_flags: AddressRegisterName::ProgramCounter.to_register_index(),
        })
    };

    let construct_branch_indirect_immediate_instruction =
        |value: u16, address_register: &AddressRegisterName| {
            InstructionData::Immediate(ImmediateInstructionData {
                op_code: tag_to_instruction_long_immediate(tag.as_str()),
                register: AddressRegisterName::ProgramCounter.to_register_index(),
                value,
                condition_flag,
                additional_flags: address_register.to_register_index(),
            })
        };

    match operands.as_slice() {
        [AddressingMode::Immediate(offset)] => match offset {
            // Shorthand, always immediate offset to PC. Can also use other address registers with full syntax below
            ImmediateType::Value(offset) => Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: construct_branch_immediate_instruction(offset.to_owned()),
                    ..Default::default()
                },
            )),
            ImmediateType::SymbolRef(ref_token) => Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: construct_branch_immediate_instruction(0x0), // 0x0 will be replaced by linker
                    symbol_ref: Some(override_ref_token_type_if_implied(
                        ref_token,
                        RefType::Offset,
                    )),
                    ..Default::default()
                },
            )),
            ImmediateType::PlaceHolder(placeholder_name) => Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: construct_branch_immediate_instruction(0x0), // 0x0 will be replaced by linker
                    placeholder_name: Some(placeholder_name.clone()),
                    ..Default::default()
                },
            )),
        },
        [AddressingMode::IndirectImmediateDisplacement(offset, address_register)] => match offset {
            ImmediateType::Value(offset) => Ok((
                i,
                InstructionToken {
                    input_length,
                    instruction: construct_branch_indirect_immediate_instruction(
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
                    instruction: construct_branch_indirect_immediate_instruction(
                        0x0,
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
                    instruction: construct_branch_indirect_immediate_instruction(
                        0x0,
                        address_register,
                    ),
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
                        op_code: tag_to_instruction_long_register(tag.as_str()),
                        r1: AddressRegisterName::ProgramCounter.to_register_index(),
                        r2: AddressRegisterName::ProgramCounter.to_register_index(),
                        r3: displacement_register.to_register_index(),
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                        condition_flag,
                        // TODO: Clamp/validate additional_flags to 10 bits in parser
                        // category=Toolchain
                        // See also under IndirectRegisterDisplacement branch below
                        // See also the load/store parsers (probably safer to just do a search)
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
                        op_code: tag_to_instruction_long_register(tag.as_str()),
                        r1: AddressRegisterName::ProgramCounter.to_register_index(),
                        r2: AddressRegisterName::ProgramCounter.to_register_index(),
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
        _ => {
            let error_string = format!(
                "The [{tag}] opcode only supports immediate addressing mode (e.g. BRAN #-3)"
            );
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}
