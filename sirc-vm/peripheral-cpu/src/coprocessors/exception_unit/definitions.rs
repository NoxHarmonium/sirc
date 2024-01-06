pub mod vectors {
    // These are all u16 because they are added to the system_ram_offset register to
    // get the full 24 bit address
    // The full vector range is 8 bits, so there are a possible 128 32-bit vector addresses
    // that can be defined. Multiply the vector ID by two to get the actually memory address
    // The first 64 addresses are privileged and can only be raised by hardware or in system
    // mode. The second 64 addresses can be raised in user mode to trap into system mode.
    //
    // Priority is determined 7 minus the value first nibble (e.g. 0x00 is priority 7, 0x40 is priority 3, 0x60 and above are all priority 1)

    // Privileged Abort Exceptions (0x00-0x07)

    /// An external device raised an error via a CPU pin
    /// This could happen, for example, if a unmapped address is presented by the CPU
    /// and another chip detects this and raises an error.
    /// It could also be used to implement virtual memory as the instrtuction is aborted
    /// and pl is not incremented for a bus fault.
    pub const BUS_FAULT: u8 = 0x00;
    /// Raised when fetching instructions if pl points to an odd address
    /// This is to simplify fetching as the second word of an instruction is always at pl | 0x1
    /// and ensures that we never have to worry about instructions overflowing segments
    /// (e.g. if the first word is at 0xFFFF and the second word at 0x0000)
    /// which is a weird edge case that might complicate fetching.
    pub const ALIGNMENT_FAULT: u8 = 0x01;
    /// Raised with some instructions if the computed address would go outside the current
    /// segment. E.g. if you are accessing data in the stack segment, and then compute
    /// an address that overflows, it is probably a stack overflow.
    /// There might be situations where you want address calculations to wrap around
    /// so it is only raised if the `TrapOnAddressOverflow` SR bit is set.
    pub const SEGMENT_OVERFLOW_FAULT: u8 = 0x02;
    /// Raised when a co-processor call is done for a non-existant co-processor
    /// or if the co-processor opcode is invalid.
    /// Can be used for forward compatibility.
    /// For example, if the next iteration of the CPU included a floating point co-processor
    /// programs written for that co-processor would trap on the older iteration of the CPU
    /// and the floating point operation could be emulated in software.
    /// Note: There is no invalid opcode detection for CPU instructions outside of co-processor calls
    /// This is just to keep things simple. There is a risk of people using undocumented instructions
    /// and those programs breaking in future iterations of the CPU but it is expected that the core
    /// of the CPU will remain stable and future ISA improvements will be done via co-processors.
    pub const INVALID_OPCODE_FAULT: u8 = 0x03;

    /// Raised when not in system mode and a privileged operation is performed:
    /// 1. Writing to the high word of any address registers
    /// 2. Writing to the high byte of the SR register
    /// 3. Triggering exception below 0x80
    pub const PRIVILEGE_VIOLATION_FAULT: u8 = 0x04;

    // 0x05-0x07 Reserved

    // Privileged Regular Exceptions (0x08-0x0F)

    /// Raised after every instruction when the `TraceMode` SR bit is set
    /// Used for debugging
    pub const INSTRUCTION_TRACE_FAULT: u8 = 0x08;

    /// Raised when a level five hardware exception is raised
    /// when one is already being handled
    /// We don't use a stack for handling exceptions, so we can't
    /// make a NMI because there is nowhere to store the return address
    /// We could just ignore any level five HW exceptions while it is
    /// masked, but it could indicate a hardware misconfiguration,
    /// so it is handy so that hardware bugs for things that should
    /// not be interrupted are picked up.
    pub const LEVEL_FIVE_HARDWARE_EXCEPTION_CONFLICT: u8 = 0x09;

    //  0x09-0x0F Reserved

    /// Hardware Exceptions

    // Special level - When level five hardware exception is masked and
    // another one is triggered, it isn't ignored, it triggers a LEVEL_FIVE_HARDWARE_EXCEPTION_CONFLICT
    // (see above)
    pub const LEVEL_FIVE_HARDWARE_EXCEPTION: u8 = 0x10;
    pub const LEVEL_FOUR_HARDWARE_EXCEPTION: u8 = 0x20;
    pub const LEVEL_THREE_HARDWARE_EXCEPTION: u8 = 0x30;
    pub const LEVEL_TWO_HARDWARE_EXCEPTION: u8 = 0x40;
    pub const LEVEL_ONE_HARDWARE_EXCEPTION: u8 = 0x50;

    /// User Exceptions
    // 159 user exception vectors triggered by the EXCP instruction (e.g. a TRAP on the 68k)

    pub const USER_EXCEPTION_VECTOR_START: u8 = 0x60;
    pub const USER_EXCEPTION_VECTOR_END: u8 = 0xFF;
}

// Instruction priorities
// 1: Software: COP
// 2-6: Hardware
// 7: Faults

// Exception types
// Abort Exception means that the instruction does not have any effect (it is cancelled after decode)
// The program address stored in the link register is the address of the faulting instruction
// so it can be retried (RETI will return to the same instruction again)
// This is important for things like privilege violation because you don't want the illegal
// instruction to do anything.
// Regular Exception means the instruction finishes (is not cancelled)
// The program address stored in the link register is the address after the faulting one
//
// Abort Exceptions (0x0-0xF): Reset, Bus Fault, Alignment Fault, Privilege violation, Invalid Op Code
// Regular Exceptions: (0x10-0xFF) COP, Hardware, All other faults

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum ExceptionPriorities {
    NoException = 0x0,
    Software = 0x1,
    LevelOneHardware = 0x2,
    LevelTwoHardware = 0x3,
    LevelThreeHardware = 0x4,
    LevelFourHardware = 0x5,
    LevelFiveHardware = 0x6,
    Fault = 0x7,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum ExceptionUnitOpCodes {
    SoftwareException = 0x1,

    // Privileged
    WaitForException = 0x9,
    ReturnFromException = 0xA,
    Reset = 0xB,
    HardwareException = 0xF,
}
