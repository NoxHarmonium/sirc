# Changes: assembly/chapters/03-operands.tex

## Findings applied
F-asm-1, F-asm-3 (merged into F-alu-1 decision), F-asm-4, F-asm-5, F-asm-21, F-con-2, F-con-11, F-alu-1 (assembly Ch.3 part).

## Applied differently
- F-con-2 asked for table rows plus a sentence; F-asm-4 asked for a paragraph. Both done: three rows in Table 3.1 and the F-asm-4 paragraph (LJMP/LJSR take `addr`).
- F-asm-5: the Table 8.1 row belongs to another chapter and was not touched.
- F-asm-21/F-con-11 merged into one sentence naming save-result forms such as ADDR and MOVR as the exception.
- Copy edit: caption to Title Case; "architecture manual" to `\archmanual{}`; "keep values in range" rewritten without an imperative.

## Open questions
- F-con-2 evidence says `LJSR (src)+` is legal, while F-asm-4 says LJMP/LJSR take `addr` rather than `(addr)`. The F-asm-4 wording was used.
