# SIRCAS Reference Manual

The SIRC-1 CPU is documented in three standalone manuals (the CPU was sold separately from the SIRCUS console); SIRCADA and
SIRCAS, which are made for the SIRCUS system, have one each. The five parts are:

| Manual | Scope | Location |
| --- | --- | --- |
| SIRC Architecture Reference Manual | ISA, instruction encodings, canonical assembly notation, exceptions, programmer-visible state | `../reference` |
| SIRC Assembly Programmer's Manual | Complete assembly syntax, directives, symbols, meta-instructions, and toolchain usage | `../assembly` |
| SIRC Hardware Reference Manual | Pinout, bus interface, timing, electrical characteristics | `../hardware` |
| SIRCADA Reference Manual | The picture processing unit | `../video` |
| SIRCAS Reference Manual | The sound processor: package and pinout, memory map, registers, data formats, voices, filter, sequencer, timing | this directory |

This manual describes the sound device (SIRCAS) as a single component: its package and pins, what is mapped where on the
CPU's bus, what each register does, how the device turns audio memory into sound, and the timing of its signals. The
device is one package, so unlike the CPU its pinout and timing live in this manual rather than a separate hardware manual.
Anything about the CPU belongs in the architecture manual; anything about CPU pins belongs in the hardware manual.

## Voice

The manual is written as if it were a period hardware reference for a real console: it states what the device does, with no
comparisons to other products, no mention of simulators, prototypes, tests or design history, and no project status in the
body chapters. It does not name real converters, chips or consoles. Status, open questions and the choices the reference model
made live in `STATUS.md`. Rationale, precedents and the measurements behind the numbers live in `DESIGN-NOTES.md`.

## Status

This manual is a working draft and the device it describes is not finished. It is derived from the draft specification
(`SIRCAS-draft-specification.md`) and the reference model in `sirc-vm/device-audio`, which is its source of truth where the
two differ. `STATUS.md` lists each difference and each choice the model made for a point the draft left open.

Behaviour in the manual that is not in the draft specification (for example the semantics of `WAIT`, `END` and a missed tick)
is **proposed**: it is what the model does, listed in `STATUS.md`, and awaits approval. Two decisions depart from the draft and
have been approved: portamento steps on the envelope clock, and the first envelope step is taken at note-on.

Not yet specified, and so marked in the manual: the package and pin numbers, the electrical interface to the audio memory, the
`SCK` frequency and phase, the serial output edge timing and `BCLK` duty cycle, the reset pulse width, the NTSC segment of the
device, and the reset value of registers not listed in the reset table. The `CLKI` range (9.0 to 12.0 MHz)
is specified, but its limits are proposed targets (`STATUS.md`, Decisions, number 10).

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

The LaTeX preamble, house style (`../reference/STYLE.md`) and latexindent configuration are shared with the other manuals.
`latexindent.yaml` here is a symlink to the architecture manual's copy.

References between the manuals are plain text (a LaTeX `\ref` cannot point into another PDF): refer to the other manuals
with `\archmanual{}`, `\hwmanual{}`, `\asmmanual{}` or `\videomanual{}` and name the chapter, not its number.

## Keeping the Manual and the Model Honest

The numbers in the manual come from the tests and studies in `sirc-vm/device-audio`:

```bash
cd ../../sirc-vm
cargo test -p sirc_umf -p device_audio
cargo run --release -p sirc_umf --example filter_study
cargo run --release -p sirc_umf --example budget_study
cargo run --release -p sirc_umf --example resample_study
cargo run --release -p sirc_umf --example render_demo -- <output dir>   # a demo piece and stems as WAV
```

| Manual figure | Source |
| --- | --- |
| Cutoff and resonance tables (Chapter 10) | `filter_study` |
| Memory schedule, latency, tick limits (Chapter 12) | `acceptance` tests, `budget_study` |
| Pitch resolution and modulation depth tables (Chapters 6 and 9) | `budget_study` |
| Playback error by waveform length (Chapters 6 and 14) | `resample_study` |
| Envelope step and time (Chapter 7) | arithmetic of the 64-sample update |
| Instrument and sequence examples (Appendix B) | `cargo test -p sirc_umf --test recipes -- --nocapture` |
