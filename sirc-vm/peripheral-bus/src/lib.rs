#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    // I don't like this rule
    clippy::module_name_repetitions,
    // Will tackle this at the next clean up
    clippy::too_many_lines,
    // Might be good practice but too much work for now
    clippy::missing_errors_doc,
    // Not stable yet - try again later
    clippy::missing_const_for_fn,
    // I have a lot of temporary panics for debugging that will probably be cleaned up
    clippy::missing_panics_doc
)]
#![deny(warnings)]

// Addressing is technically only 24 bit
// Only use the 24 bits to match segments
const ADDRESS_MASK: u32 = 0x00FF_FFFF;

pub mod conversion;
pub mod device;
pub mod helpers;
pub mod memory_mapped_device;

use std::cell::RefCell;
use std::fs::read;
use std::path::Path;

use device::{BusAssertions, Device};
use log::{debug, warn};
use memory_mapped_device::MemoryMappedDevice;

pub struct Segment {
    pub label: String,
    pub address: u32,
    pub size: u32,
    pub writable: bool,
    device: RefCell<Box<dyn MemoryMappedDevice>>,
}

impl Segment {
    fn address_is_in_segment_range(&self, address: u32) -> bool {
        // TODO: Warn about address masking?
        let masked_segment_address = self.address & ADDRESS_MASK;
        let masked_input_address = address & ADDRESS_MASK;
        masked_input_address >= masked_segment_address
            && masked_input_address < masked_segment_address + self.size
    }
}

pub struct BusPeripheral {
    assertions: BusAssertions,
    pub bus_master: Box<dyn Device>,
    segments: Vec<Segment>,
}

#[must_use]
pub fn new_bus_peripheral(bus_master: Box<dyn Device>) -> BusPeripheral {
    BusPeripheral {
        assertions: BusAssertions::default(),
        bus_master,
        segments: vec![],
    }
}

impl BusPeripheral {
    #[must_use]
    pub fn get_segment_for_label(&self, label: &str) -> Option<&Segment> {
        self.segments.iter().find(|s| s.label == label)
    }

    #[must_use]
    pub fn get_segment_for_address(&self, address: u32) -> Option<&Segment> {
        // TODO: More efficient way to simulate memory mapping? E.g. range map
        self.segments
            .iter()
            .find(|s| s.address_is_in_segment_range(address))
    }

    pub fn map_segment(
        &mut self,
        label: &str,
        address: u32,
        size: u32,
        writable: bool,
        device: Box<dyn MemoryMappedDevice>,
    ) {
        debug!(
            "Map segment {} from 0x{:08x} to 0x{:08x}",
            label,
            address,
            address + size
        );

        self.segments.push(Segment {
            label: String::from(label),
            address,
            size,
            writable,
            device: RefCell::new(device),
        });
    }

    /// Loads data from a path into a segment
    ///
    /// # Panics
    /// Will panic if there was an error loading the binary data
    pub fn load_binary_data_into_segment_from_file(&mut self, label: &str, path: &Path) {
        let maybe_binary_data = read(path);
        match maybe_binary_data {
            Ok(binary_data) => {
                self.load_binary_data_into_segment(label, &binary_data);
            }
            Err(error) => {
                panic!(
                    "Could not load binary data from {} ({})",
                    path.display(),
                    error
                );
            }
        }
    }

    /// Loads raw binary data into a segment
    ///
    /// # Panics
    /// Will panic if the specified segment is not found, or if the binary data will not fit in the segment
    pub fn load_binary_data_into_segment(&self, label: &str, binary_data: &Vec<u8>) {
        let maybe_segment = self.get_segment_for_label(label);

        let (segment_size, device) = maybe_segment.map_or_else(
            || panic!("Could not find segment with name: {label}"),
            |segment| (segment.size, &segment.device),
        );

        assert!(
            binary_data.len() <= ((segment_size + 1) * 2) as usize,
            "Loaded binary data is {} bytes long but segment has size of {} words",
            binary_data.len(),
            segment_size
        );

        device.borrow_mut().write_raw_bytes(binary_data);
    }

    /// Dumps a segment to raw binary data
    ///
    /// # Panics
    /// Will panic if the specified segment is not found
    #[must_use]
    pub fn dump_segment(&self, label: &str) -> Vec<u8> {
        let maybe_segment = self.get_segment_for_label(label);

        let (segment_size, device) = maybe_segment.map_or_else(
            || panic!("Could not find segment with name: {label}"),
            |segment| (segment.size, &segment.device),
        );

        device.borrow_mut().read_raw_bytes(segment_size)
    }

    /// Reads a single 16 bit value out of a memory address
    ///
    /// # Panics
    /// Will panic if the segment is in use (unlikely) or if the internal address calculation goes out of bounds.
    #[must_use]
    pub fn read_address(&self, address: u32) -> u16 {
        self.get_segment_for_address(address).map_or_else(
            || {
                warn!(
                "Warning: No segment mapped to address 0x{address:08x}. Value read will always be 0x0000"
            );
                // If a segment isn't mapped, the address just maps to nothing
                0x0000
            },
            |segment| {
                let relative_address = address - segment.address;

                segment.device.borrow().read_address(relative_address)
            },
        )
    }

    /// Writes a single 16 bit value into  a memory address
    ///
    /// ```
    /// use peripheral_bus::new_bus_peripheral;
    /// use peripheral_bus::device::new_stub_device;
    /// use peripheral_bus::memory_mapped_device::new_stub_memory_mapped_device;
    ///
    /// let stub_master = new_stub_device();
    /// let mut mem = new_bus_peripheral(Box::new(stub_master));
    /// mem.map_segment("doctest", 0x00F0_0000, 0xFFFF, true, Box::new(new_stub_memory_mapped_device()));
    /// let address = 0x00F0_CAFE;
    /// let value = 0xFEAB;
    /// mem.write_address(address, value);
    /// assert_eq!(mem.read_address(address), value);
    /// ```
    ///
    /// # Panics
    /// Will panic if the segment is in use (unlikely), the segment is readonly or if the internal address calculation goes out of bounds.
    pub fn write_address(&self, address: u32, value: u16) {
        self.get_segment_for_address(address).map_or_else(|| {
             // If a segment isn't mapped, the value just goes into a black hole
             warn!(
                "Warning: No segment mapped to address 0x{address:08x}. Value will be ignored (not written)"
            );
        } , |segment| {
            assert!(
                segment.writable,
                "Segment {} is read-only and cannot be written to",
                segment.label
            );

            let relative_address = address - segment.address;
            segment.device.borrow_mut().write_address(relative_address , value);
        });
    }

    ///
    /// Runs each device, and then combines all their bus assertions into a single one.
    ///
    #[must_use]
    pub fn poll_all(&mut self) -> BusAssertions {
        let assertions = self.assertions;

        // TODO:: Assert no conflicts (e.g. two devices asserting the address or data bus at the same time)

        let master_assertions = self.bus_master.poll(assertions, true);
        println!("Master assertions: {master_assertions:X?}");

        let segments = &self.segments;
        let merged_assertions = segments.iter().fold(master_assertions, |prev, segment| {
            let selected = segment.address_is_in_segment_range(prev.address);
            println!(
                "selected: {selected} address 0x{:X} label: {} SA: 0x{:X}",
                prev.address, segment.label, segment.address
            );

            let mut device = segment.device.borrow_mut();
            let assertions = device.poll(prev, selected);
            BusAssertions {
                // Interrupts are all merged together
                interrupt_assertion: prev.interrupt_assertion | assertions.interrupt_assertion,
                // If at least one device has a bus error, then a fault will be raised
                // The devices will have to be polled by the program to find the cause of the error at the moment
                // (I don't really want to implement complex error signalling like the 68k has)
                bus_error: prev.bus_error | assertions.bus_error,
                data: if selected { assertions.data } else { prev.data },
                ..prev
            }
        });
        self.assertions = merged_assertions;
        merged_assertions
    }

    /// Runs the CPU for six cycles. Only to keep tests functioning at the moment. Will be removed
    ///
    /// # Panics
    /// Will panic if a coprocessor instruction is executed with a COP ID of neither 0 or 1
    // #[cfg(test)]
    pub fn run_full_cycle(&mut self, max_cycles: u32) -> BusAssertions {
        let mut cycle_count = 0;
        loop {
            let result = self.poll_all();
            cycle_count += 1;
            if cycle_count >= max_cycles {
                return result;
            }
        }
    }
}
