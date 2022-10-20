use enum_dispatch::enum_dispatch;
use peripheral_mem::MemoryPeripheral;

use crate::instructions::definitions::*;
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

// Special

impl Executor for NullInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        set_sr_bit(StatusRegisterFields::CpuHalted, registers)
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
        registers.set_at_index(self.data.r2, src_value)
    }
}

// Arithmetic

impl Executor for AddInstructionData {
    // TODO: Set status bits for overflow/carry etc.
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        registers.set_at_index(self.data.r2, val_1 + val_2)
    }
}

impl Executor for SubtractInstructionData {
    // TODO: Set status bits for overflow/carry etc.
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        registers.set_at_index(self.data.r2, val_2 - val_1)
    }
}

impl Executor for MultiplyInstructionData {
    // TODO: Set status bits for overflow/carry etc.
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        registers.set_at_index(self.data.r2, val_2 * val_1)
    }
}

impl Executor for DivideInstructionData {
    // TODO: Set status bits for overflow/carry etc.
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        registers.set_at_index(self.data.r2, val_2 / val_1)
    }
}

// Comparison

impl Executor for IsEqualInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        set_comparison_result(registers, val_1 == val_2)
    }
}

impl Executor for IsNotEqualInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        set_comparison_result(registers, val_1 != val_2)
    }
}

impl Executor for IsLessThanInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        set_comparison_result(registers, val_1 < val_2)
    }
}

impl Executor for IsGreaterThanInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        set_comparison_result(registers, val_1 > val_2)
    }
}

impl Executor for IsLessOrEqualThanInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        set_comparison_result(registers, val_1 <= val_2)
    }
}

impl Executor for IsGreaterOrEqualThanInstructionData {
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        set_comparison_result(registers, val_1 >= val_2)
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
/// LastComparisonResult bit in the status register
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
/// LastComparisonResult bit in the status register
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
