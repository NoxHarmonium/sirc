use nom_supreme::{
    error::ErrorTree,
    final_parser::{final_parser, Location},
};
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
    RegisterInstructionData, ShiftType, ShortImmediateInstructionData, StatusRegisterUpdateSource,
};
use peripheral_cpu::registers::RegisterName;
use toolchain::{parsers::shared::parse_tokens, types::shared::Token};

// Mnemonic family, then the immediate, short immediate and register opcodes
const TEST_VARIANTS: [(&str, Instruction, Instruction, Instruction); 7] = [
    (
        "CMN",
        Instruction::CompareNegativeImmediate,
        Instruction::CompareNegativeShortImmediate,
        Instruction::CompareNegativeRegister,
    ),
    (
        "CNC",
        Instruction::CompareNegativeImmediateWithCarry,
        Instruction::CompareNegativeShortImmediateWithCarry,
        Instruction::CompareNegativeRegisterWithCarry,
    ),
    (
        "CMP",
        Instruction::CompareImmediate,
        Instruction::CompareShortImmediate,
        Instruction::CompareRegister,
    ),
    (
        "CPC",
        Instruction::CompareImmediateWithCarry,
        Instruction::CompareShortImmediateWithCarry,
        Instruction::CompareRegisterWithCarry,
    ),
    (
        "TSA",
        Instruction::TestAndImmediate,
        Instruction::TestAndShortImmediate,
        Instruction::TestAndRegister,
    ),
    (
        "TSO",
        Instruction::TestOrImmediate,
        Instruction::TestOrShortImmediate,
        Instruction::TestOrRegister,
    ),
    (
        "TSX",
        Instruction::TestXorImmediate,
        Instruction::TestXorShortImmediate,
        Instruction::TestXorRegister,
    ),
];

fn parse_instruction(input: &str) -> InstructionData {
    let tokens =
        final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(parse_tokens)(input)
            .unwrap_or_else(|error| panic!("Error parsing instruction:\n{error}"));

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

fn short_immediate_data(input: &str) -> ShortImmediateInstructionData {
    match parse_instruction(input) {
        InstructionData::ShortImmediate(data) => data,
        instruction => panic!("Expected short immediate instruction, got {instruction:?}"),
    }
}

fn register_data(input: &str) -> RegisterInstructionData {
    match parse_instruction(input) {
        InstructionData::Register(data) => data,
        instruction => panic!("Expected register instruction, got {instruction:?}"),
    }
}

#[test]
fn test_variants_use_the_documented_opcodes() {
    let opcodes: Vec<(u8, u8, u8)> = TEST_VARIANTS
        .iter()
        .map(|(_, immediate, short, register)| (*immediate as u8, *short as u8, *register as u8))
        .collect();

    assert_eq!(
        opcodes,
        vec![
            (0x08, 0x28, 0x38),
            (0x09, 0x29, 0x39),
            (0x0A, 0x2A, 0x3A),
            (0x0B, 0x2B, 0x3B),
            (0x0C, 0x2C, 0x3C),
            (0x0D, 0x2D, 0x3D),
            (0x0E, 0x2E, 0x3E),
        ]
    );
}

#[test]
fn test_variants_parse_immediate_forms() {
    for (family, immediate, short, _) in TEST_VARIANTS {
        let data = immediate_data(&format!("{family}I r2, #0x1234\n"));
        assert_eq!(data.op_code, immediate, "{family}I");
        assert_eq!(data.register, RegisterName::R2.to_register_index());
        assert_eq!(data.value, 0x1234);
        assert_eq!(data.condition_flag, ConditionFlags::Always);
        assert_eq!(
            data.additional_flags,
            StatusRegisterUpdateSource::Alu.to_flags()
        );

        let data = short_immediate_data(&format!("{family}I|== ah, #0x12, LSL #2\n"));
        assert_eq!(data.op_code, short, "{family}I with a shift");
        assert_eq!(data.register, RegisterName::Ah.to_register_index());
        assert_eq!(data.value, 0x12);
        assert_eq!(data.shift_type, ShiftType::LogicalLeftShift);
        assert_eq!(data.shift_count, 2);
        assert_eq!(data.condition_flag, ConditionFlags::Equal);
    }
}

#[test]
fn test_variants_parse_register_forms() {
    for (family, _, _, register) in TEST_VARIANTS {
        // Two operands: the first register is also the (ignored) destination field
        let data = register_data(&format!("{family}R r1, sl\n"));
        assert_eq!(data.op_code, register, "{family}R");
        assert_eq!(data.r1, RegisterName::R1.to_register_index());
        assert_eq!(data.r2, RegisterName::R1.to_register_index());
        assert_eq!(data.r3, RegisterName::Sl.to_register_index());

        let data = register_data(&format!("{family}R|!= r3, r4, RTL #4\n"));
        assert_eq!(data.op_code, register, "{family}R with a shift");
        assert_eq!(data.r2, RegisterName::R3.to_register_index());
        assert_eq!(data.r3, RegisterName::R4.to_register_index());
        assert_eq!(data.shift_type, ShiftType::RotateLeft);
        assert_eq!(data.shift_count, 4);
        assert_eq!(data.condition_flag, ConditionFlags::NotEqual);
    }
}

#[test]
fn test_variants_accept_status_update_overrides() {
    for (family, immediate, short, register) in TEST_VARIANTS {
        let data = immediate_data(&format!("{family}I[N] r1, #5\n"));
        assert_eq!(data.op_code, immediate);
        assert_eq!(
            data.additional_flags,
            StatusRegisterUpdateSource::None.to_flags()
        );

        let data = short_immediate_data(&format!("{family}I[S] r1, #0, ASR #3\n"));
        assert_eq!(data.op_code, short);
        assert_eq!(
            data.additional_flags,
            StatusRegisterUpdateSource::Shift.to_flags()
        );

        let data = register_data(&format!("{family}R[S] r1, r2, LSR #1\n"));
        assert_eq!(data.op_code, register);
        assert_eq!(
            data.additional_flags,
            StatusRegisterUpdateSource::Shift.to_flags()
        );
    }
}

#[test]
fn compare_and_test_reject_three_registers() {
    for (family, ..) in TEST_VARIANTS {
        for suffix in ["", "[N]|!=", "[S]|=="] {
            for shift in ["", ", ASR #2"] {
                let input = format!("{family}R{suffix} r1, r2, r3{shift}\n");
                let error = parse_tokens(&input).unwrap_err().to_string();
                assert!(error.contains("exactly two registers"), "{input}: {error}");
            }
            let _ = register_data(&format!("{family}R{suffix} r1, r2\n"));
            let _ = register_data(&format!("{family}R{suffix} r1, r2, ASR #2\n"));
        }
    }
    let data = register_data("ADDR r1, r2, r3\n");
    assert_eq!((data.r1, data.r2, data.r3), (0, 1, 2));
}
