use std::any::Any;

use log::debug;
use peripheral_bus::{
    device::BusAssertions, device::Device, memory_mapped_device::MemoryMappedDevice,
};

// Some reference: https://www.raphnet.net/divers/retro_challenge_2019_03/qsnesdoc.html#Reg2115

// 64kb = 32kw
const VRAM_SIZE: usize = 32_000;

pub struct VideoDevice {
    /// Software renderer.
    vram: Vec<u16>,
}

#[must_use]
pub fn new_video_device() -> VideoDevice {
    VideoDevice {
        vram: vec![0; VRAM_SIZE],
    }
}

impl Device for VideoDevice {
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
        self.perform_bus_io(bus_assertions, selected)
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl MemoryMappedDevice for VideoDevice {
    fn read_address(&self, address: u32) -> u16 {
        debug!("Reading from address 0x{address:X}");
        match address {
            // First FF addresses are control registers
            // TODO: Actually implement some control registers
            0x0000..=0x00FF => 0x0,
            // After that range
            _ => self.vram[(address as usize) - 0x00FF],
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn write_address(&mut self, address: u32, value: u16) {
        debug!("Writing 0x{value:X} to address 0x{address:X}");
        match address {
            // First FF addresses are control registers
            // TODO: Actually implement some control registers
            0x0000..=0x00FF => {
                // TODO
            }
            // After that range
            _ => self.vram[(address as usize) - 0x00FF] = value,
        }
    }
}
