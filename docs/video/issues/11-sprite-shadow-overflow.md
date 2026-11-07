# PPU-11: sprite shadow colour and overflow flag

**Milestone:** C. Sprites
**Depends on:** PPU-10
**Labels:** ppu

## Goal

Add the two sprite extras: the shadow colour (`SHD`) and the sprite overflow status flag.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 7: Shadow Pixels; Chapter 3: Base Configuration (`SHD`), Status Register (`SOV`)
- Chapter 7: How Many Sprites

## State on `main`

- Neither exists.

## Tasks

- [ ] When `CONFIG.SHD` = 1, a sprite pixel of value 15 in sprite palette 7 draws nothing and darkens (halves each channel of) whatever is behind its layer. With `SHD` = 0 it is an ordinary colour (palette entry `0xFF`).
- [ ] A shadow pixel never covers a drawn sprite pixel; a drawn pixel always replaces a shadow pixel.
- [ ] `STATUS.SOV`: set when a line has more than 32 sprites; cleared at the start of line 0; so it is readable in vertical blanking.

## Acceptance criteria

- [ ] Test: shadow darkens the right layers (S2 shadow darkens B2 but not B1; S3 shadow behind B2 changes nothing).
- [ ] Test: `SHD` = 0 draws palette entry `0xFF`.
- [ ] Test: `SOV` is not set at 32 sprites, is set at 33, stays set to the end of the frame and clears at the next line 0.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): `sprite.rs` (`SHD` handling in `build_line`), `composite_palette_addr`, `shade_pixel`; tests `shadow_sprite_pixels_darken_what_is_behind_their_layer`, `the_sprite_overflow_flag_is_sticky_until_the_next_frame`.
