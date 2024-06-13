use std::any::Any;

use log::debug;
use minifb::{Window, WindowOptions};
use peripheral_bus::{
    device::BusAssertions, device::Device, memory_mapped_device::MemoryMappedDevice,
};

// Some reference: https://www.raphnet.net/divers/retro_challenge_2019_03/qsnesdoc.html#Reg2115

// 64kb = 32kw
const VRAM_SIZE: usize = 32_000;
// PAL can have a higher resolution but to keep things simple
// the renderer size can be static and PAL can have black bars
const WIDTH_PIXELS: usize = 256;
const HEIGHT_PIXELS: usize = 224;

pub const VSYNC_INTERRUPT: u8 = 0x1 << 3; // l4 - 1

/// Representation of the application state. In this example, a box will bounce around the screen.

pub struct VideoDevice {
    // Video stuff
    buffer: Vec<u32>,
    window: Window,

    // Native
    vram: Vec<u16>,
}

#[must_use]
pub fn new_video_device() -> VideoDevice {
    let mut window = Window::new(
        "SIRC - Video Device",
        WIDTH_PIXELS,
        HEIGHT_PIXELS,
        WindowOptions::default(),
    )
    .unwrap();

    window.set_target_fps(60);

    VideoDevice {
        buffer: vec![0; WIDTH_PIXELS * HEIGHT_PIXELS],
        window,
        vram: vec![0; VRAM_SIZE],
    }
}

impl Device for VideoDevice {
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
        if self.window.is_open() && bus_assertions.interrupt_assertion == VSYNC_INTERRUPT {
            for i in self.buffer.iter_mut() {
                *i += 1; // write something more funny here!
            }

            // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
            self.window
                .update_with_buffer(&self.buffer, WIDTH_PIXELS, HEIGHT_PIXELS)
                .unwrap();
        }

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
