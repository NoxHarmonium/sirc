//! Prototype sprite engine shared by Mode 1 and Mode 2.
//!
//! Sprites are drawn with a line buffer. Before each visible line the engine scans the sprite
//! table (internal RAM, so it costs no video memory bandwidth), picks the sprites that cross the
//! line, fetches one row of tile data for each from video memory during the horizontal blanking
//! interval, and paints them into a 256 entry line buffer. The renderer then merges that buffer
//! with the backgrounds. This costs the same in both modes because it happens in the porches,
//! where neither mode fetches background data.
//!
//! Sprite table entry (two 16 bit words, proposal):
//!
//! ```text
//! word 0: V H PPP LL XXXXXXXXX   V,H flips; P palette 0-7; L layer 0-2 (3 = hidden); X signed 9 bit
//! word 1: YYYYYYYY TTTTTTTT      Y top line (wraps at 256); T tile number
//! ```

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

/// Number of entries in the sprite table.
pub const SPRITE_COUNT: usize = 128;
/// Sprite table base, as an offset in the device segment (256 words, two per entry).
pub const SPRITE_RAM_BASE: u32 = 0x7000;
/// Words in the sprite table.
pub const SPRITE_RAM_WORDS: usize = SPRITE_COUNT * 2;
/// Pixels in the line buffer.
pub const LINE_WIDTH: usize = 256;
/// `layer` value that hides a sprite.
pub const HIDDEN_LAYER: u8 = 3;
/// Palette entry of the first sprite palette; sprite palettes are 16 entries each, eight of them.
pub const SPRITE_PALETTE_BASE: u8 = 0x80;

/// The sprite colour that acts as a shadow: value 15 of sprite palette 7.
pub const SHADOW_PALETTE: u8 = 7;
/// See [`SHADOW_PALETTE`].
pub const SHADOW_VALUE: u8 = 15;

/// Sprites the engine will accept for one line (the capacity of its line registers).
pub const MAX_SPRITES_PER_LINE: usize = 32;
/// PPU cycles in the horizontal blanking interval (back porch of one line plus the front porch of
/// the next: 80 + 260 master clocks, two per cycle).
pub const PORCH_CYCLES: u32 = 170;
/// Cycles of the back porch that the background fetch needs.
///
/// 16 for the two isometric layers' prefetch of two groups, and 6 for the status layer's prefetch
/// of four groups (two tilemap entries and four tile words).
pub const BACKGROUND_PREFETCH_CYCLES: u32 = 22;
/// Video memory reads available to the sprite engine per line: one per PPU cycle.
pub const FETCH_BUDGET: u32 = PORCH_CYCLES - BACKGROUND_PREFETCH_CYCLES;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpriteEntry {
    /// Left edge, -256..255 (so a sprite can enter from the left).
    pub x: i16,
    pub y: u8,
    pub tile: u8,
    pub palette: u8,
    pub flip_h: bool,
    pub flip_v: bool,
    pub layer: u8,
}

impl SpriteEntry {
    #[must_use]
    pub fn from_words(word0: u16, word1: u16) -> Self {
        // Sign extend the 9 bit X
        let x = (((word0 & 0x1FF) << 7) as i16) >> 7;
        Self {
            x,
            y: (word1 >> 8) as u8,
            tile: (word1 & 0xFF) as u8,
            palette: ((word0 >> 11) & 7) as u8,
            flip_h: word0 & 0x4000 != 0,
            flip_v: word0 & 0x8000 != 0,
            layer: ((word0 >> 9) & 3) as u8,
        }
    }

    #[must_use]
    pub fn to_words(self) -> (u16, u16) {
        let word0 = (u16::from(self.flip_v) << 15)
            | (u16::from(self.flip_h) << 14)
            | (u16::from(self.palette & 7) << 11)
            | (u16::from(self.layer & 3) << 9)
            | (self.x as u16 & 0x1FF);
        (word0, (u16::from(self.y) << 8) | u16::from(self.tile))
    }
}

/// What the engine needs to know about the device's registers.
#[derive(Clone, Copy, Debug)]
pub struct SpriteConfig {
    /// Sprite size (8 or 16) of layers S1 to S3.
    pub sizes: [usize; 3],
    pub disabled: [bool; 3],
    /// Address of the sprite tile data.
    pub tile_base: u16,
    /// Value 15 of sprite palette 7 is a shadow (otherwise an ordinary colour)
    pub shadows: bool,
    pub max_per_line: usize,
    pub fetch_budget: u32,
}

/// One sprite pixel in the line buffer. `layer` says which sprite layer it came from, which the
/// compositor needs to place it among the backgrounds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpritePixel {
    pub opaque: bool,
    /// A shadow pixel: it draws nothing but darkens what is behind its layer.
    pub shadow: bool,
    pub layer: u8,
    pub palette_addr: u8,
    /// Priority key (layer then table index), used while painting.
    key: u16,
}

#[derive(Clone, Debug)]
pub struct SpriteLine {
    pub pixels: [SpritePixel; LINE_WIDTH],
}

impl Default for SpriteLine {
    fn default() -> Self {
        Self {
            pixels: [SpritePixel::default(); LINE_WIDTH],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpriteLineStats {
    /// Sprites that cross the line (visible layers only).
    pub in_range: usize,
    /// Sprites actually drawn.
    pub drawn: usize,
    /// Dropped because the line registers were full.
    pub dropped_limit: usize,
    /// Dropped because the fetch budget ran out.
    pub dropped_time: usize,
    /// Video memory reads used.
    pub fetch_cycles: u32,
}

impl SpriteLineStats {
    /// The status flags a program would want: range over (too many sprites) and time over.
    #[must_use]
    pub fn overflowed(&self) -> bool {
        self.dropped_limit > 0 || self.dropped_time > 0
    }
}

/// Words of tile data in one row of a sprite.
const fn words_per_row(size: usize) -> usize {
    size / 4
}

/// Pixel columns of a sprite that are on screen for word `w`, given its left edge.
fn word_on_screen(x: i32, size: usize, flip_h: bool, w: usize) -> bool {
    let first = if flip_h { size - 4 - w * 4 } else { w * 4 };
    let left = x + first as i32;
    left + 4 > 0 && left < LINE_WIDTH as i32
}

/// The video memory words a sprite needs to draw `line`, or `None` if it does not cross it.
/// Words that are entirely off screen are not read.
#[must_use]
pub fn sprite_row_words(
    entry: &SpriteEntry,
    size: usize,
    line: u16,
    tile_base: u16,
) -> Option<Vec<(usize, u16)>> {
    let row = (i32::from(line) - i32::from(entry.y)).rem_euclid(256) as usize;
    if row >= size {
        return None;
    }
    let row = if entry.flip_v { size - 1 - row } else { row };
    let words_per_tile = size * size / 4;
    // A 16 by 16 tile is four 8 by 8 quadrants (top left, top right, bottom left, bottom right) of 16 words
    let address = |w: usize| -> u16 {
        let quadrant = if size == 16 { 2 * (row / 8) + w / 2 } else { 0 };
        tile_base.wrapping_add(
            (usize::from(entry.tile) * words_per_tile + 16 * quadrant + 2 * (row % 8) + w % 2)
                as u16,
        )
    };
    Some(
        (0..words_per_row(size))
            .filter(|&w| word_on_screen(i32::from(entry.x), size, entry.flip_h, w))
            .map(|w| (w, address(w)))
            .collect(),
    )
}

/// Build the sprite line buffer for `line`. `read` reads a word of video memory.
pub fn build_line(
    table: &[SpriteEntry],
    config: &SpriteConfig,
    line: u16,
    read: impl Fn(u16) -> u16,
    out: &mut SpriteLine,
) -> SpriteLineStats {
    *out = SpriteLine::default();
    let mut stats = SpriteLineStats::default();

    // 1. Scan: the first `max_per_line` sprites (in table order) that cross the line
    let mut selected = Vec::with_capacity(config.max_per_line);
    for (index, entry) in table.iter().enumerate() {
        let layer = usize::from(entry.layer);
        if layer >= 3 || config.disabled[layer] {
            continue;
        }
        let row = (i32::from(line) - i32::from(entry.y)).rem_euclid(256) as usize;
        if row >= config.sizes[layer] {
            continue;
        }
        stats.in_range += 1;
        if selected.len() < config.max_per_line {
            selected.push((index, *entry));
        } else {
            stats.dropped_limit += 1;
        }
    }

    // 2. Fetch and paint, one video memory read per cycle, stopping when the budget runs out
    let mut out_of_time = false;
    for (index, entry) in selected {
        if out_of_time {
            stats.dropped_time += 1;
            continue;
        }
        let size = config.sizes[usize::from(entry.layer)];
        let words = sprite_row_words(&entry, size, line, config.tile_base)
            .expect("selected sprites cross the line");
        let cost = words.len() as u32;
        if stats.fetch_cycles + cost > config.fetch_budget {
            out_of_time = true;
            stats.dropped_time += 1;
            continue;
        }
        stats.fetch_cycles += cost;
        stats.drawn += 1;
        let key = (u16::from(entry.layer) << 8) | index as u16;
        for (w, address) in words {
            let data = read(address);
            for nibble in 0..4 {
                let value = ((data >> (12 - 4 * nibble)) & 0xF) as u8;
                // Column of this pixel within the sprite as it appears on screen
                let drawn_column = if entry.flip_h {
                    size - 1 - (w * 4 + nibble)
                } else {
                    w * 4 + nibble
                };
                let sx = i32::from(entry.x) + drawn_column as i32;
                if value == 0 || !(0..LINE_WIDTH as i32).contains(&sx) {
                    continue;
                }
                let slot = &mut out.pixels[sx as usize];
                let is_shadow =
                    config.shadows && entry.palette == SHADOW_PALETTE && value == SHADOW_VALUE;
                // A shadow pixel never covers a drawn sprite pixel, and a drawn one always covers a shadow
                let takes_slot = if is_shadow {
                    !slot.opaque && (!slot.shadow || key < slot.key)
                } else {
                    slot.shadow || !slot.opaque || key < slot.key
                };
                if takes_slot {
                    *slot = SpritePixel {
                        opaque: !is_shadow,
                        shadow: is_shadow,
                        layer: entry.layer,
                        palette_addr: SPRITE_PALETTE_BASE
                            .wrapping_add(entry.palette << 4)
                            .wrapping_add(value),
                        key,
                    };
                }
            }
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(size: usize) -> SpriteConfig {
        SpriteConfig {
            sizes: [size; 3],
            disabled: [false; 3],
            tile_base: 0x8000,
            shadows: true,
            max_per_line: MAX_SPRITES_PER_LINE,
            fetch_budget: FETCH_BUDGET,
        }
    }

    #[test]
    fn entry_words_round_trip() {
        for x in [-256, -1, 0, 1, 255] {
            for (flip_h, flip_v) in [(false, false), (true, false), (false, true), (true, true)] {
                let entry = SpriteEntry {
                    x,
                    y: 0xA5,
                    tile: 0x5A,
                    palette: 5,
                    flip_h,
                    flip_v,
                    layer: 2,
                };
                let (w0, w1) = entry.to_words();
                assert_eq!(entry, SpriteEntry::from_words(w0, w1));
            }
        }
        assert_eq!(HIDDEN_LAYER, SpriteEntry::from_words(0xFFFF, 0xFFFF).layer);
    }

    /// A tile of 16 distinguishable words: word n of the tile holds nibbles 4n+1..4n+4 (mod 16)
    fn tile_memory(address: u16) -> u16 {
        let n = address - 0x8000;
        (0..4).fold(0, |acc, i| (acc << 4) | ((n * 4 + i) % 15 + 1))
    }

    #[test]
    fn eight_by_eight_sprite_is_placed_flipped_and_clipped() {
        let mut line = SpriteLine::default();
        let entry = |x, flip_h| SpriteEntry {
            x,
            y: 10,
            tile: 0,
            palette: 1,
            flip_h,
            flip_v: false,
            layer: 0,
        };
        // Line 12 is row 2 of the sprite: words 4 and 5 of tile 0
        let stats = build_line(&[entry(20, false)], &config(8), 12, tile_memory, &mut line);
        assert_eq!((1, 1, 2), (stats.in_range, stats.drawn, stats.fetch_cycles));
        for c in 0..8 {
            let want = if c >= 4 {
                ((5 * 4 + (c - 4)) % 15 + 1) as u8
            } else {
                ((4 * 4 + c) % 15 + 1) as u8
            };
            let px = line.pixels[20 + c];
            assert_eq!(
                SPRITE_PALETTE_BASE + 16 + want,
                px.palette_addr,
                "column {c}"
            );
        }
        assert!(!line.pixels[19].opaque && !line.pixels[28].opaque);

        // Flipped: the columns are reversed
        let mut flipped = SpriteLine::default();
        build_line(
            &[entry(20, true)],
            &config(8),
            12,
            tile_memory,
            &mut flipped,
        );
        for c in 0..8 {
            assert_eq!(
                line.pixels[20 + c].palette_addr,
                flipped.pixels[27 - c].palette_addr
            );
        }

        // Half off the left edge: x = -4 keeps columns 4..8 and only needs word 1 of the row
        let mut clipped = SpriteLine::default();
        let stats = build_line(
            &[entry(-4, false)],
            &config(8),
            12,
            tile_memory,
            &mut clipped,
        );
        assert_eq!(1, stats.fetch_cycles);
        for c in 4..8 {
            assert_eq!(
                line.pixels[20 + c].palette_addr,
                clipped.pixels[c - 4].palette_addr
            );
        }
        // Fully off screen: in range but costs nothing
        let stats = build_line(
            &[entry(-8, false)],
            &config(8),
            12,
            tile_memory,
            &mut clipped,
        );
        assert_eq!((1, 0), (stats.in_range, stats.fetch_cycles));
    }

    #[test]
    fn sprites_wrap_in_y_and_flip_vertically() {
        let entry = SpriteEntry {
            x: 0,
            y: 250,
            ..SpriteEntry::default()
        };
        let mut line = SpriteLine::default();
        // y = 250 with size 8 covers lines 250..=255 and 0..=1
        for (l, hit) in [
            (249, false),
            (250, true),
            (1, true),
            (2, false),
            (100, false),
        ] {
            let s = build_line(&[entry], &config(8), l, tile_memory, &mut line);
            assert_eq!(hit, s.in_range == 1, "line {l}");
        }
        let flipped = SpriteEntry {
            flip_v: true,
            ..entry
        };
        let a = sprite_row_words(&entry, 8, 250, 0x8000).unwrap();
        let b = sprite_row_words(&flipped, 8, 257 % 256, 0x8000).unwrap();
        // Row 0 unflipped is row 7 flipped... row 7 of the flipped sprite is line 257-ish; check
        // by address: flipped row r reads row 7-r
        assert_eq!(a[0].1, 0x8000);
        assert_eq!(b[0].1, 0x8000);
        let top_of_flipped = sprite_row_words(&flipped, 8, 250, 0x8000).unwrap();
        assert_eq!(top_of_flipped[0].1, 0x8000 + 2 * 7);
    }

    #[test]
    fn lower_layer_then_lower_index_wins() {
        let mk = |layer, tile| SpriteEntry {
            x: 0,
            y: 0,
            tile,
            palette: layer,
            flip_h: false,
            flip_v: false,
            layer,
        };
        let mut line = SpriteLine::default();
        // Three overlapping opaque sprites, listed back to front
        let read = |_| 0x1111;
        build_line(
            &[mk(2, 0), mk(1, 1), mk(0, 2)],
            &config(8),
            0,
            read,
            &mut line,
        );
        assert_eq!(0, line.pixels[0].layer);
        // Same layer: the lower index wins
        build_line(&[mk(1, 0), mk(1, 1)], &config(8), 0, read, &mut line);
        assert_eq!(1, line.pixels[0].layer);
    }

    #[test]
    fn the_shadow_colour_is_only_a_shadow_when_enabled() {
        let entry = SpriteEntry {
            palette: SHADOW_PALETTE,
            ..SpriteEntry::default()
        };
        let mut line = SpriteLine::default();
        build_line(&[entry], &config(8), 0, |_| 0xFFFF, &mut line);
        assert!(line.pixels[0].shadow && !line.pixels[0].opaque);
        let mut cfg = config(8);
        cfg.shadows = false;
        build_line(&[entry], &cfg, 0, |_| 0xFFFF, &mut line);
        assert!(!line.pixels[0].shadow && line.pixels[0].opaque);
        assert_eq!(0xFF, line.pixels[0].palette_addr);
    }

    #[test]
    fn disabled_and_hidden_sprites_cost_nothing() {
        let table = [
            SpriteEntry {
                layer: 3,
                ..SpriteEntry::default()
            },
            SpriteEntry {
                layer: 1,
                ..SpriteEntry::default()
            },
        ];
        let mut cfg = config(8);
        cfg.disabled[1] = true;
        let mut line = SpriteLine::default();
        let stats = build_line(&table, &cfg, 0, |_| 0x1111, &mut line);
        assert_eq!(SpriteLineStats::default(), stats);
    }

    /// How many sprites the line registers and the fetch budget allow, per size
    #[test]
    fn capacity_per_line() {
        let table_of = |n: usize, y: u8| -> Vec<SpriteEntry> {
            (0..n)
                .map(|i| SpriteEntry {
                    x: (i as i16 * 3) % 200,
                    y,
                    tile: i as u8,
                    ..SpriteEntry::default()
                })
                .collect()
        };
        println!("fetch budget {FETCH_BUDGET} cycles per line");
        for size in [8_usize, 16] {
            // With the register limit lifted, how many fit in the time budget?
            let mut unlimited = config(size);
            unlimited.max_per_line = SPRITE_COUNT;
            let mut line = SpriteLine::default();
            let stats = build_line(
                &table_of(SPRITE_COUNT, 5),
                &unlimited,
                5,
                |_| 0x1111,
                &mut line,
            );
            println!(
                "{size}x{size}: {} sprites fit in the time budget ({} cycles), expected {}",
                stats.drawn,
                stats.fetch_cycles,
                FETCH_BUDGET as usize / (size / 4)
            );
            assert_eq!(FETCH_BUDGET as usize / (size / 4), stats.drawn);

            // With the register limit, 32 sprites always fit
            let stats = build_line(
                &table_of(SPRITE_COUNT, 5),
                &config(size),
                5,
                |_| 0x1111,
                &mut line,
            );
            assert_eq!(MAX_SPRITES_PER_LINE, stats.drawn);
            assert_eq!(SPRITE_COUNT - MAX_SPRITES_PER_LINE, stats.dropped_limit);
            assert_eq!(0, stats.dropped_time);
        }
    }
}
