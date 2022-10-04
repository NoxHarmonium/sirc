use clap::Parser;
use peripheral_cpu::instructions::definitions::INSTRUCTION_SIZE_WORDS;
use peripheral_cpu::instructions::encoding::encode_address_instruction;

use std::fs::{read, write};
use std::io;
use std::path::PathBuf;

use toolchain::types::object::ObjectDefinition;

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
        let target_offset = (target_symbol.offset / INSTRUCTION_SIZE_WORDS) + args.segment_offset;

        // TODO: This is a hack. Think of a better solution
        // The linker might need to decode/rencode instructions I guess
        let patched_instruction = encode_address_instruction(0x0E, target_offset);

        // TODO: How do we keep track of this? The assembler should do it but the offset will need to be in bytes
        let program_offset = symbol_ref.offset as usize;
        linked_program[program_offset] = patched_instruction[0];
        linked_program[program_offset + 1] = patched_instruction[1];
        linked_program[program_offset + 2] = patched_instruction[2];
        linked_program[program_offset + 3] = patched_instruction[3];
    }

    write(args.output_file, linked_program)?;

    Ok(())
}
