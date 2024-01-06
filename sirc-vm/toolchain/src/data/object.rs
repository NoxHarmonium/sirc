use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ImmediateInstructionData, InstructionData, ShortImmediateInstructionData,
    INSTRUCTION_SIZE_BYTES,
};

use crate::parsers::data::DataType;
use crate::parsers::instruction::{DataToken, Token};
use crate::types::object::{ObjectDefinition, SymbolDefinition, SymbolRef};
use peripheral_cpu::coprocessors::processing_unit::encoding::encode_instruction;

use std::collections::HashMap;

fn resolve_placeholder(
    placeholders: &HashMap<String, u32>,
    placeholder_name: &String,
    instruction_data: &InstructionData,
) -> InstructionData {
    let resolved_value = placeholders.get(placeholder_name)
        .unwrap_or_else(|| panic!("Could not find a value for placeholder name [{placeholder_name}]. Make sure it is defined with the .EQU directive."));
    match instruction_data {
        InstructionData::Immediate(immediate_instruction) => {
            if resolved_value > &0xFFFF {
                panic!("Immediate value (resolved from [{placeholder_name}] placeholder) can only be up to 16 bits ({resolved_value} > 0xFFFF)");
            } else {
                InstructionData::Immediate(ImmediateInstructionData {
                    value: resolved_value
                        .to_owned()
                        .try_into()
                        .expect("Value should fit into a u8 as it is checked above"),
                    ..immediate_instruction.clone()
                })
            }
        }
        InstructionData::ShortImmediate(short_immediate_instruction) => {
            if resolved_value > &0xFF {
                panic!("Immediate value (resolved from [{placeholder_name}] placeholder) can only be up to 8 bits when using a shift definition ({resolved_value} > 0xFF)");
            } else {
                InstructionData::ShortImmediate(ShortImmediateInstructionData {
                    value: resolved_value
                        .to_owned()
                        .try_into()
                        .expect("Value should fit into a u8 as it is checked above"),
                    ..short_immediate_instruction.clone()
                })
            }
        }
        InstructionData::Register(_) => instruction_data.clone(),
    }
}

#[allow(clippy::cast_possible_truncation)]
fn inject_data_value(
    data: DataToken,
    program: &mut [[u8; 4]],
    program_offset: usize,
    symbol_refs: &mut Vec<SymbolRef>,
    offset: u32,
    placeholders: &HashMap<String, u32>,
) {
    match data.value {
        DataType::Value(value) => {
            // Align on instruction sizes
            // TODO: Make packing smaller data sizes more efficient
            // E.g. put 4 DBs in one 32 bit chunk
            // TODO: Clean up this mess
            let bytes: [u8; 4] = match data.size_bytes {
                1 => [0x0, 0x0, 0x0, value as u8],
                2 => {
                    let word_bytes = u16::to_be_bytes(value as u16);
                    [0x0, 0x0, word_bytes[0], word_bytes[1]]
                }
                4 => u32::to_be_bytes(value),
                _ => panic!("Unsupported data size bytes {}", data.size_bytes),
            };

            program[program_offset] = bytes;
        }
        DataType::SymbolRef(symbol_ref) => {
            program[program_offset] = [0x0, 0x0, 0x0, 0x0];
            symbol_refs.push(SymbolRef {
                name: symbol_ref.name,
                offset,
                ref_type: symbol_ref.ref_type,
                data_only: true,
            });
        }
        DataType::PlaceHolder(placeholder_name) => {
            let resolved_value = placeholders.get(&placeholder_name)
                .unwrap_or_else(|| panic!("Could not find a value for placeholder name [{placeholder_name}]. Make sure it is defined with the .EQU directive."));
            program[program_offset] = u32::to_be_bytes(resolved_value.to_owned());
        }
    }
}

fn ensure_program_size(program: &mut Vec<[u8; 4]>, min_size: usize) {
    if min_size > program.len() {
        program.resize(min_size + 1, [0x0, 0x0, 0x0, 0x0]);
    }
}

pub fn build_object(tokens: Vec<Token>) -> ObjectDefinition {
    let mut symbols: Vec<SymbolDefinition> = vec![];
    let mut symbol_refs: Vec<SymbolRef> = vec![];
    let mut placeholders: HashMap<String, u32> = HashMap::new();
    let mut offset: u32 = 0x0;
    let mut program: Vec<[u8; 4]> = vec![];

    for token in tokens {
        let program_offset: usize = offset as usize / 4;
        ensure_program_size(&mut program, program_offset + 1);

        match token {
            Token::Instruction(data) => {
                if let Some(symbol_ref) = data.symbol_ref {
                    symbol_refs.push(SymbolRef {
                        name: symbol_ref.name,
                        offset,
                        ref_type: symbol_ref.ref_type,
                        data_only: false,
                    });
                }

                let instruction = if let Some(placeholder_name) = data.placeholder_name {
                    resolve_placeholder(&placeholders, &placeholder_name, &data.instruction)
                } else {
                    data.instruction
                };

                program[program_offset] = encode_instruction(&instruction);

                offset += INSTRUCTION_SIZE_BYTES;
            }
            Token::Label(data) => symbols.push(SymbolDefinition {
                name: data.name,
                offset,
            }),
            Token::Comment => {
                // Do nothing.
            }
            Token::Origin(data) => {
                // Word based addressing to match CPU
                offset = data.offset * 2;
            }
            Token::Data(data) => {
                inject_data_value(
                    data,
                    &mut program,
                    program_offset,
                    &mut symbol_refs,
                    offset,
                    &placeholders,
                );
                offset += INSTRUCTION_SIZE_BYTES;
            }
            Token::Equ(data) => {
                placeholders.insert(data.placeholder_name, data.value);
            }
        }
    }

    let bytes: Vec<u8> = program
        .iter()
        .flat_map(std::borrow::ToOwned::to_owned)
        .collect();

    ObjectDefinition {
        symbols,
        symbol_refs,
        program: bytes,
    }
}
