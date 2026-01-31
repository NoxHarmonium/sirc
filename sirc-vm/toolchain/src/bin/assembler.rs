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

use nom_supreme::error::{ErrorTree, GenericErrorTree};
use nom_supreme::final_parser::{final_parser, Location};

use toolchain::data::object::build_object;

use core::panic;
use std::error::Error;
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

fn collect_line_with_error(
    error: &GenericErrorTree<
        Location,
        &'static str,
        &'static str,
        Box<dyn Error + Send + Sync + 'static>,
    >,
) -> Vec<usize> {
    match &error {
        GenericErrorTree::Base { location, kind: _ } => vec![location.line],
        GenericErrorTree::Stack { base: _, contexts } => {
            contexts.first().map(|c| vec![c.0.line]).unwrap_or(vec![])
        }
        GenericErrorTree::Alt(sub_trees) => {
            sub_trees.iter().flat_map(collect_line_with_error).collect()
        }
    }
}

fn log_line_with_error(
    input_file: &str,
    file_contents_with_new_line: &str,
    error: &GenericErrorTree<
        Location,
        &'static str,
        &'static str,
        Box<dyn Error + Send + Sync + 'static>,
    >,
) {
    // TODO: Why is there so many duplicate lines?
    let lines = collect_line_with_error(error);
    for line in lines {
        if let Some(text) = file_contents_with_new_line.lines().nth(line - 1) {
            println!("In file {input_file}, at line {line}:\n{text}");
        }
    }
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
            log_line_with_error(
                args.input_file.to_str().unwrap_or(""),
                &file_contents_with_new_line,
                &error,
            );
            panic!("Error parsing file:\n{error}")
        }
    };
    let object_definition = build_object(
        tokens,
        args.input_file.display().to_string(),
        file_contents_with_new_line,
    );
    let bytes_to_write = match postcard::to_allocvec(&object_definition) {
        Ok(bytes_to_write) => bytes_to_write,
        Err(error) => panic!("Error encoding file: {error}"),
    };

    write(args.output_file, bytes_to_write)?;
    Ok(())
}
