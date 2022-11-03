// Decode

use core::mem::size_of;

use super::definitions::*;

const INSTRUCTION_ID_LENGTH: u32 = 6; // bits
const INSTRUCTION_ID_MASK: u32 = 0x0000003F;
const REGISTER_ID_LENGTH: u32 = 4; // bits
const REGISTER_ID_MASK: u32 = 0x0000000F;
const CONDITION_FLAGS_MASK: u32 = 0x0000000F;
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
    let condition_flag = decode_condition_flags(raw_instruction);

    match instruction_id {
        0x00 => Instruction::Halt(HaltInstructionData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x01 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::Add(AddInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x02 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::Subtract(SubtractInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x03 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::Multiply(MultiplyInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x04 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::Divide(DivideInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x05 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::And(AndInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x06 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::Or(OrInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x07 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::Xor(XorInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x08 => {
            let (src_register, dest_register, _) = decode_register_instruction(raw_instruction);
            Instruction::Compare(CompareInstructionData {
                data: RegisterInstructionData {
                    r1: src_register,
                    r2: dest_register,
                    r3: 0x0000,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x09 => Instruction::ShortJump(ShortJumpInstructionData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x0A => Instruction::LongJump(LongJumpInstructionData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x0B => {
            let (register, value) = decode_immediate_instruction(raw_instruction);
            Instruction::Branch(BranchInstructionData {
                data: ImmediateInstructionData {
                    register,
                    value,
                    condition_flag,
                    additional_flags: 0x0,
                },
            })
        }
        0x0C => {
            let (register, value) = decode_immediate_instruction(raw_instruction);
            Instruction::LoadRegisterFromImmediate(LoadRegisterFromImmediateData {
                data: ImmediateInstructionData {
                    register,
                    value,
                    condition_flag,
                    additional_flags: 0x0,
                },
            })
        }
        0x0D => {
            let (r1, r2, r3) = decode_register_instruction(raw_instruction);
            Instruction::LoadRegisterFromRegister(LoadRegisterFromRegisterData {
                data: RegisterInstructionData {
                    r1,
                    r2,
                    r3,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x0E => {
            let (register, value) = decode_immediate_instruction(raw_instruction);
            Instruction::LoadRegisterFromIndirectImmediate(LoadRegisterFromIndirectImmediateData {
                data: ImmediateInstructionData {
                    register,
                    value,
                    condition_flag,
                    additional_flags: 0x0,
                },
            })
        }
        0x0F => {
            let (r1, r2, r3) = decode_register_instruction(raw_instruction);
            Instruction::LoadRegisterFromIndirectRegister(LoadRegisterFromIndirectRegisterData {
                data: RegisterInstructionData {
                    r1,
                    r2,
                    r3,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x12 => {
            let (register, value) = decode_immediate_instruction(raw_instruction);
            Instruction::StoreRegisterToIndirectImmediate(StoreRegisterToIndirectImmediateData {
                data: ImmediateInstructionData {
                    register,
                    value,
                    condition_flag,
                    additional_flags: 0x0,
                },
            })
        }
        0x13 => {
            let (r1, r2, r3) = decode_register_instruction(raw_instruction);
            Instruction::StoreRegisterToIndirectRegister(StoreRegisterToIndirectRegisterData {
                data: RegisterInstructionData {
                    r1,
                    r2,
                    r3,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x16 => Instruction::WaitForInterrupt(WaitForInterruptInstructionData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x17 => Instruction::ReturnFromInterrupt(ReturnFromInterruptData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x18 => {
            let (_, value) = decode_immediate_instruction(raw_instruction);
            Instruction::TriggerSoftwareInterrupt(TriggerSoftwareInterruptData {
                data: ImmediateInstructionData {
                    register: 0x0,
                    value,
                    condition_flag,
                    additional_flags: 0x0,
                },
            })
        }
        0x19 => Instruction::DisableInterrupts(DisableInterruptsData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x1A => Instruction::EnableInterrupts(EnableInterruptsData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x1B => {
            let (_, value) = decode_immediate_instruction(raw_instruction);
            Instruction::BranchToSubroutine(BranchToSubroutineData {
                data: ImmediateInstructionData {
                    register: 0x0,
                    value,
                    condition_flag,
                    additional_flags: 0x0,
                },
            })
        }
        0x1C => Instruction::ShortJumpToSubroutine(ShortJumpToSubroutineData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x1D => Instruction::LongJumpToSubroutine(LongJumpToSubroutineData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x1E => Instruction::ReturnFromSubroutine(ReturnFromSubroutineData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x1F => {
            let (r1, r2, _) = decode_register_instruction(raw_instruction);
            Instruction::LogicalShiftLeft(LogicalShiftLeftInstructionData {
                data: RegisterInstructionData {
                    r1,
                    r2,
                    r3: 0x0,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x20 => {
            let (r1, r2, _) = decode_register_instruction(raw_instruction);
            Instruction::LogicalShiftRight(LogicalShiftRightInstructionData {
                data: RegisterInstructionData {
                    r1,
                    r2,
                    r3: 0x0,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x21 => {
            let (r1, r2, _) = decode_register_instruction(raw_instruction);
            Instruction::ArithmeticShiftLeft(ArithmeticShiftLeftInstructionData {
                data: RegisterInstructionData {
                    r1,
                    r2,
                    r3: 0x0,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x22 => {
            let (r1, r2, _) = decode_register_instruction(raw_instruction);
            Instruction::ArithmeticShiftRight(ArithmeticShiftRightInstructionData {
                data: RegisterInstructionData {
                    r1,
                    r2,
                    r3: 0x0,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x23 => {
            let (r1, r2, _) = decode_register_instruction(raw_instruction);
            Instruction::RotateLeft(RotateLeftInstructionData {
                data: RegisterInstructionData {
                    r1,
                    r2,
                    r3: 0x0,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x24 => {
            let (r1, r2, _) = decode_register_instruction(raw_instruction);
            Instruction::RotateRight(RotateRightInstructionData {
                data: RegisterInstructionData {
                    r1,
                    r2,
                    r3: 0x0,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x25 => Instruction::NoOperation(NoOperationInstructionData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x26 => Instruction::ClearAluStatus(ClearAluStatusInstructionData {
            data: ImpliedInstructionData { condition_flag },
        }),
        0x27 => {
            let (r1, r2, r3) = decode_register_instruction(raw_instruction);
            Instruction::SplitWord(SplitWordInstructionData {
                data: RegisterInstructionData {
                    r1,
                    r2,
                    r3,
                    condition_flag,
                    additional_flags: 0x00,
                },
            })
        }
        0x28 => {
            let (r1, r2, r3) = decode_register_instruction(raw_instruction);
            Instruction::JoinWord(JoinWordInstructionData {
                data: RegisterInstructionData {
                    r1,
                    r2,
                    r3,
                    condition_flag,
                    additional_flags: 0x00,
                },
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
        Instruction::Add(data) => {
            encode_register_instruction(0x01, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::Subtract(data) => {
            encode_register_instruction(0x02, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::Multiply(data) => {
            encode_register_instruction(0x03, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::Divide(data) => {
            encode_register_instruction(0x04, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::And(data) => {
            encode_register_instruction(0x05, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::Or(data) => encode_register_instruction(0x06, data.data.r1, data.data.r2, 0x0),
        Instruction::Xor(data) => {
            encode_register_instruction(0x07, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::Compare(data) => {
            encode_register_instruction(0x08, data.data.r1, data.data.r2, 0x0)
        }

        Instruction::ShortJump(_) => encode_null_instruction(0x09),
        Instruction::LongJump(_) => encode_null_instruction(0x0A),
        Instruction::Branch(data) => encode_immediate_instruction(0x0B, 0x0, data.data.value),
        Instruction::LoadRegisterFromImmediate(data) => {
            encode_immediate_instruction(0x0C, data.data.register, data.data.value)
        }
        Instruction::LoadRegisterFromRegister(data) => {
            encode_register_instruction(0x0D, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::LoadRegisterFromIndirectImmediate(data) => {
            encode_immediate_instruction(0x0E, data.data.register, data.data.value)
        }
        Instruction::LoadRegisterFromIndirectRegister(data) => {
            encode_register_instruction(0x0F, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::StoreRegisterToIndirectImmediate(data) => {
            encode_immediate_instruction(0x12, data.data.register, data.data.value)
        }
        Instruction::StoreRegisterToIndirectRegister(data) => {
            encode_register_instruction(0x13, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::WaitForInterrupt(_) => encode_null_instruction(0x16),
        Instruction::ReturnFromInterrupt(_) => encode_null_instruction(0x17),
        Instruction::TriggerSoftwareInterrupt(_) => encode_null_instruction(0x18),
        Instruction::DisableInterrupts(_) => encode_null_instruction(0x19),
        Instruction::EnableInterrupts(_) => encode_null_instruction(0x1A),
        Instruction::BranchToSubroutine(data) => {
            encode_immediate_instruction(0x1B, 0x0, data.data.value)
        }
        Instruction::ShortJumpToSubroutine(_) => encode_null_instruction(0x1C),
        Instruction::LongJumpToSubroutine(_) => encode_null_instruction(0x1D),
        Instruction::ReturnFromSubroutine(_) => encode_null_instruction(0x1E),
        Instruction::LogicalShiftLeft(data) => {
            encode_register_instruction(0x1F, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::LogicalShiftRight(data) => {
            encode_register_instruction(0x20, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::ArithmeticShiftLeft(data) => {
            encode_register_instruction(0x21, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::ArithmeticShiftRight(data) => {
            encode_register_instruction(0x22, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::RotateLeft(data) => {
            encode_register_instruction(0x23, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::RotateRight(data) => {
            encode_register_instruction(0x24, data.data.r1, data.data.r2, 0x0)
        }
        Instruction::NoOperation(_) => encode_null_instruction(0x25),
        Instruction::ClearAluStatus(_) => encode_null_instruction(0x26),
        Instruction::SplitWord(data) => {
            encode_register_instruction(0x27, data.data.r1, data.data.r2, data.data.r3)
        }
        Instruction::JoinWord(data) => {
            encode_register_instruction(0x28, data.data.r1, data.data.r2, data.data.r3)
        }
    }
}
