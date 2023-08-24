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
    let exception_op_code = 0x1400 | exception_code;

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

    mem.write_address(expected_vector_address, (PROGRAM_OFFSET >> 16) as u16);
    mem.write_address(expected_vector_address + 1, 0xFAFF);

    // First six cycles will run the COPI instruction and load the cause register
    cpu.run_cpu(CYCLES_PER_INSTRUCTION)
        .expect("expected CPU to run six cycles successfully");

    assert_eq!(exception_op_code, cpu.eu_registers.cause_register);
    assert_eq!(0x1, cpu.eu_registers.exception_level);

    // The next six cycles the exception unit should run and do the actual jump
    cpu.run_cpu(CYCLES_PER_INSTRUCTION)
        .expect("expected CPU to run six cycles successfully");

    assert_eq!(0x00CE_FAFF, cpu.registers.get_full_pc_address());
    assert_eq!(
        0x00CE_0002,
        cpu.eu_registers.link_registers[1].return_address
    );
    assert_eq!(0x1, get_interrupt_mask(&cpu.registers));
}
