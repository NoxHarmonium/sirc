use assert_hex::assert_eq_hex;
use peripheral_bus::conversion::bytes_to_words;
use peripheral_bus::device::{BusAssertions, Device};
use peripheral_cpu::coprocessors::processing_unit::definitions::InstructionData;
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ConditionFlags, ImmediateInstructionData, Instruction,
};
use peripheral_cpu::coprocessors::processing_unit::encoding::encode_instruction;
use peripheral_cpu::registers::SegmentedAddress;
use peripheral_cpu::CpuPeripheral;

pub fn build_test_instruction() -> InstructionData {
    InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::AddImmediate,
        register: 0x1,
        value: 0xFA,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    })
}

pub fn build_rete_instruction() -> InstructionData {
    InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::CoprocessorCallImmediate,
        register: 0x0,
        value: 0x1A00,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    })
}

pub struct Expectation {
    pub bus_data: Option<u16>,
    pub bus_assertions: Option<BusAssertions>,
    pub cpu_address: Option<u32>,
    pub cpu_data: Option<u16>,
}

pub fn expectation(
    bus_data: Option<u16>,
    bus_assertions: Option<BusAssertions>,
    cpu_address: Option<u32>,
    cpu_data: Option<u16>,
) -> Expectation {
    Expectation {
        bus_data,
        bus_assertions,
        cpu_address,
        cpu_data,
    }
}

pub fn run_expectations(
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
                    ..expectation.bus_assertions.unwrap_or_default()
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

pub fn expect_main_program_cycle(address: u32) -> Vec<Option<Expectation>> {
    let test_instruction = build_test_instruction();
    expect_main_program_cycle_with_instruction(address, &test_instruction)
}

pub fn expect_main_program_cycle_with_instruction(
    address: u32,
    test_instruction: &InstructionData,
) -> Vec<Option<Expectation>> {
    let test_instruction_bytes: [u8; 4] = encode_instruction(test_instruction);
    let test_instruction_words = bytes_to_words(&test_instruction_bytes);

    // Smoke test - Add 0xFA to r1
    vec![
        Some(expectation(None, None, Some(address), None)),
        Some(expectation(
            Some(test_instruction_words[0]),
            None,
            Some(address + 1),
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

pub fn expect_exception_handler(
    interrupt_assertion: u8,
    vector: u32,
    vector_value: (u16, u16),
) -> Vec<Option<Expectation>> {
    let dummy_instruction = bytes_to_words(&encode_instruction(&build_test_instruction()));
    let masked_vector_value = vector_value.to_full_address() & 0x00FF_FFFF;

    vec![
        // EU reads vector and jumps to it
        Some(expectation(
            None,
            Some(BusAssertions {
                interrupt_assertion,
                ..BusAssertions::default()
            }),
            Some(vector * 2),
            None,
        )),
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
        // Execute instruction at vector address
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

pub fn expect_exception_handler_masked(
    interrupt_assertion: u8,
    vector: u32,
    vector_value: (u16, u16),
) -> Vec<Option<Expectation>> {
    vec![
        // EU reads vector and jumps to it
        Some(expectation(
            None,
            Some(BusAssertions {
                interrupt_assertion,
                ..BusAssertions::default()
            }),
            Some(vector * 2),
            None,
        )),
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
    ]
}

pub fn run_return_from_exception(
    interrupt_assertion: u8,
    vector_value: (u16, u16),
) -> Vec<Option<Expectation>> {
    let return_instruction = bytes_to_words(&encode_instruction(&build_rete_instruction()));
    let masked_vector_value = vector_value.to_full_address() & 0x00FF_FFFF;
    let interrupt_assertion = BusAssertions {
        interrupt_assertion,
        ..BusAssertions::default()
    };

    vec![
        // EU Returns back to main program
        Some(expectation(
            None,
            Some(interrupt_assertion),
            Some(masked_vector_value + 2),
            None,
        )),
        Some(expectation(
            Some(return_instruction[0]),
            Some(interrupt_assertion),
            Some(masked_vector_value + 3),
            None,
        )),
        Some(expectation(
            Some(return_instruction[1]),
            Some(interrupt_assertion),
            None,
            None,
        )),
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
