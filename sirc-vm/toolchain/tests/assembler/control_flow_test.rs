use nom_supreme::{
    error::ErrorTree,
    final_parser::{final_parser, Location},
};
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ImmediateInstructionData, Instruction, InstructionData, RegisterInstructionData,
    },
    registers::{AddressRegisterName, RegisterName},
};
use toolchain::{
    parsers::shared::parse_tokens,
    types::{instruction::InstructionToken, object::RefType, shared::Token},
};

fn parse_instruction(input: &str) -> InstructionToken {
    let tokens =
        final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(parse_tokens)(input)
            .unwrap_or_else(|error| panic!("Error parsing instruction:\n{error}"));

    assert_eq!(tokens.len(), 1);

    match tokens
        .into_iter()
        .next()
        .expect("one token should be present")
    {
        Token::Instruction(instruction) => instruction,
        token => panic!("Expected instruction token, got {token:?}"),
    }
}

fn parse_result(input: &str) -> Result<Vec<Token>, ErrorTree<Location>> {
    final_parser::<&str, Vec<Token>, ErrorTree<&str>, ErrorTree<Location>>(parse_tokens)(input)
}

fn immediate_data(input: &str) -> ImmediateInstructionData {
    match parse_instruction(input).instruction {
        InstructionData::Immediate(data) => data,
        instruction => panic!("Expected immediate instruction, got {instruction:?}"),
    }
}

fn register_data(input: &str) -> RegisterInstructionData {
    match parse_instruction(input).instruction {
        InstructionData::Register(data) => data,
        instruction => panic!("Expected register instruction, got {instruction:?}"),
    }
}

#[test]
fn ldel_parses_normal_forms() {
    let data = immediate_data("LDEL a, (#5, s)\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressAndLinkFromIndirectImmediate
    );
    assert_eq!(data.op_code as u8, 0x1C);
    assert_eq!(
        data.register,
        AddressRegisterName::Address.to_register_index()
    );
    assert_eq!(data.value, 5);
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::StackPointer.to_register_index()
    );

    let data = register_data("LDEL p, (r3, a)\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressAndLinkFromIndirectRegister
    );
    assert_eq!(data.op_code as u8, 0x1D);
    assert_eq!(
        data.r1,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    assert_eq!(data.r3, RegisterName::R3.to_register_index());
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::Address.to_register_index()
    );
}

#[test]
fn ldel_parses_post_increment_forms() {
    let data = immediate_data("LDEL a, (#0, s)+\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressAndLinkFromIndirectImmediatePostIncrement
    );
    assert_eq!(data.op_code as u8, 0x1E);
    assert_eq!(
        data.register,
        AddressRegisterName::Address.to_register_index()
    );
    assert_eq!(data.value, 0);
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::StackPointer.to_register_index()
    );

    let data = register_data("LDEL p, (r3, a)+\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressAndLinkFromIndirectRegisterPostIncrement
    );
    assert_eq!(data.op_code as u8, 0x1F);
    assert_eq!(
        data.r1,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    assert_eq!(data.r3, RegisterName::R3.to_register_index());
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::Address.to_register_index()
    );
}

#[test]
fn ldea_parses_pre_decrement_forms() {
    let data = immediate_data("LDEA a, -(#2, s)\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressFromIndirectImmediatePreDecrement
    );
    assert_eq!(data.op_code as u8, 0x1A);
    assert_eq!(
        data.register,
        AddressRegisterName::Address.to_register_index()
    );
    assert_eq!(data.value, 2);
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::StackPointer.to_register_index()
    );

    let data = register_data("LDEA p, -(r3, a)\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressFromIndirectRegisterPreDecrement
    );
    assert_eq!(data.op_code as u8, 0x1B);
    assert_eq!(
        data.r1,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    assert_eq!(data.r3, RegisterName::R3.to_register_index());
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::Address.to_register_index()
    );
}

#[test]
fn ljsr_alias_parses_normal_forms() {
    let data = immediate_data("LJSR a\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressAndLinkFromIndirectImmediate
    );
    assert_eq!(data.op_code as u8, 0x1C);
    assert_eq!(
        data.register,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    assert_eq!(data.value, 0);
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::Address.to_register_index()
    );

    let data = immediate_data("LJSR a, #4\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressAndLinkFromIndirectImmediate
    );
    assert_eq!(data.value, 4);
    assert_eq!(
        data.register,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::Address.to_register_index()
    );

    let data = register_data("LJSR a, r3\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressAndLinkFromIndirectRegister
    );
    assert_eq!(data.op_code as u8, 0x1D);
    assert_eq!(
        data.r1,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    assert_eq!(data.r3, RegisterName::R3.to_register_index());
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::Address.to_register_index()
    );
}

#[test]
fn ljsr_alias_parses_post_increment_forms() {
    let data = immediate_data("LJSR (#0, a)+\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressAndLinkFromIndirectImmediatePostIncrement
    );
    assert_eq!(data.op_code as u8, 0x1E);
    assert_eq!(
        data.register,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    assert_eq!(data.value, 0);
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::Address.to_register_index()
    );

    let data = register_data("LJSR (r3, a)+\n");
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressAndLinkFromIndirectRegisterPostIncrement
    );
    assert_eq!(data.op_code as u8, 0x1F);
    assert_eq!(
        data.r1,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    assert_eq!(data.r3, RegisterName::R3.to_register_index());
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::Address.to_register_index()
    );
}

#[test]
fn branch_aliases_parse_pc_relative_labels() {
    let bran = parse_instruction("BRAN @target\n");
    let data = match bran.instruction {
        InstructionData::Immediate(data) => data,
        instruction => panic!("Expected immediate instruction, got {instruction:?}"),
    };
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressFromIndirectImmediate
    );
    assert_eq!(data.op_code as u8, 0x18);
    assert_eq!(
        data.register,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    let symbol_ref = bran.symbol_ref.expect("BRAN should carry a symbol ref");
    assert_eq!(symbol_ref.name, "target");
    assert_eq!(symbol_ref.ref_type, RefType::Offset);

    let brsr = parse_instruction("BRSR @target\n");
    let data = match brsr.instruction {
        InstructionData::Immediate(data) => data,
        instruction => panic!("Expected immediate instruction, got {instruction:?}"),
    };
    assert_eq!(
        data.op_code,
        Instruction::LoadEffectiveAddressAndLinkFromIndirectImmediate
    );
    assert_eq!(data.op_code as u8, 0x1C);
    assert_eq!(
        data.register,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    assert_eq!(
        data.additional_flags,
        AddressRegisterName::ProgramCounter.to_register_index()
    );
    let symbol_ref = brsr.symbol_ref.expect("BRSR should carry a symbol ref");
    assert_eq!(symbol_ref.name, "target");
    assert_eq!(symbol_ref.ref_type, RefType::Offset);
}

#[test]
fn branch_aliases_reject_non_pc_relative_forms() {
    assert!(parse_result("BRAN (r3, a)\n").is_err());
    assert!(parse_result("BRSR (r3, a)\n").is_err());
}
