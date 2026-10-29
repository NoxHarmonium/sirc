# SIRC-1 Assembly Programmer's Manual

The SIRC manual set has three parts:

| Manual | Scope | Location |
| --- | --- | --- |
| SIRC Architecture Reference Manual | ISA, instruction encodings, canonical assembly notation, exceptions, programmer-visible state | `../reference` |
| SIRC Assembly Programmer's Manual | Complete assembly syntax, directives, macros, pseudoinstructions | this directory |
| SIRC Hardware Reference Manual | Pinout, bus interface, timing, electrical characteristics | `../hardware` |

This manual describes the assembly language and the toolchain (`assembler`, `linker`, `sirc_vm`) as they exist in this
repository. The architecture manual defines what each instruction does; this manual does not repeat that. It documents
only what the tools actually do, and lists what they do not support (chapter 8). When the toolchain gains a feature
(macros, includes), update that chapter at the same time.

## Building

```bash
# Build the PDF (three passes, then checks for unresolved references)
make

# Format the LaTeX with latexindent, or check formatting without changing anything
make format
make format-check

# Assemble every code listing marked `% qa:assemble` with the real assembler
make listings

# Everything CI runs: format check, full build, and the listings
make qa
```

Code listings that are real assembly must be preceded by a `% qa:assemble` comment so `make listings` checks them with
the real assembler (the extractor is shared with the architecture manual: `examples/manual-examples/extract-listings.awk`).
Listings of shell commands and diagnostics are written with `[language={}]` and are not checked.

The LaTeX preamble, house style (`../reference/STYLE.md`) and latexindent configuration are shared with the architecture
manual. References between the manuals are plain text: name the chapter, not its number (`\archmanual{}`, `\hwmanual{}`).
