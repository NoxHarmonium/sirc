use enum_dispatch::enum_dispatch;
use peripheral_mem::MemoryPeripheral;

use crate::instructions::definitions::*;
use crate::interrupts::vectors;
use crate::registers::*;

#[enum_dispatch]
pub trait Executor {
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral);
}

///
/// Sets or clears the LastComparisonResult bit of the status register
/// based on the given boolean value.
///
/// ```
/// use peripheral_cpu::executors::set_comparison_result;
/// use peripheral_cpu::registers::{Registers, StatusRegisterFields};
///
/// let mut registers = Registers::default();
///
/// set_comparison_result(&mut registers, true);
/// assert_eq!(registers.sr & StatusRegisterFields::LastComparisonResult as u16, StatusRegisterFields::LastComparisonResult as u16);
///
/// set_comparison_result(&mut registers, false);
/// assert_eq!(registers.sr & StatusRegisterFields::LastComparisonResult as u16, 0x00);
/// ```
///
/// TODO: Don't really want this public but it is exposed for tests
/// Maybe we need a test utility module or something, should look into it
pub fn set_comparison_result(registers: &mut Registers, result: bool) {
    if result {
        set_sr_bit(StatusRegisterFields::LastComparisonResult, registers);
    } else {
        clear_sr_bit(StatusRegisterFields::LastComparisonResult, registers);
    }
}

pub fn push_address_to_stack(registers: &mut Registers, mem: &MemoryPeripheral, address: u32) {
    let (hw, lw) = ((address >> u16::BITS) as u16, (address & 0x0000FFFF) as u16);
    push_value_to_stack(registers, mem, hw);
    push_value_to_stack(registers, mem, lw);
}

pub fn pop_address_from_stack(registers: &mut Registers, mem: &MemoryPeripheral) -> u32 {
    let (lw, hw) = (
        pop_value_from_stack(registers, mem),
        pop_value_from_stack(registers, mem),
    );
    ((hw as u32) << u16::BITS) & lw as u32
}

pub fn push_value_to_stack(registers: &mut Registers, mem: &MemoryPeripheral, value: u16) {
    let current_stack_pointer = registers.get_full_sp_address();
    mem.write_address(current_stack_pointer, value);
    // Stack pointers wrap around a segment so that interrupts will still work on a stack overflow
    registers.set_segmented_sp(current_stack_pointer.wrapping_sub(1).to_segmented_address());
}

pub fn pop_value_from_stack(registers: &mut Registers, mem: &MemoryPeripheral) -> u16 {
    let current_stack_pointer = registers.get_full_sp_address();
    let value = mem.read_address(current_stack_pointer);
    // Stack pointers wrap around a segment so that interrupts will still work on a stack overflow
    registers.set_segmented_sp(current_stack_pointer.wrapping_add(1).to_segmented_address());
    value
}

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

pub fn jump_to_interrupt(vector_offset: u8, registers: &mut Registers, mem: &MemoryPeripheral) {
    // Store the SR here because we need to flip to system mode to use the system stack
    // which will affect the SR
    let old_sr = registers.sr;
    // Flip into system mode so we can use the system stack etc.
    set_sr_bit(StatusRegisterFields::SystemMode, registers);

    // Save important registers to restore after the ISR
    push_address_to_stack(registers, mem, registers.get_full_pc_address());
    push_value_to_stack(registers, mem, old_sr);

    // Jump to ISR
    let vector_address = registers.system_ram_offset + vector_offset as u32;
    (registers.ph, registers.pl) = (
        mem.read_address(vector_address),
        mem.read_address(vector_address + 1),
    )
}

pub fn return_from_interrupt(registers: &mut Registers, mem: &MemoryPeripheral) {
    // Get the important register values before we switch out of system mode
    // and can't access them anymore
    registers.sr = pop_value_from_stack(registers, mem);
    (registers.ph, registers.pl) = pop_address_from_stack(registers, mem).to_segmented_address();
}

// Special

///
/// Halts the CPU by setting the CpuHalted flag.
///
/// Usually used for debugging or for preventing run away computation
/// (e.g. if the PC runs into an empty chunk of memory it will be interpreted as a halt).
///
/// If you want to wait for an event, another instruction should be used such as
/// the WAIT instruction.
///
impl Executor for NullInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        set_sr_bit(StatusRegisterFields::CpuHalted, registers);
    }
}

// Register Transfer

impl Executor for SetInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        registers.set_at_index(self.data.register, self.data.value);
    }
}

impl Executor for SetAddressInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        (registers.ah, registers.al) = self.data.address.to_segmented_address();
    }
}

impl Executor for CopyInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let src_value = registers.get_at_index(self.data.r1);
        registers.set_at_index(self.data.r2, src_value);
    }
}

// Arithmetic

///
/// Executes an addition operation on two registers, storing the result in
/// the second operand.
///
/// If an unsigned overflow occurs, the carry status flag will be set.
/// If a signed overflow occurs, the overflow status flag will be set.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, AddInstructionData};
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set, StatusRegisterFields};
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
/// // Thanks: https://stackoverflow.com/a/69125543/1153203
///
/// let mut mem = new_memory_peripheral();
///
/// let mut registers = Registers::default();
///
/// let instructionData = AddInstructionData {
///     data: RegisterInstructionData {
///         r1: 0x00, // x1
///         r2: 0x01, // x2
///         r3: 0x00, // unused
///     }
/// };
///
/// // Unsigned Overflow
/// registers.x1 = 0xFFFF;
/// registers.x2 = 0x0001;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x2, 0x0000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), false);
///
/// // Signed Overflow
/// registers.x1 = 0x7FFF;
/// registers.x2 = 0x2000;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x2, 0x9FFF);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), false);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), true);
///
/// // Both Overflow
/// registers.x1 = 0x9FFF;
/// registers.x2 = 0x9000;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x2, 0x2FFF);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), true);
/// ```
///
impl Executor for AddInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        let (result, carry) = val_2.overflowing_add(val_1);
        let (_, overflow) = (val_2 as i16).overflowing_add(val_1 as i16);
        if carry {
            set_sr_bit(StatusRegisterFields::Carry, registers);
        } else {
            clear_sr_bit(StatusRegisterFields::Carry, registers);
        }
        if overflow {
            set_sr_bit(StatusRegisterFields::Overflow, registers);
        } else {
            clear_sr_bit(StatusRegisterFields::Overflow, registers);
        }

        registers.set_at_index(self.data.r2, result);
    }
}

///
/// Executes an subtraction operation on two registers, storing the result in
/// the second operand.
///
/// The first operand is subtracted from the second operand.
///
/// If an unsigned overflow occurs, the carry status flag will be set.
/// If a signed overflow occurs, the overflow status flag will be set.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, SubtractInstructionData};
/// use peripheral_cpu::registers::{Registers, sr_bit_is_set, StatusRegisterFields};
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
/// // Thanks: http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
///
/// let mut mem = new_memory_peripheral();
///
/// let mut registers = Registers::default();
///
/// let instructionData = SubtractInstructionData {
///     data: RegisterInstructionData {
///         r1: 0x00, // x1
///         r2: 0x01, // x2
///         r3: 0x00, // unused
///     }
/// };
///
/// // Unsigned Overflow
/// registers.x1 = 0xFFFF;
/// registers.x2 = 0x5FFF;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x2, 0x6000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), false);
///
/// // Signed Overflow
/// registers.x1 = 0x7FFF;
/// registers.x2 = 0xDFFF;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x2, 0x6000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), false);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), true);
///
/// // Both Overflow
/// registers.x1 = 0xBFFF;
/// registers.x2 = 0x5FFF;
///
/// instructionData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x2, 0xA000);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Carry, &registers), true);
/// assert_eq!(sr_bit_is_set(StatusRegisterFields::Overflow, &registers), true);
/// ```
///
impl Executor for SubtractInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = val_2.overflowing_sub(val_1);
        let (_, overflow) = (val_2 as i16).overflowing_sub(val_1 as i16);
        if carry {
            set_sr_bit(StatusRegisterFields::Carry, registers);
        } else {
            clear_sr_bit(StatusRegisterFields::Carry, registers);
        }
        if overflow {
            set_sr_bit(StatusRegisterFields::Overflow, registers);
        } else {
            clear_sr_bit(StatusRegisterFields::Overflow, registers);
        }

        registers.set_at_index(self.data.r2, result);
    }
}

impl Executor for MultiplyInstructionData {
    // TODO: Set status bits for overflow/carry etc.
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        registers.set_at_index(self.data.r2, val_2 * val_1);
    }
}

impl Executor for DivideInstructionData {
    // TODO: Set status bits for overflow/carry etc.
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        registers.set_at_index(self.data.r2, val_2 / val_1);
    }
}

// Comparison

impl Executor for IsEqualInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        set_comparison_result(registers, val_1 == val_2);
    }
}

impl Executor for IsNotEqualInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        set_comparison_result(registers, val_1 != val_2);
    }
}

impl Executor for IsLessThanInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        set_comparison_result(registers, val_1 < val_2);
    }
}

impl Executor for IsGreaterThanInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        set_comparison_result(registers, val_1 > val_2);
    }
}

impl Executor for IsLessOrEqualThanInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        set_comparison_result(registers, val_1 <= val_2);
    }
}

impl Executor for IsGreaterOrEqualThanInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        set_comparison_result(registers, val_1 >= val_2);
    }
}

// Flow Control

///
/// Moves the CPU program counter to the specified address.
///
/// Takes a 24-bit address as an immediate value which can be applied
/// directly to the program counter without any transformation.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{AddressInstructionData, JumpInstructionData};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let jumpInstruction = JumpInstructionData {
///   data: AddressInstructionData {
///     address: 0x00CAFECA
///   }
/// };
/// let mut registers = Registers::default();
/// let mem = new_memory_peripheral();
/// jumpInstruction.execute(&mut registers, &mem);
///
/// assert_eq!((registers.ph, registers.pl), (0x00CA, 0xFECA));
///
/// ```
///
impl Executor for JumpInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        (registers.ph, registers.pl) = self.data.address.to_segmented_address();
    }
}

///
/// Moves the CPU program counter to the specified address IF the
/// LastComparisonResult bit in the status register is set.
///
/// Takes a 24-bit address as an immediate value which can be applied
/// directly to the program counter without any transformation.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{AddressInstructionData, JumpIfInstructionData};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::{Executor, set_comparison_result};
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let jumpIfInstruction = JumpIfInstructionData {
///   data: AddressInstructionData {
///     address: 0x00CAFECA
///   }
/// };
/// let mut registers = Registers::default();
/// let mem = new_memory_peripheral();
///
/// set_comparison_result(&mut registers, false);
/// jumpIfInstruction.execute(&mut registers, &mem);
/// // No Jump
/// assert_eq!((registers.ph, registers.pl), (0x0000, 0x0000));
///
/// set_comparison_result(&mut registers, true);
/// jumpIfInstruction.execute(&mut registers, &mem);
/// // Jump!
/// assert_eq!((registers.ph, registers.pl), (0x00CA, 0xFECA));
///
/// ```
///
impl Executor for JumpIfInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        if sr_bit_is_set(StatusRegisterFields::LastComparisonResult, registers) {
            (registers.ph, registers.pl) = self.data.address.to_segmented_address();
        }
    }
}

///
/// Moves the CPU program counter to the specified address IF the
/// LastComparisonResult bit in the status register is not set.
///
/// Takes a 24-bit address as an immediate value which can be applied
/// directly to the program counter without any transformation.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{AddressInstructionData, JumpIfNotInstructionData};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::{Executor, set_comparison_result};
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let jumpIfNotInstruction = JumpIfNotInstructionData {
///   data: AddressInstructionData {
///     address: 0x00CAFECA
///   }
/// };
/// let mut registers = Registers::default();
/// let mem = new_memory_peripheral();
///
/// set_comparison_result(&mut registers, true);
/// jumpIfNotInstruction.execute(&mut registers, &mem);
/// // No Jump
/// assert_eq!((registers.ph, registers.pl), (0x0000, 0x0000));
///
/// set_comparison_result(&mut registers, false);
/// jumpIfNotInstruction.execute(&mut registers, &mem);
/// // Jump!
/// assert_eq!((registers.ph, registers.pl), (0x00CA, 0xFECA));
///
/// ```
///
impl Executor for JumpIfNotInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        if !sr_bit_is_set(StatusRegisterFields::LastComparisonResult, registers) {
            (registers.ph, registers.pl) = self.data.address.to_segmented_address();
        }
    }
}

// Data Access

///
/// Loads a 16 bit value out of a memory address into a register.
///
/// The base address value is specified by the address registers (ah/al)
/// the first operand is the destination register and the second register
/// is an offset to add to the base address value.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, LoadOffsetRegisterData};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::{Executor, set_comparison_result};
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let loadOffsetRegisterInstruction = LoadOffsetRegisterData {
///   data: RegisterInstructionData {
///     r1: 0x00, // x1
///     r2: 0x03, // y1
///     r3: 0x00, // unused
///   }
/// };
/// let mut registers = Registers { ah: 0x1011, al: 0x1110, y1: 0x0001, ..Registers::default() };
///
/// let mut mem = new_memory_peripheral();
/// mem.map_segment("TEST", 0x00110000, 0xFFFF, true);
/// mem.write_address(0x00111111, 0xCAFE);
///
/// loadOffsetRegisterInstruction.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x1, 0xCAFE);
///
/// ```
///
impl Executor for LoadOffsetRegisterData {
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let (segment, address) = registers.get_segmented_address();
        // TODO: Segment overflow?
        let offset_address = address + registers.get_at_index(self.data.r2);
        let address_to_read = (segment, offset_address).to_full_address();
        let read_value = mem.read_address(address_to_read);
        registers.set_at_index(self.data.r1, read_value);
    }
}

impl Executor for StoreOffsetRegisterData {
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let (segment, address) = registers.get_segmented_address();
        // TODO: Segment overflow?
        let offset_address = address + registers.get_at_index(self.data.r2);
        let address_to_write = (segment, offset_address).to_full_address();
        mem.write_address(address_to_write, registers.get_at_index(self.data.r1));
    }
}

impl Executor for LoadOffsetImmediateData {
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let (segment, address) = registers.get_segmented_address();
        // TODO: Segment overflow?
        let offset_address = address + self.data.value;
        let address_to_read = (segment, offset_address).to_full_address();
        let read_value = mem.read_address(address_to_read);
        registers.set_at_index(self.data.register, read_value);
    }
}

impl Executor for StoreOffsetImmediateData {
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let (segment, address) = registers.get_segmented_address();
        // TODO: Segment overflow?
        let offset_address = address + self.data.value;
        let address_to_write = (segment, offset_address).to_full_address();
        mem.write_address(address_to_write, registers.get_at_index(self.data.register));
    }
}

// Interrupts

impl Executor for WaitForInterruptInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        set_sr_bit(StatusRegisterFields::WaitingForInterrupt, registers);
    }
}

impl Executor for ReturnFromInterruptData {
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        return_from_interrupt(registers, mem);
    }
}

impl Executor for TriggerSoftwareInterruptData {
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        // Instruction immediate operand is truncated to 7 bits and used as the vector offset
        // for the exception. In effect this means that 128 user exceptions can be defined
        let vector_offset = (self.data.value & 0x0000007F) as u8;
        jump_to_interrupt(vector_offset, registers, mem);
    }
}

impl Executor for DisableInterruptsData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        // No interrupt is associated seventh priority level so it effectively
        // masks all interrupts (except NMI)
        set_interrupt_mask(registers, 0b0111);
    }
}

impl Executor for EnableInterruptsData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        // There is no level zero interrupt, so if the mask is zero,
        // no interrupts are masked
        set_interrupt_mask(registers, 0);
    }
}

// Subroutines

impl Executor for JumpToSubroutineData {
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        push_address_to_stack(registers, mem, self.data.address);
        (registers.ph, registers.pl) = self.data.address.to_segmented_address();
    }
}

impl Executor for ReturnFromSubroutineData {
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let return_address = pop_address_from_stack(registers, mem);
        (registers.ph, registers.pl) = return_address.to_segmented_address();
    }
}
