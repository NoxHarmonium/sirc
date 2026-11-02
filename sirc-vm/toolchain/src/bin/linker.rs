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
use sirc_vm::debug_adapter::debug_map::write_debug_map;

use std::fs::{read, write};
use std::io;
use std::path::PathBuf;

use toolchain::types::object::{try_merge_object_definitions, ObjectDefinition};

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

    let (object_file, debug_info_map) = try_merge_object_definitions(&object_files)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;

    let linked_program = toolchain::data::linker::link_object(&object_file, args.segment_offset)?;

    write(args.output_file.clone(), linked_program)?;

    write_debug_map(&debug_info_map, args.output_file)?;

    Ok(())
}
