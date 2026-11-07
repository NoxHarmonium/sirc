# PPU: implement the video device (tracking)

Implement the picture processing unit described by the **SIRC-1 Video Device Reference Manual** (`docs/video`, a 57-page draft;
build with `make` there). The manual is the specification. The code on `main` (`sirc-vm/device-video`) is an early Mode 1
prototype; this tracking issue lists the work to take it to the full device: three video modes, sprites, shading, interrupts
and PAL.

A reference implementation of most of the new features exists, uncommitted, on branch
`claude/video-chip-isometric-mode-f565fa` (`iso.rs`, `sprite.rs`, `iso_tests.rs`, `sprite_tests.rs`, edits to `lib.rs`).
It is there to check ideas against, not to copy: each issue names the relevant parts.

## What exists on `main` today

- Registers `0x0000`-`0x0017` with read/write and a round-trip test; palette at `0x6000`; 32,768 words of video memory at `0x8000`.
- Timing: 1364 master clocks per line, 262 lines, 256 by 224 picture, frame rate from the master clock (NTSC only).
- Mode 1 only: three backgrounds of 8 by 8, 4-bit tiles in one 32 by 32 tilemap each, no scrolling, no flips, no priority,
  no sprites; windowed output via `minifb`; one snapshot test and one per-clock state test.
- A vertical sync interrupt on IRQ4 at line 0.
- `examples/basic-video` and the `sirc-tiledit` asset tool (Qt GUI) that feed it 8 by 8 data.

## Known bugs on `main` (fixed in the issues named)

| Bug | Issue |
| --- | --- |
| Palette decode includes `0x6100`, one past the end (out-of-bounds index) | PPU-01 |
| Status register read byte-swapped relative to all others | PPU-01 |
| Mode 1 picture is 8 pixels too far right: the first 8 pixels of each line are blank, the last tile column never shows | PPU-05 |
| Mode 1 layer order is B3 in front of B1 (reverse of S1,B1,S2,B2,S3,B3) | PPU-08 |
| Mode 1 palette add is unchecked `u8` and can overflow in debug builds | PPU-08 |
| CPU access to video memory is unrestricted and can disturb the picture | PPU-03 |
| `device-terminal` counts one poll as one master clock; the video device counts it as two | PPU-02 |

## Plan

Issues are grouped by milestone and listed in a sensible order; **Depends on** lines in each issue give the hard ordering.

### A. Foundations

- [ ] **PPU-01** register file, memory map and reset state
- [ ] **PPU-02** timing generator, clock accounting and PAL (after PPU-01)
- [ ] **PPU-03** video memory access rules and forced blank (after PPU-01, PPU-02)
- [ ] **PPU-04** vertical sync NMI and line-compare IRQ4 (after PPU-01, PPU-02)

### B. Orthogonal layers and Mode 1

- [ ] **PPU-05** orthogonal layer engine and Mode 1 rewrite (scrolling, fix 8-pixel offset) (after PPU-02, PPU-03)
- [ ] **PPU-06** orthogonal tiles - 16 by 16 tiles, double tilemaps and flips (after PPU-05)
- [ ] **PPU-07** 2-bit orthogonal tiles (T2) (after PPU-05)
- [ ] **PPU-08** compositor - layer order, priority bit and layer disables (after PPU-05)
- [ ] **PPU-09** output stage - graphics disable and brightness (after PPU-08)

### C. Sprites

- [ ] **PPU-10** sprite table, line engine and sprite layers (after PPU-01, PPU-03, PPU-08)
- [ ] **PPU-11** sprite shadow colour and overflow flag (after PPU-10)

### D. Isometric

- [ ] **PPU-12** isometric geometry and staircase tile format (pure model)
- [ ] **PPU-13** isometric layer engine and Mode 2 (after PPU-05, PPU-08, PPU-12)
- [ ] **PPU-14** isometric options - 2-bit tiles, double tilemaps and shading (after PPU-13, PPU-07)
- [ ] **PPU-15** status layer and spare-slot queue (Mode 2, background 3) (after PPU-13, PPU-08)
- [ ] **PPU-16** Mode 3 - two orthogonal backgrounds over one isometric background (after PPU-05, PPU-07, PPU-13)

### E. Tooling and integration

- [ ] **PPU-17** Tools: sirc-tiledit exports for the new tile formats (after PPU-06, PPU-07, PPU-12)
- [ ] **PPU-18** examples, integration tests and keeping the manual honest (after PPU-04, PPU-05)


## Conventions for every issue

- Link the manual sections the issue implements (they are named in each issue) and update the manual's status table
  (Chapter 9) in the same PR.
- Every behaviour has a test; the manual's numbers come from tests, so keep them true. Random-content tests against a plain
  reference sampler are the main tool: see `iso_tests.rs`/`sprite_tests.rs` on the reference branch.
- `cargo test -p device_video`, `cargo clippy` (the crate is `clippy::pedantic`) and `cargo fmt` pass.
- Do not change the manual's behaviour in an implementation PR without discussing it; if the manual turns out to be wrong or
  unclear, fix the manual first (or in the same PR) and say so.

## Open questions (decide before or during the issues named)

- The line-compare interrupt level (IRQ4) and the NMI vector assignment, to agree with the Hardware Manual (PPU-04).

## Decisions recorded while drafting

- `CLKI` is the master clock / 2 (one CPU phase = one PPU cycle = 2 master clocks).
- `GD` = 1 is a forced blank: black output, no renderer reads, video memory and the sprite table open to the CPU.
- Sprite and background 16 by 16 tiles share one quadrant layout.
- Vertical sync is the NMI (no acknowledgement); line compare is IRQ4 held until a write to `LINE`.
