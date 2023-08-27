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
fn test_assembler_arithmetic_register() {
    let hex_config = HexConfig {
        width: 4,
        ..HexConfig::default()
    };

    let tokens =
        match final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(parse_tokens)(
            r#"
    ; Two argument - destination and source register the same
    ADDR r1, ph
    ADCR r2, pl
    SUBR r3, sh
    SBCR r4, sl
    ANDR r5, ah
    ORRR r6, al
    XORR r7, lh
    CMPR ll, r7
    TSAR ah, r6
    TSXR al, r5
    COPR sh, r4
    ADDR sl, r3
    ADCR ph, r2
    SUBR pl, r1

    ; Three argument
    ADDR r1, ph, lh
    ADCR r2, pl, ll
    SUBR r3, sh, ah
    SBCR r4, sl, al
    ANDR r5, ah, sh
    ORRR r6, al, sl
    XORR r7, lh, ph
    CMPR ll, r7, r1
    TSAR ah, r6, r2
    TSXR al, r5, r3
    COPR sh, r4, r4
    ADDR sl, r3, r5
    ADCR ph, r2, r6
    SUBR pl, r1, r7
"#,
        ) {
            Ok(tokens) => tokens,
            Err(error) => panic!("Error parsing file:\n{error}"),
        };

    let object = build_object(tokens);
    insta::assert_display_snapshot!(config_hex(&object.program.as_slice(), hex_config));
}
