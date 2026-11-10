# PPU-05: orthogonal layer engine and Mode 1 rewrite (scrolling, fix 8-pixel offset)

**Milestone:** B. Orthogonal layers and Mode 1
**Depends on:** PPU-02, PPU-03
**Labels:** ppu

## Goal

Replace Mode 1's monolithic fetch state machine with the group-based orthogonal layer engine of the manual, which adds scrolling and fixes the picture offset. 8 by 8 tiles in one 32 by 32 tilemap per layer; wider tiles, flips and 2-bit tiles follow in PPU-06/07.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 5 *Mode 1*: Backgrounds (Scrolling), Fetch Schedule
- Chapter 6b *Mode 3*: The Orthogonal Backgrounds, Scrolling (the same mechanism)
- Chapter 8: Fetch Budget, Pixel Pipeline; Chapter 3: Scroll Registers

## State on `main`

- `lib.rs` `poll` is a `RenderStateMachine` (`FetchTilemapEntry`, `FetchTile`, `FetchSpriteTilemapEntry`, `FetchSpriteTile`) with three tilemap entry reads then three tile reads then two sprite stubs per group (the manual's Mode 1 has a fourth background in the two spare slots) of four pixels. Entries are re-read for every group.
- **Bug (measured):** the first two groups of a line are flushed to blank before anything has been fetched, so the left 8 pixels of every line are blank, and the tilemap is drawn 8 pixels to the right of its position (the last tile column is never shown). A solid tilemap shows `........################` along the first 24 pixels (`.` blank, `#` drawn).
- Scroll registers are stored and ignored; there is no front-porch prefetch.
- `test_video_line_timing` asserts the exact per-clock state sequence, and the `last_frame` snapshot encodes the offset picture. Both will change.

## Tasks

- [ ] Implement the orthogonal engine: per layer, per group of four pixels, a tilemap entry read then a tile word read, for the plane group `(scroll_x >> 2) + group` (group 0 starts at plane column `4 * (scroll_x >> 2)`); fine scroll via a two-group window and offset `scroll_x & 3`; 32 by 32 wrap (256 by 256 pixels).
- [ ] Sample the scroll registers once per line before the first visible pixel; prefetch two groups in the front porch.
- [ ] Use the slot assignment of Table *Mode 1 Fetch Slots*: four backgrounds, slots 0-3 entries, 4-7 words, nothing reserved for sprites. Add background 4 as an ordinary orthogonal layer (it is drawn in Mode 1 only).
- [ ] Keep the pipeline structure (fetch group g+2 while showing group g) so the same engine serves Mode 3.
- [ ] Regenerate the snapshot (the picture moves 8 pixels left) and replace the per-clock state test with a slot-level test of the new schedule.

## Acceptance criteria

- [ ] Test: a solid tilemap fills pixels 0-255 of every visible line.
- [ ] Test: scroll X/Y of 0, 1, 3, 4, 7, 8, 255, 256, 1023 match a straightforward reference sampler pixel for pixel on random content.
- [ ] Test: reads per group match the slot table; no reads in lines >= 224.
- [ ] New `last_frame` snapshot reviewed by eye; clippy clean.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): `flat_run_slot`, the window/staging code (`iso_shift_window`, `IsoLayerState`) and `flat_reference_sample` in `iso_tests.rs` implement and check the same mechanism for Mode 3's orthogonal layers.

## Notes

Look at `iso_shift_window`/`iso_mux_group` on the branch for the window idea, but write the engine yourself; the scheduling picture is Figure *Who Uses Each Fetch Slot*.
