# Changes — chapters/10-condition-codes.tex

## Findings applied
- F-data-19: "Most SIRCIS instructions..." -> "Every SIRC-1 instruction...", also fixes the stray processor name "SIRCIS".
- F-data-18: Reworded the false-condition sentence (line 9-10) and the NV entry (Special Conditions) to state "no
  register, flag, memory, address-register or coprocessor state changes, and no privilege check is performed"
  instead of defining a false condition in terms of NOOP.
- F-data-5 / F-con-19 (duplicate fix, same target line): `ADDI r1, #0, #10 ; i = 10` -> `LOAD r1, #10 ; i = 10` in
  the Loop Construction example. (F-con-19 also touches appendix-b-timing.tex:74, which is out of scope for this
  file/editor.)
- F-data-15: Replaced the "Conditional instructions do not branch, avoiding pipeline flushes" bullet with the
  six-cycles/no-pipeline-to-flush explanation from the finding's fix text.
- G1-T4-10: Numbered the seven example subsections `Example 10-1` through `Example 10-7` (Unsigned Comparison,
  Signed Comparison, Equality Testing, Conditional Assignment, Conditional Increment, Conditional Function Call,
  Loop Construction), following the `Example N-M: Title` convention already used in Chapters 3, 5, and 8.

## Findings skipped
- F-data-1 (`status: code-wrong`): per Gate 2 ruling B, the ARM-style HI/LO/CS/CC glosses, the truth tables, and the
  worked examples are correct as written; the emulator's predicates are to be fixed instead. No text in this
  chapter was touched on this point (lines 30-31, 74-77, 80-86, 138/142-equivalent lines, 181-184, 219-223, 249,
  263-266 all left as-is).
- F-con-1: filed against `chapters/07-instruction-formats.tex`, not this file. Chapter 10's own encoding table
  (`tab:condition-codes`) already has `1100 = <<` (Signed Less Than) and `1110 = <=` (Signed Less or Equal), which
  is the resolution Chapter 7 is being brought into line with — no change needed here.
- All other findings in triaged.md with `file` outside `chapters/10-condition-codes.tex`, and any `defer`/`rejected`
  entries, were left untouched (none of the latter applied to this chapter besides the code-wrong item above).

## Copy edit (beyond triaged findings)
- Tightened "specifies under what conditions the instruction should execute" to "specifies the conditions under
  which the instruction executes" (removed a non-normative "should").
- Fixed a tense inconsistency: "Signed overflow occurred." -> "Indicates signed overflow." (matches the present-tense
  pattern of the surrounding flag descriptions).
- Added terminal periods to full-sentence list items in Performance Considerations and Common Pitfalls that were
  missing one, per STYLE.md §5 ("full sentences in a list get a period").
- Wrapped the bare mnemonic in the Selection Guide table header, "After CMP", as `After \mnemonic{CMP}`, matching
  the manual-wide rule that instruction mnemonics always use `\mnemonic{}` outside of code/syntax cells.
- No `e.g.`, no bare hex literals, and no bare-`rN` register violations were found in this chapter; no changes
  needed for those mechanical passes.

## Open questions
- None. The chapter's remaining HI/LO/CS/CC unsigned-comparison language is intentionally left as-is per Gate 2
  ruling B; it is not an open question, just a documented skip.

## Verification
`grep -c 'begin{' 10-condition-codes.tex` and `grep -c 'end{' 10-condition-codes.tex` both report 24.

## Phase 4

Findings applied:
- F-con2-27: In both truth tables (Unsigned Comparison, Signed Comparison), replaced the don't-care `*` glyph with `X` to avoid collision with Chapter 11's `*` = "updated" flag symbol. Added a one-line legend under each table: "`X` = the flag value does not affect this relationship." Gave the LO row in the Unsigned Comparison table a real predicate by spanning the Z/C columns with the expression "C = 0 OR Z = 1" (per Gate 2 ruling B: LO = unsigned lower or same), instead of leaving it as `X X` with no stated relationship.

Findings not applied as written: none; the fix was applied as specified, using the multicolumn-expression option offered in the finding for the LO row rather than the "1 or --" alternative, since "--" is already reserved for "preserved" in the flag-symbol set and would reintroduce the same ambiguity this fix is meant to remove.

Open questions: none.
