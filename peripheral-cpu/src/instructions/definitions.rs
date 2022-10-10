// Instruction (32 bit)
//
// Instruction formats:
//
// Immediate: (e.g. SET)
// 6 bit instruction identifier (max 64 instructions)
// 4 bit register identifier
// 16 bit value
// 6 bit arguments (if any)
//
// Register: (e.g. COPY)
// 6 bit instruction identifier (max 64 instructions)
// 4 bit register identifier
// 4 bit register identifier
// 4 bit register identifier (if any)
// 14 bit arguments (if any)
//
// Address: (e.g. JUMP)
// 6 bit instruction identifier (max 64 instructions)
// 24 bit value
// 2 bit arguments (if any)

use crate::executors::Executor;
use crate::registers::Registers;
use enum_dispatch::enum_dispatch;
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
    pub r1: u8,
    pub r2: u8,
    pub r3: u8,
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
pub struct SetAddressInstructionData {
    pub data: AddressInstructionData,
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
pub struct LoadOffsetRegisterData {
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct StoreOffsetRegisterData {
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct LoadOffsetImmediateData {
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct StoreOffsetImmediateData {
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
#[enum_dispatch(Executor)]
pub enum Instruction {
    // Special
    Halt(NullInstructionData),
    // Register Transfer
    Set(SetInstructionData),
    SetAddress(SetAddressInstructionData),
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
    // Data Access
    LoadOffsetRegister(LoadOffsetRegisterData),
    StoreOffsetRegister(StoreOffsetRegisterData),
    LoadOffsetImmediate(LoadOffsetImmediateData),
    StoreOffsetImmediate(StoreOffsetImmediateData),
}
