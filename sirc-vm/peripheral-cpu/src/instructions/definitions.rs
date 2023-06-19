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
// Register: (e.g. ADD r1, r2)
// 6 bit instruction identifier (max 64 instructions)
// 4 bit register identifier
// 4 bit register identifier
// 4 bit register identifier (if any)
// 6 bit args
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

// Condition Flags

#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, Eq, Clone, Copy, Default)]
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
    pub op_code: Instruction,
    // TODO: Do we need anything more than DecodedInstruction
    // for these *InstructionData structs?
    pub condition_flag: ConditionFlags,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImmediateInstructionData {
    pub op_code: Instruction,
    pub register: u8,
    pub value: u16,
    pub condition_flag: ConditionFlags,
    // Max 2 bits (& 0x0003)
    pub additional_flags: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RegisterInstructionData {
    pub op_code: Instruction,
    pub r1: u8,
    pub r2: u8,
    pub r3: u8,
    pub condition_flag: ConditionFlags,
    // Max 10 bits (& 0x03FF)
    pub additional_flags: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InstructionData {
    Implied(ImpliedInstructionData),
    Immediate(ImmediateInstructionData),
    Register(RegisterInstructionData),
}

// TODO: Rename to OpCode or something?
#[derive(Debug, PartialEq, Eq, FromPrimitive, ToPrimitive, Default)]
// #[enum_dispatch(Executor)]
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
    // Shifts (Immediate)
    LogicalShiftLeftImmediate = 0x07,
    LogicalShiftRightImmediate = 0x08,
    ArithmeticShiftLeftImmediate = 0x09,
    ArithmeticShiftRightImmediate = 0x0A,
    RotateLeftImmediate = 0x0B,
    RotateRightImmediate = 0x0C,
    // Comparison (Immediate)
    CompareImmediate = 0x0D,

    // Arithmetic (Register)
    AddRegister = 0x10,
    AddRegisterWithCarry = 0x11,
    SubtractRegister = 0x12,
    SubtractRegisterWithCarry = 0x13,
    // Logic (Register)
    AndRegister = 0x14,
    OrRegister = 0x15,
    XorRegister = 0x16,
    // Shifts (Register)
    LogicalShiftLeftRegister = 0x17,
    LogicalShiftRightRegister = 0x18,
    ArithmeticShiftLeftRegister = 0x19,
    ArithmeticShiftRightRegister = 0x1A,
    RotateLeftRegister = 0x1B,
    RotateRightRegister = 0x1C,
    // Comparison (Register)
    CompareRegister = 0x1D,

    // Flow Control
    BranchImmediate = 0x20,
    BranchToSubroutineImmediate = 0x21,
    LoadEffectiveAddressFromIndirectImmediate = 0x22,
    LoadEffectiveAddressFromIndirectRegister = 0x23,
    ReturnFromSubroutine = 0x24,
    LongJumpWithImmediateDisplacement = 0x25,
    LongJumpWithRegisterDisplacement = 0x26,
    LongJumpToSubroutineWithImmediateDisplacement = 0x28,
    LongJumpToSubroutineWithRegisterDisplacement = 0x29,
    ShortJumpImmediate = 0x2A,
    ShortJumpToSubroutineImmediate = 0x2B,

    // Data Access
    LoadRegisterFromImmediate = 0x30,
    LoadRegisterFromRegister = 0x31,
    LoadRegisterFromIndirectImmediate = 0x32,
    LoadRegisterFromIndirectRegister = 0x33,
    LoadRegisterFromIndirectRegisterPostIncrement = 0x34,
    StoreRegisterToIndirectImmediate = 0x35,
    StoreRegisterToIndirectRegister = 0x36,
    StoreRegisterToIndirectRegisterPreDecrement = 0x37,

    // NOOP
    #[default]
    NoOperation = 0x3C,

    // Exception Handler
    WaitForException = 0x3D,
    Exception = 0x3E,
    ReturnFromException = 0x3F,
}

// Pending Instructions
// Throw privilege error if try to write to SR etc.
