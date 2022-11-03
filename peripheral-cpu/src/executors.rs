use enum_dispatch::enum_dispatch;

use peripheral_mem::MemoryPeripheral;

use crate::instructions::definitions::*;
use crate::interrupts::vectors;
use crate::registers::*;

#[enum_dispatch]
pub trait Executor {
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral);
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
impl Executor for HaltInstructionData {
    // ID: 0x00
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        set_sr_bit(StatusRegisterFields::CpuHalted, registers);
    }
}

// Arithmetic

// TODO: Surely there is a way to extract common logic from all the ALU executors

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
    // ID: 0x01
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        let (result, carry) = val_1.overflowing_add(val_2);
        set_alu_bits(registers, result, carry, Some((val_1, val_2, result)));

        registers.set_at_index(self.data.r1, result);
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
    // ID: 0x02
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = val_1.overflowing_sub(val_2);
        set_alu_bits(registers, result, carry, Some((val_1, val_2, result)));

        registers.set_at_index(self.data.r1, result);
    }
}

impl Executor for MultiplyInstructionData {
    // ID: 0x03
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = val_1.overflowing_mul(val_2);
        set_alu_bits(registers, result, carry, Some((val_1, val_2, result)));

        registers.set_at_index(self.data.r1, result);
    }
}

impl Executor for DivideInstructionData {
    // ID: 0x04
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = val_1.overflowing_div(val_2);
        set_alu_bits(registers, result, carry, Some((val_1, val_2, result)));

        registers.set_at_index(self.data.r1, result);
    }
}

// Logic

impl Executor for AndInstructionData {
    // ID: 0x05
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let result = val_1 & val_2;
        set_alu_bits(registers, result, false, Some((val_1, val_2, result)));

        registers.set_at_index(self.data.r1, result);
    }
}

impl Executor for OrInstructionData {
    // ID: 0x06
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let result = val_1 | val_2;
        set_alu_bits(registers, result, false, Some((val_1, val_2, result)));

        registers.set_at_index(self.data.r1, result);
    }
}

impl Executor for XorInstructionData {
    // ID: 0x07
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let result = val_1 ^ val_2;
        set_alu_bits(registers, result, false, Some((val_1, val_2, result)));

        registers.set_at_index(self.data.r1, result);
    }
}

// Comparison

impl Executor for CompareInstructionData {
    // ID: 0x08
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        // TODO: Deduplicate this with the subtract executor
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = val_1.overflowing_sub(val_2);
        set_alu_bits(registers, result, carry, Some((val_1, val_2, result)));

        // Exactly the same as a subtraction but does not store the result (only the status bits)
    }
}

// Flow Control

///
/// Transfer the lower word address register to the lower word of the program counter.
///
/// Allows a jump to anywhere within a segment.
///
/// Not a privileged operation because it only allows the program to jump within its
/// current segment.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{ImpliedInstructionData, ShortJumpInstructionData};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let jumpInstruction = ShortJumpInstructionData {
///   data: ImpliedInstructionData {}
/// };
/// let mut registers = Registers::default();
/// registers.set_full_address_address(0x00CAFECA);
/// let mem = new_memory_peripheral();
/// jumpInstruction.execute(&mut registers, &mem);
///
/// assert_eq!((registers.ph, registers.pl), (0x0000, 0xFECA));
///
/// ```
///
impl Executor for ShortJumpInstructionData {
    // ID: 0x09
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        registers.pl = registers.al;
    }
}

///
/// Transfer the address registers to the program counter.
///
/// Allows a full 24-bit address to be loaded into the address registers
/// and used to specify where execution starts.
///
/// This should be a privileged operation because it allows the program to jump
/// anywhere it wants (escape its segment).
///
/// ```
/// use peripheral_cpu::instructions::definitions::{ImpliedInstructionData, LongJumpInstructionData};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let jumpInstruction = LongJumpInstructionData {
///   data: ImpliedInstructionData {}
/// };
/// let mut registers = Registers::default();
/// registers.set_full_address_address(0x00CAFECA);
/// let mem = new_memory_peripheral();
/// jumpInstruction.execute(&mut registers, &mem);
///
/// assert_eq!((registers.ph, registers.pl), (0x00CA, 0xFECA));
///
/// ```
///
impl Executor for LongJumpInstructionData {
    // ID: 0x0A
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        (registers.ph, registers.pl) = (registers.ah, registers.al);
    }
}

///
/// Moves the CPU program counter by the specified offset, relative
/// to the current program counter.
///
/// Useful for making position independent programs (and avoiding having to
/// store in the address register)
///
///
impl Executor for BranchInstructionData {
    // ID: 0x0B
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        // TODO: Segment overflow?
        let effective_address = registers.pl + self.data.value;
        registers.pl = effective_address;
    }
}

// Data Access

///
/// Loads an immediate 16 bit value encoded in an instruction into a register.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{ImmediateInstructionData, LoadRegisterFromImmediateData};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let loadRegisterFromImmediateInstruction = LoadRegisterFromImmediateData {
///   data: ImmediateInstructionData {
///     r1: 0x02, // x3
///     value: 0xCAFE,
///     condition_flag: 0x00, // always
///   }
/// };
/// let mut registers = Registers::default()
///
/// let mut mem = new_memory_peripheral();
///
/// loadRegisterFromImmediateInstruction.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x3, 0xCAFE);
///
/// ```
///
impl Executor for LoadRegisterFromImmediateData {
    // ID: 0x0C
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        registers.set_at_index(self.data.register, self.data.value);
    }
}

///
/// Loads a 16 bit value from a register into another register.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{RegisterInstructionData, LoadRegisterFromRegisterData};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::Executor;
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let loadRegisterFromRegisterData = LoadRegisterFromRegisterData {
///   data: RegisterInstructionData {
///     r1: 0x02, // x3
///     r2: 0x01, // x1
///     r3: 0x00,
///     condition_flag: 0x00, // always
///   }
/// };
/// let mut registers = { x1: 0xCAFE, ..Registers::default() }
///
/// let mut mem = new_memory_peripheral();
///
/// loadRegisterFromRegisterData.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x3, 0xCAFE);
///
/// ```
///
impl Executor for LoadRegisterFromRegisterData {
    // ID: 0x0D
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let source_value = registers.get_at_index(self.data.r2);
        registers.set_at_index(self.data.r1, source_value)
    }
}

///
/// Loads a 16 bit value out of a memory address into a register.
///
/// The base address value is specified by the address registers (ah/al)
/// the first operand is the destination register and the operand is an
/// immediate offset to add to the base address value.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{ImmediateInstructionData, LoadOffsetRegisterData};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::{Executor, set_comparison_result};
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let loadRegisterFromIndirectImmediate = LoadRegisterFromIndirectImmediateData {
///   data: ImmediateInstructionData {
///     r1: 0x02, // x3
///     value: 0x0001,
///     condition_flag: 0x00, // always
///   }
/// };
/// let mut registers = Registers { ah: 0x1011, al: 0x1110, ..Registers::default() };
///
/// let mut mem = new_memory_peripheral();
/// mem.map_segment("TEST", 0x00110000, 0xFFFF, true);
/// mem.write_address(0x00111111, 0xCAFE);
///
/// loadRegisterFromIndirectImmediate.execute(&mut registers, &mem);
///
/// assert_eq!(registers.x3, 0xCAFE);
///
/// ```
///
impl Executor for LoadRegisterFromIndirectImmediateData {
    // ID: 0x0E
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let (segment, address) = registers.get_segmented_address();
        // TODO: Segment overflow?
        let offset_address = address + self.data.value;
        let address_to_read = (segment, offset_address).to_full_address();
        let read_value = mem.read_address(address_to_read);
        registers.set_at_index(self.data.register, read_value);
    }
}

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
impl Executor for LoadRegisterFromIndirectRegisterData {
    // ID: 0x0F
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let (segment, address) = registers.get_segmented_address();
        // TODO: Segment overflow?
        // TODO: Implement split load (double_register_target_type)
        let offset_address = address + registers.get_at_index(self.data.r2);
        let address_to_read = (segment, offset_address).to_full_address();
        let read_value = mem.read_address(address_to_read);
        registers.set_at_index(self.data.r1, read_value);
    }
}

///
/// Stores a 16 bit value from a register into a memory address.
///
/// The base address value is specified by the address registers (ah/al)
/// the first operand is the destination register and the operand is an
/// immediate offset to add to the base address value.
///
/// ```
/// use peripheral_cpu::instructions::definitions::{ImmediateInstructionData, LoadOffsetRegisterData};
/// use peripheral_cpu::registers::Registers;
/// use peripheral_cpu::executors::{Executor, set_comparison_result};
/// use peripheral_mem::new_memory_peripheral;
///
///
/// let storeRegisterFromIndirectImmediate = StoreRegisterToIndirectImmediateData {
///   data: ImmediateInstructionData {
///     r1: 0x02, // x3
///     value: 0x0001,
///     condition_flag: 0x00, // always
///   }
/// };
/// let mut registers = Registers { x3: 0xCAFE, ah: 0x1011, al: 0x1110, ..Registers::default() };
///
/// let mut mem = new_memory_peripheral();
/// mem.map_segment("TEST", 0x00110000, 0xFFFF, true);
///
/// storeRegisterFromIndirectImmediate.execute(&mut registers, &mem);
/// let stored_value = mem.read_address(0x00111111, 0xCAFE);
///
/// assert_eq!(stored_value, 0xCAFE);
///
/// ```
///
impl Executor for StoreRegisterToIndirectImmediateData {
    // ID: 0x12
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let (segment, address) = registers.get_segmented_address();
        // TODO: Segment overflow?
        let offset_address = address + self.data.value;
        let address_to_write = (segment, offset_address).to_full_address();
        mem.write_address(address_to_write, registers.get_at_index(self.data.register));
    }
}

///
/// Stores a 16 bit value from a register into a memory address.
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
/// let storeRegisterToIndirectRegister = StoreRegisterToIndirectRegisterData {
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
///
/// storeRegisterToIndirectRegister.execute(&mut registers, &mem);
/// let stored_value = mem.read_address(0x00111111, 0xCAFE);
///
/// assert_eq!(stored_value, 0xCAFE);
/// ```
///
impl Executor for StoreRegisterToIndirectRegisterData {
    // ID: 0x13
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let (segment, address) = registers.get_segmented_address();
        // TODO: Segment overflow?
        // TODO: Implement split store (double_register_target_type)
        let offset_address = address + registers.get_at_index(self.data.r2);
        let address_to_write = (segment, offset_address).to_full_address();
        mem.write_address(address_to_write, registers.get_at_index(self.data.r1));
    }
}

// Interrupts

impl Executor for WaitForInterruptInstructionData {
    // ID: 0x16
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        set_sr_bit(StatusRegisterFields::WaitingForInterrupt, registers);
    }
}

impl Executor for ReturnFromInterruptData {
    // ID: 0x17
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        return_from_interrupt(registers, mem);
    }
}

impl Executor for TriggerSoftwareInterruptData {
    // ID: 0x18
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        // Instruction immediate operand is truncated to 7 bits and used as the vector offset
        // for the exception. In effect this means that 128 user exceptions can be defined
        let vector_offset = (self.data.value & 0x0000007F) as u8;
        jump_to_interrupt(vector_offset, registers, mem);
    }
}

impl Executor for DisableInterruptsData {
    // ID: 0x19
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        // No interrupt is associated seventh priority level so it effectively
        // masks all interrupts (except NMI)
        set_interrupt_mask(registers, 0b0111);
    }
}

impl Executor for EnableInterruptsData {
    // ID: 0x1A
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        // There is no level zero interrupt, so if the mask is zero,
        // no interrupts are masked
        set_interrupt_mask(registers, 0);
    }
}

// Subroutines

impl Executor for BranchToSubroutineData {
    // ID: 0x1B
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        // Even though it is only a short jump and we only need a partial address
        // to return, we push the entire register to be consistent and avoid issues
        // if the system mode bit is flipped during a subroutine
        push_address_to_stack(registers, mem, registers.get_full_pc_address());
        // TODO: Segment overflow?
        let effective_address = registers.pl + self.data.value;
        registers.pl = effective_address;
    }
}

impl Executor for ShortJumpToSubroutineData {
    // ID: 0x1C
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        // Even though it is only a short jump and we only need a partial address
        // to return, we push the entire register to be consistent and avoid issues
        // if the system mode bit is flipped during a subroutine
        push_address_to_stack(registers, mem, registers.get_full_pc_address());
        registers.pl = registers.al;
    }
}

// Note: Privileged as stack could be overwritten to jump anywhere
impl Executor for LongJumpToSubroutineData {
    // ID: 0x1D
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        push_address_to_stack(registers, mem, registers.get_full_pc_address());
        (registers.ph, registers.pl) = registers.get_segmented_address();
    }
}

impl Executor for ReturnFromSubroutineData {
    // ID: 0x1E
    fn execute(&self, registers: &mut Registers, mem: &MemoryPeripheral) {
        let return_address = pop_address_from_stack(registers, mem);
        if sr_bit_is_set(StatusRegisterFields::SystemMode, registers) {
            (registers.ph, registers.pl) = return_address.to_segmented_address();
        } else {
            // Make sure that stack can't be used to do a unprivileged long jump
            (_, registers.pl) = return_address.to_segmented_address();
        }
    }
}

// Shifts

impl Executor for LogicalShiftLeftInstructionData {
    // ID: 0x1F
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = val_1.overflowing_shl(val_2 as u32);
        set_alu_bits(registers, result, carry, None);

        registers.set_at_index(self.data.r1, result);
    }
}

impl Executor for LogicalShiftRightInstructionData {
    // ID: 0x20
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = val_1.overflowing_shr(val_2 as u32);
        set_alu_bits(registers, result, carry, None);

        registers.set_at_index(self.data.r1, result);
    }
}

impl Executor for ArithmeticShiftLeftInstructionData {
    // ID: 0x21
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);

        let (result, carry) = val_1.overflowing_shl(val_2 as u32);

        // There can never be an overflow bit set because the sign bit is preserved
        set_alu_bits(registers, result, carry, None);

        registers.set_at_index(self.data.r1, result);
    }
}

impl Executor for ArithmeticShiftRightInstructionData {
    // ID: 0x22
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        let sign_bit = val_1 & 0x8000;

        // TODO: Check this - I think logical/arithmetic shift right are different
        // but arithmetic right shift can be simulated with:
        // - Copy sign bit into carry
        // - Rotate right
        // See: https://stackoverflow.com/a/59658066/1153203
        // But better study the 68k because it has separate instructions
        let (result, carry) = val_1.overflowing_shr(val_2 as u32);
        let signed_result = result | sign_bit;

        // There can never be an overflow bit set because the sign bit is preserved
        set_alu_bits(registers, signed_result, carry, None);

        registers.set_at_index(self.data.r1, signed_result);
    }
}

impl Executor for RotateLeftInstructionData {
    // ID: 0x23
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        let result = val_1.rotate_left(val_2 as u32);
        // Bit shifted out goes into carry (that's what the 68k does :shrug:)
        // TODO: Extract bit checking to function
        // TODO: Check the purpose of bit shifted out going to carry
        let carry = result & 1 == 1;
        set_alu_bits(registers, result, carry, None);

        registers.set_at_index(self.data.r1, result);
    }
}

impl Executor for RotateRightInstructionData {
    // ID: 0x24
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let val_1 = registers.get_at_index(self.data.r1);
        let val_2 = registers.get_at_index(self.data.r2);
        let result = val_1.rotate_right(val_2 as u32);
        // Bit shifted out goes into carry (that's what the 68k does :shrug:)
        // TODO: Extract bit checking to function
        // TODO: Check the purpose of bit shifted out going to carry
        let carry = (result >> 15) & 1 == 1;
        set_alu_bits(registers, result, carry, None);

        registers.set_at_index(self.data.r1, result);
    }
}

// Misc

impl Executor for NoOperationInstructionData {
    // ID: 0x26
    fn execute(&self, _registers: &mut Registers, _mem: &MemoryPeripheral) {}
}

impl Executor for ClearAluStatusInstructionData {
    // ID: 0x27
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let mask: u16 = StatusRegisterFields::Carry as u16
            | StatusRegisterFields::Negative as u16
            | StatusRegisterFields::Overflow as u16
            | StatusRegisterFields::Zero as u16;

        registers.sr &= !mask;
    }
}

// Byte Manipulation

///
/// Splits a 16 bit word stored in a register into two registers.
///
/// The first operand is the register for the upper byte,
/// the second operand is the register for the lower byte,
/// and the third operand is the source register.
///
/// The higher byte of each of the target registers is always zero
///
impl Executor for SplitWordInstructionData {
    // ID: 0x27
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let combined = registers.get_at_index(self.data.r3);
        let upper = (combined >> u8::BITS) & 0x00FF;
        let lower = combined & 0x00FF;
        registers.set_at_index(self.data.r1, upper);
        registers.set_at_index(self.data.r2, lower);
    }
}

///
/// Joins two registers storing a byte each into a single 16 bit word.
///
/// The first operand is the register where the combined word will be stored,
/// the second operand is the register for the upper byte,
/// and the third operand is the register for the lower byte.
///
/// The upper bytes of the source registers are ignored.
///
impl Executor for JoinWordInstructionData {
    // ID: 0x28
    fn execute(&self, registers: &mut Registers, _mem: &MemoryPeripheral) {
        let upper = registers.get_at_index(self.data.r1) & 0x00FF;
        let lower = registers.get_at_index(self.data.r2) & 0x00FF;
        let combined = upper << u8::BITS | lower;
        registers.set_at_index(self.data.r3, combined);
    }
}
