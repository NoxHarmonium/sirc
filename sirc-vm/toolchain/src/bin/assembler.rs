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

use toolchain::data::object::try_build_object;
use toolchain::utils::error_formatter::format_line_with_error;

use std::fs::{read_to_string, write};
use std::io;
use std::path::PathBuf;
use toolchain::parsers::shared::parse_tokens;
use toolchain::types::shared::Token;

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

    let file_contents = read_to_string(args.input_file.clone())?;
    // Workaround for parsing errors where there isn't a newline at the end of the file
    let file_contents_with_new_line = file_contents.trim_end().to_owned() + "\n";
    let tokens = match final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(
        parse_tokens,
    )(file_contents_with_new_line.as_str())
    {
        Ok(tokens) => tokens,
        Err(error) => {
            let error_message = format_line_with_error(
                args.input_file.to_str().unwrap_or(""),
                &file_contents_with_new_line,
                &error,
            );
            eprintln!("{error_message}\n{error}");
            std::process::exit(1);
        }
    };
    let object_definition = try_build_object(
        tokens,
        args.input_file.display().to_string(),
        file_contents_with_new_line.clone(),
    )
    .unwrap_or_else(|error| {
        let (line, column) =
            toolchain::utils::source::line_column(&file_contents_with_new_line, error.input_offset);
        eprintln!(
            "error: {}\n  --> {}:{line}:{column}",
            error.message,
            args.input_file.display()
        );
        std::process::exit(1);
    });
    let bytes_to_write = match postcard::to_allocvec(&object_definition) {
        Ok(bytes_to_write) => bytes_to_write,
        Err(error) => return Err(io::Error::new(io::ErrorKind::InvalidData, error)),
    };

    write(args.output_file, bytes_to_write)?;
    Ok(())
}
