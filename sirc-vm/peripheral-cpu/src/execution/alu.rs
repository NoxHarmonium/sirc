use std::ops::Shl;

use super::shared::IntermediateRegisters;
use crate::{
    instructions::definitions::ShiftType,
    registers::{clear_sr_bit, set_sr_bit, sr_bit_is_set_value, Registers, StatusRegisterFields},
};

const MSB_MASK: u32 = 0x8000_0000;

#[derive(PartialEq, Eq, Debug)]
pub enum Sign {
    Positive,
    Negative,
}

#[must_use]
pub fn sign(value: u16) -> Sign {
    let sign_bit = 0x8000;

    if (value & sign_bit) == sign_bit {
        Sign::Negative
    } else {
        Sign::Positive
    }
}

#[derive(FromPrimitive, ToPrimitive, Debug, Default, PartialEq, Eq)]
pub enum AluOp {
    #[default]
    Add = 0x0,
    AddWithCarry = 0x1,
    Subtract = 0x2,
    SubtractWithCarry = 0x3,
    And = 0x4,
    Or = 0x5,
    Xor = 0x6,
    Load = 0x7,
}

// Arithmetic

// TODO: Surely there is a way to extract common logic from all the ALU executors

///
/// Executes an addition operation on two registers, storing the result in
/// the first operand.
///
/// If an unsigned overflow occurs, the carry status flag will be set.
/// If a signed overflow occurs, the overflow status flag will be set.
///
/// ```
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set_value, StatusRegisterFields};
/// use peripheral_cpu::execution::shared::IntermediateRegisters;
/// use peripheral_cpu::execution::alu::perform_add;
///
/// // Thanks: https://stackoverflow.com/a/69125543/1153203
///
/// let mut intermediate_registers = IntermediateRegisters::default();
///
/// // Unsigned Overflow
/// perform_add(0xFFFF, 0x0001, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x0000);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Carry, intermediate_registers.alu_status_register), true);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Overflow, intermediate_registers.alu_status_register), false);
///
/// // Signed Overflow
/// perform_add(0x7FFF, 0x2000, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x9FFF);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Carry, intermediate_registers.alu_status_register), false);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Overflow, intermediate_registers.alu_status_register), true);
///
/// // Both Overflow
/// perform_add(0x9FFF, 0x9000, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x2FFF);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Carry, intermediate_registers.alu_status_register), true);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Overflow, intermediate_registers.alu_status_register), true);
/// ```
///
pub fn perform_add(a: u16, b: u16, intermediate_registers: &mut IntermediateRegisters) {
    let (result, carry) = a.overflowing_add(b);
    set_alu_bits(
        &mut intermediate_registers.alu_status_register,
        result,
        carry,
        Some((a, b, result)),
    );
    intermediate_registers.alu_output = result;
}

///
/// Executes an addition operation on two registers, storing the result in
/// the first operand.
///
/// If the carry flag is set, it will add one to the result. This
/// allows addition operations to be "chained" over multiple registers.
///
/// If an unsigned overflow occurs, the carry status flag will be set.
/// If a signed overflow occurs, the overflow status flag will be set.
///
/// ```
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set_value, StatusRegisterFields, set_sr_bit_value};
/// use peripheral_cpu::execution::shared::IntermediateRegisters;
/// use peripheral_cpu::execution::alu::perform_add_with_carry;
///
/// // Thanks: https://stackoverflow.com/a/69125543/1153203
///
/// let mut intermediate_registers = IntermediateRegisters::default();
///
/// // Operation that produces carry
/// perform_add_with_carry(0xFFFF, 0xFFFF, intermediate_registers.alu_status_register, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0xFFFE);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Carry, intermediate_registers.alu_status_register), true);
///
/// set_sr_bit_value(StatusRegisterFields::Carry, &mut intermediate_registers.alu_status_register);
///
/// // Do the same operation now that the carry bit is set
/// perform_add_with_carry(0xFFFF, 0xFFFF, intermediate_registers.alu_status_register, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0xFFFF);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Carry, intermediate_registers.alu_status_register), true);
///
/// // Wrapping to zero
/// perform_add_with_carry(0x0000, 0xFFFF, intermediate_registers.alu_status_register, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x0000);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Carry, intermediate_registers.alu_status_register), true);
/// ```
///
pub fn perform_add_with_carry(
    a: u16,
    b: u16,
    status_register: u16,
    intermediate_registers: &mut IntermediateRegisters,
) {
    let carry_from_previous = u16::from(sr_bit_is_set_value(
        StatusRegisterFields::Carry,
        status_register,
    ));

    let (r1, c1) = a.overflowing_add(b);
    let (r2, c2) = r1.overflowing_add(carry_from_previous);
    set_alu_bits(
        &mut intermediate_registers.alu_status_register,
        r2,
        c1 | c2,
        Some((a, b, r2)),
    );

    intermediate_registers.alu_output = r2;
}

///
/// Executes an subtraction operation on two registers, storing the result in
/// the first operand.
///
/// The second operand is subtracted from the first operand.
///
/// If an unsigned overflow occurs, the carry status flag will be set.
/// If a signed overflow occurs, the overflow status flag will be set.
///
/// ```
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set_value, StatusRegisterFields};
/// use peripheral_cpu::execution::shared::IntermediateRegisters;
/// use peripheral_cpu::execution::alu::perform_subtract;
///
/// // Thanks: http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
///
/// let mut intermediate_registers = IntermediateRegisters::default();
///
/// // Unsigned Overflow
/// perform_subtract(0x5FFF, 0xFFFF, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x6000);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Carry, intermediate_registers.alu_status_register), true);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Overflow, intermediate_registers.alu_status_register), false);
///
/// // Signed Overflow
/// perform_subtract(0xDFFF, 0x7FFF, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x6000);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Carry, intermediate_registers.alu_status_register), false);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Overflow, intermediate_registers.alu_status_register), true);
///
/// // Both Overflow
/// perform_subtract(0x5FFF, 0xBFFF, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0xA000);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Carry, intermediate_registers.alu_status_register), true);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Overflow, intermediate_registers.alu_status_register), true);
/// ```
///
pub fn perform_subtract(a: u16, b: u16, intermediate_registers: &mut IntermediateRegisters) {
    let (result, carry) = a.overflowing_sub(b);
    //The ones compliment of val_2 is used here for the overflow calculation because it is subtraction
    set_alu_bits(
        &mut intermediate_registers.alu_status_register,
        result,
        carry,
        Some((a, !b, result)),
    );

    intermediate_registers.alu_output = result;
}

///
/// Executes an subtraction operation on two registers, storing the result in
/// the first operand.
///
/// The second operand is subtracted from the first operand.
///
/// If the carry flag (which is really borrow flag in this context) is set,
/// it will subtract one from the result. This allows subtraction operations to be "chained"
/// over multiple registers.
///
/// If an unsigned overflow occurs, the carry status flag will be set.
/// If a signed overflow occurs, the overflow status flag will be set.
///
/// ```
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set_value, StatusRegisterFields, set_sr_bit_value};
/// use peripheral_cpu::execution::shared::IntermediateRegisters;
/// use peripheral_cpu::execution::alu::perform_subtract_with_carry;
///
/// // Thanks: http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
///
/// let mut intermediate_registers = IntermediateRegisters::default();
///
/// // Subtraction that causes borrow
/// perform_subtract_with_carry(0x0000, 0xFFFF, intermediate_registers.alu_status_register, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x0001);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Carry, intermediate_registers.alu_status_register), true);
///
/// set_sr_bit_value(StatusRegisterFields::Carry, &mut intermediate_registers.alu_status_register);
///
/// // Subtraction with borrow (carry) bit set
///  perform_subtract_with_carry(0x0000, 0xFFFF,  intermediate_registers.alu_status_register, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x0000);
/// assert_eq!(sr_bit_is_set_value(StatusRegisterFields::Carry, intermediate_registers.alu_status_register), true);
/// ```
///
pub fn perform_subtract_with_carry(
    a: u16,
    b: u16,
    status_register: u16,
    intermediate_registers: &mut IntermediateRegisters,
) {
    let carry_from_previous = u16::from(sr_bit_is_set_value(
        StatusRegisterFields::Carry,
        status_register,
    ));

    let (r1, c1) = a.overflowing_sub(b);
    let (r2, c2) = r1.overflowing_sub(carry_from_previous);
    //The ones compliment of val_2 is used here for the overflow calculation because it is subtraction
    set_alu_bits(
        &mut intermediate_registers.alu_status_register,
        r2,
        c1 | c2,
        Some((a, !b, r2)),
    );

    intermediate_registers.alu_output = r2;
}

// Logic

fn perform_and(a: u16, b: u16, intermediate_registers: &mut IntermediateRegisters) {
    let result = a & b;
    set_alu_bits(
        &mut intermediate_registers.alu_status_register,
        result,
        false,
        Some((a, b, result)),
    );

    intermediate_registers.alu_output = result;
}

fn perform_or(a: u16, b: u16, intermediate_registers: &mut IntermediateRegisters) {
    let result = a | b;
    set_alu_bits(
        &mut intermediate_registers.alu_status_register,
        result,
        false,
        Some((a, b, result)),
    );

    intermediate_registers.alu_output = result;
}

fn perform_xor(a: u16, b: u16, intermediate_registers: &mut IntermediateRegisters) {
    let result = a ^ b;
    set_alu_bits(
        &mut intermediate_registers.alu_status_register,
        result,
        false,
        Some((a, b, result)),
    );

    intermediate_registers.alu_output = result;
}

fn perform_load(a: u16, b: u16, intermediate_registers: &mut IntermediateRegisters) {
    let result = b;
    set_alu_bits(
        &mut intermediate_registers.alu_status_register,
        result,
        false,
        Some((a, b, result)),
    );

    intermediate_registers.alu_output = result;
}

// Shifts

#[must_use]
pub fn perform_shift(
    operand: u16,
    shift_type: ShiftType,
    shift_count: u16,
    short_immediate: bool, // TODO: Find a smarter solution to this
) -> (u16, u16) {
    // println!(
    //     "!!SHIFT!! {:#?} | {:#?} | {:#?}",
    //     operand, shift_type, shift_count
    // );

    // TODO: This is terrible. It needs a proper refactor, but in the interest of time at the moment
    // we will conform to the current interface and fix it later
    let mut intermediate_registers = IntermediateRegisters::default();

    match shift_type {
        ShiftType::None => {
            intermediate_registers.alu_output = operand;
            set_alu_bits(
                &mut intermediate_registers.alu_status_register,
                operand,
                false,
                None,
            );
            return (operand, intermediate_registers.alu_status_register);
        }
        ShiftType::LogicalLeftShift => {
            perform_logical_left_shift(operand, shift_count, &mut intermediate_registers);
        }
        ShiftType::LogicalRightShift => {
            perform_logical_right_shift(operand, shift_count, &mut intermediate_registers);
        }
        ShiftType::ArithmeticLeftShift => {
            perform_arithmetic_left_shift(operand, shift_count, &mut intermediate_registers);
        }
        ShiftType::ArithmeticRightShift => {
            perform_arithmetic_right_shift(
                operand,
                shift_count,
                &mut intermediate_registers,
                short_immediate,
            );
        }
        ShiftType::RotateLeft => {
            perform_rotate_left(operand, shift_count, &mut intermediate_registers);
        }
        ShiftType::RotateRight => {
            perform_rotate_right(operand, shift_count, &mut intermediate_registers);
        }
        ShiftType::Reserved => {
            intermediate_registers.alu_output = operand;
        }
    };

    (
        intermediate_registers.alu_output,
        intermediate_registers.alu_status_register,
    )
}

#[allow(clippy::cast_possible_truncation)]
fn perform_logical_left_shift(a: u16, b: u16, intermediate_registers: &mut IntermediateRegisters) {
    let extended_a = u32::from(a);
    let clamped_b = b.clamp(0, u16::BITS as u16);

    // There doesn't seem to be a built in method to shift left and get a flag
    // when a bit goes off the end so we can calculate carry.
    // Therefore, we can shift a 32 bit value temporarily and check manually if a bit
    // goes off the end. The hardware implementation will look different to this
    let wide_result = extended_a.shl(clamped_b);
    let result = wide_result as u16;
    let carry = (wide_result.rotate_left(u16::BITS) & 0x1) == 0x1;

    set_alu_bits(
        &mut intermediate_registers.alu_status_register,
        result,
        carry,
        None,
    );

    intermediate_registers.alu_output = result;
}

#[allow(clippy::cast_possible_truncation)]
fn perform_logical_right_shift(a: u16, b: u16, intermediate_registers: &mut IntermediateRegisters) {
    let extended_a = u32::from(a);
    let clamped_b = b.clamp(0, u16::BITS as u16);

    // There doesn't seem to be a built in method to shift right and get a flag
    // when a bit goes off the end so we can calculate carry.
    // Therefore, we can shift a 32 bit value temporarily and check manually if a bit
    // rotates into the first position on the left side.
    // The hardware implementation will look different to this
    let wide_result = extended_a.rotate_right(u32::from(clamped_b));
    let result = wide_result as u16;
    let carry = (wide_result & MSB_MASK) == MSB_MASK;

    set_alu_bits(
        &mut intermediate_registers.alu_status_register,
        result,
        carry,
        None,
    );

    intermediate_registers.alu_output = result;
}

#[allow(clippy::cast_possible_truncation)]
fn perform_arithmetic_left_shift(
    a: u16,
    b: u16,
    intermediate_registers: &mut IntermediateRegisters,
) {
    // Same as LSL but will set the overflow bit if the sign changes

    let extended_a = u32::from(a);
    let clamped_b = b.clamp(0, u16::BITS as u16);

    // There doesn't seem to be a built in method to shift left and get a flag
    // when a bit goes off the end so we can calculate carry.
    // Therefore, we can shift a 32 bit value temporarily and check manually if a bit
    // goes off the end. The hardware implementation will look different to this
    let wide_result = extended_a.shl(clamped_b);
    let result = wide_result as u16;
    let carry = (wide_result.rotate_left(u16::BITS) & 0x1) == 0x1;

    set_alu_bits(
        &mut intermediate_registers.alu_status_register,
        result,
        carry,
        Some((a, a, result)),
    );

    intermediate_registers.alu_output = result;
}

#[allow(clippy::cast_possible_truncation)]
fn perform_arithmetic_right_shift(
    a: u16,
    b: u16,
    intermediate_registers: &mut IntermediateRegisters,
    short_immediate: bool, // TODO: Find a smarter solution to this
) {
    // Same as LSR but preserves the sign bit
    let sign_bit = u32::from(a) & if short_immediate { 0x80 } else { 0x8000 }; // TODO: Find a smarter solution to this

    let extended_a = u32::from(a);
    let clamped_b = b.clamp(0, u16::BITS as u16);

    // There doesn't seem to be a built in method to shift right and get a flag
    // when a bit goes off the end so we can calculate carry.
    // Therefore, we can shift a 32 bit value temporarily and check manually if a bit
    // rotates into the first position on the left side.
    // The hardware implementation will look different to this
    let wide_result = extended_a.rotate_right(u32::from(clamped_b)) | sign_bit;
    let result = wide_result as u16;
    let carry = (wide_result & MSB_MASK) == MSB_MASK;

    println!(
        "sign_bit: {sign_bit:#b} extended_a: {extended_a:#b} clamped_b: {clamped_b:#b} wide_result: {wide_result:#b} result: {result:#b} carry: {carry}",
    );

    set_alu_bits(
        &mut intermediate_registers.alu_status_register,
        result,
        carry,
        Some((a, a, result)),
    );

    intermediate_registers.alu_output = result;
}

fn perform_rotate_left(a: u16, b: u16, intermediate_registers: &mut IntermediateRegisters) {
    let result: u16 = a.rotate_left(u32::from(b));
    // Bit shifted out goes into carry (that's what the 68k does :shrug:)
    // TODO: Extract bit checking to function
    // TODO: Check the purpose of bit shifted out going to carry
    let carry = result & 1 == 1;
    set_alu_bits(
        &mut intermediate_registers.alu_status_register,
        result,
        carry,
        None,
    );
    intermediate_registers.alu_output = result;
}

fn perform_rotate_right(a: u16, b: u16, intermediate_registers: &mut IntermediateRegisters) {
    let result: u16 = a.rotate_right(u32::from(b));
    // Bit shifted out goes into carry (that's what the 68k does :shrug:)
    // TODO: Extract bit checking to function
    // TODO: Check the purpose of bit shifted out going to carry
    let carry = (result >> 15) & 1 == 1;
    set_alu_bits(
        &mut intermediate_registers.alu_status_register,
        result,
        carry,
        None,
    );
    intermediate_registers.alu_output = result;
}

pub fn perform_alu_operation(
    alu_op: &AluOp,
    simulate: bool,
    a: u16,
    b: u16,
    status_register: u16,
    intermediate_registers: &mut IntermediateRegisters,
) {
    // println!(
    //     "!!ALU!! {:#?} | {} | {} | {:#?} | {:#?}",
    //     alu_op, a, b, registers, intermediate_registers
    // );

    match alu_op {
        AluOp::Add => perform_add(a, b, intermediate_registers),
        AluOp::AddWithCarry => {
            perform_add_with_carry(a, b, status_register, intermediate_registers);
        }
        AluOp::Subtract => perform_subtract(a, b, intermediate_registers),
        AluOp::SubtractWithCarry => {
            perform_subtract_with_carry(a, b, status_register, intermediate_registers);
        }
        AluOp::And => perform_and(a, b, intermediate_registers),
        AluOp::Or => perform_or(a, b, intermediate_registers),
        AluOp::Xor => perform_xor(a, b, intermediate_registers),
        AluOp::Load => {
            perform_load(a, b, intermediate_registers);
        }
    }

    if simulate {
        // Alu op from 0x8-0xF don't actually store the ALU result, just the status code
        // TODO: Do this in a cleaner way, probably without passing boolean as function param
        intermediate_registers.alu_output = 0x0;
    }

    // println!("!!ALU DONE!! {:#?} | {:#?}", alu_op, intermediate_registers);
}

#[allow(clippy::cast_possible_wrap)]
pub fn set_alu_bits(
    sr: &mut u16,
    value: u16,
    carry: bool,
    inputs_and_result: Option<(u16, u16, u16)>,
) {
    // TODO: REFACTOR this monstrosity to be sensible
    // I just hacked in a temporary register to avoid refactoring and get
    // the tests done faster. Once the tests are done, refactoring will be easier.

    let mut registers = Registers {
        sr: *sr,
        ..Registers::default()
    };

    if value == 0 {
        set_sr_bit(StatusRegisterFields::Zero, &mut registers);
    } else {
        clear_sr_bit(StatusRegisterFields::Zero, &mut registers);
    }
    if (value as i16) < 0 {
        set_sr_bit(StatusRegisterFields::Negative, &mut registers);
    } else {
        clear_sr_bit(StatusRegisterFields::Negative, &mut registers);
    }
    if carry {
        set_sr_bit(StatusRegisterFields::Carry, &mut registers);
    } else {
        clear_sr_bit(StatusRegisterFields::Carry, &mut registers);
    }

    // See http://www.csc.villanova.edu/~mdamian/Past/csc2400fa16/labs/ALU.html
    // The logic is follows: when adding, if the sign of the two inputs is the same, but the result sign is different, then we have an overflow.
    if let Some((i1, i2, result)) = inputs_and_result {
        if sign(i1) == sign(i2) && sign(result) != sign(i1) {
            set_sr_bit(StatusRegisterFields::Overflow, &mut registers);
        } else {
            // TODO: Can we collapse this if statement so we only have one else?
            // The double clear seems redundant
            clear_sr_bit(StatusRegisterFields::Overflow, &mut registers);
        }
    } else {
        clear_sr_bit(StatusRegisterFields::Overflow, &mut registers);
    }

    *sr = registers.sr;
}
