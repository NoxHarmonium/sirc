# Changes: reference/chapters/12-instruction-summary.tex

## Findings applied
F-sum-1, F-sum-2 (merged into F-arch-7; coprocessor 0 decision: 6 cycles, no dispatch slot), F-tim-2 (RSET has no dispatch slot, 18 cycles), F-sum-3, F-con-21 and F-sum-14 (line 101), F-arch-7.

## Applied differently
- F-sum-1/F-sum-2/F-tim-2/F-arch-7 were merged into one timing paragraph plus a four-item list (false condition, coprocessor 0x0, RSET, longer cases) so each rule is stated once.
- F-con-21 and F-sum-14 conflict on line 101; used one sentence combining F-con-21's second option with a reference to Table tab:flag-effect-symbols. The Flags column itself is unchanged.
- F-sum-3: "dispatch through the normal COPI path" kept without the "6-cycle" wording, as written in the fix.

## Open questions
- The Flags column still uses "NZCV" and "NZ (C,V=0)" rather than the Chapter 11 symbols; converting the column to symbol columns is a structural change left to the author.
- The 18-cycle RSET figure and "WAIT takes longer than 12" are taken from Appendix B; confirm they match after its edit.

Also copy edited: "commonly-used" to "commonly used"; example list now has a full introducing sentence.

## Round 2

- F-con2-8: the +0F line now says coprocessor call is 0F and 3F only, and that 2F is undocumented.
- F-con2-4: the "longer than 12 cycles" item now says "can take longer" and gives the reason for each of WAIT, DMA and maths operations.
- No decisions in phase3-decisions.md concern this chapter, and it has no typewriter-font claims about signal names.
- Open questions: none.

## Improvements

- F-CR-24: Section 12.4 meta-instruction table marked as a summary (caption now 'Meta-Instructions (Summary)'), pointing to Chapter 17 as canonical; added label sec:instruction-summary-meta.
- F-CR-15: the 12-cycle rule and four exceptions in Instruction Timing replaced by a pointer to Appendix B Section coprocessor-call-timing.
