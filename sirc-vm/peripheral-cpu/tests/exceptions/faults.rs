use assert_hex::assert_eq_hex;
use peripheral_bus::{conversion::bytes_to_words, device::BusAssertions};
use peripheral_cpu::{
    coprocessors::{
        exception_unit::definitions::{
            vectors::{
                ALIGNMENT_FAULT, BUS_FAULT, DOUBLE_FAULT_VECTOR, INVALID_OPCODE_FAULT,
                SEGMENT_OVERFLOW_FAULT,
            },
            Faults,
        },
        processing_unit::{
            definitions::{ConditionFlags, ImmediateInstructionData, Instruction, InstructionData},
            encoding::encode_instruction,
        },
    },
    decode_fault_metadata_register, encode_fault_metadata_register, new_cpu_peripheral,
    registers::{
        set_sr_bit, sr_bit_is_set, FullAddressRegisterAccess, SegmentedAddress,
        StatusRegisterFields,
    },
    FaultMetadataRegister,
};

// TODO: Make sure that all test files have the same suffix
// category=Testing

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

pub fn build_invalid_opcode_instruction() -> InstructionData {
    InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::CoprocessorCallImmediate,
        register: 0x0, // Unused
        value: 0xFFFF, // COP ID: F, Opcode: F, Value FF
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0, // unused
    })
}

pub fn expect_instruction(
    instruction_data: &InstructionData,
    address: u32,
    abort_fetch: bool,
) -> Vec<Option<Expectation>> {
    let dummy_instruction = bytes_to_words(&encode_instruction(instruction_data));
    vec![
        Some(expectation(None, None, Some(address), None)),
        Some(expectation(
            Some(dummy_instruction[0]),
            None,
            Some(if abort_fetch { 0x0 } else { address + 1 }),
            None,
        )),
        Some(expectation(Some(dummy_instruction[1]), None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
        Some(expectation(None, None, None, None)),
    ]
}

pub fn expect_dummy_instruction(address: u32, abort_fetch: bool) -> Vec<Option<Expectation>> {
    expect_instruction(&build_test_instruction(), address, abort_fetch)
}

pub fn expect_invalid_opcode_instruction(
    address: u32,
    abort_fetch: bool,
) -> Vec<Option<Expectation>> {
    expect_instruction(&build_invalid_opcode_instruction(), address, abort_fetch)
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

pub fn expect_fault(vector: u8, vector_value: (u16, u16)) -> Vec<Option<Expectation>> {
    let dummy_instruction = bytes_to_words(&encode_instruction(&build_test_instruction()));
    let masked_vector_value = vector_value.to_full_address() & 0x00FF_FFFF;

    vec![
        // Handle vector
        Some(expectation(None, None, Some(u32::from(vector) * 2), None)),
        Some(expectation(
            Some(vector_value.0),
            None,
            Some((u32::from(vector) * 2) + 1),
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
        &expect_dummy_instruction(0x1, true),
        &mut clocks,
    );

    run_expectations(
        &mut cpu_peripheral,
        &expect_fault(ALIGNMENT_FAULT, (0x00AB, 0xCDE0)),
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
        &expect_fault(SEGMENT_OVERFLOW_FAULT, (0x00AB, 0xCDE0)),
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
        &expect_dummy_instruction(0x0000_FFFE, false),
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
        &expect_dummy_instruction(0x0000_FFFE, false),
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
        &expect_fault(SEGMENT_OVERFLOW_FAULT, (0x00AB, 0xCDE0)),
        &mut clocks,
    );

    assert!(!sr_bit_is_set(
        StatusRegisterFields::ProtectedMode,
        &cpu_peripheral.registers
    ));

    assert_eq_hex!(0x00AB_CDE2, cpu_peripheral.registers.get_full_pc_address());
}

#[test]
fn test_invalid_opcode_fault() {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    let mut clocks = 0;

    run_expectations(
        &mut cpu_peripheral,
        &expect_invalid_opcode_instruction(0x0, false),
        &mut clocks,
    );
    // Let the exception executor handle the COP call
    run_expectations(
        &mut cpu_peripheral,
        &vec![None, None, None, None, None, None],
        &mut clocks,
    );

    assert!(
        cpu_peripheral
            .eu_registers
            .pending_fault
            .is_some_and(|fault| fault == Faults::InvalidOpCode),
        "A fault should be raised because F is an invalid coprocessor ID for this CPU model"
    );

    // Set protected mode to test if the fault flips into privileged mode.
    set_sr_bit(
        StatusRegisterFields::ProtectedMode,
        &mut cpu_peripheral.registers,
    );

    run_expectations(
        &mut cpu_peripheral,
        &expect_fault(INVALID_OPCODE_FAULT, (0x00AB, 0xCDE0)),
        &mut clocks,
    );

    assert!(!sr_bit_is_set(
        StatusRegisterFields::ProtectedMode,
        &cpu_peripheral.registers
    ));

    assert_eq_hex!(0x00AB_CDE2, cpu_peripheral.registers.get_full_pc_address());
}

#[test]
fn test_double_fault() {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    let mut clocks = 0;

    run_expectations(
        &mut cpu_peripheral,
        &expect_invalid_opcode_instruction(0x0, false),
        &mut clocks,
    );
    // Let the exception executor handle the COP call
    run_expectations(
        &mut cpu_peripheral,
        &vec![None, None, None, None, None, None],
        &mut clocks,
    );

    assert!(
        cpu_peripheral
            .eu_registers
            .pending_fault
            .is_some_and(|fault| fault == Faults::InvalidOpCode),
        "A fault should be raised because F is an invalid coprocessor ID for this CPU model"
    );

    // Set protected mode to test if the fault flips into privileged mode.
    set_sr_bit(
        StatusRegisterFields::ProtectedMode,
        &mut cpu_peripheral.registers,
    );

    run_expectations(
        &mut cpu_peripheral,
        &expect_fault(INVALID_OPCODE_FAULT, (0x00AB, 0xCDE0)),
        &mut clocks,
    );

    assert!(!sr_bit_is_set(
        StatusRegisterFields::ProtectedMode,
        &cpu_peripheral.registers
    ));

    assert_eq_hex!(0x00AB_CDE2, cpu_peripheral.registers.get_full_pc_address());
    assert_eq!(
        FaultMetadataRegister {
            phase: 0,
            double_fault: false,
            fault: Faults::InvalidOpCode,
            // Still set when not a double fault to make hardware less complex - doesn't hurt I guess
            original_fault: Faults::InvalidOpCode
        },
        decode_fault_metadata_register(
            cpu_peripheral.eu_registers.link_registers[7].return_status_register,
        ),
    );
    assert_eq_hex!(
        0x0000_0002,
        cpu_peripheral.eu_registers.link_registers[6].return_address
    );
    assert_eq_hex!(
        // No bus address because invalid opcode happens before coprocessors are engaged
        0x0000_0000,
        cpu_peripheral.eu_registers.link_registers[7].return_address
    );

    // Second fault
    run_expectations(
        &mut cpu_peripheral,
        &expect_invalid_opcode_instruction(0x00AB_CDE2, false),
        &mut clocks,
    );
    // Let the exception executor handle the COP call
    run_expectations(
        &mut cpu_peripheral,
        &vec![None, None, None, None, None, None],
        &mut clocks,
    );
    assert!(
        cpu_peripheral
            .eu_registers
            .pending_fault
            .is_some_and(|fault| fault == Faults::DoubleFault),
        "A double fault should be raised because one is already being handled"
    );

    // We faulted inside a fault so now we jump straight to the double fault vector for last rites (e.g. dumping the registers)
    // and probable reset
    run_expectations(
        &mut cpu_peripheral,
        &expect_fault(DOUBLE_FAULT_VECTOR, (0x00BA, 0xCDE0)),
        &mut clocks,
    );

    assert_eq_hex!(0x00BA_CDE2, cpu_peripheral.registers.get_full_pc_address());
    assert_eq!(
        FaultMetadataRegister {
            phase: 0,
            double_fault: true,
            fault: Faults::DoubleFault,
            original_fault: Faults::InvalidOpCode,
        },
        decode_fault_metadata_register(
            cpu_peripheral.eu_registers.link_registers[7].return_status_register,
        ),
    );
    assert_eq_hex!(
        0x00AB_CDE4,
        cpu_peripheral.eu_registers.link_registers[6].return_address
    );
    assert_eq_hex!(
        // No bus address because invalid opcode happens before coprocessors are engaged
        0x0000_0000,
        cpu_peripheral.eu_registers.link_registers[7].return_address
    );
}

#[test]
fn test_fault_metadata_register_encode_decode() {
    // Test all combinations of fields to ensure encoding/decoding is correct
    let test_cases = vec![
        // phase, double_fault, fault, original_fault
        (0, false, Faults::Bus, Faults::Bus),
        (1, false, Faults::Alignment, Faults::SegmentOverflow),
        (2, true, Faults::InvalidOpCode, Faults::DoubleFault),
        (3, false, Faults::PrivilegeViolation, Faults::Bus),
        (4, true, Faults::InstructionTrace, Faults::Alignment),
        (
            5,
            false,
            Faults::LevelFiveInterruptConflict,
            Faults::InvalidOpCode,
        ),
        (7, true, Faults::DoubleFault, Faults::PrivilegeViolation),
        // Edge case: all zeros (except Bus=1 is the enum default)
        (0, false, Faults::Bus, Faults::Bus),
        // Edge case: max values
        (7, true, Faults::DoubleFault, Faults::DoubleFault),
    ];

    for (phase, double_fault, fault, original_fault) in test_cases {
        let original_reg = FaultMetadataRegister {
            phase,
            double_fault,
            fault,
            original_fault,
        };

        // Encode to u16
        let encoded = encode_fault_metadata_register(&original_reg);

        // Decode back to struct
        let decoded = decode_fault_metadata_register(encoded);

        // Verify all fields match
        assert_eq!(
            decoded.phase, phase,
            "Phase mismatch for input ({phase}, {double_fault}, {fault:?}, {original_fault:?})"
        );
        assert_eq!(
                decoded.double_fault, double_fault,
                "Double fault flag mismatch for input ({phase}, {double_fault}, {fault:?}, {original_fault:?})"
            );
        assert_eq!(
            decoded.fault as u16, fault as u16,
            "Fault mismatch for input ({phase}, {double_fault}, {fault:?}, {original_fault:?})"
        );
        assert_eq!(
                decoded.original_fault as u16, original_fault as u16,
                "Original fault mismatch for input ({phase}, {double_fault}, {fault:?}, {original_fault:?})"
            );
    }
}

#[test]
fn test_fault_metadata_register_bit_layout() {
    // Verify the bit layout is correct by checking specific patterns

    // Test phase bits (0-2)
    let reg = FaultMetadataRegister {
        phase: 0b111,
        double_fault: false,
        fault: Faults::Bus,
        original_fault: Faults::Bus,
    };
    let encoded = encode_fault_metadata_register(&reg);
    assert_eq!(encoded & 0x7, 0b111, "Phase should occupy bits 0-2");

    // Test double_fault bit (3)
    let reg = FaultMetadataRegister {
        phase: 0,
        double_fault: true,
        fault: Faults::Bus,
        original_fault: Faults::Bus,
    };
    let encoded = encode_fault_metadata_register(&reg);
    assert_eq!(encoded & 0x8, 0x8, "Double fault should occupy bit 3");

    // Test fault bits (4-7)
    let reg = FaultMetadataRegister {
        phase: 0,
        double_fault: false,
        fault: Faults::DoubleFault, // = 0x8
        original_fault: Faults::Bus,
    };
    let encoded = encode_fault_metadata_register(&reg);
    assert_eq!(
        (encoded & 0xF0) >> 4,
        0x8,
        "Fault 0x8 should occupy bits 4-7"
    );

    // Test with another fault value
    let reg = FaultMetadataRegister {
        phase: 0,
        double_fault: false,
        fault: Faults::LevelFiveInterruptConflict, // = 0x7
        original_fault: Faults::Bus,
    };
    let encoded = encode_fault_metadata_register(&reg);
    assert_eq!(
        (encoded & 0xF0) >> 4,
        0x7,
        "Fault 0x7 should occupy bits 4-7"
    );

    // Test original_fault bits (8-11)
    let reg = FaultMetadataRegister {
        phase: 0,
        double_fault: false,
        fault: Faults::Bus,
        original_fault: Faults::InstructionTrace, // = 0x6
    };
    let encoded = encode_fault_metadata_register(&reg);
    assert_eq!(
        (encoded & 0xF00) >> 8,
        0x6,
        "Original fault 0x6 should occupy bits 8-11"
    );

    // Test with DoubleFault in original_fault
    let reg = FaultMetadataRegister {
        phase: 0,
        double_fault: false,
        fault: Faults::Bus,
        original_fault: Faults::DoubleFault, // = 0x8
    };
    let encoded = encode_fault_metadata_register(&reg);
    assert_eq!(
        (encoded & 0xF00) >> 8,
        0x8,
        "Original fault 0x8 should occupy bits 8-11"
    );
}

// TODO: Test the metadata part of the exception link register
// category=Testing
// TODO: Clarify what happens if hardware exception is raised via COP instruction
// category=Hardware
// Is it allowed in privileged mode or should it be ignored?
// TODO: Clarify how double faults work
// category=Hardware
