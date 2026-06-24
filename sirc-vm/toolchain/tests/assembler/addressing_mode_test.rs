use nom_supreme::{
    error::ErrorTree,
    final_parser::{final_parser, Location},
};
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ImmediateInstructionData, Instruction, InstructionData,
    },
    registers::{AddressRegisterName, RegisterName},
};
use toolchain::{parsers::shared::parse_tokens, types::shared::Token};

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

fn parse_result(input: &str) -> Result<Vec<Token>, ErrorTree<Location>> {
    final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(parse_tokens)(input)
}

fn immediate_data(input: &str) -> ImmediateInstructionData {
    match parse_instruction(input) {
        InstructionData::Immediate(data) => data,
        instruction => panic!("Expected immediate instruction, got {instruction:?}"),
    }
}

#[test]
fn zero_displacement_shorthand_parses_memory_forms() {
    let data = immediate_data("LOAD r1, (a)\n");
    assert_eq!(data.op_code, Instruction::LoadRegisterFromIndirectImmediate);
    assert_eq!(data.register, RegisterName::R1.to_register_index());
    assert_eq!(data.value, 0);
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::Address.to_register_index()
    );

    let data = immediate_data("LOAD r2, (s)+\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadRegisterFromIndirectImmediatePostIncrement
    );
    assert_eq!(data.register, RegisterName::R2.to_register_index());
    assert_eq!(data.value, 0);
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::StackPointer.to_register_index()
    );

    let data = immediate_data("STOR (a), r3\n");
    assert_eq!(data.op_code, Instruction::StoreRegisterToIndirectImmediate);
    assert_eq!(data.register, RegisterName::R3.to_register_index());
    assert_eq!(data.value, 0);
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::Address.to_register_index()
    );

    let data = immediate_data("STOR -(s), r4\n");
    assert_eq!(
        data.op_code,
        Instruction::StoreRegisterToIndirectImmediatePreDecrement
    );
    assert_eq!(data.register, RegisterName::R4.to_register_index());
    assert_eq!(data.value, 0);
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::StackPointer.to_register_index()
    );
}

#[test]
fn zero_displacement_shorthand_parses_effective_address_forms() {
    let data = immediate_data("LDEA p, (a)\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressFromIndirectImmediate
    );
    assert_eq!(
        data.register,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    assert_eq!(data.value, 0);
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::Address.to_register_index()
    );

    let data = immediate_data("LDEA a, -(s)\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressFromIndirectImmediatePreDecrement
    );
    assert_eq!(
        data.register,
        AddressRegisterName::Address.to_register_index()
    );
    assert_eq!(data.value, 0);
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::StackPointer.to_register_index()
    );

    let data = immediate_data("LDEL p, (a)\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressAndLinkFromIndirectImmediate
    );
    assert_eq!(
        data.register,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    assert_eq!(data.value, 0);
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::Address.to_register_index()
    );

    let data = immediate_data("LDEL p, (a)+\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressAndLinkFromIndirectImmediatePostIncrement
    );
    assert_eq!(
        data.register,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    assert_eq!(data.value, 0);
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::Address.to_register_index()
    );

    let data = immediate_data("LJSR (a)+\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressAndLinkFromIndirectImmediatePostIncrement
    );
    assert_eq!(
        data.register,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    assert_eq!(data.value, 0);
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::Address.to_register_index()
    );
}

#[test]
fn zero_displacement_shorthand_does_not_make_illegal_families_legal() {
    assert!(parse_result("LOAD r1, -(a)\n").is_err());
    assert!(parse_result("STOR (a)+, r1\n").is_err());
    assert!(parse_result("LDEA p, (a)+\n").is_err());
    assert!(parse_result("LDEL p, -(a)\n").is_err());
    assert!(parse_result("LJMP (a)\n").is_err());
    assert!(parse_result("LJSR (a)\n").is_err());
}
