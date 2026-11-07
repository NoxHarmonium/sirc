# PPU-09: output stage - graphics disable and brightness

**Milestone:** B. Orthogonal layers and Mode 1
**Depends on:** PPU-08
**Labels:** ppu

## Goal

Apply the final stage to every pixel: graphics disable and brightness.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 3: Base Configuration Register (`GD`, `BRT`); Chapter 8: Pixel Pipeline

## State on `main`

- `GD` and `BRT` are stored and ignored. Colours go from the palette straight to the 8-bit-per-channel host buffer by shifting left 3.

## Tasks

- [ ] `GD` = 1: black output (see PPU-03 for the memory side).
- [ ] Brightness: each 5-bit channel `c` becomes `c x (BRT + 1) / 16` (integer, truncating); `BRT` = 15 is unchanged. Apply after shading (PPU-14) and sprite shadows (PPU-11).
- [ ] Decide where the final rounding to 8 bits happens (host-side only).

## Acceptance criteria

- [ ] Test: `BRT` 15 is the identity; `BRT` 0 gives at most 1/16 of each channel; a table test for every channel value x `BRT`.
- [ ] Test: `GD` = 1 gives all-black frames.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): Not implemented on the branch.

## Notes

A multiplier is 5 x 4 bits per channel in hardware; shifts and adds are enough if you want to avoid one.
