// Instruction (32 bit)
//
// Instruction formats:
//
// Implied: (e.g. HALT)
// 6 bit instruction identifier (max 64 instructions)
// 22 bit reserved
// 4 bit condition flags
//
// Immediate: (e.g. BRAN #-3)
// 6 bit instruction identifier (max 64 instructions)
// 4 bit register identifier
// 16 bit value
// 2 bit reserved
// 4 bit conditions flags
//
// Register: (e.g. ADD r1, r2)
// 6 bit instruction identifier (max 64 instructions)
// 4 bit register identifier
// 4 bit register identifier
// 4 bit register identifier (if any)
// 8 bit args
// 4 bit condition flags
//
// Segment 0x00 is reserved by the CPU for parameters.
// The other segments are flexible because they are defined in this hardcoded segment.
//
// 0x00 0000 : DW Initial PC
// 0x00 0002 : DW System SP
// 0x00 0004 : DW Base System RAM (for storing in interrupt vectors etc.)
// ...

use crate::executors::Executor;
use crate::registers::{sr_bit_is_set, Registers, StatusRegisterFields};
use enum_dispatch::enum_dispatch;
use peripheral_mem::MemoryPeripheral;

// 32 bits = 2x 16 bit
pub const INSTRUCTION_SIZE_WORDS: u32 = 2;
pub const INSTRUCTION_SIZE_BYTES: u32 = INSTRUCTION_SIZE_WORDS * 2;

// Condition Flags

#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum ConditionFlags {
    Always = 0b000,
    Equal,
    NotEqual,
    CarrySet,
    CarryClear,
    NegativeSet,
    NegativeClear,
    OverflowSet,
    OverflowClear,
    UnsignedHigher,
    UnsignedLowerOrSame,
    GreaterOrEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    Never = 0b1111,
}

// TODO: Define trait and move somewhere else?
impl ConditionFlags {
    pub fn should_execute(&self, registers: &Registers) -> bool {
        match self {
            ConditionFlags::Always => true,
            ConditionFlags::Equal => sr_bit_is_set(StatusRegisterFields::Zero, registers),
            ConditionFlags::NotEqual => !sr_bit_is_set(StatusRegisterFields::Zero, registers),
            ConditionFlags::CarrySet => sr_bit_is_set(StatusRegisterFields::Carry, registers),
            ConditionFlags::CarryClear => !sr_bit_is_set(StatusRegisterFields::Carry, registers),
            ConditionFlags::NegativeSet => sr_bit_is_set(StatusRegisterFields::Negative, registers),
            ConditionFlags::NegativeClear => {
                !sr_bit_is_set(StatusRegisterFields::Negative, registers)
            }
            ConditionFlags::OverflowSet => sr_bit_is_set(StatusRegisterFields::Overflow, registers),
            ConditionFlags::OverflowClear => {
                !sr_bit_is_set(StatusRegisterFields::Overflow, registers)
            }
            ConditionFlags::UnsignedHigher => {
                sr_bit_is_set(StatusRegisterFields::Carry, registers)
                    && !sr_bit_is_set(StatusRegisterFields::Zero, registers)
            }
            ConditionFlags::UnsignedLowerOrSame => {
                !sr_bit_is_set(StatusRegisterFields::Carry, registers)
                    || sr_bit_is_set(StatusRegisterFields::Zero, registers)
            }
            ConditionFlags::GreaterOrEqual => {
                sr_bit_is_set(StatusRegisterFields::Negative, registers)
                    == sr_bit_is_set(StatusRegisterFields::Overflow, registers)
            }
            ConditionFlags::LessThan => {
                sr_bit_is_set(StatusRegisterFields::Negative, registers)
                    != sr_bit_is_set(StatusRegisterFields::Overflow, registers)
            }
            ConditionFlags::GreaterThan => {
                !sr_bit_is_set(StatusRegisterFields::Zero, registers)
                    && (sr_bit_is_set(StatusRegisterFields::Negative, registers)
                        == sr_bit_is_set(StatusRegisterFields::Overflow, registers))
            }
            ConditionFlags::LessThanOrEqual => {
                sr_bit_is_set(StatusRegisterFields::Zero, registers)
                    || (sr_bit_is_set(StatusRegisterFields::Negative, registers)
                        != sr_bit_is_set(StatusRegisterFields::Overflow, registers))
            }
            ConditionFlags::Never => false,
        }
    }
}

// Instruction Types

#[derive(Debug, PartialEq, Eq)]
pub struct ImpliedInstructionData {
    pub condition_flag: ConditionFlags,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImmediateInstructionData {
    pub register: u8,
    pub value: u16,
    pub condition_flag: ConditionFlags,
    // Max 2 bits (& 0x0003)
    pub additional_flags: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RegisterInstructionData {
    pub r1: u8,
    pub r2: u8,
    pub r3: u8,
    pub condition_flag: ConditionFlags,
    // Max 10 bits (& 0x03FF)
    pub additional_flags: u8,
}

// Implied Argument Instructions

#[derive(Debug)]
pub struct HaltInstructionData {
    // ID: 0x00
    pub data: ImpliedInstructionData,
}

#[derive(Debug)]
pub struct NoOperationInstructionData {
    // ID: 0x01
    pub data: ImpliedInstructionData,
}

#[derive(Debug)]
pub struct WaitForInterruptInstructionData {
    // ID: 0x02
    pub data: ImpliedInstructionData,
}

#[derive(Debug)]
pub struct ReturnFromInterruptData {
    // ID: 0x03
    pub data: ImpliedInstructionData,
}

#[derive(Debug)]
pub struct ReturnFromSubroutineData {
    // ID: 0x04
    pub data: ImpliedInstructionData,
}

// Arithmetic

#[derive(Debug)]
pub struct AddInstructionData {
    // ID: 0x05
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct AddWithCarryInstructionData {
    // ID: 0x06
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct SubtractInstructionData {
    // ID: 0x07
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct SubtractWithCarryInstructionData {
    // ID: 0x08
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct MultiplyInstructionData {
    // ID: 0x09
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct DivideInstructionData {
    // ID: 0x0A
    pub data: RegisterInstructionData,
}

// Logic

#[derive(Debug)]
pub struct AndInstructionData {
    // ID: 0x0B
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct OrInstructionData {
    // ID: 0x0C
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct XorInstructionData {
    // ID: 0x0D
    pub data: RegisterInstructionData,
}

// Shifts

#[derive(Debug)]
pub struct LogicalShiftLeftInstructionData {
    // ID: 0x0E
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct LogicalShiftRightInstructionData {
    // ID: 0x0F
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct ArithmeticShiftLeftInstructionData {
    // ID: 0x10
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct ArithmeticShiftRightInstructionData {
    // ID: 0x11
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct RotateLeftInstructionData {
    // ID: 0x12
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct RotateRightInstructionData {
    // ID: 0x13
    pub data: RegisterInstructionData,
}

// Comparison

#[derive(Debug)]
pub struct CompareInstructionData {
    // ID: 0x14
    pub data: RegisterInstructionData,
}

// Stack Manipulation

#[derive(Debug)]
pub struct PushInstructionData {
    // ID: 0x15
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct PopInstructionData {
    // ID: 0x16
    pub data: RegisterInstructionData,
}

// Flow Control

#[derive(Debug)]
pub struct TriggerSoftwareInterruptData {
    // ID: 0x17
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct ShortJumpWithImmediateData {
    // ID: 0x18
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct ShortJumpToSubroutineWithImmediateData {
    // ID: 0x19
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct BranchToSubroutineData {
    // ID: 0x1A
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct BranchInstructionData {
    // ID: 0x1B
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct LongJumpWithAddressRegisterData {
    // ID: 0x1C
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct LongJumpToSubroutineWithAddressRegisterData {
    // ID: 0x1D
    pub data: RegisterInstructionData,
}

// Data Access

// TODO: remove gap here (opcode 0x1E was LEA immediate which didn't make sense and was removed)
// Haven't got around to rejigging the opcodes

#[derive(Debug)]
pub struct LoadEffectiveAddressFromIndirectImmediateData {
    // ID: 0x1F
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct LoadEffectiveAddressFromIndirectRegisterData {
    // ID: 0x20
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct LoadManyRegisterFromAddressRegisterData {
    // ID: 0x21
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct LoadRegisterFromImmediateData {
    // ID: 0x22
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct LoadRegisterFromRegisterData {
    // ID: 0x23
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct LoadRegisterFromIndirectImmediateData {
    // ID: 0x24
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct LoadRegisterFromIndirectRegisterData {
    // ID: 0x25
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct StoreRegisterToIndirectImmediateData {
    // ID: 0x26
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct StoreRegisterToIndirectRegisterData {
    // ID: 0x27
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct StoreManyRegisterFromAddressRegisterData {
    // ID: 0x28
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
#[enum_dispatch(Executor)]
pub enum Instruction {
    // Implied Argument Instructions
    Halt(HaltInstructionData),
    NoOperation(NoOperationInstructionData),
    WaitForInterrupt(WaitForInterruptInstructionData),
    ReturnFromInterrupt(ReturnFromInterruptData),
    ReturnFromSubroutine(ReturnFromSubroutineData),
    // Arithmetic
    Add(AddInstructionData),
    AddWithCarry(AddWithCarryInstructionData),
    Subtract(SubtractInstructionData),
    SubtractWithCarry(SubtractWithCarryInstructionData),
    Multiply(MultiplyInstructionData),
    Divide(DivideInstructionData),
    // Logic
    And(AndInstructionData),
    Or(OrInstructionData),
    Xor(XorInstructionData),
    // Shifts
    LogicalShiftLeft(LogicalShiftLeftInstructionData),
    LogicalShiftRight(LogicalShiftRightInstructionData),
    ArithmeticShiftLeft(ArithmeticShiftLeftInstructionData),
    ArithmeticShiftRight(ArithmeticShiftRightInstructionData),
    RotateLeft(RotateLeftInstructionData),
    RotateRight(RotateRightInstructionData),
    // Comparison
    Compare(CompareInstructionData),
    // Stack Manipulation
    Push(PushInstructionData),
    Pop(PopInstructionData),
    // Flow Control
    TriggerSoftwareInterrupt(TriggerSoftwareInterruptData),
    ShortJumpWithImmediate(ShortJumpWithImmediateData),
    ShortJumpToSubroutineWithImmediate(ShortJumpToSubroutineWithImmediateData),
    BranchToSubroutine(BranchToSubroutineData),
    Branch(BranchInstructionData),
    LongJumpWithAddressRegister(LongJumpWithAddressRegisterData),
    LongJumpToSubroutineWithAddressRegister(LongJumpToSubroutineWithAddressRegisterData),
    // Data Access
    LoadEffectiveAddressIndirectImmediate(LoadEffectiveAddressFromIndirectImmediateData),
    LoadEffectiveAddressIndirectRegister(LoadEffectiveAddressFromIndirectRegisterData),
    LoadManyRegisterFromAddressRegister(LoadManyRegisterFromAddressRegisterData),
    LoadRegisterFromImmediate(LoadRegisterFromImmediateData),
    LoadRegisterFromRegister(LoadRegisterFromRegisterData),
    LoadRegisterFromIndirectImmediate(LoadRegisterFromIndirectImmediateData),
    LoadRegisterFromIndirectRegister(LoadRegisterFromIndirectRegisterData),
    StoreRegisterToIndirectImmediate(StoreRegisterToIndirectImmediateData),
    StoreRegisterToIndirectRegister(StoreRegisterToIndirectRegisterData),
    StoreManyRegisterFromAddressRegister(StoreManyRegisterFromAddressRegisterData),
}

pub fn get_clocks_for_instruction(_instruction: &Instruction) -> u32 {
    // Important!: Does not include instruction fetch time (4 cycles)
    //
    // Educated guess based on 6502 instruction set
    // (https://www.masswerk.at/6502/6502_instruction_set.html)
    // Hardware doesn't exist yet so subject to change
    // TODO: Move to executors where there is more context to calculate cycles?
    // TODO: Double check all these!
    0 // TODO
}

// Pending Instructions
// Throw privilege error if try to write to SR etc.
// Immediate versions of ALU instructions? (and shifting/rotate)
