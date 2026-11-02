# Changes: reference/chapters/16-coprocessor-instructions.tex

## Findings applied
F-con-14, F-con-15 (decision), F-con-10 (chapter sentence only), F-con-16 (decision, Ch.16 part), F-con-35,
F-cop-2, F-cop-3, F-cop-4, F-cop-5 (decision: backward count 0 is a no-op), F-cop-6 (decision), F-cop-7, F-cop-8,
F-cop-9, F-cop-12, F-cop-13, F-cop-14, F-cop-15, F-cop-16, F-cop-17, F-cop-18, F-cop-19, F-cop-20,
F-arch-7 (decision: ID-0 call takes 6 cycles; Timing row, common-semantics row), F-tim-2 (RSET: no dispatch slot in
Write-back and Timing), F-tim-5 (decision: discarded EXCP takes 12 cycles), F-con-38 (cancelled to canceled).
Copy edit: hex literals wrapped in `\texttt{}` and en dashes in prose and tables, "alias" reworded (meta-instruction),
"honest" range-check sentence reworded, straight quotes removed, DMAR/DMAW titles lowercase "to"/"from".

## Not applied as written
- F-cop-8: the range replaced by `0x1C00 | sel<<4 | n`; added one sentence before the table note defining `sel` and `n`.
- F-con-10: only the chapter sentence was changed. The "move to assembly Chapter 8" half belongs to another file; the
  lower-priority DMA count range-check sentences (`the assembler rejects a literal outside that range`, DMAR/DMAW
  Operands) were left because the finding only mentions them as optional.
- F-cop-16 is applied as written; it does not say the 6-cycle interrupt dispatch slot follows (see F-tim-9, Appendix B).
- Simulator-status findings (F-cop-10, 11, 21-29) left alone.

## Open questions
- DMA recovery bullets in "Retrying After a Fault" use imperatives ("correct", "do not return", "Use"); the
  STYLE.md rule on imperatives is ambiguous for ARCH. Left unchanged.
- Should the DMA Operands sentences "the assembler rejects a literal outside that range" move to the assembly manual
  (STYLE.md Scope)?
- `RSTO` stays `\texttt{}` pending the ruling on overbars for ARCH (F-con-32).

## Round 2

- F-con2-2: replaced the "last access performed" clause with the finding's wording (DMAR/DMAW advance takes the command's access type; DMAT advance takes the type in the list above).
- F-con2-21: "hex digit" changed to "hexadecimal digit" (two places).
- No decisions in phase3-decisions.md concern this chapter; no typewriter-font signal-name sentences found.
- Open question: none.
