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

pub mod conversion;
pub mod helpers;

use std::cell::RefCell;
use std::fs::{read, File, OpenOptions};
use std::path::Path;

use memmap::{MmapMut, MmapOptions};

pub enum SegmentMemCell {
    // At the moment, all raw segments get the maximum allowable of memory allocated
    // for a single segment (16 bit address). This is wasteful but not a huge issue
    // at the moment running on a machine with GBs of memory
    RawMemory(Box<[u8; 0xFFFF * 2]>),
    FileMapped(Box<File>, Box<MmapMut>),
}

pub struct Segment {
    pub label: String,
    pub address: u32,
    pub size: u32,
    pub writable: bool,
    pub mem_cell: RefCell<SegmentMemCell>,
}

pub struct MemoryPeripheral {
    segments: Vec<Segment>,
}

#[must_use]
pub fn new_memory_peripheral() -> MemoryPeripheral {
    MemoryPeripheral { segments: vec![] }
}

impl MemoryPeripheral {
    #[must_use]
    pub fn get_segment_for_label(&self, label: &str) -> Option<&Segment> {
        self.segments
            .iter()
            .find(|s| s.label == label)
            .map(|s| s.to_owned())
    }

    #[must_use]
    pub fn get_segment_for_address(&self, address: u32) -> Option<&Segment> {
        // TODO: More efficient way to simulate memory mapping? E.g. range map
        self.segments
            .iter()
            .find(|s| address >= s.address && address < s.address + s.size)
            .map(|s| s.to_owned())
    }

    pub fn map_segment(&mut self, label: &str, address: u32, size: u32, writable: bool) {
        println!(
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
            mem_cell: RefCell::new(SegmentMemCell::RawMemory(Box::new([0; 0xFFFF * 2]))),
        });
    }

    /// Memory maps a segment to a file.
    ///
    /// Useful to provide a basic low level interface with the outside world.
    ///
    /// # Panics
    ///
    /// Will panic if the given file cannot be opened
    ///
    pub fn map_segment_to_file(
        &mut self,
        label: &str,
        address: u32,
        size: u32,
        writable: bool,
        file_path: &Path,
    ) {
        println!(
            "Map segment {} from 0x{:08x} to 0x{:08x} to file {}",
            label,
            address,
            address + size,
            file_path.to_string_lossy()
        );

        // TODO: Proper error handling?
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(file_path)
            .unwrap();
        // TODO: Proper error handling here too?
        let mmap = unsafe { MmapOptions::new().map_mut(&file).unwrap() };

        self.segments.push(Segment {
            label: String::from(label),
            address,
            size,
            writable,
            mem_cell: RefCell::new(SegmentMemCell::FileMapped(Box::new(file), Box::new(mmap))),
        });
    }

    /// Loads data from a path into a segment
    ///
    /// # Panics
    /// Will panic if there was an error loading the binary data
    pub fn load_binary_data_into_segment_from_file(&self, label: &str, path: &Path) {
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

        let (segment_size, mem_cell) = maybe_segment.map_or_else(
            || panic!("Could not find segment with name: {label}"),
            |segment| (segment.size, &segment.mem_cell),
        );

        assert!(
            binary_data.len() <= (segment_size * 2) as usize,
            "Loaded binary data is {} bytes long but segment has size of {} words",
            binary_data.len(),
            segment_size
        );

        let mut cell = mem_cell.borrow_mut();
        let raw_memory: &mut [u8] = match *cell {
            SegmentMemCell::RawMemory(ref mut mem) => &mut mem[..],
            SegmentMemCell::FileMapped(_, ref mut mmap) => &mut mmap[..],
        };

        raw_memory[0..binary_data.len()].copy_from_slice(binary_data);
    }

    /// Dumps a segment to raw binary data
    ///
    /// # Panics
    /// Will panic if the specified segment is not found
    #[must_use]
    pub fn dump_segment(&self, label: &str) -> Vec<u8> {
        let maybe_segment = self.get_segment_for_label(label);

        let (segment_size, mem_cell) = maybe_segment.map_or_else(
            || panic!("Could not find segment with name: {label}"),
            |segment| (segment.size, &segment.mem_cell),
        );

        let cell = mem_cell.borrow();

        let raw_memory: &[u8] = match *cell {
            SegmentMemCell::RawMemory(ref mem) => &mem[..],
            SegmentMemCell::FileMapped(_, ref mmap) => &mmap[..],
        };

        // TODO: Is this efficient in rust? Does it get optimised?
        raw_memory
            .iter()
            .take(segment_size as usize * 2)
            .copied()
            .collect()
    }

    /// Reads a single 16 bit value out of a memory address
    ///
    /// # Panics
    /// Will panic if the segment is in use (unlikely) or if the internal address calculation goes out of bounds.
    #[must_use]
    pub fn read_address(&self, address: u32) -> u16 {
        self.get_segment_for_address(address).map_or_else(
            || {
                println!(
                "Warning: No segment mapped to address 0x{address:08x}. Value read will always be 0x0000"
            );
                // If a segment isn't mapped, the address just maps to nothing
                0x0000
            },
            |segment| {
                // Range check?
                let base_index = (address as usize - segment.address as usize) * 2;
                let cell = segment.mem_cell.borrow();

                let raw_memory: &[u8] = match *cell {
                    SegmentMemCell::RawMemory(ref mem) => {
                       &mem[..]
                    }
                    SegmentMemCell::FileMapped(_, ref mmap) => {
                        &mmap[..]
                    }
                };
                let byte_pair: [u8; 2] = raw_memory[base_index..=base_index + 1].try_into().unwrap();
                u16::from_be_bytes(byte_pair)
            },
        )
    }

    /// Writes a single 16 bit value into  a memory address
    ///
    /// ```
    /// use peripheral_mem::new_memory_peripheral;
    ///
    /// let mut mem = new_memory_peripheral();
    /// mem.map_segment("doctest", 0x00F0_0000, 0xFFFF, true);
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
             println!(
                "Warning: No segment mapped to address 0x{address:08x}. Value will be ignored (not written)"
            );
        } , |segment| {
            assert!(
                segment.writable,
                "Segment {} is read-only and cannot be written to",
                segment.label
            );
            let byte_pair: [u8; 2] =  u16::to_be_bytes(value);

            let base_index = (address as usize - segment.address as usize) * 2;
            let mut cell = segment.mem_cell.borrow_mut();

            let raw_memory: &mut [u8] = match *cell {
                SegmentMemCell::RawMemory(ref mut mem) => {
                    &mut mem[..]
                }
                SegmentMemCell::FileMapped(_, ref mut mmap) => {
                    &mut mmap[..]
                }
            };
            raw_memory[base_index] = byte_pair[0];
            raw_memory[base_index + 1] = byte_pair[1];
        });
    }
}
