use std::{
    any::Any,
    cell::RefCell,
    fs::{File, OpenOptions},
    path::PathBuf,
};

use memmap::{MmapMut, MmapOptions};
use peripheral_bus::{
    device::BusAssertions, device::Device, memory_mapped_device::MemoryMappedDevice,
};

// The first word is generally used as a "chip select" and the second word is the address used by the device
const ADDRESS_MASK: u32 = 0x0000_FFFF;

pub enum SegmentMemCell {
    // At the moment, all raw segments get the maximum allowable of memory allocated
    // for a single segment (16 bit address). This is wasteful but not a huge issue
    // at the moment running on a machine with GBs of memory
    RawMemory(Box<[u8; (0xFFFF * 2) + 2]>),
    FileMapped(Box<File>, Box<MmapMut>),
}

pub struct RamDevice {
    // TODO: Does this still need to be RefCell?
    pub mem_cell: RefCell<SegmentMemCell>,
}

#[must_use]
pub fn new_ram_device_standard() -> RamDevice {
    RamDevice {
        mem_cell: RefCell::new(SegmentMemCell::RawMemory(Box::new([0; (0xFFFF * 2) + 2]))),
    }
}

pub fn new_ram_device_file_mapped(file_path: PathBuf) -> RamDevice {
    // TODO: Proper error handling?
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(file_path)
        .unwrap();
    // TODO: Proper error handling here too?
    let mmap = unsafe { MmapOptions::new().map_mut(&file).unwrap() };

    RamDevice {
        mem_cell: RefCell::new(SegmentMemCell::FileMapped(Box::new(file), Box::new(mmap))),
    }
}
impl Device for RamDevice {
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
        if !selected {
            return BusAssertions::default();
        }
        let address = bus_assertions.address & ADDRESS_MASK;
        match bus_assertions.op {
            peripheral_bus::device::BusOperation::Read => {
                println!(
                    "Reading 0x{:X} from address 0x{:X}",
                    self.read_address(address),
                    address
                );
                BusAssertions {
                    data: self.read_address(address),
                    ..BusAssertions::default()
                }
            }
            peripheral_bus::device::BusOperation::Write => {
                self.write_address(address, bus_assertions.data);
                BusAssertions::default()
            }
        }
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl MemoryMappedDevice for RamDevice {
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

    fn read_raw_bytes(&self, limit: u32) -> Vec<u8> {
        let cell = self.mem_cell.borrow();

        let raw_memory: &[u8] = match *cell {
            SegmentMemCell::RawMemory(ref mem) => &mem[..],
            SegmentMemCell::FileMapped(_, ref mmap) => &mmap[..],
        };

        // TODO: Is this efficient in rust? Does it get optimised?
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
