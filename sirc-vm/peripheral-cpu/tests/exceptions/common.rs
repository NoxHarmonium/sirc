use device_ram::new_ram_device_standard;
use peripheral_cpu::coprocessors::processing_unit::definitions::InstructionData;
use peripheral_cpu::coprocessors::processing_unit::encoding::encode_instruction;
use peripheral_mem::{new_memory_peripheral, MemoryPeripheral};

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
) -> MemoryPeripheral {
    let mut memory_peripheral = new_memory_peripheral();

    let program_data = encode_instruction(instruction_data);

    memory_peripheral.map_segment(
        VECTOR_SEGMENT,
        system_ram_offset,
        0xFF,
        true,
        Box::new(new_ram_device_standard()),
    );
    memory_peripheral.map_segment(
        PROGRAM_SEGMENT,
        program_offset,
        u16::MAX as u32,
        true,
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
