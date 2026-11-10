# PPU-01: register file, memory map and reset state

**Milestone:** A. Foundations
**Depends on:** none
**Labels:** ppu

## Goal

Make the device's registers, palette and memory map match the manual, fix the two known decode bugs, and give the device its specified reset state.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 2 *Memory Map*: Access Rules, The Palette Boundary
- Chapter 3 *Registers*: every register section, *Reset State*
- Chapter 9: Discrepancies (register addresses, palette boundary, status byte order)

## State on `main`

- `sirc-vm/device-video/src/types.rs` has `PpuRegisters` with registers `0x0000`-`0x0017` as `modular_bitfield` structs, read/write handlers and `test_address_round_trip`.
- **Bug:** the palette decode in `lib.rs` is `0x6000..=0x6100` (inclusive) but the palette array has 256 entries, so an access to `0x6100` indexes out of bounds and panics.
- **Bug:** the status register is read with `u16::from_be_bytes`, which is byte-swapped relative to every other register (harmless only while the value is 0).
- No sprite table (`0x7000`-`0x70FF`), no `INTCTL` (`0x0011` in the old layout), no `LINE` (`0x0015` in the old layout), no `SHD` (CONFIG bit 7), no `T2` (palette config bit 2), no `SOV`/`LIP` status bits. The status register always reads 0 (version 0, NTSC).
- Reset state is all zeros, so brightness is 0 and the picture is enabled. The manual specifies `CONFIG = 0x8078`.

## Tasks

- [ ] Fix the palette decode to `0x6000..=0x60FF` (reads of `0x6100` return 0, writes are ignored, no panic).
- [ ] Fix the status register byte order so it reads like the other registers.
- [ ] Repack the register map into groups of four backgrounds as the manual's register map shows (`B1MAP`-`B4MAP` at `0x0002`-`0x0005`, tile data, scroll X, scroll Y and palette configuration likewise; `INTCTL` `0x0016`, `LINE` `0x0017`, `STILES` `0x0018`, `STATUS` `0x0019`, `VLINE` `0x001A`) and add the fourth background's registers and `B4D`, `B4S`, `B4X`, `B4Y`. Update the existing tests that use the old offsets.
- [ ] Add the new fields/registers: `INTCTL` (`VIE` bit 0, `LIE` bit 1), `LINE` (bits 8-0), `CONFIG.SHD` (bit 7), `BxPAL.T2` (bit 2), `STATUS.SOV` (bit 5) and `STATUS.LIP` (bit 6). Behaviour comes from later issues; this issue only stores and reads them.
- [ ] Add the sprite table storage: 256 words at `0x7000`-`0x70FF`, open bus elsewhere in `0x7000`-`0x7FFF`. Fill it with ones at reset (hides every sprite) or document another choice.
- [ ] Reset values: `CONFIG = 0x8078` (`GD` = 1, `BRT` = 15, `MODE` = 0), every other register 0. Video memory, palette and sprite table contents are undefined on hardware; the simulator clears them.
- [ ] `VER` reads 1 and the region (`OUT`) is set by the constructor, not hard-coded (the simulator must be able to configure both).
- [ ] Update the manual's status table (Chapter 9) for the rows you complete.

## Acceptance criteria

- [ ] Unit test: reads and writes round trip for every writable register and every field of the new registers.
- [ ] Unit test: `0x60FF` is the last palette word; `0x6100` reads 0 and a write to it changes nothing.
- [ ] Unit test: a fresh device reads `CONFIG = 0x8078`, all other registers 0, `STATUS` = `VER` 1 (and `OUT` per constructor).
- [ ] Unit test: `STATUS` is read-only and reads the same bit layout as the manual's Status Register figure.
- [ ] `cargo test -p device_video` and `cargo clippy` pass; the existing Mode 1 snapshot test still passes.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): `types.rs` (`StatusRegister`, `PaletteRegister`, `BaseConfigRegister`), tests in `iso_tests.rs`/`sprite_tests.rs` (`the_palette_ends_at_0x60ff`, `mode_register_defaults_to_orthogonal`).
