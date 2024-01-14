use std::ops::Range;

use device_ram::new_ram_device_standard;
use peripheral_bus::{new_bus_peripheral, BusPeripheral};
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    InstructionData, INSTRUCTION_SIZE_WORDS,
};
use peripheral_cpu::coprocessors::processing_unit::encoding::encode_instruction;
use peripheral_cpu::{new_cpu_peripheral, registers::Registers, CpuPeripheral};

static PROGRAM_SEGMENT: &str = "PROGRAM";
static SCRATCH_SEGMENT: &str = "SCRATCH";

pub const SCRATCH_SEGMENT_BEGIN: u32 = 0x00FA_0000;

pub struct TestCpuState {
    pub registers: Registers,
    pub memory_dump: Vec<u8>,
}

fn capture_cpu_state(cpu: &CpuPeripheral) -> TestCpuState {
    TestCpuState {
        registers: cpu.registers,
        memory_dump: cpu.memory_peripheral.dump_segment(SCRATCH_SEGMENT),
    }
}

#[allow(clippy::cast_lossless)]
pub fn set_up_instruction_test(
    instruction_data: &InstructionData,
    program_offset: u32,
) -> BusPeripheral {
    let mut memory_peripheral = new_bus_peripheral();

    let program_data = encode_instruction(instruction_data);

    memory_peripheral.map_segment(
        PROGRAM_SEGMENT,
        program_offset,
        u16::MAX as u32,
        false,
        Box::new(new_ram_device_standard()),
    );
    memory_peripheral.load_binary_data_into_segment(PROGRAM_SEGMENT, &program_data.to_vec());
    memory_peripheral.map_segment(
        SCRATCH_SEGMENT,
        SCRATCH_SEGMENT_BEGIN,
        u16::MAX as u32,
        true,
        Box::new(new_ram_device_standard()),
    );
    memory_peripheral
}

pub fn run_instruction<F>(
    instruction_data: &InstructionData,
    register_setup: F,
    program_offset: u32,
) -> (TestCpuState, TestCpuState)
where
    F: Fn(&mut Registers, &BusPeripheral),
{
    let memory_peripheral = set_up_instruction_test(instruction_data, program_offset);
    let mut cpu = new_cpu_peripheral(&memory_peripheral, PROGRAM_SEGMENT);

    println!("run_instruction: {instruction_data:#?}");

    register_setup(&mut cpu.registers, &memory_peripheral);

    let previous = capture_cpu_state(&cpu);
    cpu.run_cpu()
        .expect("expected CPU to run six cycles successfully");
    let current = capture_cpu_state(&cpu);
    (previous, current)
}

#[allow(clippy::cast_possible_truncation)]
pub fn get_expected_registers<F>(previous: &Registers, register_setup: F) -> Registers
where
    F: Fn(&mut Registers),
{
    let mut initial = Registers {
        ph: previous.ph,
        pl: previous.pl.overflowing_add(INSTRUCTION_SIZE_WORDS as u16).0,
        ..Registers::default()
    };
    register_setup(&mut initial);
    initial
}

pub fn get_register_index_range() -> Range<u8> {
    1..14
}

pub fn get_non_address_register_index_range() -> Range<u8> {
    1..8
}

pub fn get_address_register_index_range() -> Range<u8> {
    0..4
}
