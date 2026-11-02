# Changes: reference / 09-shift-operations.tex

## Findings applied
F-enc-7 (new bullet in Assembler Syntax Notes), F-enc-10, F-enc-11, F-enc-12 (RTL/RTR bullets and Operation lines), F-enc-13, F-enc-14, F-enc-18.

F-enc-5 targets Chapter 8 only; no change here (the Table 3.2 row `ADDR r1, r2, LSL #1` already agrees with its fix). F-enc-2 and F-enc-3 are `simulator`; left alone.

## Applied with adjustment
- F-enc-7: the new bullet is worded for this chapter ("`[S]` clears N, Z, C and V") rather than copying the Chapter 7 sentence.
- F-enc-13: label renamed to `@shifted_negative`; comments in the listing lower-cased to match STYLE.md section 7.

## Copy edit
Removed "powerful"; "Use for unsigned/signed values only" became "Should be used ..."; " -- " replaced by a colon; terminal period added to the multi-sentence carry-flag bullet; two captions put in Title Case; all listing comments lower-cased (STYLE section 7).

## Open questions
- Lines 55-56 say the shifted operand is "the second operand" of the three-operand form, while the table calls it the first source operand. Presumably it counts the destination as the first written operand; the wording needs the author's choice.
- Bare listings (Mode 0, Mode 1, Testing Shift Results, Preserving Flags, SHFT) have no `% qa:assemble` marker. I did not add one because the Testing Shift Results and Preserving Flags listings reference undefined labels and would fail.
- Examples in this chapter are not numbered `Example N-M`; that rule is stated for instruction entries only.
