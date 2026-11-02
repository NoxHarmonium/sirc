# Changes: reference/chapters/14-memory-instructions.tex

## Findings applied
F-mem-4, F-mem-5, F-mem-6, F-mem-7, F-mem-8, F-mem-9 (STOR R1 field only), F-mem-10, F-mem-11, F-mem-12.
F-con-30 lists this file but it has no "cancelled" forms, so nothing changed.

## Not applied as written
- F-mem-6: besides the table row and Status Flag Effects paragraph, I qualified Table `tab:memory-status-flag-effects` (LOAD row) and the LOAD `Flags:` field. I added `\label{sec:memory-status-flags}` to the Status Flag Effects section so the common-semantics row can cross-reference it.
- F-mem-4: the clause was added to the narrative paragraph and to the LOAD and STOR Exceptions fields (post-increment and pre-decrement respectively).
- F-mem-8: appended to the Ch14 "Condition false" row only; the other two locations are in Ch15.
- F-mem-7: the loaded `Description` text and Instruction Fields were left unchanged; the new wording is in `Operands`.
- F-mem-2 (simulator) and F-exc-15 (appendix) were not touched. The "Aliased register writes" paragraph still points to Appendix C.
- Pass two: removed the filler "seamlessly".

## Open questions
- Operation pseudocode and the Privilege field still say little about LOAD into `sr`/`pl`; confirm whether to cross-reference these from the Description.
- LOAD "Condition field" and Ch14 Common-semantics prose do not mention the segment-overflow check for a skipped instruction beyond the table row.
