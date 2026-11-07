//! Device level tests for sprites in both modes: the composited frame is checked against a
//! straightforward per-pixel reference, the layer order against the backgrounds is checked, and the
//! fetch budget is measured.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use crate::sprite::{
    SpriteEntry, FETCH_BUDGET, MAX_SPRITES_PER_LINE, SPRITE_COUNT, SPRITE_PALETTE_BASE,
    SPRITE_RAM_BASE,
};
use crate::types::{
    BaseConfigRegister, PaletteRegister, PaletteSize, PpuPixel, TileSize, TileSizeRegister,
    VIDEO_MODE_ISOMETRIC, VIDEO_MODE_ISO_ORTHOGONAL, VIDEO_MODE_ORTHOGONAL,
};
use crate::{
    new_video_device, ppu_colour_to_minifb_rgb, unpack_rgb, VideoDevice, HEIGHT_PIXELS,
    TOTAL_LINES, WIDTH_PIXELS,
};
use peripheral_bus::device::{BusAssertions, Device};
use peripheral_bus::memory_mapped_device::MemoryMapped;

const TILEMAPS: [u16; 3] = [0x8000, 0x8400, 0x8800];
const BG_TILES: [u16; 3] = [0x9000, 0xA000, 0xB000];
const SPRITE_TILES: u16 = 0xC000;

fn device(mode: u8) -> VideoDevice {
    let mut dev = new_video_device(21_477_272);
    dev.ppu_registers.base_config = BaseConfigRegister::new().with_video_mode(mode);
    dev.ppu_registers.b1_tilemap_addr = TILEMAPS[0];
    dev.ppu_registers.b2_tilemap_addr = TILEMAPS[1];
    dev.ppu_registers.b3_tilemap_addr = TILEMAPS[2];
    dev.ppu_registers.b1_tile_addr = BG_TILES[0];
    dev.ppu_registers.b2_tile_addr = BG_TILES[1];
    dev.ppu_registers.b3_tile_addr = BG_TILES[2];
    dev.ppu_registers.s_tile_addr = SPRITE_TILES;
    for (i, entry) in dev.palette.iter_mut().enumerate() {
        *entry = PpuPixel::from(((i as u16 + 1) * 127) & 0x7FFF);
    }
    // Backgrounds use palette offsets 0x10, 0x20, 0x30 with 16 entry palettes
    let cfg = |offset| {
        PaletteRegister::new()
            .with_palette_size(PaletteSize::Sixteen)
            .with_palette_offset(offset)
    };
    dev.ppu_registers.b1_palette_config = cfg(0x10);
    dev.ppu_registers.b2_palette_config = cfg(0x20);
    dev.ppu_registers.b3_palette_config = cfg(0x30);
    dev
}

fn run_frame(dev: &mut VideoDevice) {
    let polls = u32::from(TOTAL_LINES) * u32::from(dev.clocks_per_line) / 2;
    for _ in 0..polls {
        dev.poll(BusAssertions::default(), true);
    }
}

/// Run until the sprite engine has built `line` (the following poll is the first of the line).
fn run_to_line(dev: &mut VideoDevice, line: u16) {
    while !(dev.line == line && dev.line_clock > 0) {
        dev.poll(BusAssertions::default(), true);
    }
}

fn put_sprite(dev: &mut VideoDevice, index: usize, entry: SpriteEntry) {
    let (w0, w1) = entry.to_words();
    dev.write_address(SPRITE_RAM_BASE + 2 * index as u32, w0);
    dev.write_address(SPRITE_RAM_BASE + 2 * index as u32 + 1, w1);
}

fn pixel(dev: &VideoDevice, x: usize, y: usize) -> (u8, u8, u8) {
    unpack_rgb(dev.buffer[x + y * usize::from(WIDTH_PIXELS)])
}

fn colour(dev: &VideoDevice, palette_addr: u8) -> (u8, u8, u8) {
    unpack_rgb(ppu_colour_to_minifb_rgb(
        dev.palette[usize::from(palette_addr)],
    ))
}

fn vram(dev: &mut VideoDevice, address: u16) -> &mut u16 {
    &mut dev.vram[usize::from(address - 0x8000)]
}

/// Sprite sheet: sprite tile `t` of size `size` has the nibble `(row + col + t) % 15 + 1` at each
/// position, so every position is opaque and different from its neighbours.
fn pattern_nibble(t: usize, row: usize, col: usize) -> u16 {
    ((row + col + t) % 15 + 1) as u16
}

fn load_sprite_sheet(dev: &mut VideoDevice, tiles: usize, size: usize) {
    let words_per_row = size / 4;
    for t in 0..tiles {
        for row in 0..size {
            for w in 0..words_per_row {
                let word = (0..4).fold(0, |acc, n| (acc << 4) | pattern_nibble(t, row, w * 4 + n));
                // 16 by 16 tiles are four quadrants of 16 words, as for background tiles
                let quadrant = if size == 16 { 2 * (row / 8) + w / 2 } else { 0 };
                let address = SPRITE_TILES
                    + (t * size * words_per_row + 16 * quadrant + 2 * (row % 8) + w % 2) as u16;
                *vram(dev, address) = word;
            }
        }
    }
}

/// Independent per-pixel reference: the sprite colour at `(x, y)`, front sprite first.
fn reference_sprite_pixel(table: &[SpriteEntry], sizes: [usize; 3], x: i32, y: i32) -> Option<u8> {
    let mut best: Option<(u8, usize, u8)> = None; // (layer, index, palette address)
    for (index, e) in table.iter().enumerate() {
        if e.layer >= 3 {
            continue;
        }
        let size = sizes[usize::from(e.layer)] as i32;
        let row = (y - i32::from(e.y)).rem_euclid(256);
        let col = x - i32::from(e.x);
        if row >= size || !(0..size).contains(&col) {
            continue;
        }
        let row = if e.flip_v { size - 1 - row } else { row };
        let col = if e.flip_h { size - 1 - col } else { col };
        let value = pattern_nibble(usize::from(e.tile), row as usize, col as usize) as u8;
        let addr = SPRITE_PALETTE_BASE + (e.palette << 4) + value;
        if best.is_none_or(|(l, i, _)| (e.layer, index) < (l, i)) {
            best = Some((e.layer, index, addr));
        }
    }
    best.map(|(_, _, addr)| addr)
}

fn scattered_sprites() -> Vec<SpriteEntry> {
    let mk = |x, y, tile, layer, flip_h, flip_v| SpriteEntry {
        x,
        y,
        tile,
        palette: tile % 7, // palette 7 is reserved for the shadow colour
        flip_h,
        flip_v,
        layer,
    };
    vec![
        mk(10, 10, 0, 0, false, false),
        mk(14, 12, 1, 0, true, false),
        mk(100, 100, 2, 1, false, true),
        mk(104, 96, 3, 2, true, true),
        mk(-4, 50, 4, 0, false, false),   // half off the left edge
        mk(252, 60, 5, 1, false, false),  // half off the right edge
        mk(40, 252, 6, 0, false, false),  // wraps through the top
        mk(200, 215, 7, 2, false, false), // runs off the bottom
    ]
}

fn tile_size(size: usize) -> TileSize {
    if size == 8 {
        TileSize::EightByEight
    } else {
        TileSize::SixteenBySixteen
    }
}

/// All three sprite layers use `size` (8 or 16) pixel sprites.
fn all_layers_size(size: usize) -> TileSizeRegister {
    TileSizeRegister::new()
        .with_s1_size(tile_size(size))
        .with_s2_size(tile_size(size))
        .with_s3_size(tile_size(size))
}

fn assert_sprites_only_frame(mode: u8, size: usize) {
    let mut dev = device(mode);
    dev.ppu_registers.tile_size = all_layers_size(size);
    let pixel_sizes = [size; 3];
    load_sprite_sheet(&mut dev, 8, size);
    let table = scattered_sprites();
    for (i, entry) in table.iter().enumerate() {
        put_sprite(&mut dev, i, *entry);
    }
    run_frame(&mut dev);

    let mut covered = 0;
    for y in 0..usize::from(HEIGHT_PIXELS) {
        for x in 0..usize::from(WIDTH_PIXELS) {
            let want = reference_sprite_pixel(&table, pixel_sizes, x as i32, y as i32);
            let want_colour = colour(&dev, want.unwrap_or(0));
            covered += usize::from(want.is_some());
            assert_eq!(
                want_colour,
                pixel(&dev, x, y),
                "mode {mode} size {size}: pixel ({x}, {y})"
            );
        }
    }
    assert!(
        covered > 300,
        "the scene should have drawn something ({covered})"
    );
}

#[test]
fn sprites_render_like_the_reference_in_mode_1() {
    assert_sprites_only_frame(VIDEO_MODE_ORTHOGONAL, 8);
    assert_sprites_only_frame(VIDEO_MODE_ORTHOGONAL, 16);
}

#[test]
fn sprites_render_like_the_reference_in_mode_2() {
    assert_sprites_only_frame(VIDEO_MODE_ISOMETRIC, 8);
    assert_sprites_only_frame(VIDEO_MODE_ISOMETRIC, 16);
}

/// Fill background `layer` (0..3) so that every pixel is opaque (`solid`) or transparent.
fn fill_background(dev: &mut VideoDevice, mode: u8, layer: usize, solid: bool) {
    // Isometric layers (all of Mode 2's two, and Mode 3's background 3) use 64 word tiles
    let isometric =
        mode == VIDEO_MODE_ISOMETRIC || (mode == VIDEO_MODE_ISO_ORTHOGONAL && layer == 2);
    let words_per_tile = if isometric { 64 } else { 16 };
    for i in 0..1024 {
        *vram(dev, TILEMAPS[layer] + i) = u16::from(solid); // tile 1 is solid, tile 0 is empty
    }
    for w in 0..words_per_tile {
        *vram(dev, BG_TILES[layer] + words_per_tile as u16 + w as u16) = 0x1111;
    }
}

#[test]
fn sprite_layers_sit_between_the_backgrounds() {
    // Palette addresses: S1 0x87, S2 0x97, S3 0xA7 (the sprite pixel tested has value 7);
    // B1 0x11, B2 0x21, B3 0x31 (Mode 1 only)
    let cases: [(&str, [bool; 3]); 3] = [
        ("all backgrounds solid", [true, true, true]),
        ("B1 empty", [false, true, true]),
        ("B1 and B2 empty", [false, false, true]),
    ];
    for mode in [
        VIDEO_MODE_ORTHOGONAL,
        VIDEO_MODE_ISOMETRIC,
        VIDEO_MODE_ISO_ORTHOGONAL,
    ] {
        for (label, solid) in cases {
            let mut dev = device(mode);
            for (layer, &colour) in solid.iter().enumerate() {
                fill_background(&mut dev, mode, layer, colour);
            }
            load_sprite_sheet(&mut dev, 1, 8);
            // One solid-ish sprite per layer at the same place, and a copy of each beside it so the
            // layers can be seen separately
            for layer in 0..3_u8 {
                put_sprite(
                    &mut dev,
                    usize::from(layer),
                    SpriteEntry {
                        x: 100,
                        y: 50,
                        tile: 0,
                        palette: layer,
                        layer,
                        ..SpriteEntry::default()
                    },
                );
            }
            // Hide S1 and S2 in other cases by disabling them so that each layer is tested alone
            let hide = |dev: &mut VideoDevice, s1: bool, s2: bool| {
                dev.ppu_registers.base_config = BaseConfigRegister::new()
                    .with_video_mode(mode)
                    .with_s1_disable(u8::from(s1))
                    .with_s2_disable(u8::from(s2));
            };
            // All three sprite layers drawn: the frontmost visible one wins. (Two frames after each
            // change: the line the device is stopped on was built with the old settings.)
            hide(&mut dev, false, false);
            run_to_line(&mut dev, 53);
            let top = dev.sprite_line.pixels[100];
            assert!(
                top.opaque && top.layer == 0,
                "{label}: S1 is topmost of the sprites"
            );
            run_frame(&mut dev);
            run_frame(&mut dev);
            // Pattern nibble at row 3, col 3 of tile 0 is 7: sprite colour (palette << 4) + 0x80 + 7
            let sprite_addr = |layer: u8| SPRITE_PALETTE_BASE + (layer << 4) + 7;
            let bg_addr = |n: u8| 0x10 * (n + 1) + 1;
            let expected = |s1: bool, s2: bool, s3: bool| -> u8 {
                // front to back: S1 B1 S2 B2 S3 B3
                if s1 {
                    return sprite_addr(0);
                }
                if solid[0] {
                    return bg_addr(0);
                }
                if s2 {
                    return sprite_addr(1);
                }
                if solid[1] {
                    return bg_addr(1);
                }
                if s3 {
                    return sprite_addr(2);
                }
                if mode != VIDEO_MODE_ISOMETRIC && solid[2] {
                    return bg_addr(2);
                }
                0
            };
            assert_eq!(
                colour(&dev, expected(true, true, true)),
                pixel(&dev, 103, 53),
                "mode {mode}, {label}: all sprite layers"
            );
            // S1 off: S2 and S3 compete with the backgrounds
            hide(&mut dev, true, false);
            run_frame(&mut dev);
            run_frame(&mut dev);
            assert_eq!(
                colour(&dev, expected(false, true, true)),
                pixel(&dev, 103, 53),
                "mode {mode}, {label}: S1 disabled"
            );
            // S1 and S2 off: only S3 is left
            hide(&mut dev, true, true);
            run_frame(&mut dev);
            run_frame(&mut dev);
            assert_eq!(
                colour(&dev, expected(false, false, true)),
                pixel(&dev, 103, 53),
                "mode {mode}, {label}: S1 and S2 disabled"
            );
        }
    }
}

#[test]
fn per_line_limit_and_fetch_cost() {
    for mode in [VIDEO_MODE_ORTHOGONAL, VIDEO_MODE_ISOMETRIC] {
        for size in [8, 16] {
            let mut dev = device(mode);
            dev.ppu_registers.tile_size = all_layers_size(size);
            load_sprite_sheet(&mut dev, 8, size);
            // Every entry of the table sits on line 100
            for i in 0..SPRITE_COUNT {
                put_sprite(
                    &mut dev,
                    i,
                    SpriteEntry {
                        x: (i as i16 * 2) % 240,
                        y: 100,
                        tile: (i % 8) as u8,
                        ..SpriteEntry::default()
                    },
                );
            }
            run_to_line(&mut dev, 100);
            let stats = dev.sprite_stats;
            assert_eq!(SPRITE_COUNT, stats.in_range);
            assert_eq!(MAX_SPRITES_PER_LINE, stats.drawn);
            assert_eq!(SPRITE_COUNT - MAX_SPRITES_PER_LINE, stats.dropped_limit);
            assert_eq!(0, stats.dropped_time);
            assert_eq!((MAX_SPRITES_PER_LINE * size / 4) as u32, stats.fetch_cycles);
            assert!(stats.fetch_cycles <= FETCH_BUDGET);
            // A line the table does not touch costs nothing
            run_to_line(&mut dev, 120);
            assert_eq!(0, dev.sprite_stats.in_range);
        }
    }
}

#[test]
fn the_table_is_hidden_at_reset() {
    let mut dev = device(VIDEO_MODE_ORTHOGONAL);
    for line in [0, 100, 223] {
        run_to_line(&mut dev, line);
        assert_eq!(0, dev.sprite_stats.in_range);
    }
    assert_eq!(0xFFFF, dev.read_address(SPRITE_RAM_BASE));
}

/// How much video memory bandwidth Mode 2 leaves over during the visible part of a line.
///
/// Each layer has four fetch slots in a group of four pixels. It uses them for a tile word per
/// cell the group touches and a tilemap entry whenever the cell changes (see `iso_group_reads`);
/// the rest are free. The result depends only on the scroll positions, so it is measured over all
/// of them (a layer's pattern repeats every 32 plane pixels in x and 16 lines in y).
#[test]
fn mode_2_visible_slots_left_over_for_sprites() {
    let mut min_free = u32::MAX;
    let mut max_free = 0;
    let mut total = 0_u64;
    let mut lines = 0_u64;
    let layer_free: Vec<Vec<u32>> = (0..8)
        .flat_map(|ax| (0..16).map(move |ay| (ax, ay)))
        .map(|(ax, ay)| {
            let reads = crate::iso_tests::iso_group_reads(ax * 4, ay, crate::ISO_ENTRY_CACHE);
            (0..64).map(|t| 4 - reads[t + 2]).collect()
        })
        .collect();
    for a in &layer_free {
        for b in &layer_free {
            let free: u32 = a.iter().zip(b).map(|(x, y)| x + y).sum();
            min_free = min_free.min(free);
            max_free = max_free.max(free);
            total += u64::from(free);
            lines += 1;
        }
    }
    println!(
        "mode 2 free fetch slots per visible line: min {min_free}, mean {:.1}, max {max_free} (of 512)",
        total as f64 / lines as f64
    );
    assert!(min_free > 150 && max_free <= 512);
}

#[test]
fn shadow_sprite_pixels_darken_what_is_behind_their_layer() {
    for mode in [
        VIDEO_MODE_ORTHOGONAL,
        VIDEO_MODE_ISOMETRIC,
        VIDEO_MODE_ISO_ORTHOGONAL,
    ] {
        let mut dev = device(mode);
        dev.ppu_registers.base_config = BaseConfigRegister::new()
            .with_video_mode(mode)
            .with_shadow_enable(1);
        // Background 2 is solid (palette address 0x21 whichever mode it is)
        fill_background(&mut dev, mode, 1, true);
        dev.palette[0x21] = PpuPixel::new().with_r(20).with_g(10).with_b(5);
        // Sprite tile 5 is all shadow pixels (value 15) and is used with palette 7
        for w in 0..16 {
            *vram(&mut dev, SPRITE_TILES + 5 * 16 + w) = 0xFFFF;
        }
        // S2 sits in front of background 2, S3 behind it
        for (index, layer, x) in [(0, 1_u8, 40), (1, 2, 120)] {
            put_sprite(
                &mut dev,
                index,
                SpriteEntry {
                    x,
                    y: 50,
                    tile: 5,
                    palette: 7,
                    layer,
                    ..SpriteEntry::default()
                },
            );
        }
        run_frame(&mut dev);
        let backdrop = pixel(&dev, 20, 100);
        assert_eq!(
            colour(&dev, 0x21),
            backdrop,
            "mode {mode}: plain background"
        );
        assert_eq!(
            (10, 5, 2),
            {
                let (r, g, b) = pixel(&dev, 43, 53);
                (r >> 3, g >> 3, b >> 3)
            },
            "mode {mode}: shadow on S2 darkens background 2"
        );
        assert_eq!(
            backdrop,
            pixel(&dev, 123, 53),
            "mode {mode}: shadow on S3 is behind background 2"
        );
    }
}

#[test]
fn the_priority_bit_puts_an_orthogonal_tile_in_front_of_sprite_layer_1() {
    let mut dev = device(VIDEO_MODE_ISO_ORTHOGONAL);
    // Orthogonal background 1 is solid (palette address 0x11); the isometric background 3 is empty
    fill_background(&mut dev, VIDEO_MODE_ISO_ORTHOGONAL, 0, true);
    load_sprite_sheet(&mut dev, 1, 8);
    put_sprite(
        &mut dev,
        0,
        SpriteEntry {
            x: 100,
            y: 50,
            tile: 0,
            palette: 0,
            layer: 0,
            ..SpriteEntry::default()
        },
    );
    run_frame(&mut dev);
    // Without the priority bit the sprite on layer 1 is in front of background 1
    assert_eq!(colour(&dev, SPRITE_PALETTE_BASE + 7), pixel(&dev, 103, 53));
    for i in 0..1024 {
        *vram(&mut dev, TILEMAPS[0] + i) = 1 | 0x2000;
    }
    run_frame(&mut dev);
    assert_eq!(colour(&dev, 0x11), pixel(&dev, 103, 53));
}

#[test]
fn the_sprite_overflow_flag_is_sticky_until_the_next_frame() {
    let mut dev = device(VIDEO_MODE_ORTHOGONAL);
    load_sprite_sheet(&mut dev, 8, 8);
    // Not set at reset, and the register reads the same as every other register (bit 5, not bit 13)
    assert_eq!(0, dev.read_address(0x0017));
    // 33 sprites on line 100: one more than a line can hold
    for i in 0..33 {
        put_sprite(
            &mut dev,
            i,
            SpriteEntry {
                x: (i as i16) * 7,
                y: 100,
                tile: (i % 8) as u8,
                ..SpriteEntry::default()
            },
        );
    }
    run_to_line(&mut dev, 99);
    assert_eq!(0, dev.read_address(0x0017), "not before the line");
    run_to_line(&mut dev, 101);
    assert_eq!(1 << 5, dev.read_address(0x0017));
    // Still set later in the frame, even though no later line overflows
    run_to_line(&mut dev, 230);
    // (bit 7, vertical blanking, is also set by now)
    assert_eq!(1 << 5, dev.read_address(0x0017) & !(1 << 7));
    // Cleared at line 0 of the next frame, and set again only when a line overflows
    for i in 0..33 {
        put_sprite(
            &mut dev,
            i,
            SpriteEntry {
                layer: 3,
                ..SpriteEntry::default()
            },
        );
    }
    run_to_line(&mut dev, 5);
    assert_eq!(0, dev.read_address(0x0017));
}

#[test]
fn the_line_compare_interrupt_fires_once_per_frame_at_its_line() {
    use crate::{LINE_INTERRUPT, VSYNC_INTERRUPT};
    for (enable, expected) in [(1, vec![100_u16]), (0, vec![])] {
        let mut dev = device(VIDEO_MODE_ORTHOGONAL);
        dev.ppu_registers.base_config =
            BaseConfigRegister::new().with_line_interrupt_enable(enable);
        dev.write_address(0x0015, 100);
        let (mut line_hits, mut vsync_hits) = (Vec::new(), Vec::new());
        let polls = u32::from(TOTAL_LINES) * u32::from(dev.clocks_per_line) / 2;
        for _ in 0..polls {
            let (line, clock) = (dev.line, dev.line_clock);
            let assertions = dev.poll(BusAssertions::default(), true);
            if assertions.interrupt_assertion & LINE_INTERRUPT != 0 {
                line_hits.push((line, clock));
            }
            if assertions.interrupt_assertion & VSYNC_INTERRUPT != 0 {
                vsync_hits.push(line);
            }
        }
        assert_eq!(
            expected.iter().map(|l| (*l, 0)).collect::<Vec<_>>(),
            line_hits
        );
        assert_eq!(vec![224], vsync_hits);
    }
}

#[test]
fn the_palette_ends_at_0x60ff() {
    let mut dev = device(VIDEO_MODE_ORTHOGONAL);
    dev.write_address(0x60FF, 0x1234);
    dev.write_address(0x6100, 0x5678); // one past the end: open bus, ignored
    assert_eq!(0x1234, dev.read_address(0x60FF));
    assert_eq!(0, dev.read_address(0x6100));
}
