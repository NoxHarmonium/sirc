use crate::registers::{
    AddressRegisterIndexing, FullAddress, RegisterIndexing, Registers, SegmentedAddress,
    SegmentedRegisterAccess,
};

/// Extends an 8 bit value that is signed to a 16 bit value
///
/// Most of the values we pass around in the virtual machine are represented as unsigned
/// (even if they are technically 2's compliment signed)
/// This is fine most of the time, however, if we want to cast a smaller value to a
/// larger value, if the compiler doesn't know that the bits actually represent a
/// 2's complement number, it will fill the new space with zeros which will cause
/// the number's value to change.
///
///
/// ```
/// use peripheral_cpu::microcode::address::sign_extend_small_offset;
///
/// assert_eq!(sign_extend_small_offset(0xFF), 0xFFFF);
/// assert_eq!(sign_extend_small_offset(0x00), 0x0000);
/// ```
///
pub fn sign_extend_small_offset(small_offset: u8) -> u16 {
    // 1. Convert to i8 so the compiler knows we are dealing with a signed number
    // 2. Extend it to i16, the compiler knows it is signed so it preserves the sign
    // 3. Cast it back to unsigned which we deal with
    ((small_offset as i8) as i16) as u16
}

pub fn calculate_effective_address_with_pc_offset(registers: &Registers, offset: u16) -> u32 {
    let (h, l) = registers.get_segmented_pc();
    // TODO: Segment overflow?
    let new_l = l.wrapping_add(offset);
    (h, new_l).to_full_address()
}

pub fn calculate_effective_address_with_immediate(
    registers: &Registers,
    address_register_index: u8,
    immediate_value: u16,
) -> u32 {
    let (h, l) = registers
        .get_address_register_at_index(address_register_index)
        .to_segmented_address();
    // TODO: Segment overflow?
    let new_l = l.wrapping_add(immediate_value);
    (h, new_l).to_full_address()
}

pub fn calculate_effective_address_with_register(
    registers: &Registers,
    address_register_index: u8,
    register_index: u8,
) -> u32 {
    let (h, l) = registers
        .get_address_register_at_index(address_register_index)
        .to_segmented_address();

    let offset_value = registers.get_at_index(register_index);
    // TODO: Segment overflow?
    let new_l = l.wrapping_add(offset_value);

    (h, new_l).to_full_address()
}
