#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    // I don't like this rule
    clippy::module_name_repetitions,
    // Will tackle this at the next clean up
    clippy::too_many_lines,
    // Might be good practice but too much work for now
    clippy::missing_errors_doc,
    // Not stable yet - try again later
    clippy::missing_const_for_fn
)]

use std::cell::RefCell;
use std::fs::read;
use std::path::PathBuf;

pub struct Segment {
    pub label: String,
    pub address: u32,
    pub size: u32,
    pub writable: bool,
    // At the moment, all segments get the maximum allowable of memory allocated
    // for a single segment (16 bit address). This is wasteful but not a huge issue
    // at the moment running on a machine with GBs of memory
    mem_cell: RefCell<[u16; 65536]>,
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
            .find(|s| address >= s.address && address <= s.address + s.size)
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
            mem_cell: RefCell::new([0; 65536]),
        });
    }

    /// Loads data from a path into a segment
    ///
    /// # Panics
    /// Will panic if there was an error loading the binary data
    pub fn load_binary_data_into_segment_from_file(&self, label: &str, path: &PathBuf) {
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
            binary_data.len() <= segment_size as usize,
            "Loaded binary data is {} bytes long but segment has size of {}",
            binary_data.len(),
            segment_size
        );
        // Convert bytes to words
        let mut mem = mem_cell.borrow_mut();
        for i in 0..binary_data.len() / 2 {
            let destination_address = i;
            let source_address = i * 2;
            let data =
                u16::from_be_bytes([binary_data[source_address + 1], binary_data[source_address]]);
            mem[destination_address] = data;
        }
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

        let segment_data = mem_cell.borrow();
        // TODO: Is this efficient in rust? Does it get optimised?
        segment_data
            .iter()
            .take(segment_size as usize)
            .flat_map(|&word| u16::to_be_bytes(word))
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
                "Warning: No segment mapped to address 0x{address:08x}. Value will always be 0x0000"
            );
                // If a segment isn't mapped, the address just maps to nothing
                0x0000
            },
            |segment| {
                // Range check?
                let mem = segment.mem_cell.borrow();
                mem.get(address as usize - segment.address as usize)
                    .unwrap()
                    .to_owned()
            },
        )
    }

    /// Writes a single 16 bit value into  a memory address
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
            let mut mem = segment.mem_cell.borrow_mut();
            mem[address as usize - segment.address as usize] = value;
        });
    }
}
