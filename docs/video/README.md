# SIRCADA Reference Manual

The SIRC-1 CPU is documented in three standalone manuals (the CPU was sold separately from the SIRCUS console); SIRCADA, which
is made for the SIRCUS system, has its own. The four parts are:

| Manual | Scope | Location |
| --- | --- | --- |
| SIRC Architecture Reference Manual | ISA, instruction encodings, canonical assembly notation, exceptions, programmer-visible state | `../reference` |
| SIRC Assembly Programmer's Manual | Complete assembly syntax, directives, symbols, meta-instructions, and toolchain usage | `../assembly` |
| SIRC Hardware Reference Manual | Pinout, bus interface, timing, electrical characteristics | `../hardware` |
| SIRCADA Reference Manual | The picture processing unit: package and pinout, memory map, registers, data formats, video modes, timing | this directory |

This manual describes the video device (the PPU) as a single component: its package and pins, what is mapped where on the
CPU's bus, what each register does, how the device turns video memory into a picture, and the timing of its signals. The
device is one package, so unlike the CPU its pinout and timing live in this manual rather than a separate hardware manual.
Anything about the CPU belongs in the architecture manual; anything about CPU pins belongs in the hardware manual.

## Voice

The manual is written as if it were a period hardware reference for a real console: it states what the device does, with no
comparisons to other products, no mention of simulators, prototypes, tests or design history, and no project status in the
body chapters. Status, open questions and the discrepancies with the old wiki live only in the last chapter (Status and Open Issues), which is a working
record that is not part of the specification. Rationale, precedents (the Super Nintendo PPU, the Mega Drive VDP and so on) and
the measurements behind the numbers live in `DESIGN-NOTES.md`, not in the manual.

## Status

This manual is a working draft and the device it describes is not finished. It is derived from the reference
simulator in `sirc-vm/device-video`, which is its source of truth. Where the simulator and the older wiki page
(`PPU-(video-device)`) disagree, the simulator wins and the disagreement is listed in the last chapter.

The isometric Mode 2, the orthogonal-over-isometric Mode 3, sprites, shading and 2-bit tiles (Chapters 6, 6b and 7) are
**proposed**, not approved. They are prototyped in the simulator (`sirc-vm/device-video`: `iso.rs`, `sprite.rs` and
the lib), and the numbers in them come from tests in that crate. Several features are specified here but not yet
implemented there; the status table in the last chapter says which. Do not read the proposals as a commitment to build
them.

Do not invent values for anything the manual lists as "not yet specified".

## Building

```bash
# Build the PDF (three passes, then checks for unresolved references)
make

# Format the LaTeX with latexindent, or check formatting without changing anything
make format
make format-check

# Everything CI runs: format check plus a full build
make qa
```

The LaTeX preamble, house style (`../reference/STYLE.md`) and latexindent configuration are shared with the other
manuals. `latexindent.yaml` here is a symlink to the architecture manual's copy.

References between the manuals are plain text (a LaTeX `\ref` cannot point into another PDF): refer to the other manuals
with `\archmanual{}`, `\hwmanual{}` or `\asmmanual{}` and name the chapter, not its number.

## Keeping the Manual and the Simulator Honest

The numbers in Chapters 3, 4, 6 and 8 are checked by the unit tests in `sirc-vm/device-video`:

```bash
cd ../../sirc-vm && cargo test -p device_video -- --nocapture
```

`--nocapture` prints the figures the manual quotes (for example the share of pixel groups that straddle two tiles).
The pictures in Figures 6.1, 6.2 and 7.1 are snapshots (`iso_demo`, `iso_shading_demo`, `iso_sprite_demo`), for example `src/snapshots/device_video__iso_tests__iso_demo.snap.png`; regenerate
`figures/iso-demo.png` from it when the demo scene changes.
