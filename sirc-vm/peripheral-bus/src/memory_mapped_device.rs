use crate::conversion::{bytes_to_words, words_to_bytes};

pub trait MemoryMappedDevice {
    /// Called every clock so the device can do work and raise interrupts etc.
    /// TODO: Allow device to return a value(s) to assert bus lines (e.g. interrupts)
    ///       A return value will avoid having to pass in the parent pmem/bus and cause circular dependencies
    fn poll(&mut self);

    /// TODO: Eventually maybe we should simulate a bus with address/data/control lines
    fn read_address(&self, address: u32) -> u16;
    fn write_address(&mut self, address: u32, value: u16);

    /// Shortcut to get data out of the device fast for testing/debugging
    /// Does not represent anything in hardware
    fn read_raw_bytes(&self, limit: u32) -> Vec<u8>;
    /// Shortcut to get
    fn write_raw_bytes(&mut self, binary_data: &[u8]);
}

pub struct StubMemoryMappedDevice {
    data: Vec<u16>,
}

impl MemoryMappedDevice for StubMemoryMappedDevice {
    fn poll(&mut self) {
        // Nothing
    }

    fn read_address(&self, address: u32) -> u16 {
        self.data[address as usize]
    }

    fn write_address(&mut self, address: u32, value: u16) {
        self.data[address as usize] = value;
    }

    fn read_raw_bytes(&self, limit: u32) -> Vec<u8> {
        let words = &self.data[..limit as usize];
        words_to_bytes(words)
    }

    fn write_raw_bytes(&mut self, binary_data: &[u8]) {
        let words = bytes_to_words(binary_data);
        self.data[..words.len()].copy_from_slice(words.as_slice());
    }
}

#[must_use]
pub fn new_stub_memory_mapped_device() -> StubMemoryMappedDevice {
    StubMemoryMappedDevice {
        data: vec![0; u16::MAX as usize],
    }
}
