use peripheral_cpu::{
    instructions::{definitions::InstructionData, encoding::encode_instruction},
    new_cpu_peripheral,
    registers::Registers,
    CpuPeripheral, CYCLES_PER_INSTRUCTION,
};
use peripheral_mem::{new_memory_peripheral, MemoryPeripheral};

static PROGRAM_SEGMENT: &str = "PROGRAM";
static SCRATCH_SEGMENT: &str = "SCRATCH";

pub const SCRATCH_SEGMENT_BEGIN: u32 = 0xAAF0;

pub struct TestCpuState {
    pub registers: Registers,
    pub memory_dump: Vec<u8>,
}

fn capture_cpu_state(cpu: &CpuPeripheral) -> TestCpuState {
    TestCpuState {
        registers: cpu.registers.clone(),
        memory_dump: cpu.memory_peripheral.dump_segment(SCRATCH_SEGMENT),
    }
}

pub fn set_up_instruction_test(instruction_data: &InstructionData) -> MemoryPeripheral {
    let mut memory_peripheral = new_memory_peripheral();

    let program_data = encode_instruction(instruction_data);

    memory_peripheral.map_segment(PROGRAM_SEGMENT, 0x0100, 1024, false);
    memory_peripheral.load_binary_data_into_segment(PROGRAM_SEGMENT, program_data.to_vec());
    memory_peripheral.map_segment(SCRATCH_SEGMENT, SCRATCH_SEGMENT_BEGIN, 0x00FF, true);
    memory_peripheral
}

pub fn run_instruction<F>(
    instruction_data: &InstructionData,
    register_setup: F,
) -> (TestCpuState, TestCpuState)
where
    F: Fn(&mut Registers),
{
    let memory_peripheral = set_up_instruction_test(instruction_data);
    let mut cpu = new_cpu_peripheral(&memory_peripheral, PROGRAM_SEGMENT);

    register_setup(&mut cpu.registers);

    let previous = capture_cpu_state(&cpu);
    cpu.run_cpu(CYCLES_PER_INSTRUCTION)
        .expect("expect CPU to run six cycles successfully");
    let current = capture_cpu_state(&cpu);
    (previous, current)
}
