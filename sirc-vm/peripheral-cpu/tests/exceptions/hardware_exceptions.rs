use assert_hex::assert_eq_hex;
use peripheral_cpu::coprocessors::exception_unit::definitions::vectors;
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

fn build_test_instruction() -> InstructionData {
    InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::AddImmediate,
        register: 0x0,
        value: 0xFA,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    })
}

#[allow(clippy::cast_lossless)]
#[test]
fn test_hardware_exception_trigger() {
    let initial_instruction = build_test_instruction();
    let mem = set_up_instruction_test(&initial_instruction, PROGRAM_OFFSET, SYSTEM_RAM_OFFSET);
    let mut cpu = new_cpu_peripheral(&mem, PROGRAM_SEGMENT);
    cpu.reset();
    cpu.registers.system_ram_offset = SYSTEM_RAM_OFFSET;

    let expected_vector_address =
        SYSTEM_RAM_OFFSET + (vectors::LEVEL_ONE_HARDWARE_EXCEPTION as u32 * 2);
    let vector_target_address = PROGRAM_OFFSET | (EXCEPTION_JUMP_ADDRESS as u32);

    // 32 bit vector: write upper word (the program segment) / write lower word (the offset in the segment)
    write_bytes(
        &mem,
        expected_vector_address,
        &u32::to_be_bytes(PROGRAM_OFFSET | EXCEPTION_JUMP_ADDRESS as u32),
    );

    cpu.raise_hardware_interrupt(2);

    assert_eq_hex!(0x0, get_interrupt_mask(&cpu.registers));
    assert_eq_hex!(0x2, cpu.eu_registers.pending_hardware_exception_level);

    cpu.run_cpu(CYCLES_PER_INSTRUCTION)
        .expect("expected CPU to run six cycles successfully");

    assert_eq_hex!(0x2, get_interrupt_mask(&cpu.registers));
    assert_eq_hex!(0x0, cpu.eu_registers.pending_hardware_exception_level);

    assert_eq_hex!(
        PROGRAM_OFFSET | vector_target_address,
        cpu.registers.get_full_pc_address()
    );
    println!("{:X?}", cpu.eu_registers.link_registers);
    assert_eq_hex!(
        0x00CE_0000,
        cpu.eu_registers.link_registers[1].return_address
    );
    assert_eq_hex!(
        0x0000_0000,
        cpu.eu_registers.link_registers[1].return_status_register
    );
}
