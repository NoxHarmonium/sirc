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

fn assert_copi_immediate(input: &str, value: u16, condition_flag: ConditionFlags) {
    let instruction = parse_single_instruction(input);
    let InstructionData::Immediate(inner) = instruction else {
        panic!("Expected immediate COPI instruction, got {instruction:?}");
    };

    assert_eq!(inner.op_code, Instruction::CoprocessorCallImmediate);
    assert_eq!(inner.register, 0x0);
    assert_eq!(inner.value, value);
    assert_eq!(inner.additional_flags, 0x0);
    assert_eq!(inner.condition_flag, condition_flag);
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

#[test]
fn dma_meta_instructions_lower_to_supervisor_copi_commands() {
    assert_copi_immediate("DMAR #0\n", 0x2800, ConditionFlags::Always);
    assert_copi_immediate("DMAR #7\n", 0x2807, ConditionFlags::Always);
    assert_copi_immediate("DMAW|== #5\n", 0x2905, ConditionFlags::Equal);
    assert_copi_immediate("DMAT #0\n", 0x2A00, ConditionFlags::Always);
    assert_copi_immediate("DMAT #255\n", 0x2AFF, ConditionFlags::Always);
}

#[test]
fn maths_meta_instructions_lower_to_user_copi_commands() {
    assert_copi_immediate("MULU\n", 0x3000, ConditionFlags::Always);
    assert_copi_immediate("MULS|!=\n", 0x3100, ConditionFlags::NotEqual);
    assert_copi_immediate("DIVU\n", 0x3200, ConditionFlags::Always);
    assert_copi_immediate("DIVS\n", 0x3300, ConditionFlags::Always);
}

#[test]
fn dma_meta_instructions_reject_invalid_counts() {
    assert_rejected("DMAR #8\n", "only supports counts in the range 0-7");
    assert_rejected("DMAW #8\n", "only supports counts in the range 0-7");
    assert_rejected("DMAT #256\n", "only supports counts in the range 0-255");
}

#[test]
fn standard_coprocessor_meta_instructions_reject_invalid_forms() {
    assert_rejected("DMAR r1\n", "Invalid addressing mode for DMAR");
    assert_rejected("DMAR #1, r1\n", "Invalid addressing mode for DMAR");
    assert_rejected("MULU #1\n", "Invalid addressing mode for MULU");
    assert_rejected(
        "MULU[N]\n",
        "does not support an explicit status register update source",
    );
    assert_rejected(
        "DMAT[N] #7\n",
        "does not support an explicit status register update source",
    );
}
