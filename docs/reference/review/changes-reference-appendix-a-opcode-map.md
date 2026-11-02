# Changes: reference/appendix-a-opcode-map.tex

## Findings applied
- F-con-16 (per the author's decision): opcodes 27 and 2F removed from the MOV and COP cells of the Instruction Variant Quick Reference table and given an "Undocumented" row. The asterisk footnote is replaced by a sentence that refers to Appendix C and says they are not short-immediate MOVI or COPI forms. Table A.1 and the Undocumented Instructions section already treated them as undocumented.

## Not applied as written
- None. The Ch.13 and Ch.16 edits in the decision belong to those chapters' editors.

## Copy edit
- "Do not use them in production code" became "Software must not use them" (no imperative or second person; normative vocabulary).
- latexindent re-aligned the longtable columns, which accounts for the larger diff.

## Open questions
- The caption "Complete SIRCIS Opcode Map" and the intro use "SIRCIS", while the rest of the manual says SIRC-1. Left alone; confirm the intended name.
