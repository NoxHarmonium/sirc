use assert_hex::assert_eq_hex;
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
    },
    new_cpu_peripheral,
    registers::{FullAddressRegisterAccess, StatusRegisterFields, set_sr_bit, sr_bit_is_set},
};

use crate::exceptions::common::{
    expect_exception_handler, expect_exception_handler_masked,
    expect_main_program_cycle_with_instruction, run_expectations, run_return_from_exception,
};

use super::common::build_test_instruction;

pub fn build_software_exception_instruction() -> InstructionData {
    InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::CoprocessorCallImmediate,
        register: 0x0,
        value: 0x1170,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    })
}

#[test]
fn test_software_exception_vector() {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    let mut clocks = 0;

    let software_exception: InstructionData = build_software_exception_instruction();

    // Set protected mode to test if the exception flips into privileged mode.
    set_sr_bit(
        StatusRegisterFields::ProtectedMode,
        &mut cpu_peripheral.registers,
    );

    run_expectations(
        &mut cpu_peripheral,
        &expect_main_program_cycle_with_instruction(0x0000_0000, &software_exception),
        &mut clocks,
    );

    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0x0, 0x70, (0xABCD, 0xAB00)),
        &mut clocks,
    );

    assert!(!sr_bit_is_set(
        StatusRegisterFields::ProtectedMode,
        &cpu_peripheral.registers
    ));

    assert_eq_hex!(0x00CD_AB02, cpu_peripheral.registers.get_full_pc_address());

    run_expectations(
        &mut cpu_peripheral,
        &run_return_from_exception(0x0, (0xABCD, 0xAB00)),
        &mut clocks,
    );

    assert_eq_hex!(0x0000_0002, cpu_peripheral.registers.get_full_pc_address());

    assert!(sr_bit_is_set(
        StatusRegisterFields::ProtectedMode,
        &cpu_peripheral.registers
    ));
}

#[test]
fn test_software_exception_cannot_interrupt_another_software_exception() {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    let mut clocks = 0;

    let software_exception: InstructionData = build_software_exception_instruction();
    let test_instruction: InstructionData = build_test_instruction();

    run_expectations(
        &mut cpu_peripheral,
        &expect_main_program_cycle_with_instruction(0x0000_0000, &software_exception),
        &mut clocks,
    );

    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0x0, 0x70, (0xABCD, 0xAB00)),
        &mut clocks,
    );

    assert_eq_hex!(0x00CD_AB02, cpu_peripheral.registers.get_full_pc_address());

    run_expectations(
        &mut cpu_peripheral,
        &expect_main_program_cycle_with_instruction(0x00CD_AB02, &software_exception),
        &mut clocks,
    );

    assert_eq_hex!(0x00CD_AB04, cpu_peripheral.registers.get_full_pc_address());

    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler_masked(0x0, 0x70, (0x00FF, 0xCCC0)),
        &mut clocks,
    );

    assert_eq_hex!(0x00CD_AB04, cpu_peripheral.registers.get_full_pc_address());

    run_expectations(
        &mut cpu_peripheral,
        &expect_main_program_cycle_with_instruction(0x00CD_AB04, &test_instruction),
        &mut clocks,
    );

    assert_eq!(0x00CD_AB06, cpu_peripheral.registers.get_full_pc_address());
}

// TODO: Unit test software exception priorities
// category=Testing
// It is currently tested by the some example projects but there isn't unit test coverage
// TODO: Test software exception being out of range
// category=Testing
