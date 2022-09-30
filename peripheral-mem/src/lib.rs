use std::cell::RefCell;
use std::fs::read;
use std::path::PathBuf;

pub struct Segment {
    pub label: String,
    pub address: u16,
    pub size: u16,
    pub writable: bool,
}

pub struct MemoryPeripheral {
    mem_cell: RefCell<[u16; 65536]>,
    segments: Vec<Segment>,
}

pub fn new_memory_peripheral() -> MemoryPeripheral {
    MemoryPeripheral {
        mem_cell: RefCell::new([0; 65536]),
        segments: vec![],
    }
}

impl MemoryPeripheral {
    pub fn get_segment_for_label(&self, label: &str) -> Option<&Segment> {
        self.segments
            .iter()
            .find(|s| s.label == label)
            .map(|s| s.to_owned())
    }

    pub fn get_segment_for_address(&self, address: u16) -> Option<&Segment> {
        // TODO: More efficient way to simulate memory mapping? E.g. range map
        self.segments
            .iter()
            .find(|s| s.address >= address && address < s.address + s.size)
            .map(|s| s.to_owned())
    }

    pub fn map_segment(&mut self, label: &str, address: u16, size: u16, writable: bool) {
        self.segments.push(Segment {
            label: String::from(label),
            address,
            size,
            writable,
        })
    }

    pub fn load_binary_data_into_segment(&self, label: &str, path: PathBuf) {
        let maybe_segment = self.get_segment_for_label(label);
        let (segment_address, segment_size) = match maybe_segment {
            // TODO: Make this nicer. Borrow checker was complaining if matchers are nested
            Some(segment) => (segment.address, segment.size),
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
                let mut mem = self.mem_cell.borrow_mut();
                for i in 0..binary_data.len() / 2 {
                    let destination_address = segment_address as usize + i;
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

    pub fn read_address(&self, address: u16) -> u16 {
        // Range check?
        let mem = self.mem_cell.borrow();
        mem.get(address as usize).unwrap().to_owned()
    }

    pub fn write_address(&self, address: u16, value: u16) -> () {
        let maybe_segment = self.get_segment_for_address(address);
        match maybe_segment {
            Some(segment) => {
                if !segment.writable {
                    panic!(
                        "Segment {} is read-only and cannot be written to",
                        segment.label
                    )
                }
            }
            None => {}
        }
        let mut mem = self.mem_cell.borrow_mut();
        mem[address as usize] = value;
    }
}
