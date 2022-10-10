// Decode

use core::mem::size_of;

use super::definitions::*;

const INSTRUCTION_ID_LENGTH: u32 = 6; // bits
const INSTRUCTION_ID_MASK: u32 = 0x0000003F;
const REGISTER_ID_LENGTH: u32 = 4; // bits
const REGISTER_ID_MASK: u32 = 0x0000000F;
const VALUE_LENGTH: u32 = 16;
const VALUE_MASK: u32 = 0x0000FFFF;
const ADDRESS_LENGTH: u32 = 24;
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

///
/// Decodes the arguments for an "immediate" instruction (an instruction that applies a value to
/// a register)
///
/// 6 bit instruction identifier (max 64 instructions)
/// 4 bit register identifier
/// 16 bit value
/// 6 bit arguments (if any)
///
/// ```
/// use peripheral_cpu::instructions::encoding::decode_immediate_instruction;
///
/// assert_eq!(decode_immediate_instruction([0x08, 0xBF, 0xEA, 0x80]), (0x02, 0xFFAA));
/// assert_eq!(decode_immediate_instruction([0x0B, 0xFF, 0xFF, 0xF0]), (0x0F, 0xFFFF));
/// assert_eq!(decode_immediate_instruction([0x08, 0x00, 0x00, 0x00]), (0x00, 0x0000));
///
/// ```
pub fn decode_immediate_instruction(raw_instruction: [u8; 4]) -> (u8, u16) {
    let combined = u32::from_be_bytes(raw_instruction);
    let initial_offset = INSTRUCTION_ID_LENGTH;
    let r1 = combined.rotate_left(initial_offset + REGISTER_ID_LENGTH) & REGISTER_ID_MASK;
    let value =
        combined.rotate_left(initial_offset + REGISTER_ID_LENGTH + VALUE_LENGTH) & VALUE_MASK;

    // No args are used at this point (reserved for more complex instructions)
    (r1 as u8, value as u16)
}

///
/// Decodes the arguments for a "register" instruction (an instruction operates on 1-3 registers)
///
/// 6 bit instruction identifier (max 64 instructions)
/// 4 bit register identifier
/// 4 bit register identifier
/// 4 bit register identifier (if any)
/// 14 bit arguments (if any)
///
/// ```
/// use peripheral_cpu::instructions::encoding::decode_register_instruction;
///
/// assert_eq!(decode_register_instruction([0x10, 0x48, 0xC0, 0x00]), (0x01, 0x02, 0x03));
///
/// ```
pub fn decode_register_instruction(raw_instruction: [u8; 4]) -> (u8, u8, u8) {
    let combined = u32::from_be_bytes(raw_instruction);
    let initial_offset = INSTRUCTION_ID_LENGTH;
    let r1 = combined.rotate_left(initial_offset + REGISTER_ID_LENGTH) & REGISTER_ID_MASK;
    let r2 = combined.rotate_left(initial_offset + REGISTER_ID_LENGTH * 2) & REGISTER_ID_MASK;
    let r3 = combined.rotate_left(initial_offset + REGISTER_ID_LENGTH * 3) & REGISTER_ID_MASK;
    // No args are used at this point (reserved for more complex instructions)

    (r1 as u8, r2 as u8, r3 as u8)
}

///
/// Decodes the arguments for an "address" instruction (an instruction with a single 24 bit immediate value)
///
/// 6 bit instruction identifier (max 64 instructions)
/// 24 bit value
/// 2 bit arguments (if any)
///
/// ```
/// use peripheral_cpu::instructions::encoding::decode_address_instruction;
///
/// assert_eq!(decode_address_instruction([0x37, 0xFF, 0xFF, 0xFF]), 0x00FFFFFF);
/// assert_eq!(decode_address_instruction([0x37, 0x2B, 0xFB, 0x28]), 0x00CAFECA);
///
/// ```
pub fn decode_address_instruction(raw_instruction: [u8; 4]) -> u32 {
    let combined = u32::from_be_bytes(raw_instruction);
    let initial_offset = INSTRUCTION_ID_LENGTH;
    // No args are used at this point (reserved for more complex instructions)

    combined.rotate_left(initial_offset + ADDRESS_LENGTH) & ADDRESS_MASK
}

// Since it is a "16-bit" processor, we read/write 16 bits at a time (align on 16 bits)
pub fn decode_instruction(raw_instruction: [u8; 4]) -> Instruction {
    let instruction_id = decode_instruction_id(raw_instruction);

    match instruction_id {
        0x00 => Instruction::Halt(NullInstructionData {}),
        0x01 => {
            let (register, value) = decode_immediate_instruction(raw_instruction);
            Instruction::Set(SetInstructionData {
                data: ImmediateInstructionData { register, value },
            })
        }
        0x02 => {
            let address = decode_address_instruction(raw_instruction);
            Instruction::SetAddress(SetAddressInstructionData {
                data: AddressInstructionData { address },
            })
        }
        0x03 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::Copy(CopyInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                },
            })
        }
        0x04 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::Add(AddInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                },
            })
        }
        0x05 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::Subtract(SubtractInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                },
            })
        }
        0x06 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::Multiply(MultiplyInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                },
            })
        }
        0x07 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::Divide(DivideInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                },
            })
        }
        0x08 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::IsEqual(IsEqualInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                },
            })
        }
        0x09 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::IsNotEqual(IsNotEqualInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                },
            })
        }
        0x0A => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::IsLessThan(IsLessThanInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                },
            })
        }
        0x0B => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::IsGreaterThan(IsGreaterThanInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                },
            })
        }
        0x0C => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::IsLessOrEqualThan(IsLessOrEqualThanInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                },
            })
        }
        0x0D => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::IsGreaterOrEqualThan(IsGreaterOrEqualThanInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                },
            })
        }
        0x0E => {
            let address = decode_address_instruction(raw_instruction);
            Instruction::Jump(JumpInstructionData {
                data: AddressInstructionData { address },
            })
        }
        0x0F => {
            let address = decode_address_instruction(raw_instruction);
            Instruction::JumpIf(JumpIfInstructionData {
                data: AddressInstructionData { address },
            })
        }
        0x10 => {
            let address = decode_address_instruction(raw_instruction);
            Instruction::JumpIfNot(JumpIfNotInstructionData {
                data: AddressInstructionData { address },
            })
        }
        0x11 => {
            let (r1, r2, _) = decode_register_instruction(raw_instruction);
            Instruction::LoadOffsetRegister(LoadOffsetRegisterData {
                data: RegisterInstructionData { r1, r2, r3: 0x0000 },
            })
        }
        0x12 => {
            let (r1, r2, _) = decode_register_instruction(raw_instruction);
            Instruction::StoreOffsetRegister(StoreOffsetRegisterData {
                data: RegisterInstructionData { r1, r2, r3: 0x0000 },
            })
        }
        0x13 => {
            let (register, value) = decode_immediate_instruction(raw_instruction);
            Instruction::LoadOffsetImmediate(LoadOffsetImmediateData {
                data: ImmediateInstructionData { register, value },
            })
        }
        0x14 => {
            let (register, value) = decode_immediate_instruction(raw_instruction);
            Instruction::StoreOffsetImmediate(StoreOffsetImmediateData {
                data: ImmediateInstructionData { register, value },
            })
        }
        _ => panic!("Fatal: Invalid instruction ID: 0x{:02x}", instruction_id),
    }
}

// Encode

///
/// Encodes a "null" instruction (an instruction with no arguments)
///
/// The only variable is the instruction ID, other than that, the rest of the
/// instruction will always be zeros.
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
/// 6 bit arguments (if any)
///
/// ```
/// use peripheral_cpu::instructions::encoding::encode_immediate_instruction;
///
///
/// assert_eq!(encode_immediate_instruction(0x02, 0x01, 0x0002), [0x08, 0x40, 0x00, 0x80]);
///
/// ```
pub fn encode_immediate_instruction(instruction_id: u8, register: u8, value: u16) -> [u8; 4] {
    let a = (instruction_id as u32).rotate_right(INSTRUCTION_ID_LENGTH);
    let b = (register as u32).rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH);
    let c = (value as u32).rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH + VALUE_LENGTH);

    u32::to_be_bytes(a | b | c)
}

///
/// Encodes a "register" instruction (an instruction operates on 1-3 registers)
///
/// 6 bit instruction identifier (max 64 instructions)
/// 4 bit register identifier
/// 4 bit register identifier
/// 4 bit register identifier (if any)
/// 14 bit arguments (if any)
///
/// ```
/// use peripheral_cpu::instructions::encoding::encode_register_instruction;
/// /// 0011 0011 0011 0011 0000
/// assert_eq!(encode_register_instruction(0x0C, 0x0C, 0x0C, 0x0C), ([0x33, 0x33, 0x00, 0x00]));
///
/// ```
pub fn encode_register_instruction(instruction_id: u8, r1: u8, r2: u8, r3: u8) -> [u8; 4] {
    let a = (instruction_id as u32).rotate_right(INSTRUCTION_ID_LENGTH);
    let b = (r1 as u32).rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH);
    let c = (r2 as u32).rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH * 2);
    let d = (r3 as u32).rotate_right(INSTRUCTION_ID_LENGTH + REGISTER_ID_LENGTH * 3);

    u32::to_be_bytes(a | b | c | d)
}

///
/// Encodes an "address" instruction (an instruction with a single 24 bit immediate value)
///
/// 6 bit instruction identifier (max 64 instructions)
/// 24 bit value
/// 2 bit arguments (if any)
///
///  
/// ```
/// use peripheral_cpu::instructions::encoding::encode_address_instruction;
///
/// assert_eq!(encode_address_instruction(0x0C, 0xCAFECA), [0x33, 0x2B, 0xFB, 0x28]);
///
/// ```
pub fn encode_address_instruction(instruction_id: u8, value: u32) -> [u8; 4] {
    let a = (instruction_id as u32).rotate_right(INSTRUCTION_ID_LENGTH);
    let b = (value as u32).rotate_right(INSTRUCTION_ID_LENGTH + ADDRESS_LENGTH);

    u32::to_be_bytes(a | b)
}

pub fn encode_instruction(instruction: &Instruction) -> [u8; 4] {
    match instruction {
        Instruction::Halt(_) => encode_null_instruction(0x00),
        Instruction::Set(data) => {
            encode_immediate_instruction(0x01, data.data.register, data.data.value)
        }
        Instruction::SetAddress(data) => encode_address_instruction(0x02, data.data.address),
        Instruction::Copy(data) => {
            encode_register_instruction(0x03, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::Add(data) => {
            encode_register_instruction(0x04, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::Subtract(data) => {
            encode_register_instruction(0x05, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::Multiply(data) => {
            encode_register_instruction(0x06, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::Divide(data) => {
            encode_register_instruction(0x07, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::IsEqual(data) => {
            encode_register_instruction(0x08, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::IsNotEqual(data) => {
            encode_register_instruction(0x09, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::IsLessThan(data) => {
            encode_register_instruction(0x0A, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::IsGreaterThan(data) => {
            encode_register_instruction(0x0B, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::IsLessOrEqualThan(data) => {
            encode_register_instruction(0x0C, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::IsGreaterOrEqualThan(data) => {
            encode_register_instruction(0x0D, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::Jump(data) => encode_address_instruction(0x0E, data.data.address),
        Instruction::JumpIf(data) => encode_address_instruction(0x0F, data.data.address),
        Instruction::JumpIfNot(data) => encode_address_instruction(0x10, data.data.address),
        Instruction::LoadOffsetRegister(data) => {
            encode_register_instruction(0x11, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::StoreOffsetRegister(data) => {
            encode_register_instruction(0x12, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::LoadOffsetImmediate(data) => {
            encode_immediate_instruction(0x13, data.data.register, data.data.value)
        }
        Instruction::StoreOffsetImmediate(data) => {
            encode_immediate_instruction(0x14, data.data.register, data.data.value)
        }
    }
}
