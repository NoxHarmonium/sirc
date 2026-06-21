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
use nom::error::FromExternalError;
use nom::{branch::alt, error::ErrorKind};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ImmediateInstructionData, Instruction, InstructionData,
    },
    registers::AddressRegisterName,
};

fn tag_to_alias_instruction(tag: &str) -> Instruction {
    match tag {
        "BRAN" => Instruction::LoadEffectiveAddressFromIndirectImmediate,
        "BRSR" => Instruction::LongJumpToSubroutineWithImmediateDisplacement,
        _ => panic!("No tag mapping for instruction [{tag}]"),
    }
}

use super::super::shared::AsmResult;

///
/// Parses branch aliases (LDEA/LDEL with p implied as the destination and source)
///
/// ```
/// use toolchain::parsers::opcodes::branching::branching;
/// use toolchain::types::instruction::InstructionToken;
/// use peripheral_cpu::coprocessors::processing_unit::definitions::{ConditionFlags, Instruction, InstructionData};
/// use nom_supreme::error::ErrorTree;
/// use nom_supreme::final_parser::{final_parser, Location};
///
/// let parsed_instruction = match final_parser::<&str, InstructionToken, ErrorTree<&str>, ErrorTree<Location>>(branching)("BRAN|!= #-4\n") {
///   Ok(tokens) => tokens,
///   Err(error) => panic!("Error parsing instruction:\n{}", error),
/// };
/// let (op_code, register, value, condition_flag, additional_flags) = match parsed_instruction.instruction {
///     InstructionData::Immediate(inner) => (inner.op_code, inner.register, inner.value, inner.condition_flag, inner.additional_flags),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// assert_eq!(op_code, Instruction::LoadEffectiveAddressFromIndirectImmediate);
/// assert_eq!(register, 0x03);
/// assert_eq!(value, 0xFFFC);
/// assert_eq!(condition_flag, ConditionFlags::NotEqual);
/// assert_eq!(additional_flags, 0x03);
/// ```
pub fn branching(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();
    let mut instructions = alt((parse_instruction_tag("BRAN"), parse_instruction_tag("BRSR")));

    let (i_after_instruction, (tag, condition_flag, status_register_update_source)) =
        instructions(i)?;

    let (i, operands) = parse_instruction_operands0(i_after_instruction)?;

    if status_register_update_source.is_some() {
        let error_string =
            format!("The [{tag}] opcode does not support an explicit status register update source. Only ALU instructions can update the status register as a side-effect.");
        return Err(nom::Err::Failure(ErrorTree::from_external_error(
            i_after_instruction,
            ErrorKind::Fail,
            error_string.as_str(),
        )));
    }

    let construct_branch_immediate_instruction = |value: u16| {
        InstructionData::Immediate(ImmediateInstructionData {
            op_code: tag_to_alias_instruction(tag.as_str()),
            register: AddressRegisterName::ProgramCounter.to_register_index(),
            value,
            condition_flag,
            additional_flags: AddressRegisterName::ProgramCounter.to_register_index(),
        })
    };

    if let [AddressingMode::Immediate(offset)] = operands.as_slice() {
        match offset {
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
        }
    } else {
        let error_string = format!(
            "The [{tag}] alias only supports PC-relative immediate or label operands (e.g. BRAN #-3 or BRAN @label)"
        );
        Err(nom::Err::Failure(ErrorTree::from_external_error(
            i_after_instruction,
            ErrorKind::Fail,
            error_string.as_str(),
        )))
    }
}
