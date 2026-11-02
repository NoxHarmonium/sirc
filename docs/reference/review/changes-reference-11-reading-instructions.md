# Changes: reference/11-reading-instructions

## Findings applied
- F-con-29 (labelled -> labeled)
- F-sum-5 (secondary location: `@label` row now reads "a low-word address in the memory, effective-address and MOVI operands that accept one")
- F-alu-4 / AF = 11 decision (applied to the Status Update Source Encoding table row only: "Reserved; N, Z, C and V architecturally undefined"; Ch.11 is not named by the decision, changed so the rule agrees across chapters)

## Not applied as written
- F-alu-2: the Ch.11 text ("SHFT defaults to [S]") already agrees with the accepted fix; no change.

## Open questions
- "SIRCIS" in the opening sentence: confirm the intended name (SIRC-1?).
