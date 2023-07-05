use peripheral_cpu::{
    instructions::definitions::{
        ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
    },
    registers::{set_sr_bit, AddressRegisterName, RegisterName, Registers, StatusRegisterFields},
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
        op_code: Instruction::BranchWithImmediateDisplacement,
        register: RegisterName::Pl.to_register_index(),
        value: offset as u16,
        condition_flag,
        additional_flags: AddressRegisterName::ProgramCounter.to_register_index(),
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
    assert_eq!(
        expected_registers, current.registers,
        "Not equal:\nleft: {:X?}\nright:{:X?}\n",
        expected_registers, current.registers
    );
}

#[allow(
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_possible_truncation
)]
fn test_immediate_branch_with_subroutine_instruction(
    initial_pl: u16,
    offset: i16,
    expected_pl: u16,
    expected_link: (u16, u16),
    condition_flag: ConditionFlags,
    initial_status_flags: &Vec<StatusRegisterFields>,
) {
    // TODO: Test what happens if high 8 bits are filled in (spoiler alert, the segment mapping fails)
    let default_ph: u16 = 0x00FE;
    let instruction_data = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::BranchToSubroutineWithImmediateDisplacement,
        register: RegisterName::Pl.to_register_index(),
        value: offset as u16,
        condition_flag,
        additional_flags: AddressRegisterName::ProgramCounter.to_register_index(),
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
            registers.lh = expected_link.0;
            registers.ll = expected_link.1;
            for &status_register_field in initial_status_flags {
                set_sr_bit(status_register_field, registers);
            }
        });
    assert_eq!(
        expected_registers, current.registers,
        "Not equal:\nleft: {:X?}\nright:{:X?}\n",
        expected_registers, current.registers
    );
}

#[test]
fn test_immediate_branch_basic() {
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

#[test]
fn test_immediate_branch_overflow() {
    test_immediate_branch_instruction(0xFFFF, 0x0001, 0x0000, ConditionFlags::Always, &vec![]);
    test_immediate_branch_instruction(0x0000, -0x0001, 0xFFFF, ConditionFlags::Always, &vec![]);
}

#[test]
fn test_immediate_branch_with_subroutine_basic() {
    test_immediate_branch_with_subroutine_instruction(
        0xFAC0,
        0x000E,
        0xFACE,
        (0x00FE, 0xFAC2),
        ConditionFlags::Always,
        &vec![],
    );
    test_immediate_branch_with_subroutine_instruction(
        0xFACE,
        -0x000E,
        0xFAC0,
        (0x00FE, 0xFAD0),
        ConditionFlags::Always,
        &vec![],
    );
    test_immediate_branch_with_subroutine_instruction(
        0xFAC0,
        0x000E,
        0xFACE,
        (0x00FE, 0xFAC2),
        ConditionFlags::CarrySet,
        &vec![StatusRegisterFields::Carry],
    );

    test_immediate_branch_with_subroutine_instruction(
        0xFAC0,
        0x000E,
        0xFACE,
        (0x00FE, 0xFAC2),
        ConditionFlags::CarrySet,
        &vec![StatusRegisterFields::Carry],
    );

    test_immediate_branch_with_subroutine_instruction(
        0xFAC0,
        0x000E,
        0xFAC2, // Normal 2 word PC increment
        (0x0, 0x0),
        ConditionFlags::CarryClear,
        &vec![StatusRegisterFields::Carry],
    );

    test_immediate_branch_with_subroutine_instruction(
        0xFAC0,
        0x000E,
        0xFAC2, // Normal 2 word PC increment
        (0x0, 0x0),
        ConditionFlags::CarryClear,
        &vec![StatusRegisterFields::Carry],
    );
}

#[test]
fn test_immediate_branch_with_subroutine_overflow() {
    test_immediate_branch_with_subroutine_instruction(
        0xFFFF,
        0x0001,
        0x0000,
        (0x00FE, 0x0001),
        ConditionFlags::Always,
        &vec![],
    );
    test_immediate_branch_with_subroutine_instruction(
        0x0000,
        -0x0001,
        0xFFFF,
        (0x00FE, 0x0002),
        ConditionFlags::Always,
        &vec![],
    );
}

// TODO: Test ShiftOperand::Register
// TODO: deduplicate test functions
