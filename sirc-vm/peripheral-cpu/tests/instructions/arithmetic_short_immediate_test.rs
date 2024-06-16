use peripheral_bus::BusPeripheral;
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ConditionFlags, Instruction, InstructionData, ShiftOperand, ShiftType,
        ShortImmediateInstructionData, StatusRegisterUpdateSource,
    },
    registers::{set_sr_bit, sr_bit_is_set, RegisterIndexing, Registers, StatusRegisterFields},
};

use crate::instructions::common;

use super::common::{get_expected_registers, get_register_index_range};

#[allow(clippy::too_many_arguments)]
fn test_short_immediate_arithmetic_instruction(
    instruction: Instruction,
    target_register: u8,
    register_value: u16,
    immediate_value: u8,
    shift_operand: ShiftOperand,
    shift_type: ShiftType,
    shift_count: u8,
    expected_value: u16,
    initial_status_flags: &Vec<StatusRegisterFields>,
    expected_status_flags: &Vec<StatusRegisterFields>,
    status_register_update_source: StatusRegisterUpdateSource,
) {
    let instruction_data = InstructionData::ShortImmediate(ShortImmediateInstructionData {
        op_code: instruction,
        register: target_register,
        value: immediate_value,
        shift_operand,
        shift_type,
        shift_count,
        condition_flag: ConditionFlags::Always,
        additional_flags: status_register_update_source as u8,
    });
    let (previous, current) = common::run_instruction(
        &instruction_data,
        |registers: &mut Registers, _: &mut BusPeripheral| {
            registers.set_at_index(target_register, register_value);
            for &status_register_field in initial_status_flags {
                set_sr_bit(status_register_field, registers);
            }
        },
        0xFACE_0000,
    );
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.set_at_index(target_register, expected_value);
            for &status_register_field in expected_status_flags {
                set_sr_bit(status_register_field, registers);
            }
        });
    assert_eq!(expected_registers, current.registers);
    for &status_register_field in expected_status_flags {
        assert!(sr_bit_is_set(status_register_field, &current.registers));
    }
}

//
// #### ADDI ####
//

#[test]
fn test_add_short_immediate_basic() {
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
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
            StatusRegisterUpdateSource::Alu,
        );
    }
}

#[test]
fn test_add_short_immediate_unsigned_overflow() {
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
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Zero],
            StatusRegisterUpdateSource::Alu,
        );
    }
}

#[test]
fn test_add_short_immediate_signed_overflow() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0x7FFF,
            0x0001,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x8000,
            &vec![],
            &vec![
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Negative,
            ],
            StatusRegisterUpdateSource::Alu,
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
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
            StatusRegisterUpdateSource::Alu,
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
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Zero],
            StatusRegisterUpdateSource::Alu,
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
            ShiftType::None,
            0,
            0x801F,
            &vec![],
            &vec![
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Negative,
            ],
            StatusRegisterUpdateSource::Alu,
        );
    }
}

#[test]
fn test_add_immediate_with_carry_with_carry() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediateWithCarry,
            register_index,
            0xFFFF,
            0x01,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x0001,
            &vec![StatusRegisterFields::Carry],
            &vec![StatusRegisterFields::Carry],
            StatusRegisterUpdateSource::Alu,
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
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
            StatusRegisterUpdateSource::Alu,
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
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
            StatusRegisterUpdateSource::Alu,
        );
    }
}

#[test]
fn test_subtract_immediate_with_carry_unsigned_overflow() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::SubtractShortImmediateWithCarry,
            register_index,
            0x005F,
            0xFF,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0xFF60,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
            StatusRegisterUpdateSource::Alu,
        );
    }
}

#[test]
fn test_subtract_immediate_with_carry_signed_overflow() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::SubtractShortImmediateWithCarry,
            register_index,
            0x805F,
            0xFF,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x7F60,
            &vec![],
            &vec![StatusRegisterFields::Overflow],
            StatusRegisterUpdateSource::Alu,
        );
    }
}
#[test]
fn test_subtract_immediate_with_carry_with_carry() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::SubtractShortImmediateWithCarry,
            register_index,
            0x5F00,
            0xBF,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x5E40,
            &vec![StatusRegisterFields::Carry],
            &vec![],
            StatusRegisterUpdateSource::Alu,
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
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
            StatusRegisterUpdateSource::Alu,
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
            &vec![],
            &vec![StatusRegisterFields::Zero],
            StatusRegisterUpdateSource::Alu,
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
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
            StatusRegisterUpdateSource::Alu,
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
            &vec![],
            &vec![StatusRegisterFields::Negative],
            StatusRegisterUpdateSource::Alu,
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
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![StatusRegisterFields::Zero],
            StatusRegisterUpdateSource::Alu,
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
            &vec![],
            &vec![StatusRegisterFields::Negative],
            StatusRegisterUpdateSource::Alu,
        );
    }
}

//
// #### LOAD ####
//

#[test]
fn test_load_immediate() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::LoadRegisterFromShortImmediate,
            register_index,
            0xCAFE,
            0xFA,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0xFA,
            // Test flag clearing (these flags do not reflect the initial register value)
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
            StatusRegisterUpdateSource::Alu,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::LoadRegisterFromShortImmediate,
            register_index,
            0xF0F0,
            0x0,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x0,
            &vec![],
            &vec![StatusRegisterFields::Zero],
            StatusRegisterUpdateSource::Alu,
        );
    }
}

//
// #### Shifting ####
// (Adding with zero to try to test shifting in isolation)
//

#[test]
fn test_logical_shift_left_immediate() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b0011_0011,
            0x0,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            1,
            0b0000_0000_0110_0110,
            // Test flag clearing (these flags do not reflect the initial register value)
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1011_0011,
            0x0,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            1,
            0b0000_0001_0110_0110,
            &vec![],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1011_0011,
            0x0,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            6,
            0b0010_1100_1100_0000,
            &vec![],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1011_0011,
            0x0,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            15,
            0b1000_0000_0000_0000,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1011_0011,
            0x0,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            0,
            0b0000_0000_1011_0011,
            &vec![],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1011_0011,
            0x0,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            // Max immediate shift is 15
            u8::MAX,
            0b1000_0000_0000_0000,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
            StatusRegisterUpdateSource::Shift,
        );
    }
}

#[test]
fn test_logical_shift_right_immediate() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1100_1100,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalRightShift,
            0,
            0b0000_0000_1100_1100,
            // Test flag clearing (these flags do not reflect the initial register value)
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalRightShift,
            1,
            0b0000_0000_0110_0110,
            &vec![],
            &vec![StatusRegisterFields::Carry],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalRightShift,
            6,
            0b0000_0000_0000_0011,
            &vec![],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalRightShift,
            15,
            0b0000_0000_0000_0000,
            &vec![],
            &vec![StatusRegisterFields::Zero],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalRightShift,
            // Max immediate shift is 15, truncates to zero
            16,
            0b0000_0000_1100_1101,
            &vec![],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalRightShift,
            // Max immediate shift is 15, truncates to 1
            17,
            0b0000_0000_0110_0110,
            &vec![],
            &vec![StatusRegisterFields::Carry],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalRightShift,
            // Max immediate shift is 15, truncates to 15
            u8::MAX,
            0b0000_0000_0000_0000,
            &vec![],
            &vec![StatusRegisterFields::Zero],
            StatusRegisterUpdateSource::Shift,
        );
    }
}

#[test]
fn test_arithmetic_shift_left_immediate() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b0011_0011,
            0x0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticLeftShift,
            1,
            0b0000_0000_0110_0110,
            // Test flag clearing (these flags do not reflect the initial register value)
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1011_0011,
            0x0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticLeftShift,
            1,
            0b0000_0001_0110_0110,
            &vec![],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1011_0011,
            0x0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticLeftShift,
            6,
            0b0010_1100_1100_0000,
            &vec![],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1011_0011,
            0x0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticLeftShift,
            15,
            0b1000_0000_0000_0000,
            &vec![],
            &vec![
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Carry,
            ],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1011_0011,
            0x0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticLeftShift,
            0,
            0b0000_0000_1011_0011,
            &vec![],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1011_0011,
            0x0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticLeftShift,
            // Max immediate shift is 15
            u8::MAX,
            0b1000_0000_0000_0000,
            &vec![],
            &vec![
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Carry,
            ],
            StatusRegisterUpdateSource::Shift,
        );
    }
}

#[test]
fn test_arithmetic_shift_right_immediate() {
    for register_index in get_register_index_range() {
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1100_1100,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticRightShift,
            0,
            0b0000_0000_1100_1100,
            // Test flag clearing (these flags do not reflect the initial register value)
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticRightShift,
            1,
            0b0000_0000_1110_0110,
            &vec![],
            &vec![StatusRegisterFields::Carry],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticRightShift,
            6,
            0b0000_0000_1000_0011,
            &vec![],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticRightShift,
            15,
            0b0000_0000_1000_0000,
            &vec![],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticRightShift,
            // Max immediate shift is 15, truncates to zero
            16,
            0b0000_0000_1100_1101,
            &vec![],
            &vec![],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticRightShift,
            // Max immediate shift is 15, truncates to 1
            17,
            0b0000_0000_1110_0110,
            &vec![],
            &vec![StatusRegisterFields::Carry],
            StatusRegisterUpdateSource::Shift,
        );
        test_short_immediate_arithmetic_instruction(
            Instruction::AddShortImmediate,
            register_index,
            0b0100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticRightShift,
            // Max immediate shift is 15, truncates to 15
            u8::MAX,
            0b0000_0000_0000_0000,
            &vec![],
            &vec![StatusRegisterFields::Zero],
            StatusRegisterUpdateSource::Shift,
        );
    }
}

// TODO: Improve unit test coverage for arithmetic short immediate instructions
// category=Testing
// - Test ShiftOperand::Register
// - Test Rotates
// - Test conditionals
// - Test COPI
// - Test CMPI/TSAI/TSXI etc.
