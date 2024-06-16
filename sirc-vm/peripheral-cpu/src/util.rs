///
/// Finds the highest active bit in a bit field of mixed flags.
///
/// E.g. if bits 3, 2 and 0 were set, the highest active bit is 4 (e.g 0xC would be 0x8)
///
/// ```
/// use peripheral_cpu::util::find_highest_active_bit;
/// use assert_hex::assert_eq_hex;
///
/// assert_eq_hex!(0x00, find_highest_active_bit(0x00));
/// assert_eq_hex!(0x08, find_highest_active_bit(0x0D));
/// assert_eq_hex!(0x02, find_highest_active_bit(0x03));
/// assert_eq_hex!(0x10, find_highest_active_bit(0x11));
/// assert_eq_hex!(0x08, find_highest_active_bit(0x0F));
/// assert_eq_hex!(0x40, find_highest_active_bit(0x52));
/// assert_eq_hex!(0x80, find_highest_active_bit(0xFF));
/// ```
pub fn find_highest_active_bit(bit_field: u8) -> u8 {
    // TODO: Confirm `find_highest_active_bit` is using the best algorithm
    // category=Performance
    // It seems like a common operation that would have an efficient algorithm but for now this will do and it can be improved later

    let mut last_value = 0x0;
    let mut mask: u8 = 0xFE;
    let mut intermediate = bit_field;
    while intermediate > 0 {
        last_value = intermediate;
        intermediate &= mask;
        mask <<= 1;
    }

    last_value
}
