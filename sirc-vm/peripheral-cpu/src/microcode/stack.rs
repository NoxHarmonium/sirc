use peripheral_mem::MemoryPeripheral;

use crate::registers::{
    FullAddress, FullAddressRegisterAccess, Registers, SegmentedRegisterAccess,
};

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
