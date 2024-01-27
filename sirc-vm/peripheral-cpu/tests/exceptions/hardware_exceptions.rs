use assert_hex::assert_eq_hex;
use peripheral_bus::conversion::bytes_to_words;
use peripheral_bus::device::{BusAssertions, Device};
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
};
use peripheral_cpu::coprocessors::processing_unit::encoding::encode_instruction;
use peripheral_cpu::registers::SegmentedAddress;
use peripheral_cpu::{new_cpu_peripheral, CpuPeripheral};

fn build_test_instruction() -> InstructionData {
    InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::AddImmediate,
        register: 0x1,
        value: 0xFA,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    })
}

fn build_rete_instruction() -> InstructionData {
    InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::CoprocessorCallImmediate,
        register: 0x0,
        value: 0x1A00,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    })
}

struct Expectation {
    pub bus_data: Option<u16>,
    pub bus_interrupts: Option<u8>,
    pub cpu_address: Option<u32>,
    pub cpu_data: Option<u16>,
}

fn expectation(
    bus_data: Option<u16>,
    bus_interrupts: Option<u8>,
    cpu_address: Option<u32>,
    cpu_data: Option<u16>,
) -> Expectation {
    Expectation {
        bus_data,
        bus_interrupts,
        cpu_address,
        cpu_data,
    }
}

fn run_expectations(
    cpu: &mut CpuPeripheral,
    expectations: &Vec<Option<Expectation>>,
    clock_count: &mut usize,
) {
    for expectation in expectations {
        let input_bus_assertions =
            expectation
                .as_ref()
                .map_or_else(BusAssertions::default, |expectation| BusAssertions {
                    data: expectation.bus_data.unwrap_or(0),
                    interrupt_assertion: expectation.bus_interrupts.unwrap_or(0),
                    ..BusAssertions::default() // TODO: should we assert on op?
                });
        let output_bus_assertions = cpu.poll(input_bus_assertions, true);
        if let Some(expectation) = expectation {
            if let Some(expected_cpu_address) = expectation.cpu_address {
                assert_eq_hex!(
                    expected_cpu_address,
                    output_bus_assertions.address,
                    "failed assertion at clock {clock_count}"
                );
            }
            if let Some(expected_cpu_data) = expectation.cpu_data {
                assert_eq_hex!(
                    expected_cpu_data,
                    output_bus_assertions.data,
                    "failed assertion at clock {clock_count}"
                );
            }
        }
        *clock_count += 1;
    }
}

fn expect_main_program_cycle() -> Vec<Option<Expectation>> {
    let test_instruction = build_test_instruction();
    let test_instruction_bytes: [u8; 4] = encode_instruction(&test_instruction);
    let test_instruction_words = bytes_to_words(&test_instruction_bytes);

    // Smoke test - Add 1 to r1
    vec![
        Some(expectation(None, None, Some(0x0000_0000), None)),
        Some(expectation(
            Some(test_instruction_words[0]),
            None,
            Some(0x0000_0001),
            None,
        )),
        Some(expectation(
            Some(test_instruction_words[1]),
            None,
            None,
            None,
        )),
        None,
        None,
        None,
    ]
}

fn expect_exception_handler(
    interrupt_assertion: u8,
    vector: u32,
    vector_value: (u16, u16),
) -> Vec<Option<Expectation>> {
    let dummy_instruction = bytes_to_words(&encode_instruction(&build_test_instruction()));
    let return_instruction = bytes_to_words(&encode_instruction(&build_rete_instruction()));
    let masked_vector_value = vector_value.to_full_address() & 0x00FF_FFFF;

    vec![
        // EU reads vector and jumps to it
        Some(expectation(
            None,
            Some(interrupt_assertion),
            Some(vector * 2),
            None,
        )),
        Some(expectation(
            Some(vector_value.0),
            Some(interrupt_assertion),
            Some((vector * 2) + 1),
            None,
        )),
        Some(expectation(
            Some(vector_value.1),
            Some(interrupt_assertion),
            None,
            None,
        )),
        Some(expectation(None, Some(interrupt_assertion), None, None)),
        Some(expectation(None, Some(interrupt_assertion), None, None)),
        Some(expectation(None, Some(interrupt_assertion), None, None)),
        // Execute instruction at vector address
        Some(expectation(None, None, Some(masked_vector_value), None)),
        Some(expectation(
            Some(dummy_instruction[0]),
            None,
            Some(masked_vector_value + 1),
            None,
        )),
        Some(expectation(Some(dummy_instruction[1]), None, None, None)),
        Some(expectation(None, Some(interrupt_assertion), None, None)),
        Some(expectation(None, Some(interrupt_assertion), None, None)),
        Some(expectation(None, Some(interrupt_assertion), None, None)),
        // EU Returns back to main program
        Some(expectation(None, None, Some(masked_vector_value + 2), None)),
        Some(expectation(
            Some(return_instruction[0]),
            None,
            Some(masked_vector_value + 3),
            None,
        )),
        Some(expectation(Some(return_instruction[1]), None, None, None)),
        Some(expectation(None, Some(interrupt_assertion), None, None)),
        Some(expectation(None, Some(interrupt_assertion), None, None)),
        Some(expectation(None, Some(interrupt_assertion), None, None)),
        // Execute the RETE
        Some(expectation(None, Some(interrupt_assertion), None, None)),
        Some(expectation(None, Some(interrupt_assertion), None, None)),
        Some(expectation(None, Some(interrupt_assertion), None, None)),
        Some(expectation(None, Some(interrupt_assertion), None, None)),
        Some(expectation(None, Some(interrupt_assertion), None, None)),
        Some(expectation(None, Some(interrupt_assertion), None, None)),
    ]
}

#[test]
fn test_hardware_exception_triggers_in_order() {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    let mut clocks = 0;

    run_expectations(
        &mut cpu_peripheral,
        &expect_main_program_cycle(),
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
        &expect_exception_handler(0x7, 0x30, (0xDCBA, 0xCD00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0x3, 0x40, (0xFAFA, 0xEF00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0x1, 0x50, (0xAFAF, 0xAF00)),
        &mut clocks,
    );
}

#[test]
fn test_hardware_exception_trigger_repeats() {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    let mut clocks = 0;

    run_expectations(
        &mut cpu_peripheral,
        &expect_main_program_cycle(),
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
        &expect_exception_handler(0xF, 0x20, (0xABCD, 0xAB00)),
        &mut clocks,
    );
    run_expectations(
        &mut cpu_peripheral,
        &expect_exception_handler(0xF, 0x20, (0xABCD, 0xAB00)),
        &mut clocks,
    );
}

// TODO: Test that a higher priority exception overrides a lower level handler
// TODO: Test software exceptions
// TODO: Test fault priorities
