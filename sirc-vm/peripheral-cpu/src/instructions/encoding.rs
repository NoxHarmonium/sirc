use super::definitions::*;

const INSTRUCTION_ID_LENGTH: u32 = 6; // bits
const INSTRUCTION_ID_MASK: u32 = 0x0000003F;
const REGISTER_ID_LENGTH: u32 = 4; // bits
const REGISTER_ID_MASK: u32 = 0x0000000F;
const ADDRESS_REGISTER_ARGS_LENGTH: u32 = 2; // bits
const ADDRESS_REGISTER_ARGS_MASK: u32 = 0x00000003;
const VALUE_LENGTH: u32 = 16;
const VALUE_MASK: u32 = 0x0000FFFF;
const SHORT_VALUE_LENGTH: u32 = 8;
const SHORT_VALUE_MASK: u32 = 0x000000FF;
const SHIFT_OPERAND_TYPE_LENGTH: u32 = 1; // bit
const SHIFT_OPERAND_TYPE_MASK: u32 = 0x00000001;
const SHIFT_TYPE_ARGS_LENGTH: u32 = 3; // bits
const SHIFT_TYPE_ARGS_MASK: u32 = 0x000000007;
const SHIFT_COUNT_ARGS_LENGTH: u32 = 4; // bits
const SHIFT_COUNT_ARGS_MASK: u32 = 0x00000000F;
const SHIFT_ALL_LENGTH: u32 =
    SHIFT_OPERAND_TYPE_LENGTH + SHIFT_TYPE_ARGS_LENGTH + SHIFT_COUNT_ARGS_LENGTH; // bits

const CONDITION_FLAGS_LENGTH: u32 = 4; // bits
const CONDITION_FLAGS_MASK: u32 = 0x0000000F;

///
/// Extracts the instruction ID from a full 32 bit instruction.
/// This is the same for every instruction type
///
/// ```
/// use peripheral_cpu::instructions::encoding::decode_instruction_id;
///
/// assert_eq!(decode_instruction_id([0x00, 0x00, 0x00, 0x00]), 0);
/// assert_eq!(decode_instruction_id([0xA0, 0x00, 0x00, 0x00]), 40);
/// assert_eq!(decode_instruction_id([0xF0, 0x00, 0x00, 0x00]), 60);
/// assert_eq!(decode_instruction_id([0xFC, 0x00, 0x00, 0x00]), 63);
/// // Max value is 63, any higher value will clamp to 63
/// assert_eq!(decode_instruction_id([0xFF, 0x00, 0x00, 0x00]), 63);
/// ```
pub fn decode_instruction_id(raw_instruction: [u8; 4]) -> u8 {
    // First 6 bits of every instruction is its ID
    let combined = u32::from_be_bytes(raw_instruction);
    ((combined.rotate_left(INSTRUCTION_ID_LENGTH)) & INSTRUCTION_ID_MASK) as u8
}

pub fn decode_condition_flags(raw_instruction: [u8; 4]) -> ConditionFlags {
    // Last 4 bits of every instruction are the condition flags
    let combined = u32::from_be_bytes(raw_instruction);
    let raw_value = (combined & CONDITION_FLAGS_MASK) as u8;
    num::FromPrimitive::from_u8(raw_value).expect("Condition flag can only be 4 bits long")
}

pub fn decode_shift_operand(raw_instruction: [u8; 4]) -> ShiftOperand {
    let combined = u32::from_be_bytes(raw_instruction);
    let raw_value = ((combined
        >> (CONDITION_FLAGS_LENGTH
            + ADDRESS_REGISTER_ARGS_LENGTH
            + SHIFT_COUNT_ARGS_LENGTH
            + SHIFT_TYPE_ARGS_LENGTH))
        & SHIFT_OPERAND_TYPE_MASK) as u8;
    num::FromPrimitive::from_u8(raw_value).expect("Shift operand can only be one or zero")
}

pub fn decode_shift_type(raw_instruction: [u8; 4]) -> ShiftType {
    let combined = u32::from_be_bytes(raw_instruction);
    let raw_value = ((combined
        >> (CONDITION_FLAGS_LENGTH + ADDRESS_REGISTER_ARGS_LENGTH + SHIFT_COUNT_ARGS_LENGTH))
        & SHIFT_TYPE_ARGS_MASK) as u8;
    num::FromPrimitive::from_u8(raw_value).expect("Shift type can only be 3 bits long")
}

pub fn decode_shift_count(raw_instruction: [u8; 4]) -> u8 {
    let combined = u32::from_be_bytes(raw_instruction);
    ((combined >> (CONDITION_FLAGS_LENGTH + ADDRESS_REGISTER_ARGS_LENGTH)) & SHIFT_COUNT_ARGS_MASK)
        as u8
}

///
/// Decodes the arguments for an "implied" instruction (an instruction has no arguments)
///
/// 6 bit instruction identifier (max 64 instructions)
/// 22 bit reserved
/// 4 bit conditions flags
///
/// ```
/// use peripheral_cpu::instructions::encoding::{decode_implied_instruction};
/// use peripheral_cpu::instructions::definitions::{Instruction, ImpliedInstructionData, ConditionFlags};
///
/// assert_eq!(decode_implied_instruction([0xF0, 0xBF, 0xEA, 0x80]), ImpliedInstructionData {
///     op_code: Instruction::NoOperation, condition_flag: ConditionFlags::Always
/// });
/// assert_eq!(decode_implied_instruction([0xF4, 0xFF, 0xFF, 0xF0]), ImpliedInstructionData {
///     op_code: Instruction::WaitForException, condition_flag: ConditionFlags::Always
/// });
///
/// ```
pub fn decode_implied_instruction(raw_instruction: [u8; 4]) -> ImpliedInstructionData {
    let op_code = decode_instruction_id(raw_instruction);
    let condition_flag = decode_condition_flags(raw_instruction);
    // No args are used at this point (reserved for more complex instructions)
    ImpliedInstructionData {
        // TODO: Handle better than unwrap
        op_code: num::FromPrimitive::from_u8(op_code)
            .expect("instruction ID to to map to instruction enum"),
        condition_flag,
    }
}

///
/// Decodes the arguments for an "immediate" instruction (an instruction that applies a value to
/// a register)
///
/// 6 bit instruction identifier (max 64 instructions)
/// 4 bit register identifier
/// 16 bit value
/// 2 bit reserved
/// 4 bit conditions flags
///
/// ```
/// use peripheral_cpu::instructions::encoding::{decode_immediate_instruction, encode_immediate_instruction};
/// use peripheral_cpu::instructions::definitions::{Instruction, ImmediateInstructionData, ConditionFlags};
///
/// assert_eq!(decode_immediate_instruction([0x08, 0xBF, 0xEA, 0x80]), ImmediateInstructionData {
///     op_code: Instruction::SubtractImmediate, register: 0x02, value: 0xFFAA, condition_flag: ConditionFlags::Always, additional_flags: 0x0
/// });
/// assert_eq!(decode_immediate_instruction([0x80, 0xFF, 0xFF, 0xF0]), ImmediateInstructionData {
///     op_code: Instruction::AddShortImmediate, register: 0x03, value: 0xFFFF, condition_flag: ConditionFlags::Always, additional_flags: 0x3
/// });
/// assert_eq!(decode_immediate_instruction([0xA0, 0x00, 0x00, 0x00]), ImmediateInstructionData {
///     op_code: Instruction::TestAndShortImmediate, register: 0x00, value: 0x0000, condition_flag: ConditionFlags::Always, additional_flags: 0x0
/// });
///
/// ```
pub fn decode_immediate_instruction(raw_instruction: [u8; 4]) -> ImmediateInstructionData {
    let combined = u32::from_be_bytes(raw_instruction);
    let op_code = decode_instruction_id(raw_instruction);
    let condition_flag = decode_condition_flags(raw_instruction);
    let initial_offset = INSTRUCTION_ID_LENGTH;
    let register = combined.rotate_left(initial_offset + REGISTER_ID_LENGTH) & REGISTER_ID_MASK;
    let value =
        combined.rotate_left(initial_offset + REGISTER_ID_LENGTH + VALUE_LENGTH) & VALUE_MASK;
    let additional_flags = combined.rotate_left(
        initial_offset + REGISTER_ID_LENGTH + VALUE_LENGTH + ADDRESS_REGISTER_ARGS_LENGTH,
    ) & ADDRESS_REGISTER_ARGS_MASK;
    // No args are used at this point (reserved for more complex instructions)
    ImmediateInstructionData {
        // TODO: Handle better than unwrap
        op_code: num::FromPrimitive::from_u8(op_code)
            .expect("instruction ID to to map to instruction enum"),
        register: register as u8,
        value: value as u16,
        condition_flag,
        additional_flags: additional_flags as u8,
    }
}

///
/// Decodes the arguments for an short "immediate" instruction (an instruction that applies a value to
/// a register)
///
/// A short immediate instruction has a value of only 8 bits so that a shift can be squeezed in.
///
/// 6 bit instruction identifier (max 64 instructions)
/// 4 bit register identifier
/// 8 bit value
/// 8 bit shift
/// 2 bit address register a, p or s (if any)
/// 4 bit conditions flags
///
/// ```
/// use peripheral_cpu::instructions::encoding::{decode_short_immediate_instruction, encode_immediate_instruction};
/// use peripheral_cpu::instructions::definitions::{Instruction, ShortImmediateInstructionData, ConditionFlags, ShiftType, ShiftOperand};
///
/// assert_eq!(decode_short_immediate_instruction([0x08, 0xBF, 0xEA, 0x80]), ShortImmediateInstructionData {
///     op_code: Instruction::SubtractImmediate, register: 0x02, value: 0xFF, shift_operand: ShiftOperand::Register, shift_type: ShiftType::LogicalRightShift, shift_count: 10, condition_flag: ConditionFlags::Always, additional_flags: 0x0
/// });
/// assert_eq!(decode_short_immediate_instruction([0x80, 0xFF, 0xFF, 0xF0]), ShortImmediateInstructionData {
///     op_code: Instruction::AddShortImmediate, register: 0x03, value: 0xFF, shift_operand: ShiftOperand::Register, shift_type: ShiftType::Reserved, shift_count: 15, condition_flag: ConditionFlags::Always, additional_flags: 0x3
/// });
/// assert_eq!(decode_short_immediate_instruction([0xA0, 0x00, 0x00, 0x00]), ShortImmediateInstructionData {
///     op_code: Instruction::TestAndShortImmediate, shift_operand: ShiftOperand::Immediate, shift_type: ShiftType::None, shift_count: 0, register: 0x0, value: 0x0, condition_flag: ConditionFlags::Always, additional_flags: 0
/// });
///
/// ```
pub fn decode_short_immediate_instruction(
    raw_instruction: [u8; 4],
) -> ShortImmediateInstructionData {
    let combined = u32::from_be_bytes(raw_instruction);
    let op_code = decode_instruction_id(raw_instruction);
    let condition_flag = decode_condition_flags(raw_instruction);
    let initial_offset = INSTRUCTION_ID_LENGTH;
    let register = combined.rotate_left(initial_offset + REGISTER_ID_LENGTH) & REGISTER_ID_MASK;
    let value = combined.rotate_left(initial_offset + REGISTER_ID_LENGTH + SHORT_VALUE_LENGTH)
        & SHORT_VALUE_MASK;
    let shift_operand = decode_shift_operand(raw_instruction);
    let shift_type = decode_shift_type(raw_instruction);
    let shift_count = decode_shift_count(raw_instruction);
    let additional_flags = combined.rotate_left(
        initial_offset
            + REGISTER_ID_LENGTH
            + SHORT_VALUE_LENGTH
            + SHIFT_ALL_LENGTH
            + ADDRESS_REGISTER_ARGS_LENGTH,
    ) & ADDRESS_REGISTER_ARGS_MASK;

    // TODO TODO: Adjust al these offsets because shift operand messed them p

    // No args are used at this point (reserved for more complex instructions)
    ShortImmediateInstructionData {
        // TODO: Handle better than unwrap
        op_code: num::FromPrimitive::from_u8(op_code)
            .expect("instruction ID to to map to instruction enum"),
        register: register as u8,
        value: value as u8,
        shift_operand,
        shift_type,
        shift_count,
        condition_flag,
        additional_flags: additional_flags as u8,
    }
}

///
/// Decodes the arguments for a "register" instruction (an instruction operates on 1-3 registers)
///
/// 6 bit instruction identifier (max 64 instructions)
/// 4 bit register identifier
/// 4 bit register identifier
/// 4 bit register identifier (if any)
/// 8 bit args
/// 4 bit condition flags
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, ConditionFlags, Instruction, ShiftType, ShiftOperand};
/// use peripheral_cpu::instructions::encoding::decode_register_instruction;
///
/// assert_eq!(decode_register_instruction([0xDA, 0xAF, 0x08, 0xEA]), RegisterInstructionData {
///     op_code: Instruction::XorRegister,
///     r1: 0x0A,
///     r2: 0x0B,
///     r3: 0x0C,
///     shift_operand: ShiftOperand::Immediate,
///     shift_type: ShiftType::LogicalRightShift,
///     shift_count: 3,
///     condition_flag: ConditionFlags::UnsignedLowerOrSame,
///     additional_flags: 0x02
/// });
///
/// ```
pub fn decode_register_instruction(raw_instruction: [u8; 4]) -> RegisterInstructionData {
    let combined = u32::from_be_bytes(raw_instruction);
    let condition_flag = decode_condition_flags(raw_instruction);
    let op_code = decode_instruction_id(raw_instruction);
    let initial_offset = INSTRUCTION_ID_LENGTH;
    let r1 = combined.rotate_left(initial_offset + REGISTER_ID_LENGTH) & REGISTER_ID_MASK;
    let r2 = combined.rotate_left(initial_offset + REGISTER_ID_LENGTH * 2) & REGISTER_ID_MASK;
    let r3 = combined.rotate_left(initial_offset + REGISTER_ID_LENGTH * 3) & REGISTER_ID_MASK;
    let shift_operand = decode_shift_operand(raw_instruction);
    let shift_type = decode_shift_type(raw_instruction);
    let shift_count = decode_shift_count(raw_instruction);
    let additional_flags = combined.rotate_left(
        (initial_offset + REGISTER_ID_LENGTH * 3) + SHIFT_ALL_LENGTH + ADDRESS_REGISTER_ARGS_LENGTH,
    ) & ADDRESS_REGISTER_ARGS_MASK;
    RegisterInstructionData {
        // TODO: Handle better than unwrap
        op_code: num::FromPrimitive::from_u8(op_code)
            .expect("instruction ID to to map to instruction enum"),
        r1: r1 as u8,
        r2: r2 as u8,
        r3: r3 as u8,
        shift_operand,
        shift_type,
        shift_count,
        condition_flag,
        additional_flags: additional_flags as u8,
    }
}

// Since it is a "16-bit" processor, we read/write 16 bits at a time (align on 16 bits)
pub fn decode_instruction(raw_instruction: [u8; 4]) -> InstructionData {
    let instruction_id = decode_instruction_id(raw_instruction);
    match instruction_id {
        0x00..=0x0F => InstructionData::Immediate(decode_immediate_instruction(raw_instruction)), // Immediate arithmetic/logic and short jumps (e.g. SUBI, XORI)
        0x10 => InstructionData::Immediate(decode_immediate_instruction(raw_instruction)), // LDEA Indirect Immediate
        0x11 => InstructionData::Register(decode_register_instruction(raw_instruction)), // LDEA Indirect Register
        0x12 => InstructionData::Immediate(decode_immediate_instruction(raw_instruction)), // LJMP Indirect Immediate
        0x13 => InstructionData::Register(decode_register_instruction(raw_instruction)), // LJMP Indirect Register
        0x14 => InstructionData::Immediate(decode_immediate_instruction(raw_instruction)), // LJSR Indirect Immediate
        0x15 => InstructionData::Register(decode_register_instruction(raw_instruction)), // LJSR Indirect Register
        0x16 => InstructionData::Immediate(decode_immediate_instruction(raw_instruction)), // LOAD Immediate
        0x17 => InstructionData::Register(decode_register_instruction(raw_instruction)), // LOAD R-R
        0x18 => InstructionData::Immediate(decode_immediate_instruction(raw_instruction)), //
        0x19 => InstructionData::Register(decode_register_instruction(raw_instruction)), //
        0x1A => InstructionData::Immediate(decode_immediate_instruction(raw_instruction)), //
        0x1B => InstructionData::Register(decode_register_instruction(raw_instruction)), //
        0x1C => InstructionData::Immediate(decode_immediate_instruction(raw_instruction)), //
        0x1D => InstructionData::Register(decode_register_instruction(raw_instruction)), //
        0x1E => InstructionData::Immediate(decode_immediate_instruction(raw_instruction)), //
        0x1F => InstructionData::Register(decode_register_instruction(raw_instruction)), //
        0x20..=0x2F => {
            InstructionData::ShortImmediate(decode_short_immediate_instruction(raw_instruction))
        } // SHORT Immediate arithmetic/logic and short jumps (e.g. SUBI, XORI)
        0x30..=0x3A => InstructionData::Register(decode_register_instruction(raw_instruction)), // Register-Register arithmetic/logic (e.g. SUBR, XORR, CMPR)
        0x3B..=0x3F => InstructionData::Implied(decode_implied_instruction(raw_instruction)), // RETS, NOOP, WAIY, RETE
        _ => panic!("Fatal: Invalid instruction ID: 0x{:02x}", instruction_id),
    }
}

// Encode

///
/// Encodes a condition flag enum into a 32 bit integer that can be ORed with
/// a 32 bit instruction to apply the condition flag to it.
///
pub fn encode_condition_flags(condition_flags: &ConditionFlags) -> u32 {
    // Last 4 bits of every instruction are the condition flags
    // Therefore it should be safe to just convert this to 32 bit int and OR it with the final instruction data
    num::ToPrimitive::to_u32(condition_flags).expect("Condition flag should fit into 32 bits")
        & CONDITION_FLAGS_MASK
}

///
/// Encodes a shift operand enum into a 32 bit integer that can be ORed with
/// a 32 bit instruction to apply the condition flag to it.
///
pub fn encode_shift_operand(shift_operand: &ShiftOperand) -> u32 {
    let raw_flags = num::ToPrimitive::to_u32(shift_operand)
        .expect("Rotate type should fit into 32 bits")
        & SHIFT_OPERAND_TYPE_MASK;
    raw_flags
        << (CONDITION_FLAGS_LENGTH
            + ADDRESS_REGISTER_ARGS_LENGTH
            + SHIFT_TYPE_ARGS_LENGTH
            + SHIFT_COUNT_ARGS_LENGTH)
}

///
/// Encodes a shift type enum into a 32 bit integer that can be ORed with
/// a 32 bit instruction to apply the condition flag to it.
///
pub fn encode_shift_type(shift_type: &ShiftType) -> u32 {
    let raw_flags = num::ToPrimitive::to_u32(shift_type)
        .expect("Rotate type should fit into 32 bits")
        & SHIFT_TYPE_ARGS_MASK;
    raw_flags << (CONDITION_FLAGS_LENGTH + ADDRESS_REGISTER_ARGS_LENGTH + SHIFT_COUNT_ARGS_LENGTH)
}

///
/// Encodes a shift count into a 32 bit integer that can be ORed with
/// a 32 bit instruction to apply the condition flag to it.
///
pub fn encode_shift_count(shift_count: u8) -> u32 {
    let raw_flags = shift_count as u32 & SHIFT_COUNT_ARGS_MASK;
    raw_flags << (CONDITION_FLAGS_LENGTH + ADDRESS_REGISTER_ARGS_LENGTH)
}

///
/// Encodes all the shift components of an instruction into a 32 bit integer that can be ORed with
/// a 32 bit instruction to apply the condition flag to it.
///
pub fn encode_shift(shift_operand: &ShiftOperand, shift_type: &ShiftType, shift_count: u8) -> u32 {
    encode_shift_operand(shift_operand)
        | encode_shift_type(shift_type)
        | encode_shift_count(shift_count)
}

///
/// Encodes an "implied" instruction (an instruction has no arguments)
///
/// 6 bit instruction identifier (max 64 instructions)
/// 22 bit reserved
/// 4 bit conditions flags
///
/// ```
/// use peripheral_cpu::instructions::encoding::encode_implied_instruction;
/// use peripheral_cpu::instructions::definitions::{ImpliedInstructionData, ConditionFlags, Instruction};
///
/// assert_eq!(encode_implied_instruction(&ImpliedInstructionData {
///   op_code: Instruction::NoOperation,
///   condition_flag: ConditionFlags::LessThan,
/// }), [0xF0, 0x00, 0x00, 0x0C]);
///
/// ```
pub fn encode_implied_instruction(
    ImpliedInstructionData {
        op_code,
        condition_flag,
    }: &ImpliedInstructionData,
) -> [u8; 4] {
    // TODO: Unwrap?
    let op_code_raw = num::ToPrimitive::to_u32(op_code).unwrap();
    let a = (op_code_raw & INSTRUCTION_ID_MASK).rotate_right(INSTRUCTION_ID_LENGTH);
    let b = encode_condition_flags(condition_flag);
    u32::to_be_bytes(a | b)
}

///
/// Encodes an "immediate" instruction (an instruction that applies a value to
/// a register)
///
/// 6 bit instruction identifier (max 64 instructions)
/// 4 bit register identifier
/// 16 bit value
/// 2 bit reserved
/// 4 bit conditions flags
///
/// ```
/// use peripheral_cpu::instructions::encoding::encode_immediate_instruction;
/// use peripheral_cpu::instructions::definitions::{ImmediateInstructionData, ConditionFlags, Instruction};
///
///
/// assert_eq!(encode_immediate_instruction(&ImmediateInstructionData {
///   op_code: Instruction::BranchImmediate,
///   register: 0x4,
///   value: 0xCAFE,
///   condition_flag: ConditionFlags::LessThan,
///   additional_flags: 0x1,
/// }), [0x2D, 0x32, 0xBF, 0x9C]);
///
/// ```
pub fn encode_immediate_instruction(
    ImmediateInstructionData {
        op_code,
        register,
        value,
        condition_flag,
        additional_flags,
    }: &ImmediateInstructionData,
) -> [u8; 4] {
    let op_code_raw =
        num::ToPrimitive::to_u32(op_code).expect("instruction should fit into 32 bits");
    let a = (op_code_raw & INSTRUCTION_ID_MASK).rotate_right(INSTRUCTION_ID_LENGTH);
    let b = (*register as u32 & REGISTER_ID_MASK)
        .rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH);
    let c = (*value as u32 & VALUE_MASK)
        .rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH + VALUE_LENGTH);
    let d = (*additional_flags as u32 & ADDRESS_REGISTER_ARGS_MASK).rotate_right(
        INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH + VALUE_LENGTH + ADDRESS_REGISTER_ARGS_LENGTH,
    );

    let e = encode_condition_flags(condition_flag);
    u32::to_be_bytes(a | b | c | d | e)
}

///
/// Encodes a short "immediate" instruction (an instruction that applies a value to
/// a register)
///
/// The immediate value is only 8 bit so a shift value can fit in.
///
/// 6 bit instruction identifier (max 64 instructions)
/// 4 bit register identifier
/// 8 bit value
/// 8 bit shift
/// 2 bit reserved
/// 4 bit conditions flags
///
/// ```
/// use peripheral_cpu::instructions::encoding::encode_short_immediate_instruction;
/// use peripheral_cpu::instructions::definitions::{ShortImmediateInstructionData, ConditionFlags, Instruction, ShiftOperand, ShiftType};
///
///
/// assert_eq!(encode_short_immediate_instruction(&ShortImmediateInstructionData {
///   op_code: Instruction::BranchShortImmediate,
///   register: 0x4,
///   value: 0xFE,
///   shift_operand: ShiftOperand::Register,
///   shift_type: ShiftType::RotateRight,
///   shift_count: 2,
///   condition_flag: ConditionFlags::LessThan,
///   additional_flags: 0x1,
/// }), [0xAD, 0x3F, 0xB8, 0x9C]);
///
/// ```
pub fn encode_short_immediate_instruction(
    ShortImmediateInstructionData {
        op_code,
        register,
        value,
        shift_operand,
        shift_type,
        shift_count,
        condition_flag,
        additional_flags,
    }: &ShortImmediateInstructionData,
) -> [u8; 4] {
    let op_code_raw =
        num::ToPrimitive::to_u32(op_code).expect("instruction should fit into 32 bits");
    let a = (op_code_raw & INSTRUCTION_ID_MASK).rotate_right(INSTRUCTION_ID_LENGTH);
    let b = (*register as u32 & REGISTER_ID_MASK)
        .rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH);
    let c = (*value as u32 & SHORT_VALUE_MASK)
        .rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH + SHORT_VALUE_LENGTH);
    let d = encode_shift(shift_operand, shift_type, *shift_count);
    let e = (*additional_flags as u32 & ADDRESS_REGISTER_ARGS_MASK).rotate_right(
        INSTRUCTION_ID_LENGTH
            + REGISTER_ID_LENGTH
            + SHORT_VALUE_LENGTH
            + SHIFT_ALL_LENGTH
            + ADDRESS_REGISTER_ARGS_LENGTH,
    );
    let f = encode_condition_flags(condition_flag);
    u32::to_be_bytes(a | b | c | d | e | f)
}

///
/// Encodes a "register" instruction (an instruction operates on 1-3 registers)
///
/// 6 bit instruction identifier (max 64 instructions)
/// 4 bit register identifier
/// 4 bit register identifier
/// 4 bit register identifier (if any)
/// 8 bit shift
/// 2 bit address register a, p or s (if any)
/// 4 bit condition flags
///
/// ```
/// use peripheral_cpu::instructions::encoding::encode_register_instruction;
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, ConditionFlags, Instruction, ShiftType, ShiftOperand};
///
/// assert_eq!(encode_register_instruction(&RegisterInstructionData {
///     op_code: Instruction::XorRegister,
///     r1: 0x0A,
///     r2: 0x0B,
///     r3: 0x0C,
///     shift_operand: ShiftOperand::Immediate,
///     shift_type: ShiftType::LogicalRightShift,
///     shift_count: 3,
///     condition_flag: ConditionFlags::UnsignedLowerOrSame,
///     additional_flags: 0x2,
/// }), [0xDA, 0xAF, 0x08, 0xEA]);
///
/// ```
pub fn encode_register_instruction(
    RegisterInstructionData {
        op_code,
        r1,
        r2,
        r3,
        shift_operand,
        shift_type,
        shift_count,
        condition_flag,
        additional_flags,
    }: &RegisterInstructionData,
) -> [u8; 4] {
    let op_code_raw =
        num::ToPrimitive::to_u32(op_code).expect("instruction should fit into 32 bits");
    let a = (op_code_raw & INSTRUCTION_ID_MASK).rotate_right(INSTRUCTION_ID_LENGTH);
    let b =
        (*r1 as u32 & REGISTER_ID_MASK).rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH);
    let c = (*r2 as u32 & REGISTER_ID_MASK)
        .rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH * 2);
    let d = (*r3 as u32 & REGISTER_ID_MASK)
        .rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH * 3);
    let e = encode_shift(shift_operand, shift_type, *shift_count);
    let f = (*additional_flags as u32 & ADDRESS_REGISTER_ARGS_MASK).rotate_right(
        (INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH * 3)
            + SHIFT_ALL_LENGTH
            + ADDRESS_REGISTER_ARGS_LENGTH,
    );
    let g = encode_condition_flags(condition_flag);
    u32::to_be_bytes(a | b | c | d | e | f | g)
}

pub fn encode_instruction(instruction_data: &InstructionData) -> [u8; 4] {
    match instruction_data {
        InstructionData::Implied(data) => encode_implied_instruction(data),
        InstructionData::Immediate(data) => encode_immediate_instruction(data),
        InstructionData::ShortImmediate(data) => encode_short_immediate_instruction(data),
        InstructionData::Register(data) => encode_register_instruction(data),
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use quickcheck::{Arbitrary, Gen};

    const VALID_IMPLIED_OP_CODES: &[Instruction] = &[
        Instruction::ReturnFromSubroutine,
        Instruction::NoOperation,
        Instruction::WaitForException,
        Instruction::ReturnFromException,
    ];

    const VALID_IMMEDIATE_OP_CODES: &[Instruction] = &[
        Instruction::AddImmediate,
        Instruction::AddImmediateWithCarry,
        Instruction::SubtractImmediate,
        Instruction::SubtractImmediateWithCarry,
        Instruction::AndImmediate,
        Instruction::OrImmediate,
        Instruction::XorImmediate,
        Instruction::CompareImmediate,
        Instruction::TestAndImmediate,
        Instruction::TestXorImmediate,
        Instruction::ShiftImmediate,
        Instruction::BranchImmediate,
        Instruction::BranchToSubroutineImmediate,
        Instruction::ShortJumpImmediate,
        Instruction::ShortJumpToSubroutineImmediate,
        Instruction::Exception,
        Instruction::LoadEffectiveAddressFromIndirectImmediate,
        Instruction::LongJumpWithImmediateDisplacement,
        Instruction::LongJumpToSubroutineWithImmediateDisplacement,
        Instruction::LoadRegisterFromImmediate,
        Instruction::LoadRegisterFromIndirectImmediate,
        Instruction::StoreRegisterToIndirectImmediate,
    ];

    const VALID_SHORT_IMMEDIATE_OP_CODES: &[Instruction] = &[
        Instruction::AddShortImmediate,
        Instruction::AddShortImmediateWithCarry,
        Instruction::SubtractShortImmediate,
        Instruction::SubtractShortImmediateWithCarry,
        Instruction::AndShortImmediate,
        Instruction::OrShortImmediate,
        Instruction::XorShortImmediate,
        Instruction::CompareShortImmediate,
        Instruction::TestAndShortImmediate,
        Instruction::TestXorShortImmediate,
        Instruction::ShiftShortImmediate,
        Instruction::BranchShortImmediate,
        Instruction::BranchToSubroutineShortImmediate,
        Instruction::ShortJumpShortImmediate,
        Instruction::ShortJumpToSubroutineShortImmediate,
        Instruction::ExceptionShort,
    ];

    const VALID_REGISTER_OP_CODES: &[Instruction] = &[
        Instruction::AddRegister,
        Instruction::AddRegisterWithCarry,
        Instruction::SubtractRegister,
        Instruction::SubtractRegisterWithCarry,
        Instruction::AndRegister,
        Instruction::OrRegister,
        Instruction::XorRegister,
        Instruction::CompareRegister,
        Instruction::TestAndRegister,
        Instruction::TestXorRegister,
        Instruction::ShiftRegister,
        Instruction::LoadEffectiveAddressFromIndirectRegister,
        Instruction::LongJumpWithRegisterDisplacement,
        Instruction::LongJumpToSubroutineWithRegisterDisplacement,
        Instruction::LoadRegisterFromRegister,
        Instruction::LoadRegisterFromIndirectRegister,
        Instruction::LoadRegisterFromIndirectRegisterPostIncrement,
        Instruction::StoreRegisterToIndirectRegister,
        Instruction::StoreRegisterToIndirectRegisterPreDecrement,
    ];

    use crate::instructions::{
        definitions::{
            all_condition_flags, all_instructions, all_shift_operands, all_shift_types,
            ImmediateInstructionData, ImpliedInstructionData, Instruction, InstructionData,
            RegisterInstructionData, ShortImmediateInstructionData,
        },
        encoding::decode_instruction,
    };

    use super::encode_instruction;

    fn check_instruction_coverage() {
        let instructions = all_instructions();
        let mut instruction_set: HashSet<&Instruction> = instructions.iter().collect();
        let valid_op_code_count_total = VALID_IMPLIED_OP_CODES.len()
            + VALID_IMMEDIATE_OP_CODES.len()
            + VALID_SHORT_IMMEDIATE_OP_CODES.len()
            + VALID_REGISTER_OP_CODES.len();

        let a: Vec<_> = VALID_IMPLIED_OP_CODES
            .iter()
            .filter(|&i| !instruction_set.remove(i))
            .collect();

        let b: Vec<_> = VALID_IMMEDIATE_OP_CODES
            .iter()
            .filter(|&i| !instruction_set.remove(i))
            .collect();

        let c: Vec<_> = VALID_SHORT_IMMEDIATE_OP_CODES
            .iter()
            .filter(|&i| !instruction_set.remove(i))
            .collect();

        let d: Vec<_> = VALID_REGISTER_OP_CODES
            .iter()
            .filter(|&i| !instruction_set.remove(i))
            .collect();

        let missing_op_codes: Vec<&Instruction> =
            vec![a, b, c, d].iter().flatten().copied().collect();

        assert_eq!(valid_op_code_count_total, instructions.len());
        assert_eq!(missing_op_codes, Vec::<&Instruction>::new());
        assert_eq!(instruction_set.len(), 0);
    }

    impl Arbitrary for ImpliedInstructionData {
        fn arbitrary(g: &mut Gen) -> ImpliedInstructionData {
            check_instruction_coverage();

            ImpliedInstructionData {
                condition_flag: *g.choose(all_condition_flags().as_slice()).unwrap(),
                op_code: g.choose(VALID_IMPLIED_OP_CODES).unwrap().to_owned(),
            }
        }
    }

    impl Arbitrary for ImmediateInstructionData {
        fn arbitrary(g: &mut Gen) -> ImmediateInstructionData {
            check_instruction_coverage();

            ImmediateInstructionData {
                condition_flag: *g.choose(all_condition_flags().as_slice()).unwrap(),
                op_code: g.choose(VALID_IMMEDIATE_OP_CODES).unwrap().to_owned(),
                additional_flags: u8::arbitrary(g) & 0x3,
                register: u8::arbitrary(g) & 0xF,
                value: u16::arbitrary(g),
            }
        }
    }

    impl Arbitrary for ShortImmediateInstructionData {
        fn arbitrary(g: &mut Gen) -> ShortImmediateInstructionData {
            check_instruction_coverage();

            ShortImmediateInstructionData {
                condition_flag: *g.choose(all_condition_flags().as_slice()).unwrap(),
                op_code: g.choose(VALID_SHORT_IMMEDIATE_OP_CODES).unwrap().to_owned(),
                additional_flags: u8::arbitrary(g) & 0x3,
                register: u8::arbitrary(g) & 0xF,
                value: u8::arbitrary(g),
                shift_operand: *g.choose(all_shift_operands().as_slice()).unwrap(),
                shift_type: *g.choose(all_shift_types().as_slice()).unwrap(),
                shift_count: u8::arbitrary(g) & 0xF,
            }
        }
    }

    impl Arbitrary for RegisterInstructionData {
        fn arbitrary(g: &mut Gen) -> RegisterInstructionData {
            check_instruction_coverage();

            RegisterInstructionData {
                condition_flag: *g.choose(all_condition_flags().as_slice()).unwrap(),
                op_code: g.choose(VALID_REGISTER_OP_CODES).unwrap().to_owned(),
                r1: u8::arbitrary(g) & 0xF,
                r2: u8::arbitrary(g) & 0xF,
                r3: u8::arbitrary(g) & 0xF,
                shift_operand: *g.choose(all_shift_operands().as_slice()).unwrap(),
                shift_type: *g.choose(all_shift_types().as_slice()).unwrap(),
                shift_count: u8::arbitrary(g) & 0xF,
                additional_flags: u8::arbitrary(g) & 0x3,
            }
        }
    }

    impl Arbitrary for InstructionData {
        fn arbitrary(g: &mut Gen) -> InstructionData {
            let choices = vec![
                InstructionData::Implied(ImpliedInstructionData::arbitrary(g)),
                InstructionData::Immediate(ImmediateInstructionData::arbitrary(g)),
                InstructionData::ShortImmediate(ShortImmediateInstructionData::arbitrary(g)),
                InstructionData::Register(RegisterInstructionData::arbitrary(g)),
            ];

            g.choose(choices.as_slice()).unwrap().to_owned()
        }
    }

    #[quickcheck]
    fn round_trip_encoding_test(instruction_data: InstructionData) -> bool {
        let raw_bytes = encode_instruction(&instruction_data);
        let decoded = decode_instruction(raw_bytes);
        instruction_data == decoded
    }
}
