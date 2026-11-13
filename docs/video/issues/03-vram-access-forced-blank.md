# PPU-03: video memory access rules and forced blank

**Milestone:** A. Foundations
**Depends on:** PPU-01, PPU-02
**Labels:** ppu

## Goal

Give the renderer and the CPU separate paths to video memory, and enforce the access rule: the CPU can use video memory and the sprite table only in vertical blanking or while the picture is off (`GD` = 1).

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 2 *Memory Map*: Access Rules, Access During Rendering
- Chapter 3: Base Configuration Register (`GD`)
- Chapter 11: Video Memory Bus (a read of video memory takes one wait state, two after a write)

## State on `main`

- `read_address`/`write_address` on `VideoDevice` serve both the CPU (through the bus) and the renderer's own fetches, so there is no distinction and every CPU access succeeds, even in the middle of a line.
- `GD` (graphics disable) is stored and ignored.
- `examples/basic-video` loads the tileset, tilemaps and palette with CPU stores while the PPU is already rendering; it relies on this.

## Tasks

- [ ] Split an internal raw read used by the renderer from the `MemoryMapped` entry points used by the bus.
- [ ] CPU writes to video memory (`0x8000`-`0xFFFF`) and the sprite table (`0x7000`-`0x70FF`) are ignored, and reads return 0, unless the line is >= 224 or `GD` = 1. Palette and registers are always accessible.
- [ ] Registers, the palette, the sprite table and writes to video memory complete in the cycle they are made (no wait states), whether or not they take effect. A read of video memory that is allowed takes one wait state, two straight after a write; one that is not allowed returns 0 at once.
- [ ] `GD` = 1: the output is black and the device makes no reads. Timing, interrupts and the status flags continue.
- [ ] Check `examples/basic-video` still works: it loads video memory at start-up, which is fine with `GD` = 1 at reset; it clears `GD` when it writes `CONFIG = 0x0078`. Update the example if anything needs reordering.

## Acceptance criteria

- [ ] Test: a CPU write to VRAM during a visible line is ignored and the read returns 0; the same write in line 224+ is stored.
- [ ] Test: with `GD` = 1 the same write succeeds in a visible line.
- [ ] Test: palette and register writes during a visible line take effect (a palette write mid-line changes later pixels).
- [ ] Test: with `GD` = 1 the frame is all black and no VRAM fetch occurs (count reads through the raw path).
- [ ] `examples/basic-video` check target passes.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): No access rule on the branch; the renderer/CPU split is not implemented there. The raw-read idea is sketched in the tests, which read through a helper.

## Notes

Palette RAM is dual-ported in effect: the renderer reads it every pixel and the CPU may write at any time. A write affects pixels drawn after it.
