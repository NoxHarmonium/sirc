use std::{any::Any, time::Duration};

use log::{debug, info};
use minifb::{Window, WindowOptions};
use peripheral_bus::{
    device::BusAssertions, device::Device, memory_mapped_device::MemoryMappedDevice,
};

// Some reference:
// https://www.raphnet.net/divers/retro_challenge_2019_03/qsnesdoc.html#Reg2115
// https://martin.hinner.info/vga/pal.html#:~:text=PAL%20details&text=CCIR%2FPAL%20standard%20video%20signal,Thus%20field%20rate%20is%2050.
// https://www.nesdev.org/wiki/NTSC_video

// 64kb = 32kw
const VRAM_SIZE: usize = 32_000;
// PAL can have a higher resolution but to keep things simple
// the renderer size can be static and PAL can have black bars
// 8 pixels less than 256 so the the PPU pixel is a round number of master clocks at 25 Mhz
const WIDTH_PIXELS: usize = 238;
const HEIGHT_PIXELS: usize = 224;

const TOTAL_LINES: usize = 262; // NTSC
                                // This just refers to the lines the TV can technically display
                                // the console will output less than this
                                // Number of vsync lines = TOTAL_LINES - VSYNC_LINE

pub const VSYNC_INTERRUPT: u8 = 0x1 << 3; // l4 - 1

fn hsv_to_rgb(h: f32) -> (u8, u8, u8) {
    let i = (h * 6.0).floor() as u32;
    let f = h * 6.0 - i as f32;
    let q = 1.0 - f;

    let (r, g, b) = match i % 6 {
        0 => (1.0, f, 0.0),
        1 => (q, 1.0, 0.0),
        2 => (0.0, 1.0, f),
        3 => (0.0, q, 1.0),
        4 => (f, 0.0, 1.0),
        5 => (1.0, 0.0, q),
        _ => unreachable!(),
    };

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn pack_rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

pub struct VideoDevice {
    // Video stuff
    buffer: Vec<u32>,
    window: Window,

    // Native
    vram: Vec<u16>,

    // Other
    frame_count: usize,
    frame_clock: usize,
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

    // For reference - the SNES NTSC scanline timing is:
    // 12090 ns of preamble
    // 47616 ns of visible
    // 3720 ns of postamble
    // = 63426ns total which is the NTSC hsync time
    // https://www.nesdev.org/wiki/NTSC_video

    let preamble_time = Duration::from_nanos(12090);
    let visible_time = Duration::from_nanos(47616);
    let postamble_time = Duration::from_nanos(3720);

    let line_time = preamble_time + visible_time + postamble_time;
    let frame_time = line_time * TOTAL_LINES as u32;

    let preamble_clocks = (master_clock_freq as f64 * preamble_time.as_secs_f64()).floor() as usize;
    let visible_clocks = (master_clock_freq as f64 * visible_time.as_secs_f64()).floor() as usize;
    let postamble_clocks =
        (master_clock_freq as f64 * postamble_time.as_secs_f64()).floor() as usize;

    // This is fixed, everything else can change based on PAL/NTSC
    let clocks_per_line = preamble_clocks + visible_clocks + postamble_clocks;
    let vsync_frequency = 1f64 / frame_time.as_secs_f64();

    window.set_target_fps(vsync_frequency.floor() as usize);

    let total_pixels = WIDTH_PIXELS * HEIGHT_PIXELS;
    let clocks_per_pixel = visible_clocks / WIDTH_PIXELS;

    info!("
        preamble_time: {preamble_time:?} visible_time: {visible_time:?}  postamble_time: {postamble_time:?}
        line_time: {line_time:?} frame_time: {frame_time:?} '
        preamble_clocks: {preamble_clocks} visible_clocks: {visible_clocks} postamble_clocks: {postamble_clocks}
        clocks_per_line: {clocks_per_line} vsync_frequency: {vsync_frequency} total_pixels: {total_pixels}
        clocks_per_pixel: {clocks_per_pixel}
    ");

    assert_eq!(
        (visible_clocks as f64 / WIDTH_PIXELS as f64).fract(),
        0f64,
        "Only a round number of clocks per pixel is supported at this time"
    );

    VideoDevice {
        buffer: vec![0; total_pixels],
        window,
        vram: vec![0; VRAM_SIZE],
        frame_count: 0,
        frame_clock: 0,
        clocks_per_line,
        clocks_per_pixel: visible_clocks / WIDTH_PIXELS,
        line_preamble_clocks: preamble_clocks,
        line_visible_clocks: visible_clocks,
        vsync_frequency,
    }
}

impl Device for VideoDevice {
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
        let line = self.frame_clock / self.clocks_per_line;

        let line_clock = self.frame_clock % self.clocks_per_line;
        if line < HEIGHT_PIXELS // Lines below the rendered height are vsync
            && self.frame_clock % self.clocks_per_pixel == 0 // Only update for each PPU pixel, not for each master clock
            && line_clock >= self.line_preamble_clocks // No drawing happens in the pre/postable
            && line_clock < self.line_preamble_clocks + self.line_visible_clocks
        {
            let v = line * WIDTH_PIXELS;
            let h = (line_clock - self.line_preamble_clocks) / self.clocks_per_pixel;
            let pixel = v + h;
            let hue = ((self.frame_count as f32 + (line_clock as f32 / 400.0)) / 200.0) % 1.0;
            let (r, g, b) = hsv_to_rgb(hue);
            let color = pack_rgb(r, g, b);
            self.buffer[pixel] = color;
        }

        if line >= TOTAL_LINES {
            self.frame_clock = 0;
            self.frame_count += 1;
            if self.window.is_open() {
                // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
                self.window
                    .update_with_buffer(&self.buffer, WIDTH_PIXELS, HEIGHT_PIXELS)
                    .unwrap();
            }
        }

        let assertions = BusAssertions {
            interrupt_assertion: if self.frame_clock == 0 {
                VSYNC_INTERRUPT
            } else {
                0x0
            },
            ..self.perform_bus_io(bus_assertions, selected)
        };

        self.frame_clock += 1;

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
            // TODO: Design and implement PPU control registers
            // category=Features
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
            0x0000..=0x00FF => {
                // TODO
            }
            // After that range
            _ => self.vram[(address as usize) - 0x00FF] = value,
        }
    }
}
