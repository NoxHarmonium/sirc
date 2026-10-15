use nom_supreme::{
    error::ErrorTree,
    final_parser::{final_parser, Location},
};
use toolchain::parsers::shared::parse_tokens;
use toolchain::types::shared::Token;

fn parse(input: &str) -> Vec<Token> {
    final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(parse_tokens)(input)
        .unwrap_or_else(|error| panic!("Error parsing input:\n{error}"))
}

fn assert_rejected(input: &str) {
    let result =
        final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(parse_tokens)(input);
    assert!(
        result.is_err(),
        "Expected input [{input}] to be rejected, but it parsed as {result:?}"
    );
}

// A bare ';' with nothing after it used to fail: `is_not("\n\r")` requires at least one
// character, so it couldn't match an empty comment body.
#[test]
fn blank_comment_before_another_line_parses() {
    let tokens = parse("NOOP\n;\nRETS\n");
    let [Token::Instruction(_), Token::Comment(text), Token::Instruction(_)] = tokens.as_slice()
    else {
        panic!("Expected instruction, comment, instruction, got {tokens:?}");
    };
    assert_eq!(text, "");
}

#[test]
fn blank_comment_at_end_of_file_with_no_trailing_newline_parses() {
    let tokens = parse("NOOP\n;");
    let [Token::Instruction(_), Token::Comment(text)] = tokens.as_slice() else {
        panic!("Expected instruction, comment, got {tokens:?}");
    };
    assert_eq!(text, "");
}

// NOOP/RETS used to require the very next character after the tag to be a line ending, with no
// tolerance for trailing whitespace or a comment -- unlike every operand-bearing instruction,
// which already left the top-level tokenizer to handle a trailing comment.
#[test]
fn zero_operand_instruction_accepts_a_trailing_comment() {
    for (input, expected_comment) in [
        ("NOOP ; a comment\n", " a comment"),
        ("RETS ; a comment\n", " a comment"),
    ] {
        let tokens = parse(input);
        let [Token::Instruction(_), Token::Comment(text)] = tokens.as_slice() else {
            panic!("Expected instruction, comment for input [{input}], got {tokens:?}");
        };
        assert_eq!(text, expected_comment);
    }
}

// Regression guard: WAIT/RETE are parsed by a different function (exception.rs's `exception`,
// not meta.rs's `meta`) and never had this bug -- confirm that stays true.
#[test]
fn exception_unit_zero_operand_instructions_already_accept_a_trailing_comment() {
    for input in ["WAIT ; a comment\n", "RETE ; a comment\n"] {
        let tokens = parse(input);
        let [Token::Instruction(_), Token::Comment(_)] = tokens.as_slice() else {
            panic!("Expected instruction, comment for input [{input}], got {tokens:?}");
        };
    }
}

// Genuine garbage after a zero-operand instruction must still be rejected.
#[test]
fn zero_operand_instruction_still_rejects_unexpected_content() {
    assert_rejected("NOOP GARBAGE\n");
    assert_rejected("NOOP r1\n");
}
