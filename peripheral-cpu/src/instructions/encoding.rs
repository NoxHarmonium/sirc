// Decode

use core::mem::size_of;

use super::definitions::*;

const INSTRUCTION_ID_LENGTH: u32 = 6; // bits
const INSTRUCTION_ID_MASK: u32 = 0x0000003F;
const REGISTER_ID_LENGTH: u32 = 4; // bits
const REGISTER_ID_MASK: u32 = 0x0000000F;
const REGISTER_ARGS_LENGTH: u32 = 8; // bits
const REGISTER_ARGS_MASK: u32 = 0x000000FF;
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
/// use peripheral_cpu::instructions::definitions::{ImmediateInstructionData, ConditionFlags};
///
/// assert_eq!(decode_immediate_instruction([0x08, 0xBF, 0xEA, 0x80]), ImmediateInstructionData {
///     register: 0x02, value: 0xFFAA, condition_flag: ConditionFlags::Always, additional_flags: 0x0
/// });
/// assert_eq!(decode_immediate_instruction([0x0B, 0xFF, 0xFF, 0xF0]), ImmediateInstructionData {
///     register: 0x0F, value: 0xFFFF, condition_flag: ConditionFlags::Always, additional_flags: 0x3
/// });
/// assert_eq!(decode_immediate_instruction([0x00, 0x00, 0x00, 0x00]), ImmediateInstructionData {
///     register: 0x00, value: 0x0000, condition_flag: ConditionFlags::Always, additional_flags: 0x0
/// });
///
/// ```
pub fn decode_immediate_instruction(raw_instruction: [u8; 4]) -> ImmediateInstructionData {
    let combined = u32::from_be_bytes(raw_instruction);
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
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, ConditionFlags};
/// use peripheral_cpu::instructions::encoding::decode_register_instruction;
///
/// assert_eq!(decode_register_instruction([0x10, 0x48, 0xC0, 0xF0]), RegisterInstructionData {
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
    let initial_offset = INSTRUCTION_ID_LENGTH;
    let r1 = combined.rotate_left(initial_offset + REGISTER_ID_LENGTH) & REGISTER_ID_MASK;
    let r2 = combined.rotate_left(initial_offset + REGISTER_ID_LENGTH * 2) & REGISTER_ID_MASK;
    let r3 = combined.rotate_left(initial_offset + REGISTER_ID_LENGTH * 3) & REGISTER_ID_MASK;
    let additional_flags = combined
        .rotate_left((initial_offset + REGISTER_ID_LENGTH * 3) + REGISTER_ARGS_LENGTH)
        & REGISTER_ARGS_MASK;
    RegisterInstructionData {
        r1: r1 as u8,
        r2: r2 as u8,
        r3: r3 as u8,
        condition_flag,
        additional_flags: additional_flags as u8,
    }
}

// Since it is a "16-bit" processor, we read/write 16 bits at a time (align on 16 bits)
pub fn decode_instruction(raw_instruction: [u8; 4]) -> Instruction {
    let instruction_id = decode_instruction_id(raw_instruction);
    let condition_flag = decode_condition_flags(raw_instruction);

    match instruction_id {
        0x00 => Instruction::Halt(HaltInstructionData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x01 => Instruction::NoOperation(NoOperationInstructionData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x02 => Instruction::WaitForInterrupt(WaitForInterruptInstructionData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x03 => Instruction::ReturnFromInterrupt(ReturnFromInterruptData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x04 => Instruction::ReturnFromSubroutine(ReturnFromSubroutineData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x05 => Instruction::Add(AddInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x06 => Instruction::AddWithCarry(AddWithCarryInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x07 => Instruction::Subtract(SubtractInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x08 => Instruction::SubtractWithCarry(SubtractWithCarryInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x09 => Instruction::Multiply(MultiplyInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x0A => Instruction::Divide(DivideInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x0B => Instruction::And(AndInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x0C => Instruction::Or(OrInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x0D => Instruction::Xor(XorInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x0E => Instruction::LogicalShiftLeft(LogicalShiftLeftInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x0F => Instruction::LogicalShiftRight(LogicalShiftRightInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x10 => Instruction::ArithmeticShiftLeft(ArithmeticShiftLeftInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x11 => Instruction::ArithmeticShiftRight(ArithmeticShiftRightInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x12 => Instruction::RotateLeft(RotateLeftInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x13 => Instruction::RotateRight(RotateRightInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x14 => Instruction::Compare(CompareInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x15 => Instruction::Push(PushInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x16 => Instruction::Pop(PopInstructionData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x17 => Instruction::TriggerSoftwareInterrupt(TriggerSoftwareInterruptData {
            data: decode_immediate_instruction(raw_instruction),
        }),
        0x18 => Instruction::ShortJumpWithImmediate(ShortJumpWithImmediateData {
            data: decode_immediate_instruction(raw_instruction),
        }),
        0x19 => Instruction::ShortJumpToSubroutineWithImmediate(
            ShortJumpToSubroutineWithImmediateData {
                data: decode_immediate_instruction(raw_instruction),
            },
        ),
        0x1A => Instruction::BranchToSubroutine(BranchToSubroutineData {
            data: decode_immediate_instruction(raw_instruction),
        }),
        0x1B => Instruction::Branch(BranchInstructionData {
            data: decode_immediate_instruction(raw_instruction),
        }),
        0x1C => Instruction::LongJumpWithAddressRegister(LongJumpWithAddressRegisterData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x1D => Instruction::LongJumpToSubroutineWithAddressRegister(
            LongJumpToSubroutineWithAddressRegisterData {
                data: decode_register_instruction(raw_instruction),
            },
        ),
        0x1F => Instruction::LoadEffectiveAddressIndirectImmediate(
            LoadEffectiveAddressFromIndirectImmediateData {
                data: decode_immediate_instruction(raw_instruction),
            },
        ),
        0x20 => Instruction::LoadEffectiveAddressIndirectRegister(
            LoadEffectiveAddressFromIndirectRegisterData {
                data: decode_register_instruction(raw_instruction),
            },
        ),
        0x21 => Instruction::LoadManyRegisterFromAddressRegister(
            LoadManyRegisterFromAddressRegisterData {
                data: decode_register_instruction(raw_instruction),
            },
        ),
        0x22 => Instruction::LoadRegisterFromImmediate(LoadRegisterFromImmediateData {
            data: decode_immediate_instruction(raw_instruction),
        }),
        0x23 => Instruction::LoadRegisterFromRegister(LoadRegisterFromRegisterData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x24 => {
            Instruction::LoadRegisterFromIndirectImmediate(LoadRegisterFromIndirectImmediateData {
                data: decode_immediate_instruction(raw_instruction),
            })
        }
        0x25 => {
            Instruction::LoadRegisterFromIndirectRegister(LoadRegisterFromIndirectRegisterData {
                data: decode_register_instruction(raw_instruction),
            })
        }
        0x26 => {
            Instruction::StoreRegisterToIndirectImmediate(StoreRegisterToIndirectImmediateData {
                data: decode_immediate_instruction(raw_instruction),
            })
        }
        0x27 => Instruction::StoreRegisterToIndirectRegister(StoreRegisterToIndirectRegisterData {
            data: decode_register_instruction(raw_instruction),
        }),
        0x28 => Instruction::StoreManyRegisterFromAddressRegister(
            StoreManyRegisterFromAddressRegisterData {
                data: decode_register_instruction(raw_instruction),
            },
        ),
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
/// Encodes a "null" instruction (an instruction with no arguments)
///
/// The only variables are the instruction ID and condition flags,
/// other than that, the rest of the instruction will always be zeros.
///
/// 6 bit instruction identifier (max 64 instructions)
/// 22 bit reserved
/// 4 bit condition flags
///
/// ```
/// use peripheral_cpu::instructions::encoding::encode_null_instruction;
///
/// assert_eq!(encode_null_instruction(0x0A), [0x28, 0x00, 0x00, 0x00]);
///
/// ```
pub fn encode_null_instruction(instruction_id: u8) -> [u8; 4] {
    [
        instruction_id << (size_of::<u8>() * u8::BITS as usize - (INSTRUCTION_ID_LENGTH as usize)),
        0x0,
        0x0,
        0x0,
    ]
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
/// use peripheral_cpu::instructions::definitions::{ImmediateInstructionData, ConditionFlags};
///
///
/// assert_eq!(encode_immediate_instruction(0x02, &ImmediateInstructionData {
///   register: 0x01,
///   value: 0x0002,
///   condition_flag: ConditionFlags::Always,
///   additional_flags: 0x00,
/// }), [0x08, 0x40, 0x00, 0x80]);
///
/// ```
pub fn encode_immediate_instruction(
    instruction_id: u32,
    ImmediateInstructionData {
        register,
        value,
        condition_flag,
        additional_flags,
    }: &ImmediateInstructionData,
) -> [u8; 4] {
    let a = (instruction_id as u32).rotate_right(INSTRUCTION_ID_LENGTH);
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
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, ConditionFlags};
///
/// assert_eq!(encode_register_instruction(0x0C, &RegisterInstructionData {
///     r1: 0x0C,
///     r2: 0x0C,
///     r3: 0x0C,
///     condition_flag: ConditionFlags::Always,
///     additional_flags: 0x00,
/// }), [0x33, 0x33, 0x00, 0x00]);
///
/// ```
pub fn encode_register_instruction(
    instruction_id: u32,
    RegisterInstructionData {
        r1,
        r2,
        r3,
        condition_flag,
        additional_flags,
    }: &RegisterInstructionData,
) -> [u8; 4] {
    let a = instruction_id.rotate_right(INSTRUCTION_ID_LENGTH);
    let b = (*r1 as u32).rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH);
    let c = (*r2 as u32).rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH * 2);
    let d = (*r3 as u32).rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH * 3);
    let e = (*additional_flags as u32)
        .rotate_right((INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH * 4) + REGISTER_ARGS_LENGTH);
    let f = encode_condition_flags(condition_flag);

    u32::to_be_bytes(a | b | c | d | e | f)
}

pub fn encode_instruction(instruction: &Instruction) -> [u8; 4] {
    match instruction {
        Instruction::Halt(_) => encode_null_instruction(0x00),
        Instruction::NoOperation(_) => encode_null_instruction(0x01),
        Instruction::WaitForInterrupt(_) => encode_null_instruction(0x02),
        Instruction::ReturnFromInterrupt(_) => encode_null_instruction(0x03),
        Instruction::ReturnFromSubroutine(_) => encode_null_instruction(0x04),
        Instruction::Add(data) => encode_register_instruction(0x05, &data.data),
        Instruction::AddWithCarry(data) => encode_register_instruction(0x06, &data.data),
        Instruction::Subtract(data) => encode_register_instruction(0x07, &data.data),
        Instruction::SubtractWithCarry(data) => encode_register_instruction(0x08, &data.data),
        Instruction::Multiply(data) => encode_register_instruction(0x09, &data.data),
        Instruction::Divide(data) => encode_register_instruction(0x0A, &data.data),
        Instruction::And(data) => encode_register_instruction(0x0B, &data.data),
        Instruction::Or(data) => encode_register_instruction(0x0C, &data.data),
        Instruction::Xor(data) => encode_register_instruction(0x0D, &data.data),
        Instruction::LogicalShiftLeft(data) => encode_register_instruction(0x0E, &data.data),
        Instruction::LogicalShiftRight(data) => encode_register_instruction(0x0F, &data.data),
        Instruction::ArithmeticShiftLeft(data) => encode_register_instruction(0x10, &data.data),
        Instruction::ArithmeticShiftRight(data) => encode_register_instruction(0x11, &data.data),
        Instruction::RotateLeft(data) => encode_register_instruction(0x12, &data.data),
        Instruction::RotateRight(data) => encode_register_instruction(0x13, &data.data),
        Instruction::Compare(data) => encode_register_instruction(0x14, &data.data),
        Instruction::Push(data) => encode_register_instruction(0x15, &data.data),
        Instruction::Pop(data) => encode_register_instruction(0x16, &data.data),
        Instruction::TriggerSoftwareInterrupt(_) => encode_null_instruction(0x17),
        Instruction::ShortJumpWithImmediate(data) => encode_immediate_instruction(0x18, &data.data),
        Instruction::ShortJumpToSubroutineWithImmediate(data) => {
            encode_immediate_instruction(0x19, &data.data)
        }
        Instruction::BranchToSubroutine(data) => encode_immediate_instruction(0x1A, &data.data),
        Instruction::Branch(data) => encode_immediate_instruction(0x1B, &data.data),
        Instruction::LongJumpWithAddressRegister(data) => {
            encode_register_instruction(0x1C, &data.data)
        }
        Instruction::LongJumpToSubroutineWithAddressRegister(data) => {
            encode_register_instruction(0x1D, &data.data)
        }
        Instruction::LoadEffectiveAddressIndirectImmediate(data) => {
            encode_immediate_instruction(0x1F, &data.data)
        }
        Instruction::LoadEffectiveAddressIndirectRegister(data) => {
            encode_register_instruction(0x20, &data.data)
        }
        Instruction::LoadManyRegisterFromAddressRegister(data) => {
            encode_register_instruction(0x21, &data.data)
        }
        Instruction::LoadRegisterFromImmediate(data) => {
            encode_immediate_instruction(0x22, &data.data)
        }
        Instruction::LoadRegisterFromRegister(data) => {
            encode_register_instruction(0x23, &data.data)
        }
        Instruction::LoadRegisterFromIndirectImmediate(data) => {
            encode_immediate_instruction(0x24, &data.data)
        }
        Instruction::LoadRegisterFromIndirectRegister(data) => {
            encode_register_instruction(0x25, &data.data)
        }
        Instruction::StoreRegisterToIndirectImmediate(data) => {
            encode_immediate_instruction(0x26, &data.data)
        }
        Instruction::StoreRegisterToIndirectRegister(data) => {
            encode_register_instruction(0x27, &data.data)
        }
        Instruction::StoreManyRegisterFromAddressRegister(data) => {
            encode_register_instruction(0x28, &data.data)
        }
    }
}
