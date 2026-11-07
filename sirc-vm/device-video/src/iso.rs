//! Mode 2 (isometric) pure model.
//!
//! Everything in here is plain integer maths with no reference to the PPU clock, so the same
//! functions can describe the hardware, drive the tests and be reused by an asset tool.
//!
//! # The idea
//!
//! A Mode 2 background is an orthogonal map of square 16x16 texel tiles that is displayed as a 2:1
//! isometric (dimetric) ground plane. Each 16x16 tile appears as a 32x16 pixel diamond. The
//! mapping from a plane pixel `(x, y)` (screen position plus scroll) to a map texel `(u, v)` is
//!
//! ```text
//! u = y + (x >> 1)
//! v = y - ((x + 1) >> 1)
//! ```
//!
//! which needs two adders and no multiplier. It is a bijection between pixels and texels (every
//! texel is shown exactly once), so an artist's pixel-art diamond can be unwrapped into a square
//! tile and shown again with no loss. `u - v == x` always holds, which is what makes the tile
//! format below work.
//!
//! # The staircase tile format
//!
//! Four neighbouring pixels on one scanline (starting on a multiple of four in `x`) sample a
//! diagonal staircase of four texels: `(a,b) (a,b-1) (a+1,b-1) (a+1,b-2)`. A tile is stored as 64
//! words with each word holding one such staircase, nibble 0 (the high nibble) being the leftmost
//! pixel. A tile is treated as if it wrapped around, so every one of the 64 staircases exists
//! exactly once per tile. A staircase that crosses into the next tile is assembled from the
//! same-numbered word of both tiles, taking each pixel's nibble from the tile that pixel falls in.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

use crate::types::TilemapEntry;
use std::collections::HashMap;

/// Texels along one side of a tile (the only tile size Mode 2 supports so far).
pub const TILE_TEXELS: i32 = 16;
/// Words in one swizzled tile: 16 x 16 texels x 4 bits / 16 bits.
pub const TILE_WORDS: u16 = 64;
/// Tilemap width and height in cells (a single 32 x 32 tilemap; double tilemaps are not supported).
pub const MAP_CELLS: i32 = 32;

/// A cell of the tilemap, after wrapping.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    pub cu: u16,
    pub cv: u16,
}

impl Cell {
    /// Offset of this cell's entry from the tilemap base address (row-major, `cv` is the row).
    #[must_use]
    pub const fn tilemap_offset(self) -> u16 {
        self.cv * MAP_CELLS as u16 + self.cu
    }
}

/// Map texel shown at plane pixel `(x, y)`. Coordinates may be negative; callers wrap with
/// [`cell_of`] and `& 15`.
#[must_use]
pub const fn texel_at(x: i32, y: i32) -> (i32, i32) {
    (y + (x >> 1), y - ((x + 1) >> 1))
}

/// Tilemap cell containing the map texel `(u, v)`, wrapped to the 32 x 32 map.
#[must_use]
pub fn cell_of(u: i32, v: i32) -> Cell {
    Cell {
        cu: (u >> 4).rem_euclid(MAP_CELLS) as u16,
        cv: (v >> 4).rem_euclid(MAP_CELLS) as u16,
    }
}

/// Word index (0..64) and nibble (0..4, 0 = high nibble) of in-tile texel `(tu, tv)`.
#[must_use]
pub fn locate(tu: i32, tv: i32) -> (u16, u8) {
    let i = (tu - tv) & 3;
    // Anchor of the staircase (the texel shown by its first pixel), wrapped within the tile.
    let a = tu - (i >> 1);
    let b = tv + [0, 1, 1, 2][i as usize];
    let diagonal = ((a - b) & 15) >> 2;
    ((diagonal * 16 + (a & 15)) as u16, i as u8)
}

/// Pack a tile (`art[v][u]`, values 0..16) into the staircase format.
#[must_use]
pub fn swizzle_tile(art: &[[u8; 16]; 16]) -> [u16; TILE_WORDS as usize] {
    let mut words = [0u16; TILE_WORDS as usize];
    for (tv, row) in art.iter().enumerate() {
        for (tu, value) in row.iter().enumerate() {
            let (word, nibble) = locate(tu as i32, tv as i32);
            words[word as usize] |= u16::from(value & 0xF) << (12 - 4 * u32::from(nibble));
        }
    }
    words
}

/// Read one texel back out of a swizzled tile.
#[must_use]
pub fn texel_from_words(words: &[u16], tu: i32, tv: i32) -> u8 {
    let (word, nibble) = locate(tu, tv);
    ((words[word as usize] >> (12 - 4 * u32::from(nibble))) & 0xF) as u8
}

/// Undo [`swizzle_tile`].
#[must_use]
pub fn unswizzle_tile(words: &[u16; TILE_WORDS as usize]) -> [[u8; 16]; 16] {
    let mut art = [[0u8; 16]; 16];
    for (tv, row) in art.iter_mut().enumerate() {
        for (tu, value) in row.iter_mut().enumerate() {
            *value = texel_from_words(words, tu as i32, tv as i32);
        }
    }
    art
}

/// What the PPU has to fetch for one group of four pixels of one layer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GroupPlan {
    /// The distinct cells the four pixels fall in (only `cell_count` are valid).
    pub cells: [Cell; 2],
    pub cell_count: u8,
    /// For each pixel, which of `cells` it belongs to.
    pub pixel_cell: [u8; 4],
    /// Word offset within each cell's tile (the same for both cells; see the module docs).
    pub word_in_tile: u16,
}

/// Plan the fetch for the four pixels starting at plane position `(x0, y)`; `x0` must be a
/// multiple of four. A group never touches more than two cells (checked exhaustively in tests,
/// and enforced here).
#[must_use]
pub fn plan_group(x0: i32, y: i32) -> GroupPlan {
    debug_assert_eq!(x0 & 3, 0, "groups are aligned to four plane pixels");
    let mut plan = GroupPlan::default();
    for k in 0..4 {
        let (u, v) = texel_at(x0 + k, y);
        let cell = cell_of(u, v);
        let known = plan.cells[..plan.cell_count as usize]
            .iter()
            .position(|c| *c == cell);
        let index = known.unwrap_or_else(|| {
            assert!(plan.cell_count < 2, "a group touched three cells");
            plan.cells[plan.cell_count as usize] = cell;
            plan.cell_count += 1;
            plan.cell_count as usize - 1
        });
        plan.pixel_cell[k as usize] = index as u8;
    }
    let (u0, _) = texel_at(x0, y);
    plan.word_in_tile = ((((x0 >> 2) & 3) * 16) + (u0 & 15)) as u16;
    plan
}

/// Word address of a tile's first word, given the tile data base and the tilemap entry.
#[must_use]
pub fn tile_address(tile_base: u16, entry: TilemapEntry) -> u16 {
    tile_base.wrapping_add(entry.tile_index().wrapping_mul(TILE_WORDS))
}

/// Straightforward (unpipelined) lookup of the layer's palette-relative pixel value (0 =
/// transparent) at plane pixel `(x, y)`. This is the specification the pipelined fetch is
/// tested against.
#[must_use]
pub fn reference_sample(
    read: &dyn Fn(u16) -> u16,
    tilemap_base: u16,
    tile_base: u16,
    x: i32,
    y: i32,
) -> (u8, TilemapEntry) {
    let (u, v) = texel_at(x, y);
    let entry: TilemapEntry =
        read(tilemap_base.wrapping_add(cell_of(u, v).tilemap_offset())).into();
    let (word, nibble) = locate(u & 15, v & 15);
    let data = read(tile_address(tile_base, entry).wrapping_add(word));
    (((data >> (12 - 4 * u32::from(nibble))) & 0xF) as u8, entry)
}

/// The result of [`bake_layer`]: ready to copy into VRAM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BakedLayer {
    /// 32 x 32 tilemap entries (tile index only; palette select 0). Tile 0 is always empty.
    pub tilemap: Vec<u16>,
    /// Swizzled tiles; `tiles[0]` is the all-transparent tile.
    pub tiles: Vec<[u16; TILE_WORDS as usize]>,
}

/// Slice a screen-space image into a tilemap and a set of unique tiles.
///
/// The layer will be scrolled by `(scroll_x, scroll_y)`. This is the algorithm an asset tool
/// would use: `art(x, y)` is the image an artist drew, in screen pixels, and the result renders
/// back to exactly that image. Texels the image never reaches stay transparent.
pub fn bake_layer(
    width: i32,
    height: i32,
    scroll_x: i32,
    scroll_y: i32,
    art: &dyn Fn(i32, i32) -> u8,
) -> BakedLayer {
    let mut textures: HashMap<Cell, [[u8; 16]; 16]> = HashMap::new();
    for y in 0..height {
        for x in 0..width {
            let (u, v) = texel_at(x + scroll_x, y + scroll_y);
            textures.entry(cell_of(u, v)).or_insert([[0; 16]; 16])[(v & 15) as usize]
                [(u & 15) as usize] = art(x, y) & 0xF;
        }
    }
    let mut tiles = vec![[0u16; TILE_WORDS as usize]];
    let mut seen: HashMap<[u16; TILE_WORDS as usize], u16> = HashMap::new();
    seen.insert(tiles[0], 0);
    let mut tilemap = vec![0u16; (MAP_CELLS * MAP_CELLS) as usize];
    // Iterate in a fixed order so the output is deterministic.
    for cv in 0..MAP_CELLS as u16 {
        for cu in 0..MAP_CELLS as u16 {
            let cell = Cell { cu, cv };
            if let Some(texture) = textures.get(&cell) {
                let words = swizzle_tile(texture);
                let index = *seen.entry(words).or_insert_with(|| {
                    tiles.push(words);
                    (tiles.len() - 1) as u16
                });
                assert!(index < 1024, "tile index does not fit in 10 bits");
                tilemap[cell.tilemap_offset() as usize] = index;
            }
        }
    }
    BakedLayer { tilemap, tiles }
}

// ---------------------------------------------------------------------------------------------
// 2 bit tiles
// ---------------------------------------------------------------------------------------------
//
// A layer can use 2 bit tiles to halve the video memory its tiles take. Groups are then eight
// pixels, aligned to the plane, and one 16 bit word holds the eight 2 bit pixels of a staircase
// of eight texels. A tile is 32 words (2 groups x 16 anchors), and the 32 staircases of a tile
// cover its 256 texels exactly once, the same way the 4 bit format does.

/// Words in one 2 bit tile.
pub const TILE_WORDS_2BIT: u16 = 32;

/// What the PPU has to fetch for one group of eight pixels of a 2 bit layer.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GroupPlan8 {
    pub cells: [Cell; 2],
    pub cell_count: u8,
    pub pixel_cell: [u8; 8],
    pub word_in_tile: u16,
}

/// Plan the fetch for the eight pixels starting at plane position `(x0, y)`; `x0` must be a
/// multiple of eight. A group never touches more than two cells.
#[must_use]
pub fn plan_group8(x0: i32, y: i32) -> GroupPlan8 {
    debug_assert_eq!(x0 & 7, 0, "2 bit groups are aligned to eight plane pixels");
    let mut plan = GroupPlan8::default();
    for k in 0..8 {
        let (u, v) = texel_at(x0 + k, y);
        let cell = cell_of(u, v);
        let known = plan.cells[..plan.cell_count as usize]
            .iter()
            .position(|c| *c == cell);
        let index = known.unwrap_or_else(|| {
            assert!(plan.cell_count < 2, "a group touched three cells");
            plan.cells[plan.cell_count as usize] = cell;
            plan.cell_count += 1;
            plan.cell_count as usize - 1
        });
        plan.pixel_cell[k as usize] = index as u8;
    }
    let (u0, _) = texel_at(x0, y);
    plan.word_in_tile = ((((x0 >> 3) & 1) * 16) + (u0 & 15)) as u16;
    plan
}

/// Word index (0..32) and pixel (0..8, 0 = bits 15-14) of in-tile texel `(tu, tv)` in a 2 bit tile.
#[must_use]
pub fn locate_2bit(tu: i32, tv: i32) -> (u16, u8) {
    // The pixel's plane column is congruent to tu - tv modulo 16; the group parity and the pixel's
    // place in its group follow from that, and the group's anchor from the texel it shows
    let column = (tu - tv) & 15;
    let k = column & 7;
    let anchor = tu - (k >> 1);
    ((((column >> 3) * 16) + (anchor & 15)) as u16, k as u8)
}

/// Pack a tile (`art[v][u]`, values 0..4) into the 2 bit staircase format.
#[must_use]
pub fn swizzle_tile_2bit(art: &[[u8; 16]; 16]) -> [u16; TILE_WORDS_2BIT as usize] {
    let mut words = [0u16; TILE_WORDS_2BIT as usize];
    for (tv, row) in art.iter().enumerate() {
        for (tu, value) in row.iter().enumerate() {
            let (word, pixel) = locate_2bit(tu as i32, tv as i32);
            words[word as usize] |= u16::from(value & 3) << (14 - 2 * u32::from(pixel));
        }
    }
    words
}

/// Word address of a 2 bit tile's first word.
#[must_use]
pub fn tile_address_2bit(tile_base: u16, entry: TilemapEntry) -> u16 {
    tile_base.wrapping_add(entry.tile_index().wrapping_mul(TILE_WORDS_2BIT))
}

/// [`reference_sample`] for a 2 bit layer.
#[must_use]
pub fn reference_sample_2bit(
    read: &dyn Fn(u16) -> u16,
    tilemap_base: u16,
    tile_base: u16,
    x: i32,
    y: i32,
) -> (u8, TilemapEntry) {
    let (u, v) = texel_at(x, y);
    let entry: TilemapEntry =
        read(tilemap_base.wrapping_add(cell_of(u, v).tilemap_offset())).into();
    let (word, pixel) = locate_2bit(u & 15, v & 15);
    let data = read(tile_address_2bit(tile_base, entry).wrapping_add(word));
    (((data >> (14 - 2 * u32::from(pixel))) & 3) as u8, entry)
}

/// The result of [`bake_layer_2bit`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BakedLayer2bit {
    pub tilemap: Vec<u16>,
    /// `tiles[0]` is the all-transparent tile.
    pub tiles: Vec<[u16; TILE_WORDS_2BIT as usize]>,
}

/// [`bake_layer`] for 2 bit tiles: `art` returns values 0..4.
pub fn bake_layer_2bit(
    width: i32,
    height: i32,
    scroll_x: i32,
    scroll_y: i32,
    art: &dyn Fn(i32, i32) -> u8,
) -> BakedLayer2bit {
    let mut textures: HashMap<Cell, [[u8; 16]; 16]> = HashMap::new();
    for y in 0..height {
        for x in 0..width {
            let (u, v) = texel_at(x + scroll_x, y + scroll_y);
            textures.entry(cell_of(u, v)).or_insert([[0; 16]; 16])[(v & 15) as usize]
                [(u & 15) as usize] = art(x, y) & 3;
        }
    }
    let mut tiles = vec![[0u16; TILE_WORDS_2BIT as usize]];
    let mut seen: HashMap<[u16; TILE_WORDS_2BIT as usize], u16> = HashMap::new();
    seen.insert(tiles[0], 0);
    let mut tilemap = vec![0u16; (MAP_CELLS * MAP_CELLS) as usize];
    for cv in 0..MAP_CELLS as u16 {
        for cu in 0..MAP_CELLS as u16 {
            let cell = Cell { cu, cv };
            if let Some(texture) = textures.get(&cell) {
                let words = swizzle_tile_2bit(texture);
                let index = *seen.entry(words).or_insert_with(|| {
                    tiles.push(words);
                    (tiles.len() - 1) as u16
                });
                assert!(index < 1024, "tile index does not fit in 10 bits");
                tilemap[cell.tilemap_offset() as usize] = index;
            }
        }
    }
    BakedLayer2bit { tilemap, tiles }
}

#[cfg(test)]
mod tests_two_bit {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn two_bit_staircase_is_a_perfect_packing() {
        let mut used = HashSet::new();
        for tv in 0..16 {
            for tu in 0..16 {
                let (word, pixel) = locate_2bit(tu, tv);
                assert!(word < 32 && pixel < 8);
                assert!(used.insert((word, pixel)), "({tu},{tv}) collides");
            }
        }
        assert_eq!(used.len(), 256);
    }

    #[test]
    fn two_bit_group_agrees_with_every_pixels_own_lookup() {
        let mut two_cell = 0;
        let mut total = 0;
        for y in -32..32 {
            for g in -32..32 {
                let x0 = g * 8;
                let plan = plan_group8(x0, y);
                total += 1;
                two_cell += i32::from(plan.cell_count == 2);
                for k in 0..8 {
                    let (u, v) = texel_at(x0 + k, y);
                    let (word, pixel) = locate_2bit(u & 15, v & 15);
                    assert_eq!(word, plan.word_in_tile, "x0={x0} y={y} k={k}");
                    assert_eq!(i32::from(pixel), k);
                }
            }
        }
        println!("2 bit groups with a second cell: {two_cell} of {total}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn pixel_to_texel_is_a_bijection() {
        // Over a large window every texel of a tile is shown exactly once.
        let mut seen: HashMap<(i32, i32), u32> = HashMap::new();
        for y in -64..192 {
            for x in -256..256 {
                let (u, v) = texel_at(x, y);
                if (16..32).contains(&u) && (48..64).contains(&v) {
                    *seen.entry((u, v)).or_default() += 1;
                }
            }
        }
        assert_eq!(seen.len(), 256);
        assert!(seen.values().all(|n| *n == 1));
    }

    #[test]
    fn x_equals_u_minus_v() {
        for y in -20..20 {
            for x in -20..20 {
                let (u, v) = texel_at(x, y);
                assert_eq!(u - v, x);
            }
        }
    }

    #[test]
    fn a_cell_covers_256_pixels_and_32x16() {
        let mut count = 0;
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (i32::MAX, i32::MIN, i32::MAX, i32::MIN);
        for y in -64..128 {
            for x in -128..128 {
                let (u, v) = texel_at(x, y);
                if cell_of(u, v) == (Cell { cu: 0, cv: 0 }) {
                    count += 1;
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                }
            }
        }
        // The window contains exactly one copy of cell (0, 0) (the map wraps every 512 texels).
        assert_eq!(count, 256);
        assert_eq!((max_x - min_x + 1, max_y - min_y + 1), (31, 16));
    }

    #[test]
    fn staircase_is_a_perfect_packing() {
        let mut used = HashSet::new();
        for tv in 0..16 {
            for tu in 0..16 {
                let (word, nibble) = locate(tu, tv);
                assert!(word < 64 && nibble < 4);
                assert!(used.insert((word, nibble)), "({tu},{tv}) collides");
            }
        }
        assert_eq!(used.len(), 256);
    }

    #[test]
    fn swizzle_round_trips() {
        let mut art = [[0u8; 16]; 16];
        for (v, row) in art.iter_mut().enumerate() {
            for (u, px) in row.iter_mut().enumerate() {
                *px = ((u * 7 + v * 3) % 16) as u8;
            }
        }
        assert_eq!(unswizzle_tile(&swizzle_tile(&art)), art);
    }

    #[test]
    fn group_never_touches_more_than_two_cells_and_shares_one_word_index() {
        let mut two_cell = 0u32;
        let mut total = 0u32;
        // The cell lattice repeats every 16 texels, so a few periods cover everything.
        for y in -64..64 {
            for g in -64..64 {
                let x0 = g * 4;
                let plan = plan_group(x0, y);
                total += 1;
                two_cell += u32::from(plan.cell_count == 2);
                // Every pixel's own texel lookup must agree with the group's word and nibble.
                for k in 0..4 {
                    let (u, v) = texel_at(x0 + k, y);
                    let (word, nibble) = locate(u & 15, v & 15);
                    assert_eq!(word, plan.word_in_tile, "x0={x0} y={y} k={k}");
                    assert_eq!(i32::from(nibble), k);
                }
            }
        }
        // Print the straddle rate for the manual (run with --nocapture).
        println!(
            "groups touching two cells: {two_cell}/{total} = {:.1}%",
            f64::from(two_cell) * 100.0 / f64::from(total)
        );
        assert!(two_cell > 0);
    }

    #[test]
    fn bake_then_sample_returns_the_original_image() {
        let art = |x: i32, y: i32| -> u8 { ((x * 3 + y * 5 + (x ^ y)) & 0xF) as u8 };
        for (sx, sy) in [(0, 0), (1, 0), (3, 7), (130, 9)] {
            let baked = bake_layer(256, 224, sx, sy, &art);
            let read = |addr: u16| -> u16 {
                // tilemap at 0x0000.., tiles at 0x1000..
                if addr < 0x1000 {
                    baked.tilemap[addr as usize]
                } else {
                    let t = (addr - 0x1000) / TILE_WORDS;
                    baked.tiles[t as usize][((addr - 0x1000) % TILE_WORDS) as usize]
                }
            };
            for y in 0..224 {
                for x in 0..256 {
                    let (value, _) = reference_sample(&read, 0x0000, 0x1000, x + sx, y + sy);
                    assert_eq!(value, art(x, y) & 0xF, "scroll=({sx},{sy}) x={x} y={y}");
                }
            }
        }
    }
}
