use std::any::Any;

use log::trace;

use crate::{
    conversion::{bytes_to_words, words_to_bytes},
    device::{BusAssertions, BusOperation, Device},
};

// The first word is generally used as a "chip select" and the second word is the address used by the device
pub const ADDRESS_MASK: u32 = 0x0000_FFFF;

#[allow(clippy::cast_possible_truncation)]
pub trait MemoryMappedDevice: Device {
    /// TODO: Eventually maybe we should simulate a bus with address/data/control lines
    fn read_address(&self, address: u32) -> u16;
    fn write_address(&mut self, address: u32, value: u16);

    /// Shortcut to get data out of the device fast for testing/debugging
    /// Does not represent anything in hardware
    fn read_raw_bytes(&self, limit: u32) -> Vec<u8> {
        // This default implementation will probably be very slow for devices that store a lot of data
        // You'll probably want to write a specialisation for them. It will be fine for devices that just have a
        // few registers.
        let words: Vec<u16> = (0..limit)
            .map(|address| self.read_address(address))
            .collect();
        words_to_bytes(&words)
    }
    /// Shortcut to get data into the device fast for testing/debugging
    /// Does not represent anything in hardware
    fn write_raw_bytes(&mut self, binary_data: &[u8]) {
        // This default implementation will probably be very slow for devices that store a lot of data
        // You'll probably want to write a specialisation for them. It will be fine for devices that just have a
        // few registers.
        let words = bytes_to_words(binary_data);
        for (address, word) in words.iter().enumerate() {
            self.write_address(address as u32, *word);
        }
    }

    fn perform_bus_io(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
        if selected {
            let address = bus_assertions.address & ADDRESS_MASK;
            match bus_assertions.op {
                BusOperation::Read => BusAssertions {
                    data: self.read_address(address),
                    device_was_activated: true,
                    ..BusAssertions::default()
                },
                BusOperation::Write => {
                    self.write_address(address, bus_assertions.data);
                    BusAssertions {
                        device_was_activated: true,
                        ..BusAssertions::default()
                    }
                }
            }
        } else {
            BusAssertions::default()
        }
    }
}

pub struct StubMemoryMappedDevice {
    data: Vec<u16>,
}

impl Device for StubMemoryMappedDevice {
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
        self.perform_bus_io(bus_assertions, selected)
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl MemoryMappedDevice for StubMemoryMappedDevice {
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
        trace!("write_raw_bytes: len: {}", binary_data.len());
        let words = bytes_to_words(binary_data);
        self.data[..words.len()].copy_from_slice(words.as_slice());
    }
}

#[must_use]
pub fn new_stub_memory_mapped_device() -> StubMemoryMappedDevice {
    StubMemoryMappedDevice {
        data: vec![0; u16::MAX as usize + 1],
    }
}
