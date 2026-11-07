//! Device level tests for Mode 2 (isometric): the pipelined renderer is checked against the
//! unpipelined reference model in `iso`, and a demo scene is baked from "artist" images.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use crate::iso::{
    bake_layer, bake_layer_2bit, reference_sample, reference_sample_2bit, texel_at, BakedLayer,
    TILE_WORDS, TILE_WORDS_2BIT,
};
use crate::sprite::{SpriteEntry, SPRITE_RAM_BASE};
use crate::types::{
    BaseConfigRegister, PaletteRegister, PaletteSize, PpuPixel, TileSize, TileSizeRegister,
    TilemapEntry, VIDEO_MODE_ISOMETRIC, VIDEO_MODE_ISO_ORTHOGONAL,
};
use crate::{
    new_video_device, ppu_colour_to_minifb_rgb, resolve_palette_addr, unpack_rgb, VideoDevice,
    HEIGHT_PIXELS, TOTAL_LINES, WIDTH_PIXELS,
};
use image::ImageFormat;
use oxipng::{optimize_from_memory, Options, StripChunks};
use peripheral_bus::device::{BusAssertions, Device};
use peripheral_bus::memory_mapped_device::MemoryMapped;
use std::io::Cursor;

// VRAM layout used by these tests (addresses as the CPU sees them)
const B1_TILEMAP: u16 = 0x8000;
const B2_TILEMAP: u16 = 0x8400;
const B1_TILES: u16 = 0x9000;
const B2_TILES: u16 = 0xA000;

fn iso_device() -> VideoDevice {
    let mut dev = new_video_device(21_477_272);
    dev.ppu_registers.base_config = BaseConfigRegister::new()
        .with_video_mode(VIDEO_MODE_ISOMETRIC)
        .with_b3_disable(1);
    dev.ppu_registers.b1_tilemap_addr = B1_TILEMAP;
    dev.ppu_registers.b2_tilemap_addr = B2_TILEMAP;
    dev.ppu_registers.b1_tile_addr = B1_TILES;
    dev.ppu_registers.b2_tile_addr = B2_TILES;
    dev
}

fn run_frame(dev: &mut VideoDevice) {
    let polls = u32::from(TOTAL_LINES) * u32::from(dev.clocks_per_line) / 2;
    for _ in 0..polls {
        dev.poll(BusAssertions::default(), true);
    }
}

fn set_scroll(dev: &mut VideoDevice, b1: (u16, u16), b2: (u16, u16)) {
    let regs = &mut dev.ppu_registers;
    regs.b1_scroll_x = regs.b1_scroll_x.with_scroll_amount(b1.0);
    regs.b1_scroll_y = regs.b1_scroll_y.with_scroll_amount(b1.1);
    regs.b2_scroll_x = regs.b2_scroll_x.with_scroll_amount(b2.0);
    regs.b2_scroll_y = regs.b2_scroll_y.with_scroll_amount(b2.1);
}

/// What the pixel at screen position `(x, y)` should be according to the reference model.
fn expected_colour(dev: &VideoDevice, x: i32, y: i32) -> u32 {
    let regs = &dev.ppu_registers;
    let read = |addr: u16| dev.read_address(u32::from(addr));
    let layers = [
        (
            regs.b1_tilemap_addr,
            regs.b1_tile_addr,
            regs.b1_scroll_x,
            regs.b1_scroll_y,
            regs.b1_palette_config,
            regs.base_config.b1_disable(),
        ),
        (
            regs.b2_tilemap_addr,
            regs.b2_tile_addr,
            regs.b2_scroll_x,
            regs.b2_scroll_y,
            regs.b2_palette_config,
            regs.base_config.b2_disable(),
        ),
    ];
    // The status layer (background 3), if enabled: behind everything unless the priority bit is set
    let hud = if regs.base_config.b3_disable() == 1 {
        None
    } else {
        let entry: TilemapEntry =
            read(regs.b3_tilemap_addr + (y as u16 / 8) * 32 + x as u16 / 8).into();
        let word = read(
            regs.b3_tile_addr + entry.tile_index() * 16 + (y as u16 % 8) * 2 + (x as u16 % 8) / 4,
        );
        let value = ((word >> (12 - 4 * (x % 4))) & 0xF) as u8;
        (value > 0).then(|| {
            (
                resolve_palette_addr(entry.palette_select(), regs.b3_palette_config)
                    .wrapping_add(value),
                entry.priority() == 1,
            )
        })
    };
    if let Some((addr, true)) = hud {
        return ppu_colour_to_minifb_rgb(dev.palette[addr as usize]);
    }
    let mut palette_addr = hud.map_or(0, |(addr, _)| addr);
    let mut shade = 0;
    for (map, tiles, sx, sy, palette_config, disabled) in layers {
        if disabled == 1 {
            continue;
        }
        let sample = if palette_config.two_bit_tiles() == 1 {
            reference_sample_2bit
        } else {
            reference_sample
        };
        let (value, entry) = sample(
            &read,
            map,
            tiles,
            x + i32::from(sx.scroll_amount()),
            y + i32::from(sy.scroll_amount()),
        );
        if value > 0 {
            palette_addr =
                resolve_palette_addr(entry.palette_select(), palette_config).wrapping_add(value);
            shade = crate::entry_shade(entry);
            break;
        }
    }
    ppu_colour_to_minifb_rgb(crate::shade_pixel(
        dev.palette[palette_addr as usize],
        shade,
    ))
}

fn assert_frame_matches_reference(dev: &VideoDevice, label: &str) {
    for y in 0..i32::from(HEIGHT_PIXELS) {
        for x in 0..i32::from(WIDTH_PIXELS) {
            let got = dev.buffer[(x + y * i32::from(WIDTH_PIXELS)) as usize];
            let want = expected_colour(dev, x, y);
            assert_eq!(
                unpack_rgb(got),
                unpack_rgb(want),
                "{label}: mismatch at x={x} y={y}"
            );
        }
    }
}

struct XorShift(u32);
impl XorShift {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
}

/// Distinct 15 bit colours so that a wrong palette lookup cannot go unnoticed.
fn fill_distinct_palette(dev: &mut VideoDevice) {
    for (i, entry) in dev.palette.iter_mut().enumerate() {
        *entry = PpuPixel::from((i as u16 * 127) & 0x7FFF);
    }
}

fn fill_random_vram(dev: &mut VideoDevice, seed: u32) {
    let mut rng = XorShift(seed);
    let vram_index = |addr: u16| usize::from(addr - 0x8000);
    for map in [B1_TILEMAP, B2_TILEMAP] {
        for i in 0..1024 {
            // A handful of tile indices, any palette select, any flips (flips must be ignored)
            dev.vram[vram_index(map) + i] = (rng.next() & 0xFC00) as u16 | (rng.next() % 6) as u16;
        }
    }
    for tiles in [B1_TILES, B2_TILES] {
        for i in 0..6 * usize::from(TILE_WORDS) {
            // AND two random words so a good share of the nibbles are zero (transparent)
            dev.vram[vram_index(tiles) + i] = (rng.next() & rng.next()) as u16;
        }
    }
    fill_distinct_palette(dev);
    dev.ppu_registers.b1_palette_config = PaletteRegister::new()
        .with_palette_size(PaletteSize::Sixteen)
        .with_palette_offset((rng.next() & 0xFF) as u8);
    dev.ppu_registers.b2_palette_config = PaletteRegister::new()
        .with_palette_size(PaletteSize::Eight)
        .with_palette_offset((rng.next() & 0xFF) as u8);
}

#[test]
fn pipelined_renderer_matches_reference_for_random_content_and_scrolls() {
    let cases = [
        ((0, 0), (0, 0)),
        ((1, 0), (2, 5)),
        ((3, 7), (13, 100)),
        ((1023, 1023), (500, 17)),
        ((130, 9), (131, 10)),
        ((4, 4), (7, 1022)),
    ];
    for (i, (b1, b2)) in cases.into_iter().enumerate() {
        let mut dev = iso_device();
        fill_random_vram(&mut dev, 0x1234_5678 ^ (i as u32 * 7919));
        set_scroll(&mut dev, b1, b2);
        run_frame(&mut dev);
        assert_frame_matches_reference(&dev, &format!("scroll b1={b1:?} b2={b2:?}"));
    }
}

#[test]
fn disabled_layers_do_not_draw() {
    let mut dev = iso_device();
    fill_random_vram(&mut dev, 99);
    set_scroll(&mut dev, (5, 5), (9, 3));
    dev.ppu_registers.base_config = BaseConfigRegister::new()
        .with_video_mode(VIDEO_MODE_ISOMETRIC)
        .with_b3_disable(1)
        .with_b1_disable(1);
    run_frame(&mut dev);
    assert_frame_matches_reference(&dev, "b1 disabled");
    dev.ppu_registers.base_config = BaseConfigRegister::new()
        .with_video_mode(VIDEO_MODE_ISOMETRIC)
        .with_b3_disable(1)
        .with_b1_disable(1)
        .with_b2_disable(1);
    run_frame(&mut dev);
    // Everything transparent: the whole frame is palette entry 0
    let backdrop = ppu_colour_to_minifb_rgb(dev.palette[0]);
    assert!(dev.buffer.iter().all(|px| *px == backdrop));
}

#[test]
fn mode_register_defaults_to_orthogonal() {
    let dev = new_video_device(21_477_272);
    // Reset state: blank (GD), full brightness (BRT = 15), Mode 1
    assert_eq!(0x8078, dev.read_address(0x0000));
    assert_eq!(0, dev.ppu_registers.base_config.video_mode());
    // The mode field is bits 0-2 of register 0x0000
    let mut dev = dev;
    dev.write_address(0x0000, 0x0001);
    assert_eq!(
        VIDEO_MODE_ISOMETRIC,
        dev.ppu_registers.base_config.video_mode()
    );
}

// ---------------------------------------------------------------------------------------------
// Demo scene
// ---------------------------------------------------------------------------------------------

fn rgb5(r: u8, g: u8, b: u8) -> PpuPixel {
    PpuPixel::new().with_r(r).with_g(g).with_b(b)
}

fn demo_palette(dev: &mut VideoDevice) {
    let ground = [
        rgb5(12, 20, 28),
        rgb5(8, 20, 6),   // 1 grass
        rgb5(10, 23, 8),  // 2 grass, light
        rgb5(11, 11, 12), // 3 road
        rgb5(26, 26, 8),  // 4 road marking
        rgb5(6, 12, 26),  // 5 water
        rgb5(10, 17, 29), // 6 water, light
        rgb5(26, 22, 13), // 7 sand
        rgb5(5, 14, 4),   // 8 cell outline
    ];
    dev.palette[..ground.len()].copy_from_slice(&ground);
    let building = [
        rgb5(0, 0, 0),
        rgb5(29, 27, 22), // 1 roof
        rgb5(23, 12, 9),  // 2 left wall
        rgb5(15, 7, 6),   // 3 right wall
    ];
    dev.palette[0x10..0x10 + building.len()].copy_from_slice(&building);
    let cfg = |offset| {
        PaletteRegister::new()
            .with_palette_size(PaletteSize::Sixteen)
            .with_palette_offset(offset)
    };
    // Buildings are layer 1 (in front of everything), ground is layer 2
    dev.ppu_registers.b1_palette_config = cfg(0x10);
    dev.ppu_registers.b2_palette_config = cfg(0x00);
}

/// Which kind of ground is in cell `(cu, cv)`; `cv` may be negative (the map wraps).
fn ground_kind(cu: i32, cv: i32) -> u8 {
    let (lake_u, lake_v) = ((5..=8).contains(&cu), (-2..=1).contains(&cv));
    let (shore_u, shore_v) = ((4..=9).contains(&cu), (-3..=2).contains(&cv));
    match (cu == 12, cv == 2) {
        (true, true) => 3,
        (true, false) => 2,
        (false, true) => 1,
        _ if lake_u && lake_v => 4,
        _ if shore_u && shore_v => 5,
        _ => 0,
    }
}

/// An artist's ground tile, drawn in texture space (value 0 is never used, so it covers the screen).
fn ground_texel(kind: u8, tu: i32, tv: i32) -> u8 {
    if tu == 0 || tv == 0 {
        return 8; // outline, so the diamonds show
    }
    let grass = if ((tu >> 2) + (tv >> 2)) & 1 == 0 {
        1
    } else {
        2
    };
    match kind {
        1 if (6..10).contains(&tv) => {
            if (7..9).contains(&tv) && (tu >> 2) & 1 == 0 {
                4
            } else {
                3
            }
        }
        2 if (6..10).contains(&tu) => {
            if (7..9).contains(&tu) && (tv >> 2) & 1 == 0 {
                4
            } else {
                3
            }
        }
        3 if (6..10).contains(&tu) || (6..10).contains(&tv) => 3,
        4 => {
            if ((tu + tv) >> 3) & 1 == 0 {
                5
            } else {
                6
            }
        }
        5 => 7,
        _ => grass,
    }
}

fn ground_art(x: i32, y: i32) -> u8 {
    let (u, v) = texel_at(x, y);
    // The view only reaches map rows -8..24; make the row signed so features can sit at v < 0
    let cu = u >> 4;
    let cv = v >> 4;
    ground_texel(ground_kind(cu, cv), u & 15, v & 15)
}

/// A prism standing on a diamond footprint, drawn the way an artist would: in screen space with
/// walls that overhang many cells. Returns 0 (transparent) outside the prism.
fn prism_art(x: i32, y: i32, centre: (f64, f64), half: (f64, f64), height: f64) -> u8 {
    let dx = f64::from(x) + 0.5 - centre.0;
    let dy = f64::from(y) + 0.5 - centre.1;
    // The prism is the diamond swept upwards by `height`; a pixel is inside if some t in
    // [0, height] puts it within the diamond translated up by t.
    let reach = half.1 * (1.0 - dx.abs() / half.0);
    if reach < 0.0 {
        return 0;
    }
    let lo = (-dy - reach).max(0.0);
    let hi = (-dy + reach).min(height);
    if lo > hi {
        return 0;
    }
    let top = (dy + height).abs() <= reach;
    if top {
        1
    } else if dx < 0.0 {
        2
    } else {
        3
    }
}

fn building_art(x: i32, y: i32) -> u8 {
    let tower = prism_art(x, y, (128.0, 176.0), (32.0, 16.0), 26.0);
    if tower > 0 {
        return tower;
    }
    prism_art(x, y, (60.0, 150.0), (16.0, 8.0), 14.0)
}

fn load_layer(dev: &mut VideoDevice, baked: &BakedLayer, tilemap: u16, tiles: u16) {
    let base = |addr: u16| usize::from(addr - 0x8000);
    for (i, entry) in baked.tilemap.iter().enumerate() {
        dev.vram[base(tilemap) + i] = *entry;
    }
    for (t, words) in baked.tiles.iter().enumerate() {
        let start = base(tiles) + t * usize::from(TILE_WORDS);
        dev.vram[start..start + words.len()].copy_from_slice(words);
    }
}

fn frame_png(dev: &VideoDevice) -> Vec<u8> {
    let mut imgbuf = image::ImageBuffer::new(u32::from(WIDTH_PIXELS), u32::from(HEIGHT_PIXELS));
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let packed = dev.buffer[(x + y * u32::from(WIDTH_PIXELS)) as usize];
        *pixel = image::Rgb::from(<[u8; 3]>::from(unpack_rgb(packed)));
    }
    let mut png = Cursor::new(Vec::new());
    imgbuf.write_to(&mut png, ImageFormat::Png).unwrap();
    optimize_from_memory(
        png.into_inner().as_slice(),
        &Options {
            strip: StripChunks::All,
            force: true,
            ..Options::default()
        },
    )
    .unwrap()
}

#[test]
fn demo_scene_baked_from_screen_space_art_renders_exactly() {
    let mut dev = iso_device();
    demo_palette(&mut dev);

    // Layer 2 (behind): ground, no scroll. Layer 1 (in front): buildings, sliced against a grid
    // that sits 8 pixels lower, which is what "a Y offset layer" means; the picture on screen is
    // the same however the layer is sliced.
    let layer2_scroll = (0, 8); // (the building layer's scroll)
    let ground = bake_layer(256, 224, 0, 0, &ground_art);
    let buildings = bake_layer(256, 224, layer2_scroll.0, layer2_scroll.1, &building_art);
    println!(
        "demo scene: ground uses {} tiles, buildings use {} tiles",
        ground.tiles.len(),
        buildings.tiles.len()
    );
    load_layer(&mut dev, &ground, B2_TILEMAP, B2_TILES);
    load_layer(&mut dev, &buildings, B1_TILEMAP, B1_TILES);
    set_scroll(
        &mut dev,
        (layer2_scroll.0 as u16, layer2_scroll.1 as u16),
        (0, 0),
    );

    // First check the baked VRAM against the artwork with the reference model alone, so a failure
    // can be pinned on the baker rather than the pipeline
    let read = |addr: u16| dev.read_address(u32::from(addr));
    for y in 0..i32::from(HEIGHT_PIXELS) {
        for x in 0..i32::from(WIDTH_PIXELS) {
            let (b, _) = reference_sample(&read, B1_TILEMAP, B1_TILES, x, y + layer2_scroll.1);
            assert_eq!(b, building_art(x, y), "baked building layer at x={x} y={y}");
            let (g, _) = reference_sample(&read, B2_TILEMAP, B2_TILES, x, y);
            assert_eq!(g, ground_art(x, y), "baked ground layer at x={x} y={y}");
        }
    }

    run_frame(&mut dev);

    // The authored images must come back pixel for pixel
    let mut bad = Vec::new();
    for y in 0..i32::from(HEIGHT_PIXELS) {
        for x in 0..i32::from(WIDTH_PIXELS) {
            let b = building_art(x, y);
            let want = if b > 0 {
                dev.palette[0x10 + usize::from(b)]
            } else {
                dev.palette[usize::from(ground_art(x, y))]
            };
            if unpack_rgb(dev.buffer[(x + y * i32::from(WIDTH_PIXELS)) as usize])
                != unpack_rgb(ppu_colour_to_minifb_rgb(want))
            {
                bad.push((x, y, b));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "{} mismatches, first: {:?}",
        bad.len(),
        &bad[..bad.len().min(24)]
    );
    insta::assert_binary_snapshot!("iso_demo.png", frame_png(&dev));
}

// ---------------------------------------------------------------------------------------------
// Sprites over the demo scene
// ---------------------------------------------------------------------------------------------

const SPRITE_TILES: u16 = 0xC000;

/// A 16 by 16 lorry facing right: 1 body, 2 cab, 3 window, 4 wheel. 0 is transparent.
fn lorry_pixel(row: usize, col: usize) -> u16 {
    match (row, col) {
        (4..=6, 11..=13) => 3,
        (3..=11, 10..=14) => 2,
        (5..=11, 1..=9) => 1,
        (12..=14, 2..=4 | 9..=11) => 4,
        _ => 0,
    }
}

/// A 16 by 16 tree: 5 trunk, 6 and 7 leaves.
fn tree_pixel(row: usize, col: usize) -> u16 {
    let (dr, dc) = (row as i32 - 6, col as i32 - 8);
    match (row, col) {
        (10..=15, 7..=8) => 5,
        _ if dr * dr + dc * dc <= 18 => 6 + ((row + col) & 1) as u16,
        _ => 0,
    }
}

/// A 16 by 16 shadow under a lorry: value 15 is the shadow colour (used with sprite palette 7).
fn shadow_pixel(row: usize, col: usize) -> u16 {
    let (dr, dc) = (row as i32 - 11, col as i32 - 7);
    if dr * dr * 4 + dc * dc <= 36 {
        15
    } else {
        0
    }
}

fn load_sprite_art(dev: &mut VideoDevice) {
    for (tile, art) in [
        lorry_pixel as fn(usize, usize) -> u16,
        tree_pixel,
        shadow_pixel,
    ]
    .into_iter()
    .enumerate()
    {
        for row in 0..16 {
            for w in 0..4 {
                let word = (0..4).fold(0, |acc, n| (acc << 4) | art(row, w * 4 + n));
                // Four quadrants of 16 words: top left, top right, bottom left, bottom right
                let address = SPRITE_TILES
                    + (tile * 64 + 16 * (2 * (row / 8) + w / 2) + 2 * (row % 8) + w % 2) as u16;
                dev.vram[usize::from(address - 0x8000)] = word;
            }
        }
    }
    let colours = [
        (0x81, rgb5(28, 6, 5)),   // palette 0: red lorry
        (0x82, rgb5(26, 18, 8)),  // cab
        (0x83, rgb5(14, 22, 29)), // window
        (0x84, rgb5(2, 2, 3)),    // wheel
        (0x85, rgb5(14, 8, 4)),   // trunk
        (0x86, rgb5(24, 12, 3)),  // leaves (autumn, so they show against the grass)
        (0x87, rgb5(29, 19, 4)),  // leaves, light
        (0x91, rgb5(6, 10, 28)),  // palette 1: blue lorry
        (0x92, rgb5(26, 18, 8)),
        (0x93, rgb5(14, 22, 29)),
        (0x94, rgb5(2, 2, 3)),
    ];
    for (addr, colour) in colours {
        dev.palette[addr] = colour;
    }
}

#[test]
fn sprites_move_between_the_isometric_layers() {
    let mut dev = iso_device();
    demo_palette(&mut dev);
    load_sprite_art(&mut dev);
    dev.ppu_registers.tile_size = TileSizeRegister::new()
        .with_s1_size(TileSize::SixteenBySixteen)
        .with_s2_size(TileSize::SixteenBySixteen)
        .with_s3_size(TileSize::SixteenBySixteen);
    dev.ppu_registers.s_tile_addr = SPRITE_TILES;

    let building_scroll = (0, 8);
    let ground = bake_layer(256, 224, 0, 0, &ground_art);
    let buildings = bake_layer(
        256,
        224,
        building_scroll.0,
        building_scroll.1,
        &building_art,
    );
    load_layer(&mut dev, &ground, B2_TILEMAP, B2_TILES);
    load_layer(&mut dev, &buildings, B1_TILEMAP, B1_TILES);
    set_scroll(&mut dev, (0, 8), (0, 0));

    let sprite = |x, y, tile, palette, layer, flip_h| SpriteEntry {
        x,
        y,
        tile,
        palette,
        flip_h,
        flip_v: false,
        layer,
    };
    let table = [
        sprite(112, 168, 0, 0, 1, false), // S2: behind the tower (B1), in front of the ground
        sprite(140, 178, 0, 1, 0, true),  // S1: in front of the tower
        sprite(30, 120, 1, 0, 0, false),  // a tree in front of everything
        sprite(190, 100, 0, 1, 1, false), // S2 on open ground
    ];
    for (i, entry) in table.iter().enumerate() {
        let (w0, w1) = entry.to_words();
        dev.write_address(SPRITE_RAM_BASE + 2 * i as u32, w0);
        dev.write_address(SPRITE_RAM_BASE + 2 * i as u32 + 1, w1);
    }
    run_frame(&mut dev);

    let at = |dev: &VideoDevice, x: usize, y: usize| unpack_rgb(dev.buffer[x + y * 256]);
    let colour =
        |dev: &VideoDevice, addr: usize| unpack_rgb(ppu_colour_to_minifb_rgb(dev.palette[addr]));
    // The S1 lorry's body (row 8, col 8 mirrored is col 7 of the art, a body pixel) is drawn
    assert_eq!(colour(&dev, 0x91), at(&dev, 140 + 8, 178 + 8));
    // The S2 lorry is hidden where the tower (B1) is opaque, and visible where it is not
    let tower_covers = building_art(112 + 8, 168 + 8) > 0;
    assert!(
        tower_covers,
        "the scene was laid out so that the tower covers this pixel"
    );
    assert_ne!(colour(&dev, 0x81), at(&dev, 112 + 8, 168 + 8));
    // The S2 lorry on open ground (nothing in B1 there) is visible: body pixel at row 8, col 4
    assert_eq!(colour(&dev, 0x91), at(&dev, 190 + 4, 100 + 8));
    insta::assert_binary_snapshot!("iso_sprite_demo.png", frame_png(&dev));
}

/// Video memory reads one isometric layer makes for each of the 66 groups of a line (two in the
/// porch, 64 while the line is drawn): a tile word per cell touched, plus a tilemap entry for each
/// (or, with `cache`, whenever the cell is not the one looked up last).
pub fn iso_group_reads(scroll_x: i32, plane_y: i32, cache: bool) -> Vec<u32> {
    use crate::iso::plan_group;
    let mut cached = None;
    (0..66)
        .map(|g| {
            let plan = plan_group(((scroll_x >> 2) + g) * 4, plane_y);
            let mut reads = 0;
            for cell in &plan.cells[..usize::from(plan.cell_count)] {
                reads += 1;
                if !cache || cached != Some(*cell) {
                    reads += 1;
                    cached = Some(*cell);
                }
            }
            reads
        })
        .collect()
}

#[test]
fn entry_cache_reads_match_the_model() {
    for (b1, b2) in [
        ((0, 0), (0, 0)),
        ((13, 7), (500, 300)),
        ((1023, 5), (77, 1000)),
    ] {
        let mut dev = iso_device();
        fill_random_vram(&mut dev, 4242);
        set_scroll(&mut dev, b1, b2);
        let line = 100_u16;
        let end_of_visible = dev.line_preamble_clocks + dev.line_visible_clocks;
        while !(dev.line == line && dev.line_clock >= end_of_visible) {
            dev.poll(BusAssertions::default(), true);
        }
        for (i, (sx, sy)) in [b1, b2].into_iter().enumerate() {
            let expected: u32 = iso_group_reads(
                i32::from(sx),
                i32::from(line) + i32::from(sy),
                crate::ISO_ENTRY_CACHE,
            )
            .iter()
            .sum();
            assert_eq!(
                expected, dev.iso.layers[i].reads,
                "layer {i}, scrolls {b1:?} {b2:?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Design studies: 2 bit ground tiles and an orthogonal third layer (not part of the Mode 2 prototype)
// ---------------------------------------------------------------------------------------------

/// A 2 bit tile in 32 words, one word per eight pixel staircase, would pack perfectly, and how many
/// cells does an eight pixel group touch (it needs a tile fetch for each)?
#[test]
fn study_two_bit_tiles_with_eight_pixel_groups() {
    use crate::iso::{cell_of, texel_at};
    use std::collections::HashSet;

    // Perfect packing: 2 groups x 16 anchors, each covering 8 texels, must cover the tile once
    let mut covered = HashSet::new();
    for x0 in [0, 8] {
        for u0 in 0..16 {
            let y = u0 - (x0 >> 1);
            for k in 0..8 {
                let (u, v) = texel_at(x0 + k, y);
                assert!(covered.insert((u & 15, v & 15)), "texel covered twice");
            }
        }
    }
    assert_eq!(256, covered.len());

    let mut histogram = [0_u32; 9];
    for x0 in (0..512).step_by(8) {
        for y in 0..512 {
            let cells: HashSet<_> = (0..8)
                .map(|k| {
                    let (u, v) = texel_at(x0 + k, y);
                    cell_of(u, v)
                })
                .collect();
            histogram[cells.len()] += 1;
        }
    }
    let total: u32 = histogram.iter().sum();
    println!(
        "cells touched by an eight pixel group: {:?} (of {total})",
        &histogram[1..5]
    );
}

/// Reads per line for one isometric layer if the tilemap entry of the current cell is kept between
/// groups instead of being fetched again for every group (as the Mode 2 prototype does).
#[test]
fn study_tilemap_entry_cache() {
    use crate::iso::plan_group;
    let (mut min_reads, mut max_reads, mut total, mut count) = (u32::MAX, 0, 0_u64, 0_u64);
    let mut straddle_groups = 0_u32;
    for ax in 0..8 {
        for ay in 0..16 {
            let mut reads = 0_u32;
            let mut cached = None;
            for g in 0..64 {
                let plan = plan_group((ax + g) * 4, ay);
                for cell in &plan.cells[..usize::from(plan.cell_count)] {
                    reads += 1; // the tile word
                    if cached != Some(*cell) {
                        reads += 1; // the tilemap entry
                        cached = Some(*cell);
                    }
                }
                straddle_groups += u32::from(plan.cell_count == 2);
            }
            min_reads = min_reads.min(reads);
            max_reads = max_reads.max(reads);
            total += u64::from(reads);
            count += 1;
        }
    }
    println!(
        "one layer with an entry cache: reads per line min {min_reads}, mean {:.1}, max {max_reads} (prototype: 128 to 192 of 256 slots; straddling groups {:.2}%)",
        total as f64 / count as f64,
        100.0 * f64::from(straddle_groups) / (count as f64 * 64.0)
    );
}

/// Smallest number of groups a layer's fetches must run ahead of the picture for every deadline to
/// be met, given the spare slots per time group and the reads each group needs. Group `g` must be
/// fetched by time group `g - lead` (2 for an isometric layer, 1 for the status layer); before time 0
/// (the porch) there are 8 slots.
fn lookahead_needed(free: &[u32], demand: &[u32], lead: i32) -> Option<usize> {
    'candidates: for lookahead in 0..=8_i32 {
        let mut t = -(lookahead + 2);
        let slots_at = |t: i32| if t < 0 { 8 } else { free[t as usize] };
        let mut slots = slots_at(t);
        for (group, reads) in demand.iter().enumerate() {
            for _ in 0..*reads {
                t = t.max(group as i32 - lead - lookahead);
                while slots == 0 {
                    t += 1;
                    if t > group as i32 - lead {
                        continue 'candidates;
                    }
                    slots = slots_at(t);
                }
                slots -= 1;
            }
        }
        return Some(lookahead as usize);
    }
    None
}

/// Run with `cargo test --release -- --ignored study_third --nocapture` (about 2 million cases).
#[test]
#[ignore = "slow in a debug build"]
fn study_third_isometric_layer() {
    for cache in [false, true] {
        let patterns: Vec<Vec<u32>> = (0..8)
            .flat_map(|ax| (0..16).map(move |ay| (ax, ay)))
            .map(|(ax, ay)| iso_group_reads(ax * 4, ay, cache))
            .collect();
        let mut histogram = [0_u32; 10];
        for a in &patterns {
            for b in &patterns {
                let free: Vec<u32> = (0..64).map(|t| 8 - a[t + 2] - b[t + 2]).collect();
                for c in &patterns {
                    let needed = lookahead_needed(&free, c, 2).map_or(9, |d| d);
                    histogram[needed] += 1;
                }
            }
        }
        println!(
            "third isometric layer, entry cache {cache}: lookahead histogram (index 9 = does not fit): {histogram:?}"
        );
    }
}

/// Reads per line for a 2 bit layer (eight pixel groups, 32 per line) with the entry cache.
#[test]
fn study_two_bit_layer_reads_per_line() {
    use crate::iso::{cell_of, texel_at};
    let (mut min, mut max, mut total, mut count) = (u32::MAX, 0, 0_u64, 0_u64);
    for ax in 0..4 {
        for ay in 0..16 {
            let mut cached = None;
            let mut reads = 0_u32;
            for g in 0..32 {
                let mut cells = Vec::new();
                for k in 0..8 {
                    let (u, v) = texel_at((ax + g) * 8 + k, ay);
                    let cell = cell_of(u, v);
                    if !cells.contains(&cell) {
                        cells.push(cell);
                    }
                }
                for cell in cells {
                    reads += 1;
                    if cached != Some(cell) {
                        reads += 1;
                        cached = Some(cell);
                    }
                }
            }
            min = min.min(reads);
            max = max.max(reads);
            total += u64::from(reads);
            count += 1;
        }
    }
    println!(
        "2 bit layer: reads per line min {min}, mean {:.1}, max {max} (4 bit layer: 88, 92.9, 97)",
        total as f64 / count as f64
    );
}

/// How many groups ahead must the status layer's fetches run so that none misses its deadline? The
/// layer needs a tile word for every group and a tilemap entry for every second group, and takes
/// the slots the two isometric layers leave spare. Exhaustive over all pairs of layer positions
/// (a layer's pattern repeats every 32 plane pixels in x and 16 lines in y).
#[test]
fn study_status_layer_queue_depth() {
    let demand: Vec<u32> = (0..64).map(|g| if g % 2 == 0 { 2 } else { 1 }).collect();
    for cache in [false, true] {
        let patterns: Vec<Vec<u32>> = (0..8)
            .flat_map(|ax| (0..16).map(move |ay| (ax, ay)))
            .map(|(ax, ay)| iso_group_reads(ax * 4, ay, cache))
            .collect();
        let mut histogram = [0_u32; 10];
        for a in &patterns {
            for b in &patterns {
                let free: Vec<u32> = (0..64).map(|t| 8 - a[t + 2] - b[t + 2]).collect();
                let needed = lookahead_needed(&free, &demand, 1).map_or(9, |d| d + 1);
                histogram[needed] += 1;
            }
        }
        println!(
            "status layer, entry cache {cache}: queue depth in groups (index 9 = does not fit): {histogram:?}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The status layer (background 3 in Mode 2)
// ---------------------------------------------------------------------------------------------

const B3_TILEMAP: u16 = 0x8800;
const B3_TILES: u16 = 0xB000;

fn fill_random_status_layer(dev: &mut VideoDevice, seed: u32) {
    let mut rng = XorShift(seed);
    let index = |addr: u16| usize::from(addr - 0x8000);
    for i in 0..1024 {
        // Tile 0-7, any palette select, priority and flips (flips are ignored)
        dev.vram[index(B3_TILEMAP) + i] = (rng.next() & 0xFC00) as u16 | (rng.next() % 8) as u16;
    }
    for i in 0..8 * 16 {
        dev.vram[index(B3_TILES) + i] = (rng.next() & rng.next()) as u16;
    }
    dev.ppu_registers.b3_tilemap_addr = B3_TILEMAP;
    dev.ppu_registers.b3_tile_addr = B3_TILES;
    dev.ppu_registers.b3_palette_config = PaletteRegister::new()
        .with_palette_size(PaletteSize::Eight)
        .with_palette_offset((rng.next() & 0xFF) as u8);
    dev.ppu_registers.base_config = BaseConfigRegister::new().with_video_mode(VIDEO_MODE_ISOMETRIC);
}

#[test]
fn status_layer_matches_reference_and_the_queue_never_runs_dry() {
    let cases = [
        ((0, 0), (0, 0)),
        ((1, 0), (2, 5)),
        ((3, 7), (13, 100)),
        ((1023, 1023), (500, 17)),
        ((130, 9), (131, 10)),
        ((4, 4), (7, 1022)),
        ((29, 3), (28, 14)),
        ((100, 8), (101, 9)),
    ];
    for (i, (b1, b2)) in cases.into_iter().enumerate() {
        let mut dev = iso_device();
        fill_random_vram(&mut dev, 0xABCD_0001 ^ (i as u32 * 104_729));
        fill_random_status_layer(&mut dev, 0x5555_0000 + i as u32);
        set_scroll(&mut dev, b1, b2);
        run_frame(&mut dev);
        assert_eq!(0, dev.hud.underruns, "queue ran dry, scrolls {b1:?} {b2:?}");
        assert_frame_matches_reference(&dev, &format!("status layer, scroll b1={b1:?} b2={b2:?}"));
    }
}

#[test]
fn status_layer_can_be_disabled_and_sits_behind_unless_priority_is_set() {
    let mut dev = iso_device();
    fill_random_vram(&mut dev, 77);
    fill_random_status_layer(&mut dev, 78);
    run_frame(&mut dev);
    assert_frame_matches_reference(&dev, "enabled");
    // Both priorities occur in the random tilemap, so the order is exercised both ways
    let priorities: Vec<u16> = (0..1024)
        .map(|i| (dev.vram[usize::from(B3_TILEMAP - 0x8000) + i] >> 13) & 1)
        .collect();
    assert!(priorities.contains(&0) && priorities.contains(&1));
    dev.ppu_registers.base_config = BaseConfigRegister::new()
        .with_video_mode(VIDEO_MODE_ISOMETRIC)
        .with_b3_disable(1);
    run_frame(&mut dev);
    assert_frame_matches_reference(&dev, "disabled");
}

// ---------------------------------------------------------------------------------------------
// 2 bit isometric layers
// ---------------------------------------------------------------------------------------------

fn set_two_bit(dev: &mut VideoDevice, b1: bool, b2: bool) {
    let regs = &mut dev.ppu_registers;
    regs.b1_palette_config = regs.b1_palette_config.with_two_bit_tiles(u8::from(b1));
    regs.b2_palette_config = regs.b2_palette_config.with_two_bit_tiles(u8::from(b2));
}

#[test]
fn two_bit_layers_match_the_reference_in_every_combination() {
    let cases = [
        ((0, 0), (0, 0)),
        ((1, 0), (2, 5)),
        ((3, 7), (13, 100)),
        ((1023, 1023), (500, 17)),
        ((130, 9), (131, 10)),
        ((4, 4), (7, 1022)),
        ((36, 12), (40, 3)),
    ];
    for (b1_two_bit, b2_two_bit) in [(true, false), (false, true), (true, true)] {
        for (i, (b1, b2)) in cases.into_iter().enumerate() {
            let mut dev = iso_device();
            fill_random_vram(&mut dev, 0x2B17 ^ (i as u32 * 7919));
            fill_random_status_layer(&mut dev, 0x77 + i as u32);
            set_two_bit(&mut dev, b1_two_bit, b2_two_bit);
            set_scroll(&mut dev, b1, b2);
            run_frame(&mut dev);
            assert_eq!(0, dev.hud.underruns);
            assert_frame_matches_reference(
                &dev,
                &format!("2 bit b1={b1_two_bit} b2={b2_two_bit}, scroll b1={b1:?} b2={b2:?}"),
            );
        }
    }
}

/// The ground of the demo scene needs only three colours, so it fits 2 bit tiles.
fn ground_art_three_colours(x: i32, y: i32) -> u8 {
    match ground_art(x, y) {
        3 | 4 => 2, // road
        5 | 6 => 3, // water
        _ => 1,     // grass, sand and outlines
    }
}

#[test]
fn two_bit_ground_takes_less_video_memory_and_renders_exactly() {
    let mut dev = iso_device();
    demo_palette(&mut dev);
    for (i, colour) in [
        rgb5(0, 0, 0),
        rgb5(8, 20, 6),
        rgb5(11, 11, 12),
        rgb5(6, 12, 26),
    ]
    .into_iter()
    .enumerate()
    {
        dev.palette[0x20 + i] = colour;
    }
    // Ground on background 2 as 2 bit tiles with a four colour palette at 0x20
    dev.ppu_registers.b2_palette_config = PaletteRegister::new()
        .with_palette_size(PaletteSize::Four)
        .with_palette_offset(0x20)
        .with_two_bit_tiles(1);
    let ground2 = bake_layer_2bit(256, 224, 0, 0, &ground_art_three_colours);
    let ground4 = bake_layer(256, 224, 0, 0, &|x, y| ground_art_three_colours(x, y));
    let base = |addr: u16| usize::from(addr - 0x8000);
    for (i, entry) in ground2.tilemap.iter().enumerate() {
        dev.vram[base(B2_TILEMAP) + i] = *entry;
    }
    for (t, words) in ground2.tiles.iter().enumerate() {
        let start = base(B2_TILES) + t * usize::from(TILE_WORDS_2BIT);
        dev.vram[start..start + words.len()].copy_from_slice(words);
    }
    let words2 = ground2.tiles.len() * usize::from(TILE_WORDS_2BIT);
    let words4 = ground4.tiles.len() * usize::from(TILE_WORDS);
    println!(
        "ground layer: {} tiles, {words2} words as 2 bit tiles against {} tiles, {words4} words as 4 bit tiles",
        ground2.tiles.len(),
        ground4.tiles.len()
    );
    assert_eq!(ground2.tiles.len(), ground4.tiles.len());
    assert_eq!(words4, 2 * words2);

    run_frame(&mut dev);
    for y in 0..i32::from(HEIGHT_PIXELS) {
        for x in 0..i32::from(WIDTH_PIXELS) {
            let want = dev.palette[0x20 + usize::from(ground_art_three_colours(x, y))];
            assert_eq!(
                unpack_rgb(dev.buffer[(x + y * i32::from(WIDTH_PIXELS)) as usize]),
                unpack_rgb(ppu_colour_to_minifb_rgb(want)),
                "x={x} y={y}"
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Mode 3: two orthogonal backgrounds in front of one isometric background
// ---------------------------------------------------------------------------------------------

/// Reference for an orthogonal background: a 32 by 32 tilemap of 8 by 8 tiles that wraps.
fn orthogonal_reference_sample(
    read: &dyn Fn(u16) -> u16,
    tilemap_base: u16,
    tile_base: u16,
    two_bit: bool,
    x: i32,
    y: i32,
) -> (u8, TilemapEntry) {
    let (x, y) = ((x & 255) as u16, (y & 255) as u16);
    let entry: TilemapEntry = read(tilemap_base + (y >> 3) * 32 + (x >> 3)).into();
    if two_bit {
        let word = read(tile_base + entry.tile_index() * 8 + (y & 7));
        (((word >> (14 - 2 * (x & 7))) & 3) as u8, entry)
    } else {
        let word = read(tile_base + entry.tile_index() * 16 + (y & 7) * 2 + ((x & 7) >> 2));
        (((word >> (12 - 4 * (x & 3))) & 0xF) as u8, entry)
    }
}

fn expected_colour_mode_3(dev: &VideoDevice, x: i32, y: i32) -> u32 {
    let regs = &dev.ppu_registers;
    let read = |addr: u16| dev.read_address(u32::from(addr));
    let disabled = [
        regs.base_config.b1_disable(),
        regs.base_config.b2_disable(),
        regs.base_config.b3_disable(),
    ];
    let orthogonal = [
        (
            regs.b1_tilemap_addr,
            regs.b1_tile_addr,
            regs.b1_scroll_x,
            regs.b1_scroll_y,
            regs.b1_palette_config,
        ),
        (
            regs.b2_tilemap_addr,
            regs.b2_tile_addr,
            regs.b2_scroll_x,
            regs.b2_scroll_y,
            regs.b2_palette_config,
        ),
    ];
    // Sample both orthogonal layers; a tile with its priority bit set is in front of everything
    let mut samples = Vec::new();
    for (i, (map, tiles, sx, sy, config)) in orthogonal.into_iter().enumerate() {
        if disabled[i] == 1 {
            continue;
        }
        let (value, entry) = orthogonal_reference_sample(
            &read,
            map,
            tiles,
            config.two_bit_tiles() == 1,
            x + i32::from(sx.scroll_amount()),
            y + i32::from(sy.scroll_amount()),
        );
        if value > 0 {
            let addr = resolve_palette_addr(entry.palette_select(), config).wrapping_add(value);
            samples.push((addr, entry.priority() == 1));
        }
    }
    let front = samples.iter().find(|(_, front)| *front);
    if let Some((addr, _)) = front.or_else(|| samples.first()) {
        return ppu_colour_to_minifb_rgb(dev.palette[*addr as usize]);
    }
    let mut addr = 0;
    let mut shade = 0;
    if disabled[2] == 0 {
        let sample = if regs.b3_palette_config.two_bit_tiles() == 1 {
            reference_sample_2bit
        } else {
            reference_sample
        };
        let (value, entry) = sample(
            &read,
            regs.b3_tilemap_addr,
            regs.b3_tile_addr,
            x + i32::from(regs.b3_scroll_x.scroll_amount()),
            y + i32::from(regs.b3_scroll_y.scroll_amount()),
        );
        if value > 0 {
            addr = resolve_palette_addr(entry.palette_select(), regs.b3_palette_config)
                .wrapping_add(value);
            shade = crate::entry_shade(entry);
        }
    }
    ppu_colour_to_minifb_rgb(crate::shade_pixel(dev.palette[addr as usize], shade))
}

fn mode_3_device(seed: u32, two_bit: [bool; 3]) -> VideoDevice {
    let mut dev = iso_device();
    dev.ppu_registers.base_config =
        BaseConfigRegister::new().with_video_mode(VIDEO_MODE_ISO_ORTHOGONAL);
    let mut rng = XorShift(seed);
    let index = |addr: u16| usize::from(addr - 0x8000);
    // Orthogonal backgrounds 1 and 2: tiles 0-7 (an orthogonal tile is 16 words, or 8 with 2 bit tiles)
    for (map, tiles) in [(B1_TILEMAP, B1_TILES), (B2_TILEMAP, B2_TILES)] {
        for i in 0..1024 {
            dev.vram[index(map) + i] = (rng.next() & 0xFC00) as u16 | (rng.next() % 8) as u16;
        }
        for i in 0..8 * 16 {
            dev.vram[index(tiles) + i] = (rng.next() & rng.next() & rng.next()) as u16;
        }
    }
    // Isometric background 3: tiles 0-5
    for i in 0..1024 {
        dev.vram[index(B3_TILEMAP) + i] = (rng.next() & 0xFC00) as u16 | (rng.next() % 6) as u16;
    }
    for i in 0..6 * usize::from(TILE_WORDS) {
        dev.vram[index(B3_TILES) + i] = (rng.next() & rng.next()) as u16;
    }
    fill_distinct_palette(&mut dev);
    let config = |two_bit: bool, size, offset| {
        PaletteRegister::new()
            .with_palette_size(size)
            .with_palette_offset(offset)
            .with_two_bit_tiles(u8::from(two_bit))
    };
    let regs = &mut dev.ppu_registers;
    regs.b1_tilemap_addr = B1_TILEMAP;
    regs.b2_tilemap_addr = B2_TILEMAP;
    regs.b3_tilemap_addr = B3_TILEMAP;
    regs.b1_tile_addr = B1_TILES;
    regs.b2_tile_addr = B2_TILES;
    regs.b3_tile_addr = B3_TILES;
    regs.b1_palette_config = config(two_bit[0], PaletteSize::Eight, (rng.next() & 0xFF) as u8);
    regs.b2_palette_config = config(two_bit[1], PaletteSize::Four, (rng.next() & 0xFF) as u8);
    regs.b3_palette_config = config(two_bit[2], PaletteSize::Sixteen, (rng.next() & 0xFF) as u8);
    dev
}

#[test]
fn mode_3_matches_the_reference_in_every_tile_format_and_scroll() {
    let scrolls = [
        [(0, 0), (0, 0), (0, 0)],
        [(1, 0), (2, 5), (3, 7)],
        [(13, 100), (255, 255), (500, 17)],
        [(1023, 1023), (4, 4), (130, 9)],
        [(37, 11), (36, 12), (29, 3)],
    ];
    let formats = [
        [false, false, false],
        [true, false, false],
        [false, true, true],
        [true, true, true],
    ];
    for (f, two_bit) in formats.into_iter().enumerate() {
        for (i, [s1, s2, s3]) in scrolls.into_iter().enumerate() {
            let mut dev = mode_3_device(0x00C0_FFEE ^ ((f * 31 + i) as u32 * 7919), two_bit);
            set_scroll(&mut dev, s1, s2);
            dev.ppu_registers.b3_scroll_x = dev.ppu_registers.b3_scroll_x.with_scroll_amount(s3.0);
            dev.ppu_registers.b3_scroll_y = dev.ppu_registers.b3_scroll_y.with_scroll_amount(s3.1);
            run_frame(&mut dev);
            for y in 0..i32::from(HEIGHT_PIXELS) {
                for x in 0..i32::from(WIDTH_PIXELS) {
                    let got = dev.buffer[(x + y * i32::from(WIDTH_PIXELS)) as usize];
                    assert_eq!(
                        unpack_rgb(expected_colour_mode_3(&dev, x, y)),
                        unpack_rgb(got),
                        "mode 3, 2 bit {two_bit:?}, scrolls {s1:?} {s2:?} {s3:?}: pixel ({x}, {y})"
                    );
                }
            }
        }
    }
}

#[test]
fn mode_3_layers_can_be_disabled() {
    let mut dev = mode_3_device(5, [false, true, false]);
    for (b1, b2, b3) in [(1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 1)] {
        dev.ppu_registers.base_config = BaseConfigRegister::new()
            .with_video_mode(VIDEO_MODE_ISO_ORTHOGONAL)
            .with_b1_disable(b1)
            .with_b2_disable(b2)
            .with_b3_disable(b3);
        run_frame(&mut dev);
        run_frame(&mut dev);
        for y in (0..224).step_by(3) {
            for x in 0..256 {
                assert_eq!(
                    unpack_rgb(expected_colour_mode_3(&dev, x, y)),
                    unpack_rgb(dev.buffer[(x + y * 256) as usize]),
                    "disables {b1}{b2}{b3}: pixel ({x}, {y})"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Shading: the flip bits of an isometric layer's tilemap entries
// ---------------------------------------------------------------------------------------------

#[test]
fn isometric_tiles_darken_and_lighten_with_the_flip_bits() {
    let mut dev = iso_device();
    // Background 2 is solid tile 1 (value 1, palette address 0x21) everywhere
    for i in 0..1024 {
        dev.vram[usize::from(B2_TILEMAP - 0x8000) + i] = 1;
    }
    for w in 0..usize::from(TILE_WORDS) {
        dev.vram[usize::from(B2_TILES - 0x8000) + usize::from(TILE_WORDS) + w] = 0x1111;
    }
    dev.ppu_registers.b2_palette_config = PaletteRegister::new()
        .with_palette_size(PaletteSize::Sixteen)
        .with_palette_offset(0x20);
    dev.palette[0x21] = rgb5(20, 10, 5);
    // Cells in the top left corner of the map get different flip bits (they are on screen)
    let flags = [
        (0_u16, "normal"),
        (0x8000, "dark"),
        (0x4000, "light"),
        (0xC000, "both"),
    ];
    for (cu, (flag, _)) in flags.into_iter().enumerate() {
        for cv in 0..3 {
            dev.vram[usize::from(B2_TILEMAP - 0x8000) + cv * 32 + cu] = 1 | flag;
        }
    }
    run_frame(&mut dev);
    let mut seen = std::collections::HashSet::new();
    for y in 0..224 {
        for x in 0..256 {
            seen.insert(unpack_rgb(dev.buffer[x + y * 256]));
        }
    }
    let channel = |p: PpuPixel| unpack_rgb(ppu_colour_to_minifb_rgb(p));
    for (name, colour) in [
        ("normal", rgb5(20, 10, 5)),
        ("dark", rgb5(10, 5, 2)),
        ("light", rgb5(25, 20, 18)),
    ] {
        assert!(
            seen.contains(&channel(colour)),
            "{name} tiles should appear on screen"
        );
    }
}

#[test]
fn shading_demo_fog_of_war_and_a_vehicle_shadow() {
    let mut dev = iso_device();
    dev.ppu_registers.base_config = BaseConfigRegister::new()
        .with_video_mode(VIDEO_MODE_ISOMETRIC)
        .with_b3_disable(1)
        .with_shadow_enable(1);
    demo_palette(&mut dev);
    load_sprite_art(&mut dev);
    dev.ppu_registers.tile_size = TileSizeRegister::new()
        .with_s1_size(TileSize::SixteenBySixteen)
        .with_s2_size(TileSize::SixteenBySixteen)
        .with_s3_size(TileSize::SixteenBySixteen);
    dev.ppu_registers.s_tile_addr = SPRITE_TILES;

    let building_scroll = (0, 8);
    let mut ground = bake_layer(256, 224, 0, 0, &ground_art);
    // Fog of war: the cells of the map's far corner are darkened by the vertical flip bit
    for cv in 0..32_u16 {
        for cu in 0..32_u16 {
            let (screen_x, screen_y) = (
                16 * (i32::from(cu) - i32::from(cv)),
                8 * (i32::from(cu) + i32::from(cv)),
            );
            if screen_x > 56 && ground.tilemap[usize::from(cv * 32 + cu)] != 0 {
                let _ = screen_y;
                ground.tilemap[usize::from(cv * 32 + cu)] |= 0x8000;
            }
        }
    }
    let buildings = bake_layer(
        256,
        224,
        building_scroll.0,
        building_scroll.1,
        &building_art,
    );
    load_layer(&mut dev, &ground, B2_TILEMAP, B2_TILES);
    load_layer(&mut dev, &buildings, B1_TILEMAP, B1_TILES);
    set_scroll(&mut dev, (0, 8), (0, 0));

    let sprite = |x, y, tile, palette, layer, flip_h| SpriteEntry {
        x,
        y,
        tile,
        palette,
        flip_h,
        flip_v: false,
        layer,
    };
    let table = [
        sprite(14, 176, 0, 1, 0, true),  // S1: the lorry
        sprite(14, 181, 2, 7, 1, false), // S2: its shadow, in front of the ground
        sprite(30, 120, 1, 0, 0, false), // a tree
    ];
    for (i, entry) in table.iter().enumerate() {
        let (w0, w1) = entry.to_words();
        dev.write_address(SPRITE_RAM_BASE + 2 * i as u32, w0);
        dev.write_address(SPRITE_RAM_BASE + 2 * i as u32 + 1, w1);
    }
    run_frame(&mut dev);
    insta::assert_binary_snapshot!("iso_shading_demo.png", frame_png(&dev));
}

// ---------------------------------------------------------------------------------------------
// Moving around the map (manual, "Moving Around the Map")
// ---------------------------------------------------------------------------------------------

/// A move of `(du, dv)` texels along the map is `dx = du - dv`, `dy = (du + dv) / 2` in scroll.
#[test]
fn scrolling_by_the_documented_amounts_moves_the_view_along_the_map_axes() {
    for (x, y) in [(0, 0), (3, 9), (255, 223), (1000, 1000)] {
        let (u, v) = texel_at(x, y);
        for (du, dv) in [
            (2, 0),
            (0, 2),
            (5, 3),
            (-4, 6),
            (16, 16),
            (-7, -1),
            (10, -10),
        ] {
            // The documented rule needs du + dv even (a whole number of pixels)
            let (sx, sy) = (du - dv, i32::midpoint(du, dv));
            assert_eq!(
                texel_at(x + sx, y + sy),
                (u + du, v + dv),
                "pixel ({x}, {y}) moved by ({du}, {dv})"
            );
        }
    }
}

/// Bringing `SCRX` back by 1024 and `SCRY` back by 512 leaves the picture unchanged, for every
/// map size (width and height of 32 or 64 cells).
#[test]
fn wrapping_scrx_by_1024_with_scry_by_512_leaves_the_picture_unchanged() {
    for (w, h) in [(32, 32), (64, 32), (32, 64), (64, 64)] {
        let (mw, mh) = (16 * w, 16 * h);
        for (x, y) in [(0, 0), (1, 1), (255, 223), (1023, 1023), (1200, 40)] {
            let (u, v) = texel_at(x, y);
            let (u2, v2) = texel_at(x - 1024, y - 512);
            assert_eq!(
                (u.rem_euclid(mw), v.rem_euclid(mh)),
                (u2.rem_euclid(mw), v2.rem_euclid(mh)),
                "map {w}x{h} cells at plane pixel ({x}, {y})"
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Vertical blanking flag and current line
// ---------------------------------------------------------------------------------------------

/// `VLINE` (0x0018) holds the line the beam is on, and `STATUS` bit 7 (`VBL`) is set from line 224
/// to the end of the frame, both changing at the first cycle of the line.
#[test]
fn vline_and_the_vbl_flag_follow_the_beam() {
    let mut dev = iso_device();
    let cycles_per_line = u32::from(dev.clocks_per_line) / 2;
    let mut seen = [false; TOTAL_LINES as usize];
    for _ in 0..u32::from(TOTAL_LINES) * cycles_per_line {
        dev.poll(BusAssertions::default(), true);
        let line = dev.read_address(0x0018);
        let status = dev.read_address(0x0017);
        assert_eq!(line, dev.line, "VLINE");
        assert_eq!(
            (status >> 7) & 1,
            u16::from(dev.line >= HEIGHT_PIXELS),
            "VBL on line {}",
            dev.line
        );
        seen[usize::from(line)] = true;
    }
    assert!(seen.iter().all(|s| *s), "every line was seen");
    // Both registers are read only
    dev.write_address(0x0018, 0x55);
    dev.write_address(0x0017, 0xFFFF);
    assert_eq!(dev.read_address(0x0018), dev.line);
    assert_eq!(
        (dev.read_address(0x0017) >> 7) & 1,
        u16::from(dev.line >= HEIGHT_PIXELS)
    );
}
