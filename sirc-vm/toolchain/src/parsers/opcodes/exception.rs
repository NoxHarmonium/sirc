use super::super::shared::AsmResult;
use crate::parsers::instruction::{
    parse_instruction_operands0, parse_instruction_tag, AddressingMode, ImmediateType,
};
use crate::types::instruction::InstructionToken;
use nom::branch::alt;
use nom::error::{ErrorKind, FromExternalError};
use nom::sequence::tuple;
use nom_supreme::error::ErrorTree;
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ImmediateInstructionData, Instruction, InstructionData,
};
use peripheral_cpu::registers::{AddressRegisterName, RegisterName};

///
/// Parses meta instructions that encode into exception co-processor COP instructions
///
/// ```
/// use toolchain::parsers::opcodes::exception::exception;
/// use toolchain::types::instruction::InstructionToken;
/// use peripheral_cpu::coprocessors::processing_unit::definitions::{ConditionFlags, Instruction, InstructionData, RegisterInstructionData};
/// use nom_supreme::error::ErrorTree;
/// use nom_supreme::final_parser::{final_parser, Location};
///
/// let parsed_instruction = match final_parser::<&str, InstructionToken, ErrorTree<&str>, ErrorTree<Location>>(exception)("RSET|==") {
///   Ok(tokens) => tokens,
///   Err(error) => panic!("Error parsing instruction:\n{}", error),
/// };
/// let (op_code, condition_flag) = match parsed_instruction.instruction {
///     InstructionData::Immediate(inner) => (inner.op_code, inner.condition_flag),
///     _ => panic!("Incorrect instruction was parsed")
/// };
///
/// assert_eq!(op_code, Instruction::CoprocessorCallImmediate);
/// assert_eq!(condition_flag, ConditionFlags::Equal);
/// ```
pub fn exception(i: &str) -> AsmResult<InstructionToken> {
    let input_length = i.len();
    let instructions = alt((
        parse_instruction_tag("EXCP"),
        parse_instruction_tag("WAIT"),
        parse_instruction_tag("RETE"),
        parse_instruction_tag("RSET"),
        parse_instruction_tag("ETFR"),
        parse_instruction_tag("ETTR"),
    ));

    let (i, ((tag, condition_flag, status_register_update_source), operands)) =
        tuple((instructions, parse_instruction_operands0))(i)?;
    if status_register_update_source.is_some() {
        let error_string =
            format!("The [{tag}] opcode does not support an explicit status register update source. Only ALU instructions can update the status register as a side-effect.");
        return Err(nom::Err::Failure(ErrorTree::from_external_error(
            i,
            ErrorKind::Fail,
            error_string.as_str(),
        )));
    }

    // Implied instructions
    match (tag.as_str(), operands.as_slice()) {
        ("EXCP", [AddressingMode::Immediate(immediate_type)]) => {
            if let ImmediateType::Value(value) = immediate_type {
                Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: InstructionData::Immediate(ImmediateInstructionData {
                            op_code: Instruction::CoprocessorCallImmediate,
                            register: 0x0,
                            value: 0x01100 | (value & 0x00FF), // TODO:  Throw error if value doesn't fit?
                            additional_flags: 0x0,
                            condition_flag,
                        }),
                        ..Default::default()
                    },
                ))
            } else {
                let error_string = format!(
                    "The [{tag}] opcode does not support symbol refs or placeholders at this time"
                );
                Err(nom::Err::Failure(ErrorTree::from_external_error(
                    i,
                    ErrorKind::Fail,
                    error_string.as_str(),
                )))
            }
        }
        // Tell the exception module to halt the CPU until an event comes through
        ("WAIT", []) => Ok((
            i,
            InstructionToken {
                input_length,
                instruction: InstructionData::Immediate(ImmediateInstructionData {
                    op_code: Instruction::CoprocessorCallImmediate,
                    register: 0x0,
                    value: 0x01900,
                    additional_flags: 0x0,
                    condition_flag,
                }),
                ..Default::default()
            },
        )),
        // Tell the exception module to return from the current exception
        ("RETE", []) => Ok((
            i,
            InstructionToken {
                input_length,
                instruction: InstructionData::Immediate(ImmediateInstructionData {
                    op_code: Instruction::CoprocessorCallImmediate,
                    register: 0x0,
                    value: 0x1A00,
                    additional_flags: 0x0,
                    condition_flag,
                }),
                ..Default::default()
            },
        )),
        // Tell the exception module to reset the CPU
        ("RSET", []) => Ok((
            i,
            InstructionToken {
                input_length,
                instruction: InstructionData::Immediate(ImmediateInstructionData {
                    op_code: Instruction::CoprocessorCallImmediate,
                    register: 0x0,
                    value: 0x01B00,
                    additional_flags: 0x0,
                    condition_flag,
                }),
                ..Default::default()
            },
        )),
        ("ETFR", [AddressingMode::Immediate(immediate_type)]) => {
            // If just an immediate value is defined, it specifies the link register offset
            // and will transfer to both the address register _and_ r7 (the destination is implied)
            if let ImmediateType::Value(value) = immediate_type {
                Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: InstructionData::Immediate(ImmediateInstructionData {
                            op_code: Instruction::CoprocessorCallImmediate,
                            register: 0x0,
                            // 3 => transfer both registers
                            value: 0x01C30 | (value & 0x000F), // TODO:  Throw error if value doesn't fit?
                            additional_flags: 0x0,
                            condition_flag,
                        }),
                        ..Default::default()
                    },
                ))
            } else {
                let error_string = format!(
                    "The [{tag}] opcode does not support symbol refs or placeholders at this time"
                );
                Err(nom::Err::Failure(ErrorTree::from_external_error(
                    i,
                    ErrorKind::Fail,
                    error_string.as_str(),
                )))
            }
        }
        (
            "ETFR",
            [AddressingMode::DirectRegister(register), AddressingMode::Immediate(immediate_type)],
        ) => {
            // If the destination is r7, only transfer the return status register, and not the return address
            if let ImmediateType::Value(value) = immediate_type {
                if register == &RegisterName::R7 {
                    Ok((
                        i,
                        InstructionToken {
                            input_length,
                            instruction: InstructionData::Immediate(ImmediateInstructionData {
                                op_code: Instruction::CoprocessorCallImmediate,
                                register: 0x0,
                                // 2 => transfer only to r7
                                value: 0x01C20 | (value & 0x000F), // TODO:  Throw error if value doesn't fit?
                                additional_flags: 0x0,
                                condition_flag,
                            }),
                            ..Default::default()
                        },
                    ))
                } else {
                    let error_string = format!(
                        "The return_status_register can only be transferred to r7. Try ETFR r7, #{value} instead"
                    );
                    Err(nom::Err::Failure(ErrorTree::from_external_error(
                        i,
                        ErrorKind::Fail,
                        error_string.as_str(),
                    )))
                }
            } else {
                let error_string = format!(
                    "The [{tag}] opcode does not support symbol refs or placeholders at this time"
                );
                Err(nom::Err::Failure(ErrorTree::from_external_error(
                    i,
                    ErrorKind::Fail,
                    error_string.as_str(),
                )))
            }
        }
        (
            "ETFR",
            [AddressingMode::DirectAddressRegister(address_register), AddressingMode::Immediate(immediate_type)],
        ) => {
            // If the destination is a, only transfer the return address, and not the return status register
            if let ImmediateType::Value(value) = immediate_type {
                if address_register == &AddressRegisterName::Address {
                    Ok((
                        i,
                        InstructionToken {
                            input_length,
                            instruction: InstructionData::Immediate(ImmediateInstructionData {
                                op_code: Instruction::CoprocessorCallImmediate,
                                register: 0x0,
                                // 1 => transfer only to a
                                value: 0x01C10 | (value & 0x000F), // TODO:  Throw error if value doesn't fit?
                                additional_flags: 0x0,
                                condition_flag,
                            }),
                            ..Default::default()
                        },
                    ))
                } else {
                    let error_string = format!(
                        "The return_address can only be transferred to a. Try ETFR a, #{value} instead"
                    );
                    Err(nom::Err::Failure(ErrorTree::from_external_error(
                        i,
                        ErrorKind::Fail,
                        error_string.as_str(),
                    )))
                }
            } else {
                let error_string = format!(
                    "The [{tag}] opcode does not support symbol refs or placeholders at this time"
                );
                Err(nom::Err::Failure(ErrorTree::from_external_error(
                    i,
                    ErrorKind::Fail,
                    error_string.as_str(),
                )))
            }
        }
        ("ETTR", [AddressingMode::Immediate(immediate_type)]) => {
            // If just an immediate value is defined, it specifies the link register offset
            // and will transfer to both the address register _and_ r7 (the destination is implied)
            if let ImmediateType::Value(value) = immediate_type {
                Ok((
                    i,
                    InstructionToken {
                        input_length,
                        instruction: InstructionData::Immediate(ImmediateInstructionData {
                            op_code: Instruction::CoprocessorCallImmediate,
                            register: 0x0,
                            // 3 => transfer both registers
                            value: 0x01D30 | (value & 0x000F), // TODO:  Throw error if value doesn't fit?
                            additional_flags: 0x0,
                            condition_flag,
                        }),
                        ..Default::default()
                    },
                ))
            } else {
                let error_string = format!(
                    "The [{tag}] opcode does not support symbol refs or placeholders at this time"
                );
                Err(nom::Err::Failure(ErrorTree::from_external_error(
                    i,
                    ErrorKind::Fail,
                    error_string.as_str(),
                )))
            }
        }
        (
            "ETTR",
            [AddressingMode::Immediate(immediate_type), AddressingMode::DirectRegister(register)],
        ) => {
            // If the destination is r7, only transfer the return status register, and not the return address
            if let ImmediateType::Value(value) = immediate_type {
                if *register == RegisterName::R7 {
                    Ok((
                        i,
                        InstructionToken {
                            input_length,
                            instruction: InstructionData::Immediate(ImmediateInstructionData {
                                op_code: Instruction::CoprocessorCallImmediate,
                                register: 0x0,
                                // 2 => transfer only to r7
                                value: 0x01D20 | (value & 0x000F), // TODO:  Throw error if value doesn't fit?
                                additional_flags: 0x0,
                                condition_flag,
                            }),
                            ..Default::default()
                        },
                    ))
                } else {
                    let error_string = format!(
                        "The return_status_register can only be transferred to r7. Try ETTR r7, #{value} instead"
                    );
                    Err(nom::Err::Failure(ErrorTree::from_external_error(
                        i,
                        ErrorKind::Fail,
                        error_string.as_str(),
                    )))
                }
            } else {
                let error_string = format!(
                    "The [{tag}] opcode does not support symbol refs or placeholders at this time"
                );
                Err(nom::Err::Failure(ErrorTree::from_external_error(
                    i,
                    ErrorKind::Fail,
                    error_string.as_str(),
                )))
            }
        }
        (
            "ETTR",
            [AddressingMode::Immediate(immediate_type), AddressingMode::DirectAddressRegister(address_register)],
        ) => {
            // If the destination is a, only transfer the return address, and not the return status register
            if let ImmediateType::Value(value) = immediate_type {
                if *address_register == AddressRegisterName::Address {
                    Ok((
                        i,
                        InstructionToken {
                            input_length,
                            instruction: InstructionData::Immediate(ImmediateInstructionData {
                                op_code: Instruction::CoprocessorCallImmediate,
                                register: 0x0,
                                // 1 => transfer only to a
                                value: 0x01D10 | (value & 0x000F), // TODO:  Throw error if value doesn't fit?
                                additional_flags: 0x0,
                                condition_flag,
                            }),
                            ..Default::default()
                        },
                    ))
                } else {
                    let error_string = format!(
                        "The return_address can only be transferred to a. Try ETTR a, #{value} instead"
                    );
                    Err(nom::Err::Failure(ErrorTree::from_external_error(
                        i,
                        ErrorKind::Fail,
                        error_string.as_str(),
                    )))
                }
            } else {
                let error_string = format!(
                    "The [{tag}] opcode does not support symbol refs or placeholders at this time"
                );
                Err(nom::Err::Failure(ErrorTree::from_external_error(
                    i,
                    ErrorKind::Fail,
                    error_string.as_str(),
                )))
            }
        }

        modes => {
            let error_string = format!("Invalid addressing mode for {tag}: ({modes:?})");
            Err(nom::Err::Failure(ErrorTree::from_external_error(
                i,
                ErrorKind::Fail,
                error_string.as_str(),
            )))
        }
    }
}
