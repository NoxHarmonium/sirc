use crate::{
    parsers::instruction::{
        override_ref_token_type_if_implied, parse_instruction_operands, parse_instruction_tag,
        AddressingMode, ImmediateType, InstructionToken,
    },
    types::object::RefType,
};
use nom::combinator::map;
use nom::sequence::tuple;
use peripheral_cpu::instructions::definitions::{
    Instruction, RegisterInstructionData, StoreManyRegisterFromAddressRegisterData,
};

use super::super::shared::AsmResult;
pub fn stmr(i: &str) -> AsmResult<InstructionToken> {
    map(
        tuple((parse_instruction_tag("STMR"), parse_instruction_operands)),
        |(condition_flag, operands)| match operands.as_slice() {
            [AddressingMode::DirectRegisterRange((first_register, second_register)), AddressingMode::IndirectImmediateDisplacement(offset, address_register)] =>
            {
                match offset {
                    ImmediateType::Value(offset) => {
                        let maybe_shrunk_offset: Option<u8> =
                            (offset.to_owned() as i16).try_into().ok();
                        match maybe_shrunk_offset {
                            Some(shrunk_offset) => InstructionToken {
                                // TODO: Validate register range is valid (e.g. no x1->x1 or reverse ranges)
                                instruction: Instruction::StoreManyRegisterFromAddressRegister(
                                    StoreManyRegisterFromAddressRegisterData {
                                        // The load/store many instructions are weird in that they use the RegisterInstructionData struct
                                        // but they actually work with immediate displacement. This is due to having to encode the range
                                        // of registers which doesn't fit into the immediate encoding
                                        data: RegisterInstructionData {
                                            r1: first_register.to_register_index(),
                                            r2: second_register.to_register_index(),
                                            r3: address_register.to_register_index(),
                                            condition_flag,
                                            additional_flags: shrunk_offset,
                                        },
                                    },
                                ),
                                symbol_ref: None,
                            },
                            // TODO: Better error handling (need some sort of parser error that can be collected etc.)
                            None => panic!("Immediate offset can only be 8 bits when using the load/store many instructions"),
                        }
                    }
                    ImmediateType::SymbolRef(ref_token) => InstructionToken {
                        instruction: Instruction::StoreManyRegisterFromAddressRegister(
                            StoreManyRegisterFromAddressRegisterData {
                                data: RegisterInstructionData {
                                    r1: first_register.to_register_index(),
                                    r2: second_register.to_register_index(),
                                    r3: address_register.to_register_index(),
                                    condition_flag,
                                    additional_flags: 0x0, // Placeholder
                                },
                            },
                        ),
                        symbol_ref: Some(override_ref_token_type_if_implied(
                            ref_token,
                            RefType::SmallOffset,
                        )),
                    },
                }
            }
            // TODO: Better error message without being too verbose?
            _ => panic!("Invalid addressing mode for STMR"),
        },
    )(i)
}
