use assert_hex::assert_eq_hex;
use log::info;
use peripheral_bus::conversion::bytes_to_words;
use peripheral_bus::device::{BusAssertions, Device};
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
};
use peripheral_cpu::coprocessors::processing_unit::encoding::encode_instruction;
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

struct Expectation {
    pub cycle: usize,
    pub bus_data: Option<u16>,
    pub bus_interrupts: Option<u8>,
    pub cpu_address: Option<u32>,
    pub cpu_data: Option<u16>,
}

fn expectation(
    cycle: usize,
    bus_data: Option<u16>,
    bus_interrupts: Option<u8>,
    cpu_address: Option<u32>,
    cpu_data: Option<u16>,
) -> Expectation {
    Expectation {
        cycle,
        bus_data,
        bus_interrupts,
        cpu_address,
        cpu_data,
    }
}

fn run_expectations(cpu: &mut CpuPeripheral, expectations: &Vec<Option<Expectation>>) -> usize {
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
            info!("Cycle: {}", expectation.cycle);
            if let Some(expected_cpu_address) = expectation.cpu_address {
                assert_eq_hex!(expected_cpu_address, output_bus_assertions.address);
            }
            if let Some(expected_cpu_data) = expectation.cpu_data {
                assert_eq_hex!(expected_cpu_data, output_bus_assertions.data);
            }
        }
    }

    expectations.len()
}

#[allow(clippy::cast_lossless)]
#[test]
fn test_hardware_exception_trigger() {
    let test_instruction = build_test_instruction();
    let test_instruction_bytes: [u8; 4] = encode_instruction(&test_instruction);
    let test_instruction_words = bytes_to_words(&test_instruction_bytes);
    let mut cpu_peripheral = new_cpu_peripheral(0x0);

    // Smoke test - Add 1 to r1
    let expectations = vec![
        Some(expectation(0, None, None, Some(0x0000_0000), None)),
        Some(expectation(
            1,
            Some(test_instruction_words[0]),
            None,
            Some(0x0000_0001),
            None,
        )),
        Some(expectation(
            2,
            Some(test_instruction_words[1]),
            None,
            None,
            None,
        )),
        None,
        None,
        None,
    ];

    run_expectations(&mut cpu_peripheral, &expectations);

    println!("{:X?}", cpu_peripheral.registers);
    assert_eq_hex!(0xFA, cpu_peripheral.registers.r1);

    // Smoke test - Add 1 to r1
    let expectations2 = vec![
        // Level 4 interrupt vector
        Some(expectation(7, None, Some(0x0F), Some(0x0000_0040), None)),
        Some(expectation(8, Some(0xABCD), None, Some(0x0000_0041), None)),
        Some(expectation(9, Some(0xEF00), None, None, None)),
        None,
        None,
        None,
        Some(expectation(13, None, None, Some(0x00CD_EF00), None)),
        Some(expectation(
            14,
            Some(test_instruction_words[0]),
            None,
            Some(0x00CD_EF01),
            None,
        )),
        Some(expectation(
            15,
            Some(test_instruction_words[1]),
            None,
            None,
            None,
        )),
    ];

    run_expectations(&mut cpu_peripheral, &expectations2);
}
