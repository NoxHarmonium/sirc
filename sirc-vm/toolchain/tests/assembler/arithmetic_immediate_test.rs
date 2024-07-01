use insta::assert_snapshot;
use nom_supreme::{
    error::ErrorTree,
    final_parser::{final_parser, Location},
};
use pretty_hex::{config_hex, HexConfig};
use toolchain::data::object::build_object;
use toolchain::parsers::shared::parse_tokens;
use toolchain::types::shared::Token;

static PARSER_INPUT: &str = r"
; Two argument - destination and source register the same
ADDI r1, #0
ADCI r2, #0x0
SUBI r3, #1
SBCI r4, #0x1
ANDI r5, #254
ORRI r6, #0xF0
XORI r7, #0xFA
CMPI ll, #0xF
TSAI ah, #100
TSXI al, #0x10
COPI sh, #0xF0
ADDI sl, #0
ADCI ph, #0x0
SUBI pl, #1

; Two argument - immediate source shift
ADDI r1, #0, LSL #0
ADCI r2, #0x0, LSR #1
SUBI r3, #1, ASL #2
SBCI r4, #0x1, ASR #3
ANDI r5, #254, RTL #4
ORRI r6, #0xF0, RTR #5
XORI r7, #0xFA, NUL #6
CMPI ll, #0xF, LSR #8
TSAI ah, #100, ASL #9
TSXI al, #0x10, ASR #10
COPI sh, #0xF0, RTL #11
ADDI sl, #0, RTR #12
ADCI ph, #0x0, NUL #13
SUBI pl, #1, NUL #14
ADDI r1, #0, LSL #15

; Two argument - register source shift
ADDI r1, #0, LSL r1
ADCI r2, #0x0, LSR r2
SUBI r3, #1, ASL r3
SBCI r4, #0x1, ASR r4
ANDI r5, #254, RTL r5
ORRI r6, #0xAF, RTR r6
XORI r7, #0xFA, NUL r7
";

#[test]
fn test_assembler_arithmetic_immediate() {
    let hex_config = HexConfig {
        width: 4,
        ..HexConfig::default()
    };

    let tokens = match final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(
        parse_tokens,
    )(PARSER_INPUT)
    {
        Ok(tokens) => tokens,
        Err(error) => panic!("Error parsing file:\n{error}"),
    };

    let object = build_object(tokens, "UNIT_TEST".to_string(), PARSER_INPUT.to_string());
    assert_snapshot!(config_hex(&object.program.as_slice(), hex_config));
}
