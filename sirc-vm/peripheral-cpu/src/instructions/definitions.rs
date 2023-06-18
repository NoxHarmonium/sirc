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

use std::ops::Range;

use crate::registers::{sr_bit_is_set, Registers, StatusRegisterFields};

// 32 bits = 2x 16 bit
pub const INSTRUCTION_SIZE_WORDS: u32 = 2;
pub const INSTRUCTION_SIZE_BYTES: u32 = INSTRUCTION_SIZE_WORDS * 2;

// Condition Flags

#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, Eq, Clone, Copy)]
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

#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub enum ExecutionStepInstructionType {
    NoOp,
    MemoryRefRegDisplacement,
    MemoryRefImmDisplacement,
    RegisterRegisterAlu,
    RegisterImmediateAlu,
    Branch,
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub enum MemoryAccessInstructionType {
    NoOp,
    MemoryLoad,
    MemoryStore,
    BranchOrJump,
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub enum WriteBackInstructionType {
    NoOp,
    MemoryLoad,
    AluToRegister,
    LoadEffectiveAddress,
}

pub fn decode_execution_step_instruction_type(
    instruction: &Instruction,
    decoded_instruction: &DecodedInstruction,
) -> ExecutionStepInstructionType {
    if !decoded_instruction.con_ {
        return ExecutionStepInstructionType::NoOp;
    }

    match instruction {
        // Arithmetic (Immediate)
        Instruction::AddImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        Instruction::AddImmediateWithCarry => ExecutionStepInstructionType::RegisterImmediateAlu,
        Instruction::SubtractImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        Instruction::SubtractImmediateWithCarry => {
            ExecutionStepInstructionType::RegisterImmediateAlu
        }
        // Logic (Immediate)
        Instruction::AndImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        Instruction::OrImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        Instruction::XorImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        // Shifts (Immediate)
        Instruction::LogicalShiftLeftImmediate => {
            ExecutionStepInstructionType::RegisterImmediateAlu
        }
        Instruction::LogicalShiftRightImmediate => {
            ExecutionStepInstructionType::RegisterImmediateAlu
        }
        Instruction::ArithmeticShiftLeftImmediate => {
            ExecutionStepInstructionType::RegisterImmediateAlu
        }
        Instruction::ArithmeticShiftRightImmediate => {
            ExecutionStepInstructionType::RegisterImmediateAlu
        }
        Instruction::RotateLeftImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        Instruction::RotateRightImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        // Comparison (Immediate)
        Instruction::CompareImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        // Flow Control (Immediate)
        Instruction::ShortJumpImmediate => ExecutionStepInstructionType::Branch,
        Instruction::ShortJumpToSubroutineImmediate => ExecutionStepInstructionType::Branch,

        // Arithmetic (Register)
        Instruction::AddRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::AddRegisterWithCarry => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::SubtractRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::SubtractRegisterWithCarry => ExecutionStepInstructionType::RegisterRegisterAlu,
        // Logic (Register)
        Instruction::AndRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::OrRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::XorRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        // Shifts (Register)
        Instruction::LogicalShiftLeftRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::LogicalShiftRightRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::ArithmeticShiftLeftRegister => {
            ExecutionStepInstructionType::RegisterRegisterAlu
        }
        Instruction::ArithmeticShiftRightRegister => {
            ExecutionStepInstructionType::RegisterRegisterAlu
        }
        Instruction::RotateLeftRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::RotateRightRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        // Comparison (Register)
        Instruction::CompareRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        // NOOP (Register)
        Instruction::NoOperation => ExecutionStepInstructionType::NoOp,

        // Flow Control (Immediate)
        Instruction::BranchImmediate => ExecutionStepInstructionType::Branch,
        Instruction::BranchToSubroutineImmediate => ExecutionStepInstructionType::Branch,
        Instruction::WaitForException => ExecutionStepInstructionType::NoOp, // Handled by Exception Unit
        Instruction::ReturnFromException => ExecutionStepInstructionType::NoOp, // Handled by Exception Unit
        Instruction::Exception => ExecutionStepInstructionType::NoOp, // Handled by Exception Unit

        // Data Access
        Instruction::LoadRegisterFromImmediate => {
            ExecutionStepInstructionType::RegisterImmediateAlu
        }
        Instruction::LoadRegisterFromRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::LoadRegisterFromIndirectImmediate => {
            ExecutionStepInstructionType::RegisterImmediateAlu
        }
        Instruction::LoadRegisterFromIndirectRegister => {
            ExecutionStepInstructionType::RegisterRegisterAlu
        }
        Instruction::LoadRegisterFromIndirectRegisterPostIncrement => {
            ExecutionStepInstructionType::RegisterRegisterAlu
        }
        Instruction::StoreRegisterToIndirectImmediate => {
            ExecutionStepInstructionType::RegisterImmediateAlu
        }
        Instruction::StoreRegisterToIndirectRegister => {
            ExecutionStepInstructionType::RegisterRegisterAlu
        }
        Instruction::StoreRegisterToIndirectRegisterPreDecrement => {
            ExecutionStepInstructionType::RegisterRegisterAlu
        }

        Instruction::LongJumpWithImmediateDisplacement => {
            ExecutionStepInstructionType::MemoryRefImmDisplacement
        }
        Instruction::LongJumpToSubroutineWithRegisterDisplacement => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::ReturnFromSubroutine => ExecutionStepInstructionType::MemoryRefImmDisplacement, // Encoded as zero offset from link register
        Instruction::LoadEffectiveAddressFromIndirectImmediate => {
            ExecutionStepInstructionType::MemoryRefImmDisplacement
        }
        Instruction::LoadEffectiveAddressFromIndirectRegister => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::LongJumpWithRegisterDisplacement => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::LongJumpToSubroutineWithImmediateDisplacement => {
            ExecutionStepInstructionType::MemoryRefImmDisplacement
        }
    }
}

pub fn decode_memory_access_step_instruction_type(
    instruction: &Instruction,
    decoded_instruction: &DecodedInstruction,
) -> MemoryAccessInstructionType {
    if !decoded_instruction.con_ {
        return MemoryAccessInstructionType::NoOp;
    }

    match instruction {
        // Flow Control (Immediate)
        Instruction::ShortJumpImmediate => MemoryAccessInstructionType::BranchOrJump,
        Instruction::ShortJumpToSubroutineImmediate => MemoryAccessInstructionType::BranchOrJump,
        // Flow Control (Immediate)
        Instruction::BranchImmediate => MemoryAccessInstructionType::BranchOrJump,
        Instruction::BranchToSubroutineImmediate => MemoryAccessInstructionType::BranchOrJump,

        // Data Access
        Instruction::LoadRegisterFromImmediate => MemoryAccessInstructionType::MemoryLoad,
        Instruction::LoadRegisterFromRegister => MemoryAccessInstructionType::MemoryLoad,
        Instruction::LoadRegisterFromIndirectImmediate => MemoryAccessInstructionType::MemoryLoad,
        Instruction::LoadRegisterFromIndirectRegister => MemoryAccessInstructionType::MemoryLoad,
        Instruction::LoadRegisterFromIndirectRegisterPostIncrement => {
            MemoryAccessInstructionType::MemoryLoad
        }
        Instruction::StoreRegisterToIndirectImmediate => MemoryAccessInstructionType::MemoryStore,
        Instruction::StoreRegisterToIndirectRegister => MemoryAccessInstructionType::MemoryStore,
        Instruction::StoreRegisterToIndirectRegisterPreDecrement => {
            MemoryAccessInstructionType::MemoryStore
        }

        Instruction::LongJumpWithImmediateDisplacement => MemoryAccessInstructionType::BranchOrJump,
        Instruction::LongJumpToSubroutineWithRegisterDisplacement => {
            MemoryAccessInstructionType::BranchOrJump
        }
        Instruction::LongJumpWithRegisterDisplacement => MemoryAccessInstructionType::BranchOrJump,
        Instruction::LongJumpToSubroutineWithImmediateDisplacement => {
            MemoryAccessInstructionType::BranchOrJump
        }

        // Flow Control (Address Register Direct)
        _ => MemoryAccessInstructionType::NoOp,
    }
}

pub fn decode_write_back_step_instruction_type(
    instruction: &Instruction,
    decoded_instruction: &DecodedInstruction,
) -> WriteBackInstructionType {
    if !decoded_instruction.con_ {
        return WriteBackInstructionType::NoOp;
    }

    match instruction {
        // Arithmetic (Immediate)
        Instruction::AddImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::AddImmediateWithCarry => WriteBackInstructionType::AluToRegister,
        Instruction::SubtractImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::SubtractImmediateWithCarry => WriteBackInstructionType::AluToRegister,
        // Logic (Immediate)
        Instruction::AndImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::OrImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::XorImmediate => WriteBackInstructionType::AluToRegister,
        // Shifts (Immediate)
        Instruction::LogicalShiftLeftImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::LogicalShiftRightImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::ArithmeticShiftLeftImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::ArithmeticShiftRightImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::RotateLeftImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::RotateRightImmediate => WriteBackInstructionType::AluToRegister,
        // Comparison (Immediate)
        Instruction::CompareImmediate => WriteBackInstructionType::AluToRegister,
        // Arithmetic (Register)
        Instruction::AddRegister => WriteBackInstructionType::AluToRegister,
        Instruction::AddRegisterWithCarry => WriteBackInstructionType::AluToRegister,
        Instruction::SubtractRegister => WriteBackInstructionType::AluToRegister,
        Instruction::SubtractRegisterWithCarry => WriteBackInstructionType::AluToRegister,
        // Logic (Register)
        Instruction::AndRegister => WriteBackInstructionType::AluToRegister,
        Instruction::OrRegister => WriteBackInstructionType::AluToRegister,
        Instruction::XorRegister => WriteBackInstructionType::AluToRegister,
        // Shifts (Register)
        Instruction::LogicalShiftLeftRegister => WriteBackInstructionType::AluToRegister,
        Instruction::LogicalShiftRightRegister => WriteBackInstructionType::AluToRegister,
        Instruction::ArithmeticShiftLeftRegister => WriteBackInstructionType::AluToRegister,
        Instruction::ArithmeticShiftRightRegister => WriteBackInstructionType::AluToRegister,
        Instruction::RotateLeftRegister => WriteBackInstructionType::AluToRegister,
        Instruction::RotateRightRegister => WriteBackInstructionType::AluToRegister,
        // Comparison (Register)
        Instruction::CompareRegister => WriteBackInstructionType::AluToRegister,

        // Data Access
        Instruction::LoadRegisterFromImmediate => WriteBackInstructionType::MemoryLoad,
        Instruction::LoadRegisterFromRegister => WriteBackInstructionType::MemoryLoad,
        Instruction::LoadRegisterFromIndirectImmediate => WriteBackInstructionType::MemoryLoad,
        Instruction::LoadRegisterFromIndirectRegister => WriteBackInstructionType::MemoryLoad,
        Instruction::LoadRegisterFromIndirectRegisterPostIncrement => {
            WriteBackInstructionType::MemoryLoad
        }

        Instruction::LoadEffectiveAddressFromIndirectImmediate => {
            WriteBackInstructionType::LoadEffectiveAddress
        }
        Instruction::LoadEffectiveAddressFromIndirectRegister => {
            WriteBackInstructionType::LoadEffectiveAddress
        }
        _ => WriteBackInstructionType::NoOp,
    }
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

/**
* The instruction mapped out into components.
*
* Simulates the temporary registers the CPU would have when an instruction
* is being decoded.
*
* To avoid microcode/branching etc. all instructions are mapped out to the the
* same set of registers, however, depending on the instruction, some of the
* fields might be zero or full of garbage. You will need to make sure
* you know what instruction you are using before interpreting these
* registers.
*
* Future work: it might be a good idea in the future to type this so
* only the relevant registers are available for each instruction type.

*/
#[derive(Debug, Default, PartialEq, Eq)]
pub struct DecodedInstruction {
    // Raw Instruction Decode
    pub ins: u8,
    pub des: u8,
    pub sr_a: u8,
    pub sr_b: u8,
    pub con: u8,
    pub imm: u16,
    pub adr: u8,
    // Inferred
    pub ad_l: u8,
    pub ad_h: u8,
    pub addr_inc: i8,
    pub des_ad_l: u8,
    pub des_ad_h: u8,
    // Dereferenced
    pub des_: u16,
    pub sr_a_: u16,
    pub sr_b_: u16,
    pub ad_l_: u16,
    pub ad_h_: u16,
    pub con_: bool,
    pub sr: u16,
}

fn extract_range<T: TryFrom<u32>>(source: u32, range: Range<u8>) -> T {
    let shifted = source >> range.start;
    let masked = shifted & (u32::MAX >> (range.end - range.start));
    match T::try_from(masked) {
        Ok(val) => val,
        Err(_) => panic!(
            "Cannot fit u32 into [{}]. shifted:[{:#08x}] masked: [{:#08x}]",
            std::any::type_name::<T>(),
            shifted,
            masked
        ),
    }
}

pub fn decode_register_fetch(
    raw_instruction: [u8; 4],
    registers: &Registers,
) -> DecodedInstruction {
    let combined = u32::from_be_bytes(raw_instruction);

    let ins = extract_range(combined, 0..5);
    let adr = extract_range(combined, 24..27);
    let des = extract_range(combined, 6..9);
    let sr_a = extract_range(combined, 10..13);
    let sr_b = extract_range(combined, 14..17);
    let ad_l = (adr * 2) + 8;
    let ad_h = (adr * 2) + 7;
    let des_ad_l = (des * 2) + 8;
    let des_ad_h = (des * 2) + 7;
    let con = extract_range(combined, 28..31);
    let condition_flag: ConditionFlags =
        num::FromPrimitive::from_u8(con).expect("Condition flag can only be 4 bits long");

    let addr_inc: i8 = match ins {
        0x34 => 1,  // TODO: Match LOAD (a)+
        0x37 => -1, // TODO: Match STOR -(a)
        _ => 0,
    };

    DecodedInstruction {
        ins,
        des,
        sr_a,
        sr_b,
        con,
        imm: extract_range(combined, 10..26),
        adr,
        ad_l,
        ad_h,
        des_ad_l,
        des_ad_h,
        addr_inc,
        des_: registers[des],
        sr_a_: registers[sr_a],
        sr_b_: registers[sr_b],
        ad_l_: registers[ad_l],
        ad_h_: registers[ad_h],
        con_: condition_flag.should_execute(registers),
        sr: registers.sr,
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
#[derive(Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
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
    NoOperation = 0x3C,

    // Exception Handler
    WaitForException = 0x3D,
    Exception = 0x3E,
    ReturnFromException = 0x3F,
}

// Pending Instructions
// Throw privilege error if try to write to SR etc.
