use insta::assert_snapshot;
use nom_supreme::{
    error::ErrorTree,
    final_parser::{final_parser, Location},
};
use toolchain::parsers::shared::parse_tokens;
use toolchain::printers::shared::print_tokens;
use toolchain::types::shared::Token;

static PARSER_INPUT: &str = r"
; EQU Tests
.EQU $SOME_PLACEHOLDER          #0xCAFE
.EQU $SOME_OTHER_PLACEHOLDER    #1234

:some_label

; Data Tests
.DB #0xFF
.DW #0xFFFF
.DQ #0xFFFFFFFF
.DB #255
.DB #65535
.DQ #4294967295
.DB @some_label
.DW @some_label
.DQ @some_label
";

#[test]
fn test_printing_round_trip() {
    let tokens = match final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(
        parse_tokens,
    )(PARSER_INPUT)
    {
        Ok(tokens) => tokens,
        Err(error) => panic!("Error parsing file:\n{error}"),
    };

    let printed = print_tokens(&tokens);

    assert_snapshot!(printed);
}
