mod parsers;

use crate::parsers::instruction::parse_instructions;

use clap::Parser;

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

    let file_contents = read_to_string(args.input_file);

    match file_contents {
        Ok(contents) => {
            let parse_result = parse_instructions(contents.as_str());

            match parse_result {
                Ok((_, parsed_instructions)) => {
                    write(args.output_file, parsed_instructions)?;
                }
                Err(error) => {
                    panic!("Error during assembly: {}", error)
                }
            }
        }
        Err(err) => return Err(err),
    }

    Ok(())
}
