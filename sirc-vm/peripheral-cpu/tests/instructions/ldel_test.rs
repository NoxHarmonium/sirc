use peripheral_bus::BusPeripheral;
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
        RegisterInstructionData, ShiftOperand, ShiftType,
    },
    registers::{AddressRegisterIndexing, AddressRegisterName, RegisterName, Registers},
};

use crate::instructions::common;

use super::common::get_expected_registers;

#[allow(clippy::cast_sign_loss)]
#[test]
fn test_ldel_indirect_immediate_post_increment_updates_link_source_and_destination() {
    let instruction_data = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::LoadEffectiveAddressAndLinkFromIndirectImmediatePostIncrement,
        register: AddressRegisterName::Address.to_register_index(),
        value: 0x0004,
        condition_flag: ConditionFlags::Always,
        additional_flags: AddressRegisterName::StackPointer.to_register_index(),
    });
    let (previous, current) = common::run_instruction(
        &instruction_data,
        |registers: &mut Registers, _: &mut BusPeripheral| {
            registers.set_address_register_at_index(
                AddressRegisterName::Address.to_register_index(),
                0xBBBB_2222,
            );
            registers.set_address_register_at_index(
                AddressRegisterName::StackPointer.to_register_index(),
                0xAAAA_1000,
            );
        },
        0x00CC_0000,
    );
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.lh = 0x00CC;
            registers.ll = 0x0002;
            registers.set_address_register_at_index(
                AddressRegisterName::Address.to_register_index(),
                0x00AA_1004,
            );
            registers.set_address_register_at_index(
                AddressRegisterName::StackPointer.to_register_index(),
                0x00AA_1001,
            );
        });
    assert_eq!(
        expected_registers, current.registers,
        "Not equal:\nleft: {:X?}\nright:{:X?}\n",
        expected_registers, current.registers
    );
}

#[allow(clippy::cast_sign_loss)]
#[test]
fn test_ldel_indirect_register_post_increment_updates_link_source_and_destination() {
    let instruction_data = InstructionData::Register(RegisterInstructionData {
        op_code: Instruction::LoadEffectiveAddressAndLinkFromIndirectRegisterPostIncrement,
        r1: AddressRegisterName::Address.to_register_index(),
        r2: 0x0,
        r3: RegisterName::R1.to_register_index(),
        shift_operand: ShiftOperand::Immediate,
        shift_type: ShiftType::None,
        shift_count: 0,
        condition_flag: ConditionFlags::Always,
        additional_flags: AddressRegisterName::StackPointer.to_register_index(),
    });
    let (previous, current) = common::run_instruction(
        &instruction_data,
        |registers: &mut Registers, _: &mut BusPeripheral| {
            registers.r1 = 0x0004;
            registers.set_address_register_at_index(
                AddressRegisterName::Address.to_register_index(),
                0xBBBB_2222,
            );
            registers.set_address_register_at_index(
                AddressRegisterName::StackPointer.to_register_index(),
                0xAAAA_1000,
            );
        },
        0x00CC_0000,
    );
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.r1 = 0x0004;
            registers.lh = 0x00CC;
            registers.ll = 0x0002;
            registers.set_address_register_at_index(
                AddressRegisterName::Address.to_register_index(),
                0x00AA_1004,
            );
            registers.set_address_register_at_index(
                AddressRegisterName::StackPointer.to_register_index(),
                0x00AA_1001,
            );
        });
    assert_eq!(
        expected_registers, current.registers,
        "Not equal:\nleft: {:X?}\nright:{:X?}\n",
        expected_registers, current.registers
    );
}
