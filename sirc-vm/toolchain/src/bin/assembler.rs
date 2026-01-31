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
use std::fmt::Write;
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
) -> Vec<(usize, usize)> {
    match &error {
        GenericErrorTree::Base { location, kind: _ } => vec![(location.line, location.column)],
        GenericErrorTree::Stack { base, contexts } => {
            let collected_base = collect_line_with_error(base);
            let mapped = contexts.iter().map(|c| (c.0.line, c.0.column)).collect();
            [collected_base, mapped].concat()
        }
        GenericErrorTree::Alt(sub_trees) => {
            sub_trees.iter().flat_map(collect_line_with_error).collect()
        }
    }
}

fn format_line_with_error(
    input_file: &str,
    file_contents_with_new_line: &str,
    error: &GenericErrorTree<
        Location,
        &'static str,
        &'static str,
        Box<dyn Error + Send + Sync + 'static>,
    >,
) -> String {
    // ANSI color codes - TODO: Move these somewhere
    const RESET: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";
    const RED: &str = "\x1b[31m";
    const CYAN: &str = "\x1b[36m";

    // We get a huge tree of errors which is useful when printing out all the parsers that were
    // tried, but in this case we just want the line that triggered the error, so the first line
    // should be the last that happened (at least according to the nom_supreme docs) which
    // is the faulting token.
    let lines = collect_line_with_error(error);
    if let Some((error_line, error_column)) = lines.first() {
        let all_lines: Vec<&str> = file_contents_with_new_line.lines().collect();

        // Early return if error_line is out of range
        if *error_line == 0 || *error_line > all_lines.len() {
            return "Unknown Line".to_string();
        }

        // Calculate the range of lines to display (2 before and 2 after)
        let start_line = error_line.saturating_sub(3); // -1 for 0-indexing, -2 for context
        let end_line = (error_line + 2).min(all_lines.len());

        let mut result = format!(
            "{BOLD}{RED}error{RESET}{BOLD}: parsing failed{RESET}\n  {BOLD}{CYAN}-->{RESET} {input_file}:{error_line}:{error_column}\n"
        );

        // Calculate the width needed for line numbers
        let line_num_width = end_line.to_string().len();

        // Add context lines
        for line_num in start_line..end_line {
            let line_text = all_lines.get(line_num).unwrap_or(&"");
            let display_line_num = line_num + 1;

            // Print the line
            writeln!(
                result,
                "{BOLD}{CYAN}{display_line_num:line_num_width$} |{RESET} {line_text}"
            )
            .unwrap();

            // Add pointer line if this is the error line
            if display_line_num == *error_line {
                let spaces = " ".repeat(error_column - 1);
                writeln!(
                    result,
                    "{BOLD}{CYAN}{:line_num_width$} |{RESET} {spaces}{BOLD}{RED}^{RESET}",
                    ""
                )
                .unwrap();
            }
        }

        return result;
    }
    "Unknown Line".to_string()
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
            panic!("Error parsing file:\n{error_message}\n{error}")
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
