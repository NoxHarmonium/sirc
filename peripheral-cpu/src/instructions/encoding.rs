// Decode

use super::definitions::*;

pub fn decode_immediate_instruction(raw_instruction: [u8; 4]) -> (u8, u16) {
    let [_, r1, v1, v2] = raw_instruction;
    (r1, u16::from_be_bytes([v1, v2]))
}

pub fn decode_register_instruction(raw_instruction: [u8; 4]) -> (u8, u8) {
    let [_, r1, r2, _] = raw_instruction;
    (r1, r2)
}

pub fn decode_address_instruction(raw_instruction: [u8; 4]) -> u32 {
    let [_, b1, b2, b3] = raw_instruction;
    u32::from_be_bytes([0x00, b1, b2, b3])
}

// Since it is a "16-bit" processor, we read/write 16 bits at a time (align on 16 bits)
pub fn decode_instruction(raw_instruction: [u8; 4]) -> Instruction {
    let instruction_id = raw_instruction[0];

    match instruction_id {
        0x00 => Instruction::Halt(NullInstructionData {}),
        0x01 => {
            let (register, value) = decode_immediate_instruction(raw_instruction);
            Instruction::Set(SetInstructionData {
                data: ImmediateInstructionData { register, value },
            })
        }
        0x02 => {
            let (src_register, dest_register) = decode_register_instruction(raw_instruction);
            Instruction::Copy(CopyInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x03 => {
            let (src_register, dest_register) = decode_register_instruction(raw_instruction);
            Instruction::Add(AddInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x04 => {
            let (src_register, dest_register) = decode_register_instruction(raw_instruction);
            Instruction::Subtract(SubtractInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x05 => {
            let (src_register, dest_register) = decode_register_instruction(raw_instruction);
            Instruction::Multiply(MultiplyInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x06 => {
            let (src_register, dest_register) = decode_register_instruction(raw_instruction);
            Instruction::Divide(DivideInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x07 => {
            let (src_register, dest_register) = decode_register_instruction(raw_instruction);
            Instruction::IsEqual(IsEqualInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x08 => {
            let (src_register, dest_register) = decode_register_instruction(raw_instruction);
            Instruction::IsNotEqual(IsNotEqualInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x09 => {
            let (src_register, dest_register) = decode_register_instruction(raw_instruction);
            Instruction::IsLessThan(IsLessThanInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x0A => {
            let (src_register, dest_register) = decode_register_instruction(raw_instruction);
            Instruction::IsGreaterThan(IsGreaterThanInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x0B => {
            let (src_register, dest_register) = decode_register_instruction(raw_instruction);
            Instruction::IsLessOrEqualThan(IsLessOrEqualThanInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x0C => {
            let (src_register, dest_register) = decode_register_instruction(raw_instruction);
            Instruction::IsGreaterOrEqualThan(IsGreaterOrEqualThanInstructionData {
                data: RegisterInstructionData {
                    src_register,
                    dest_register,
                },
            })
        }
        0x0D => {
            let address = decode_address_instruction(raw_instruction);
            Instruction::Jump(JumpInstructionData {
                data: AddressInstructionData { address },
            })
        }
        0x0E => {
            let address = decode_address_instruction(raw_instruction);
            Instruction::JumpIf(JumpIfInstructionData {
                data: AddressInstructionData { address },
            })
        }
        0x0F => {
            let address = decode_address_instruction(raw_instruction);
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

pub fn encode_immediate_instruction(instruction_id: u8, register: u8, value: u16) -> [u8; 4] {
    let [b3, b4] = u16::to_be_bytes(value);

    [instruction_id, register, b3, b4]
}

pub fn encode_register_instruction(
    instruction_id: u8,
    src_register: u8,
    dest_register: u8,
) -> [u8; 4] {
    [instruction_id, src_register, dest_register, 0x0]
}

pub fn encode_address_instruction(instruction_id: u8, value: u32) -> [u8; 4] {
    let [_, b1, b2, b3] = u32::to_be_bytes(value);
    [instruction_id, b1, b2, b3]
}

pub fn encode_instruction(instruction: &Instruction) -> [u8; 4] {
    match instruction {
        Instruction::Halt(_) => encode_null_instruction(0x00),
        Instruction::Set(data) => {
            encode_immediate_instruction(0x01, data.data.register, data.data.value)
        }
        Instruction::Copy(data) => {
            encode_register_instruction(0x02, data.data.src_register, data.data.dest_register)
        }
        Instruction::Add(data) => {
            encode_register_instruction(0x03, data.data.src_register, data.data.dest_register)
        }
        Instruction::Subtract(data) => {
            encode_register_instruction(0x04, data.data.src_register, data.data.dest_register)
        }
        Instruction::Multiply(data) => {
            encode_register_instruction(0x05, data.data.src_register, data.data.dest_register)
        }
        Instruction::Divide(data) => {
            encode_register_instruction(0x06, data.data.src_register, data.data.dest_register)
        }
        Instruction::IsEqual(data) => {
            encode_register_instruction(0x07, data.data.src_register, data.data.dest_register)
        }
        Instruction::IsNotEqual(data) => {
            encode_register_instruction(0x08, data.data.src_register, data.data.dest_register)
        }
        Instruction::IsLessThan(data) => {
            encode_register_instruction(0x09, data.data.src_register, data.data.dest_register)
        }
        Instruction::IsGreaterThan(data) => {
            encode_register_instruction(0x0A, data.data.src_register, data.data.dest_register)
        }
        Instruction::IsLessOrEqualThan(data) => {
            encode_register_instruction(0x0B, data.data.src_register, data.data.dest_register)
        }
        Instruction::IsGreaterOrEqualThan(data) => {
            encode_register_instruction(0x0C, data.data.src_register, data.data.dest_register)
        }
        Instruction::Jump(data) => encode_address_instruction(0x0D, data.data.address),
        Instruction::JumpIf(data) => encode_address_instruction(0x0E, data.data.address),
        Instruction::JumpIfNot(data) => encode_address_instruction(0x0F, data.data.address),
    }
}
