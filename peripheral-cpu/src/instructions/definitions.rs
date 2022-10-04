// Instruction (32 bit)
// Every instruction starts with:
// 8 bit instruction identifier (max 256 instructions)
//
// Instruction formats:
//
// Immediate: (e.g. SET)
// 8 bit register identifier
// 16 bit value
//
// Register: (e.g. COPY)
// 8 bit register identifier
// 8 bit register identifier
// 8 bit padding
//
// Address: (e.g. JUMP)
// 24 bit value

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
