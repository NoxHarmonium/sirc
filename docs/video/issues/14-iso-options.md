# PPU-14: isometric options - 2-bit tiles, double tilemaps and shading

**Milestone:** D. Isometric
**Depends on:** PPU-13, PPU-07
**Labels:** ppu

## Goal

Add the three optional isometric features.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 6: Two-Bit Tiles, Shading, Tilemap, Scrolling and Wrapping; Chapter 4: Tilemap Sizes

## State on `main`

- None of them exists.

## Tasks

- [ ] **2-bit tiles** (`T2`): groups of eight pixels at even plane groups, 32-word tiles, one word per staircase of eight; odd groups reuse the second half (and the first group of a line may need the previous even group).
- [ ] **Tilemap sizes**: 32 or 64 cells per axis; entry address from the screen formula; wrap the cell at `W` and `H`.
- [ ] **Shading**: tilemap entry `V` darkens (halve each channel), `H` lightens (move halfway to 31), both = normal; applied after the palette lookup; does not accumulate with a sprite shadow beyond one level.

## Acceptance criteria

- [ ] Test: 2-bit and 4-bit layers in every combination against the reference, with odd scrolls.
- [ ] Test: 64-cell maps wrap at 1024 texels.
- [ ] Test: shading values for every channel value; shadow + dark tile = one level.
- [ ] Snapshot: the shading demo (fog of war).

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): `iso_run_slot_two_bit`, `shade_pixel`, `entry_shade`; tests `two_bit_layers_match_the_reference_in_every_combination`, `isometric_tiles_darken_and_lighten_with_the_flip_bits`. The reference does not implement 64-cell maps.
