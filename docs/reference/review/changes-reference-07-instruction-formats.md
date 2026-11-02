# Changes: reference/chapters/07-instruction-formats.tex

## Findings applied
F-enc-1, F-enc-6 (merged into F-alu-4), F-alu-4 (author decision: AF = 11 leaves N, Z, C, V architecturally undefined, result still written, assemblers never emit it; applied to the three AF lists and the three decode-tree AF rows), F-con-20, F-enc-16 (same fix: column renamed "Test-only opcodes (Imm / Short / Reg)"), F-enc-7, F-enc-17.

## Not applied as written
None. F-con-16 (0x27/0x2F) needs no change here; Chapter 7 already calls them undocumented.

## Open questions
- Lines about LDEA/LDEL bits 25--24 say "always write them as zero", which STYLE.md says reserved fields should not require. Left unchanged because removing the requirement is a technical decision.
- The immediate-format notes list AF for ALU ops as 00/01/10 only; the 11 case is covered in the field descriptions, not repeated in the notes.

## Round 2

Applied:
- Decision 7: the reserved LDEA/LDEL bits 25--24 now read "are written as zero; a non-zero value is architecturally undefined" in both the immediate and register field descriptions (the "future revision" clause is dropped).
- Added AF = 11 (reserved; N, Z, C and V architecturally undefined, result still written) to the AF notes under the immediate, short immediate and register encoding examples.
- No consistency-2 findings target this chapter; no typewriter-font signal-name sentences found.

Open questions: none.

## Round 2

Applied:
- Decision 7: the reserved LDEA/LDEL bits 25--24 now read "are written as zero; a non-zero value is architecturally undefined" in both the immediate and register field descriptions (the "future revision" clause is dropped).
- Added AF = 11 (reserved; N, Z, C and V architecturally undefined, result still written) to the AF notes under the immediate, short immediate and register encoding examples.
- No consistency-2 findings target this chapter; no typewriter-font signal-name sentences found.

Open questions: none.
