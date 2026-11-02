# Changes: reference/chapters/01-introduction.tex

## Findings applied
F-arch-4, F-arch-5, F-arch-6, F-arch-7 (with its decision), F-arch-9, F-tim-2 (Ch. 1 part: RSET has no dispatch slot).

## Not applied as written
- F-arch-4: the Hardware row wording was kept as in the fix; the vector-ID column is "0x10--0x5F" and the Purpose cell carries the list of five vectors.
- F-arch-6/7 and F-tim-2 were merged into one passage of the Execution Model paragraph (condition-false call = 6 cycles; coprocessor 0x0 call = 6 cycles, no dispatch slot; RSET has no dispatch slot).
- F-arch-9: the reset-vector sentence was placed in the same position, as a new sentence.
- Other findings that cite this chapter (F-con-13, F-con-14) concern other files; nothing to change here. The \hwmanual/\asmmanual finding needs no change here (no manual titles appear).

## Copy edits
"ideal for" to "intended for"; "ensuring simplicity of CPU implementation" reworded; "will" to present tense; "may raise" an invalid-opcode fault; "emulated" to "implemented"; straight quotes to TeX quotes; hex literals wrapped in \texttt; \reg for r1--r7 in Table; caption "List of Coprocessors".

## Open questions
- Line 49 says the CPU "should not be used in memory-constrained environments"; a design-intent statement, left as advice.
- "32 megabytes of external byte storage" depends on a 2-byte word; left unchanged.
