use peripheral_bus::BusPeripheral;
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
        RegisterInstructionData, ShiftOperand, ShiftType,
    },
    registers::{
        AddressRegisterName, RegisterIndexing, Registers, SegmentedAddress, StatusRegisterFields,
        set_sr_bit,
    },
};

use crate::instructions::common;

use super::common::{
    get_expected_registers, get_non_address_register_index_range, get_register_index_range,
};

// NOTE: LJMP is really just a LDEA with a hardcoded destination of the p address register,
// but I guess these tests will test more jump related logic and the LDEA tests will test
// more offset stuff

#[allow(clippy::cast_sign_loss, clippy::cast_lossless)]
fn test_immediate_branch_instruction(
    initial_pl: (u16, u16),
    offset: i16,
    expected_pl: (u16, u16),
    condition_flag: ConditionFlags,
    initial_status_flags: &Vec<StatusRegisterFields>,
) {
    let instruction_data = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::LoadEffectiveAddressFromIndirectImmediate,
        register: AddressRegisterName::ProgramCounter.to_register_index(),
        value: offset as u16,
        condition_flag,
        additional_flags: AddressRegisterName::ProgramCounter.to_register_index(),
    });
    let (previous, current) = common::run_instruction(
        &instruction_data,
        |registers: &mut Registers, _: &mut BusPeripheral| {
            registers.ph = initial_pl.0;
            registers.pl = initial_pl.1;
            for &status_register_field in initial_status_flags {
                set_sr_bit(status_register_field, registers);
            }
        },
        initial_pl.to_full_address(),
    );
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.ph = expected_pl.0;
            registers.pl = expected_pl.1;
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

// TODO: Check naming of ljmp_test tests (and probably other tests too)
// category=Testing
// Seems to be some copy/paste issues
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_possible_truncation
)]
fn test_immediate_branch_with_subroutine_instruction(
    initial_pl: (u16, u16),
    offset: i16,
    expected_pl: (u16, u16),
    expected_link: (u16, u16),
    condition_flag: ConditionFlags,
    initial_status_flags: &Vec<StatusRegisterFields>,
) {
    let instruction_data = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::LongJumpToSubroutineWithImmediateDisplacement,
        register: AddressRegisterName::ProgramCounter.to_register_index(),
        value: offset as u16,
        condition_flag,
        additional_flags: AddressRegisterName::ProgramCounter.to_register_index(),
    });
    let (previous, current) = common::run_instruction(
        &instruction_data,
        |registers: &mut Registers, _: &mut BusPeripheral| {
            registers.ph = initial_pl.0;
            registers.pl = initial_pl.1;
            for &status_register_field in initial_status_flags {
                set_sr_bit(status_register_field, registers);
            }
        },
        initial_pl.to_full_address(),
    );
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.ph = expected_pl.0;
            registers.pl = expected_pl.1;
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

#[allow(
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_possible_truncation
)]
fn test_register_branch_instruction(
    initial_pl: (u16, u16),
    offset: i16,
    expected_pl: (u16, u16),
    condition_flag: ConditionFlags,
    initial_status_flags: &Vec<StatusRegisterFields>,
) {
    for src_register_index in get_register_index_range() {
        let instruction_data = InstructionData::Register(RegisterInstructionData {
            op_code: Instruction::LoadEffectiveAddressFromIndirectRegister,
            r1: AddressRegisterName::ProgramCounter.to_register_index(),
            r2: AddressRegisterName::ProgramCounter.to_register_index(),
            r3: src_register_index,
            condition_flag,
            additional_flags: AddressRegisterName::ProgramCounter.to_register_index(),
            shift_operand: ShiftOperand::Immediate,
            shift_type: ShiftType::None,
            shift_count: 0x0,
        });
        let (previous, current) = common::run_instruction(
            &instruction_data,
            |registers: &mut Registers, _: &mut BusPeripheral| {
                registers.set_at_index(src_register_index, offset as u16);
                registers.ph = initial_pl.0;
                registers.pl = initial_pl.1;
                for &status_register_field in initial_status_flags {
                    set_sr_bit(status_register_field, registers);
                }
            },
            initial_pl.to_full_address(),
        );
        let expected_registers =
            get_expected_registers(&previous.registers, |registers: &mut Registers| {
                registers.set_at_index(src_register_index, offset as u16);
                registers.ph = expected_pl.0;
                registers.pl = expected_pl.1;
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
}

#[allow(
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::cast_possible_truncation
)]
fn test_register_branch_with_subroutine_instruction(
    initial_pl: (u16, u16),
    offset: i16,
    expected_pl: (u16, u16),
    expected_link: (u16, u16),
    condition_flag: ConditionFlags,
    initial_status_flags: &Vec<StatusRegisterFields>,
) {
    for src_register_index in get_non_address_register_index_range() {
        let instruction_data = InstructionData::Register(RegisterInstructionData {
            op_code: Instruction::LongJumpToSubroutineWithRegisterDisplacement,
            r1: AddressRegisterName::ProgramCounter.to_register_index(),
            r2: AddressRegisterName::ProgramCounter.to_register_index(),
            r3: src_register_index,
            condition_flag,
            additional_flags: AddressRegisterName::ProgramCounter.to_register_index(),
            shift_operand: ShiftOperand::Immediate,
            shift_type: ShiftType::None,
            shift_count: 0x0,
        });
        let (previous, current) = common::run_instruction(
            &instruction_data,
            |registers: &mut Registers, _: &mut BusPeripheral| {
                registers.set_at_index(src_register_index, offset as u16);
                registers.ph = initial_pl.0;
                registers.pl = initial_pl.1;
                for &status_register_field in initial_status_flags {
                    set_sr_bit(status_register_field, registers);
                }
            },
            initial_pl.to_full_address(),
        );
        let expected_registers =
            get_expected_registers(&previous.registers, |registers: &mut Registers| {
                registers.set_at_index(src_register_index, offset as u16);
                registers.ph = expected_pl.0;
                registers.pl = expected_pl.1;
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
}

#[test]
fn test_immediate_branch_basic() {
    test_immediate_branch_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFACE),
        ConditionFlags::Always,
        &vec![],
    );
    test_immediate_branch_instruction(
        (0x00CC, 0xFACE),
        -0x000E,
        (0x00CC, 0xFAC0),
        ConditionFlags::Always,
        &vec![],
    );
    test_immediate_branch_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFACE),
        ConditionFlags::CarrySet,
        &vec![StatusRegisterFields::Carry],
    );

    test_immediate_branch_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFACE),
        ConditionFlags::CarrySet,
        &vec![StatusRegisterFields::Carry],
    );

    test_immediate_branch_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFAC2), // Normal 2 word PC increment
        ConditionFlags::CarryClear,
        &vec![StatusRegisterFields::Carry],
    );

    test_immediate_branch_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFAC2), // Normal 2 word PC increment
        ConditionFlags::CarryClear,
        &vec![StatusRegisterFields::Carry],
    );
}

#[test]
fn test_immediate_branch_overflow() {
    test_immediate_branch_instruction(
        (0xCC, 0xFFFE),
        0x0002,
        (0x00CC, 0x0000),
        ConditionFlags::Always,
        &vec![],
    );
    test_immediate_branch_instruction(
        (0xCC, 0x0000),
        -0x0002,
        (0x00CC, 0xFFFE),
        ConditionFlags::Always,
        &vec![],
    );
}

#[test]
fn test_immediate_branch_with_subroutine_basic() {
    test_immediate_branch_with_subroutine_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFACE),
        (0x00CC, 0xFAC2),
        ConditionFlags::Always,
        &vec![],
    );
    test_immediate_branch_with_subroutine_instruction(
        (0x00CC, 0xFACE),
        -0x000E,
        (0x00CC, 0xFAC0),
        (0x00CC, 0xFAD0),
        ConditionFlags::Always,
        &vec![],
    );
    test_immediate_branch_with_subroutine_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFACE),
        (0x00CC, 0xFAC2),
        ConditionFlags::CarrySet,
        &vec![StatusRegisterFields::Carry],
    );

    test_immediate_branch_with_subroutine_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFACE),
        (0x00CC, 0xFAC2),
        ConditionFlags::CarrySet,
        &vec![StatusRegisterFields::Carry],
    );

    test_immediate_branch_with_subroutine_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFAC2), // Normal 2 word PC increment
        (0x0, 0x0),
        ConditionFlags::CarryClear,
        &vec![StatusRegisterFields::Carry],
    );

    test_immediate_branch_with_subroutine_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFAC2), // Normal 2 word PC increment
        (0x0, 0x0),
        ConditionFlags::CarryClear,
        &vec![StatusRegisterFields::Carry],
    );
}

#[test]
fn test_immediate_branch_with_subroutine_overflow() {
    test_immediate_branch_with_subroutine_instruction(
        (0x00CC, 0xFFFE),
        0x0002,
        (0x00CC, 0x0000),
        (0x00CC, 0x0000),
        ConditionFlags::Always,
        &vec![],
    );
    test_immediate_branch_with_subroutine_instruction(
        (0xCC, 0x0000),
        -0x0001,
        (0x00CC, 0xFFFF),
        (0x00CC, 0x0002),
        ConditionFlags::Always,
        &vec![],
    );
}

// REGISTER

#[test]
fn test_register_branch_basic() {
    test_register_branch_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFACE),
        ConditionFlags::Always,
        &vec![],
    );
    test_register_branch_instruction(
        (0x00CC, 0xFACE),
        -0x000E,
        (0x00CC, 0xFAC0),
        ConditionFlags::Always,
        &vec![],
    );
    test_register_branch_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFACE),
        ConditionFlags::CarrySet,
        &vec![StatusRegisterFields::Carry],
    );

    test_register_branch_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFACE),
        ConditionFlags::CarrySet,
        &vec![StatusRegisterFields::Carry],
    );

    test_register_branch_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFAC2), // Normal 2 word PC increment
        ConditionFlags::CarryClear,
        &vec![StatusRegisterFields::Carry],
    );

    test_register_branch_instruction(
        (0x00CC, 0xFAC0),
        0x000E,
        (0x00CC, 0xFAC2), // Normal 2 word PC increment
        ConditionFlags::CarryClear,
        &vec![StatusRegisterFields::Carry],
    );
}

#[test]
fn test_register_branch_overflow() {
    test_register_branch_instruction(
        (0x00CC, 0xFFFE),
        0x0002,
        (0x00CC, 0x0000),
        ConditionFlags::Always,
        &vec![],
    );
    test_register_branch_instruction(
        (0x00CC, 0x0000),
        -0x0001,
        (0x00CC, 0xFFFF),
        ConditionFlags::Always,
        &vec![],
    );
}

#[test]
fn test_register_branch_with_subroutine_basic() {
    test_register_branch_with_subroutine_instruction(
        (0x00FE, 0xFAC0),
        0x000E,
        (0x00FE, 0xFACE),
        (0x00FE, 0xFAC2),
        ConditionFlags::Always,
        &vec![],
    );
    test_register_branch_with_subroutine_instruction(
        (0x00FE, 0xFACE),
        -0x000E,
        (0x00FE, 0xFAC0),
        (0x00FE, 0xFAD0),
        ConditionFlags::Always,
        &vec![],
    );
    test_register_branch_with_subroutine_instruction(
        (0x00FE, 0xFAC0),
        0x000E,
        (0x00FE, 0xFACE),
        (0x00FE, 0xFAC2),
        ConditionFlags::CarrySet,
        &vec![StatusRegisterFields::Carry],
    );

    test_register_branch_with_subroutine_instruction(
        (0x00FE, 0xFAC0),
        0x000E,
        (0x00FE, 0xFACE),
        (0x00FE, 0xFAC2),
        ConditionFlags::CarrySet,
        &vec![StatusRegisterFields::Carry],
    );

    test_register_branch_with_subroutine_instruction(
        (0x00FE, 0xFAC0),
        0x000E,
        (0x00FE, 0xFAC2), // Normal 2 word PC increment
        (0x0, 0x0),
        ConditionFlags::CarryClear,
        &vec![StatusRegisterFields::Carry],
    );

    test_register_branch_with_subroutine_instruction(
        (0x00FE, 0xFAC0),
        0x000E,
        (0x00FE, 0xFAC2), // Normal 2 word PC increment
        (0x0, 0x0),
        ConditionFlags::CarryClear,
        &vec![StatusRegisterFields::Carry],
    );
}

#[test]
fn test_register_branch_with_subroutine_overflow() {
    test_register_branch_with_subroutine_instruction(
        (0xCC, 0xFFFE),
        0x0002,
        (0x00CC, 0x0000),
        (0x00CC, 0x0000),
        ConditionFlags::Always,
        &vec![],
    );
    test_register_branch_with_subroutine_instruction(
        (0xCC, 0x0000),
        -0x0001,
        (0x00CC, 0xFFFF),
        (0x00CC, 0x0002),
        ConditionFlags::Always,
        &vec![],
    );
}

// TODO: Improve unit test coverage for jump instructions
// category=Testing
// - Test ShiftOperand::Register
// - Deduplicate test functions
// - Check for misaligned jumps (should fault when that is implemented)
