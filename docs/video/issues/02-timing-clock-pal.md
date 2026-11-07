# PPU-02: timing generator, clock accounting and PAL

**Milestone:** A. Foundations
**Depends on:** PPU-01
**Labels:** ppu

## Goal

Pin down how a `poll` relates to the CPU clock and the master clock, make the frame geometry configurable (NTSC 262 lines, PAL 312), and expose the line/frame counters the other issues rely on.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 8 *Timing*: Clocks, The CPU Clock, Line Timing, Frame Timing (PAL)
- Chapter 3 *Registers*: Status Register (`OUT`)

## State on `main`

- `new_video_device(master_clock_freq: usize)` hard-codes 1364 master clocks per line and `TOTAL_LINES = 262`; frame rate is computed from the master clock; there is no region.
- `poll` advances `line_clock` by 2 per call, i.e. the video device assumes one poll is one PPU cycle (two master clocks).
- `sirc-vm` polls the CPU and every device once per CPU phase (`BusPeripheral::poll_all`), so in the VM one poll is one `CLKI` cycle. That only agrees with the video device if `CLKI` = master clock / 2, which is what the manual now states.
- **Inconsistency:** `device-terminal` increments `clock_counter` once per poll and divides `master_clock_freq` by the baud rate, i.e. it treats one poll as one master clock. Its baud rate is therefore off by a factor of two relative to the video device.

## Tasks

- [ ] Document the clock model in code as named constants (`CLKI` = 1 PPU cycle = 2 master clocks) and use it everywhere.
- [ ] Make the lines per frame a property of the region: NTSC 262, PAL 312; visible lines 0-223; vertical blanking starts at 224.
- [ ] Add a region to the constructor (and to the `sirc-vm` command line); `STATUS.OUT` reflects it. PAL frame rate with a 21,281,370 Hz master clock is 50.0 Hz.
- [ ] Decide and fix the terminal device's clock accounting (or file a separate issue for it); do not leave the two devices disagreeing.
- [ ] Expose `line`, `line_clock` and a frame counter for tests (already `pub` on `main`).

## Acceptance criteria

- [ ] Test: an NTSC device completes a frame in 262 x 1364 master clocks (178,684 polls); PAL in 312 x 1364 (212,784 polls).
- [ ] Test: `vsync_frequency` is 60.1 Hz (NTSC at 21,477,272 Hz) and 50.0 Hz (PAL at 21,281,370 Hz).
- [ ] Test: `line_clock` wraps at 1364 and the line counter at the frame's line count.
- [ ] The terminal device (or a linked issue) has a test that one baud period is the right number of master clocks.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): Constants and `new_video_device` in `lib.rs` (NTSC only on the branch); no PAL yet.
