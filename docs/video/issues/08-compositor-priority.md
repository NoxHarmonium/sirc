# PPU-08: compositor - layer order, priority bit and layer disables

**Milestone:** B. Orthogonal layers and Mode 1
**Depends on:** PPU-05
**Labels:** ppu

## Goal

Implement the compositing rule once, for every mode: S1, B1, S2, B2, S3, B3, B4 front to back (B4 in Mode 1 only), the priority bit, and the disable bits.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 4: Compositing Rule, Tilemap Entries (`P`); Chapter 5: Priority and Compositing
- Chapter 3: Base Configuration Register (`B1D`-`B4D`, `S1D`-`S3D`)

## State on `main`

- `resolve_first_visible_pixel` checks background 3 first and background 1 last - the reverse of the intended order.
- **Bug:** it computes `palette_offsets[i] + pixel_values[i]` as unchecked `u8` arithmetic; a large palette offset can overflow and panic in a debug build. (`resolve_palette_addr` already wraps.)
- The priority bit and the layer disable bits are stored and ignored in Mode 1. There are no sprite pixels to composite.

## Tasks

- [ ] Write one compositor function taking the pixel of each background (palette address or transparent) and the sprite pixel and returning the colour address: the first opaque pixel in the order S1, B1, S2, B2, S3, B3, B4; palette entry 0 if none.
- [ ] Priority bit set: that tile's pixels are in front of everything, sprite layer 1 included. Zero keeps the layer's place.
- [ ] `B1D`-`B4D` make a background transparent (all modes; background 4 exists in Mode 1 only). `S1D`-`S3D` are used by the sprite engine (PPU-10).
- [ ] Wrap the palette add (`wrapping_add`).

## Acceptance criteria

- [ ] Test: all 2^3 combinations of transparent/opaque backgrounds, with and without a sprite pixel on each layer, give the order above.
- [ ] Test: priority bit puts a tile above S1 and nothing else changes.
- [ ] Test: disables.
- [ ] Test: no panic with `palette_offset` = 0xFF and value 15.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): `composite_palette_addr` and the Mode 1 call site; tests `sprite_layers_sit_between_the_backgrounds` and `the_priority_bit_puts_a_flat_tile_in_front_of_sprite_layer_1`.
