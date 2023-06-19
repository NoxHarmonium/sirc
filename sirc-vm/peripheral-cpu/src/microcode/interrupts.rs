pub mod vectors {
    // These are all u16 because they are added to the system_ram_offset register to
    // get the full 24 bit address
    // The full vector range is 8 bits, so there are a possible 128 32-bit vector addresses
    // that can be defined

    /// An external device raised an error via a CPU pin
    /// This could happen, for example, if a unmapped address is presented by the CPU
    /// and another chip detects this and raises an error
    pub const BUS_ERROR: u16 = 0xFF00;
    /// This is raised internally by the CPU for invalid memory accesses etc.
    pub const ADDRESS_ERROR: u16 = 0xFF02;
    /// Raised with some instructions if the computed address would go outside the current
    /// segment. E.g. if you are accessing data in the program segment, and then compute
    /// and address that overflows into the next segment, you are probably doing something wrong.
    pub const SEGMENT_OVERFLOW: u16 = 0xFF04;
    /// Raised if dividing anything by zero.
    pub const DIVIDE_BY_ZERO: u16 = 0xFF06;
    /// Raised when an instruction is decoded that isn't explicitly documented AND
    /// does not map to the unimplemented instruction vectors below.
    pub const INVALID_OPCODE: u16 = 0xFF08;

    // TODO: Privilege violation?

    // ... up to 16 vectors reserved for future CPU errors ...

    pub const LEVEL_ONE_INTERRUPT: u16 = 0xFF20;
    pub const LEVEL_TWO_INTERRUPT: u16 = 0xFF22;
    pub const LEVEL_THREE_INTERRUPT: u16 = 0xFF24;
    pub const LEVEL_FOUR_INTERRUPT: u16 = 0xFF26;
    pub const LEVEL_FIVE_INTERRUPT: u16 = 0xFF28;
    pub const LEVEL_SIX_INTERRUPT: u16 = 0xFF2A;
    /// The highest priority interrupt that can never be masked
    /// It can interrupt anything, even another level seven interrupt
    pub const LEVEL_SEVEN_INTERRUPT: u16 = 0xFF2C;

    // ...more reserved vectors...

    // 16 Unimplemented instruction vectors...

    // These are useful for implementing "forward compatibility" with later
    // CPU versions. For example, if a new CPU version comes out that adds
    // new instructions, and a program is written that uses these instructions,
    // but is run on the older version of the CPU, these vectors will be triggered.
    // and the new CPU features can be emulated in software.
    //
    // Due to the instruction encoding, the number of instructions is limited to
    // 5 bits (64 instructions). The last 16 instruction opcodes (0x30-0x3F) map
    // to each of these vectors (e.g. 0x30 will trigger vector 0xFF70, 0x31 will
    // trigger vector 0x0FF72 and so on...)

    /// The start address of the 16 word range of unimplemented instruction vectors
    pub const UNIMPLEMENTED_INSTRUCTION_START: u16 = 0xFF60;
    /// The end address of the 16 word range of unimplemented instruction vectors
    pub const UNIMPLEMENTED_INSTRUCTION_END: u16 = 0xFF7F;

    // 64 user exception vectors triggered by the EXCP instruction (e.g. a TRAP on the 68k)

    pub const USER_EXCEPTION_VECTOR_START: u16 = 0xFF80;
    pub const USER_EXCEPTION_VECTOR_END: u16 = 0xFFFF;
}

use peripheral_mem::MemoryPeripheral;

use crate::registers::{set_interrupt_mask, set_sr_bit, Registers, StatusRegisterFields};

pub fn jump_to_interrupt(vector_offset: u8, registers: &mut Registers, mem: &MemoryPeripheral) {
    // Store the SR here because we need to flip to system mode to use the system stack
    // which will affect the SR
    // let old_sr = registers.sr;
    // Flip into system mode so we can use the system stack etc.
    set_sr_bit(StatusRegisterFields::SystemMode, registers);

    // Save important registers to restore after the ISR
    // push_address_to_stack(registers, mem, registers.get_full_pc_address());
    // push_value_to_stack(registers, mem, old_sr);

    // Jump to ISR
    let vector_address = registers.system_ram_offset + vector_offset as u32;
    (registers.ph, registers.pl) = (
        mem.read_address(vector_address),
        mem.read_address(vector_address + 1),
    )
}

// pub fn return_from_interrupt(registers: &mut Registers, mem: &MemoryPeripheral) {
// Get the important register values before we switch out of system mode
// and can't access them anymore
// registers.sr = pop_value_from_stack(registers, mem);
// (registers.ph, registers.pl) = pop_address_from_stack(registers, mem).to_segmented_address();
// }

pub fn trigger_hardware_interrupt(
    interrupt_level: u8,
    registers: &mut Registers,
    mem: &MemoryPeripheral,
) {
    if interrupt_level == 0 || interrupt_level > 0b111 {
        panic!("Interrupt level (0x{:08x}) must be greater than zero and fit in three bits (max 7 in decimal).", interrupt_level);
    }

    let vector_offset_start: u8 = vectors::LEVEL_ONE_INTERRUPT as u8 - 1;
    let vector_offset = vector_offset_start + interrupt_level;

    jump_to_interrupt(vector_offset, registers, mem);

    // TODO: Does it matter that we do this after the jump?
    // Make sure that interrupts of the same or lower priority don't interrupt this ISR
    set_interrupt_mask(registers, interrupt_level);
}
