# Changes: chapters/12-instruction-summary.tex

## Findings applied
- F-cons-5 (merges F-data-3, F-sum-14): reworded the LOAD status-override sentence to
  "an encoding of the LOAD opcode with a non-[N] AF field, not reachable from public
  assembly syntax, leaves all four flags architecturally undefined (U U U U)".
- F-cons-9: corrected the save/test-only opcode-offset ranges to +0x00–0x06 (save),
  +0x07 (LOAD, noting 0x27 is undocumented), +0x08–0x0E (test-only where a documented
  mnemonic exists), +0x0F (coprocessor call).

## Findings applied with deviation
- None. Both fixes applied as written (with macro wrapping per STYLE.md).

## Open questions
- None.
