use insta::assert_snapshot;
use nom_supreme::{
    error::ErrorTree,
    final_parser::{final_parser, Location},
};
use toolchain::parsers::shared::parse_tokens;
use toolchain::types::shared::Token;
use toolchain::utils::error_formatter::format_line_with_error;

fn test_error_formatting(input: &str, expected_panic_msg: &str) -> String {
    let result =
        final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(parse_tokens)(input);

    match result {
        Ok(_) => panic!("{}", expected_panic_msg),
        Err(error) => {
            let formatted_error = format_line_with_error("test.sasm", input, &error);
            let error_tree = error.to_string();
            format!("{formatted_error}\n{error_tree}\n")
        }
    }
}

#[test]
fn test_no_input() {
    let input = "\n";
    let combined = test_error_formatting(
        input,
        "Expected parsing to fail for invalid LOAD addressing mode",
    );
    assert_snapshot!(combined, @r#"
    Unknown Line
    one of:
      in section "comment" at line 2, column 1,
      expected ';' at line 2, column 1, or
      in section "instruction" at line 2, column 1,
      one of:
        in section "Arithmetic Immediate Instruction" at line 2, column 1,
        one of:
          expected "ADDI" at line 2, column 1, or
          expected "ADCI" at line 2, column 1, or
          expected "SUBI" at line 2, column 1, or
          expected "SBCI" at line 2, column 1, or
          expected "ANDI" at line 2, column 1, or
          expected "ORRI" at line 2, column 1, or
          expected "XORI" at line 2, column 1, or
          expected "CMPI" at line 2, column 1, or
          expected "TSAI" at line 2, column 1, or
          expected "TSXI" at line 2, column 1, or
          expected "COPI" at line 2, column 1, or
          expected "SHFT" at line 2, column 1, or
        in section "Arithmetic Register Instruction" at line 2, column 1,
        one of:
          expected "ADDR" at line 2, column 1, or
          expected "ADCR" at line 2, column 1, or
          expected "SUBR" at line 2, column 1, or
          expected "SBCR" at line 2, column 1, or
          expected "ANDR" at line 2, column 1, or
          expected "ORRR" at line 2, column 1, or
          expected "XORR" at line 2, column 1, or
          expected "CMPR" at line 2, column 1, or
          expected "TSAR" at line 2, column 1, or
          expected "TSXR" at line 2, column 1, or
          expected "COPR" at line 2, column 1, or
        in section "Branching instruction" at line 2, column 1,
        one of:
          expected "BRAN" at line 2, column 1, or
          expected "BRSR" at line 2, column 1, or
        in section "Implied instruction" at line 2, column 1,
        one of:
          expected "RETS" at line 2, column 1, or
          expected "NOOP" at line 2, column 1, or
        in section "LDEA instruction" at line 2, column 1,
        expected "LDEA" at line 2, column 1, or
        in section "LJMP instruction" at line 2, column 1,
        expected "LJMP" at line 2, column 1, or
        in section "LJSR instruction" at line 2, column 1,
        expected "LJSR" at line 2, column 1, or
        in section "LOAD instruction" at line 2, column 1,
        expected "LOAD" at line 2, column 1, or
        in section "STOR instruction" at line 2, column 1,
        expected "STOR" at line 2, column 1, or
        in section "Exception Unit Instruction" at line 2, column 1,
        one of:
          expected "EXCP" at line 2, column 1, or
          expected "WAIT" at line 2, column 1, or
          expected "RETE" at line 2, column 1, or
          expected "RSET" at line 2, column 1, or
          expected "ETFR" at line 2, column 1, or
          expected "ETTR" at line 2, column 1, or
      in section "label" at line 2, column 1,
      expected ':' at line 2, column 1, or
      in section "origin" at line 2, column 1,
      expected ".ORG" at line 2, column 1, or
      in section "data directive" at line 2, column 1,
      one of:
        expected ".DB" at line 2, column 1, or
        expected ".DW" at line 2, column 1, or
        expected ".DQ" at line 2, column 1, or
      in section "equ directive" at line 2, column 1,
      expected ".EQU" at line 2, column 1
    "#);
}

#[test]
fn test_short_input() {
    let input = "LOAD r1\n";
    let combined = test_error_formatting(
        input,
        "Expected parsing to fail for invalid LOAD addressing mode",
    );
    assert_snapshot!(combined, @r#"
    error: parsing failed
      --> test.sasm:1:6
    1 | LOAD r1
      |      ^

    in section "instruction" at line 1, column 1,
    in section "LOAD instruction" at line 1, column 1,
    external error:
      Invalid addressing mode for LOAD: ([DirectRegister(R1)]) at line 1, column 6
    "#);
}

#[test]
fn test_rets_invalid_addressing_mode_error_context() {
    let input = "LOAD r1, #0\nRETS r3\nLOAD r2, #5\n";
    let combined = test_error_formatting(
        input,
        "Expected parsing to fail for invalid LOAD addressing mode",
    );
    assert_snapshot!(combined, @r#"
    error: parsing failed
      --> test.sasm:2:6
    1 | LOAD r1, #0
    2 | RETS r3
      |      ^
    3 | LOAD r2, #5

    in section "instruction" at line 2, column 1,
    in section "Implied instruction" at line 2, column 1,
    external error:
      The [RETS] does not support any addressing modes (e.g. NOOP or RETE) at line 2, column 6
    "#);
}

#[test]
fn test_wait_invalid_addressing_mode_error_context() {
    let input = "LOAD r1, #0\nWAIT r3\nLOAD r2, #5\n";
    let combined = test_error_formatting(
        input,
        "Expected parsing to fail for invalid LOAD addressing mode",
    );
    assert_snapshot!(combined, @r#"
    error: parsing failed
      --> test.sasm:2:6
    1 | LOAD r1, #0
    2 | WAIT r3
      |      ^
    3 | LOAD r2, #5

    in section "instruction" at line 2, column 1,
    in section "Exception Unit Instruction" at line 2, column 1,
    external error:
      Invalid addressing mode for WAIT: (("WAIT", [DirectRegister(R3)])) at line 2, column 6
    "#);
}

#[test]
fn test_load_invalid_addressing_mode_error_context() {
    let input = "LOAD r1, #0\nLOAD r1, r2, r3\nLOAD r2, #5\n";
    let combined = test_error_formatting(
        input,
        "Expected parsing to fail for invalid LOAD addressing mode",
    );
    assert_snapshot!(combined, @r#"
            error: parsing failed
              --> test.sasm:2:6
            1 | LOAD r1, #0
            2 | LOAD r1, r2, r3
              |      ^
            3 | LOAD r2, #5

            in section "instruction" at line 2, column 1,
            in section "LOAD instruction" at line 2, column 1,
            external error:
              Invalid addressing mode for LOAD: ([DirectRegister(R1), DirectRegister(R2), DirectRegister(R3)]) at line 2, column 6
            "#);
}

#[test]
fn test_store_invalid_addressing_mode_error_context() {
    let input = "STOR (#0, a), r1\nSTOR r1, r2\nSTOR (#5, a), r2\n";
    let combined = test_error_formatting(
        input,
        "Expected parsing to fail for invalid STOR addressing mode",
    );
    assert_snapshot!(combined, @r#"
            error: parsing failed
              --> test.sasm:2:6
            1 | STOR (#0, a), r1
            2 | STOR r1, r2
              |      ^
            3 | STOR (#5, a), r2

            in section "instruction" at line 2, column 1,
            in section "STOR instruction" at line 2, column 1,
            external error:
              Invalid addressing mode for STOR: ([DirectRegister(R1), DirectRegister(R2)]) at line 2, column 6
            "#);
}

#[test]
fn test_arithmetic_immediate_invalid_mode_error_context() {
    let input = "ADDI r1, #5\nADDI r1, r2\nADDI r2, #10\n";
    let combined = test_error_formatting(
        input,
        "Expected parsing to fail for ADDI with register operand",
    );
    assert_snapshot!(combined, @r#"
            error: parsing failed
              --> test.sasm:2:6
            1 | ADDI r1, #5
            2 | ADDI r1, r2
              |      ^
            3 | ADDI r2, #10

            in section "instruction" at line 2, column 1,
            in section "Arithmetic Immediate Instruction" at line 2, column 1,
            external error:
              The [ADDI] opcode only supports immediate->register addressing mode (e.g. ADDI y1, #1) at line 2, column 6
            "#);
}

#[test]
fn test_arithmetic_register_invalid_mode_error_context() {
    let input = "ADDR r1, r2\nADDR r1, #5\nADDR r3, r4\n";
    let combined = test_error_formatting(
        input,
        "Expected parsing to fail for ADDR with immediate operand",
    );
    assert_snapshot!(combined, @r#"
            error: parsing failed
              --> test.sasm:2:6
            1 | ADDR r1, r2
            2 | ADDR r1, #5
              |      ^
            3 | ADDR r3, r4

            in section "instruction" at line 2, column 1,
            in section "Arithmetic Register Instruction" at line 2, column 1,
            external error:
              The [ADDR] opcode only supports register->register or register->register->register addressing mode (e.g. ADDR y1, z2, a2 or SUBR y1, a2) at line 2, column 6
            "#);
}

#[test]
fn test_branching_invalid_mode_error_context() {
    let input = "BRAN #-5\nBRAN r1\nBRAN #10\n";
    let combined = test_error_formatting(
        input,
        "Expected parsing to fail for BRAN with register operand",
    );
    assert_snapshot!(combined, @r#"
            error: parsing failed
              --> test.sasm:2:6
            1 | BRAN #-5
            2 | BRAN r1
              |      ^
            3 | BRAN #10

            in section "instruction" at line 2, column 1,
            in section "Branching instruction" at line 2, column 1,
            external error:
              The [BRAN] opcode only supports immediate addressing mode (e.g. BRAN #-3) at line 2, column 6
            "#);
}

#[test]
fn test_ldea_invalid_mode_error_context() {
    let input = "LDEA a, (#0, a)\nLDEA r1, r2\nLDEA s, (#5, s)\n";
    let combined = test_error_formatting(
        input,
        "Expected parsing to fail for LDEA with invalid addressing mode",
    );
    assert_snapshot!(combined, @r#"
            error: parsing failed
              --> test.sasm:2:6
            1 | LDEA a, (#0, a)
            2 | LDEA r1, r2
              |      ^
            3 | LDEA s, (#5, s)

            in section "instruction" at line 2, column 1,
            in section "LDEA instruction" at line 2, column 1,
            external error:
              Invalid addressing mode for LDEA: ([DirectRegister(R1), DirectRegister(R2)]) at line 2, column 6
            "#);
}

#[test]
fn test_ljmp_invalid_mode_error_context() {
    let input = "LJMP a\nLJMP r1\nLJMP #5\n";
    let combined = test_error_formatting(
        input,
        "Expected parsing to fail for LJMP with invalid addressing mode",
    );
    assert_snapshot!(combined, @r#"
    error: parsing failed
      --> test.sasm:2:6
    1 | LJMP a
    2 | LJMP r1
      |      ^
    3 | LJMP #5

    in section "instruction" at line 2, column 1,
    in section "LJMP instruction" at line 2, column 1,
    external error:
      Invalid addressing mode for LJMP: ([DirectRegister(R1)]) at line 2, column 6
    "#);
}

#[test]
fn test_ljsr_invalid_mode_error_context() {
    let input = "LJSR a, #5\nLJSR s, r3\nLJSR r1\nADDI r2, #1";
    let combined = test_error_formatting(
        input,
        "Expected parsing to fail for LJSR with invalid addressing mode",
    );
    assert_snapshot!(combined, @r#"
    error: parsing failed
      --> test.sasm:3:6
    1 | LJSR a, #5
    2 | LJSR s, r3
    3 | LJSR r1
      |      ^
    4 | ADDI r2, #1

    in section "instruction" at line 3, column 1,
    in section "LJSR instruction" at line 3, column 1,
    external error:
      Invalid addressing mode for LJSR: ([DirectRegister(R1)]) at line 3, column 6
    "#);
}
