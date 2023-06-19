// Decode

use super::definitions::*;

const INSTRUCTION_ID_LENGTH: u32 = 6; // bits
const INSTRUCTION_ID_MASK: u32 = 0x0000003F;
const REGISTER_ID_LENGTH: u32 = 4; // bits
const REGISTER_ID_MASK: u32 = 0x0000000F;
// TODO: These 6 bits here in between the registers and the address register reference might be useful for something
const REGISTER_EXTRA_ARGS_LENGTH: u32 = 6; // bits
const REGISTER_ARGS_LENGTH: u32 = 2; // bits
const REGISTER_ARGS_MASK: u32 = 0x00000003;
const IMMEDIATE_ARGS_LENGTH: u32 = 2; // bits
const IMMEDIATE_ARGS_MASK: u32 = 0x00000003;
const CONDITION_FLAGS_MASK: u32 = 0x0000000F;
const VALUE_LENGTH: u32 = 16;
const VALUE_MASK: u32 = 0x0000FFFF;
pub const ADDRESS_MASK: u32 = 0x00FFFFFF;

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
/// assert_eq!(decode_implied_instruction([0x90, 0x00, 0x00, 0x00]), ImpliedInstructionData {
///     op_code: Instruction::ReturnFromSubroutine, condition_flag: ConditionFlags::Always
/// });
///
/// ```
pub fn decode_implied_instruction(raw_instruction: [u8; 4]) -> ImpliedInstructionData {
    let op_code = decode_instruction_id(raw_instruction);
    let condition_flag = decode_condition_flags(raw_instruction);
    // No args are used at this point (reserved for more complex instructions)
    ImpliedInstructionData {
        // TODO: Handle better than unwrap
        op_code: num::FromPrimitive::from_u8(op_code).unwrap(),
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
///     op_code: Instruction::BranchImmediate, register: 0x03, value: 0xFFFF, condition_flag: ConditionFlags::Always, additional_flags: 0x3
/// });
/// assert_eq!(decode_immediate_instruction([0xA0, 0x00, 0x00, 0x00]), ImmediateInstructionData {
///     op_code: Instruction::LongJumpToSubroutineWithImmediateDisplacement, register: 0x00, value: 0x0000, condition_flag: ConditionFlags::Always, additional_flags: 0x0
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
    let additional_flags = combined
        .rotate_left(initial_offset + REGISTER_ID_LENGTH + VALUE_LENGTH + IMMEDIATE_ARGS_LENGTH)
        & IMMEDIATE_ARGS_MASK;
    // No args are used at this point (reserved for more complex instructions)
    ImmediateInstructionData {
        // TODO: Handle better than unwrap
        op_code: num::FromPrimitive::from_u8(op_code).unwrap(),
        register: register as u8,
        value: value as u16,
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
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, ConditionFlags, Instruction};
/// use peripheral_cpu::instructions::encoding::decode_register_instruction;
///
/// assert_eq!(decode_register_instruction([0x80, 0x48, 0xC0, 0xF0]), RegisterInstructionData {
///     op_code: Instruction::BranchImmediate,
///     r1: 0x01,
///     r2: 0x02,
///     r3: 0x03,
///     condition_flag: ConditionFlags::Always,
///     additional_flags: 0x03
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
    let additional_flags = combined.rotate_left(
        (initial_offset + REGISTER_ID_LENGTH * 3)
            + REGISTER_EXTRA_ARGS_LENGTH
            + REGISTER_ARGS_LENGTH,
    ) & REGISTER_ARGS_MASK;
    RegisterInstructionData {
        // TODO: Handle better than unwrap
        op_code: num::FromPrimitive::from_u8(op_code).unwrap(),
        r1: r1 as u8,
        r2: r2 as u8,
        r3: r3 as u8,
        condition_flag,
        additional_flags: additional_flags as u8,
    }
}

// Since it is a "16-bit" processor, we read/write 16 bits at a time (align on 16 bits)
pub fn decode_instruction(raw_instruction: [u8; 4]) -> InstructionData {
    let instruction_id = decode_instruction_id(raw_instruction);
    match instruction_id {
        0x01..=0x0F => InstructionData::Immediate(decode_immediate_instruction(raw_instruction)), // Immediate arithmetic/logic (e.g. SUBI, XORI, SJMP)
        0x10..=0x1F => InstructionData::Register(decode_register_instruction(raw_instruction)), // Register-Register arithmetic/logic (e.g. SUBR, XORR, CMPR)
        0x20..=0x21 => InstructionData::Immediate(decode_immediate_instruction(raw_instruction)), // Branch (BRAN, BRSR)
        0x22..=0x26 => InstructionData::Implied(decode_implied_instruction(raw_instruction)), // Long jumps and transfers (LJMP, TSTA)
        0x30 => InstructionData::Immediate(decode_immediate_instruction(raw_instruction)), // LOAD Immediate
        0x31 => InstructionData::Register(decode_register_instruction(raw_instruction)), // LOAD R-R
        0x32 => InstructionData::Immediate(decode_immediate_instruction(raw_instruction)), // LOAD Indirect Immediate
        0x33 => InstructionData::Register(decode_register_instruction(raw_instruction)), // LOAD Indirect Register
        0x34 => InstructionData::Register(decode_register_instruction(raw_instruction)), // LOAD Post Increment
        0x35 => InstructionData::Immediate(decode_immediate_instruction(raw_instruction)), // STOR Indirect Immediate
        0x36 => InstructionData::Register(decode_register_instruction(raw_instruction)), // STOR Indirect Register
        0x37 => InstructionData::Register(decode_register_instruction(raw_instruction)), // STOR Pre Decrement
        0x3D..=0x3F => InstructionData::Implied(decode_implied_instruction(raw_instruction)), // Special (exception and noop)
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
    let a = (op_code_raw).rotate_right(INSTRUCTION_ID_LENGTH);
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
/// }), [0x81, 0x32, 0xBF, 0x9C]);
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
    // TODO: Unwrap?
    let op_code_raw = num::ToPrimitive::to_u32(op_code).unwrap();
    let a = (op_code_raw).rotate_right(INSTRUCTION_ID_LENGTH);
    let b = (*register as u32).rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH);
    let c = (*value as u32).rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH + VALUE_LENGTH);
    let d = (*additional_flags as u32).rotate_right(
        INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH + VALUE_LENGTH + IMMEDIATE_ARGS_LENGTH,
    );
    let e = encode_condition_flags(condition_flag);
    u32::to_be_bytes(a | b | c | d | e)
}

///
/// Encodes a "register" instruction (an instruction operates on 1-3 registers)
///
/// 6 bit instruction identifier (max 64 instructions)
/// 4 bit register identifier
/// 4 bit register identifier
/// 4 bit register identifier (if any)
/// 8 bit args
/// 4 bit condition flags
///
/// ```
/// use peripheral_cpu::instructions::encoding::encode_register_instruction;
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, ConditionFlags, Instruction};
///
/// assert_eq!(encode_register_instruction(&RegisterInstructionData {
///     op_code: Instruction::XorRegister,
///     r1: 0x0C,
///     r2: 0x0C,
///     r3: 0x0C,
///     condition_flag: ConditionFlags::UnsignedLowerOrSame,
///     additional_flags: 0x11,
/// }), [0x5B, 0x33, 0x01, 0x1A]);
///
/// ```
pub fn encode_register_instruction(
    RegisterInstructionData {
        op_code,
        r1,
        r2,
        r3,
        condition_flag,
        additional_flags,
    }: &RegisterInstructionData,
) -> [u8; 4] {
    let op_code_raw = num::ToPrimitive::to_u32(op_code).unwrap();
    let a = op_code_raw.rotate_right(INSTRUCTION_ID_LENGTH);
    let b = (*r1 as u32).rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH);
    let c = (*r2 as u32).rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH * 2);
    let d = (*r3 as u32).rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH * 3);
    let e = (*additional_flags as u32).rotate_right(
        (INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH * 4) + REGISTER_EXTRA_ARGS_LENGTH,
    );
    let f = encode_condition_flags(condition_flag);

    u32::to_be_bytes(a | b | c | d | e | f)
}

pub fn encode_instruction(instruction_data: &InstructionData) -> [u8; 4] {
    match instruction_data {
        InstructionData::Implied(data) => encode_implied_instruction(data),
        InstructionData::Immediate(data) => encode_immediate_instruction(data),
        InstructionData::Register(data) => encode_register_instruction(data),
    }
}
