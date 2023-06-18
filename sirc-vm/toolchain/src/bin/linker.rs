use clap::Parser;
use peripheral_cpu::instructions::definitions::{
    ImmediateInstructionData, Instruction, InstructionData, INSTRUCTION_SIZE_WORDS,
};
use peripheral_cpu::instructions::encoding::{decode_instruction, encode_instruction};

use core::panic;
use std::fs::{read, write};
use std::io;
use std::path::PathBuf;

use toolchain::types::object::{ObjectDefinition, RefType};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser, value_name = "INPUT FILES")]
    input_files: Vec<PathBuf>,

    #[clap(short, long, value_parser, value_name = "FILE")]
    output_file: PathBuf,

    // TODO: Can we pass a hex string to this somehow?
    #[clap(short, long, value_parser, value_name = "SEGMENT_OFFSET")]
    segment_offset: u32,
}

// TODO: Fix linker to do placeholder replacement based on instruction

fn main() -> io::Result<()> {
    let args = Args::parse();

    let object_files: Vec<ObjectDefinition> = args
        .input_files
        .iter()
        // TODO: Don't use unwrap here!
        .map(|object_file_path| read(object_file_path).unwrap())
        .map(|file_contents| postcard::from_bytes(&file_contents).unwrap())
        .collect();

    // TODO: Support merging (more than one file)!
    let object_file = object_files.get(0).unwrap();

    let mut linked_program = object_file.program.clone();
    for symbol_ref in &object_file.symbol_refs {
        // TODO: Don't use unwrap!
        let target_symbol = object_file
            .symbols
            .iter()
            .find(|symbol| symbol.name == symbol_ref.name)
            .unwrap();
        // TODO: Clear up confusion between byte addressing and instruction addressing
        let target_offset_words =
            (target_symbol.offset / INSTRUCTION_SIZE_WORDS) + args.segment_offset;
        let program_offset_bytes = symbol_ref.offset as usize;
        let program_offset_words =
            (program_offset_bytes as u32 / INSTRUCTION_SIZE_WORDS) + args.segment_offset;

        // TODO: Surely there is a better way to do this 🤦‍♀️
        let raw_instruction: [u8; 4] = [
            linked_program[program_offset_bytes],
            linked_program[program_offset_bytes + 1],
            linked_program[program_offset_bytes + 2],
            linked_program[program_offset_bytes + 3],
        ];

        let full_offset = target_offset_words as i32 - program_offset_words as i32;

        // let calculate_8_bit_value = || match symbol_ref.ref_type {
        //     RefType::SmallOffset => i8::try_from(full_offset).unwrap_or_else(|_| {
        //         panic!(
        //             "Offset {} ({} - {}) does not fit into a 8 bit signed integer ({}-{})",
        //             full_offset,
        //             target_offset_words,
        //             program_offset_words,
        //             i8::MIN,
        //             i8::MAX
        //         )
        //     }) as u8,
        //     RefType::Implied => {
        //         panic!("RefType should not be Implied at this point (it should be resolved in the linker)")
        //     }
        //     _ => panic!("Only SmallOffset RefType is supported by the LDMR/STMR instructions"),
        // };

        let calculate_16_bit_value = || match symbol_ref.ref_type {
            RefType::Offset => i16::try_from(full_offset).unwrap_or_else(|_| {
                panic!(
                    "Offset {} ({} - {}) does not fit into a 16 bit signed integer ({}-{})",
                    full_offset,
                    target_offset_words,
                    program_offset_words,
                    i16::MIN,
                    i16::MAX
                )
            }) as u16,
            RefType::SmallOffset => {
                panic!("SmallOffset RefType is only supported by the LDMR/STMR instructions")
            }
            RefType::LowerByte => bytemuck::cast::<u32, [u16; 2]>(target_offset_words)[1],
            RefType::UpperByte => bytemuck::cast::<u32, [u16; 2]>(target_offset_words)[0],
            RefType::Implied => {
                panic!("RefType should not be Implied at this point (it should be resolved in the linker)")
            }
        };

        let instruction = decode_instruction(raw_instruction);
        let patched_instruction = match instruction {
            InstructionData::Immediate(data) => match data.op_code {
                Instruction::ShortJumpImmediate => {
                    InstructionData::Immediate(ImmediateInstructionData {
                        op_code: data.op_code,
                        register: data.register,
                        value: calculate_16_bit_value(),
                        condition_flag: data.condition_flag,
                        additional_flags: data.additional_flags,
                    })
                }
                Instruction::ShortJumpToSubroutineImmediate => {
                    InstructionData::Immediate(ImmediateInstructionData {
                        op_code: data.op_code,
                        register: data.register,
                        value: calculate_16_bit_value(),
                        condition_flag: data.condition_flag,
                        additional_flags: data.additional_flags,
                    })
                }
                Instruction::BranchToSubroutineImmediate => {
                    InstructionData::Immediate(ImmediateInstructionData {
                        op_code: data.op_code,
                        register: data.register,
                        value: calculate_16_bit_value(),
                        condition_flag: data.condition_flag,
                        additional_flags: data.additional_flags,
                    })
                }
                Instruction::BranchImmediate => {
                    InstructionData::Immediate(ImmediateInstructionData {
                        op_code: data.op_code,
                        register: data.register,
                        value: calculate_16_bit_value(),
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
                Instruction::LoadRegisterFromImmediate => {
                    InstructionData::Immediate(ImmediateInstructionData {
                        op_code: data.op_code,
                        register: data.register,
                        value: calculate_16_bit_value(),
                        condition_flag: data.condition_flag,
                        additional_flags: data.additional_flags,
                    })
                }
                Instruction::LoadRegisterFromIndirectImmediate => {
                    InstructionData::Immediate(ImmediateInstructionData {
                        op_code: data.op_code,
                        register: data.register,
                        value: calculate_16_bit_value(),
                        condition_flag: data.condition_flag,
                        additional_flags: data.additional_flags,
                    })
                }
                Instruction::StoreRegisterToIndirectImmediate => {
                    InstructionData::Immediate(ImmediateInstructionData {
                        op_code: data.op_code,
                        register: data.register,
                        value: calculate_16_bit_value(),
                        condition_flag: data.condition_flag,
                        additional_flags: data.additional_flags,
                    })
                }
                _ => panic!(
                    "Can't patch address/offset for instruction: {:?}",
                    data.op_code
                ),
            },
            _ => panic!(
                "Can't patch address/offset for instruction: {:?}",
                instruction
            ),
        };

        let raw_patched_instruction = encode_instruction(&patched_instruction);

        // TODO: How do we keep track of this? The assembler should do it but the offset will need to be in bytes
        linked_program[program_offset_bytes..=program_offset_bytes + 3]
            .copy_from_slice(&raw_patched_instruction);
    }

    write(args.output_file, linked_program)?;

    Ok(())
}
