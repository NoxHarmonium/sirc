use nom_supreme::{
    error::ErrorTree,
    final_parser::{final_parser, Location},
};
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
        RegisterInstructionData, ShiftType,
    },
    registers::RegisterName,
};
use toolchain::{parsers::shared::parse_tokens, types::shared::Token};

fn parse_result(input: &str) -> Result<Vec<Token>, ErrorTree<Location>> {
    final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(parse_tokens)(input)
}

fn parse_instruction(input: &str) -> InstructionData {
    let tokens =
        parse_result(input).unwrap_or_else(|error| panic!("Error parsing instruction:\n{error}"));

    assert_eq!(tokens.len(), 1);

    match tokens
        .into_iter()
        .next()
        .expect("one token should be present")
    {
        Token::Instruction(instruction) => instruction.instruction,
        token => panic!("Expected instruction token, got {token:?}"),
    }
}

fn immediate_data(input: &str) -> ImmediateInstructionData {
    match parse_instruction(input) {
        InstructionData::Immediate(data) => data,
        instruction => panic!("Expected immediate instruction, got {instruction:?}"),
    }
}

fn register_data(input: &str) -> RegisterInstructionData {
    match parse_instruction(input) {
        InstructionData::Register(data) => data,
        instruction => panic!("Expected register instruction, got {instruction:?}"),
    }
}

#[test]
fn movi_loads_an_immediate_value() {
    let data = immediate_data("MOVI r3, #0x1234\n");
    assert_eq!(data.op_code, Instruction::LoadRegisterFromImmediate);
    assert_eq!(data.register, RegisterName::R3.to_register_index());
    assert_eq!(data.value, 0x1234);
    assert_eq!(data.condition_flag, ConditionFlags::Always);
    assert_eq!(data.additional_flags, 0x0);

    let data = immediate_data("MOVI|== ah, #5\n");
    assert_eq!(data.register, RegisterName::Ah.to_register_index());
    assert_eq!(data.value, 5);
    assert_eq!(data.condition_flag, ConditionFlags::Equal);
}

#[test]
fn movr_copies_a_register() {
    let data = register_data("MOVR r1, sl\n");
    assert_eq!(data.op_code, Instruction::LoadRegisterFromRegister);
    assert_eq!(data.r1, RegisterName::R1.to_register_index());
    assert_eq!(data.r2, 0x0);
    assert_eq!(data.r3, RegisterName::Sl.to_register_index());
    assert_eq!(data.shift_type, ShiftType::None);
    assert_eq!(data.shift_count, 0);
    assert_eq!(data.condition_flag, ConditionFlags::Always);
    assert_eq!(data.additional_flags, 0x0);

    let data = register_data("MOVR|!= r2, r3\n");
    assert_eq!(data.condition_flag, ConditionFlags::NotEqual);
}

#[test]
fn mov_rejects_forms_the_cpu_cannot_encode() {
    // The MOV opcodes never update the status register
    assert!(parse_result("MOVI[A] r1, #5\n").is_err());
    assert!(parse_result("MOVR[S] r1, r2\n").is_err());

    // No shift on a register move: use SHFT instead
    assert!(parse_result("MOVR r1, r2, ASL #1\n").is_err());

    // The operand kind must match the mnemonic
    assert!(parse_result("MOVI r1, r2\n").is_err());
    assert!(parse_result("MOVR r1, #5\n").is_err());
    assert!(parse_result("MOVI r1, (#0, a)\n").is_err());
    assert!(parse_result("MOVR r1, (#0, a)\n").is_err());
}

#[test]
fn load_only_accepts_memory_forms() {
    assert!(parse_result("LOAD r1, (#0, a)\n").is_ok());
    assert!(parse_result("LOAD r1, (r2, a)+\n").is_ok());

    assert!(parse_result("LOAD r1, #5\n").is_err());
    assert!(parse_result("LOAD r1, r2\n").is_err());
}
