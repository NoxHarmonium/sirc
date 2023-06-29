use peripheral_cpu::{
    instructions::definitions::{
        ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
    },
    registers::{set_sr_bit, Registers, StatusRegisterFields},
};

use crate::instructions::common;

use super::common::get_expected_registers;

#[allow(clippy::cast_sign_loss, clippy::cast_lossless)]
fn test_immediate_branch_instruction(
    initial_pl: u16,
    offset: i16,
    expected_pl: u16,
    condition_flag: ConditionFlags,
    initial_status_flags: &Vec<StatusRegisterFields>,
) {
    // TODO: Test what happens if high 8 bits are filled in (spoiler alert, the segment mapping fails)
    let default_ph: u16 = 0x00FE;
    let instruction_data = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::BranchImmediate,
        register: 0x0, // Unused
        value: offset as u16,
        condition_flag,
        additional_flags: 0x0, // Unused
    });
    let (previous, current) = common::run_instruction(
        &instruction_data,
        |registers: &mut Registers| {
            registers.ph = default_ph;
            registers.pl = initial_pl;
            for &status_register_field in initial_status_flags {
                set_sr_bit(status_register_field, registers);
            }
        },
        (default_ph as u32) << u16::BITS | (initial_pl as u32),
    );
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.ph = default_ph;
            registers.pl = expected_pl;
            for &status_register_field in initial_status_flags {
                set_sr_bit(status_register_field, registers);
            }
        });
    assert_eq!(expected_registers, current.registers);
}

#[test]
fn test_add_immediate_basic() {
    test_immediate_branch_instruction(0xFAC0, 0x000E, 0xFACE, ConditionFlags::Always, &vec![]);
    test_immediate_branch_instruction(0xFACE, -0x000E, 0xFAC0, ConditionFlags::Always, &vec![]);
    test_immediate_branch_instruction(
        0xFAC0,
        0x000E,
        0xFACE,
        ConditionFlags::CarrySet,
        &vec![StatusRegisterFields::Carry],
    );

    test_immediate_branch_instruction(
        0xFAC0,
        0x000E,
        0xFACE,
        ConditionFlags::CarrySet,
        &vec![StatusRegisterFields::Carry],
    );

    test_immediate_branch_instruction(
        0xFAC0,
        0x000E,
        0xFAC2, // Normal 2 word PC increment
        ConditionFlags::CarryClear,
        &vec![StatusRegisterFields::Carry],
    );

    test_immediate_branch_instruction(
        0xFAC0,
        0x000E,
        0xFAC2, // Normal 2 word PC increment
        ConditionFlags::CarryClear,
        &vec![StatusRegisterFields::Carry],
    );
}
