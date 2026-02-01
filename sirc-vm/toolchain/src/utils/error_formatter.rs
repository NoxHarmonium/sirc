use colored::Colorize;
use nom_supreme::error::GenericErrorTree;
use nom_supreme::final_parser::Location;
use std::error::Error;
use std::fmt::Write;

/// Recursively collects all (line, column) pairs from an error tree
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

/// Formats a parsing error with context lines and a pointer to the error location
///
/// This function creates a Rust-compiler-style error message with:
/// - Colored "error:" prefix and arrow
/// - File location with line and column
/// - Context lines (2 before and 2 after the error)
/// - Line numbers with consistent width
/// - A caret (^) pointing to the error column
///
/// Colors are automatically disabled when outputting to a non-TTY environment.
pub fn format_line_with_error(
    input_file: &str,
    file_contents_with_new_line: &str,
    error: &GenericErrorTree<
        Location,
        &'static str,
        &'static str,
        Box<dyn Error + Send + Sync + 'static>,
    >,
) -> String {
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
            "{}: parsing failed\n  {} {}:{error_line}:{error_column}\n",
            "error".red().bold(),
            "-->".cyan().bold(),
            input_file
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
                "{} {line_text}",
                format!("{display_line_num:line_num_width$} |")
                    .cyan()
                    .bold()
            )
            .unwrap();

            // Add pointer line if this is the error line
            if display_line_num == *error_line {
                let spaces = " ".repeat(error_column - 1);
                writeln!(
                    result,
                    "{} {spaces}{}",
                    format!("{:line_num_width$} |", "").cyan().bold(),
                    "^".red().bold()
                )
                .unwrap();
            }
        }

        return result;
    }
    "Unknown Line".to_string()
}
