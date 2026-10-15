# Changes — chapters/13-alu-instructions.tex

## Findings applied
- F-cons-2: Rewrote the SHFT instruction box's Instruction Fields shift-count description
  (~line 261) and its Notes bullet (~line 316) from "a register count above 15 is
  architecturally undefined" to the defined-behavior wording used by the other ten ALU
  instruction boxes in this chapter ("a register count is used in full (16 or more clamps
  to a full shift for LSL/LSR/ASL/ASR, and is taken modulo 16 for RTL/RTR)").
- F-cons-14: Dropped the "(status register unchanged)" parenthetical from the AF=11
  reserved-encoding description at line 220, and from all ten instruction-box occurrences
  of "11 = Reserved (status register unchanged)" (found via grep; the finding listed two
  example locations but the phrase appears ten times in this chapter, not six). All now
  read plain "AF encoding 11 (binary) is reserved and has no assembly syntax." / "11 = Reserved."
- F-cons-5: Verified — both occurrences of the LOAD status-override wording (~178-180 and
  ~1219-1221) already match each other and the target wording from the finding. No
  discrepancy found, so no edit was made.

## Findings I could not apply as written
- None. All three findings tagged to this chapter applied cleanly with no textual drift
  from the finding's description.

## Additional copy-edit (not from triaged findings)
- Two instances of a bare `\texttt{0x27}` opcode reference (lines ~107, ~1248) were changed
  to `\opcode{27}` for consistency with STYLE.md §3 and with the chapter's own other use of
  `\opcode{27}` at line 1189.

## Open questions
- None.
