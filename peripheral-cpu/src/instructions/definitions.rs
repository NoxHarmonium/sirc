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

// Segment 0x00 is reserved by the CPU for parameters.
// The other segments are flexible because they are defined in this hardcoded segment.
//
// 0x00 0000 : DW Initial PC
// 0x00 0002 : DW System SP
// 0x00 0004 : DW Base System RAM (for storing in interrupt vectors etc.)
// ...

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
pub struct WaitForInterruptInstructionData {
    pub data: NullInstructionData,
}

#[derive(Debug)]
pub struct ReturnFromInterruptData {
    pub data: NullInstructionData,
}

#[derive(Debug)]
pub struct TriggerSoftwareInterruptData {
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct DisableInterruptsData {
    pub data: NullInstructionData,
}

#[derive(Debug)]
pub struct EnableInterruptsData {
    pub data: NullInstructionData,
}

#[derive(Debug)]
pub struct JumpToSubroutineData {
    pub data: AddressInstructionData,
}

#[derive(Debug)]
pub struct ReturnFromSubroutineData {
    pub data: NullInstructionData,
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
    // Interrupts
    WaitForInterrupt(WaitForInterruptInstructionData),
    ReturnFromInterrupt(ReturnFromInterruptData),
    TriggerSoftwareInterrupt(TriggerSoftwareInterruptData),
    DisableInterrupts(DisableInterruptsData),
    EnableInterrupts(EnableInterruptsData),
    // Subroutines
    JumpToSubroutine(JumpToSubroutineData),
    ReturnFromSubroutine(ReturnFromSubroutineData),
}

pub fn get_clocks_for_instruction(instruction: &Instruction) -> u32 {
    // Educated guess based on 6502 instruction set
    // (https://www.masswerk.at/6502/6502_instruction_set.html)
    // Hardware doesn't exist yet so subject to change
    match instruction {
        Instruction::Halt(_) => 2,
        Instruction::Set(_) => 2,
        Instruction::SetAddress(_) => 2,
        Instruction::Copy(_) => 2,
        Instruction::Add(_) => 2,
        Instruction::Subtract(_) => 2,
        // 70 is worst case for the 68k - maybe in the future it could be dynamic based on input
        // See: https://retrocomputing.stackexchange.com/a/7670
        Instruction::Multiply(_) => 70,
        // Worst case: Signed 156 / Unsigned 136
        // See: https://www.atari-forum.com/viewtopic.php?t=6484
        Instruction::Divide(_) => 156,
        Instruction::IsEqual(_) => 2,
        Instruction::IsNotEqual(_) => 2,
        Instruction::IsLessThan(_) => 2,
        Instruction::IsGreaterThan(_) => 2,
        Instruction::IsLessOrEqualThan(_) => 2,
        Instruction::IsGreaterOrEqualThan(_) => 2,
        Instruction::Jump(_) => 3,
        Instruction::JumpIf(_) => 4,
        Instruction::JumpIfNot(_) => 4,
        Instruction::LoadOffsetRegister(_) => 4,
        Instruction::StoreOffsetRegister(_) => 4,
        Instruction::LoadOffsetImmediate(_) => 4,
        Instruction::StoreOffsetImmediate(_) => 4,
        Instruction::WaitForInterrupt(_) => 1,
        Instruction::ReturnFromInterrupt(_) => 12,
        Instruction::TriggerSoftwareInterrupt(_) => 1,
        Instruction::DisableInterrupts(_) => 2,
        Instruction::EnableInterrupts(_) => 2,
        Instruction::JumpToSubroutine(_) => 12,
        Instruction::ReturnFromSubroutine(_) => 12,
    }
}

// Pending Instructions
// RNGS - Seed a register with an RNG seed (or just a random value but is slow)
// SHFL/SHFR - Bit Shift
// ROTL/ROTR - Bit Rotate
// NOP?
// Clear specific carry/overflow flags? (do that automatically at next ALU operation?)
// Jumping based on carry/overflow? How that work?
// Reset interrupt
