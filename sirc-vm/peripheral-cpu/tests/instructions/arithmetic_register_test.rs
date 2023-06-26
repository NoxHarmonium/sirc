use std::ops::Range;

use peripheral_cpu::{
    self,
    instructions::definitions::{
        ConditionFlags, Instruction, InstructionData, ShiftOperand, ShiftType,
        ShortImmediateInstructionData, StatusRegisterUpdateSource, INSTRUCTION_SIZE_WORDS,
    },
    registers::{set_sr_bit, sr_bit_is_set, RegisterIndexing, Registers, StatusRegisterFields},
};

use super::common;

fn get_register_index_range() -> Range<u8> {
    0..13
}

fn get_expected_registers<F>(previous: &Registers, register_setup: F) -> Registers
where
    F: Fn(&mut Registers),
{
    let mut initial = Registers {
        ph: previous.ph,
        pl: previous.pl + (INSTRUCTION_SIZE_WORDS as u16),
        ..Registers::default()
    };
    register_setup(&mut initial);
    initial
}

fn test_short_immediate_arithmetic_instruction(
    instruction: Instruction,
    target_register: u8,
    register_value: u16,
    immediate_value: u8,
    shift_operand: ShiftOperand,
    shift_type: ShiftType,
    shift_count: u8,
    expected_value: u16,
    initial_status_flags: Vec<StatusRegisterFields>,
    expected_status_flags: Vec<StatusRegisterFields>,
) {
    let instruction_data = InstructionData::ShortImmediate(ShortImmediateInstructionData {
        op_code: instruction,
        register: target_register,
        value: immediate_value,
        shift_operand,
        shift_type,
        shift_count,
        condition_flag: ConditionFlags::Always,
        additional_flags: if instruction == Instruction::ShiftShortImmediate {
            StatusRegisterUpdateSource::Shift as u8
        } else {
            StatusRegisterUpdateSource::Alu as u8
        },
    });
    let (previous, current) =
        common::run_instruction(&instruction_data, |registers: &mut Registers| {
            registers.set_at_index(target_register, register_value);
            for &status_register_field in &initial_status_flags {
                set_sr_bit(status_register_field, registers);
            }
        });
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.set_at_index(target_register, expected_value);
            for &status_register_field in &expected_status_flags {
                set_sr_bit(status_register_field, registers);
            }
        });
    assert_eq!(expected_registers, current.registers);
    for &status_register_field in &expected_status_flags {
        assert!(sr_bit_is_set(status_register_field, &current.registers));
    }
}

//
// #### ADDI ####
//

#[test]
fn test_add_immediate_basic() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0x1111,
            0x01,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x1112,
            // Test flag clearing (these flags do not reflect the initial register value)
            vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            vec![],
        );
    }
}

#[test]
fn test_add_immediate_unsigned_overflow() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0xFFFF,
            0x0001,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x0000,
            vec![],
            vec![StatusRegisterFields::Carry, StatusRegisterFields::Zero],
        );
    }
}

#[test]
fn test_add_immediate_signed_overflow() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0x7FFF,
            0x20,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            8,
            0x9FFF,
            vec![],
            vec![
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Negative,
            ],
        );
    }
}
#[test]
fn test_add_immediate_both_overflow() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0x9FFF,
            0x90,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            8,
            0x2FFF,
            vec![],
            vec![StatusRegisterFields::Carry, StatusRegisterFields::Overflow],
        );
    }
}

//
// #### ADCI ####
//

#[test]
fn test_add_immediate_with_carry_basic() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediateWithCarry,
            register_index,
            0x2212,
            0x01,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x2214,
            // Test flag clearing (these flags do not reflect the initial register value)
            vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            vec![],
        );
    }
}

#[test]
fn test_add_immediate_with_carry_unsigned_overflow() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediateWithCarry,
            register_index,
            0xFFFF,
            0x01,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x0000,
            vec![],
            vec![StatusRegisterFields::Carry, StatusRegisterFields::Zero],
        );
    }
}

#[test]
fn test_add_immediate_with_carry_signed_overflow() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediateWithCarry,
            register_index,
            0x7FFF,
            0x20,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            8,
            0x9FFF,
            vec![],
            vec![
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Negative,
            ],
        );
    }
}
#[test]
fn test_add_immediate_with_carry_both_overflow() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediateWithCarry,
            register_index,
            0x9FFF,
            0x90,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            8,
            0x2FFF,
            vec![],
            vec![StatusRegisterFields::Carry, StatusRegisterFields::Overflow],
        );
    }
}

//
// #### SUBI ####
//

#[test]
fn test_subtract_immediate_basic() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::SubtractShortImmediate,
            register_index,
            0x5245,
            0x43,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x5202,
            // Test flag clearing (these flags do not reflect the initial register value)
            vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            vec![],
        );
    }
}

//
// #### SBCI ####
//

#[test]
fn test_subtract_immediate_with_carry_basic() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::SubtractShortImmediateWithCarry,
            register_index,
            0x5245,
            0x43,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x5201,
            // Test flag clearing (these flags do not reflect the initial register value)
            vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            vec![],
        );
    }
}

#[test]
fn test_subtract_immediate_with_carry_unsigned_overflow() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::SubtractShortImmediateWithCarry,
            register_index,
            0x5F00,
            0xFF,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            8,
            0x6000,
            vec![],
            vec![StatusRegisterFields::Carry],
        );
    }
}

#[test]
fn test_subtract_immediate_with_carry_signed_overflow() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::SubtractShortImmediateWithCarry,
            register_index,
            0xDF00,
            0x7F,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            8,
            0x6000,
            vec![],
            vec![StatusRegisterFields::Overflow],
        );
    }
}
#[test]
fn test_subtract_immediate_with_carry_both_overflow() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::SubtractShortImmediateWithCarry,
            register_index,
            0x5F00,
            0xBF,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            8,
            0xA000,
            vec![],
            vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Negative,
            ],
        );
    }
}

//
// #### ANDI ####
//

#[test]
fn test_and_immediate() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AndShortImmediate,
            register_index,
            0xF0F0,
            0xF0,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x00F0,
            // Test flag clearing (these flags do not reflect the initial register value)
            vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            vec![],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AndShortImmediate,
            register_index,
            0xFFFF,
            0x0000,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x0000,
            vec![],
            vec![StatusRegisterFields::Zero],
        );
    }
}

//
// #### ORRI ####
//

#[test]
fn test_or_immediate() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::OrShortImmediate,
            register_index,
            0x0000,
            0xFF,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0xFF,
            // Test flag clearing (these flags do not reflect the initial register value)
            vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            vec![],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::OrShortImmediate,
            register_index,
            0xC0F0,
            0x0E,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0xC0FE,
            vec![],
            vec![StatusRegisterFields::Negative],
        );
    }
}

//
// #### XORI ####
//

#[test]
fn test_xor_immediate() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::XorShortImmediate,
            register_index,
            0x00FF,
            0xFF,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x0000,
            // Test flag clearing (these flags do not reflect the initial register value)
            vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            vec![StatusRegisterFields::Zero],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::XorShortImmediate,
            register_index,
            0xF0F0,
            0xF0,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0xF000,
            vec![],
            vec![StatusRegisterFields::Negative],
        );
    }
}

//
// #### SHFI ####
//

#[test]
fn test_logical_shift_left_immediate() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0x0,
            0b0011_0011_0011_0011,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            1,
            0b0110_0110_0110_0110,
            // Test flag clearing (these flags do not reflect the initial register value)
            vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            vec![],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1011_0011_0011_0011,
            0x1,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            1,
            0b0110_0110_0110_0110,
            vec![],
            vec![StatusRegisterFields::Carry],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1011_0011_0011_0011,
            6,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            6,
            0b1100_1100_1100_0000,
            vec![],
            vec![StatusRegisterFields::Negative],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1011_0011_0011_0011,
            15,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            15,
            0b1000_0000_0000_0000,
            vec![],
            vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1011_0011_0011_0011,
            16,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            16,
            0b0000_0000_0000_0000,
            vec![],
            vec![StatusRegisterFields::Zero, StatusRegisterFields::Carry],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1011_0011_0011_0011,
            17,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0000_0000_0000_0000,
            vec![],
            vec![StatusRegisterFields::Zero, StatusRegisterFields::Carry],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1011_0011_0011_0011,
            u8::MAX,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0000_0000_0000_0000,
            vec![],
            vec![StatusRegisterFields::Zero, StatusRegisterFields::Carry],
        );
    }
}

#[test]
fn test_logical_shift_right_immediate() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1100_1100_1100_1100,
            0x1,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0110_0110_0110_0110,
            // Test flag clearing (these flags do not reflect the initial register value)
            vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            vec![],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1100_1100_1100_1101,
            0x1,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0110_0110_0110_0110,
            vec![],
            vec![StatusRegisterFields::Carry],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1100_1100_1100_1101,
            6,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0000_0011_0011_0011,
            vec![],
            vec![],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1100_1100_1100_1101,
            15,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0000_0000_0000_0001,
            vec![],
            vec![StatusRegisterFields::Carry],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1100_1100_1100_1101,
            16,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0000_0000_0000_0000,
            vec![],
            vec![StatusRegisterFields::Zero, StatusRegisterFields::Carry],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1100_1100_1100_1101,
            17,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0000_0000_0000_0000,
            vec![],
            vec![StatusRegisterFields::Zero, StatusRegisterFields::Carry],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1100_1100_1100_1101,
            u8::MAX,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0000_0000_0000_0000,
            vec![],
            vec![StatusRegisterFields::Zero, StatusRegisterFields::Carry],
        );
    }
}

#[test]
fn test_arithmetic_shift_left_immediate() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b0011_0011_0011_0011,
            0x1,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0110_0110_0110_0110,
            // Test flag clearing (these flags do not reflect the initial register value)
            vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            vec![],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1011_0011_0011_0011,
            0x1,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0110_0110_0110_0110,
            vec![],
            vec![StatusRegisterFields::Carry, StatusRegisterFields::Overflow],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1011_0011_0011_0011,
            6,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b1100_1100_1100_0000,
            vec![],
            vec![StatusRegisterFields::Negative],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1011_0011_0011_0011,
            15,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b1000_0000_0000_0000,
            vec![],
            vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1011_0011_0011_0011,
            16,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0000_0000_0000_0000,
            vec![],
            vec![
                StatusRegisterFields::Zero,
                StatusRegisterFields::Carry,
                StatusRegisterFields::Overflow,
            ],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1011_0011_0011_0011,
            17,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0000_0000_0000_0000,
            vec![],
            vec![
                StatusRegisterFields::Zero,
                StatusRegisterFields::Carry,
                StatusRegisterFields::Overflow,
            ],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1011_0011_0011_0011,
            u8::MAX,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0000_0000_0000_0000,
            vec![],
            vec![
                StatusRegisterFields::Zero,
                StatusRegisterFields::Carry,
                StatusRegisterFields::Overflow,
            ],
        );
    }
}

#[test]
fn test_arithmetic_shift_right_immediate() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b0100_1100_1100_1100,
            0x1,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b0010_0110_0110_0110,
            // Test flag clearing (these flags do not reflect the initial register value)
            vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            vec![],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1100_1100_1100_1101,
            0x1,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b1110_0110_0110_0110,
            vec![],
            vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1100_1100_1100_1101,
            6,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b1000_0011_0011_0011,
            vec![],
            vec![StatusRegisterFields::Negative],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1100_1100_1100_1101,
            15,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b1000_0000_0000_0001,
            vec![],
            vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1100_1100_1100_1101,
            16,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b1000_0000_0000_0000,
            vec![],
            vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1100_1100_1100_1101,
            17,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b1000_0000_0000_0000,
            vec![],
            vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::ShiftShortImmediate,
            register_index,
            0b1100_1100_1100_1101,
            u8::MAX,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0b1000_0000_0000_0000,
            vec![],
            vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
        );
    }
}
