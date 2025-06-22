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

mod types;

use crate::types::{PpuPixel, PpuRegisters, PIXEL_BUFFER_SIZE};
use log::{debug, info};
use minifb::WindowOptions;
use peripheral_bus::memory_mapped_device::MemoryMapped;
use peripheral_bus::{
    device::BusAssertions, device::Device, memory_mapped_device::MemoryMappedDevice,
};
use std::any::Any;
use types::{Backgrounds, PixelBuffer, TileLine, TilemapEntry};
// Some reference:
// https://www.raphnet.net/divers/retro_challenge_2019_03/qsnesdoc.html#Reg2115
// https://martin.hinner.info/vga/pal.html#:~:text=PAL%20details&text=CCIR%2FPAL%20standard%20video%20signal,Thus%20field%20rate%20is%2050.
// https://www.nesdev.org/wiki/NTSC_video
// https://forums.nesdev.org/viewtopic.php?f=12&t=14467&p=211380 -- sprite calculations in hardware
// https://retrocomputing.stackexchange.com/questions/30570/snes-sprites-are-they-rendered-using-shift-registers-or-with-a-line-buffer sprites again

// 64kb = 32kw
const VRAM_SIZE: usize = 32_000;
// 256 RGB words - fast access
const PALETTE_SIZE: usize = 256;
// PAL can have a higher resolution but to keep things simple
// the renderer size can be static and PAL can have black bars
const WIDTH_PIXELS: u16 = 256;
const HEIGHT_PIXELS: u16 = 224;

const TOTAL_LINES: u16 = 262; // NTSC
                              // This just refers to the lines the TV can technically display
                              // the console will output less than this
                              // Number of vsync lines = TOTAL_LINES - VSYNC_LINE

pub const VSYNC_INTERRUPT: u8 = 0x1 << 3; // l4 - 1
fn pack_rgb(r: u8, g: u8, b: u8) -> u32 {
    (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
}

#[must_use]
pub fn unpack_rgb(packed: u32) -> (u8, u8, u8) {
    let r = ((packed >> 16) & 0xFF) as u8;
    let g = ((packed >> 8) & 0xFF) as u8;
    let b = (packed & 0xFF) as u8;
    (r, g, b)
}

fn ppu_colour_to_minifb_rgb(ppu_colour: PpuPixel) -> u32 {
    // PPU pixels have 5 bit colour channels, so we need to scale them to 8 bit so they don't
    // look dark
    pack_rgb(
        ppu_colour.r() << 3,
        ppu_colour.g() << 3,
        ppu_colour.b() << 3,
    )
}

pub trait Renderer: std::fmt::Debug {
    fn update_with_buffer(
        &mut self,
        buffer: &[u32],
        width: usize,
        height: usize,
    ) -> Result<(), minifb::Error>;
    fn set_target_fps(&mut self, fps: usize);
    fn is_open(&self) -> bool;
}

#[derive(Debug)]
pub struct MockWindow {}

impl Renderer for MockWindow {
    fn update_with_buffer(
        &mut self,
        _buffer: &[u32],
        _width: usize,
        _height: usize,
    ) -> Result<(), minifb::Error> {
        Ok(())
    }
    fn set_target_fps(&mut self, _fps: usize) {}
    fn is_open(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct MiniFbWindow {
    pub window: minifb::Window,
}

impl Renderer for MiniFbWindow {
    fn update_with_buffer(
        &mut self,
        buffer: &[u32],
        width: usize,
        height: usize,
    ) -> Result<(), minifb::Error> {
        self.window.update_with_buffer(buffer, width, height)
    }
    fn set_target_fps(&mut self, fps: usize) {
        self.window.set_target_fps(fps);
    }
    fn is_open(&self) -> bool {
        self.window.is_open()
    }
}

#[derive(Debug, Eq, PartialEq)]
enum RenderStateMachine {
    FrontPorch, // TODO: Load sprite data in front porch
    // We have a budget of 4 x 4 = 16 master cycles = 8 ppu cycles to load a group of four pixels
    FetchTilemapEntry(Backgrounds), // 3 ppu cycles (first cycle muxes all the data loaded from last render cycle)
    FetchTile(Backgrounds),         // 3 ppu cycles
    FetchSpriteTilemapEntry,        // 1 ppu cycle
    FetchSpriteTile,                // 1 ppu cycle
    BackPorch,
}

#[derive(Debug, Default)]
struct FetchRegisters {
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

    window: Box<dyn Renderer>,

    // Native
    vram: Vec<u16>,
    palette: Vec<PpuPixel>,
    ppu_registers: PpuRegisters,
    vram_fetch_register: FetchRegisters,
    pixel_mux_buffer_register: PixelBuffer,

    // Other
    pub frame_count: usize,
    pub line_clock: u16,
    pub line: u16,
    pub clocks_per_line: u16,
    pub clocks_per_pixel: u16,
    pub line_preamble_clocks: u16,
    pub line_postamble_clocks: u16,
    pub line_visible_clocks: u16,

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
    let mut window: Box<dyn Renderer> = if cfg!(test) {
        Box::new(MockWindow {})
    } else {
        Box::new(MiniFbWindow {
            window: minifb::Window::new(
                "SIRC - Video Device",
                WIDTH_PIXELS as usize,
                HEIGHT_PIXELS as usize,
                WindowOptions::default(),
            )
            .unwrap(),
        })
    };

    // For reference - the SNES NTSC scanline timing is:
    // 12090 ns of preamble
    // 47616 ns of visible
    // 3720 ns of postamble
    // = 63426ns total which is the NTSC hsync time
    // https://www.nesdev.org/wiki/NTSC_video

    // These are specified in master clocks, but the PPU actually runs at half the master clock (each poll increments clock by two)
    let preamble_clocks = 260;
    let visible_clocks = 1024;
    let postamble_clocks = 80;
    let total_clocks_per_line = 260 + 1024 + 80;

    // Make sure total matches the SNES 1364 clocks per line reference we are using
    assert_eq!(1364, total_clocks_per_line);

    // This is fixed, everything else can change based on PAL/NTSC
    let clocks_per_line = preamble_clocks + visible_clocks + postamble_clocks;
    #[allow(clippy::cast_precision_loss)]
    let vsync_frequency =
        master_clock_freq as f64 / (f64::from(total_clocks_per_line) * f64::from(TOTAL_LINES));

    let total_pixels = WIDTH_PIXELS * HEIGHT_PIXELS;
    let clocks_per_pixel = visible_clocks / WIDTH_PIXELS;

    info!("
        preamble_clocks: {preamble_clocks} visible_clocks: {visible_clocks} postamble_clocks: {postamble_clocks}
        clocks_per_line: {clocks_per_line} vsync_frequency: {vsync_frequency} total_pixels: {total_pixels}
        clocks_per_pixel: {clocks_per_pixel}
    ");

    let visible_clock_into_width_remainder = (f64::from(visible_clocks) / f64::from(WIDTH_PIXELS))
        .fract()
        .abs();
    assert!(
        visible_clock_into_width_remainder < f64::EPSILON,
        "Only a round number of clocks per pixel is supported at this time"
    );

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    window.set_target_fps(vsync_frequency.floor() as usize);

    VideoDevice {
        buffer: vec![0; total_pixels as usize],
        window,
        vram: vec![0; VRAM_SIZE],
        palette: vec![PpuPixel::new(); PALETTE_SIZE],
        vram_fetch_register: FetchRegisters::default(),
        pixel_mux_buffer_register: PixelBuffer::default(),
        ppu_registers: PpuRegisters::default(),
        frame_count: 0,
        line_clock: 0,
        line: 0,
        clocks_per_line,
        clocks_per_pixel: visible_clocks / WIDTH_PIXELS,
        line_preamble_clocks: preamble_clocks,
        line_postamble_clocks: postamble_clocks,
        line_visible_clocks: visible_clocks,
        vsync_frequency,
        state: RenderStateMachine::FrontPorch,
    }
}

impl Device for VideoDevice {
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
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

        let pixel_clock: u16 = self.line_clock.saturating_sub(self.line_preamble_clocks);

        // Two things happening at once:
        // The PPU has four pixels (20 clocks) to read the tile data and mux it and latch it to the output buffer
        // The PPU is outputting each of the four pixels of the output buffer in a loop
        //

        let tilemap_size: u16 = 32; // 32x32 - TODO Support double tile maps
        let tile_size: u16 = 8; // 8x8 - TODO: Support 16x16 tiles (are the tilemaps the same size?, how can BGs with different tile sizes fit together?)
        let pixels_per_read = 4; // 4bpp - 16 bit data bus = 4 pixels with every read
        let read_cycles_per_tile_line = tile_size / pixels_per_read;
        let clocks_per_read_cycle = 16;
        let clocks_per_tile = clocks_per_read_cycle * read_cycles_per_tile_line; // 32 master clocks = 16 ppu clocks = 8 ppu clocks per four pixels * 2 (8 pixels per tile line)
        let tile_size_words = read_cycles_per_tile_line * tile_size;

        match &self.state {
            RenderStateMachine::FrontPorch => {
                // No drawing happens in the pre/postable TODO: (should load sprite data out of memory here) and cache it in registers
                // TODO: Also the tilemap fetching should start before the first pixel
                // TODO: The PPU  should get a head start accessing accessing 2-3 tiles ahead of the pixel output.
                if self.line_clock >= self.line_preamble_clocks {
                    self.state = RenderStateMachine::FetchTilemapEntry(Backgrounds::Bg1);
                }
            }
            RenderStateMachine::FetchTilemapEntry(background) => {
                let tilemap_x = pixel_clock / clocks_per_tile; // 32 ppu cycles per tile
                let tilemap_y = (self.line / tile_size) * tilemap_size; // 32 x 32 tile map
                let base_address = match background {
                    Backgrounds::Bg1 => self.ppu_registers.b1_tilemap_addr,
                    Backgrounds::Bg2 => self.ppu_registers.b2_tilemap_addr,
                    Backgrounds::Bg3 => self.ppu_registers.b3_tilemap_addr,
                };
                let tilemap_entry_address: u16 = base_address + tilemap_x + tilemap_y;
                let tilemap_data: TilemapEntry = self.vram[tilemap_entry_address as usize].into();
                match background {
                    Backgrounds::Bg1 => {
                        if tilemap_x > 0 {
                            // First clock muxes the result from the last cycle (it is pipelined, needs an entire cycle to fill the buffer)
                            let resolved_line = resolve_tile_line(&self.vram_fetch_register);
                            self.pixel_mux_buffer_register = PixelBuffer {
                                p1: self.palette[resolved_line.p1() as usize],
                                p2: self.palette[resolved_line.p2() as usize],
                                p3: self.palette[resolved_line.p3() as usize],
                                p4: self.palette[resolved_line.p4() as usize],
                            };
                        } else {
                            // Flush buffer so the last chunk of the line doesn't end up on the next line
                            // (We won't need this when is implemented properly and the fetch starts early)
                            self.pixel_mux_buffer_register = PixelBuffer::default();
                        }

                        self.vram_fetch_register.bg1_tilemap = tilemap_data;
                        self.state = RenderStateMachine::FetchTilemapEntry(Backgrounds::Bg2);
                    }
                    Backgrounds::Bg2 => {
                        self.vram_fetch_register.bg2_tilemap = tilemap_data;
                        self.state = RenderStateMachine::FetchTilemapEntry(Backgrounds::Bg3);
                    }
                    Backgrounds::Bg3 => {
                        self.vram_fetch_register.bg3_tilemap = tilemap_data;
                        self.state = RenderStateMachine::FetchTile(Backgrounds::Bg1);
                    }
                }
            }
            RenderStateMachine::FetchTile(background) => {
                let tile_address = match background {
                    // Yes the tilemaps are fetched twice redundantly and we should probably
                    // just fetch it once and do two tile fetches for the full 8 pixels but
                    // I'd have to increase the pixel buffers, so for now I'll leave this redundant fetch
                    Backgrounds::Bg1 => {
                        self.ppu_registers.b1_tile_addr
                            + self.vram_fetch_register.bg1_tilemap.tile_index() * tile_size_words
                    }
                    Backgrounds::Bg2 => {
                        self.ppu_registers.b2_tile_addr
                            + self.vram_fetch_register.bg2_tilemap.tile_index() * tile_size_words
                    }
                    Backgrounds::Bg3 => {
                        self.ppu_registers.b3_tile_addr
                            + self.vram_fetch_register.bg3_tilemap.tile_index() * tile_size_words
                    }
                };
                let tile_x = ((pixel_clock - 8) / (tile_size * 2)) % 2; // minus 8 for the cycles to load the tile maps
                let tile_y = (self.line % tile_size) * read_cycles_per_tile_line;
                let tile_data: TileLine =
                    self.vram[(tile_address + tile_x + tile_y) as usize].into();
                match background {
                    Backgrounds::Bg1 => {
                        // TODO: Pixel offset per line (e.g. scrolling) (tiles are 8x8, we read one line at a time)
                        // TODO: We probably should read the full 8 pixels at once, which is two reads for the 32 bits
                        self.vram_fetch_register.bg1_tile = tile_data;
                        self.state = RenderStateMachine::FetchTile(Backgrounds::Bg2);
                    }
                    Backgrounds::Bg2 => {
                        self.vram_fetch_register.bg2_tile = tile_data;
                        self.state = RenderStateMachine::FetchTile(Backgrounds::Bg3);
                    }
                    Backgrounds::Bg3 => {
                        self.vram_fetch_register.bg3_tile = tile_data;
                        self.state = RenderStateMachine::FetchSpriteTilemapEntry;
                    }
                }
            }
            RenderStateMachine::FetchSpriteTilemapEntry => {
                // TODO: Sprites
                self.state = RenderStateMachine::FetchSpriteTile;
            }
            RenderStateMachine::FetchSpriteTile => {
                // TODO: Sprites
                if self.line_clock >= self.line_preamble_clocks + self.line_visible_clocks {
                    // Line is done, finish displaying pixels
                    self.state = RenderStateMachine::BackPorch;
                } else {
                    // Output next pixel
                    self.state = RenderStateMachine::FetchTilemapEntry(Backgrounds::Bg1);
                }
            }
            RenderStateMachine::BackPorch => {
                // No pixels rendering, can do some preparation
                if self.line_clock == 0 {
                    self.state = RenderStateMachine::FrontPorch;
                }
            }
        }

        // Pixels are only output for 224 lines
        // TODO: This should probably be in the middle of the lines, not at the end
        // E.g. (black bars below and above) but should check
        if self.state != RenderStateMachine::BackPorch
            && self.state != RenderStateMachine::FrontPorch
            && self.line < HEIGHT_PIXELS
        {
            let x = pixel_clock / PIXEL_BUFFER_SIZE;
            let y = self.line * WIDTH_PIXELS;

            self.buffer[(x + y) as usize] = ppu_colour_to_minifb_rgb(
                self.pixel_mux_buffer_register[(x % PIXEL_BUFFER_SIZE) as u8],
            );
        }

        let assertions = BusAssertions {
            interrupt_assertion: if self.line_clock == 0 && self.line == 0 {
                VSYNC_INTERRUPT
            } else {
                0x0
            },
            ..self.perform_bus_io(bus_assertions, selected)
        };

        // PPU runs at half the master clock rate
        self.line_clock += 2;
        if self.line_clock >= self.clocks_per_line {
            self.line_clock = 0;
            self.line += 1;
        }

        if self.line >= TOTAL_LINES {
            self.frame_count += 1;
            self.line = 0;
            if self.window.is_open() {
                // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
                self.window
                    .update_with_buffer(&self.buffer, WIDTH_PIXELS as usize, HEIGHT_PIXELS as usize)
                    .unwrap();
            }
        }

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
            0x6000..=0x6100 => self.palette[(address - 0x6000) as usize].into(),
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
                self.palette[(address - 0x6000) as usize] = PpuPixel::from(value);
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

#[cfg(test)]
mod tile_data;

#[cfg(test)]
mod tests {
    use crate::tile_data::{DIGIT_TILES, PALETTE_1, TEST_PATTERNS, TEST_TILEMAP};
    use crate::types::{Backgrounds, TileLine, TilemapEntry};
    use crate::{
        new_video_device, unpack_rgb, RenderStateMachine, VideoDevice, HEIGHT_PIXELS, TOTAL_LINES,
        WIDTH_PIXELS,
    };
    use image::ImageFormat;
    use oxipng::{optimize_from_memory, Options, StripChunks};
    use peripheral_bus::device::{BusAssertions, Device};
    use std::io::Cursor;

    // Assume orig_digit_tiles: [[u16; 64]; 10] where each u16 is a palette index (0..15)
    // THIS IS SUCH A HACK, need to restructure the constant data to match what we need
    fn convert_digit_tiles(orig_digit_tiles: &[[u8; 64]; 10]) -> [[TileLine; 16]; 10] {
        let mut result = [[TileLine::new(); 16]; 10];

        for (digit_idx, tile) in orig_digit_tiles.iter().enumerate() {
            // 2x8 = 16
            for i in 0..16 {
                // Each line in output: 4 pixels per TileLine, so two TileLines per row
                let base = i * 4;
                let px0 = tile[base];
                let px1 = tile[base + 1];
                let px2 = tile[base + 2];
                let px3 = tile[base + 3];
                let tl = TileLine::new()
                    .with_p1(px0)
                    .with_p2(px1)
                    .with_p3(px2)
                    .with_p4(px3);
                result[digit_idx][i] = tl;
            }
        }
        result
    }

    fn copy_palettes_to_vram(video_device: &mut VideoDevice) {
        video_device.palette[0x0..0x10].copy_from_slice(PALETTE_1.as_slice());
    }

    fn copy_tiles_to_vram(video_device: &mut VideoDevice) {
        let all_tiles = [DIGIT_TILES, TEST_PATTERNS];
        for (tile_set_idx, tile_set) in all_tiles.iter().enumerate() {
            for (digit_idx, tile) in convert_digit_tiles(tile_set).iter().enumerate() {
                let tile_offset = (tile_set_idx * 10) + digit_idx;
                let tile_as_u16: [u16; 16] = tile
                    .iter()
                    .map(|t| u16::from(*t))
                    .collect::<Vec<u16>>()
                    .try_into()
                    .unwrap();
                video_device.vram[0x1000 + tile_offset * 16..0x1000 + ((tile_offset + 1) * 16)]
                    .copy_from_slice(tile_as_u16.as_slice());
                video_device.vram[0x2000 + tile_offset * 16..0x2000 + ((tile_offset + 1) * 16)]
                    .copy_from_slice(tile_as_u16.as_slice());
                video_device.vram[0x3000 + tile_offset * 16..0x3000 + ((tile_offset + 1) * 16)]
                    .copy_from_slice(tile_as_u16.as_slice());
            }
        }
    }

    fn copy_tilemaps_to_vram(video_device: &mut VideoDevice) {
        video_device.vram[0x0100..0x0500].copy_from_slice(TEST_TILEMAP.as_slice());
        video_device.vram[0x0500..0x0900].copy_from_slice(TEST_TILEMAP.as_slice());
        video_device.vram[0x0900..0x0D00].copy_from_slice(TEST_TILEMAP.as_slice());
    }

    #[test]
    fn test_parsing_tilemap() {
        let tilemap_bytes = [0x83, 0xFF];
        let tilemap_from_hex: u16 = u16::from_be_bytes([0x83, 0xFF]);
        let tilemap_from_constructor = TilemapEntry::new()
            .with_flip_vertical(1)
            .with_flip_horizontal(0)
            .with_priority(0)
            .with_palette_select(0)
            .with_tile_index(0x3FF);

        let x: u16 = tilemap_from_constructor.into();
        assert_eq!(tilemap_from_hex, x);
        assert_eq!(tilemap_bytes, x.to_be_bytes());
    }

    #[test]
    fn test_parsing_tile() {
        let tile_hex = [0x12, 0x34];
        let tile: TileLine = u16::from_be_bytes(tile_hex).into();
        assert_eq!(tile.p1(), 1);
        assert_eq!(tile.p2(), 2);
        assert_eq!(tile.p3(), 3);
        assert_eq!(tile.p4(), 4);
    }

    #[test]
    fn test_video_line_timing() {
        let master_clock_freq = 21_477_272;

        let mut video_device = new_video_device(master_clock_freq);

        copy_palettes_to_vram(&mut video_device);
        copy_tiles_to_vram(&mut video_device);
        copy_tilemaps_to_vram(&mut video_device);

        video_device.ppu_registers.b1_tilemap_addr = 0x0100;
        video_device.ppu_registers.b2_tilemap_addr = 0x0500;
        video_device.ppu_registers.b3_tilemap_addr = 0x0900;
        video_device.ppu_registers.b1_tile_addr = 0x1000;
        video_device.ppu_registers.b2_tile_addr = 0x2000;
        video_device.ppu_registers.b3_tile_addr = 0x3000;

        for line in 0..TOTAL_LINES {
            // Clocks are divided by two to turn master clocks into PPU clocks
            for clock in 0..video_device.line_preamble_clocks / 2 {
                let assertions = video_device.poll(BusAssertions::default(), true);
                assert_eq!(
                    assertions.interrupt_assertion,
                    if line == 0 && clock == 0 {
                        0x1 << 3
                    } else {
                        0x0
                    }
                );
                assert_eq!(video_device.state, RenderStateMachine::FrontPorch);
            }
            for clock in 0..video_device.line_visible_clocks / 2 {
                let assertions = video_device.poll(BusAssertions::default(), true);
                assert_eq!(0x0, assertions.interrupt_assertion);
                let seq = clock % 8;
                let expected_state = match seq {
                    // Two ppu/frame clocks for every call to poll() because the ppu runs at half the master clock rate
                    // A pixel is output every 4 ppu clocks so 4 pixels every 16 clock cycle
                    // So we have 16 ppu clocks to mux four pixels
                    // A tile covers 8 pixels (in 8x8 mode) so
                    0 => RenderStateMachine::FetchTilemapEntry(Backgrounds::Bg1), // Tilemap x incremented every 32 ppu/frame clocks
                    1 => RenderStateMachine::FetchTilemapEntry(Backgrounds::Bg2),
                    2 => RenderStateMachine::FetchTilemapEntry(Backgrounds::Bg3),
                    3 => RenderStateMachine::FetchTile(Backgrounds::Bg1),
                    4 => RenderStateMachine::FetchTile(Backgrounds::Bg2),
                    5 => RenderStateMachine::FetchTile(Backgrounds::Bg3),
                    6 => RenderStateMachine::FetchSpriteTilemapEntry,
                    7 => RenderStateMachine::FetchSpriteTile,
                    _ => panic!("seq out of range"),
                };
                assert_eq!(expected_state, video_device.state);
            }
            for _ in 0..video_device.line_postamble_clocks / 2 {
                let assertions = video_device.poll(BusAssertions::default(), true);
                assert_eq!(0x0, assertions.interrupt_assertion);
                assert_eq!(RenderStateMachine::BackPorch, video_device.state);
            }
        }

        let mut imgbuf = image::ImageBuffer::new(u32::from(WIDTH_PIXELS), u32::from(HEIGHT_PIXELS));
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let packed_pixel = video_device.buffer[(x + y * u32::from(WIDTH_PIXELS)) as usize];
            *pixel = image::Rgb::from(<[u8; 3]>::from(unpack_rgb(packed_pixel)));
        }

        let mut png_buffer = Cursor::new(Vec::new());
        imgbuf
            .write_to(&mut png_buffer, ImageFormat::Png)
            .expect("Failed to write PNG data to buffer");

        let result = optimize_from_memory(
            png_buffer.into_inner().as_slice(),
            &Options {
                // Strip metadata (e.g. create/update date) that causes the snapshot tests to fail
                strip: StripChunks::All,
                force: true,
                ..Options::default()
            },
        )
        .expect("Failed to optimize PNG");
        insta::assert_binary_snapshot!("last_frame.png", result);
    }
}
