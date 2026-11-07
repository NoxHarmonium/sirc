# PPU-13: isometric layer engine and Mode 2

**Milestone:** D. Isometric
**Depends on:** PPU-05, PPU-08, PPU-12
**Labels:** ppu

## Goal

Render two isometric backgrounds in Mode 2 with the specified fetch schedule, fine scrolling and layer disables.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 6: Fetch Schedule and Pipeline (Slots, Groups and Fine Scrolling, Per-Line Sequence), Layers Priority and Height, Restrictions
- Chapter 8: Fetch Budget; Chapter 3: Base Configuration (`MODE`)

## State on `main`

- Nothing: `MODE` is stored (reserved bits) and ignored.

## Tasks

- [ ] Mode select: sample `MODE` at the first cycle of each line; 0 = Mode 1, 1 = Mode 2, 2 = Mode 3, others reserved.
- [ ] Per group of four pixels each layer has four slots: entry A, word A, entry B, word B (B only when the group straddles two cells).
- [ ] Group g+2 is fetched while group g is shown; two groups are prefetched in the front porch, 32 master clocks before the first pixel; scroll registers are sampled once per line there.
- [ ] Fine scroll: window of two groups, offset `scroll_x & 3`.
- [ ] Background 1 in front of 2; `B1D`/`B2D`; palette rule per layer; tile numbers address 64-word tiles.
- [ ] The tilemap entry cache is not part of the device - read the entry afresh for every group (rationale in `DESIGN-NOTES.md`).

## Acceptance criteria

- [ ] Test: random content x six scroll cases (including odd and large scrolls) against the PPU-12 reference, pixel for pixel.
- [ ] Test: slot counts (average 4.75 of 8 over all scroll positions; worst case 8).
- [ ] Test: Mode 2 draws nothing and fetches nothing in lines >= 224.
- [ ] Snapshot test of a baked demo scene.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): `step_isometric`, `iso_run_slot`, `iso_mux_group`; tests `pipelined_renderer_matches_reference_for_random_content_and_scrolls`, `demo_scene_baked_from_screen_space_art_renders_exactly`.
