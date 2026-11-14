# SIRCUS System Manual

The SIRC-1 CPU is documented in three standalone manuals (the CPU was sold separately from the SIRCUS console); SIRCADA and
SIRCAS, which are made for the SIRCUS system, have one each. This manual is the sixth: it describes the console that the
parts are built into. The six parts are:

| Manual | Scope | Location |
| --- | --- | --- |
| SIRC Architecture Reference Manual | ISA, instruction encodings, canonical assembly notation, exceptions, programmer-visible state | `../reference` |
| SIRC Assembly Programmer's Manual | Complete assembly syntax, directives, symbols, meta-instructions, and toolchain usage | `../assembly` |
| SIRC Hardware Reference Manual | Pinout, bus interface, timing, electrical characteristics | `../hardware` |
| SIRCADA Reference Manual | The picture processing unit | `../video` |
| SIRCAS Reference Manual | The sound processor | `../audio` |
| SIRCUS System Manual | Clocks, bus, address map, interrupts, reset, connections and timing between chips | this directory |

This manual covers what no chip manual can: the block diagram, the clock tree for NTSC and PAL, the bus and the address map,
the interrupt and reset wiring, a connection table for every signal of every chip, and the timing relationships between the
CPU, the display and the sound.

## Voice

Like the other manuals, it is written as if it were a period hardware reference for a real console: it states what the system
does, with no comparisons to other products, no mention of simulators, prototypes or tests, and no project status in the
body chapters. Unlike the chip manuals, it **names the real parts** the console is built from (such as the TDA1541A converter),
because it describes the actual design of the console. The SIRCAS and SIRCADA manuals must stay generic, since the chips can
be used with any part of the right specification. Status, open questions and the choices made live in `STATUS.md`.

## Status

This manual is a working draft. It states only what the chip manuals and the system's own clocks fix. Everything else is
marked "not yet specified" in the body and tracked in `STATUS.md`, which lists each open question with an ID (U1, U2, ...) and
the chapters that change when it is answered. **Do not invent values for anything the manual lists as "not yet specified".**

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

The preamble, house style (`../reference/STYLE.md`) and `latexindent` configuration are shared with the other manuals.
References between the manuals are plain text: refer to the others with `\archmanual{}`, `\hwmanual{}`, `\asmmanual{}`,
`\videomanual{}` or `\audiomanual{}` and name the chapter, not its number.

## Keeping the Manual Honest

Every figure comes from one of three places, and must change with it:

| Figure | Source |
| --- | --- |
| Master clocks, line and frame timing, vertical blanking | `../video` (timing chapter) |
| Instruction length, reset hold, pin numbers of the CPU | `../hardware` |
| `CLKI` range, sample period, window latency, memory interface | `../audio` (timing, signal timing and package chapters) |
| Derived figures (instructions per frame, samples per frame, tick drift, load times) | arithmetic of the figures above |
