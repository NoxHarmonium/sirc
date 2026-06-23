use nom_supreme::{
    error::ErrorTree,
    final_parser::{final_parser, Location},
};
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ConditionFlags, Instruction, InstructionData, ShiftOperand, ShiftType,
};
use toolchain::parsers::shared::parse_tokens;
use toolchain::types::shared::Token;

fn parse_single_instruction(input: &str) -> InstructionData {
    let tokens =
        final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(parse_tokens)(input)
            .unwrap_or_else(|error| panic!("Error parsing instruction:\n{error}"));

    let [Token::Instruction(instruction_token)] = tokens.as_slice() else {
        panic!("Expected one instruction token, got {tokens:?}");
    };

    instruction_token.instruction.clone()
}

fn assert_rejected(input: &str, message_part: &str) {
    let result =
        final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(parse_tokens)(input);

    let error = result.expect_err("Expected COP syntax to be rejected");
    let error_string = error.to_string();
    assert!(
        error_string.contains(message_part),
        "Expected error to contain [{message_part}], got:\n{error_string}"
    );
}

#[test]
fn copi_accepts_source_only_immediate_form() {
    let instruction = parse_single_instruction("COPI #0x14FF\n");
    let InstructionData::Immediate(inner) = instruction else {
        panic!("Expected immediate COPI instruction, got {instruction:?}");
    };

    assert_eq!(inner.op_code, Instruction::CoprocessorCallImmediate);
    assert_eq!(inner.register, 0x0);
    assert_eq!(inner.value, 0x14FF);
    assert_eq!(inner.additional_flags, 0x0);
    assert_eq!(inner.condition_flag, ConditionFlags::Always);
}

#[test]
fn copi_preserves_condition_code() {
    let instruction = parse_single_instruction("COPI|== #0x1900\n");
    let InstructionData::Immediate(inner) = instruction else {
        panic!("Expected immediate COPI instruction, got {instruction:?}");
    };

    assert_eq!(inner.op_code, Instruction::CoprocessorCallImmediate);
    assert_eq!(inner.register, 0x0);
    assert_eq!(inner.value, 0x1900);
    assert_eq!(inner.additional_flags, 0x0);
    assert_eq!(inner.condition_flag, ConditionFlags::Equal);
}

#[test]
fn copr_accepts_source_only_register_form() {
    let instruction = parse_single_instruction("COPR r4\n");
    let InstructionData::Register(inner) = instruction else {
        panic!("Expected register COPR instruction, got {instruction:?}");
    };

    assert_eq!(inner.op_code, Instruction::CoprocessorCallRegister);
    assert_eq!(inner.r1, 0x0);
    assert_eq!(inner.r2, 0x0);
    assert_eq!(inner.r3, 0x4);
    assert_eq!(inner.shift_operand, ShiftOperand::Immediate);
    assert_eq!(inner.shift_type, ShiftType::None);
    assert_eq!(inner.shift_count, 0x0);
    assert_eq!(inner.additional_flags, 0x0);
    assert_eq!(inner.condition_flag, ConditionFlags::Always);
}

#[test]
fn copi_rejects_old_destination_register_forms() {
    assert_rejected(
        "COPI r1, #0x14FF\n",
        "does not support a destination register field",
    );
    assert_rejected(
        "COPI r1, #0x14FF, LSL #1\n",
        "does not support a destination register field",
    );
}

#[test]
fn copi_rejects_shift_syntax() {
    assert_rejected("COPI #0x14, LSL #1\n", "does not support shift syntax");
}

#[test]
fn copr_rejects_old_destination_register_forms() {
    assert_rejected(
        "COPR sh, r4\n",
        "does not support a destination register field",
    );
    assert_rejected(
        "COPR sh, r4, r4\n",
        "does not support a destination register field",
    );
}

#[test]
fn coprocessor_rejects_status_update_overrides() {
    assert_rejected(
        "COPI[N] #0x14FF\n",
        "does not support an explicit status register update source",
    );
    assert_rejected(
        "COPR[N] r4\n",
        "does not support an explicit status register update source",
    );
}
