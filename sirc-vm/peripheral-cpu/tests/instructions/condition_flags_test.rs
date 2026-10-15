use assert_hex::assert_eq_hex;
use peripheral_bus::BusPeripheral;
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::{
        ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
        StatusRegisterUpdateSource,
    },
    registers::{clear_sr_bit, set_sr_bit, RegisterIndexing, Registers, StatusRegisterFields},
};

use crate::instructions::common;

use super::common::get_expected_registers;

// `HI` (Unsigned Higher) and `LO` (Unsigned Lower-or-same) are evaluated purely from the
// (Carry, Zero) pair left by a prior comparison; this CPU sets Carry on subtraction as a borrow
// flag, so `Carry = a < b` (unsigned). Table below matches `docs/reference/chapters/10-condition-codes.tex`.
#[test]
fn test_unsigned_higher_lower_or_same_truth_table() {
    let cases = [
        // (carry, zero, expected_hi, expected_lo)
        (false, false, true, false), // a > b
        (false, true, false, true),  // a == b
        (true, false, false, true),  // a < b
        (true, true, false, true),   // unreachable via a real subtraction, but must still hold
    ];

    for (carry, zero, expected_hi, expected_lo) in cases {
        let mut registers = Registers::default();
        if carry {
            set_sr_bit(StatusRegisterFields::Carry, &mut registers);
        } else {
            clear_sr_bit(StatusRegisterFields::Carry, &mut registers);
        }
        if zero {
            set_sr_bit(StatusRegisterFields::Zero, &mut registers);
        } else {
            clear_sr_bit(StatusRegisterFields::Zero, &mut registers);
        }

        assert_eq!(
            expected_hi,
            ConditionFlags::UnsignedHigher.should_execute(&registers),
            "HI with carry={carry} zero={zero}"
        );
        assert_eq!(
            expected_lo,
            ConditionFlags::UnsignedLowerOrSame.should_execute(&registers),
            "LO with carry={carry} zero={zero}"
        );
        // LO is always the complement of HI.
        assert_eq!(
            !ConditionFlags::UnsignedHigher.should_execute(&registers),
            ConditionFlags::UnsignedLowerOrSame.should_execute(&registers),
            "LO should always be !HI, carry={carry} zero={zero}"
        );
    }
}

/// Runs a single conditionally-executed `ADDI` with the given condition flag and initial
/// Carry/Zero (as if a preceding `CMPR` had just set them), and asserts whether the write took
/// effect -- exercising `HI`/`LO` through the real assemble/decode/execute path, not just the
/// pure `should_execute` function.
fn assert_conditional_addi(
    condition_flag: ConditionFlags,
    carry: bool,
    zero: bool,
    should_take_effect: bool,
) {
    let target_register = 1;
    let register_value = 0x1000;
    let immediate_value = 0x0001;

    let instruction_data = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::AddImmediate,
        register: target_register,
        value: immediate_value,
        condition_flag,
        additional_flags: StatusRegisterUpdateSource::Alu as u8,
    });
    let (previous, current) = common::run_instruction(
        &instruction_data,
        |registers: &mut Registers, _: &mut BusPeripheral| {
            registers.set_at_index(target_register, register_value);
            if carry {
                set_sr_bit(StatusRegisterFields::Carry, registers);
            }
            if zero {
                set_sr_bit(StatusRegisterFields::Zero, registers);
            }
        },
        0xFACE_0000,
    );

    if should_take_effect {
        let expected_registers =
            get_expected_registers(&previous.registers, |registers: &mut Registers| {
                registers.set_at_index(target_register, register_value + immediate_value);
            });
        assert_eq_hex!(expected_registers, current.registers);
    } else {
        // Condition not met: write-back is a no-op, so the register and SR are untouched (only
        // the PC advances, as `get_expected_registers` already accounts for).
        let expected_registers =
            get_expected_registers(&previous.registers, |registers: &mut Registers| {
                registers.set_at_index(target_register, register_value);
                if carry {
                    set_sr_bit(StatusRegisterFields::Carry, registers);
                }
                if zero {
                    set_sr_bit(StatusRegisterFields::Zero, registers);
                }
            });
        assert_eq_hex!(expected_registers, current.registers);
    }
}

#[test]
fn test_hi_fires_when_first_operand_greater() {
    // a > b: no borrow, not equal -> Carry=0, Zero=0
    assert_conditional_addi(ConditionFlags::UnsignedHigher, false, false, true);
}

#[test]
fn test_hi_does_not_fire_when_first_operand_less() {
    // a < b: borrow -> Carry=1, Zero=0
    assert_conditional_addi(ConditionFlags::UnsignedHigher, true, false, false);
}

#[test]
fn test_hi_does_not_fire_when_operands_equal() {
    // a == b: no borrow, equal -> Carry=0, Zero=1
    assert_conditional_addi(ConditionFlags::UnsignedHigher, false, true, false);
}

#[test]
fn test_lo_fires_when_first_operand_less() {
    assert_conditional_addi(ConditionFlags::UnsignedLowerOrSame, true, false, true);
}

#[test]
fn test_lo_fires_when_operands_equal() {
    assert_conditional_addi(ConditionFlags::UnsignedLowerOrSame, false, true, true);
}

#[test]
fn test_lo_does_not_fire_when_first_operand_greater() {
    assert_conditional_addi(ConditionFlags::UnsignedLowerOrSame, false, false, false);
}
