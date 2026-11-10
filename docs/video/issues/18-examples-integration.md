# PPU-18: examples, integration tests and keeping the manual honest

**Milestone:** E. Tooling and integration
**Depends on:** PPU-04, PPU-05
**Labels:** ppu

## Goal

Keep `examples/basic-video` working through the changes, add an example per mode, and keep the manual's status table in sync.

## Spec

Video Device Reference Manual (`docs/video`):

- `docs/video/STATUS.md`: Implementation Status

## State on `main`

- `examples/basic-video` loads data with CPU stores during rendering, installs a Level 4 vsync handler at word address `0x40`, writes `CONFIG = 0x0078`, and has a register-dump expectation checked by `make check`.

## Tasks

- [ ] Update `basic-video` for each behaviour change that affects it (NMI vector, reset blank state, the Mode 1 picture offset).
- [ ] Add small examples for Mode 2, Mode 3, sprites and the line interrupt, each with a snapshot or register-dump check in CI (`.github/workflows/sirc-vm.yml` already lists `basic-video`).
- [ ] After each issue, update the Chapter 9 status table (the body chapters no longer carry implementation notes).

## Acceptance criteria

- [ ] `examples/check-all.sh` passes; every new example runs in CI.
- [ ] The status table has no row that disagrees with the code.
