// TODO: Find a way to not have to copy/paste all the clippy config around to each crate root
// category=Refactoring
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    // I don't like this rule
    clippy::module_name_repetitions,
    // Not sure what this is, will have to revisit
    clippy::must_use_candidate,
    // Will tackle this at the next clean up
    clippy::too_many_lines,
    // Might be good practice but too much work for now
    clippy::missing_errors_doc,
    // Not stable yet - try again later
    clippy::missing_const_for_fn
)]
#![deny(warnings)]

use clap::Parser;
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ImmediateInstructionData, Instruction, InstructionData, INSTRUCTION_SIZE_WORDS,
};
use peripheral_cpu::coprocessors::processing_unit::encoding::{
    decode_instruction, encode_instruction,
};
use sirc_vm::debug_adapter::debug_map::write_debug_map;

use core::panic;
use std::fs::{read, write};
use std::io;
use std::path::PathBuf;

use toolchain::types::object::{merge_object_definitions, ObjectDefinition, RefType};

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]
fn resolve_16_bit_ref_value(
    ref_type: RefType,
    target_offset_words: u32,
    program_offset_words: u32,
    full_offset: i32,
) -> u16 {
    match ref_type {
        RefType::Offset => i16::try_from(full_offset).unwrap_or_else(|_| {
            panic!(
                "Offset {full_offset} ({target_offset_words} - {program_offset_words}) does not fit into a 16 bit signed integer ({}-{})",
                i16::MIN,
                i16::MAX
            )
        }) as u16,
        RefType::LowerWord => bytemuck::cast::<u32, [u16; 2]>(target_offset_words)[0],
        RefType::UpperWord => bytemuck::cast::<u32, [u16; 2]>(target_offset_words)[1],
        RefType::FullAddress => {
            panic!("RefType should not be FullAddress when resolving a 16-bit value (try the DQ directive)")
        }
        RefType::Implied => {
            panic!("RefType should not be Implied at this point (it should be resolved in the assembler)")
        }
    }
}

fn resolve_data_symbol_ref_bytes(
    ref_type: RefType,
    target_offset_words: u32,
    program_offset_words: u32,
    full_offset: i32,
) -> [u8; 4] {
    match ref_type {
        RefType::FullAddress => u32::to_be_bytes(target_offset_words),
        RefType::Offset | RefType::LowerWord | RefType::UpperWord => {
            let word_bytes = u16::to_be_bytes(resolve_16_bit_ref_value(
                ref_type,
                target_offset_words,
                program_offset_words,
                full_offset,
            ));
            [0x0, 0x0, word_bytes[0], word_bytes[1]]
        }
        RefType::Implied => {
            panic!("RefType should not be Implied at this point (it should be resolved in the assembler)")
        }
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser, value_name = "INPUT FILES")]
    input_files: Vec<PathBuf>,

    #[clap(short, long, value_parser, value_name = "FILE")]
    output_file: PathBuf,

    #[clap(short, long, value_parser, value_name = "SEGMENT_OFFSET")]
    segment_offset: u32,
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]
fn main() -> io::Result<()> {
    let args = Args::parse();

    let object_files: Vec<ObjectDefinition> = args
        .input_files
        .iter()
        // TODO: Handle IO errors in linker
        // category=Refactoring
        .map(|object_file_path| read(object_file_path).unwrap())
        .map(|file_contents| postcard::from_bytes(&file_contents).unwrap())
        .collect();

    let (object_file, debug_info_map) = merge_object_definitions(&object_files);

    let mut linked_program = object_file.program.clone();

    for symbol_ref in &object_file.symbol_refs {
        // TODO: Better error handling when target symbol is not found in linker
        // category=Refactoring
        let target_symbol = object_file
            .symbols
            .iter()
            .find(|symbol| symbol.name == symbol_ref.name)
            .unwrap_or_else(|| {
                panic!(
                    "Cannot find symbol [{}] in symbol definitions. Check you have a label with that name defined in your program.",
                    symbol_ref.name
                );
            });

        // TODO: Clear up confusion between byte addressing and instruction addressing in linker
        // category=Refactoring
        // Sometimes we use words, sometimes bytes, sometimes even double words (see also debug info etc.)
        let target_offset_words =
            (target_symbol.offset / INSTRUCTION_SIZE_WORDS) + args.segment_offset;
        let program_offset_bytes = symbol_ref.offset as usize;
        let program_offset_words =
            (program_offset_bytes as u32 / INSTRUCTION_SIZE_WORDS) + args.segment_offset;

        // TODO: Find a more elegant way to extract instruction in linker
        // category=Refactoring
        // Surely there is a better way to do this 🤦‍♀️
        let raw_instruction: [u8; 4] = [
            linked_program[program_offset_bytes],
            linked_program[program_offset_bytes + 1],
            linked_program[program_offset_bytes + 2],
            linked_program[program_offset_bytes + 3],
        ];

        let full_offset = target_offset_words as i32 - program_offset_words as i32;

        if symbol_ref.data_only {
            let value_to_insert_bytes = resolve_data_symbol_ref_bytes(
                symbol_ref.ref_type,
                target_offset_words,
                program_offset_words,
                full_offset,
            );
            linked_program[program_offset_bytes..=program_offset_bytes + 3]
                .copy_from_slice(&value_to_insert_bytes);
        } else {
            let instruction = decode_instruction(raw_instruction);
            let patched_instruction = match instruction {
                InstructionData::Immediate(data) => match data.op_code {
                    Instruction::LoadEffectiveAddressAndLinkFromIndirectImmediatePostIncrement
                    | Instruction::LoadEffectiveAddressFromIndirectImmediatePreDecrement
                    | Instruction::LoadEffectiveAddressFromIndirectImmediate
                    | Instruction::LoadEffectiveAddressAndLinkFromIndirectImmediate
                    | Instruction::LoadRegisterFromImmediate
                    | Instruction::LoadRegisterFromIndirectImmediate
                    | Instruction::StoreRegisterToIndirectImmediate => {
                        InstructionData::Immediate(ImmediateInstructionData {
                            op_code: data.op_code,
                            register: data.register,
                            value: resolve_16_bit_ref_value(
                                symbol_ref.ref_type,
                                target_offset_words,
                                program_offset_words,
                                full_offset,
                            ),
                            condition_flag: data.condition_flag,
                            additional_flags: data.additional_flags,
                        })
                    }
                    // Instruction::LoadEffectiveAddressIndirectImmediate(data) => {
                    //     Instruction::LoadEffectiveAddressIndirectImmediate(
                    //         LoadEffectiveAddressFromIndirectImmediateData {
                    //             data: ImmediateInstructionData {
                    //                 register: data.register,
                    //                 value: calculate_16_bit_value(),
                    //                 condition_flag: data.condition_flag,
                    //                 additional_flags: data.additional_flags,
                    //             },
                    //         },
                    //     )
                    // }
                    _ => panic!(
                        "Can't patch address/offset for instruction: {:?}",
                        data.op_code
                    ),
                },
                _ => panic!("Can't patch address/offset for instruction: {instruction:?}"),
            };

            let raw_patched_instruction = encode_instruction(&patched_instruction);

            linked_program[program_offset_bytes..=program_offset_bytes + 3]
                .copy_from_slice(&raw_patched_instruction);
        }
    }

    write(args.output_file.clone(), linked_program)?;

    write_debug_map(&debug_info_map, args.output_file)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_full_address_for_data_directive() {
        assert_eq!(
            resolve_data_symbol_ref_bytes(RefType::FullAddress, 0x00_CAFE, 0, 0),
            [0x00, 0x00, 0xCA, 0xFE]
        );
    }

    #[test]
    fn resolves_lower_word_for_data_directive() {
        // IB-5 (docs/reference/review/implementation-bugs.md): this used to panic --
        // "Only FullAddress ref type currently supported for data directives".
        assert_eq!(
            resolve_data_symbol_ref_bytes(RefType::LowerWord, 0x12_CAFE, 0, 0),
            [0x00, 0x00, 0xCA, 0xFE]
        );
    }

    #[test]
    fn resolves_upper_word_for_data_directive() {
        assert_eq!(
            resolve_data_symbol_ref_bytes(RefType::UpperWord, 0x12_CAFE, 0, 0),
            [0x00, 0x00, 0x00, 0x12]
        );
    }

    #[test]
    fn resolves_offset_for_data_directive() {
        assert_eq!(
            resolve_data_symbol_ref_bytes(RefType::Offset, 100, 40, 60),
            [0x00, 0x00, 0x00, 0x3C]
        );
    }

    #[test]
    fn resolves_lower_word_for_instruction() {
        assert_eq!(
            resolve_16_bit_ref_value(RefType::LowerWord, 0x12_CAFE, 0, 0),
            0xCAFE
        );
    }

    #[test]
    #[should_panic(expected = "does not fit into a 16 bit signed integer")]
    fn offset_overflow_panics() {
        resolve_16_bit_ref_value(RefType::Offset, 0, 0, i32::from(i16::MAX) + 1);
    }
}
