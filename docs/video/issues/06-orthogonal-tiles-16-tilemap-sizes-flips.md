# PPU-06: orthogonal tiles - 16 by 16 tiles, double tilemaps and flips

**Milestone:** B. Orthogonal layers and Mode 1
**Depends on:** PPU-05
**Labels:** ppu

## Goal

Support the remaining orthogonal tile features: 16 by 16 tiles, 64 by 32 / 32 by 64 / 64 by 64 tilemaps, and flips.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 4 *Data Formats*: 16 by 16 Orthogonal Tiles, Flipping Orthogonal Tiles, Tilemap Sizes, Tilemap Entries
- Chapter 3: Tile Size Register (`BnS`, `BnX`, `BnY`)

## State on `main`

- `TileSizeRegister` stores `BnS`/`BnX`/`BnY` and nothing reads them; the renderer is fixed at 8 by 8 tiles, one 32 by 32 tilemap.
- Tilemap entry flip bits and the priority bit are parsed (`TilemapEntry`) and ignored.

## Tasks

- [ ] Per-layer tile size: 16 by 16 tile = four 8 by 8 quadrants (TL, TR, BL, BR), 64 words; word address `base + 64t + 16q + 2(row mod 8) + half`.
- [ ] Per-layer tilemap size: screens of 32 by 32 entries, 1,024 words each, in the order A B / C D; entry address `base + 1024 s + 32 (r mod 32) + (c mod 32)`; maps wrap at their size.
- [ ] Flips: `H` reads the mirrored column and reverses the pixel order of the word; `V` reads the mirrored row. For 16 by 16 tiles the whole tile is mirrored (quadrants swap).
- [ ] Each layer chooses independently; mixed sizes need no special case.

## Acceptance criteria

- [ ] Test: every combination of tile size x tilemap size x scroll against the reference sampler on random content.
- [ ] Test: flip H/V/both on 8 by 8 and 16 by 16 tiles against the reference.
- [ ] Test: wrap at 256/512/1024 pixels in the right directions.
- [ ] Mode 1 snapshot unchanged for an all-8x8, no-flip scene.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): The reference implements 8 by 8 only without flips (see `flat_run_slot`); this issue is new work. The formulas are in the manual.
