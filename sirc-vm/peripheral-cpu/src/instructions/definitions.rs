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
// 2 bit address register a, p or s (if any)
// 4 bit conditions flags
//
// Short Immediate (with shift): (e.g. ADDI r1, #2, ASL #1)
// 6 bit instruction identifier (max 64 instructions)
// 4 bit register identifier
// 8 bit value
// 8 bit shift
// 2 bit address register a, p or s (if any)
// 4 bit conditions flags
//
// Register: (e.g. ADD r1, r2)
// 6 bit instruction identifier (max 64 instructions)
// 4 bit register identifier
// 4 bit register identifier
// 4 bit register identifier (if any)
// 8 bit args
// 2 bit address register a, p or s (if any)
// 4 bit condition flags
//
// Segment 0x00 is reserved by the CPU for parameters.
// The other segments are flexible because they are defined in this hardcoded segment.
//
// 0x00 0000 : DW Initial PC
// 0x00 0002 : DW System SP
// 0x00 0004 : DW Base System RAM (for storing in interrupt vectors etc.)
// ...

use crate::registers::{sr_bit_is_set, Registers, StatusRegisterFields};

// 32 bits = 2x 16 bit
pub const INSTRUCTION_SIZE_WORDS: u32 = 2;
pub const INSTRUCTION_SIZE_BYTES: u32 = INSTRUCTION_SIZE_WORDS * 2;

pub const MAX_SHIFT_COUNT: u16 = 15; // 4 bits (the size of the shift count in the instruction)

// Condition Flags

#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, Eq, Clone, Copy, Default)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum ConditionFlags {
    #[default]
    Always = 0b0000,
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

#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, Eq, Clone, Copy, Default)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum ShiftType {
    #[default]
    None = 0x0,
    LogicalLeftShift,
    LogicalRightShift,
    ArithmeticLeftShift,
    ArithmeticRightShift,
    RotateLeft,
    RotateRight,
    Reserved,
}

#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, Eq, Clone, Copy, Default)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum ShiftOperand {
    #[default]
    Immediate = 0b0,
    Register = 0b1,
}

// At the moment hardcoded to Alu for all ALU instructions except for SHFI/SHFR
// In the future it can be leveraged to disable status register updates for ALU
// instructions to be able to chain multiple instructions off the one CMP
// It is supported by the decoded, I just need to work out a syntax for the assembler
#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, Eq, Clone, Copy, Default)]
pub enum StatusRegisterUpdateSource {
    #[default]
    None = 0b00,
    Alu = 0b01,
    Shift = 0b10,
    Reserved = 0b11,
}

impl StatusRegisterUpdateSource {
    #[must_use]
    pub fn to_flags(&self) -> u8 {
        num::ToPrimitive::to_u8(self).expect("Could not convert enum to u8") & 0x3
    }
}

// TODO: Define trait and move somewhere else?
impl ConditionFlags {
    #[must_use]
    pub fn should_execute(&self, registers: &Registers) -> bool {
        match self {
            Self::Always => true,
            Self::Equal => sr_bit_is_set(StatusRegisterFields::Zero, registers),
            Self::NotEqual => !sr_bit_is_set(StatusRegisterFields::Zero, registers),
            Self::CarrySet => sr_bit_is_set(StatusRegisterFields::Carry, registers),
            Self::CarryClear => !sr_bit_is_set(StatusRegisterFields::Carry, registers),
            Self::NegativeSet => sr_bit_is_set(StatusRegisterFields::Negative, registers),
            Self::NegativeClear => !sr_bit_is_set(StatusRegisterFields::Negative, registers),
            Self::OverflowSet => sr_bit_is_set(StatusRegisterFields::Overflow, registers),
            Self::OverflowClear => !sr_bit_is_set(StatusRegisterFields::Overflow, registers),
            Self::UnsignedHigher => {
                sr_bit_is_set(StatusRegisterFields::Carry, registers)
                    && !sr_bit_is_set(StatusRegisterFields::Zero, registers)
            }
            Self::UnsignedLowerOrSame => {
                !sr_bit_is_set(StatusRegisterFields::Carry, registers)
                    || sr_bit_is_set(StatusRegisterFields::Zero, registers)
            }
            Self::GreaterOrEqual => {
                sr_bit_is_set(StatusRegisterFields::Negative, registers)
                    == sr_bit_is_set(StatusRegisterFields::Overflow, registers)
            }
            Self::LessThan => {
                sr_bit_is_set(StatusRegisterFields::Negative, registers)
                    != sr_bit_is_set(StatusRegisterFields::Overflow, registers)
            }
            Self::GreaterThan => {
                !sr_bit_is_set(StatusRegisterFields::Zero, registers)
                    && (sr_bit_is_set(StatusRegisterFields::Negative, registers)
                        == sr_bit_is_set(StatusRegisterFields::Overflow, registers))
            }
            Self::LessThanOrEqual => {
                sr_bit_is_set(StatusRegisterFields::Zero, registers)
                    || (sr_bit_is_set(StatusRegisterFields::Negative, registers)
                        != sr_bit_is_set(StatusRegisterFields::Overflow, registers))
            }
            Self::Never => false,
        }
    }
}

// Instruction Types

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ImpliedInstructionData {
    pub op_code: Instruction,
    // TODO: Do we need anything more than DecodedInstruction
    // for these *InstructionData structs?
    pub condition_flag: ConditionFlags,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ImmediateInstructionData {
    pub op_code: Instruction,
    pub register: u8,
    pub value: u16,
    pub condition_flag: ConditionFlags,
    // Max 2 bits (& 0x0003)
    pub additional_flags: u8,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ShortImmediateInstructionData {
    pub op_code: Instruction,
    pub register: u8,
    pub value: u8,
    pub shift_operand: ShiftOperand,
    pub shift_type: ShiftType,
    pub shift_count: u8,
    pub condition_flag: ConditionFlags,
    // Max 2 bits (& 0x0003)
    pub additional_flags: u8,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RegisterInstructionData {
    pub op_code: Instruction,
    pub r1: u8,
    pub r2: u8,
    pub r3: u8,
    pub shift_operand: ShiftOperand,
    pub shift_type: ShiftType,
    pub shift_count: u8,
    pub condition_flag: ConditionFlags,
    // Max 2 bits (& 0x0003)
    pub additional_flags: u8,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InstructionData {
    Implied(ImpliedInstructionData),
    Immediate(ImmediateInstructionData),
    ShortImmediate(ShortImmediateInstructionData),
    Register(RegisterInstructionData),
}

// TODO: Rename to OpCode or something?
#[derive(Debug, PartialEq, Eq, FromPrimitive, ToPrimitive, Default, Clone, Hash, Copy)]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum Instruction {
    // Arithmetic (Immediate)
    AddImmediate = 0x00,
    AddImmediateWithCarry = 0x01,
    SubtractImmediate = 0x02,
    SubtractImmediateWithCarry = 0x03,
    // Logic (Immediate)
    AndImmediate = 0x04,
    OrImmediate = 0x05,
    XorImmediate = 0x06,
    // Comparison (Immediate)
    CompareImmediate = 0x07,
    TestAndImmediate = 0x08,
    TestXorImmediate = 0x09,
    ShiftImmediate = 0x0A,
    // Flow Control
    BranchImmediate = 0x0B,
    BranchToSubroutineImmediate = 0x0C,
    ShortJumpImmediate = 0x0D,
    ShortJumpToSubroutineImmediate = 0x0E,
    Exception = 0x0F,

    // Addressing
    LoadEffectiveAddressFromIndirectImmediate = 0x10,
    LoadEffectiveAddressFromIndirectRegister = 0x11,
    LongJumpWithImmediateDisplacement = 0x12,
    LongJumpWithRegisterDisplacement = 0x13,
    LongJumpToSubroutineWithImmediateDisplacement = 0x14,
    LongJumpToSubroutineWithRegisterDisplacement = 0x15,

    // Data Access
    LoadRegisterFromImmediate = 0x16,
    LoadRegisterFromRegister = 0x17,
    LoadRegisterFromIndirectImmediate = 0x18,
    LoadRegisterFromIndirectRegister = 0x19,
    StoreRegisterToIndirectImmediate = 0x1A,
    StoreRegisterToIndirectRegister = 0x1B,
    LoadRegisterFromIndirectRegisterPostIncrement = 0x1D,
    StoreRegisterToIndirectRegisterPreDecrement = 0x1F,

    // Arithmetic (Immediate/SHORT)
    AddShortImmediate = 0x20,
    AddShortImmediateWithCarry = 0x21,
    SubtractShortImmediate = 0x22,
    SubtractShortImmediateWithCarry = 0x23,
    // Logic (Immediate)
    AndShortImmediate = 0x24,
    OrShortImmediate = 0x25,
    XorShortImmediate = 0x26,
    // Comparison (Immediate)
    CompareShortImmediate = 0x27,
    TestAndShortImmediate = 0x28,
    TestXorShortImmediate = 0x29,
    ShiftShortImmediate = 0x2A,
    // Flow Control
    BranchShortImmediate = 0x2B,
    BranchToSubroutineShortImmediate = 0x2C,
    ShortJumpShortImmediate = 0x2D,
    ShortJumpToSubroutineShortImmediate = 0x2E,
    ExceptionShort = 0x2F,

    // Arithmetic (Register)
    AddRegister = 0x30,
    AddRegisterWithCarry = 0x31,
    SubtractRegister = 0x32,
    SubtractRegisterWithCarry = 0x33,
    // Logic (Register)
    AndRegister = 0x34,
    OrRegister = 0x35,
    XorRegister = 0x36,
    // Comparison (Register)
    CompareRegister = 0x37,
    TestAndRegister = 0x38,
    TestXorRegister = 0x39,
    ShiftRegister = 0x3A,

    // Implied
    ReturnFromSubroutine = 0x3B,
    #[default]
    NoOperation = 0x3C,
    // Exception Handler
    WaitForException = 0x3D,
    ReturnFromException = 0x3E,
}

// Pending Instructions
// Throw privilege error if try to write to SR etc.

#[cfg(test)]
use strum::IntoEnumIterator;

#[cfg(test)]
#[must_use]
pub fn all_condition_flags() -> Vec<ConditionFlags> {
    ConditionFlags::iter().collect()
}

#[cfg(test)]
#[must_use]
pub fn all_shift_operands() -> Vec<ShiftOperand> {
    ShiftOperand::iter().collect()
}

#[cfg(test)]
#[must_use]
pub fn all_shift_types() -> Vec<ShiftType> {
    ShiftType::iter().collect()
}

#[cfg(test)]
#[must_use]
pub fn all_instructions() -> Vec<Instruction> {
    Instruction::iter().collect()
}
