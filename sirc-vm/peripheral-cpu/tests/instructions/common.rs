use std::ops::Range;

use peripheral_bus::memory_mapped_device::new_stub_memory_mapped_device;
use peripheral_bus::{BusPeripheral, new_bus_peripheral};
use peripheral_cpu::CYCLES_PER_INSTRUCTION;
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    INSTRUCTION_SIZE_WORDS, InstructionData,
};
use peripheral_cpu::coprocessors::processing_unit::encoding::encode_instruction;
use peripheral_cpu::registers::FullAddress;
use peripheral_cpu::{CpuPeripheral, new_cpu_peripheral, registers::Registers};

static DUMMY_SEGMENT: &str = "DUMMY";
static PROGRAM_SEGMENT: &str = "PROGRAM";
static SCRATCH_SEGMENT: &str = "SCRATCH";

pub const SCRATCH_SEGMENT_BEGIN: u32 = 0x00FA_0000;

pub struct TestCpuState {
    pub registers: Registers,
    pub memory_dump: Vec<u8>,
}

#[allow(clippy::cast_lossless)]
pub fn set_up_instruction_test(
    instruction_data: &InstructionData,
    program_offset: u32,
) -> BusPeripheral {
    // TODO TODO I guess give the bus ownership again instead of reference (FACEPALM)
    let cpu = new_cpu_peripheral(0x0);
    let mut bus_peripheral = new_bus_peripheral(Box::new(cpu));

    let program_data = encode_instruction(instruction_data);

    let mut program_vector = vec![0; ((program_offset & 0xFFFF) * 2) as usize];
    program_vector.extend(program_data);

    // TODO: Work out why tests are trying to access the address 0x0
    // category=Testing
    // This segment is just to suppress "No device was mapped for address [0x0]" warnings
    bus_peripheral.map_segment(
        DUMMY_SEGMENT,
        0x0,
        u16::MAX as u32,
        false,
        Box::new(new_stub_memory_mapped_device()),
    );
    bus_peripheral.map_segment(
        PROGRAM_SEGMENT,
        program_offset,
        u16::MAX as u32,
        false,
        Box::new(new_stub_memory_mapped_device()),
    );
    bus_peripheral.load_binary_data_into_segment(PROGRAM_SEGMENT, &program_vector);
    bus_peripheral.map_segment(
        SCRATCH_SEGMENT,
        SCRATCH_SEGMENT_BEGIN,
        u16::MAX as u32,
        true,
        Box::new(new_stub_memory_mapped_device()),
    );
    bus_peripheral
}

pub fn setup_test<F>(bus: &mut BusPeripheral, register_setup: F, program_offset: u32)
where
    F: Fn(&mut Registers, &mut BusPeripheral),
{
    // TODO: Clean up `setup_test` function`
    // category=Testing
    // - Hack to get tests running again - probably needs a rethink
    // - I think this might be the worst code I've ever written
    let mut registers;
    {
        let cpu: &mut CpuPeripheral = bus
            .bus_master
            .as_any()
            .downcast_mut::<CpuPeripheral>()
            .expect("failed to downcast");

        registers = cpu.registers;
        (registers.ph, registers.pl) = program_offset.to_segmented_address();
        register_setup(&mut registers, bus);
    }
    {
        let cpu: &mut CpuPeripheral = bus
            .bus_master
            .as_any()
            .downcast_mut::<CpuPeripheral>()
            .expect("failed to downcast");

        cpu.registers = registers;
    }
}

pub fn capture_cpu_state(bus: &mut BusPeripheral) -> TestCpuState {
    let cpu: &mut CpuPeripheral = bus
        .bus_master
        .as_any()
        .downcast_mut::<CpuPeripheral>()
        .expect("failed to downcast");

    TestCpuState {
        registers: cpu.registers,
        memory_dump: bus.dump_segment(SCRATCH_SEGMENT),
    }
}

pub fn run_instruction<F>(
    instruction_data: &InstructionData,
    register_setup: F,
    program_offset: u32,
) -> (TestCpuState, TestCpuState)
where
    F: Fn(&mut Registers, &mut BusPeripheral),
{
    let mut bus = set_up_instruction_test(instruction_data, program_offset);
    setup_test(&mut bus, register_setup, program_offset);

    let previous = capture_cpu_state(&mut bus);
    bus.run_full_cycle(CYCLES_PER_INSTRUCTION);
    let current = capture_cpu_state(&mut bus);
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
