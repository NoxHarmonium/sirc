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

pub fn new_memory_peripheral() -> MemoryPeripheral {
    MemoryPeripheral { segments: vec![] }
}

impl MemoryPeripheral {
    pub fn get_segment_for_label(&self, label: &str) -> Option<&Segment> {
        self.segments
            .iter()
            .find(|s| s.label == label)
            .map(|s| s.to_owned())
    }

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
        })
    }

    pub fn load_binary_data_into_segment(&self, label: &str, path: PathBuf) {
        let maybe_segment = self.get_segment_for_label(label);

        let (segment_size, mem_cell) = match maybe_segment {
            // TODO: Make this nicer. Borrow checker was complaining if matchers are nested
            Some(segment) => (segment.size, &segment.mem_cell),
            None => {
                panic!("Could not find segment with name: {}", label)
            }
        };

        let maybe_binary_data = read(&path);
        match maybe_binary_data {
            Ok(binary_data) => {
                if binary_data.len() > segment_size as usize {
                    panic!(
                        "Loaded binary data from {} is {} bytes long but segment has size of {}",
                        path.display(),
                        binary_data.len(),
                        segment_size
                    );
                }
                // Convert bytes to words
                let mut mem = mem_cell.borrow_mut();
                for i in 0..binary_data.len() / 2 {
                    let destination_address = i;
                    let source_address = i * 2;
                    let data = u16::from_le_bytes([
                        binary_data[source_address],
                        binary_data[source_address + 1],
                    ]);
                    mem[destination_address] = data;
                }
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

    pub fn read_address(&self, address: u32) -> u16 {
        if let Some(segment) = self.get_segment_for_address(address) {
            // Range check?
            let mem = segment.mem_cell.borrow();
            mem.get(address as usize - segment.address as usize)
                .unwrap()
                .to_owned()
        } else {
            println!(
                "Warning: No segment mapped to address 0x{:08x}. Value will always be 0x0000",
                address
            );
            // If a segment isn't mapped, the address just maps to nothing
            0x0000
        }
    }

    pub fn write_address(&self, address: u32, value: u16) {
        if let Some(segment) = self.get_segment_for_address(address) {
            if !segment.writable {
                panic!(
                    "Segment {} is read-only and cannot be written to",
                    segment.label
                )
            }
            let mut mem = segment.mem_cell.borrow_mut();
            mem[address as usize - segment.address as usize] = value;
        } else {
            // If a segment isn't mapped, the value just goes into a black hole
            println!(
                "Warning: No segment mapped to address 0x{:08x}. Value will be ignored (not written)",
                address
            );
        }
    }
}
