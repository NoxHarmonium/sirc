use crate::types::data::{DataToken, DataType, DB_VALUE, DQ_VALUE, DW_VALUE};
use crate::types::object::{ObjectDefinition, SymbolDefinition, SymbolRef};
use crate::types::shared::Token;

use peripheral_cpu::coprocessors::processing_unit::definitions::{
    INSTRUCTION_SIZE_BYTES, INSTRUCTION_SIZE_WORDS,
};
use peripheral_cpu::coprocessors::processing_unit::encoding::encode_instruction;
use sirc_vm::debug_adapter::types::ObjectDebugInfo;

use sha2::{Digest, Sha256};

use crate::types::shared::NumberToken;
use std::collections::HashMap;

#[allow(clippy::cast_possible_truncation)]
fn inject_data_value(
    data: DataToken,
    program: &mut [[u8; 4]],
    program_offset: usize,
    symbol_refs: &mut Vec<SymbolRef>,
    offset: u32,
    placeholders: &crate::types::numeric::Constants,
) {
    match data.value {
        DataType::Value(NumberToken { value, .. }) => {
            // TODO: Make packing smaller data sizes in assembled binaries more efficient
            // category=Toolchain
            // E.g. put 4 DBs in one 32 bit chunk
            // TODO: Clean up the data packing code in the Assembler
            // category=Refactoring
            let bytes: [u8; 4] = match data.size_bytes {
                DB_VALUE => [0x0, 0x0, 0x0, value as u8],
                DW_VALUE => {
                    let word_bytes = u16::to_be_bytes(value as u16);
                    [0x0, 0x0, word_bytes[0], word_bytes[1]]
                }
                DQ_VALUE => u32::to_be_bytes(value),
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
            let resolved_value = placeholders
                .get(&placeholder_name.name)
                .unwrap_or_else(|| panic!("Constant was validated before data injection"));
            let resolved_value = &resolved_value.value;
            // Truncate per data.size_bytes, same as the DataType::Value arm above -- the
            // placeholder's resolved value is an arbitrary u32 regardless of which directive
            // size uses it, so DB/DW must mask down rather than writing the full 4 bytes.
            let bytes: [u8; 4] = match data.size_bytes {
                DB_VALUE => [0x0, 0x0, 0x0, *resolved_value as u8],
                DW_VALUE => {
                    let word_bytes = u16::to_be_bytes(*resolved_value as u16);
                    [0x0, 0x0, word_bytes[0], word_bytes[1]]
                }
                DQ_VALUE => u32::to_be_bytes(*resolved_value),
                _ => panic!("Unsupported data size bytes {}", data.size_bytes),
            };
            program[program_offset] = bytes;
        }
    }
}

fn ensure_program_size(program: &mut Vec<[u8; 4]>, min_size: usize) {
    if min_size > program.len() {
        program.resize(min_size, [0x0, 0x0, 0x0, 0x0]);
    }
}

#[derive(Debug, Clone)]
pub struct AssemblyError {
    pub input_offset: usize,
    pub message: String,
}

pub fn build_object(
    tokens: Vec<Token>,
    original_filename: String,
    original_input: String,
) -> ObjectDefinition {
    try_build_object(tokens, original_filename, original_input)
        .expect("Invalid source passed to build_object; use try_build_object for diagnostics")
}

pub fn try_build_object(
    tokens: Vec<Token>,
    original_filename: String,
    original_input: String,
) -> Result<ObjectDefinition, AssemblyError> {
    let original_input_length = original_input.len();
    let mut symbols: Vec<SymbolDefinition> = vec![];
    let mut label_positions = HashMap::new();
    let mut symbol_refs: Vec<SymbolRef> = vec![];
    let mut placeholders = crate::types::numeric::Constants::new();
    let mut offset: u32 = 0x0;
    let mut program: Vec<[u8; 4]> = vec![];
    let mut debug_info = ObjectDebugInfo {
        original_filename,
        original_input,
        ..Default::default()
    };

    for token in tokens {
        let file_position = token
            .input_length()
            .map_or(0, |length| original_input_length - length);
        let program_offset: usize = offset as usize / 4;

        ensure_program_size(&mut program, program_offset + 1);

        match token {
            Token::Instruction(data) => {
                if let Some(symbol_ref) = &data.symbol_ref {
                    symbol_refs.push(SymbolRef {
                        name: symbol_ref.name.clone(),
                        offset,
                        ref_type: symbol_ref.ref_type,
                        data_only: false,
                    });
                }

                let instruction =
                    data.resolve_numeric_operands(&placeholders, original_input_length)?;

                debug_info.program_to_input_offset_mapping.insert(
                    // TODO: Improve error handling in assembler
                    // category=Refactoring
                    // Maybe remove this unwrap
                    (u32::try_from(program_offset).unwrap()) * INSTRUCTION_SIZE_WORDS,
                    file_position,
                );

                program[program_offset] = encode_instruction(&instruction);

                offset += INSTRUCTION_SIZE_BYTES;
            }
            Token::Label(data) => {
                let input_offset = file_position;
                if let Some(previous) = label_positions.insert(data.name.clone(), input_offset) {
                    let (line, _) =
                        crate::utils::source::line_column(&debug_info.original_input, previous);
                    return Err(AssemblyError {
                        input_offset,
                        message: format!(
                            "Duplicate label [{}]; first defined in {}:{line}",
                            data.name, debug_info.original_filename
                        ),
                    });
                }
                symbols.push(SymbolDefinition {
                    name: data.name,
                    offset,
                });
            }
            Token::Comment(_) => {
                // Do nothing.
            }
            Token::Origin(data) => {
                if data.offset & 1 != 0 {
                    return Err(AssemblyError {
                        input_offset: 0,
                        message: ".ORG requires an even word address".into(),
                    });
                }
                // Word based addressing to match CPU
                offset = data.offset * 2;
            }
            Token::Data(data) => {
                if let DataType::PlaceHolder(reference) = &data.value {
                    crate::types::numeric::NumericOperand::Constant(reference.clone())
                        .resolve(&placeholders)
                        .map_err(|message| AssemblyError {
                            input_offset: original_input_length - reference.input_length,
                            message,
                        })?;
                }
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
                let number =
                    data.value
                        .resolve(&placeholders)
                        .map_err(|message| AssemblyError {
                            input_offset: match &data.value {
                                crate::types::numeric::NumericOperand::Constant(reference) => {
                                    original_input_length - reference.input_length
                                }
                                crate::types::numeric::NumericOperand::Literal(_) => 0,
                            },
                            message,
                        })?;
                placeholders.insert(data.placeholder_name, number);
            }
        }
    }

    let bytes: Vec<u8> = program
        .iter()
        .flat_map(std::borrow::ToOwned::to_owned)
        .collect();

    // Calculate hash as a stable way to refer to sources
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    debug_info.checksum = hex::encode(hasher.finalize());

    Ok(ObjectDefinition {
        symbols,
        symbol_refs,
        program: bytes,
        debug_info: Some(debug_info),
    })
}
