use peripheral_cpu::coprocessors::processing_unit::encoding::encode_instruction;
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
    },
    new_cpu_peripheral,
    registers::{get_interrupt_mask, FullAddressRegisterAccess},
    CYCLES_PER_INSTRUCTION,
};

use super::common::{set_up_instruction_test, PROGRAM_SEGMENT};

pub const PROGRAM_OFFSET: u32 = 0x00CE_0000;
pub const SYSTEM_RAM_OFFSET: u32 = 0x0;

#[allow(clippy::cast_lossless)]
#[test]
fn test_software_exception_trigger() {
    let exception_code = 0x0F;
    let exception_op_code = 0x1100 | exception_code;

    let initial_copi_instruction = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::CoprocessorCallImmediate,
        register: 0x0, // Unused?
        value: exception_op_code,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    });
    let mem = set_up_instruction_test(&initial_copi_instruction, PROGRAM_OFFSET, SYSTEM_RAM_OFFSET);
    let mut cpu = new_cpu_peripheral(&mem, PROGRAM_SEGMENT);
    cpu.registers.system_ram_offset = SYSTEM_RAM_OFFSET;

    let expected_vector_address = SYSTEM_RAM_OFFSET + (exception_code as u32 * 2);

    // 32 bit vector: write upper word (the program segment)
    mem.write_address(expected_vector_address, (PROGRAM_OFFSET >> 16) as u16);
    // write lower word (the offset in the segment)
    mem.write_address(expected_vector_address + 1, 0xFAFF);
    // 32 bit vector: write upper word (the program segment)

    // First six cycles will run the COPI instruction and load the cause register
    cpu.run_cpu(CYCLES_PER_INSTRUCTION)
        .expect("expected CPU to run six cycles successfully");

    assert_eq!(exception_op_code, cpu.registers.pending_coprocessor_command);
    assert_eq!(0x0, get_interrupt_mask(&cpu.registers));
    assert_eq!(0x0, cpu.eu_registers.exception_level);

    // The next six cycles the exception unit should run and do the actual jump
    cpu.run_cpu(CYCLES_PER_INSTRUCTION)
        .expect("expected CPU to run six cycles successfully");

    assert_eq!(0x1, cpu.eu_registers.exception_level);
    assert_eq!(0x00CE_FAFF, cpu.registers.get_full_pc_address());
    assert_eq!(
        0x00CE_0002,
        cpu.eu_registers.link_registers[1].return_address
    );
    assert_eq!(0x1, get_interrupt_mask(&cpu.registers));

    // This instruction should be ignored because it has the same exception level as the current mask
    // Software exceptions are always level one and cannot interrupt each other (cannot be nested)
    let ignored_copi_instruction = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::CoprocessorCallImmediate,
        register: 0x0, // Unused?
        value: exception_op_code,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    });
    let ignored_copi_instruction_data = encode_instruction(&ignored_copi_instruction);

    mem.load_binary_data_into_segment(PROGRAM_SEGMENT, &ignored_copi_instruction_data.to_vec());
    // Reset PC to check if jump occurs
    cpu.registers.set_full_pc_address(PROGRAM_OFFSET);

    // First six cycles will run the COPI instruction and load the cause register
    cpu.run_cpu(CYCLES_PER_INSTRUCTION)
        .expect("expected CPU to run six cycles successfully");
    // The next six cycles the exception unit should run but ignore the exception
    cpu.run_cpu(CYCLES_PER_INSTRUCTION)
        .expect("expected CPU to run six cycles successfully");

    assert_eq!(0x00CE_0002, cpu.registers.get_full_pc_address());
}
