# Changes: reference/chapters/02-cpu-architecture.tex

## Findings applied
- F-arch-16 (reset sentence rewritten; label `sec:reset` does not exist, so `sec:reset-vector-fetch` is used)
- F-con-30 (Cancelling -> Canceling)
- F-con-38 (`\textit{SIRC-1 Hardware Reference Manual}` -> `\hwmanual{}`)
- F-arch-7 decision and F-arch-6 (coprocessor call timing: normally six further cycles; a false-condition call or a call naming coprocessor 0x0 has no dispatch slot)

## Not applied as written
- F-con-18 (segment glossary entry): the fix belongs in notation-and-glossary.tex, not this file. Nothing changed here.
- F-arch-1, F-arch-2, F-arch-8: status simulator; manual text kept.
- F-exc-14 (HALT): this chapter makes no HALT statement; nothing to change.
- Copy edit: "barrel shifters support" -> "barrel shifter supports"; "like the DMA coprocessor" -> "such as the DMA coprocessor".

## Open questions
- Should the active-low overbar rule extend to ARCH (`\texttt{RSTI}` remains as is)?
