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
pub const INSTRUCTION_SIZE_WORDS: u32 = 2;
pub const INSTRUCTION_SIZE_BYTES: u32 = INSTRUCTION_SIZE_WORDS * 2;

// Instruction Types

#[derive(Debug)]
pub struct ImmediateInstructionData {
    pub register: u8,
    pub value: u16,
}

#[derive(Debug)]
pub struct RegisterInstructionData {
    pub src_register: u8,
    pub dest_register: u8,
}

#[derive(Debug)]
pub struct AddressInstructionData {
    pub address: u32,
}

// Special

#[derive(Debug)]
pub struct NullInstructionData {}

// Register Transfer
#[derive(Debug)]
pub struct SetInstructionData {
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct CopyInstructionData {
    pub data: RegisterInstructionData,
}

// Arithmetic

#[derive(Debug)]
pub struct AddInstructionData {
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct SubtractInstructionData {
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct MultiplyInstructionData {
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct DivideInstructionData {
    pub data: RegisterInstructionData,
}

// Comparison

#[derive(Debug)]
pub struct IsEqualInstructionData {
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct IsNotEqualInstructionData {
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct IsLessThanInstructionData {
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct IsGreaterThanInstructionData {
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct IsLessOrEqualThanInstructionData {
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct IsGreaterOrEqualThanInstructionData {
    pub data: RegisterInstructionData,
}

// Flow Control

#[derive(Debug)]
pub struct JumpInstructionData {
    pub data: AddressInstructionData,
}

#[derive(Debug)]
pub struct JumpIfInstructionData {
    pub data: AddressInstructionData,
}

#[derive(Debug)]
pub struct JumpIfNotInstructionData {
    pub data: AddressInstructionData,
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

pub fn decode_reg_val_instruction(raw_instruction: [u8; 4]) -> (u8, u16) {
    let [_, r1, v1, v2] = raw_instruction;
    (r1, u16::from_be_bytes([v1, v2]))
}

pub fn decode_reg_reg_instruction(raw_instruction: [u8; 4]) -> (u8, u8) {
    let [_, r1, r2, _] = raw_instruction;
    (r1, r2)
}

pub fn decode_val_instruction(raw_instruction: [u8; 4]) -> u32 {
    let [_, b1, b2, b3] = raw_instruction;
    u32::from_be_bytes([0x00, b1, b2, b3])
}

// Since it is a "16-bit" processor, we read/write 16 bits at a time (align on 16 bits)
pub fn decode_instruction(raw_instruction: [u8; 4]) -> Instruction {
    let instruction_id = raw_instruction[0];

    match instruction_id {
        0x00 => Instruction::Halt(NullInstructionData {}),
        0x01 => {
            let (register, value) = decode_reg_val_instruction(raw_instruction);
            Instruction::Set(SetInstructionData {
                data: ImmediateInstructionData { register, value },
            })
        }
        0x02 => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::Copy(CopyInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x03 => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::Add(AddInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x04 => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::Subtract(SubtractInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x05 => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::Multiply(MultiplyInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x06 => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::Divide(DivideInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x07 => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::IsEqual(IsEqualInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x08 => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::IsNotEqual(IsNotEqualInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x09 => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::IsLessThan(IsLessThanInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x0A => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::IsGreaterThan(IsGreaterThanInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x0B => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::IsLessOrEqualThan(IsLessOrEqualThanInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x0C => {
            let (src_register, dest_register) = decode_reg_reg_instruction(raw_instruction);
            Instruction::IsGreaterOrEqualThan(IsGreaterOrEqualThanInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x0D => {
            let address = decode_val_instruction(raw_instruction);
            Instruction::Jump(JumpInstructionData {
                data: AddressInstructionData { address },
            })
        }
        0x0E => {
            let address = decode_val_instruction(raw_instruction);
            Instruction::JumpIf(JumpIfInstructionData {
                data: AddressInstructionData { address },
            })
        }
        0x0F => {
            let address = decode_val_instruction(raw_instruction);
            Instruction::JumpIfNot(JumpIfNotInstructionData {
                data: AddressInstructionData { address },
            })
        }
        _ => panic!("Fatal: Invalid instruction ID: 0x{:02x}", instruction_id),
    }
}

// Encode

pub fn encode_null_instruction(instruction_id: u8) -> [u8; 4] {
    [instruction_id, 0x0, 0x0, 0x0]
}

pub fn encode_reg_val_instruction(instruction_id: u8, register: u8, value: u16) -> [u8; 4] {
    let [b3, b4] = u16::to_be_bytes(value);

    [instruction_id, register, b3, b4]
}

pub fn encode_reg_reg_instruction(
    instruction_id: u8,
    src_register: u8,
    dest_register: u8,
) -> [u8; 4] {
    [instruction_id, src_register, dest_register, 0x0]
}

pub fn encode_val_instruction(instruction_id: u8, value: u32) -> [u8; 4] {
    let [_, b1, b2, b3] = u32::to_be_bytes(value);
    [instruction_id, b1, b2, b3]
}

pub fn encode_instruction(instruction: &Instruction) -> [u8; 4] {
    match instruction {
        Instruction::Halt(_) => encode_null_instruction(0x00),
        Instruction::Set(data) => {
            encode_reg_val_instruction(0x01, data.data.register, data.data.value)
        }
        Instruction::Copy(data) => {
            encode_reg_reg_instruction(0x02, data.data.src_register, data.data.dest_register)
        }
        Instruction::Add(data) => {
            encode_reg_reg_instruction(0x03, data.data.src_register, data.data.dest_register)
        }
        Instruction::Subtract(data) => {
            encode_reg_reg_instruction(0x04, data.data.src_register, data.data.dest_register)
        }
        Instruction::Multiply(data) => {
            encode_reg_reg_instruction(0x05, data.data.src_register, data.data.dest_register)
        }
        Instruction::Divide(data) => {
            encode_reg_reg_instruction(0x06, data.data.src_register, data.data.dest_register)
        }
        Instruction::IsEqual(data) => {
            encode_reg_reg_instruction(0x07, data.data.src_register, data.data.dest_register)
        }
        Instruction::IsNotEqual(data) => {
            encode_reg_reg_instruction(0x08, data.data.src_register, data.data.dest_register)
        }
        Instruction::IsLessThan(data) => {
            encode_reg_reg_instruction(0x09, data.data.src_register, data.data.dest_register)
        }
        Instruction::IsGreaterThan(data) => {
            encode_reg_reg_instruction(0x0A, data.data.src_register, data.data.dest_register)
        }
        Instruction::IsLessOrEqualThan(data) => {
            encode_reg_reg_instruction(0x0B, data.data.src_register, data.data.dest_register)
        }
        Instruction::IsGreaterOrEqualThan(data) => {
            encode_reg_reg_instruction(0x0C, data.data.src_register, data.data.dest_register)
        }
        Instruction::Jump(data) => encode_val_instruction(0x0D, data.data.address),
        Instruction::JumpIf(data) => encode_val_instruction(0x0E, data.data.address),
        Instruction::JumpIfNot(data) => encode_val_instruction(0x0F, data.data.address),
    }
}

pub fn fetch_instruction(mem: &MemoryPeripheral, pc: u32) -> [u8; 4] {
    // TODO: Alignment check?
    // TODO: Do we need to copy here?
    let [b1, b2] = u16::to_be_bytes(mem.read_address(pc).to_owned());
    let [b3, b4] = u16::to_be_bytes(mem.read_address(pc + 1).to_owned());
    [b2, b1, b4, b3]
}
