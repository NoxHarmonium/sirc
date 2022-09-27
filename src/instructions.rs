// Instruction (32 bit)
// 8 bit instruction identifier (max 256 instructions)
// 8 bit target register
//
// 16 bit value OR
// 8 bit target register
// 8 bit reserved

use crate::executors::Executor;
use crate::registers::Registers;
use enum_dispatch::enum_dispatch;

// 32 bits = 2x 16 bit
pub const instruction_size: u16 = 2;

// Special

#[derive(Debug)]
pub struct NullInstructionData {}

// Register Transfer
#[derive(Debug)]
pub struct SetInstructionData {
    pub register: u8,
    pub value: u16,
}

#[derive(Debug)]
pub struct CopyInstructionData {
    pub src_register: u8,
    pub dest_register: u8,
}

// Arithmetic

#[derive(Debug)]
pub struct AddInstructionData {
    pub src_register: u8,
    pub dest_register: u8,
}

#[derive(Debug)]
pub struct SubtractInstructionData {
    pub src_register: u8,
    pub dest_register: u8,
}

#[derive(Debug)]
pub struct MultiplyInstructionData {
    pub src_register: u8,
    pub dest_register: u8,
}

#[derive(Debug)]
pub struct DivideInstructionData {
    pub src_register: u8,
    pub dest_register: u8,
}

// Comparison

#[derive(Debug)]
pub struct IsEqualInstructionData {
    pub src_register: u8,
    pub dest_register: u8,
}

#[derive(Debug)]
pub struct IsNotEqualInstructionData {
    pub src_register: u8,
    pub dest_register: u8,
}

#[derive(Debug)]
pub struct IsLessThanInstructionData {
    pub src_register: u8,
    pub dest_register: u8,
}

#[derive(Debug)]
pub struct IsGreaterThanInstructionData {
    pub src_register: u8,
    pub dest_register: u8,
}

#[derive(Debug)]
pub struct IsLessOrEqualThanInstructionData {
    pub src_register: u8,
    pub dest_register: u8,
}

#[derive(Debug)]
pub struct IsGreaterOrEqualThanInstructionData {
    pub src_register: u8,
    pub dest_register: u8,
}

// Flow Control

#[derive(Debug)]
pub struct JumpInstructionData {
    pub new_pc: u16,
}

#[derive(Debug)]
pub struct JumpIfInstructionData {
    pub new_pc: u16,
}

#[derive(Debug)]
pub struct JumpIfNotInstructionData {
    pub new_pc: u16,
}

#[derive(Debug)]
#[enum_dispatch(Executor)]
pub enum Instruction {
    // Special
    HaltInstruction(NullInstructionData),
    // Register Transfer
    SetInstruction(SetInstructionData),
    CopyInstruction(CopyInstructionData),
    // Arithmetic
    AddInstruction(AddInstructionData),
    SubtractInstruction(SubtractInstructionData),
    MultiplyInstruction(MultiplyInstructionData),
    DivideInstruction(DivideInstructionData),
    // Comparison
    IsEqualInstruction(IsEqualInstructionData),
    IsNotEqualInstruction(IsNotEqualInstructionData),
    IsLessThanInstruction(IsLessThanInstructionData),
    IsGreaterThanInstruction(IsGreaterThanInstructionData),
    IsLessOrEqualThanInstruction(IsLessOrEqualThanInstructionData),
    IsGreaterOrEqualThanInstruction(IsGreaterOrEqualThanInstructionData),
    // Flow Control
    JumpInstruction(JumpInstructionData),
    JumpIfInstruction(JumpIfInstructionData),
    JumpIfNotInstruction(JumpIfNotInstructionData),
}

// Since it is a "16-bit" processor, we read/write 16 bits at a time (align on 16 bits)
pub fn decode_instruction(raw_instruction: [u16; 2]) -> Instruction {
    let [upper, lower] = raw_instruction;
    let [instruction_id, b1] = u16::to_le_bytes(upper);
    let [b2, b3] = u16::to_le_bytes(lower);

    match instruction_id {
        0x0 => Instruction::HaltInstruction(NullInstructionData {}),
        0x1 => Instruction::SetInstruction(SetInstructionData {
            register: b1,
            value: u16::from_le_bytes([b2, b3]),
        }),
        0x2 => Instruction::CopyInstruction(CopyInstructionData {
            src_register: b1,
            dest_register: b2,
        }),
        0x3 => Instruction::AddInstruction(AddInstructionData {
            src_register: b1,
            dest_register: b2,
        }),
        0x9 => Instruction::IsLessThanInstruction(IsLessThanInstructionData {
            src_register: b1,
            dest_register: b2,
        }),
        0xD => Instruction::JumpIfInstruction(JumpIfInstructionData {
            new_pc: u16::from_le_bytes([b2, b3]),
        }),
        _ => panic!("Fatal: Invalid instruction ID: {}", instruction_id),
    }
}

pub fn encode_instruction(instruction: &Instruction) -> [u16; 2] {
    match instruction {
        Instruction::HaltInstruction(_) => [0x00, 0x00],
        Instruction::SetInstruction(data) => [u16::from_le_bytes([0x1, data.register]), data.value],
        Instruction::CopyInstruction(data) => [
            u16::from_le_bytes([0x2, data.src_register]),
            u16::from_le_bytes([data.dest_register, 0x0]),
        ],
        Instruction::AddInstruction(data) => [
            u16::from_le_bytes([0x3, data.src_register]),
            u16::from_le_bytes([data.dest_register, 0x0]),
        ],
        Instruction::IsLessThanInstruction(data) => [
            u16::from_le_bytes([0x9, data.src_register]),
            u16::from_le_bytes([data.dest_register, 0x0]),
        ],
        Instruction::JumpIfInstruction(data) => [u16::from_le_bytes([0xD, 0x0]), data.new_pc],
        _ => panic!("Fatal: Invalid instruction: {:#?}", instruction),
    }
}

pub fn fetch_instruction(rom: &[u16], pc: u16) -> Option<[u16; 2]> {
    let address = (pc * instruction_size) as usize;
    // TODO: Alignment check?
    // TODO: Do we need to copy here?
    let upper = rom.get(address)?.to_owned();
    let lower = rom.get(address + 1)?.to_owned();
    return Some([upper, lower]);
}
