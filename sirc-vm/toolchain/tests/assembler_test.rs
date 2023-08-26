use nom_supreme::{
    error::ErrorTree,
    final_parser::{final_parser, Location},
};
use pretty_hex::{config_hex, HexConfig};
use toolchain::{
    data::object::build_object,
    parsers::instruction::{parse_tokens, Token},
};

#[test]
fn test_parse_tokens() {
    let hex_config = HexConfig {
        width: 4,
        ..HexConfig::default()
    };

    let tokens =
        match final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(parse_tokens)(
            r#"
    ADDI r1, #1
    ADCI r1, #1
    SUBI r1, #1
    SBCI r1, #1
    ANDI r1, #1
    ORRI r1, #1
    XORI r1, #1
    LOAD r1, #1
    CMPI r1, #1
    TSAI r1, #1
    TSXI r1, #1
    COPI r1, #1
"#
            .trim_start(),
        ) {
            Ok(tokens) => tokens,
            Err(error) => panic!("Error parsing file:\n{error}"),
        };

    let object = build_object(tokens);
    insta::assert_display_snapshot!(config_hex(&object.program.as_slice(), hex_config));
}
