use std::{
    any::Any,
    cell::RefCell,
    fs::{File, OpenOptions},
    path::PathBuf,
};

use log::{trace, warn};
use memmap::{MmapMut, MmapOptions};
use peripheral_bus::memory_mapped_device::MemoryMapped;
use peripheral_bus::{
    device::BusAssertions, device::Device, memory_mapped_device::MemoryMappedDevice,
};

pub enum SegmentMemCell {
    // At the moment, all raw segments get the maximum allowable of memory allocated
    // for a single segment (16 bit address). This is wasteful but not a huge issue
    // at the moment running on a machine with GBs of memory
    RawMemory(Box<[u8; (0xFFFF * 2) + 2]>),
    FileMapped(Box<File>, Box<MmapMut>),
}

pub struct RamDevice {
    // TODO: Does this still need to be RefCell?
    // category=Refactoring
    pub mem_cell: RefCell<SegmentMemCell>,
    /// How many clock cycles a bus access takes (1 = respond immediately, 2+ = wait states).
    access_latency_clocks: u32,
    /// Saved bus assertions from when the current operation started. `Some` means an operation
    /// is in progress and the countdown runs every poll regardless of external bus state.
    active_request: Option<BusAssertions>,
    clocks_remaining: u32,
}

#[must_use]
pub fn new_ram_device_standard() -> RamDevice {
    new_ram_device_with_latency(1)
}

#[must_use]
pub fn new_ram_device_with_latency(access_latency_clocks: u32) -> RamDevice {
    assert!(
        access_latency_clocks >= 1,
        "access_latency_clocks must be at least 1"
    );
    RamDevice {
        mem_cell: RefCell::new(SegmentMemCell::RawMemory(Box::new([0; (0xFFFF * 2) + 2]))),
        access_latency_clocks,
        active_request: None,
        clocks_remaining: 0,
    }
}

/**
 * # Panics
 *
 * Will throw if the file can't be opened for whatever reason.
 * See the open method of `OpenOptions` for possible errors
 */
#[must_use]
pub fn new_ram_device_file_mapped(file_path: PathBuf) -> RamDevice {
    // TODO: Remove `unwrap` statements
    // category=Refactoring
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(file_path)
        .unwrap();
    let mmap = unsafe { MmapOptions::new().map_mut(&file).unwrap() };

    RamDevice {
        mem_cell: RefCell::new(SegmentMemCell::FileMapped(Box::new(file), Box::new(mmap))),
        access_latency_clocks: 1,
        active_request: None,
        clocks_remaining: 0,
    }
}

impl Device for RamDevice {
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
        // TODO: What happens on a reset from the bus? Discard request?

        if selected && bus_assertions.bus_access_strobe && self.active_request.is_none() {
            trace!("Starting new request: assertions: {:?}", bus_assertions);
            self.active_request = Some(bus_assertions);
            self.clocks_remaining = self.access_latency_clocks - 1;
        }

        if let Some(saved) = self.active_request {
            if bus_assertions.bus_acknowledge {
                warn!(
                    "Another device is asserting BACK. \
                    Are there two conflicting devices on the bus? \
                    Previous request will be discarded. \
                    Active Request: {:?} \
                    Current Bus Assertions: {:?}",
                    self.active_request, bus_assertions
                );
                self.active_request = None;
            } else if self.clocks_remaining == 0 {
                trace!("clocks_remaining == 0, finishing request...");

                self.active_request = None;
                return self.perform_bus_io(saved, true);
            } else {
                self.clocks_remaining -= 1;
            }
            BusAssertions {
                device_was_activated: true,
                ..BusAssertions::default()
            }
        } else {
            BusAssertions::default()
        }
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl MemoryMapped for RamDevice {
    fn read_address(&self, address: u32) -> u16 {
        let address_pointer = address as usize * 2;

        // Range check?
        let cell = self.mem_cell.borrow();

        let raw_memory: &[u8] = match *cell {
            SegmentMemCell::RawMemory(ref mem) => &mem[..],
            SegmentMemCell::FileMapped(_, ref mmap) => &mmap[..],
        };

        let byte_pair: [u8; 2] = raw_memory[address_pointer..=address_pointer + 1]
            .try_into()
            .unwrap();
        u16::from_be_bytes(byte_pair)
    }
    fn write_address(&mut self, address: u32, value: u16) {
        let address_pointer = address as usize * 2;
        let byte_pair: [u8; 2] = u16::to_be_bytes(value);

        let mut cell = self.mem_cell.borrow_mut();

        let raw_memory: &mut [u8] = match *cell {
            SegmentMemCell::RawMemory(ref mut mem) => &mut mem[..],
            SegmentMemCell::FileMapped(_, ref mut mmap) => &mut mmap[..],
        };
        raw_memory[address_pointer] = byte_pair[0];
        raw_memory[address_pointer + 1] = byte_pair[1];
    }
}

impl MemoryMappedDevice for RamDevice {
    fn read_raw_bytes(&self, limit: u32) -> Vec<u8> {
        let cell = self.mem_cell.borrow();

        let raw_memory: &[u8] = match *cell {
            SegmentMemCell::RawMemory(ref mem) => &mem[..],
            SegmentMemCell::FileMapped(_, ref mmap) => &mmap[..],
        };

        // TODO: Is this efficient in rust? Does it get optimised?
        // category=Performance
        raw_memory
            .iter()
            .take(limit as usize * 2)
            .copied()
            .collect()
    }
    fn write_raw_bytes(&mut self, binary_data: &[u8]) {
        let mut cell = self.mem_cell.borrow_mut();
        let raw_memory: &mut [u8] = match *cell {
            SegmentMemCell::RawMemory(ref mut mem) => &mut mem[..],
            SegmentMemCell::FileMapped(_, ref mut mmap) => &mut mmap[..],
        };

        raw_memory[0..binary_data.len()].copy_from_slice(binary_data);
    }
}
