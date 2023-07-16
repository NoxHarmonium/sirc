use peripheral_cpu::{
    self,
    instructions::definitions::{
        ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
        StatusRegisterUpdateSource,
    },
    registers::{set_sr_bit, sr_bit_is_set, RegisterIndexing, Registers, StatusRegisterFields},
};

use crate::instructions::common;

use super::common::{get_expected_registers, get_register_index_range};

fn test_immediate_arithmetic_instruction(
    instruction: Instruction,
    target_register: u8,
    register_value: u16,
    immediate_value: u16,
    expected_value: u16,
    initial_status_flags: &Vec<StatusRegisterFields>,
    expected_status_flags: &Vec<StatusRegisterFields>,
) {
    let instruction_data = InstructionData::Immediate(ImmediateInstructionData {
        op_code: instruction,
        register: target_register,
        value: immediate_value,
        condition_flag: ConditionFlags::Always,
        additional_flags: if instruction == Instruction::ShiftImmediate {
            StatusRegisterUpdateSource::Shift as u8
        } else {
            StatusRegisterUpdateSource::Alu as u8
        },
    });
    let (previous, current) = common::run_instruction(
        &instruction_data,
        |registers: &mut Registers| {
            registers.set_at_index(target_register, register_value);
            for &status_register_field in initial_status_flags {
                set_sr_bit(status_register_field, registers);
            }
        },
        0xFACE,
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
fn test_add_immediate_basic() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::AddImmediate,
            register_index,
            0x1100,
            0x1101,
            0x2201,
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
fn test_add_immediate_unsigned_overflow() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::AddImmediate,
            register_index,
            0xFFFF,
            0x0001,
            0x0000,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Zero],
        );
    }
}

#[test]
fn test_add_immediate_signed_overflow() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::AddImmediate,
            register_index,
            0x7FFF,
            0x2000,
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
fn test_add_immediate_both_overflow() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::AddImmediate,
            register_index,
            0x9FFF,
            0x9000,
            0x2FFF,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Overflow],
        );
    }
}

//
// #### ADCI ####
//

#[test]
fn test_add_immediate_with_carry_basic() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::AddImmediateWithCarry,
            register_index,
            0x2212,
            0x1101,
            0x3314,
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
fn test_add_immediate_with_carry_carry_over() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::AddImmediateWithCarry,
            register_index,
            0xFFFF,
            0xFFFF,
            0xFFFE,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
        );
        test_immediate_arithmetic_instruction(
            Instruction::AddImmediateWithCarry,
            register_index,
            0xFFFE,
            0xFFFF,
            0xFFFE,
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
        );
        test_immediate_arithmetic_instruction(
            Instruction::AddImmediateWithCarry,
            register_index,
            0xFFFF,
            0xFFFF,
            0xFFFF,
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Negative],
        );
    }
}

#[test]
fn test_add_immediate_with_carry_unsigned_overflow() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::AddImmediateWithCarry,
            register_index,
            0xFFFF,
            0x0001,
            0x0000,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Zero],
        );
    }
}

#[test]
fn test_add_immediate_with_carry_signed_overflow() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::AddImmediateWithCarry,
            register_index,
            0x7FFF,
            0x2000,
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
fn test_add_immediate_with_carry_both_overflow() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::AddImmediateWithCarry,
            register_index,
            0x9FFF,
            0x9000,
            0x2FFF,
            &vec![],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Overflow],
        );
    }
}

//
// #### SUBI ####
//

#[test]
fn test_subtract_immediate_basic() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::SubtractImmediate,
            register_index,
            0x5245,
            0x2143,
            0x3102,
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
fn test_subtract_immediate_unsigned_overflow() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::SubtractImmediate,
            register_index,
            0x5FFF,
            0xFFFF,
            0x6000,
            &vec![],
            &vec![StatusRegisterFields::Carry],
        );
    }
}

#[test]
fn test_subtract_immediate_signed_overflow() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::SubtractImmediate,
            register_index,
            0xDFFF,
            0x7FFF,
            0x6000,
            &vec![],
            &vec![StatusRegisterFields::Overflow],
        );
    }
}

#[test]
fn test_subtract_immediate_both_overflow() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::SubtractImmediate,
            register_index,
            0x5FFF,
            0xBFFF,
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
// #### SBCI ####
//

#[test]
fn test_subtract_immediate_with_carry_basic() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::SubtractImmediateWithCarry,
            register_index,
            0x5245,
            0x2143,
            0x3101,
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
fn test_subtract_immediate_with_carry_carry_over() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::SubtractImmediateWithCarry,
            register_index,
            0x0000,
            0xFFFF,
            0x0001,
            &vec![StatusRegisterFields::Zero],
            &vec![StatusRegisterFields::Carry],
        );
        test_immediate_arithmetic_instruction(
            Instruction::SubtractImmediateWithCarry,
            register_index,
            0x0001,
            0xFFFF,
            0x0001,
            &vec![StatusRegisterFields::Carry],
            &vec![StatusRegisterFields::Carry],
        );
        test_immediate_arithmetic_instruction(
            Instruction::SubtractImmediateWithCarry,
            register_index,
            0x0000,
            0xFFFF,
            0x0000,
            &vec![StatusRegisterFields::Carry],
            &vec![StatusRegisterFields::Carry, StatusRegisterFields::Zero],
        );
    }
}

#[test]
fn test_subtract_immediate_with_carry_unsigned_overflow() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::SubtractImmediateWithCarry,
            register_index,
            0x5FFF,
            0xFFFF,
            0x6000,
            &vec![],
            &vec![StatusRegisterFields::Carry],
        );
    }
}

#[test]
fn test_subtract_immediate_with_carry_signed_overflow() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::SubtractImmediateWithCarry,
            register_index,
            0xDFFF,
            0x7FFF,
            0x6000,
            &vec![],
            &vec![StatusRegisterFields::Overflow],
        );
    }
}
#[test]
fn test_subtract_immediate_with_carry_both_overflow() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::SubtractImmediateWithCarry,
            register_index,
            0x5FFF,
            0xBFFF,
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
// #### ANDI ####
//

#[test]
fn test_and_immediate() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::AndImmediate,
            register_index,
            0xF0F0,
            0x0FFF,
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
        test_immediate_arithmetic_instruction(
            Instruction::AndImmediate,
            register_index,
            0xFFFF,
            0x0000,
            0x0000,
            &vec![],
            &vec![StatusRegisterFields::Zero],
        );
    }
}

//
// #### ORRI ####
//

#[test]
fn test_or_immediate() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::OrImmediate,
            register_index,
            0x0000,
            0xFFFF,
            0xFFFF,
            // Test flag clearing (these flags do not reflect the initial register value)
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![StatusRegisterFields::Negative],
        );
        test_immediate_arithmetic_instruction(
            Instruction::OrImmediate,
            register_index,
            0xC0F0,
            0x0A0E,
            0xCAFE,
            &vec![],
            &vec![StatusRegisterFields::Negative],
        );
    }
}

//
// #### XORI ####
//

#[test]
fn test_xor_immediate() {
    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::XorImmediate,
            register_index,
            0x0000,
            0xFFFF,
            0xFFFF,
            // Test flag clearing (these flags do not reflect the initial register value)
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![StatusRegisterFields::Negative],
        );
        test_immediate_arithmetic_instruction(
            Instruction::XorImmediate,
            register_index,
            0xF0F0,
            0xF0F0,
            0x0000,
            &vec![],
            &vec![StatusRegisterFields::Zero, StatusRegisterFields::Overflow],
        );
    }
}

//
// #### SHFI ####
//

#[test]
fn test_shfi_immediate() {
    // This is a strange instruction and only really exists to make decoding simpler
    // Since it is a long immediate (0x0_) instruction, the shifter will be disabled
    // The whole point of the SHFI instruction is to put the result of the shift into
    // the status register, because when shifting other instructions the shift
    // status bits are ignored. However, since the shift is disabled, the
    // target register should remain untouched, no matter what the immediate value is
    // so its effectively a no-op (although depending on the implementation it might
    // set the status register bits to the register value?)

    for register_index in get_register_index_range() {
        test_immediate_arithmetic_instruction(
            Instruction::ShiftImmediate,
            register_index,
            0xFFFF,
            0x0000,
            0xFFFF,
            // Test flag clearing (these flags do not reflect the initial register value)
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
        );
        test_immediate_arithmetic_instruction(
            Instruction::ShiftImmediate,
            register_index,
            0x0000,
            0xFFFF,
            0x0000,
            // Test flag clearing (these flags do not reflect the initial register value)
            &vec![
                StatusRegisterFields::Carry,
                StatusRegisterFields::Negative,
                StatusRegisterFields::Overflow,
                StatusRegisterFields::Zero,
            ],
            &vec![],
        );
        test_immediate_arithmetic_instruction(
            Instruction::ShiftImmediate,
            register_index,
            0xF0F0,
            0xFF00,
            0xF0F0,
            &vec![],
            &vec![],
        );
        test_immediate_arithmetic_instruction(
            Instruction::ShiftImmediate,
            register_index,
            0xF0F0,
            0xFFFF,
            0xF0F0,
            &vec![],
            &vec![],
        );
    }
}

// TODO: Test ShiftOperand::Register
// TODO: Test Rotates
// TODO: Test conditionals