pub mod vectors {
    // These are all u16 because they are added to the system_ram_offset register to
    // get the full 24 bit address
    // The full vector range is 8 bits, so there are a possible 128 32-bit vector addresses
    // that can be defined

    /// An external device raised an error via a CPU pin
    /// This could happen, for example, if a unmapped address is presented by the CPU
    /// and another chip detects this and raises an error
    pub const BUS_ERROR: u16 = 0x00;
    /// This is raised internally by the CPU for invalid memory accesses etc.
    pub const ADDRESS_ERROR: u16 = 0x02;
    /// Raised with some instructions if the computed address would go outside the current
    /// segment. E.g. if you are accessing data in the program segment, and then compute
    /// and address that overflows into the next segment, you are probably doing something wrong.
    pub const SEGMENT_OVERFLOW: u16 = 0x04;
    /// Raised if dividing anything by zero.
    pub const DIVIDE_BY_ZERO: u16 = 0x06;
    /// Raised when an instruction is decoded that isn't explicitly documented AND
    /// does not map to the unimplemented instruction vectors below.
    pub const INVALID_OPCODE: u16 = 0x08;

    // TODO: Privilege violation?

    // ... up to 16 vectors reserved for future CPU errors ...

    pub const LEVEL_ONE_INTERRUPT: u16 = 0x20;
    pub const LEVEL_TWO_INTERRUPT: u16 = 0x22;
    pub const LEVEL_THREE_INTERRUPT: u16 = 0x24;
    pub const LEVEL_FOUR_INTERRUPT: u16 = 0x26;
    pub const LEVEL_FIVE_INTERRUPT: u16 = 0x28;
    pub const LEVEL_SIX_INTERRUPT: u16 = 0x2A;
    /// The highest priority interrupt that can never be masked
    /// It can interrupt anything, even another level seven interrupt
    pub const LEVEL_SEVEN_INTERRUPT: u16 = 0x2C;

    // ...more reserved vectors...

    // 128 user exception vectors triggered by the EXCP instruction (e.g. a TRAP on the 68k)

    pub const USER_EXCEPTION_VECTOR_START: u16 = 0x80;
    pub const USER_EXCEPTION_VECTOR_END: u16 = 0xFF;
}
