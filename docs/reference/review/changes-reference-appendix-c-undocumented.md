# Changes: reference/appendix-c-undocumented.tex

## Findings applied
F-exc-3 (privilege paragraph in Overview; cross-reference in the aliased-write section), F-exc-15 (full list of undefined aliasing cases), F-alu-4 and F-enc-6 (AF = 11 pointer in the expansion list), F-con-16 (chapter already treats 0x27/0x2F as undocumented; no change needed).

## Not applied as written
F-exc-38 is status simulator; left alone. The example `LOAD r1, (#0, p)+` and the sentence that the assembler rejects hazardous forms are unchanged.
The AF = 11 pointer refers to Chapter~\ref{ch:instruction-formats}, where the other editor is adding the rule; it also states the rule here in one sentence.

## Open questions
None.
