mod types;

use std::{any::Any, time::Duration};

use crate::types::PpuRegisters;
use log::{debug, info};
use minifb::{Window, WindowOptions};
use peripheral_bus::memory_mapped_device::MemoryMapped;
use peripheral_bus::{
    device::BusAssertions, device::Device, memory_mapped_device::MemoryMappedDevice,
};
use types::{Backgrounds, TileLine, TileLineRgb, TilemapEntry};
// Some reference:
// https://www.raphnet.net/divers/retro_challenge_2019_03/qsnesdoc.html#Reg2115
// https://martin.hinner.info/vga/pal.html#:~:text=PAL%20details&text=CCIR%2FPAL%20standard%20video%20signal,Thus%20field%20rate%20is%2050.
// https://www.nesdev.org/wiki/NTSC_video

// 64kb = 32kw
const VRAM_SIZE: usize = 32_000;
// 256 RGB words - fast access
const PALETTE_SIZE: usize = 256;
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

#[derive(Debug)]
enum RenderStateMachine {
    FrontPorch, // TODO: Load sprite data in front porch
    FetchTilemapEntry(Backgrounds),
    FetchTile(Backgrounds),
    MuxTiles,
    Output,
    BackPorch,
}

#[derive(Debug, Default)]
pub struct FetchRegisters {
    bg1_tilemap: TilemapEntry,
    bg2_tilemap: TilemapEntry,
    bg3_tilemap: TilemapEntry,
    bg1_tile: TileLine,
    bg2_tile: TileLine,
    bg3_tile: TileLine,
}

#[derive(Debug)]

pub struct VideoDevice {
    // Video stuff
    buffer: Vec<u32>,
    window: Window,

    // Native
    vram: Vec<u16>,
    palette: Vec<u16>,
    ppu_registers: PpuRegisters,
    vram_fetch_register: FetchRegisters,
    pixel_mux_buffer_register: TileLineRgb,

    // Other
    frame_count: usize,
    frame_clock: usize,
    clocks_per_line: usize,
    clocks_per_pixel: usize,
    line_preamble_clocks: usize,
    line_visible_clocks: usize,

    // Public
    pub vsync_frequency: f64,

    // Sim
    state: RenderStateMachine,
}

fn resolve_first_visible_pixel(pixel_values: [u8; 3], palette_offsets: [u8; 3]) -> u8 {
    for i in 0..=2 {
        if pixel_values[i] > 0 {
            return palette_offsets[i] + pixel_values[i];
        }
    }
    0
}

fn resolve_tile_line(fetch_registers: &FetchRegisters) -> TileLine {
    // TODO: This will be efficient in hardware but not sure how well this code will optimise
    // Would be good to check since it will be in a hot loop
    let palette_offsets = [
        fetch_registers.bg3_tilemap.palette_select(),
        fetch_registers.bg2_tilemap.palette_select(),
        fetch_registers.bg1_tilemap.palette_select(),
    ];
    let p1 = resolve_first_visible_pixel(
        [
            fetch_registers.bg3_tile.p1(),
            fetch_registers.bg2_tile.p1(),
            fetch_registers.bg1_tile.p1(),
        ],
        palette_offsets,
    );
    let p2 = resolve_first_visible_pixel(
        [
            fetch_registers.bg3_tile.p2(),
            fetch_registers.bg2_tile.p2(),
            fetch_registers.bg1_tile.p2(),
        ],
        palette_offsets,
    );
    let p3 = resolve_first_visible_pixel(
        [
            fetch_registers.bg3_tile.p3(),
            fetch_registers.bg2_tile.p3(),
            fetch_registers.bg1_tile.p3(),
        ],
        palette_offsets,
    );
    let p4 = resolve_first_visible_pixel(
        [
            fetch_registers.bg3_tile.p4(),
            fetch_registers.bg2_tile.p4(),
            fetch_registers.bg1_tile.p4(),
        ],
        palette_offsets,
    );
    let mut out = TileLine::new();
    out.set_p1(p1);
    out.set_p2(p2);
    out.set_p3(p3);
    out.set_p4(p4);
    out
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
        palette: vec![0; PALETTE_SIZE],
        vram_fetch_register: FetchRegisters::default(),
        pixel_mux_buffer_register: TileLineRgb::default(),
        ppu_registers: PpuRegisters::default(),
        frame_count: 0,
        frame_clock: 0,
        clocks_per_line,
        clocks_per_pixel: visible_clocks / WIDTH_PIXELS,
        line_preamble_clocks: preamble_clocks,
        line_visible_clocks: visible_clocks,
        vsync_frequency,
        state: RenderStateMachine::FrontPorch,
    }
}

impl Device for VideoDevice {
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
        let line = self.frame_clock / self.clocks_per_line;

        // https://wiki.superfamicom.org/timing#detailed-renderer-timing-200
        // 1. In front porch, load data for 32 sprites
        // 2. Read BG data on the fly

        // We have ~5 master clocks per pixel? That isn't even one CPU clock?
        // We have to read three background tiles on the fly (the sprites have already been stored in internal registers during front porch)
        // We can read 16 bits at a time
        // It takes one CPU cycle to read from memory?

        //     VMDATAH     VMDATAL
        //     $4119       $4118
        //  15  bit  8   7  bit  0
        //   ---- ----   ---- ----
        //   VHPC CCTT   TTTT TTTT
        //   |||| ||||   |||| ||||
        //   |||| ||++---++++-++++- Tile index
        //   |||+-++--------------- Palette selection
        //   ||+------------------- Priority
        //   ++-------------------- Flip vertical (V) or horizontal (H)

        // Each tilemap entry is a 16-bit word in VRAM.
        // The whole tilemap is 32x32 - 8x8 tiles are default but can be 16x16
        // Each row is 32 tiles, left to right, and the 32 rows are top to bottom.
        // Tile index - 10 bits selecting one of 1024 tiles from VRAM relative to the base address
        // Palette selection - 0-7 selects one of up to 8 palettes from CGRAM, depending on the background mode.
        // Priority - tilemaps are separated into background (0) and foreground (1) layers which can allow sprites to appear between these layers. (Not sure if SIRC will implement this)
        // Flip - each tile can be flipped horizontally or vertically.

        // For 8 pixels (40 clocks)
        // Fetch BG1 tilemap entry
        // Fetch BG2 tilemap entry
        // Fetch BG3 tilemap entry
        // Fetch BG1 tile
        // Fetch BG2 tile
        // Fetch BG3 tile
        // Mux

        let line_clock = self.frame_clock % self.clocks_per_line;
        let h: u16 = ((line_clock - self.line_preamble_clocks) / self.clocks_per_pixel)
            .try_into()
            .unwrap();
        let v: u16 = (line * WIDTH_PIXELS).try_into().unwrap();
        let tilemap_size: u16 = 32; // 32x32 - TODO Support double tile maps
        let tile_size: u16 = 8; // 8x8 - TODO: Support 16x16 tiles (are the tilemaps the same size?, how can BGs with different tile sizes fit together?)
        let pixels_per_read = 4; // 4bpp - 16 bit data bus = 4 pixels with every read
        let reads_per_line = tile_size / pixels_per_read;

        if line < HEIGHT_PIXELS // Lines below the rendered height are vsync
            && self.frame_clock % self.clocks_per_pixel == 0 // Only update for each PPU pixel, not for each master clock
            && line_clock >= self.line_preamble_clocks
            && line_clock < self.line_preamble_clocks + self.line_visible_clocks
        {
            let pixel: usize = (v + h).into();
            let hue = ((self.frame_count as f32 + (line_clock as f32 / 400.0)) / 200.0) % 1.0;
            let (r, g, b) = hsv_to_rgb(hue);
            let color = pack_rgb(r, g, b);
            self.buffer[pixel] = color;
        }

        match &self.state {
            RenderStateMachine::FrontPorch => {
                // No drawing happens in the pre/postable (should sort out sprite data here)
                // TODO: Also the tilemap fetching should start before the first pixel
                // "The PPU seems to access memory 2-3 tiles ahead of the pixel output. At least, when we disable Force Blank mid-scanline, there is garbage for about 16-24 pixels."
                if line_clock >= self.line_preamble_clocks {
                    self.state = RenderStateMachine::FetchTilemapEntry(Backgrounds::Bg1)
                }
            }
            RenderStateMachine::FetchTilemapEntry(background) => {
                let tile_x = h / tile_size;
                let base_address = match background {
                    Backgrounds::Bg1 => self.ppu_registers.b1_tilemap_addr,
                    Backgrounds::Bg2 => self.ppu_registers.b2_tilemap_addr,
                    Backgrounds::Bg3 => self.ppu_registers.b3_tilemap_addr,
                };
                let tilemap_entry_address: u16 = base_address + ((tilemap_size * v) + tile_x);
                let tilemap_data = u16::to_be_bytes(self.vram[tilemap_entry_address as usize]);
                match background {
                    Backgrounds::Bg1 => {
                        self.vram_fetch_register.bg1_tilemap =
                            TilemapEntry::from_bytes(tilemap_data);
                        self.state = RenderStateMachine::FetchTilemapEntry(Backgrounds::Bg2);
                    }
                    Backgrounds::Bg2 => {
                        self.vram_fetch_register.bg2_tilemap =
                            TilemapEntry::from_bytes(tilemap_data);
                        self.state = RenderStateMachine::FetchTilemapEntry(Backgrounds::Bg3);
                    }
                    Backgrounds::Bg3 => {
                        self.vram_fetch_register.bg3_tilemap =
                            TilemapEntry::from_bytes(tilemap_data);
                        self.state = RenderStateMachine::FetchTile(Backgrounds::Bg1);
                    }
                }
            }
            RenderStateMachine::FetchTile(background) => {
                let tile_address = match background {
                    Backgrounds::Bg1 => {
                        self.ppu_registers.b1_tile_addr
                            + self.vram_fetch_register.bg1_tilemap.tile_index()
                    }
                    Backgrounds::Bg2 => {
                        self.ppu_registers.b2_tile_addr
                            + self.vram_fetch_register.bg1_tilemap.tile_index()
                    }
                    Backgrounds::Bg3 => {
                        self.ppu_registers.b3_tile_addr
                            + self.vram_fetch_register.bg1_tilemap.tile_index()
                    }
                };
                let tile_offset = ((h / pixels_per_read) % reads_per_line) + (v * reads_per_line);
                let tilemap_data = TileLine::from_bytes(u16::to_be_bytes(
                    self.vram[tile_address as usize + tile_offset as usize],
                ));
                match background {
                    Backgrounds::Bg1 => {
                        // TODO: Offset per line (tiles are 8x8, we read one line at a time)
                        // TODO: We probably should read the full 8 pixels at once, which is two reads for the 32 bits
                        self.vram_fetch_register.bg1_tile = tilemap_data;
                        self.state = RenderStateMachine::FetchTile(Backgrounds::Bg2)
                    }
                    Backgrounds::Bg2 => {
                        self.vram_fetch_register.bg2_tile = tilemap_data;
                        self.state = RenderStateMachine::FetchTile(Backgrounds::Bg3)
                    }
                    Backgrounds::Bg3 => {
                        self.vram_fetch_register.bg3_tile = tilemap_data;
                        self.state = RenderStateMachine::MuxTiles;
                    }
                }
            }
            // Only need to mux every 4 pixels
            RenderStateMachine::MuxTiles => {
                // 4 bits per pixel - 8x8 = 4 bytes per line x 8 lines = 32 bytes
                // Four pixels per word

                let resolved_line = resolve_tile_line(&self.vram_fetch_register);
                self.pixel_mux_buffer_register = TileLineRgb {
                    p1: self.palette[resolved_line.p1() as usize],
                    p2: self.palette[resolved_line.p2() as usize],
                    p3: self.palette[resolved_line.p3() as usize],
                    p4: self.palette[resolved_line.p4() as usize],
                };
                self.state = RenderStateMachine::Output;
            }
            RenderStateMachine::Output => {
                // TODO: Output the four pixels from pixel_mux_buffer_register

                if line_clock >= self.line_preamble_clocks + self.line_visible_clocks {
                    self.state = RenderStateMachine::BackPorch
                }
            }
            RenderStateMachine::BackPorch => {
                // No pixels rendering, can do some preparation
                if line_clock == 0 {
                    self.state = RenderStateMachine::FrontPorch
                }
            }
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

impl MemoryMapped for VideoDevice {
    fn read_address(&self, address: u32) -> u16 {
        debug!("Reading from address 0x{address:X}");
        match address {
            // First FF addresses are control registers
            0x0000..=0x00FF => self.ppu_registers.read_address(address),
            // 256 palette entries (CGRAM, fast access)
            0x6000..=0x6100 => self.palette[(address - 0x6000) as usize],
            // TODO: Sprite Data
            // After that range
            0x8000..=0xFFFF => self.vram[(address as usize) - 0x8000],
            // Else - open bus
            _ => 0x0, // Not sure how real hardware will work. Could be garbage?
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn write_address(&mut self, address: u32, value: u16) {
        debug!("Writing 0x{value:X} to address 0x{address:X}");
        match address {
            // First FF addresses are control registers
            0x0000..=0x00FF => {
                self.ppu_registers.write_address(address, value);
            }
            0x6000..=0x6100 => {
                self.palette[(address - 0x6000) as usize] = value;
            }
            // TODO: Sprite Data
            // After that range
            0x8000..=0xFFFF => self.vram[(address as usize) - 0x8000] = value,
            // Else - open bus
            _ => {}
        }
    }
}

impl MemoryMappedDevice for VideoDevice {}
