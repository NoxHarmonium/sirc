# PPU-17: Tools: sirc-tiledit exports for the new tile formats

**Milestone:** E. Tooling and integration
**Depends on:** PPU-06, PPU-07, PPU-12
**Labels:** ppu

## Goal

Make the asset tool able to produce the data the PPU now consumes. The existing 8 by 8, 4-bit square tile output is unchanged and stays valid; this issue is only about the new formats.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 4 *Data Formats*: Mode 1 Tile Layout, 16 by 16 Orthogonal Tiles, 2-Bit Orthogonal Tiles, Tilemap Entries, Tilemap Sizes
- Chapter 6: Authoring Tiles, A Worked Example; Chapter 7: Sprite Table Entry, Sprite Tile Data

## State on `main`

- `ImageTileSlicer::slice` cuts a 256 by 256 image into a fixed 32 by 32 tilemap of 8 by 8 tiles, packs four 4-bit pixels per 16-bit word in row-major order (16 words per tile, two per row) and deduplicates tiles by hash (no flip matching). This matches the PPU's 8 by 8 format, and `examples/basic-video` renders it correctly.
- `TileSize::SixteenBySixteen` (= 64 words) exists in the enum but `slice` throws "not supported yet". When it is implemented it must use the manual's **quadrant layout** (four 8 by 8 quadrants: TL, TR, BL, BR), not plain row-major.
- `ImageExporter` writes tilemap entries as the bare tile index: palette select, flip and priority bits are all 0, which stays valid. Multiple palettes are exported (`palette__N_M`) but nothing writes the 3-bit palette select into entries, and there is a `TODO: Pad palette to 16 pixels?`, so more than one palette per layer is not usable yet.
- One shared tileset for all tilemaps with a tile index assertion `< 0x3FF`; transparent colour is palette index 0 (black); the exporter emits `.sasm` data (`tileset_0`, `tilemap__P_N`, `palette__P_N`).
- README roadmap: tile export is not ticked, tilemap and sprite management are not done. I have not audited the GUI.

## Tasks

- [ ] 16 by 16 orthogonal tiles (quadrant layout) from 512 by 512 images; 2-bit tiles (8 words per 8 by 8 tile) for images with <= 3 colours plus transparent.
- [ ] Tilemaps up to 64 by 64 in screen order A B / C D (4 screens of 1,024 words).
- [ ] Emit the palette select bits per tile when more than one palette is used, and pad palettes to the layer's palette size (2-bit layers want `SIZE` = 4 colours).
- [ ] Isometric baker: slice a screen-space picture for a layer with a given scroll into a tilemap and unique staircase tiles (the algorithm in `bake_layer`), 4-bit and 2-bit.
- [ ] Sprite sheets (same tile layouts as backgrounds) and sprite table export.
- [ ] Capture the tool's conventions (256 by 256 source = 32 by 32 map, index 0 = transparent, shared tileset and its limit, label names) in the tiledit README, linking the manual for the hardware formats.

## Acceptance criteria

- [ ] Round-trip tests: bake an image, render it with the simulator, compare pixel for pixel (the isometric baker, 16 by 16 and 2-bit).
- [ ] Existing exporter tests and the `basic-video` data are byte-identical after the change (the 8 by 8 output must not move).

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): `bake_layer`/`bake_layer_2bit` in `iso.rs` define the isometric algorithm and show the expected tile counts for the demo scene.
