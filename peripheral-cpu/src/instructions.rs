// Instruction (32 bit)
// Every instruction starts with:
// 8 bit instruction identifier (max 256 instructions)
//
// Instruction formats:
//
// Register Value: (e.g. SET)
// 8 bit register identifier
// 16 bit value
//
// Dual Register: (e.g. COPY)
// 8 bit register identifier
// 8 bit register identifier
// 8 bit padding
//
// Single Value: (e.g. JUMP)
// 16 bit value
// 8 bit padding

use peripheral_mem::MemoryPeripheral;

// 32 bits = 2x 16 bit
pub const INSTRUCTION_SIZE_WORDS: u16 = 2;
pub const INSTRUCTION_SIZE_BYTES: u16 = INSTRUCTION_SIZE_WORDS * 2;

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
pub enum Instruction {
    // Special
    Halt(NullInstructionData),
    // Register Transfer
    Set(SetInstructionData),
    Copy(CopyInstructionData),
    // Arithmetic
    Add(AddInstructionData),
    Subtract(SubtractInstructionData),
    Multiply(MultiplyInstructionData),
    Divide(DivideInstructionData),
    // Comparison
    IsEqual(IsEqualInstructionData),
    IsNotEqual(IsNotEqualInstructionData),
    IsLessThan(IsLessThanInstructionData),
    IsGreaterThan(IsGreaterThanInstructionData),
    IsLessOrEqualThan(IsLessOrEqualThanInstructionData),
    IsGreaterOrEqualThan(IsGreaterOrEqualThanInstructionData),
    // Flow Control
    Jump(JumpInstructionData),
    JumpIf(JumpIfInstructionData),
    JumpIfNot(JumpIfNotInstructionData),
}

// Decode

pub fn decode_reg_val_instruction(raw_instruction: [u16; 2]) -> (u8, u16) {
    let [_, r1] = u16::to_le_bytes(raw_instruction[0]);
    (r1, raw_instruction[1])
}

pub fn decode_reg_reg_instruction(raw_instruction: [u16; 2]) -> (u8, u8) {
    let [_, r1] = u16::to_le_bytes(raw_instruction[0]);
    let [r2, _] = u16::to_le_bytes(raw_instruction[1]);
    (r1, r2)
}

pub fn decode_val_instruction(raw_instruction: [u16; 2]) -> u16 {
    let [_, b1] = u16::to_le_bytes(raw_instruction[0]);
    let [b2, _] = u16::to_le_bytes(raw_instruction[1]);
    u16::from_le_bytes([b1, b2])
}

// Since it is a "16-bit" processor, we read/write 16 bits at a time (align on 16 bits)
pub fn decode_instruction(raw_instruction: [u16; 2]) -> Instruction {
    let [instruction_id, _] = u16::to_le_bytes(raw_instruction[0]);

    match instruction_id {
        0x0 => Instruction::Halt(NullInstructionData {}),
        0x1 => {
            let (register, value) = decode_reg_val_instruction(raw_instruction);
            Instruction::Set(SetInstructionData { register, value })
        }
        0x2 => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::Copy(CopyInstructionData {
                src_register,
                dest_register,
            })
        }
        0x3 => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::Add(AddInstructionData {
                src_register,
                dest_register,
            })
        }
        0x9 => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::IsLessThan(IsLessThanInstructionData {
                src_register,
                dest_register,
            })
        }
        0xA => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::IsGreaterThan(IsGreaterThanInstructionData {
                src_register,
                dest_register,
            })
        }
        0xE => {
            let new_pc = decode_val_instruction(raw_instruction);
            Instruction::JumpIf(JumpIfInstructionData { new_pc })
        }
        _ => panic!("Fatal: Invalid instruction ID: 0x{:02x}", instruction_id),
    }
}

// Encode

pub fn encode_null_instruction(instruction_id: u8) -> [u16; 2] {
    [u16::from_le_bytes([instruction_id, 0x0]), 0x00]
}

pub fn encode_reg_val_instruction(instruction_id: u8, register: u8, value: u16) -> [u16; 2] {
    [u16::from_le_bytes([instruction_id, register]), value]
}

pub fn encode_reg_reg_instruction(
    instruction_id: u8,
    src_register: u8,
    dest_register: u8,
) -> [u16; 2] {
    [
        u16::from_le_bytes([instruction_id, src_register]),
        u16::from_le_bytes([dest_register, 0x0]),
    ]
}

pub fn encode_val_instruction(instruction_id: u8, value: u16) -> [u16; 2] {
    let [b1, b2] = u16::to_le_bytes(value);
    [
        u16::from_le_bytes([instruction_id, b1]),
        u16::from_le_bytes([b2, 0x0]),
    ]
}

pub fn encode_instruction(instruction: &Instruction) -> [u16; 2] {
    match instruction {
        Instruction::Halt(_) => encode_null_instruction(0x00),
        Instruction::Set(data) => encode_reg_val_instruction(0x01, data.register, data.value),
        Instruction::Copy(data) => {
            encode_reg_reg_instruction(0x02, data.src_register, data.dest_register)
        }
        Instruction::Add(data) => {
            encode_reg_reg_instruction(0x03, data.src_register, data.dest_register)
        }
        Instruction::IsLessThan(data) => {
            encode_reg_reg_instruction(0x09, data.src_register, data.dest_register)
        }
        Instruction::IsGreaterThan(data) => {
            encode_reg_reg_instruction(0x0A, data.src_register, data.dest_register)
        }
        Instruction::JumpIf(data) => encode_val_instruction(0x0E, data.new_pc),
        _ => panic!(
            "Fatal: Instruction decoder not implemented for: {:#?}",
            instruction
        ),
    }
}

pub fn fetch_instruction(mem: &MemoryPeripheral, pc: u16) -> [u16; 2] {
    // TODO: Alignment check?
    // TODO: Do we need to copy here?
    let upper = mem.read_address(pc).to_owned();
    let lower = mem.read_address(pc + 1).to_owned();
    [upper, lower]
}
