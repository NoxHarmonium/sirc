use super::support::assemble_object;
use std::panic::catch_unwind;
use toolchain::printers::shared::print_tokens;
use toolchain::types::numeric::{Constants, NumericField};
use toolchain::types::shared::{NumberToken, NumberType, OriginToken, Token};
use toolchain::{data::object::try_build_object, parsers::shared::parse_tokens};

fn assembled_program(source: &str) -> Vec<u8> {
    assemble_object(source).program
}

#[test]
fn named_constants_use_literal_numeric_semantics_in_every_field() {
    let named = ".EQU $N #-1\n.EQU $S #3\n.EQU $X #0x70\n.EQU $I #7\n\
        ADDI r1, $N\nADDI[S] r2, $I, ASR $S\nADDR r1, r2, RTL $S\nSHFT r1, LSL $S\n\
        EXCP $X\nCOPI $X\nETTR $S, r7\nLOAD r1, ($N, a)\n\
        .DB $N\n.DW $N\n.DQ $N\n.EQU $I #9\nADDI r1, $I\n.EQU $ALIAS $I\nADDI r2, $ALIAS\n";
    let literal = "ADDI r1, #-1\nADDI[S] r2, #7, ASR #3\nADDR r1, r2, RTL #3\nSHFT r1, LSL #3\n\
        EXCP #0x70\nCOPI #0x70\nETTR #3, r7\nLOAD r1, (#-1, a)\n\
        .DB #-1\n.DW #-1\n.DQ #-1\nADDI r1, #9\nADDI r2, #9\n";
    assert_eq!(assembled_program(named), assembled_program(literal));
    // Definitions and later rebindings must not shift the debug source positions.
    let object = assemble_object(".EQU $N #123\nADDI r1, $N\n.EQU $N #4\nADDI r2, $N\n");
    let info = object.debug_info.unwrap();
    assert_eq!(info.program_to_input_offset_mapping[&0], 13);
    assert_eq!(info.program_to_input_offset_mapping[&2], 36);
}

#[test]
fn ast_retains_multiple_constant_uses_until_object_generation() {
    let source = "ADDI r1, $VALUE, LSL $SHIFT\n";
    let (_, tokens) = parse_tokens(source).unwrap();
    let Token::Instruction(instruction) = &tokens[0] else {
        panic!("expected instruction");
    };
    assert_eq!(instruction.numeric_placeholders.len(), 2);
    let error = try_build_object(tokens, "test.sasm".into(), source.into()).unwrap_err();
    // The use location belongs to the original input, with no replacement map.
    assert_eq!(error.input_offset, source.find("$VALUE").unwrap());
}

#[test]
fn deferred_fields_preserve_memory_coprocessor_and_shift_encodings() {
    for (named, literal) in [
        ("MOVI r1, $N", "MOVI r1, #1"),
        ("LOAD r1, ($N, a)+", "LOAD r1, (#1, a)+"),
        ("STOR -($N, a), r1", "STOR -(#1, a), r1"),
        ("LDEA s, -($N, a)", "LDEA s, -(#1, a)"),
        ("LDEL a, ($N, s)+", "LDEL a, (#1, s)+"),
        ("LJMP a, $N", "LJMP a, #1"),
        ("LJSR a, $N", "LJSR a, #1"),
        ("BRAN $N", "BRAN #1"),
        ("ETFR r7, $N", "ETFR r7, #1"),
        ("ETTR $N, a", "ETTR #1, a"),
        ("DMAR s, $NEG", "DMAR s, #-3"),
        ("DMAW l, $N", "DMAW l, #1"),
        ("DMAT a, l, $N", "DMAT a, l, #1"),
        ("STOR (r2, a), r1, LSL $N", "STOR (r2, a), r1, LSL #1"),
        ("STOR -(r2, a), r1, LSL $N", "STOR -(r2, a), r1, LSL #1"),
    ] {
        let source = format!(".EQU $N #1\n.EQU $NEG #-3\n{named}\n");
        assert_eq!(
            assembled_program(&source),
            assembled_program(literal),
            "{named}"
        );
    }
}

#[test]
fn constants_cover_remaining_address_and_exception_forms() {
    for (named, literal) in [
        ("ADDR r1, r2, r3, LSL $N", "ADDR r1, r2, r3, LSL #1"),
        ("LDEA a, ($N, s)", "LDEA a, (#1, s)"),
        ("LDEL a, ($N, s)", "LDEL a, (#1, s)"),
        ("LJSR ($N, a)+", "LJSR (#1, a)+"),
        ("STOR ($N, a), r1", "STOR (#1, a), r1"),
        ("ETFR $N", "ETFR #1"),
        ("ETTR $N", "ETTR #1"),
        ("ETFR a, $N", "ETFR a, #1"),
    ] {
        assert_eq!(
            assembled_program(&format!(".EQU $N #1\n{named}\n")),
            assembled_program(literal),
            "{named}"
        );
    }
}

#[test]
fn undefined_data_and_alias_constants_report_the_original_use() {
    for source in [".DB $MISSING\n", ".EQU $ALIAS $MISSING\n"] {
        let (_, tokens) = parse_tokens(source).unwrap();
        let error = try_build_object(tokens, "test.sasm".into(), source.into()).unwrap_err();
        assert_eq!(error.input_offset, source.find("$MISSING").unwrap());
        assert!(error.message.contains("MISSING"));
    }
}

#[test]
fn numeric_only_fields_reject_label_relocations() {
    for source in ["EXCP @label\n", "ETFR r1, @label\n", "DMAT a, l, @label\n"] {
        assert!(parse_tokens(source).is_err(), "{source}");
    }
}

#[test]
fn named_data_and_aliases_round_trip_without_resolution() {
    let source = ".EQU $N #-5\n.EQU $ALIAS $N\n.DB $ALIAS\n";
    let (_, tokens) = parse_tokens(source).unwrap();
    let printed = print_tokens(&tokens);
    assert!(printed.contains(".EQU $N #-5"));
    assert!(printed.contains(".EQU $ALIAS $N"));
    assert!(printed.contains(".DB $ALIAS"));
    assert_eq!(assembled_program(source), assembled_program(&printed));
}

#[test]
fn object_builder_rejects_unaligned_origins_even_for_direct_ast_callers() {
    let error = try_build_object(
        vec![Token::Origin(OriginToken { offset: 1 })],
        "test.sasm".into(),
        String::new(),
    )
    .unwrap_err();
    assert!(error.message.contains("even word address"));
}

#[test]
fn malformed_ast_cannot_apply_a_numeric_field_to_the_wrong_format() {
    let constants = Constants::from([(
        "N".into(),
        NumberToken {
            value: 1,
            number_type: NumberType::Decimal,
        },
    )]);
    for source in ["ADDI r1, $N\n", "ADDI r1, $N, LSL #0\n"] {
        let (_, mut tokens) = parse_tokens(source).unwrap();
        let Token::Instruction(instruction) = &mut tokens[0] else {
            panic!("expected instruction");
        };
        instruction.numeric_placeholders[0].field = NumericField::Immediate8;
        if source.contains("LSL") {
            instruction.numeric_placeholders[0].field = NumericField::Immediate16;
        }
        assert!(
            catch_unwind(|| instruction.resolve_numeric_operands(&constants, source.len()))
                .is_err()
        );
    }
}
