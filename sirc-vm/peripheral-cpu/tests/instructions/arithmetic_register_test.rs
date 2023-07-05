use peripheral_cpu::{
    self,
    instructions::definitions::{
        ConditionFlags, Instruction, InstructionData, RegisterInstructionData, ShiftOperand,
        ShiftType, StatusRegisterUpdateSource,
    },
    registers::{set_sr_bit, sr_bit_is_set, RegisterIndexing, Registers, StatusRegisterFields},
};

use crate::instructions::common;

use super::common::{get_expected_registers, get_register_index_range};

// TODO: Point taken from clippy about having too many arguments.
// will fix up ASAP if possible
#[allow(clippy::too_many_arguments)]
fn test_register_arithmetic_instruction(
    instruction: Instruction,
    target_register: u8,
    register_value: u16,
    operand_value: u16,
    shift_operand: ShiftOperand,
    shift_type: ShiftType,
    shift_count: u8,
    expected_value: u16,
    initial_status_flags: &Vec<StatusRegisterFields>,
    expected_status_flags: &Vec<StatusRegisterFields>,
) {
    for src_register_index in get_register_index_range() {
        if target_register == src_register_index {
            // Having the destination and source the same should work on a real CPU
            // but breaks the test right now
            continue;
        }

        let instruction_data = InstructionData::Register(RegisterInstructionData {
            op_code: instruction,
            r1: target_register,
            r2: target_register,
            r3: src_register_index,
            shift_operand,
            shift_type,
            shift_count,
            condition_flag: ConditionFlags::Always,
            additional_flags: if instruction == Instruction::ShiftRegister {
                StatusRegisterUpdateSource::Shift as u8
            } else {
                StatusRegisterUpdateSource::Alu as u8
            },
        });
        let (previous, current) = common::run_instruction(
            &instruction_data,
            |registers: &mut Registers| {
                registers.set_at_index(target_register, register_value);
                registers.set_at_index(src_register_index, operand_value);
                for &status_register_field in initial_status_flags {
                    set_sr_bit(status_register_field, registers);
                }
            },
            0xFACE,
        );
        let expected_registers =
            get_expected_registers(&previous.registers, |registers: &mut Registers| {
                registers.set_at_index(target_register, expected_value);
                registers.set_at_index(src_register_index, operand_value);
                for &status_register_field in expected_status_flags {
                    set_sr_bit(status_register_field, registers);
                }
            });
        assert_eq!(expected_registers, current.registers);
        for &status_register_field in expected_status_flags {
            assert!(sr_bit_is_set(status_register_field, &current.registers));
        }
    }
}

//
// #### ADDR ####
//

#[test]
fn test_add_register_basic() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::AddRegister,
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
        );
    }
}

#[test]
fn test_add_register_unsigned_overflow() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::AddRegister,
            register_index,
            0xFFFF,
            0x0001,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x0000,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Zero],
        );
    }
}

#[test]
fn test_add_register_signed_overflow() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::AddRegister,
            register_index,
            0x7FFF,
            0x2000,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x9FFF,
            &vec![],
            &vec![
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Negative,
            ],
        );
    }
}
#[test]
fn test_add_register_both_overflow() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::AddRegister,
            register_index,
            0x9FFF,
            0x9000,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x2FFF,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Overflow],
        );
    }
}

//
// #### ADCR ####
//

#[test]
fn test_add_register_with_carry_basic() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::AddRegisterWithCarry,
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
        );
    }
}

#[test]
fn test_add_register_with_carry_unsigned_overflow() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::AddRegisterWithCarry,
            register_index,
            0xFFFF,
            0x01,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x0000,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Zero],
        );
    }
}

#[test]
fn test_add_register_with_carry_signed_overflow() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::AddRegisterWithCarry,
            register_index,
            0x7FFF,
            0x2000,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x9FFF,
            &vec![],
            &vec![
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Negative,
            ],
        );
    }
}
#[test]
fn test_add_register_with_carry_both_overflow() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::AddRegisterWithCarry,
            register_index,
            0x9FFF,
            0x9000,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x2FFF,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Overflow],
        );
    }
}

//
// #### SUBR ####
//

#[test]
fn test_subtract_register_basic() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::SubtractRegister,
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
        );
    }
}

//
// #### SBCR ####
//

#[test]
fn test_subtract_register_with_carry_basic() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::SubtractRegisterWithCarry,
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
        );
    }
}

#[test]
fn test_subtract_register_with_carry_unsigned_overflow() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::SubtractRegisterWithCarry,
            register_index,
            0x5F00,
            0xFF00,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x6000,
            &vec![],
            &vec![StatusRegisterFields::Carry],
        );
    }
}

#[test]
fn test_subtract_register_with_carry_signed_overflow() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::SubtractRegisterWithCarry,
            register_index,
            0xDF00,
            0x7F00,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x6000,
            &vec![],
            &vec![StatusRegisterFields::Overflow],
        );
    }
}
#[test]
fn test_subtract_register_with_carry_both_overflow() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::SubtractRegisterWithCarry,
            register_index,
            0x5F00,
            0xBF00,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0xA000,
            &vec![],
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Negative,
            ],
        );
    }
}

//
// #### ANDR ####
//

#[test]
fn test_and_register() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::AndRegister,
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
        );
        test_register_arithmetic_instruction(
            Instruction::AndRegister,
            register_index,
            0xFFFF,
            0x0000,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0x0000,
            &vec![],
            &vec![StatusRegisterFields::Zero],
        );
    }
}

//
// #### ORRR ####
//

#[test]
fn test_or_register() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::OrRegister,
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
        );
        test_register_arithmetic_instruction(
            Instruction::OrRegister,
            register_index,
            0xC0F0,
            0x0E,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0xC0FE,
            &vec![],
            &vec![StatusRegisterFields::Negative],
        );
    }
}

//
// #### XORR ####
//

#[test]
fn test_xor_register() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::XorRegister,
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
        );
        test_register_arithmetic_instruction(
            Instruction::XorRegister,
            register_index,
            0xF0F0,
            0xF0,
            ShiftOperand::Immediate,
            ShiftType::None,
            0,
            0xF000,
            &vec![],
            &vec![StatusRegisterFields::Negative],
        );
    }
}

//
// #### SHFR ####
//

#[test]
fn test_logical_shift_left_register() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b0011_0011_0011_0011,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            1,
            0b0110_0110_0110_0110,
            // Test flag clearing (these flags do not reflect the initial register value)
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1011_0011_0011_0011,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            1,
            0b0110_0110_0110_0110,
            &vec![],
            &vec![StatusRegisterFields::Carry],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1011_0011_0011_0011,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            6,
            0b1100_1100_1100_0000,
            &vec![],
            &vec![StatusRegisterFields::Negative],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1011_0011_0011_0011,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            15,
            0b1000_0000_0000_0000,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1011_0011_0011_0011,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            16,
            0b1011_0011_0011_0011,
            &vec![],
            &vec![StatusRegisterFields::Negative],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1011_0011_0011_0011,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            0,
            0b1011_0011_0011_0011,
            &vec![],
            &vec![StatusRegisterFields::Negative],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1011_0011_0011_0011,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalLeftShift,
            u8::MAX,
            0b1000_0000_0000_0000,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
        );
    }
}

#[test]
fn test_logical_shift_right_register() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1100_1100_1100_1100,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalRightShift,
            1,
            0b0110_0110_0110_0110,
            // Test flag clearing (these flags do not reflect the initial register value)
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1100_1100_1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalRightShift,
            1,
            0b0110_0110_0110_0110,
            &vec![],
            &vec![StatusRegisterFields::Carry],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1100_1100_1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalRightShift,
            6,
            0b0000_0011_0011_0011,
            &vec![],
            &vec![],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1100_1100_1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalRightShift,
            15,
            0b0000_0000_0000_0001,
            &vec![],
            &vec![StatusRegisterFields::Carry],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1100_1100_1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalRightShift,
            16,
            0b1100_1100_1100_1101,
            &vec![],
            &vec![StatusRegisterFields::Negative],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1100_1100_1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalRightShift,
            0,
            0b1100_1100_1100_1101,
            &vec![],
            &vec![StatusRegisterFields::Negative],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1100_1100_1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::LogicalRightShift,
            u8::MAX,
            0b0000_0000_0000_0001,
            &vec![],
            &vec![StatusRegisterFields::Carry],
        );
    }
}

// WHAT AM I DOING? FLIPPING SHIFT OPERAND (again?) so now operand 2 is shfited

#[test]
fn test_arithmetic_shift_left_register() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b0011_0011_0011_0011,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticLeftShift,
            1,
            0b0110_0110_0110_0110,
            // Test flag clearing (these flags do not reflect the initial register value)
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1011_0011_0011_0011,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticLeftShift,
            1,
            0b0110_0110_0110_0110,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Overflow],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1011_0011_0011_0011,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticLeftShift,
            6,
            0b1100_1100_1100_0000,
            &vec![],
            &vec![StatusRegisterFields::Negative],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1011_0011_0011_0011,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticLeftShift,
            15,
            0b1000_0000_0000_0000,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1011_0011_0011_0011,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticLeftShift,
            16,
            0b1011_0011_0011_0011,
            &vec![],
            &vec![StatusRegisterFields::Negative],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1011_0011_0011_0011,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticLeftShift,
            0,
            0b1011_0011_0011_0011,
            &vec![],
            &vec![StatusRegisterFields::Negative],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1011_0011_0011_0011,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticLeftShift,
            u8::MAX,
            0b1000_0000_0000_0000,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
        );
    }
}

#[test]
fn test_arithmetic_shift_right_register() {
    for register_index in get_register_index_range() {
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1100_1100_1100_1100,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticRightShift,
            1,
            0b1110_0110_0110_0110,
            // Test flag clearing (these flags do not reflect the initial register value)
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![StatusRegisterFields::Negative],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b0100_1100_1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticRightShift,
            1,
            0b0010_0110_0110_0110,
            &vec![],
            &vec![StatusRegisterFields::Carry],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1100_1100_1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticRightShift,
            6,
            0b1000_0011_0011_0011,
            &vec![],
            &vec![StatusRegisterFields::Negative],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1100_1100_1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticRightShift,
            15,
            0b1000_0000_0000_0001,
            &vec![],
            &vec![StatusRegisterFields::Negative, StatusRegisterFields::Carry],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1100_1100_1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticRightShift,
            16,
            0b1100_1100_1100_1101,
            &vec![],
            &vec![StatusRegisterFields::Negative],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1100_1100_1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticRightShift,
            0,
            0b1100_1100_1100_1101,
            &vec![],
            &vec![StatusRegisterFields::Negative],
        );
        test_register_arithmetic_instruction(
            Instruction::ShiftRegister,
            register_index,
            0b1100_1100_1100_1101,
            0,
            ShiftOperand::Immediate,
            ShiftType::ArithmeticRightShift,
            u8::MAX,
            0b1000_0000_0000_0001,
            &vec![],
            &vec![StatusRegisterFields::Negative, StatusRegisterFields::Carry],
        );
    }
}

// TODO: Test ShiftOperand::Register
// TODO: Test first operand register not being destination register
// TODO: Test conditionals
