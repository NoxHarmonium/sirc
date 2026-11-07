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

pub mod iso;
pub mod sprite;
mod types;

use crate::iso::{
    plan_group, plan_group8, tile_address, tile_address_2bit, Cell, GroupPlan, GroupPlan8,
    MAP_CELLS,
};
use crate::sprite::{
    build_line, SpriteConfig, SpriteEntry, SpriteLine, SpriteLineStats, SpritePixel, FETCH_BUDGET,
    MAX_SPRITES_PER_LINE, SPRITE_COUNT, SPRITE_RAM_BASE, SPRITE_RAM_WORDS,
};
use crate::types::{
    BaseConfigRegister, PaletteRegister, PaletteSize, PpuPixel, PpuRegisters, TileSize,
    PIXEL_BUFFER_SIZE, VIDEO_MODE_ISOMETRIC, VIDEO_MODE_ISO_ORTHOGONAL,
};
use log::info;
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

// The first half of the video device segment is reserved for special registers, palette, etc.
// The second half of the video device segment (starting at 0x8000) is just multipurpose vram that
// can store tile data and tilemap data
// Half a segment = 64kw = 128kb = which is twice the SNES so plenty
const VRAM_SIZE: usize = 0x8000;
const VRAM_OFFSET: u32 = 0x8000;
// 256 RGB words - fast access
const PALETTE_SIZE: usize = 256;
// PAL can have a higher resolution, but to keep things simple,
// the renderer size can be static and PAL can have black bars
const WIDTH_PIXELS: u16 = 256;
const HEIGHT_PIXELS: u16 = 224;

const TOTAL_LINES: u16 = 262; // NTSC
                              // This just refers to the lines the TV can technically display
                              // the console will output less than this
                              // Number of vsync lines = TOTAL_LINES - VSYNC_LINE

pub const VSYNC_INTERRUPT: u8 = 0x1 << 3; // l4 - 1
/// The line-compare interrupt, level 3 (to be agreed with the hardware manual)
pub const LINE_INTERRUPT: u8 = 0x1 << 2; // l3 - 1
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
    BackPorch, // TODO: Load sprite data in back porch
    // We have a budget of 4 x 4 = 16 master cycles = 8 ppu cycles to load a group of four pixels
    FetchTilemapEntry(Backgrounds), // 3 ppu cycles (first cycle muxes all the data loaded from last render cycle)
    FetchTile(Backgrounds),         // 3 ppu cycles
    FetchSpriteTilemapEntry,        // 1 ppu cycle
    FetchSpriteTile,                // 1 ppu cycle
    // Mode 2 (isometric): eight slots per group of four pixels, four for each of the two layers
    IsoFetch(Backgrounds, IsoStep),
    FrontPorch,
}

/// The four fetch slots a Mode 2 layer gets in every group of four pixels. The B slots are only
/// used when the group straddles two cells (about one group in five); otherwise they are idle.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum IsoStep {
    TilemapA,
    TileA,
    TilemapB,
    TileB,
}

/// Whether each isometric layer keeps the tilemap entry of its last cell instead of reading it for
/// every group. Off in the specification: see the manual (Section on the entry cache).
const ISO_ENTRY_CACHE: bool = false;

/// Groups of tile data the status layer (background 3 in Mode 2) can hold ahead of the picture. The
/// exhaustive study in the tests shows four are enough for every pair of isometric scroll positions.
const HUD_QUEUE_GROUPS: usize = 4;
/// The same number as `HUD_QUEUE_GROUPS`, for the group arithmetic.
const HUD_LOOKAHEAD: i32 = 4;

/// Number of PPU cycles in one group of four pixels (2 PPU cycles per pixel).
const ISO_SLOTS_PER_GROUP: u16 = 8;
/// Master clocks per `poll` (the PPU runs at half the master clock rate).
const MASTER_CLOCKS_PER_POLL: u16 = 2;
const ISO_GROUP_CLOCKS: u16 = ISO_SLOTS_PER_GROUP * MASTER_CLOCKS_PER_POLL;

/// A resolved Mode 2 pixel for one layer: `value` is the 4bpp tile value (0 = transparent) and
/// `palette_addr` the palette entry it selects.
#[derive(Debug, Default, Clone, Copy)]
struct IsoLayerPixel {
    value: u8,
    palette_addr: u8,
    /// -1 darken, 0 normal, 1 lighten (from the tilemap entry of an isometric layer)
    shade: i8,
    /// An orthogonal layer's priority bit: the pixel is in front of everything, sprite layer 1 included
    front: bool,
}

#[derive(Debug, Default)]
struct IsoLayerState {
    plan: GroupPlan,
    entries: [TilemapEntry; 2],
    words: [u16; 2],
    /// Four pixels fetched during the current group, shifted into `window` at the next group.
    staging: [IsoLayerPixel; 4],
    /// Two groups of aligned pixels; the fine horizontal scroll picks a four pixel slice from it.
    window: [IsoLayerPixel; 8],
    /// Scroll registers, sampled once per line at the start of the prefetch.
    scroll_x: i32,
    scroll_y: i32,
    /// `scroll_x` modulo four: where in the window the line's first pixel is.
    fine_x: usize,
    /// The tilemap entry of the cell most recently looked up. The cell only changes about once
    /// every sixteen pixels, so the entry is read again only when the cell changes.
    cache: Option<(Cell, TilemapEntry)>,
    /// Video memory reads this layer has made on the current line (for the tests).
    reads: u32,
    /// The layer uses 2 bit tiles (latched with the scroll registers).
    two_bit: bool,
    /// The plan of the eight pixel group being fetched (2 bit layers).
    plan8: GroupPlan8,
    /// The second four pixels of the eight pixel group last fetched (2 bit layers).
    held: [IsoLayerPixel; 4],
}

/// One group of four status layer pixels waiting in the queue.
#[derive(Debug, Default, Clone, Copy)]
struct HudItem {
    word: u16,
    palette_select: u8,
    front: bool,
    /// 2 bit tiles: the word holds eight pixels and `half` says which four this item is
    two_bit: bool,
    half: usize,
}

#[derive(Debug, Default, Clone, Copy)]
struct HudPixel {
    value: u8,
    palette_addr: u8,
    front: bool,
}

/// The status layer of Mode 2: an orthogonal layer of 8 by 8 tiles, fed from the fetch slots the two
/// isometric layers leave spare, through a queue of words fetched ahead of the picture.
#[derive(Debug, Default)]
struct HudState {
    enabled: bool,
    queue: [HudItem; HUD_QUEUE_GROUPS + 2],
    len: usize,
    /// The next group of the line to fetch (0..64).
    next_group: i32,
    /// The tilemap entry of the current tile, once read.
    entry: Option<TilemapEntry>,
    /// The four pixels being shown.
    current: [HudPixel; 4],
    /// Groups that were not in the queue when the picture needed them (should always be zero).
    underruns: u32,
    /// Reads made on the current line (including the porch).
    reads: u32,
}

#[derive(Debug, Default)]
struct IsoRegisters {
    layers: [IsoLayerState; 2],
    /// The two orthogonal layers of Mode 3 (background 1 and 2)
    orthogonal: [IsoLayerState; 2],
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
    palette: [PpuPixel; PALETTE_SIZE],
    ppu_registers: PpuRegisters,
    vram_fetch_register: FetchRegisters,
    iso: IsoRegisters,
    hud: HudState,
    pixel_mux_buffer_register: PixelBuffer,
    sprite_ram: [u16; SPRITE_RAM_WORDS],
    sprite_line: SpriteLine,
    /// What the sprite engine did on the most recent line it built
    pub sprite_stats: SpriteLineStats,

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

/// Merge one pixel of the three sprite layers and backgrounds. `backgrounds` holds each
/// background's palette address and shade, or `None` where its pixel is transparent. The
/// front-to-back order is S1, B1, S2, B2, S3, B3; a transparent result is palette entry 0. A shadow
/// sprite pixel does not draw: it darkens whatever lies behind its layer.
fn composite_palette_addr(sprite: SpritePixel, backgrounds: [Option<(u8, i8)>; 3]) -> (u8, i8) {
    let mut shade = 0_i8;
    for (layer, background) in backgrounds.into_iter().enumerate() {
        if usize::from(sprite.layer) == layer {
            if sprite.opaque {
                return (sprite.palette_addr, shade);
            }
            if sprite.shadow {
                shade -= 1;
            }
        }
        if let Some((addr, background_shade)) = background {
            return (addr, (shade + background_shade).clamp(-1, 1));
        }
    }
    (0, shade)
}

/// Darken (halve each colour channel) or lighten (move each halfway to full) a colour.
fn shade_pixel(pixel: PpuPixel, shade: i8) -> PpuPixel {
    let light = |c: u8| c + ((31 - c) >> 1);
    match shade.signum() {
        -1 => pixel
            .with_r(pixel.r() >> 1)
            .with_g(pixel.g() >> 1)
            .with_b(pixel.b() >> 1),
        1 => pixel
            .with_r(light(pixel.r()))
            .with_g(light(pixel.g()))
            .with_b(light(pixel.b())),
        _ => pixel,
    }
}

/// What a tilemap entry's flip bits mean to an isometric layer, which cannot flip tiles: the
/// vertical flip bit darkens the tile and the horizontal flip bit lightens it (both: normal).
fn entry_shade(entry: TilemapEntry) -> i8 {
    match (entry.flip_vertical(), entry.flip_horizontal()) {
        (1, 0) => -1,
        (0, 1) => 1,
        _ => 0,
    }
}

fn resolve_palette_addr(palette_select: u8, palette_register: PaletteRegister) -> u8 {
    let shift: u32 = match palette_register.palette_size() {
        PaletteSize::Four => 2,
        PaletteSize::Eight => 3,
        PaletteSize::Sixteen => 4,
        PaletteSize::ThirtyTwo => 5,
    };
    // Use shift to keep hardware simple - we shouldn't need to handle overflow when shifting
    // Max value is palette_select = 7 and palette_register.palette_size() = ThirtyTwo) which result
    // in 0b0000_0111 shifted left by 5 => 0b1110_0000 = 224 which is still in bounds.
    // We can overflow when adding the offset; in that case, the result will wrap, which is the
    // simplest way to do it in hardware.
    // assert_eq!(palette_select, 0, "palette_select: {} shift: {} palette_offset:{} result: {}", palette_select, shift, palette_register.palette_offset(), palette_select.unbounded_shl(shift).wrapping_add(palette_register.palette_offset()));
    palette_select
        .unbounded_shl(shift)
        .wrapping_add(palette_register.palette_offset())
}

fn tile_pixel(line: TileLine, k: usize) -> u8 {
    match k {
        0 => line.p1(),
        1 => line.p2(),
        2 => line.p3(),
        _ => line.p4(),
    }
}

/// Mode 1: combine the three backgrounds and the sprites into four finished palette addresses.
fn resolve_tile_line(
    fetch_registers: &FetchRegisters,
    ppu_registers: &PpuRegisters,
    sprites: [SpritePixel; 4],
) -> [(u8, i8); 4] {
    let offsets = [
        resolve_palette_addr(
            fetch_registers.bg1_tilemap.palette_select(),
            ppu_registers.b1_palette_config,
        ),
        resolve_palette_addr(
            fetch_registers.bg2_tilemap.palette_select(),
            ppu_registers.b2_palette_config,
        ),
        resolve_palette_addr(
            fetch_registers.bg3_tilemap.palette_select(),
            ppu_registers.b3_palette_config,
        ),
    ];
    let tiles = [
        &fetch_registers.bg1_tile,
        &fetch_registers.bg2_tile,
        &fetch_registers.bg3_tile,
    ];
    let mut out = [(0_u8, 0_i8); 4];
    for (k, pixel) in out.iter_mut().enumerate() {
        let backgrounds = [0, 1, 2].map(|i| {
            let value = tile_pixel(*tiles[i], k);
            (value > 0).then(|| (offsets[i].wrapping_add(value), 0))
        });
        *pixel = composite_palette_addr(sprites[k], backgrounds);
    }
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
        palette: [PpuPixel::new(); PALETTE_SIZE],
        vram_fetch_register: FetchRegisters::default(),
        iso: IsoRegisters::default(),
        hud: HudState::default(),
        pixel_mux_buffer_register: PixelBuffer::default(),
        // All ones: every sprite is hidden (layer 3) until the program writes the table
        sprite_ram: [0xFFFF; SPRITE_RAM_WORDS],
        sprite_line: SpriteLine::default(),
        sprite_stats: SpriteLineStats::default(),
        // Reset state: the picture is blank (GD = 1), brightness is full and everything else is zero
        ppu_registers: PpuRegisters {
            base_config: BaseConfigRegister::new()
                .with_graphics_disable(1)
                .with_screen_brightness(15),
            ..PpuRegisters::default()
        },
        frame_count: 0,
        line_clock: 0,
        line: 0,
        clocks_per_line,
        clocks_per_pixel: visible_clocks / WIDTH_PIXELS,
        line_preamble_clocks: preamble_clocks,
        line_postamble_clocks: postamble_clocks,
        line_visible_clocks: visible_clocks,
        vsync_frequency,
        state: RenderStateMachine::BackPorch,
    }
}

impl VideoDevice {
    /// One PPU cycle of Mode 2 (isometric) rendering.
    ///
    /// Per group of four pixels (eight PPU cycles) each of the two layers gets four slots:
    /// tilemap entry A, tile word A, tilemap entry B, tile word B. The B slots are idle unless the
    /// group straddles two cells. Groups are aligned to the *scrolled* plane, and a per-layer
    /// two-group window with a 0-3 pixel offset provides fine horizontal scrolling. Each line
    /// starts with two groups of prefetch in the back porch.
    fn step_isometric(&mut self) {
        let preamble = self.line_preamble_clocks;
        let clock = self.line_clock;
        let prefetch_start = preamble.saturating_sub(2 * ISO_GROUP_CLOCKS);

        if self.line >= HEIGHT_PIXELS || clock < prefetch_start {
            self.state = if clock < preamble {
                RenderStateMachine::BackPorch
            } else {
                RenderStateMachine::FrontPorch
            };
        } else if clock < preamble {
            // Front porch prefetch: two groups, shifting in the first so that when the line
            // starts the window holds group 0 and the staging registers hold group 1.
            let rel = clock - prefetch_start;
            let run = i32::from(rel / ISO_GROUP_CLOCKS);
            let slot = (rel % ISO_GROUP_CLOCKS) / MASTER_CLOCKS_PER_POLL;
            if rel == 0 {
                self.iso_latch_scroll();
                self.hud_start_line();
            }
            self.iso_run_slot(slot, run);
            if run == 0 && slot == ISO_SLOTS_PER_GROUP - 1 {
                self.iso_shift_window();
            }
            self.state = RenderStateMachine::BackPorch;
        } else if clock < preamble + self.line_visible_clocks {
            let rel = clock - preamble;
            let group = i32::from(rel / ISO_GROUP_CLOCKS);
            let slot = (rel % ISO_GROUP_CLOCKS) / MASTER_CLOCKS_PER_POLL;
            if slot == 0 {
                self.iso_shift_window();
                self.hud_pop();
                self.iso_mux_group(usize::from(rel / ISO_GROUP_CLOCKS));
            }
            // While showing group g we fetch the group two ahead of it (see iso_shift_window)
            let slot_used = self.iso_run_slot(slot, group + 2);
            // A slot the isometric layers do not need goes to the status layer, if its queue has room
            if !slot_used && self.hud.enabled && self.hud.next_group <= group + HUD_LOOKAHEAD {
                self.hud_read_step();
            }
            self.state = RenderStateMachine::IsoFetch(
                if slot < 4 {
                    Backgrounds::Bg1
                } else {
                    Backgrounds::Bg2
                },
                match slot % 4 {
                    0 => IsoStep::TilemapA,
                    1 => IsoStep::TileA,
                    2 => IsoStep::TilemapB,
                    _ => IsoStep::TileB,
                },
            );
        } else {
            self.state = RenderStateMachine::FrontPorch;
        }
    }

    fn iso_latch_scroll(&mut self) {
        let regs = &self.ppu_registers;
        // (scroll x, scroll y, 2 bit tiles) of each background
        let layer = |x: types::ScrollRegister, y: types::ScrollRegister, p: PaletteRegister| {
            (x.scroll_amount(), y.scroll_amount(), p.two_bit_tiles() == 1)
        };
        let b1 = layer(regs.b1_scroll_x, regs.b1_scroll_y, regs.b1_palette_config);
        let b2 = layer(regs.b2_scroll_x, regs.b2_scroll_y, regs.b2_palette_config);
        let b3 = layer(regs.b3_scroll_x, regs.b3_scroll_y, regs.b3_palette_config);
        // Mode 2: backgrounds 1 and 2 are isometric. Mode 3: 1 and 2 are orthogonal and 3 is isometric
        // (in the second isometric slot).
        let iso = if regs.base_config.video_mode() == VIDEO_MODE_ISO_ORTHOGONAL {
            [b1, b3]
        } else {
            [b1, b2]
        };
        for (state, (x, y, two_bit)) in self
            .iso
            .layers
            .iter_mut()
            .zip(iso)
            .chain(self.iso.orthogonal.iter_mut().zip([b1, b2]))
        {
            state.scroll_x = i32::from(x);
            state.scroll_y = i32::from(y);
            state.fine_x = usize::from(x & 3);
            state.two_bit = two_bit;
            state.cache = None;
            state.reads = 0;
        }
    }

    /// Slide each layer's window along by one group of four pixels.
    fn iso_shift_window(&mut self) {
        for layer in self
            .iso
            .layers
            .iter_mut()
            .chain(self.iso.orthogonal.iter_mut())
        {
            layer.window.copy_within(4..8, 0);
            layer.window[4..8].copy_from_slice(&layer.staging);
        }
    }

    /// Combine the layers and sprites into the four pixels about to be shown (group `group` of
    /// the line).
    fn iso_mux_group(&mut self, group: usize) {
        let orthogonal_mode =
            self.ppu_registers.base_config.video_mode() == VIDEO_MODE_ISO_ORTHOGONAL;
        for k in 0..4_u8 {
            let pixel_of = |layer: &IsoLayerState| {
                let pixel = layer.window[layer.fine_x + usize::from(k)];
                (pixel.value > 0).then_some((pixel.palette_addr, pixel.shade))
            };
            let mut backgrounds = if orthogonal_mode {
                // Mode 3: orthogonal backgrounds 1 and 2 in front of isometric background 3
                [
                    pixel_of(&self.iso.orthogonal[0]),
                    pixel_of(&self.iso.orthogonal[1]),
                    pixel_of(&self.iso.layers[1]),
                ]
            } else {
                [
                    pixel_of(&self.iso.layers[0]),
                    pixel_of(&self.iso.layers[1]),
                    None,
                ]
            };
            // An orthogonal tile whose priority bit is set is in front of everything, sprites included
            let front_orthogonal = if orthogonal_mode {
                self.iso.orthogonal.iter().find_map(|layer| {
                    let pixel = layer.window[layer.fine_x + usize::from(k)];
                    (pixel.value > 0 && pixel.front).then_some(pixel.palette_addr)
                })
            } else {
                None
            };
            let sprite = self.sprite_line.pixels[(group * 4 + usize::from(k)).min(255)];
            // The status layer (Mode 2) is behind everything unless its priority bit is set
            let hud = if self.hud.enabled {
                self.hud.current[usize::from(k)]
            } else {
                HudPixel::default()
            };
            if !orthogonal_mode {
                backgrounds[2] = (hud.value > 0 && !hud.front).then_some((hud.palette_addr, 0));
            }
            let front = front_orthogonal
                .or_else(|| (hud.value > 0 && hud.front).then_some(hud.palette_addr));
            let (palette_addr, shade) = front.map_or_else(
                || composite_palette_addr(sprite, backgrounds),
                |addr| (addr, 0),
            );
            self.pixel_mux_buffer_register[k] =
                shade_pixel(self.palette[palette_addr as usize], shade);
        }
    }

    /// Tilemap base, tile data base, palette configuration and disable bit of isometric layer
    /// `layer_index` (0 or 1). In Mode 3 the only isometric layer is background 3.
    fn iso_layer_registers(&self, layer_index: usize) -> (u16, u16, PaletteRegister, bool) {
        let regs = &self.ppu_registers;
        if regs.base_config.video_mode() == VIDEO_MODE_ISO_ORTHOGONAL {
            (
                regs.b3_tilemap_addr,
                regs.b3_tile_addr,
                regs.b3_palette_config,
                regs.base_config.b3_disable() == 1,
            )
        } else if layer_index == 0 {
            (
                regs.b1_tilemap_addr,
                regs.b1_tile_addr,
                regs.b1_palette_config,
                regs.base_config.b1_disable() == 1,
            )
        } else {
            (
                regs.b2_tilemap_addr,
                regs.b2_tile_addr,
                regs.b2_palette_config,
                regs.base_config.b2_disable() == 1,
            )
        }
    }

    /// One fetch slot (0..4) of Mode 3's two orthogonal backgrounds: a tilemap entry then a tile word for
    /// background 1 (slots 0 and 1) and background 2 (slots 2 and 3), for the plane group
    /// `(scroll_x >> 2) + group_offset`. An orthogonal background is a 32 by 32 tilemap of 8 by 8 tiles
    /// that wraps. With 2 bit tiles one word holds a tile row, so only every second group reads.
    /// Returns whether a read was made.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn orthogonal_run_slot(&mut self, slot: u16, group_offset: i32) -> bool {
        let k = usize::from(slot / 2);
        let regs = &self.ppu_registers;
        let (tilemap_base, tile_base, palette_config, disabled) = if k == 0 {
            (
                regs.b1_tilemap_addr,
                regs.b1_tile_addr,
                regs.b1_palette_config,
                regs.base_config.b1_disable() == 1,
            )
        } else {
            (
                regs.b2_tilemap_addr,
                regs.b2_tile_addr,
                regs.b2_palette_config,
                regs.base_config.b2_disable() == 1,
            )
        };
        let layer = &self.iso.orthogonal[k];
        let plane_group = (layer.scroll_x >> 2) + group_offset;
        let odd = plane_group & 1 == 1;
        let row = i32::from(self.line) + layer.scroll_y;
        let two_bit = layer.two_bit;
        // 2 bit tiles: the second group of a tile reuses the word, except at the line's start
        let reads_word = !two_bit || !odd || group_offset == 0;
        let reads_before = layer.reads;
        if reads_word && slot.is_multiple_of(2) {
            let address = tilemap_base
                .wrapping_add((((row >> 3) & 31) * 32 + ((plane_group >> 1) & 31)) as u16);
            let entry: TilemapEntry = self.read_address(u32::from(address)).into();
            let layer = &mut self.iso.orthogonal[k];
            layer.entries[0] = entry;
            layer.reads += 1;
        } else if slot % 2 == 1 {
            if reads_word {
                let entry = layer.entries[0];
                let in_tile = (row & 7) as u16;
                let address = if two_bit {
                    tile_base
                        .wrapping_add(entry.tile_index().wrapping_mul(8))
                        .wrapping_add(in_tile)
                } else {
                    tile_base
                        .wrapping_add(entry.tile_index().wrapping_mul(16))
                        .wrapping_add(in_tile * 2 + (plane_group & 1) as u16)
                };
                let word = self.read_address(u32::from(address));
                let layer = &mut self.iso.orthogonal[k];
                layer.words[0] = word;
                layer.reads += 1;
                let mut pixels = [IsoLayerPixel::default(); 8];
                for (j, pixel) in pixels.iter_mut().enumerate() {
                    let value = if disabled {
                        0
                    } else if two_bit {
                        ((word >> (14 - 2 * j)) & 3) as u8
                    } else if j < 4 {
                        ((word >> (12 - 4 * j)) & 0xF) as u8
                    } else {
                        0
                    };
                    if value > 0 {
                        *pixel = IsoLayerPixel {
                            value,
                            palette_addr: resolve_palette_addr(
                                entry.palette_select(),
                                palette_config,
                            )
                            .wrapping_add(value),
                            shade: 0,
                            front: entry.priority() == 1,
                        };
                    }
                }
                if two_bit && odd {
                    layer.staging.copy_from_slice(&pixels[4..8]);
                } else {
                    layer.staging.copy_from_slice(&pixels[0..4]);
                    layer.held.copy_from_slice(&pixels[4..8]);
                }
            } else {
                let layer = &mut self.iso.orthogonal[k];
                layer.staging = layer.held;
            }
        }
        self.iso.orthogonal[k].reads != reads_before
    }

    /// Perform one fetch slot (0..8). `group_offset` is the aligned group to fetch relative to
    /// the layer's scrolled plane position: 0 and 1 in the porch, then the screen group plus 2.
    fn iso_run_slot(&mut self, slot: u16, group_offset: i32) -> bool {
        if slot < 4 && self.ppu_registers.base_config.video_mode() == VIDEO_MODE_ISO_ORTHOGONAL {
            return self.orthogonal_run_slot(slot, group_offset);
        }
        let layer_index = usize::from(slot / 4);
        if self.iso.layers[layer_index].two_bit {
            return self.iso_run_slot_two_bit(layer_index, slot % 4, group_offset);
        }
        let reads_before = self.iso.layers[layer_index].reads;
        let (tilemap_base, tile_base, palette_config, disabled) =
            self.iso_layer_registers(layer_index);
        let layer = &self.iso.layers[layer_index];
        let plan = layer.plan;
        match slot % 4 {
            0 => {
                let x0 = ((layer.scroll_x >> 2) + group_offset) * 4;
                let plan = plan_group(x0, i32::from(self.line) + layer.scroll_y);
                let entry = self.iso_entry(layer_index, tilemap_base, plan.cells[0]);
                let layer = &mut self.iso.layers[layer_index];
                layer.plan = plan;
                layer.entries[0] = entry;
            }
            1 => {
                let word = self.read_tile_word(tile_base, layer.entries[0], plan);
                self.iso.layers[layer_index].words[0] = word;
                self.iso.layers[layer_index].reads += 1;
            }
            2 => {
                if plan.cell_count == 2 {
                    let entry = self.iso_entry(layer_index, tilemap_base, plan.cells[1]);
                    self.iso.layers[layer_index].entries[1] = entry;
                }
            }
            _ => {
                if plan.cell_count == 2 {
                    let word = self.read_tile_word(tile_base, layer.entries[1], plan);
                    self.iso.layers[layer_index].words[1] = word;
                    self.iso.layers[layer_index].reads += 1;
                }
                // Last slot of the layer: resolve its four pixels into the staging registers
                let layer = &mut self.iso.layers[layer_index];
                for k in 0..4_usize {
                    let which = usize::from(plan.pixel_cell[k]);
                    let value = if disabled {
                        0
                    } else {
                        ((layer.words[which] >> (12 - 4 * k)) & 0xF) as u8
                    };
                    let palette_addr = if value > 0 {
                        resolve_palette_addr(layer.entries[which].palette_select(), palette_config)
                            .wrapping_add(value)
                    } else {
                        0
                    };
                    layer.staging[k] = IsoLayerPixel {
                        value,
                        palette_addr,
                        shade: if value > 0 {
                            entry_shade(layer.entries[which])
                        } else {
                            0
                        },
                        front: false,
                    };
                }
            }
        }
        self.iso.layers[layer_index].reads != reads_before
    }

    /// One fetch slot of a layer with 2 bit tiles. The layer fetches eight pixels for every second
    /// plane group of four, and the other groups are served from the second half of the last fetch,
    /// so those groups leave all four slots spare. Returns whether a read was made.
    fn iso_run_slot_two_bit(&mut self, layer_index: usize, step: u16, group_offset: i32) -> bool {
        let (tilemap_base, tile_base, palette_config, disabled) =
            self.iso_layer_registers(layer_index);
        let layer = &self.iso.layers[layer_index];
        let reads_before = layer.reads;
        let plane_group = (layer.scroll_x >> 2) + group_offset;
        let odd = plane_group & 1 == 1;
        // The first group of a line may be the second half of a group that was never fetched
        let fetching = !odd || group_offset == 0;
        let plan = layer.plan8;
        match step {
            0 if fetching => {
                let x0 = (plane_group - i32::from(odd)) * 4;
                let plan = plan_group8(x0, i32::from(self.line) + layer.scroll_y);
                let entry = self.iso_entry(layer_index, tilemap_base, plan.cells[0]);
                let layer = &mut self.iso.layers[layer_index];
                layer.plan8 = plan;
                layer.entries[0] = entry;
            }
            1 if fetching => {
                let address =
                    tile_address_2bit(tile_base, layer.entries[0]).wrapping_add(plan.word_in_tile);
                let word = self.read_address(u32::from(address));
                let layer = &mut self.iso.layers[layer_index];
                layer.words[0] = word;
                layer.reads += 1;
            }
            2 if fetching && plan.cell_count == 2 => {
                let entry = self.iso_entry(layer_index, tilemap_base, plan.cells[1]);
                self.iso.layers[layer_index].entries[1] = entry;
            }
            3 => {
                if fetching && plan.cell_count == 2 {
                    let address = tile_address_2bit(tile_base, layer.entries[1])
                        .wrapping_add(plan.word_in_tile);
                    let word = self.read_address(u32::from(address));
                    let layer = &mut self.iso.layers[layer_index];
                    layer.words[1] = word;
                    layer.reads += 1;
                }
                let layer = &mut self.iso.layers[layer_index];
                if fetching {
                    let mut pixels = [IsoLayerPixel::default(); 8];
                    for (k, pixel) in pixels.iter_mut().enumerate() {
                        let which = usize::from(plan.pixel_cell[k]);
                        let value = if disabled {
                            0
                        } else {
                            ((layer.words[which] >> (14 - 2 * k)) & 3) as u8
                        };
                        if value > 0 {
                            *pixel = IsoLayerPixel {
                                value,
                                palette_addr: resolve_palette_addr(
                                    layer.entries[which].palette_select(),
                                    palette_config,
                                )
                                .wrapping_add(value),
                                shade: entry_shade(layer.entries[which]),
                                front: false,
                            };
                        }
                    }
                    let (first_half, second_half) = pixels.split_at(4);
                    if odd {
                        layer.staging.copy_from_slice(second_half);
                    } else {
                        layer.staging.copy_from_slice(first_half);
                        layer.held.copy_from_slice(second_half);
                    }
                } else {
                    layer.staging = layer.held;
                }
            }
            _ => {}
        }
        self.iso.layers[layer_index].reads != reads_before
    }

    /// Start of a line: clear the status layer and fill its queue in the back porch.
    fn hud_start_line(&mut self) {
        self.hud = HudState {
            enabled: self.ppu_registers.base_config.b3_disable() == 0
                && self.ppu_registers.base_config.video_mode() == VIDEO_MODE_ISOMETRIC,
            underruns: self.hud.underruns,
            ..HudState::default()
        };
        // Groups whose window opens before the first visible group are fetched now
        while self.hud.enabled && self.hud.next_group < HUD_LOOKAHEAD {
            self.hud_read_step();
        }
    }

    /// One read for the status layer: a tilemap entry (for every second group) or a tile word.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn hud_read_step(&mut self) {
        let regs = &self.ppu_registers;
        let group = self.hud.next_group;
        let tile_row = self.line / 8;
        let Some(entry) = self.hud.entry else {
            let address = regs
                .b3_tilemap_addr
                .wrapping_add(tile_row * 32 + (group / 2) as u16);
            let entry: TilemapEntry = self.read_address(u32::from(address)).into();
            self.hud.entry = Some(entry);
            self.hud.reads += 1;
            return;
        };
        if regs.b3_palette_config.two_bit_tiles() == 1 {
            // One word holds a tile row of eight pixels: queue it for this group and the next
            let address = regs
                .b3_tile_addr
                .wrapping_add(entry.tile_index().wrapping_mul(8))
                .wrapping_add(self.line % 8);
            let word = self.read_address(u32::from(address));
            for half in 0..2 {
                self.hud.queue[self.hud.len] = HudItem {
                    word,
                    palette_select: entry.palette_select(),
                    front: entry.priority() == 1,
                    two_bit: true,
                    half,
                };
                self.hud.len += 1;
            }
            self.hud.reads += 1;
            self.hud.next_group += 2;
            self.hud.entry = None;
            return;
        }
        let address = regs
            .b3_tile_addr
            .wrapping_add(entry.tile_index().wrapping_mul(16))
            .wrapping_add((self.line % 8) * 2 + (group & 1) as u16);
        let word = self.read_address(u32::from(address));
        let item = HudItem {
            word,
            palette_select: entry.palette_select(),
            front: entry.priority() == 1,
            two_bit: false,
            half: 0,
        };
        self.hud.queue[self.hud.len] = item;
        self.hud.len += 1;
        self.hud.reads += 1;
        self.hud.next_group += 1;
        if group & 1 == 1 {
            self.hud.entry = None;
        }
    }

    /// Take the next group out of the queue to be shown.
    fn hud_pop(&mut self) {
        if !self.hud.enabled {
            return;
        }
        if self.hud.len == 0 {
            self.hud.underruns += 1;
            self.hud.current = [HudPixel::default(); 4];
            return;
        }
        let item = self.hud.queue[0];
        self.hud.queue.copy_within(1.., 0);
        self.hud.len -= 1;
        let palette =
            resolve_palette_addr(item.palette_select, self.ppu_registers.b3_palette_config);
        for (k, pixel) in self.hud.current.iter_mut().enumerate() {
            let value = if item.two_bit {
                ((item.word >> (14 - 2 * (item.half * 4 + k))) & 3) as u8
            } else {
                ((item.word >> (12 - 4 * k)) & 0xF) as u8
            };
            *pixel = HudPixel {
                value,
                palette_addr: palette.wrapping_add(value),
                front: item.front,
            };
        }
    }

    /// `VLINE` and the `VBL` bit of `STATUS`: the line the beam is on, and whether it is in vertical
    /// blanking. Both change at the first cycle of a line.
    fn update_line_registers(&mut self) {
        self.ppu_registers.current_line = self.line;
        self.ppu_registers
            .status
            .set_vblank(u8::from(self.line >= HEIGHT_PIXELS));
    }

    /// Build the sprite line buffer for the line about to be drawn. In hardware this happens in
    /// the horizontal blanking interval before the line (see `sprite`); the model does it in one
    /// step and charges the fetch budget.
    fn build_sprite_line(&mut self) {
        if self.line == 0 {
            self.ppu_registers.status.set_sprite_overflow(0);
        }
        if self.line >= HEIGHT_PIXELS {
            self.sprite_line = SpriteLine::default();
            self.sprite_stats = SpriteLineStats::default();
            return;
        }
        let regs = &self.ppu_registers;
        let size = |s: TileSize| match s {
            TileSize::EightByEight => 8,
            TileSize::SixteenBySixteen => 16,
        };
        let config = SpriteConfig {
            sizes: [
                size(regs.tile_size.s1_size()),
                size(regs.tile_size.s2_size()),
                size(regs.tile_size.s3_size()),
            ],
            disabled: [
                regs.base_config.s1_disable() == 1,
                regs.base_config.s2_disable() == 1,
                regs.base_config.s3_disable() == 1,
            ],
            tile_base: regs.s_tile_addr,
            shadows: regs.base_config.shadow_enable() == 1,
            max_per_line: MAX_SPRITES_PER_LINE,
            fetch_budget: FETCH_BUDGET,
        };
        let mut table = [SpriteEntry::default(); SPRITE_COUNT];
        for (i, entry) in table.iter_mut().enumerate() {
            *entry = SpriteEntry::from_words(self.sprite_ram[2 * i], self.sprite_ram[2 * i + 1]);
        }
        let mut line = std::mem::take(&mut self.sprite_line);
        self.sprite_stats = build_line(
            &table,
            &config,
            self.line,
            |addr| self.read_address(u32::from(addr)),
            &mut line,
        );
        self.sprite_line = line;
        if self.sprite_stats.overflowed() {
            self.ppu_registers.status.set_sprite_overflow(1);
        }
    }

    /// The tilemap entry of `cell` for a layer: from the cache if it is the cell looked up last,
    /// otherwise a video memory read.
    fn iso_entry(&mut self, layer_index: usize, tilemap_base: u16, cell: Cell) -> TilemapEntry {
        if !ISO_ENTRY_CACHE {
            let entry = self.read_tilemap_entry(tilemap_base, cell);
            self.iso.layers[layer_index].reads += 1;
            return entry;
        }
        if let Some((cached_cell, entry)) = self.iso.layers[layer_index].cache {
            if cached_cell == cell {
                return entry;
            }
        }
        let entry = self.read_tilemap_entry(tilemap_base, cell);
        let layer = &mut self.iso.layers[layer_index];
        layer.cache = Some((cell, entry));
        layer.reads += 1;
        entry
    }

    fn read_tilemap_entry(&self, tilemap_base: u16, cell: Cell) -> TilemapEntry {
        debug_assert!(i32::from(cell.cu) < MAP_CELLS && i32::from(cell.cv) < MAP_CELLS);
        self.read_address(u32::from(tilemap_base.wrapping_add(cell.tilemap_offset())))
            .into()
    }

    fn read_tile_word(&self, tile_base: u16, entry: TilemapEntry, plan: GroupPlan) -> u16 {
        self.read_address(u32::from(
            tile_address(tile_base, entry).wrapping_add(plan.word_in_tile),
        ))
    }
}

impl Device for VideoDevice {
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
        // https://wiki.superfamicom.org/timing#detailed-renderer-timing-200
        // 1. In back porch, load data for 32 sprites
        // 2. Read BG data on the fly

        // We have ~5 master clocks per pixel? That isn't even one CPU clock?
        // We have to read three background tiles on the fly (the sprites have already been stored in internal registers during back porch)
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

        if self.line_clock == 0 {
            self.build_sprite_line();
        }

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

        let isometric = matches!(
            self.ppu_registers.base_config.video_mode(),
            VIDEO_MODE_ISOMETRIC | VIDEO_MODE_ISO_ORTHOGONAL
        );

        match &self.state {
            // Mode 2 has its own fetch schedule; the state is derived from the line clock
            _ if isometric => self.step_isometric(),
            // The mode was changed away from Mode 2 part way through a line: wait for the next line
            RenderStateMachine::IsoFetch(..) => self.state = RenderStateMachine::FrontPorch,
            RenderStateMachine::BackPorch => {
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
                let tilemap_data: TilemapEntry =
                    self.read_address(u32::from(tilemap_entry_address)).into();
                match background {
                    Backgrounds::Bg1 => {
                        // First clock muxes the result from the last cycle (it is pipelined, needs an entire cycle to fill the buffer)
                        let group = usize::from(pixel_clock / 16);
                        let sprites =
                            [0, 1, 2, 3].map(|k| self.sprite_line.pixels[(group * 4 + k).min(255)]);
                        // Until the first tile has been fetched the backgrounds are transparent,
                        // so the last chunk of the line doesn't end up on the next line
                        // (We won't need this when is implemented properly and the fetch starts early)
                        let empty = FetchRegisters::default();
                        let fetched = if tilemap_x > 0 {
                            &self.vram_fetch_register
                        } else {
                            &empty
                        };
                        let addrs = resolve_tile_line(fetched, &self.ppu_registers, sprites);
                        let colour = |(addr, shade): (u8, i8)| {
                            shade_pixel(self.palette[addr as usize], shade)
                        };
                        self.pixel_mux_buffer_register = PixelBuffer {
                            p1: colour(addrs[0]),
                            p2: colour(addrs[1]),
                            p3: colour(addrs[2]),
                            p4: colour(addrs[3]),
                        };

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
                let tile_data: TileLine = self
                    .read_address(u32::from(tile_address + tile_x + tile_y))
                    .into();
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
                    self.state = RenderStateMachine::FrontPorch;
                } else {
                    // Output next pixel
                    self.state = RenderStateMachine::FetchTilemapEntry(Backgrounds::Bg1);
                }
            }
            RenderStateMachine::FrontPorch => {
                // No pixels rendering, can do some preparation
                if self.line_clock == 0 {
                    self.state = RenderStateMachine::BackPorch;
                }
            }
        }

        // Pixels are only output for 224 lines
        // TODO: This should probably be in the middle of the lines, not at the end
        // E.g. (black bars below and above) but should check
        if self.state != RenderStateMachine::FrontPorch
            && self.state != RenderStateMachine::BackPorch
            && self.line < HEIGHT_PIXELS
        {
            let x = pixel_clock / PIXEL_BUFFER_SIZE;
            let y = self.line * WIDTH_PIXELS;

            self.buffer[(x + y) as usize] = ppu_colour_to_minifb_rgb(
                self.pixel_mux_buffer_register[(x % PIXEL_BUFFER_SIZE) as u8],
            );
        }

        let assertions = BusAssertions {
            interrupt_assertion: (if self.line_clock == 0 && self.line == HEIGHT_PIXELS {
                VSYNC_INTERRUPT
            } else {
                0x0
            }) | (if self.line_clock == 0
                && self.ppu_registers.base_config.line_interrupt_enable() == 1
                && self.line == self.ppu_registers.line_compare & 0x1FF
            {
                LINE_INTERRUPT
            } else {
                0x0
            }),
            ..self.perform_bus_io(bus_assertions, selected)
        };

        // PPU runs at half the master clock rate
        self.line_clock += 2;
        if self.line_clock >= self.clocks_per_line {
            self.line_clock = 0;
            self.line += 1;
            self.update_line_registers();
        }

        if self.line >= TOTAL_LINES {
            self.frame_count += 1;
            self.line = 0;
            self.update_line_registers();
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
        match address {
            // First FF addresses are control registers
            0x0000..=0x00FF => self.ppu_registers.read_address(address),
            // 256 palette entries (CGRAM, fast access)
            0x6000..=0x60FF => self.palette[(address - 0x6000) as usize].into(),
            // Sprite table: 128 entries of two words
            SPRITE_RAM_BASE..=0x70FF => self.sprite_ram[(address - SPRITE_RAM_BASE) as usize],
            // After that range
            VRAM_OFFSET..=0xFFFF => self.vram[(address - VRAM_OFFSET) as usize],
            // Else - open bus
            _ => 0x0, // Not sure how real hardware will work. Could be garbage?
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn write_address(&mut self, address: u32, value: u16) {
        match address {
            // First FF addresses are control registers
            0x0000..=0x00FF => {
                self.ppu_registers.write_address(address, value);
            }
            0x6000..=0x60FF => {
                self.palette[(address - 0x6000) as usize] = PpuPixel::from(value);
            }
            SPRITE_RAM_BASE..=0x70FF => {
                self.sprite_ram[(address - SPRITE_RAM_BASE) as usize] = value;
            }
            // After that range
            VRAM_OFFSET..=0xFFFF => self.vram[(address - VRAM_OFFSET) as usize] = value,
            // Else - open bus
            _ => {}
        }
    }
}

impl MemoryMappedDevice for VideoDevice {}

#[cfg(test)]
mod tile_data;

#[cfg(test)]
mod iso_tests;

#[cfg(test)]
mod sprite_tests;

#[cfg(test)]
mod tests {
    use crate::tile_data::{DIGIT_TILES, PALETTE_1, PALETTE_2, TEST_PATTERNS, TEST_TILEMAP};
    use crate::types::{Backgrounds, PaletteRegister, PaletteSize, TileLine, TilemapEntry};
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
            #[allow(clippy::needless_range_loop)]
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
        video_device.palette[0x10..0x20].copy_from_slice(PALETTE_2.as_slice());
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
                // vram is mapped to 0x8000, so 0x1000 will actually be 0x9000 when read by CPU
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
        // vram is mapped to 0x8000, so 0x0100 will actually be 0x8100 when read by CPU
        video_device.vram[0x0100..0x0500].copy_from_slice(TEST_TILEMAP.as_slice());
        video_device.vram[0x0500..0x0900].copy_from_slice(TEST_TILEMAP.as_slice());
        video_device.vram[0x0900..0x0D00].copy_from_slice(TEST_TILEMAP.as_slice());
    }

    #[test]
    fn test_parsing_tilemap() {
        let tilemap_bytes = [0x87, 0xFF];
        let tilemap_from_hex: u16 = u16::from_be_bytes([0x87, 0xFF]);
        let tilemap_from_constructor = TilemapEntry::new()
            .with_flip_vertical(1)
            .with_flip_horizontal(0)
            .with_priority(0)
            .with_palette_select(1)
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

        video_device.ppu_registers.b1_tilemap_addr = 0x8100;
        video_device.ppu_registers.b2_tilemap_addr = 0x8500;
        video_device.ppu_registers.b3_tilemap_addr = 0x8900;
        video_device.ppu_registers.b1_tile_addr = 0x9000;
        video_device.ppu_registers.b2_tile_addr = 0xA000;
        video_device.ppu_registers.b3_tile_addr = 0xB000;
        video_device.ppu_registers.b1_palette_config = PaletteRegister::new()
            .with_palette_size(PaletteSize::Sixteen)
            .with_palette_offset(0);
        video_device.ppu_registers.b2_palette_config = PaletteRegister::new()
            .with_palette_size(PaletteSize::Sixteen)
            .with_palette_offset(0);
        video_device.ppu_registers.b3_palette_config = PaletteRegister::new()
            .with_palette_size(PaletteSize::Sixteen)
            .with_palette_offset(0);

        for line in 0..TOTAL_LINES {
            // Clocks are divided by two to turn master clocks into PPU clocks
            for clock in 0..video_device.line_preamble_clocks / 2 {
                let assertions = video_device.poll(BusAssertions::default(), true);
                assert_eq!(
                    assertions.interrupt_assertion,
                    if line == HEIGHT_PIXELS && clock == 0 {
                        0x1 << 3
                    } else {
                        0x0
                    }
                );
                assert_eq!(video_device.state, RenderStateMachine::BackPorch);
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
                assert_eq!(RenderStateMachine::FrontPorch, video_device.state);
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
