use crate::parsers::instruction::{
    parse_instruction_operands, parse_instruction_tag, AddressingMode, ImmediateType,
    InstructionToken,
};
use nom::{
    combinator::map_res,
    error::{ErrorKind, FromExternalError},
    sequence::tuple,
};
use nom_supreme::error::ErrorTree;
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, Instruction, InstructionData,
};

use super::super::shared::AsmResult;
pub fn excp(i: &str) -> AsmResult<InstructionToken> {
    map_res(
        tuple((parse_instruction_tag("EXCP"), parse_instruction_operands)),
        |((tag, condition_flag), operands)| match operands.as_slice() {
            [AddressingMode::DirectRegister(dest_register), AddressingMode::Immediate(immediate_type)] => {
                match immediate_type {
                    ImmediateType::Value(value) => Ok(InstructionToken {
                        instruction: InstructionData::Immediate(ImmediateInstructionData {
                            op_code: Instruction::Exception,
                            register: dest_register.to_register_index(),
                            value: *value,
                            condition_flag,
                            additional_flags: 0x00,
                        }),
                        symbol_ref: None,
                    }),
                    _ => {
                        let error_string = format!(
                            "The [{}] opcode does not support symbol refs at this time",
                            tag
                        );
                        Err(nom::Err::Failure(ErrorTree::from_external_error(
                            i.to_owned(),
                            ErrorKind::Fail,
                            error_string.as_str(),
                        )))
                    }
                }
            }
            _ => {
                let error_string = format!(
                    "The [{}] opcode only supports immediate addressing mode (e.g. EXCP #123)",
                    tag
                );
                Err(nom::Err::Failure(ErrorTree::from_external_error(
                    i.to_owned(),
                    ErrorKind::Fail,
                    error_string.as_str(),
                )))
            }
        },
    )(i)
}
