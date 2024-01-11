use crate::{conversion::bytes_to_words, MemoryPeripheral};
use num::Integer;

///
/// Writes a slice of contiguous bytes to memory at the given address.
///
/// You would think this would be a simple operation but there is a lot of byte/word/endian marshalling
/// as usual. It is not currently part of the `MemoryPeripheral` API because I wanted to keep that as close
/// to the hardware as possible (one word read/write at a time). I'm not sure if that is actually wise
/// and it might change in the future
///
/// ```
/// use peripheral_mem::helpers::write_bytes;
/// use peripheral_mem::new_memory_peripheral;
/// use peripheral_mem::memory_mapped_device::new_stub_memory_mapped_device;
///
/// let mut mem = new_memory_peripheral();
/// mem.map_segment("doctest", 0x00F0_0000, 0xFFFF, true, Box::new(new_stub_memory_mapped_device()));
///
/// let bytes = "abcd".as_bytes();
/// write_bytes(&mem, 0x00F0_00FF, bytes);
///
/// // Check ASCII byte codes for "abcd"
/// assert_eq!(mem.read_address(0x00F0_00FF), 0x6162); // a
/// assert_eq!(mem.read_address(0x00F0_0100), 0x6364); // a + 1
/// ```
///
/// # Panics
/// Will panic if an odd number of bytes are provided
/// Will panic if the bytes don't fit into a 32 bit address space
/// Will panic if there is a logic error in the function which causes a misalignment of bytes to words
pub fn write_bytes(memory_peripheral: &MemoryPeripheral, start_address: u32, bytes: &[u8]) {
    let byte_count = bytes.len();
    assert!(
        !byte_count.is_odd(),
        "Can only write an even number of bytes (memory has 16 bit bus), got [{byte_count}] bytes",
    );
    bytes_to_words(bytes)
        .iter()
        .enumerate()
        .for_each(|(index, decoded)| {
            let truncated_index: u32 = index
                .try_into()
                .expect("Expected index {index} to fit within a u32 address space");
            let calculated_address = start_address
                .checked_add(truncated_index)
                .expect("Bytes do not fit into a u32 address space when offset by start_address");
            memory_peripheral.write_address(calculated_address, *decoded);
        });
}
