use assert_hex::assert_eq_hex;
use peripheral_cpu::{
    new_cpu_peripheral,
    registers::{StatusRegisterFields, set_hardware_interrupt_enable, set_sr_bit, sr_bit_is_set},
};

use crate::exceptions::common::{
    expect_exception_handler, expect_main_program_cycle, run_expectations,
    run_return_from_exception,
};

#[test]
fn test_hardware_exception_clears_protected_mode() {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    let mut clocks = 0;

    // Enable all hardware interrupts
    set_hardware_interrupt_enable(&mut cpu_peripheral.registers, 0b11111);

    // Set protected mode to test if the interrupt flips into privileged mode.
    set_sr_bit(
        StatusRegisterFields::ProtectedMode,
        &mut cpu_peripheral.registers,
    );

    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0xF, 0x20, (0xABCD, 0xAB00)),
        &mut clocks,
    );

    assert!(!sr_bit_is_set(
        StatusRegisterFields::ProtectedMode,
        &cpu_peripheral.registers
    ));

    run_expectations(
        &mut cpu_peripheral,
        &run_return_from_exception(0x0, (0xABCD, 0xAB00)),
        &mut clocks,
    );

    assert!(sr_bit_is_set(
        StatusRegisterFields::ProtectedMode,
        &cpu_peripheral.registers
    ));
}

#[test]
fn test_hardware_exception_triggers_in_order() {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    let mut clocks = 0;

    // Enable all hardware interrupts
    set_hardware_interrupt_enable(&mut cpu_peripheral.registers, 0b11111);

    run_expectations(
        &mut cpu_peripheral,
        &expect_main_program_cycle(0x0000_0000),
        &mut clocks,
    );
    assert_eq_hex!(0xFA, cpu_peripheral.registers.r1);

    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0xF, 0x20, (0xABCD, 0xAB00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &run_return_from_exception(0x0, (0xABCD, 0xAB00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0x0, 0x30, (0xDCBA, 0xCD00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &run_return_from_exception(0x0, (0xDCBA, 0xCD00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0x0, 0x40, (0xFAFA, 0xEF00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &run_return_from_exception(0x0, (0xFAFA, 0xEF00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0x0, 0x50, (0xAFAF, 0xAF00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &run_return_from_exception(0x0, (0xAFAF, 0xAF00)),
        &mut clocks,
    );
    assert_eq_hex!(0x0, cpu_peripheral.eu_registers.pending_hardware_exceptions);
}

#[test]
fn test_hardware_exception_trigger_repeats() {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    let mut clocks = 0;

    // Enable all hardware interrupts
    set_hardware_interrupt_enable(&mut cpu_peripheral.registers, 0b11111);

    run_expectations(
        &mut cpu_peripheral,
        &expect_main_program_cycle(0x0000_0000),
        &mut clocks,
    );
    assert_eq_hex!(0xFA, cpu_peripheral.registers.r1);

    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0xF, 0x20, (0xABCD, 0xAB00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &run_return_from_exception(0xF, (0xABCD, 0xAB00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0xF, 0x20, (0xABCD, 0xAB00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &run_return_from_exception(0xF, (0xABCD, 0xAB00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0xF, 0x20, (0xABCD, 0xAB00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &run_return_from_exception(0xF, (0xABCD, 0xAB00)),
        &mut clocks,
    );
}

#[test]
fn test_hardware_exception_higher_priority_interrupts_handler() {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    let mut clocks = 0;

    // Enable all hardware interrupts
    set_hardware_interrupt_enable(&mut cpu_peripheral.registers, 0b11111);

    run_expectations(
        &mut cpu_peripheral,
        &expect_main_program_cycle(0x0000_0000),
        &mut clocks,
    );
    assert_eq_hex!(0xFA, cpu_peripheral.registers.r1);

    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0x3, 0x40, (0xABCD, 0xAB00)),
        &mut clocks,
    );

    run_expectations(
        &mut cpu_peripheral,
        &expect_main_program_cycle(0x00CD_AB02),
        &mut clocks,
    );
    assert_eq_hex!(0x2EE, cpu_peripheral.registers.r1);

    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0x7, 0x30, (0xFFFA, 0xCEEA)),
        &mut clocks,
    );

    run_expectations(
        &mut cpu_peripheral,
        &expect_main_program_cycle(0x00FA_CEEC),
        &mut clocks,
    );
    assert_eq_hex!(0x4E2, cpu_peripheral.registers.r1);

    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0x1F, 0x10, (0xDCBA, 0x00AC)),
        &mut clocks,
    );

    run_expectations(
        &mut cpu_peripheral,
        &expect_main_program_cycle(0x00BA_00AE),
        &mut clocks,
    );
    assert_eq_hex!(0x6D6, cpu_peripheral.registers.r1);
}

// TODO: Unit test hardware exception priorities
// category=Testing
// It is currently tested by the faults example project but there isn't unit test coverage
