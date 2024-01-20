use device_ram::new_ram_device_standard;
use peripheral_bus::{new_bus_peripheral, BusPeripheral};
use peripheral_cpu::coprocessors::processing_unit::definitions::InstructionData;
use peripheral_cpu::coprocessors::processing_unit::encoding::encode_instruction;
use peripheral_cpu::new_cpu_peripheral;

pub static VECTOR_SEGMENT: &str = "VECTOR";
pub static PROGRAM_SEGMENT: &str = "PROGRAM";
pub static SCRATCH_SEGMENT: &str = "SCRATCH";

pub const SCRATCH_SEGMENT_BEGIN: u32 = 0x00FA_0000;

// TODO: Dedupe with instruction tests and make more generic?
#[allow(clippy::cast_lossless)]
pub fn set_up_instruction_test(
    instruction_data: &InstructionData,
    program_offset: u32,
    system_ram_offset: u32,
) -> BusPeripheral {
    let mut cpu_peripheral = new_cpu_peripheral(system_ram_offset);
    let mut bus_peripheral = new_bus_peripheral(Box::new(cpu_peripheral));

    let program_data = encode_instruction(instruction_data);

    bus_peripheral.map_segment(
        VECTOR_SEGMENT,
        system_ram_offset,
        0xFF,
        true,
        Box::new(new_ram_device_standard()),
    );
    bus_peripheral.map_segment(
        PROGRAM_SEGMENT,
        program_offset,
        u16::MAX as u32,
        true,
        Box::new(new_ram_device_standard()),
    );
    bus_peripheral.load_binary_data_into_segment(PROGRAM_SEGMENT, &program_data.to_vec());
    bus_peripheral.map_segment(
        SCRATCH_SEGMENT,
        SCRATCH_SEGMENT_BEGIN,
        u16::MAX as u32,
        true,
        Box::new(new_ram_device_standard()),
    );
    bus_peripheral
}
