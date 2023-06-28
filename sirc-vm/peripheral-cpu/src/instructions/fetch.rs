use peripheral_mem::MemoryPeripheral;

use crate::registers::SegmentedAddress;

#[must_use]
pub fn fetch_instruction(mem: &MemoryPeripheral, pc: (u16, u16)) -> [u8; 4] {
    // Only the CPU knows that the address is split into two 16 bit registers
    // Any other peripheral will only see the 24 address lines
    let full_address: u32 = pc.to_full_address();

    // TODO: Alignment check?
    // TODO: Do we need to copy here?
    let [b1, b2] = u16::to_be_bytes(mem.read_address(full_address).to_owned());
    let [b3, b4] = u16::to_be_bytes(mem.read_address(full_address + 1).to_owned());
    [b2, b1, b4, b3]
}
