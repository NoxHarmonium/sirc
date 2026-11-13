# PPU-16: Mode 3 - two orthogonal backgrounds over one isometric background

**Milestone:** D. Isometric
**Depends on:** PPU-05, PPU-07, PPU-13
**Labels:** ppu

## Goal

Add Mode 3: backgrounds 1 and 2 are orthogonal, background 3 is isometric, with fixed slot ownership and no queue.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 6b *Mode 3* (all)
- Chapter 8: Fetch Budget (Figure *Who Uses Each Fetch Slot*)

## State on `main`

- Nothing.

## Tasks

- [ ] `MODE` = 2: slots 0-1 orthogonal entries (B1, B2), 2-3 orthogonal words (B1, B2), 4-7 isometric B3 (entry A, entry B, word A, word B).
- [ ] Orthogonal layers scroll independently; priority bit puts a tile in front of sprite layer 1; `T2` per layer.
- [ ] Isometric B3 uses the B3 registers (`B3MAP`, `B3TILES`, `B3SCRX/Y`, `B3PAL`); shading applies to it.
- [ ] Order S1, B1, S2, B2, S3, B3.

## Acceptance criteria

- [ ] Test: four tile-format combinations x five scroll sets against the reference; disable bits; sprite layering; priority bit.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): Mode 3 branches in `step_isometric`, `iso_layer_registers`, `flat_run_slot`; tests in `iso_tests.rs` (`mode_3_*`) and `sprite_tests.rs`.
