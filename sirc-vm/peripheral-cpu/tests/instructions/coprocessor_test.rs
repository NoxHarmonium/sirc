use super::common::set_up_instruction_test;
use peripheral_bus::device::BusAssertions;
use peripheral_cpu::coprocessors::processing_unit::definitions::{
    ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
};

#[test]
fn old_magic_command_no_longer_terminates_the_simulation() {
    let instruction = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::CoprocessorCallImmediate,
        register: 0,
        value: 0x14FF,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0,
    });
    let mut bus = set_up_instruction_test(&instruction, 0x0200);
    super::common::setup_test(&mut bus, |_, _| {}, 0x0200);
    let mut response = BusAssertions::default();
    // Include the command's own slot after the issuing PU instruction. Reserved
    // EU operations keep their current behavior until the dispatch/#447 work.
    for _ in 0..12 {
        response = bus.poll_all(response);
        assert!(!response.exit_simulation);
    }
}
