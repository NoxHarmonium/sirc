# PPU-12: isometric geometry and staircase tile format (pure model)

**Milestone:** D. Isometric
**Depends on:** none
**Labels:** ppu

## Goal

Write the pure model of Mode 2's sampling and tile format, with exhaustive tests, before any rendering. It is plain integer maths and is also what the asset tool needs.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 6 *Mode 2*: Geometry, The Staircase Tile Format (Locating a Texel, Why It Works), Tilemap Sizes (Chapter 4)

## State on `main`

- Nothing on `main`.

## Tasks

- [ ] `texel_at(x, y) = (y + (x >> 1), y - ((x + 1) >> 1))` and the cell/texel split (wrap at the map size).
- [ ] Staircase tile format: 16 by 16 texels in 64 words; `locate(tu, tv)` gives the word and nibble; groups of four pixels aligned to plane x multiple of 4 touch at most two cells and use the same word index in both.
- [ ] `swizzle`/`unswizzle`; a group plan (cells touched, which pixel is in which cell, word index).
- [ ] Sampling is a bijection pixel <-> texel; `u - v = x`.
- [ ] Tilemap sizes 32 and 64 cells per axis.

## Acceptance criteria

- [ ] Tests (exhaustive over a period): bijection, `u - v == x`, a cell covers 256 pixels in a 31 x 16 footprint, the staircase is a perfect packing, swizzle round trip, groups touch <= 2 cells with 18.75% straddling.
- [ ] A reference sampler `sample(x, y)` reading through a closure, used as the oracle by PPU-13.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): `iso.rs` (all of it) and its unit tests.
