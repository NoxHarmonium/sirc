# PPU-04: vertical sync NMI and line-compare IRQ4

**Milestone:** A. Foundations
**Depends on:** PPU-01, PPU-02
**Labels:** ppu

## Goal

Implement the two interrupts as specified: vertical sync on the NMI with no acknowledgement, and line compare on IRQ4, held until the program writes `LINE`.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 8 *Timing*: Interrupts (Vertical Sync, Line Compare)
- Chapter 3: Interrupt Control Register, Line Compare Register, Status Register (`LIP`)
- Architecture Manual: Interrupt Recognition and Latency, Level 5 NMI-Like Behavior; Hardware Manual: IRQ1-IRQ4, NMI

## State on `main`

- `VSYNC_INTERRUPT = 0x1 << 3` (IRQ4, level 4) is asserted for one PPU cycle at line 0, clock 0, always (no enable). `test_video_line_timing` asserts exactly that.
- `sirc-vm/src/lib.rs` uses `device_video::VSYNC_INTERRUPT` in two loops to decide when a frame has finished.
- `examples/basic-video` installs its handler at vector word address `0x40` (Level 4) and enables interrupts with `ORRI sr`.
- The simulator bus has `BusAssertions::interrupt_assertion: u8`: bit n-1 is IRQn (bits 0-3) and bit 4 is the Level 5 NMI line (see the reset tests in `peripheral-cpu`). `device-terminal` uses IRQ2 (`0x2`).

## Tasks

- [ ] Vertical sync: when `INTCTL.VIE` = 1 pulse the NMI bit (`0x10`) for one PPU cycle at the first cycle of line 224. No acknowledgement. `VIE` is 0 at reset.
- [ ] Line compare: when `INTCTL.LIE` = 1 assert IRQ4 (`0x08`) at the first cycle of line `LINE` (bits 8-0) and hold it until the program writes any value to `LINE`, or clears `LIE`. `STATUS.LIP` shows it pending. A `LINE` at or beyond the frame length never matches. A second match while pending is not queued.
- [ ] `sirc-vm`: the frame loop must not depend on `VIE` or on which pin vsync uses. Add a device-side frame-complete signal (or use `frame_count`) so existing frame pacing keeps working.
- [ ] Update `examples/basic-video`: move the vsync handler to the Level 5 vector (vector `0x10`, word address `0x20`), set `VIE`, and check its register-dump expectation.
- [ ] Update Chapter 9's status table.

## Acceptance criteria

- [ ] Test: with `VIE` set, exactly one NMI pulse per frame, at line 224 clock 0, one PPU cycle wide; none with `VIE` clear.
- [ ] Test: line compare asserts at the chosen line and stays asserted across lines until `LINE` is written; `LIP` follows it.
- [ ] Test: writing `LINE` deasserts it and re-arms for the new line; clearing `LIE` deasserts it.
- [ ] Test (VM level): a program that never sets `VIE` still runs at the paced frame rate.
- [ ] `examples/basic-video` check target passes with its handler on the NMI vector.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): Line compare exists on the branch in an older arrangement (level 3, one-cycle pulse); the NMI move and the ack/hold behaviour are not implemented there. Test `the_line_compare_interrupt_fires_once_per_frame_at_its_line` shows the shape.

## Notes

A DMA transfer cannot be interrupted, the NMI included; the edge is latched and taken afterwards. That is CPU behaviour (Architecture Manual), not something the PPU implements.
