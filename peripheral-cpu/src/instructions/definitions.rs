// Instruction (32 bit)
//
// Instruction formats:
//
// Implied: (e.g. HALT)
// 6 bit instruction identifier (max 64 instructions)
// 22 bit reserved
// 4 bit condition flags
//
// Immediate: (e.g. SET)
// 6 bit instruction identifier (max 64 instructions)
// 4 bit register identifier
// 16 bit value
// 2 bit reserved
// 4 bit conditions flags
//
// Register: (e.g. COPY)
// 6 bit instruction identifier (max 64 instructions)
// 4 bit register identifier
// 4 bit register identifier
// 4 bit register identifier (if any)
// 10 bit reserved
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
use crate::registers::Registers;
use enum_dispatch::enum_dispatch;
use peripheral_mem::MemoryPeripheral;

// 32 bits = 2x 16 bit
pub const INSTRUCTION_SIZE_WORDS: u32 = 2;
pub const INSTRUCTION_SIZE_BYTES: u32 = INSTRUCTION_SIZE_WORDS * 2;

// Condition Flags

#[derive(Debug, FromPrimitive, PartialEq, Eq)]
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

#[derive(Debug)]
pub enum DoubleRegisterTargetType {
    SingleRegister,
    RegistersCombineToOneWord,
    RegistersAreTwoWords,
}

// Instruction Types

#[derive(Debug)]
pub struct ImpliedInstructionData {
    pub condition_flag: ConditionFlags,
}

#[derive(Debug)]
pub struct ImmediateInstructionData {
    pub register: u8,
    pub value: u16,
    pub condition_flag: ConditionFlags,
    // Max 2 bits (& 0x0003)
    pub additional_flags: u8,
}

#[derive(Debug)]
pub struct RegisterInstructionData {
    pub r1: u8,
    pub r2: u8,
    pub r3: u8,
    pub condition_flag: ConditionFlags,
    // Max 10 bits (& 0x03FF)
    pub additional_flags: u8,
}

// Special

#[derive(Debug)]
pub struct HaltInstructionData {
    // ID: 0x00
    pub data: ImpliedInstructionData,
}

// Arithmetic

#[derive(Debug)]
pub struct AddInstructionData {
    // ID: 0x01
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct SubtractInstructionData {
    // ID: 0x02
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct MultiplyInstructionData {
    // ID: 0x03
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct DivideInstructionData {
    // ID: 0x04
    pub data: RegisterInstructionData,
}

// Logic

#[derive(Debug)]
pub struct AndInstructionData {
    // ID: 0x05
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct OrInstructionData {
    // ID: 0x06
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct XorInstructionData {
    // ID: 0x07
    pub data: RegisterInstructionData,
}

// Comparison

#[derive(Debug)]
pub struct CompareInstructionData {
    // ID: 0x08
    pub data: RegisterInstructionData,
}

// Flow Control

#[derive(Debug)]
pub struct ShortJumpInstructionData {
    // ID: 0x09
    pub data: ImpliedInstructionData,
}

#[derive(Debug)]
pub struct LongJumpInstructionData {
    // ID: 0x0A
    pub data: ImpliedInstructionData,
}

#[derive(Debug)]
pub struct BranchInstructionData {
    // ID: 0x0B
    pub data: ImmediateInstructionData,
}

// Data Access
// TODO: Limited double word load/store to make it easier to load full 24-bit pointers?

#[derive(Debug)]
pub struct LoadRegisterFromImmediateData {
    // ID: 0x0C
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct LoadRegisterFromRegisterData {
    // ID: 0x0D
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct LoadRegisterFromIndirectImmediateData {
    // ID: 0x0E
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct LoadRegisterFromIndirectRegisterData {
    // ID: 0x0F
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct StoreRegisterToIndirectImmediateData {
    // ID: 0x12
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct StoreRegisterToIndirectRegisterData {
    // ID: 0x13
    pub data: RegisterInstructionData,
}

// Interrupts

#[derive(Debug)]
pub struct WaitForInterruptInstructionData {
    // ID: 0x16
    pub data: ImpliedInstructionData,
}

#[derive(Debug)]
pub struct ReturnFromInterruptData {
    // ID: 0x17
    pub data: ImpliedInstructionData,
}

#[derive(Debug)]
pub struct TriggerSoftwareInterruptData {
    // ID: 0x18
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct DisableInterruptsData {
    // ID: 0x19
    pub data: ImpliedInstructionData,
}

#[derive(Debug)]
pub struct EnableInterruptsData {
    // ID: 0x1A
    pub data: ImpliedInstructionData,
}

// Subroutines

#[derive(Debug)]
pub struct BranchToSubroutineData {
    // ID: 0x1B
    pub data: ImmediateInstructionData,
}

#[derive(Debug)]
pub struct ShortJumpToSubroutineData {
    // ID: 0x1C
    pub data: ImpliedInstructionData,
}

#[derive(Debug)]
pub struct LongJumpToSubroutineData {
    // ID: 0x1D
    pub data: ImpliedInstructionData,
}

#[derive(Debug)]
pub struct ReturnFromSubroutineData {
    // ID: 0x1E
    pub data: ImpliedInstructionData,
}

// Shifts

#[derive(Debug)]
pub struct LogicalShiftLeftInstructionData {
    // ID: 0x1F
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct LogicalShiftRightInstructionData {
    // ID: 0x20
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct ArithmeticShiftLeftInstructionData {
    // ID: 0x21
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct ArithmeticShiftRightInstructionData {
    // ID: 0x22
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct RotateLeftInstructionData {
    // ID: 0x23
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct RotateRightInstructionData {
    // ID: 0x24
    pub data: RegisterInstructionData,
}

// Misc

#[derive(Debug)]
pub struct NoOperationInstructionData {
    // ID: 0x25
    pub data: ImpliedInstructionData,
}

#[derive(Debug)]
pub struct ClearAluStatusInstructionData {
    // ID: 0x26
    pub data: ImpliedInstructionData,
}

#[derive(Debug)]
pub struct SplitWordInstructionData {
    // ID: 0x27
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct JoinWordInstructionData {
    // ID: 0x28
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct PushInstructionData {
    // ID: 0x27
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
pub struct PopInstructionData {
    // ID: 0x28
    pub data: RegisterInstructionData,
}

#[derive(Debug)]
#[enum_dispatch(Executor)]
pub enum Instruction {
    // Special
    Halt(HaltInstructionData),
    // Arithmetic
    Add(AddInstructionData),
    Subtract(SubtractInstructionData),
    Multiply(MultiplyInstructionData),
    Divide(DivideInstructionData),
    // Logic
    And(AndInstructionData),
    Or(OrInstructionData),
    Xor(XorInstructionData),
    // Comparison
    Compare(CompareInstructionData),
    // Flow Control
    ShortJump(ShortJumpInstructionData),
    LongJump(LongJumpInstructionData),
    Branch(BranchInstructionData),
    // Data Access
    LoadRegisterFromImmediate(LoadRegisterFromImmediateData),
    LoadRegisterFromRegister(LoadRegisterFromRegisterData),
    LoadRegisterFromIndirectImmediate(LoadRegisterFromIndirectImmediateData),
    LoadRegisterFromIndirectRegister(LoadRegisterFromIndirectRegisterData),
    StoreRegisterToIndirectImmediate(StoreRegisterToIndirectImmediateData),
    StoreRegisterToIndirectRegister(StoreRegisterToIndirectRegisterData),
    // Interrupts
    WaitForInterrupt(WaitForInterruptInstructionData),
    ReturnFromInterrupt(ReturnFromInterruptData),
    TriggerSoftwareInterrupt(TriggerSoftwareInterruptData),
    DisableInterrupts(DisableInterruptsData),
    EnableInterrupts(EnableInterruptsData),
    // Subroutines
    BranchToSubroutine(BranchToSubroutineData),
    ShortJumpToSubroutine(ShortJumpToSubroutineData),
    LongJumpToSubroutine(LongJumpToSubroutineData),
    ReturnFromSubroutine(ReturnFromSubroutineData),
    // Shifts
    LogicalShiftLeft(LogicalShiftLeftInstructionData),
    LogicalShiftRight(LogicalShiftRightInstructionData),
    ArithmeticShiftLeft(ArithmeticShiftLeftInstructionData),
    ArithmeticShiftRight(ArithmeticShiftRightInstructionData),
    RotateLeft(RotateLeftInstructionData),
    RotateRight(RotateRightInstructionData),
    // Misc
    NoOperation(NoOperationInstructionData),
    ClearAluStatus(ClearAluStatusInstructionData),
    // Byte Manipulation
    SplitWord(SplitWordInstructionData),
    JoinWord(JoinWordInstructionData),
    // Stack Manipulation
    Push(PushInstructionData),
    Pop(PopInstructionData),
}

pub fn get_clocks_for_instruction(instruction: &Instruction) -> u32 {
    // Important!: Does not include instruction fetch time (4 cycles)
    //
    // Educated guess based on 6502 instruction set
    // (https://www.masswerk.at/6502/6502_instruction_set.html)
    // Hardware doesn't exist yet so subject to change
    // TODO: Move to executors where there is more context to calculate cycles?
    // TODO: Double check all these!
    match instruction {
        Instruction::Halt(_) => 2,
        Instruction::Add(_) => 2,
        Instruction::Subtract(_) => 2,
        // 70 is worst case for the 68k - maybe in the future it could be dynamic based on input
        // See: https://retrocomputing.stackexchange.com/a/7670
        Instruction::Multiply(_) => 70,
        // Worst case: Signed 156 / Unsigned 136
        // See: https://www.atari-forum.com/viewtopic.php?t=6484
        Instruction::Divide(_) => 156,
        Instruction::And(_) => 2,
        Instruction::Or(_) => 2,
        Instruction::Xor(_) => 2,
        Instruction::Compare(_) => 2,
        Instruction::LongJump(_) => 2,
        Instruction::ShortJump(_) => 2,
        Instruction::Branch(_) => 3,
        // TODO: OffsetPc and OffsetRegister can do a "double load" in that case the cycles would be double
        Instruction::WaitForInterrupt(_) => 1,
        Instruction::ReturnFromInterrupt(_) => 12,
        Instruction::TriggerSoftwareInterrupt(_) => 1,
        Instruction::DisableInterrupts(_) => 2,
        Instruction::EnableInterrupts(_) => 2,
        Instruction::LongJumpToSubroutine(_) => 6,
        Instruction::ShortJumpToSubroutine(_) => 6,
        Instruction::BranchToSubroutine(_) => 8,
        Instruction::ReturnFromSubroutine(_) => 12,
        // Shifts are 2 * number of shift so worse case is 32
        // TODO: Calculate these properly
        Instruction::RotateLeft(_) => 32,
        Instruction::RotateRight(_) => 32,
        Instruction::NoOperation(_) => 0,
        Instruction::ClearAluStatus(_) => 2,

        Instruction::LoadRegisterFromImmediate(_) => 4,
        Instruction::LoadRegisterFromRegister(_) => 4,
        Instruction::LoadRegisterFromIndirectImmediate(_) => 4,
        Instruction::LoadRegisterFromIndirectRegister(_) => 4,
        Instruction::StoreRegisterToIndirectImmediate(_) => 4,
        Instruction::StoreRegisterToIndirectRegister(_) => 4,
        Instruction::LogicalShiftLeft(_) => 32,
        Instruction::LogicalShiftRight(_) => 32,
        Instruction::ArithmeticShiftLeft(_) => 32,
        Instruction::ArithmeticShiftRight(_) => 32,
        Instruction::SplitWord(_) => 2,
        Instruction::JoinWord(_) => 2,
        Instruction::Push(_) => 6,
        Instruction::Pop(_) => 6,
    }
}

// Pending Instructions
// Throw privilege error if try to write to SR etc.
// Immediate versions of ALU instructions? (and shifting/rotate)
