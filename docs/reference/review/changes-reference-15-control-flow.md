# Changes: reference / 15-control-flow.tex

## Findings applied
- F-mem-1 (LDEL and LJSR Operation reordered; source-read sentence added to both Descriptions; syntax comments reordered so the link write follows the destination write).
- F-mem-4 (Ch15 part: auto-update clause added to the Exceptions row, the paragraph after the common-semantics table, and the LDEA and LDEL Exceptions fields).
- F-mem-8 (Ch15 part: segment-overflow parenthetical added to the Overview sentence and the Condition false row).
- F-mem-9 (R2 and shift-field line added to the LDEA, LDEL, LJMP and LJSR field lists).
- F-mem-13 (BRAN and BRSR `$name` rows added to Table 15.1).
- F-mem-14 (lh/ph sentence added to the BRSR, LDEL and LJSR Privilege fields).
- F-mem-15 (sentence deleted from the LJMP src field).
- Copy edit: "The SIRC-1 provides" to "SIRC-1 provides"; two imperatives ("use ...") recast as statements, per STYLE.md section 5.

## Not applied as written
- F-mem-4: the "LDEA/LDEL Exceptions paragraphs" wording says "source auto-update" rather than repeating the full clause; the full clause with the cross-reference is in the shared paragraph.
- F-mem-9: the finding says "R2 (bits 21--18) and the shift fields (bits 13--6)"; written as one list item, "R2 and shift fields".
- F-mem-2, F-mem-3: status simulator, left alone.
- F-con-15, F-con-1, F-con-38, F-arch-10, F-mem-12: name this chapter only as evidence or are fixed in other files.

## Open questions
- None.
