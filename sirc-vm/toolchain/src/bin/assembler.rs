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

use nom_supreme::error::ErrorTree;
use nom_supreme::final_parser::{final_parser, Location};

use toolchain::data::object::build_object;
use toolchain::parsers::instruction::{parse_tokens, Token};

use core::panic;
use std::fs::{read_to_string, write};
use std::io;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, value_name = "FILE")]
    input_file: PathBuf,

    #[clap(short, long, value_parser, value_name = "FILE")]
    output_file: PathBuf,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let file_contents = read_to_string(args.input_file)?;
    // Workaround for parsing errors where there isn't a newline at the end of the file
    let file_contents_with_new_line = file_contents.trim_end().to_owned() + "\n";
    let tokens = match final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(
        parse_tokens,
    )(file_contents_with_new_line.as_str())
    {
        Ok(tokens) => tokens,
        Err(error) => panic!("Error parsing file:\n{error}"),
    };
    let object_definition = build_object(tokens);
    let bytes_to_write = match postcard::to_allocvec(&object_definition) {
        Ok(bytes_to_write) => bytes_to_write,
        Err(error) => panic!("Error encoding file: {error}"),
    };

    write(args.output_file, bytes_to_write)?;
    Ok(())
}
