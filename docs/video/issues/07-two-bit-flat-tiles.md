# PPU-07: 2-bit orthogonal tiles (T2)

**Milestone:** B. Orthogonal layers and Mode 1
**Depends on:** PPU-05
**Labels:** ppu

## Goal

Support 2 bits per pixel orthogonal tiles per layer, selected by `T2` in the layer's palette configuration register.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 4: 2-Bit Orthogonal Tiles, 16 by 16 Orthogonal Tiles; Chapter 3: Palette Configuration Registers (`T2`)
- Chapter 8: Fetch Budget

## State on `main`

- `T2` does not exist yet (PPU-01). The tile data path is 4-bit only (`TileLine` has four nibbles).
- No palette size handling beyond `resolve_palette_addr`, which already supports 4-colour palettes (`SIZE` = 0).

## Tasks

- [ ] 2-bit 8 by 8 tile = 8 words, one per row, leftmost pixel in bits 15-14; 16 by 16 = 4 quadrants of 8 words (32 words).
- [ ] A word covers eight pixels: read it for every second group of four (and for the first group of a line when it starts on the second half).
- [ ] Value 0 is transparent. The palette rule is unchanged; leaving `SIZE` right is the program's job.

## Acceptance criteria

- [ ] Test: 2-bit layers against the reference for 8 by 8 and 16 by 16, with and without flips and scroll offsets (odd scrolls start on the second half of a word).
- [ ] Test: reads per line are about half of the 4-bit case.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): `flat_run_slot` handles 2-bit 8 by 8 tiles (including the second-half start); test `mode_3_matches_the_reference_in_every_tile_format_and_scroll`.
