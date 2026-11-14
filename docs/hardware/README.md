# SIRC-1 Hardware Reference Manual

The SIRC manual set has three parts:

| Manual | Scope | Location |
| --- | --- | --- |
| SIRC Architecture Reference Manual | ISA, instruction encodings, canonical assembly notation, exceptions, programmer-visible state | `../reference` |
| SIRC Assembly Programmer's Manual | Complete assembly syntax, directives, symbols, meta-instructions, and toolchain usage | `../assembly` |
| SIRC Hardware Reference Manual | Pinout, bus interface, timing, electrical characteristics | this directory |

This manual describes the SIRC-1 as a physical device. Anything that is about what the processor *does* (instructions,
registers, exception semantics) belongs in the architecture manual; anything about what appears on the *pins* belongs
here. Where the two overlap, the architecture manual is authoritative for behaviour inside the CPU and this manual is
authoritative for what is driven or sampled on each signal.

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

The LaTeX preamble, house style (`../reference/STYLE.md`) and latexindent configuration are shared with the architecture
manual, so the two read as a set. `latexindent.yaml` here is a symlink to the architecture manual's copy.

References between the manuals are plain text (a LaTeX `\ref` cannot point into another PDF): refer to the
architecture manual with `\archmanual{}` and name the chapter, not its number.

## Status

Electrical characteristics (voltage levels, clock range, output delay, set-up and hold, loading) are provisional targets,
my best guess from late-1980s CMOS CPUs, and are marked as such in the last chapter. Supply current, power and the
package are not specified yet. Do not invent values for those.
