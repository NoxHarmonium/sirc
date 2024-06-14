use std::any::Any;

use log::{debug, info};
use minifb::{Window, WindowOptions};
use peripheral_bus::{
    device::BusAssertions, device::Device, memory_mapped_device::MemoryMappedDevice,
};

// Some reference:
// https://www.raphnet.net/divers/retro_challenge_2019_03/qsnesdoc.html#Reg2115
// https://martin.hinner.info/vga/pal.html#:~:text=PAL%20details&text=CCIR%2FPAL%20standard%20video%20signal,Thus%20field%20rate%20is%2050.
// https://www.nesdev.org/wiki/NTSC_video
// 262 lines

// 25_000_000 / (x * 262) = 60
// 25_000_000 = 60 * x * 262
// 25_000_000 / 262 / 60 = x
// x= 1590

// 25_000_000 / (1590 * 262) = 60._____

// 64kb = 32kw
const VRAM_SIZE: usize = 32_000;
// PAL can have a higher resolution but to keep things simple
// the renderer size can be static and PAL can have black bars
const WIDTH_PIXELS: usize = 256;
const HEIGHT_PIXELS: usize = 224;

const TOTAL_LINES: usize = 262; // NTSC
                                // This just refers to the lines the TV can technically display
                                // the console will output less than this
                                // Number of vsync lines = TOTAL_LINES - VSYNC_LINE

pub const VSYNC_INTERRUPT: u8 = 0x1 << 3; // l4 - 1

pub struct VideoDevice {
    // Video stuff
    buffer: Vec<u32>,
    window: Window,

    // Native
    vram: Vec<u16>,

    // Other
    clock: usize,
    clocks_per_line: usize,
    clocks_per_pixel: usize,
    line_preamble_clocks: usize,
    line_visible_clocks: usize,

    // Public
    pub vsync_frequency: f64,
}

#[must_use]
pub fn new_video_device(master_clock_freq: usize) -> VideoDevice {
    let mut window = Window::new(
        "SIRC - Video Device",
        WIDTH_PIXELS,
        HEIGHT_PIXELS,
        WindowOptions::default(),
    )
    .unwrap();

    let clocks_per_line = 1590;
    let vsync_frequency = master_clock_freq as f64 / (clocks_per_line * TOTAL_LINES) as f64;

    window.set_target_fps(vsync_frequency.floor() as usize);

    let pixels = WIDTH_PIXELS * HEIGHT_PIXELS;
    let clocks_per_line =
        ((master_clock_freq as f64 / vsync_frequency) / TOTAL_LINES as f64).ceil() as usize;
    let line_preable_clocks = 217; // 70%
    let line_postable_clocks = 93; // 30%
    let line_visible_clocks = clocks_per_line - (line_preable_clocks + line_postable_clocks);
    assert_eq!(
        (line_visible_clocks as f64 / WIDTH_PIXELS as f64).fract(),
        0f64,
        "Only a round number of clocks per pixel is supported at this time"
    );

    info!("Total pixels: {pixels} Clocks Per Line: {clocks_per_line} Visible Clocks Per Line: {line_visible_clocks} vsync_frequency: {vsync_frequency}");

    VideoDevice {
        buffer: vec![0; pixels],
        window,
        vram: vec![0; VRAM_SIZE],
        clock: 0,
        clocks_per_line,
        clocks_per_pixel: line_visible_clocks / WIDTH_PIXELS,
        line_preamble_clocks: line_preable_clocks,
        line_visible_clocks,
        vsync_frequency,
    }
}

impl Device for VideoDevice {
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
        let line = self.clock / self.clocks_per_line;
        let first_visible_line = TOTAL_LINES - HEIGHT_PIXELS;

        let line_clock = self.clock % self.clocks_per_line;
        if line >first_visible_line // First lines are vsync
            && self.clock % self.clocks_per_pixel == 0 // There are 5 clocks available for each pixel
            && line_clock >= self.line_preamble_clocks // No drawing happens in the pre/postable
            && line_clock < self.line_preamble_clocks + self.line_visible_clocks
        {
            let v = (line - first_visible_line) * WIDTH_PIXELS;
            let h = (line_clock - self.line_preamble_clocks) / self.clocks_per_pixel;
            self.buffer[v + h] += (self.clock % 0xFFFFFF) as u32;
        }

        if line >= TOTAL_LINES {
            self.clock = 0;
            if self.window.is_open() {
                // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
                self.window
                    .update_with_buffer(&self.buffer, WIDTH_PIXELS, HEIGHT_PIXELS)
                    .unwrap();
            }
        }

        let assertions = BusAssertions {
            interrupt_assertion: if self.clock == 0 {
                VSYNC_INTERRUPT
            } else {
                0x0
            },
            ..self.perform_bus_io(bus_assertions, selected)
        };

        self.clock += 1;

        assertions
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
