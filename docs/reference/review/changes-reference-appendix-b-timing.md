# Changes: reference / appendix-b-timing.tex

## Findings applied
F-tim-1, F-tim-2, F-tim-3, F-tim-5, F-tim-6, F-tim-7, F-tim-8, F-tim-9, F-tim-11 (phase 3 kept, no text change), F-tim-14, F-hw-1 (author decision: "example 24 MHz clock" at both places, "maximum rated clock" dropped; F-tim-10 and F-con-3 merged into it), F-arch-7 (COPI/COPR naming coprocessor 0x0 takes 6 cycles, in the overview and the COPI/COPR row), F-con-12 (rows 40-41 relabeled "Conditional processing-unit instruction"), F-con-38 (title replaced by `\hwmanual{}`), the cancelled to canceled finding (three hits).

## Not applied as written
- F-tim-4, F-tim-12, F-tim-13: status simulator, left alone.
- F-tim-2: the RSET exception is stated in the overview as a sentence that also covers the coprocessor-0x0 case, not as an inline "except" clause.
- F-tim-1: added "unsigned" to both comments (the optional part of the fix).
- F-con-32: the ARCH overbar rule is undecided, so `\texttt{BACK}`, `\texttt{BERR}`, `\texttt{BPER}`, `\texttt{RSTO}` and `\texttt{RSTI}` are unchanged.
- F-arch-2 and F-hw-21 name this chapter only as evidence and need no change.

## Open questions
- `\hwmanual{}` is not defined in `docs/reference/main.tex` (F-con-38 asks for it). The build fails until that macro is added.
- The table cells for COPI/COPR, EXCP, RSET and WAIT are now long and wrap in the 6.5 cm column. Consider widening the second column.

## Round 2
- F-con2-5: the $4w$ wait-state cost now covers an accepted EXCP or RSET.
- F-con2-6: added the three exceptions to the exception-vector fetch fault rule (double fault, reset, repeated RSTO hold).
- No decision in phase3-decisions.md concerns this chapter; no typewriter-font signal names found.
- Open questions: none.
