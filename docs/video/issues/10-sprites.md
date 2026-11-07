# PPU-10: sprite table, line engine and sprite layers

**Milestone:** C. Sprites
**Depends on:** PPU-01, PPU-03, PPU-08
**Labels:** ppu

## Goal

Implement sprites: the 128-entry table, the per-line scan, fetch and paint into a line buffer, and the three sprite layers.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 7 *Sprites*: Sprite Table Entry, Sprite Tile Data, Drawing a Line, How Many Sprites, Sprites in Modes 2 and 3
- Chapter 3: Tile Size Register (`SnS`), `STILES`; Base Configuration (`SnD`)

## State on `main`

- Nothing: the Mode 1 state machine has two reserved sprite slots marked `TODO`; no sprite table, no `STILES` use.

## Tasks

- [ ] Sprite entry: 2 words, `V H PPP LL XXXXXXXXX` / `YYYYYYYY TTTTTTTT` (flips, palette 0-7, layer 0-2 or 3 = hidden, signed 9-bit X, Y, tile).
- [ ] Per line: scan all 128 entries (internal RAM) keeping the first 32 that cross the line (visible layer, row `(line - Y) mod 256 < size`); fetch one row per kept sprite (a word per 4 pixels, skipping words fully off screen); paint into a 256-entry line buffer with the frontmost sprite pixel (lower layer, then lower index) winning.
- [ ] Sizes 8 or 16 per layer (`SnS`); tiles laid out like background tiles (16 by 16 = four 8 by 8 quadrants, 64 words); palette `0x80 + 16p + value`.
- [ ] Fetch budget: 148 reads per line (170 porch cycles minus 22 prefetch) - charge it, and drop sprites that do not fit.
- [ ] `S1D`-`S3D` skip a layer. Reset state hides every sprite.

## Acceptance criteria

- [ ] Test against an independent per-pixel reference: flips, clipping at both edges, Y wrap, 8 and 16 pixel sprites, layer priority.
- [ ] Test: 40 sprites on a line draw the first 32; the fetch cost is exactly words x sprites; 16-pixel sprites never exceed the budget at 32.
- [ ] Test: hidden/disabled sprites cost nothing.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): `sprite.rs` (`SpriteEntry`, `build_line`) and `sprite_tests.rs`.

## Notes

16 by 16 sprite tiles are stored as four 8 by 8 quadrants, the same as 16 by 16 background tiles (PPU-06), so one tile format serves both and the asset tool exports a single layout. A 16-pixel sprite row is two words from the left quadrant and two from the right.
