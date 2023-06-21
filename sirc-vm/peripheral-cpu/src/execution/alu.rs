use std::ops::Shl;

use super::shared::IntermediateRegisters;
use crate::registers::{clear_sr_bit, set_sr_bit, sr_bit_is_set, Registers, StatusRegisterFields};

const MSB_MASK: u32 = 0x8000_0000;

#[derive(PartialEq, Eq, Debug)]
pub enum Sign {
    Positive,
    Negative,
}

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
    LogicalShiftLeft = 0x7,
    LogicalShiftRight = 0x8,
    ArithmeticShiftLeft = 0x9,
    ArithmeticShiftRight = 0xA,
    RotateLeft = 0xB,
    RotateRight = 0xC,
    Compare = 0xD,
    Reserved1 = 0xE,
    Reserved2 = 0xF,
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
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set, StatusRegisterFields};
/// use peripheral_cpu::execution::shared::IntermediateRegisters;
/// use peripheral_cpu::execution::alu::perform_add;
///
/// // Thanks: https://stackoverflow.com/a/69125543/1153203
///
/// let mut registers = Registers::default();
/// let mut intermediate_registers = IntermediateRegisters::default();
///
/// // Unsigned Overflow
/// perform_add(0xFFFF, 0x0001, &mut registers, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x0000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), false);
///
/// // Signed Overflow
/// perform_add(0x7FFF, 0x2000, &mut registers, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x9FFF);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), false);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), true);
///
/// // Both Overflow
/// perform_add(0x9FFF, 0x9000, &mut registers, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x2FFF);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), true);
/// ```
///
pub fn perform_add(
    a: u16,
    b: u16,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    let (result, carry) = a.overflowing_add(b);
    set_alu_bits(registers, result, carry, Some((a, b, result)));
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
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set, StatusRegisterFields};
/// use peripheral_cpu::execution::shared::IntermediateRegisters;
/// use peripheral_cpu::execution::alu::perform_add_with_carry;
///
/// // Thanks: https://stackoverflow.com/a/69125543/1153203
///
/// let mut registers = Registers::default();
/// let mut intermediate_registers = IntermediateRegisters::default();
///
/// // Operation that produces carry
/// perform_add_with_carry(0xFFFF, 0xFFFF, &mut registers, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0xFFFE);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
///
/// // Do the same operation now that the carry bit is set
/// perform_add_with_carry(0xFFFF, 0xFFFF, &mut registers, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0xFFFF);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
///
/// // Wrapping to zero
/// perform_add_with_carry(0x0000, 0xFFFF, &mut registers, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x0000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// ```
///
pub fn perform_add_with_carry(
    a: u16,
    b: u16,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    let carry_from_previous = u16::from(sr_bit_is_set(StatusRegisterFields::Carry, registers));

    let (r1, c1) = a.overflowing_add(b);
    let (r2, c2) = r1.overflowing_add(carry_from_previous);
    set_alu_bits(registers, r2, c1 | c2, Some((a, b, r2)));

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
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set, StatusRegisterFields};
/// use peripheral_cpu::execution::shared::IntermediateRegisters;
/// use peripheral_cpu::execution::alu::perform_subtract;
///
/// // Thanks: http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
///
/// let mut registers = Registers::default();
/// let mut intermediate_registers = IntermediateRegisters::default();
///
/// // Unsigned Overflow
/// perform_subtract(0x5FFF, 0xFFFF, &mut registers, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x6000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), false);
///
/// // Signed Overflow
/// perform_subtract(0xDFFF, 0x7FFF, &mut registers, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x6000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), false);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), true);
///
/// // Both Overflow
/// perform_subtract(0x5FFF, 0xBFFF, &mut registers, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0xA000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), true);
/// ```
///
pub fn perform_subtract(
    a: u16,
    b: u16,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    let (result, carry) = a.overflowing_sub(b);
    //The ones compliment of val_2 is used here for the overflow calculation because it is subtraction
    set_alu_bits(registers, result, carry, Some((a, !b, result)));

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
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set, StatusRegisterFields};
/// use peripheral_cpu::execution::shared::IntermediateRegisters;
/// use peripheral_cpu::execution::alu::perform_subtract_with_carry;
///
/// // Thanks: http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
///
/// let mut registers = Registers::default();
/// let mut intermediate_registers = IntermediateRegisters::default();
///
/// // Subtraction that causes borrow
/// perform_subtract_with_carry(0x0000, 0xFFFF, &mut registers, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x0001);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
///
/// // Subtraction with borrow (carry) bit set
///  perform_subtract_with_carry(0x0000, 0xFFFF, &mut registers, &mut intermediate_registers);
///
/// assert_eq!(intermediate_registers.alu_output, 0x0000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// ```
///
pub fn perform_subtract_with_carry(
    a: u16,
    b: u16,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    let carry_from_previous = u16::from(sr_bit_is_set(StatusRegisterFields::Carry, registers));

    let (r1, c1) = a.overflowing_sub(b);
    let (r2, c2) = r1.overflowing_sub(carry_from_previous);
    //The ones compliment of val_2 is used here for the overflow calculation because it is subtraction
    set_alu_bits(registers, r2, c1 | c2, Some((a, !b, r2)));

    intermediate_registers.alu_output = r2;
}

// Logic

fn perform_and(
    a: u16,
    b: u16,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    let result = a & b;
    set_alu_bits(registers, result, false, Some((a, b, result)));

    intermediate_registers.alu_output = result;
}

fn perform_or(
    a: u16,
    b: u16,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    let result = a | b;
    set_alu_bits(registers, result, false, Some((a, b, result)));

    intermediate_registers.alu_output = result;
}

fn perform_xor(
    a: u16,
    b: u16,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    let result = a ^ b;
    set_alu_bits(registers, result, false, Some((a, b, result)));

    intermediate_registers.alu_output = result;
}

// Shifts

fn perform_logical_left_shift(
    a: u16,
    b: u16,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    let extended_a = a as u32;
    let clamped_b = b.clamp(0, u16::BITS as u16);

    // There doesn't seem to be a built in method to shift left and get a flag
    // when a bit goes off the end so we can calculate carry.
    // Therefore, we can shift a 32 bit value temporarily and check manually if a bit
    // goes off the end. The hardware implementation will look different to this
    let wide_result = extended_a.shl(clamped_b);
    let result = wide_result as u16;
    let carry = (wide_result.rotate_left(u16::BITS) & 0x1) == 0x1;

    set_alu_bits(registers, result, carry, None);

    intermediate_registers.alu_output = result;
}

fn perform_logical_right_shift(
    a: u16,
    b: u16,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    let extended_a = a as u32;
    let clamped_b = b.clamp(0, u16::BITS as u16);

    // There doesn't seem to be a built in method to shift right and get a flag
    // when a bit goes off the end so we can calculate carry.
    // Therefore, we can shift a 32 bit value temporarily and check manually if a bit
    // rotates into the first position on the left side.
    // The hardware implementation will look different to this
    let wide_result = extended_a.rotate_right(clamped_b as u32);
    let result = wide_result as u16;
    let carry = (wide_result & MSB_MASK) == MSB_MASK;

    set_alu_bits(registers, result, carry, None);

    intermediate_registers.alu_output = result;
}

fn perform_arithmetic_left_shift(
    a: u16,
    b: u16,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    // Same as LSL but will set the overflow bit if the sign changes

    let extended_a = a as u32;
    let clamped_b = b.clamp(0, u16::BITS as u16);

    // There doesn't seem to be a built in method to shift left and get a flag
    // when a bit goes off the end so we can calculate carry.
    // Therefore, we can shift a 32 bit value temporarily and check manually if a bit
    // goes off the end. The hardware implementation will look different to this
    let wide_result = extended_a.shl(clamped_b);
    let result = wide_result as u16;
    let carry = (wide_result.rotate_left(u16::BITS) & 0x1) == 0x1;

    set_alu_bits(registers, result, carry, Some((a, a, result)));

    intermediate_registers.alu_output = result;
}

fn perform_arithmetic_right_shift(
    a: u16,
    b: u16,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    // Same as LSR but preserves the sign bit

    let sign_bit = a as u32 & 0x8000;

    let extended_a = a as u32;
    let clamped_b = b.clamp(0, u16::BITS as u16);

    // There doesn't seem to be a built in method to shift right and get a flag
    // when a bit goes off the end so we can calculate carry.
    // Therefore, we can shift a 32 bit value temporarily and check manually if a bit
    // rotates into the first position on the left side.
    // The hardware implementation will look different to this
    let wide_result = extended_a.rotate_right(clamped_b as u32) | sign_bit;
    let result = wide_result as u16;
    let carry = (wide_result & MSB_MASK) == MSB_MASK;

    set_alu_bits(registers, result, carry, Some((a, a, result)));

    intermediate_registers.alu_output = result;
}

fn perform_rotate_left(
    a: u16,
    b: u16,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    let result: u16 = a.rotate_left(b as u32);
    // Bit shifted out goes into carry (that's what the 68k does :shrug:)
    // TODO: Extract bit checking to function
    // TODO: Check the purpose of bit shifted out going to carry
    let carry = result & 1 == 1;
    set_alu_bits(registers, result, carry, None);
    intermediate_registers.alu_output = result;
}

fn perform_rotate_right(
    a: u16,
    b: u16,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    let result: u16 = a.rotate_right(b as u32);
    // Bit shifted out goes into carry (that's what the 68k does :shrug:)
    // TODO: Extract bit checking to function
    // TODO: Check the purpose of bit shifted out going to carry
    let carry = (result >> 15) & 1 == 1;
    set_alu_bits(registers, result, carry, None);
    intermediate_registers.alu_output = result;
}

pub fn perform_alu_operation(
    alu_op: AluOp,
    a: u16,
    b: u16,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    match alu_op {
        AluOp::Add => perform_add(a, b, registers, intermediate_registers),
        AluOp::AddWithCarry => perform_add_with_carry(a, b, registers, intermediate_registers),
        AluOp::Subtract => perform_subtract(a, b, registers, intermediate_registers),
        AluOp::SubtractWithCarry => {
            perform_subtract_with_carry(a, b, registers, intermediate_registers)
        }
        AluOp::And => perform_and(a, b, registers, intermediate_registers),
        AluOp::Or => perform_or(a, b, registers, intermediate_registers),
        AluOp::Xor => perform_xor(a, b, registers, intermediate_registers),
        AluOp::LogicalShiftLeft => {
            perform_logical_left_shift(a, b, registers, intermediate_registers)
        }
        AluOp::LogicalShiftRight => {
            perform_logical_right_shift(a, b, registers, intermediate_registers)
        }
        AluOp::ArithmeticShiftLeft => {
            perform_arithmetic_left_shift(a, b, registers, intermediate_registers)
        }
        AluOp::ArithmeticShiftRight => {
            perform_arithmetic_right_shift(a, b, registers, intermediate_registers)
        }
        AluOp::RotateLeft => perform_rotate_left(a, b, registers, intermediate_registers),
        AluOp::RotateRight => perform_rotate_right(a, b, registers, intermediate_registers),
        AluOp::Compare => perform_subtract(a, b, registers, intermediate_registers), // Same as subtract in ALU land - handled differently in the write back CPU phase
        AluOp::Reserved1 => todo!(),
        AluOp::Reserved2 => todo!(),
    }
}

pub fn set_alu_bits(
    registers: &mut Registers,
    value: u16,
    carry: bool,
    inputs_and_result: Option<(u16, u16, u16)>,
) {
    if value == 0 {
        set_sr_bit(StatusRegisterFields::Zero, registers);
    } else {
        clear_sr_bit(StatusRegisterFields::Zero, registers);
    }
    if (value as i16) < 0 {
        set_sr_bit(StatusRegisterFields::Negative, registers);
    } else {
        clear_sr_bit(StatusRegisterFields::Negative, registers);
    }
    if carry {
        set_sr_bit(StatusRegisterFields::Carry, registers);
    } else {
        clear_sr_bit(StatusRegisterFields::Carry, registers);
    }

    // See http://www.csc.villanova.edu/~mdamian/Past/csc2400fa16/labs/ALU.html
    // The logic is follows: when adding, if the sign of the two inputs is the same, but the result sign is different, then we have an overflow.
    if let Some((i1, i2, result)) = inputs_and_result {
        if sign(i1) == sign(i2) && sign(result) != sign(i1) {
            set_sr_bit(StatusRegisterFields::Overflow, registers);
        } else {
            // TODO: Can we collapse this if statement so we only have one else?
            // The double clear seems redundant
            clear_sr_bit(StatusRegisterFields::Overflow, registers);
        }
    } else {
        clear_sr_bit(StatusRegisterFields::Overflow, registers);
    }
}
