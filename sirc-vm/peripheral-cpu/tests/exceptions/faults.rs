use assert_hex::assert_eq_hex;
use peripheral_bus::{conversion::bytes_to_words, device::BusAssertions};
use peripheral_cpu::{
    coprocessors::{
        exception_unit::definitions::{
            vectors::{ALIGNMENT_FAULT, BUS_FAULT, SEGMENT_OVERFLOW_FAULT},
            Faults,
        },
        processing_unit::{
            definitions::{ConditionFlags, ImmediateInstructionData, Instruction, InstructionData},
            encoding::encode_instruction,
        },
    },
    new_cpu_peripheral,
    registers::{
        set_sr_bit, sr_bit_is_set, FullAddressRegisterAccess, SegmentedAddress,
        StatusRegisterFields,
    },
};

use crate::exceptions::common::{build_test_instruction, expectation, run_expectations};

use super::common::Expectation;

pub fn build_load_instruction() -> InstructionData {
    InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::LoadRegisterFromIndirectImmediate,
        register: 0x1, // target (r1)
        value: 0xCAFE, // source 2
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x1, // source 1 (a)
    })
}

pub fn expect_dummy_instruction(address: u32) -> Vec<Option<Expectation>> {
    let dummy_instruction = bytes_to_words(&encode_instruction(&build_test_instruction()));
    vec![
        Some(expectation(None, None, Some(address), None)),
        Some(expectation(
            Some(dummy_instruction[0]),
            None,
            Some(address + 1),
            None,
        )),
        Some(expectation(Some(dummy_instruction[1]), None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
    ]
}

pub fn expect_load_instruction() -> Vec<Option<Expectation>> {
    let load_instruction_bytes: [u8; 4] = encode_instruction(&build_load_instruction());
    let load_instruction_words = bytes_to_words(&load_instruction_bytes);
    vec![
        Some(expectation(None, None, Some(0x0000_0000), None)),
        Some(expectation(
            Some(load_instruction_words[0]),
            None,
            Some(0x0000_0001),
            None,
        )),
        Some(expectation(
            Some(load_instruction_words[1]),
            None,
            None,
            None,
        )),
        None,
        None,
        None,
    ]
}

pub fn expect_bus_fault(vector_value: (u16, u16)) -> Vec<Option<Expectation>> {
    let dummy_instruction = bytes_to_words(&encode_instruction(&build_test_instruction()));
    let vector = u32::from(BUS_FAULT);
    let masked_vector_value = vector_value.to_full_address() & 0x00FF_FFFF;
    let load_instruction_bytes: [u8; 4] = encode_instruction(&build_load_instruction());
    let load_instruction_words = bytes_to_words(&load_instruction_bytes);

    vec![
        Some(expectation(None, None, Some(0x0000_0000), None)),
        Some(expectation(
            Some(load_instruction_words[0]),
            None,
            Some(0x0000_0001),
            None,
        )),
        Some(expectation(
            Some(load_instruction_words[1]),
            None,
            None,
            None,
        )),
        None,
        // Bus error when trying to load data from memory
        Some(expectation(
            None,
            Some(BusAssertions {
                address: 0x0000_CAFE,
                op: peripheral_bus::device::BusOperation::Read,
                ..BusAssertions::default()
            }),
            None,
            None,
        )),
        Some(expectation(
            Some(0x0),
            Some(BusAssertions {
                bus_error: true,
                ..BusAssertions::default()
            }),
            None,
            None,
        )),
        // Fetch vector for bus fault (vector is 0x1 so address is 0x2)
        // EU reads vector and jumps to it
        Some(expectation(None, None, Some(vector * 2), None)),
        Some(expectation(
            Some(vector_value.0),
            None,
            Some((vector * 2) + 1),
            None,
        )),
        Some(expectation(Some(vector_value.1), None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
        // PC should be pointing at contents of vector now
        Some(expectation(None, None, Some(masked_vector_value), None)),
        Some(expectation(
            Some(dummy_instruction[0]),
            None,
            Some(masked_vector_value + 1),
            None,
        )),
        Some(expectation(Some(dummy_instruction[1]), None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
    ]
}

pub fn expect_alignment_fault(vector_value: (u16, u16)) -> Vec<Option<Expectation>> {
    let dummy_instruction = bytes_to_words(&encode_instruction(&build_test_instruction()));
    let vector = u32::from(ALIGNMENT_FAULT);
    let masked_vector_value = vector_value.to_full_address() & 0x00FF_FFFF;

    vec![
        // Try to read at misaligned address
        Some(expectation(None, None, Some(0x0000_0001), None)),
        Some(expectation(
            Some(dummy_instruction[0]),
            None,
            // Instruction fetch aborted
            Some(0x0000_0000),
            None,
        )),
        Some(expectation(Some(dummy_instruction[1]), None, None, None)),
        None,
        None,
        None,
        // Handle vector
        Some(expectation(None, None, Some(vector * 2), None)),
        Some(expectation(
            Some(vector_value.0),
            None,
            Some((vector * 2) + 1),
            None,
        )),
        Some(expectation(Some(vector_value.1), None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
        // PC should be pointing at contents of vector now
        Some(expectation(None, None, Some(masked_vector_value), None)),
        Some(expectation(
            Some(dummy_instruction[0]),
            None,
            Some(masked_vector_value + 1),
            None,
        )),
        Some(expectation(Some(dummy_instruction[1]), None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
    ]
}

pub fn expect_segment_overflow_fault(vector_value: (u16, u16)) -> Vec<Option<Expectation>> {
    let dummy_instruction = bytes_to_words(&encode_instruction(&build_test_instruction()));
    let vector = u32::from(SEGMENT_OVERFLOW_FAULT);
    let masked_vector_value = vector_value.to_full_address() & 0x00FF_FFFF;

    vec![
        // Handle vector
        Some(expectation(None, None, Some(vector * 2), None)),
        Some(expectation(
            Some(vector_value.0),
            None,
            Some((vector * 2) + 1),
            None,
        )),
        Some(expectation(Some(vector_value.1), None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
        // PC should be pointing at contents of vector now
        Some(expectation(None, None, Some(masked_vector_value), None)),
        Some(expectation(
            Some(dummy_instruction[0]),
            None,
            Some(masked_vector_value + 1),
            None,
        )),
        Some(expectation(Some(dummy_instruction[1]), None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
    ]
}

#[test]
fn test_bus_fault() {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    let mut clocks = 0;

    // Set protected mode to test if the fault flips into privileged mode.
    set_sr_bit(
        StatusRegisterFields::ProtectedMode,
        &mut cpu_peripheral.registers,
    );

    run_expectations(
        &mut cpu_peripheral,
        &expect_bus_fault((0x00AB, 0xCDE0)),
        &mut clocks,
    );

    assert!(!sr_bit_is_set(
        StatusRegisterFields::ProtectedMode,
        &cpu_peripheral.registers
    ));

    assert_eq_hex!(0x00AB_CDE2, cpu_peripheral.registers.get_full_pc_address());
}

#[test]
fn test_alignment_fault() {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    let mut clocks = 0;

    // Set protected mode to test if the fault flips into privileged mode.
    set_sr_bit(
        StatusRegisterFields::ProtectedMode,
        &mut cpu_peripheral.registers,
    );

    // Misaligned PC
    cpu_peripheral.registers.pl = 0x1;

    run_expectations(
        &mut cpu_peripheral,
        &expect_alignment_fault((0x00AB, 0xCDE0)),
        &mut clocks,
    );

    assert!(!sr_bit_is_set(
        StatusRegisterFields::ProtectedMode,
        &cpu_peripheral.registers
    ));

    assert_eq_hex!(0x00AB_CDE2, cpu_peripheral.registers.get_full_pc_address());
}

#[test]
fn test_segment_overflow_fault_with_load() {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    let mut clocks = 0;

    // Set protected mode to test if the fault flips into privileged mode.
    set_sr_bit(
        StatusRegisterFields::ProtectedMode,
        &mut cpu_peripheral.registers,
    );

    // The test LOAD instruction calculates al + imm so high values like this should force a segment overflow
    cpu_peripheral.registers.al = 0xFFFE;

    // Run load instruction
    run_expectations(&mut cpu_peripheral, &expect_load_instruction(), &mut clocks);

    assert!(
        cpu_peripheral.eu_registers.pending_fault.is_none(),
        "No fault should be raised unless TrapOnAddressOverflow is set"
    );

    // Enable segment overflow
    set_sr_bit(
        StatusRegisterFields::TrapOnAddressOverflow,
        &mut cpu_peripheral.registers,
    );

    // Reset PL
    cpu_peripheral.registers.pl = 0x0;

    // Run load instruction
    run_expectations(&mut cpu_peripheral, &expect_load_instruction(), &mut clocks);

    assert!(
        cpu_peripheral
            .eu_registers
            .pending_fault
            .is_some_and(|fault| fault == Faults::SegmentOverflow),
        "A fault should be raised because TrapOnAddressOverflow is set"
    );

    run_expectations(
        &mut cpu_peripheral,
        &expect_segment_overflow_fault((0x00AB, 0xCDE0)),
        &mut clocks,
    );

    assert!(!sr_bit_is_set(
        StatusRegisterFields::ProtectedMode,
        &cpu_peripheral.registers
    ));

    assert_eq_hex!(0x00AB_CDE2, cpu_peripheral.registers.get_full_pc_address());
}

#[test]
fn test_segment_overflow_fault_with_instruction_fetch() {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    let mut clocks = 0;

    // Set protected mode to test if the fault flips into privileged mode.
    set_sr_bit(
        StatusRegisterFields::ProtectedMode,
        &mut cpu_peripheral.registers,
    );

    // The next instruction should be at 0x0000
    cpu_peripheral.registers.pl = 0xFFFE;

    run_expectations(
        &mut cpu_peripheral,
        &expect_dummy_instruction(0x0000_FFFE),
        &mut clocks,
    );

    assert!(
        cpu_peripheral.eu_registers.pending_fault.is_none(),
        "No fault should be raised unless TrapOnAddressOverflow is set"
    );

    // Enable segment overflow
    set_sr_bit(
        StatusRegisterFields::TrapOnAddressOverflow,
        &mut cpu_peripheral.registers,
    );

    // Reset PL
    cpu_peripheral.registers.pl = 0xFFFE;

    run_expectations(
        &mut cpu_peripheral,
        &expect_dummy_instruction(0x0000_FFFE),
        &mut clocks,
    );

    run_expectations(
        &mut cpu_peripheral,
        &vec![
            Some(expectation(None, None, Some(0x0), None)),
            Some(expectation(
                None,
                None,
                // Fetch aborted
                Some(0x0),
                None,
            )),
            None,
            None,
            None,
            None,
        ],
        &mut clocks,
    );

    assert!(
        cpu_peripheral
            .eu_registers
            .pending_fault
            .is_some_and(|fault| fault == Faults::SegmentOverflow),
        "A fault should be raised because TrapOnAddressOverflow is set"
    );

    run_expectations(
        &mut cpu_peripheral,
        &expect_segment_overflow_fault((0x00AB, 0xCDE0)),
        &mut clocks,
    );

    assert!(!sr_bit_is_set(
        StatusRegisterFields::ProtectedMode,
        &cpu_peripheral.registers
    ));

    assert_eq_hex!(0x00AB_CDE2, cpu_peripheral.registers.get_full_pc_address());
}

// TODO: Test metadata LR
// TODO: What happens if try to raise hardware exception directly with COP? Is it allowed in privileged mode or should it be ignored?
// TODO: Double fault

// pub enum Faults {
//     #[default]
//     Bus = 0x1,
//     Alignment = 0x2,
//     SegmentOverflow = 0x3,
//     InvalidOpCode = 0x4,
//     PrivilegeViolation = 0x5,
//     InstructionTrace = 0x6,
//     LevelFiveInterruptConflict = 0x7,
// }
