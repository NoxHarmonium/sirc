use peripheral_cpu::coprocessors::processing_unit::encoding::encode_instruction;
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
    },
    new_cpu_peripheral,
    registers::{get_interrupt_mask, FullAddressRegisterAccess},
    CYCLES_PER_INSTRUCTION,
};
use peripheral_mem::helpers::write_bytes;

use super::common::{set_up_instruction_test, PROGRAM_SEGMENT};

pub const PROGRAM_OFFSET: u32 = 0x00CE_0000;
pub const SYSTEM_RAM_OFFSET: u32 = 0x0;
pub const EXCEPTION_JUMP_ADDRESS: u16 = 0xFAFF;

fn build_copi_instruction(exception_code: u16) -> InstructionData {
    let exception_op_code = 0x1100 | exception_code;

    InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::CoprocessorCallImmediate,
        register: 0x0, // Unused?
        value: exception_op_code,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    })
}

fn build_rete_instruction() -> InstructionData {
    let exception_op_code = 0x1A00;

    InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::CoprocessorCallImmediate,
        register: 0x0, // Unused?
        value: exception_op_code,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    })
}

#[allow(clippy::cast_lossless)]
#[test]
fn test_software_exception_trigger() {
    let exception_code = 0x40;
    let exception_op_code = 0x1100 | exception_code;

    let initial_copi_instruction = build_copi_instruction(exception_code);
    let mem = set_up_instruction_test(&initial_copi_instruction, PROGRAM_OFFSET, SYSTEM_RAM_OFFSET);
    let mut cpu = new_cpu_peripheral(&mem, PROGRAM_SEGMENT);
    cpu.registers.system_ram_offset = SYSTEM_RAM_OFFSET;

    let expected_vector_address = SYSTEM_RAM_OFFSET + (exception_code as u32 * 2);
    let vector_target_address = PROGRAM_OFFSET | (EXCEPTION_JUMP_ADDRESS as u32);

    // 32 bit vector: write upper word (the program segment) / write lower word (the offset in the segment)
    write_bytes(
        &mem,
        expected_vector_address,
        &u32::to_be_bytes(PROGRAM_OFFSET | EXCEPTION_JUMP_ADDRESS as u32),
    );

    // First six cycles will run the COPI instruction and load the cause register
    cpu.run_cpu(CYCLES_PER_INSTRUCTION)
        .expect("expected CPU to run six cycles successfully");

    assert_eq!(exception_op_code, cpu.registers.pending_coprocessor_command);
    assert_eq!(0x0, get_interrupt_mask(&cpu.registers));
    assert_eq!(0x0, cpu.eu_registers.pending_hardware_exception_level);

    // The next six cycles the exception unit should run and do the actual jump
    cpu.run_cpu(CYCLES_PER_INSTRUCTION)
        .expect("expected CPU to run six cycles successfully");

    assert_eq!(
        PROGRAM_OFFSET | vector_target_address,
        cpu.registers.get_full_pc_address()
    );
    assert_eq!(0x0, cpu.eu_registers.pending_hardware_exception_level);
    assert_eq!(
        0x00CE_0002,
        cpu.eu_registers.link_registers[0].return_address
    );
    assert_eq!(
        0x0000_0000,
        cpu.eu_registers.link_registers[0].return_status_register
    );
    assert_eq!(0x1, get_interrupt_mask(&cpu.registers));

    // Reset PC to run the software interrupt again
    cpu.registers.set_full_pc_address(PROGRAM_OFFSET);

    // First six cycles will run the COPI instruction and load the cause register
    // The next six cycles the exception unit should run but ignore the exception
    cpu.run_cpu(CYCLES_PER_INSTRUCTION * 2)
        .expect("expected CPU to run twelve cycles successfully");

    // Check that no jump occurred (e.g. PC proceed normally) because an exception is already being processed
    assert_eq!(0x00CE_0002, cpu.registers.get_full_pc_address());
}

#[allow(clippy::cast_lossless)]
#[test]
fn test_software_exception_return() {
    let exception_code = 0x40;

    let initial_copi_instruction = build_copi_instruction(exception_code);
    let mem: peripheral_mem::MemoryPeripheral =
        set_up_instruction_test(&initial_copi_instruction, PROGRAM_OFFSET, SYSTEM_RAM_OFFSET);
    let mut cpu = new_cpu_peripheral(&mem, PROGRAM_SEGMENT);
    cpu.registers.system_ram_offset = SYSTEM_RAM_OFFSET;

    let expected_vector_address = SYSTEM_RAM_OFFSET + (exception_code as u32 * 2);
    let vector_target_address = PROGRAM_OFFSET | (EXCEPTION_JUMP_ADDRESS as u32);

    // 32 bit vector: write upper word (the program segment) / write lower word (the offset in the segment)
    write_bytes(
        &mem,
        expected_vector_address,
        &u32::to_be_bytes(PROGRAM_OFFSET | EXCEPTION_JUMP_ADDRESS as u32),
    );

    let return_instruction = build_rete_instruction();
    let encoded_instruction = encode_instruction(&return_instruction);

    // Write the return instruction to the location that is jumped to
    write_bytes(&mem, vector_target_address, &encoded_instruction);

    // First six cycles sets the cause register - second six cycles performs the jump
    cpu.run_cpu(CYCLES_PER_INSTRUCTION * 2)
        .expect("expected CPU to run twelve cycles successfully");

    assert_eq!(
        PROGRAM_OFFSET | vector_target_address,
        cpu.registers.get_full_pc_address()
    );

    // First six cycles sets the cause register - second six cycles performs the return
    cpu.run_cpu(CYCLES_PER_INSTRUCTION * 2)
        .expect("expected CPU to run twelve cycles successfully");

    // Check it jumped back to the instruction after the original branch
    assert_eq!(0x00CE_0002, cpu.registers.get_full_pc_address());
    assert_eq!(0x0, get_interrupt_mask(&cpu.registers));
}
