# Change log: chapters/12-instruction-summary.tex

## Findings applied

- **F-con-10 / F-sum-12** (lines 6–7): "approximately 52" corrected to "50 are documented and
  assigned; the remaining 14 are undocumented", with a cross-reference to
  `Appendix~\ref{appendix:undocumented}`.
- **F-sum-23** (lines 10–14): ALU opcode-range bullet corrected to `0x00--0x0E, 0x20--0x2E,
  0x30--0x3E` (was double-counting `0x0F`/`0x2F`/`0x3F`); Coprocessor bullet corrected to
  `(0x0F, 0x2F, 0x3F)`.
- **F-sum-1 / F-con-15** (Complete Instruction List table): Flags cell for every
  logical/test-logical row (`AND`, `ORR`, `XOR`, `TSA`, `TSX` in all three formats — 15 rows)
  changed from `NZ` to `NZ (C,V=0)`. Added a legend line pointing at
  `Table~\ref{tab:flag-effect-symbols}` after the table.
- **F-sum-22**: Format cell filled in for every previously blank "Undocumented" row: `Immediate`
  for `0x08/0x09/0x0B/0x0D`, `Short Imm+Shift` for `0x28/0x29/0x2B/0x2D`, `Register` for
  `0x38/0x39/0x3B/0x3D`; Mnemonic cell left as `--`.
- **F-sum-14 (Gate 2 H)**: `0x27` and `0x2F` reclassified as Undocumented, same class as
  `0x08/0x09/0x0B/0x0D`. Dropped the `LOAD`/`COPI` mnemonics from those two rows, kept
  `Short Imm+Shift` as the Format (the block they decode in), Flags `--`, Operation
  "Undocumented".
- **F-sum-15 (Gate 2 C)**: Added a note after the Complete Instruction List table stating that
  the Flags `--` for `LOAD` (`0x07`, `0x37`) holds only for the default `[N]` AF encoding, that
  an explicit AF suffix on `LOAD` leaves all four flags architecturally undefined, and
  cross-referencing Chapter~13 for the full statement.
- **F-sum-5** (Save vs. Test Variants): "+0x08 to +0x0F: Test only" split into "+0x08 to +0x0E:
  Test only" and a new "+0x0F: Coprocessor call (neither result nor flags are written)" bullet.
- **F-sum-2 (Gate 2 I)**: `NOOP` row corrected from `ADDI[N] r1, #0` to `ADDI[N] sr, #0`. Per the
  ruling, applied the encoding fix only — no note on protected-mode behaviour was added.
- **F-sum-13**: Added a paragraph after the phase-list `enumerate` stating that coprocessor
  calls (`COPI`, `COPR`, and every meta-instruction that assembles to one) take a second 6-cycle
  slot for dispatch, 12 cycles total.
- **F-con-50**: Phase names "Execute/Address Calculate" and "Memory Access (or NOP)" corrected
  to "Execute and Address Calculation" and "Memory Access" to match Chapter 2 and Appendix B.

## Manual-wide mechanical passes applied

- Hex literals in this chapter's prose, bullets, and the Complete Instruction List table's
  Opcode column converted to `\opcode{XX}` (concrete opcode-byte values); the two nibble-pattern
  bullets (`0x0_`, `0x1_`, etc., which are not full opcode values) wrapped in `\texttt{}` instead.
  Hex values already inside `\texttt{...}` syntax examples in the Meta-Instructions table (e.g.
  `COPI #0x1900`) were left as-is — they are illustrative syntax cells, not bare literals.
  No hex literal remains in body font.
- No `e.g.` instances were present in this chapter; none needed converting.
- Checked for a Chapter-12 "Example N-M:" numbering requirement (G1-T4-12, accepted): not
  applicable — this chapter has no instruction-entry `Example:` blocks (the informal "Examples:"
  bullet list under "Save vs. Test Variants" is not an instruction-entry field and was left
  untouched, matching STYLE.md's scope for that decision).

## Findings I could not apply as written

None. Every accepted finding scoped to this file applied cleanly against the current text.

## Open questions

None outstanding — Gate 2 rulings C, H, I, and P (vector count; not applicable to this file's
content) resolved the ambiguities that the underlying findings had flagged as needing an author
decision.

## Verification

`grep -c 'begin{'` and `grep -c 'end{'` both return 10 for the edited file; every added brace is
closed.

## Phase 4

- **F-con2-2 (blocker, applied)**: The Overview's Coprocessor bullet (line 14) still read
  `Coprocessor (\opcode{0F}, \opcode{2F}, \opcode{3F})`, re-asserting the pre-Gate-2 pattern that
  Phase 3's own F-sum-14 fix had already overturned in the Complete Instruction List table (which
  correctly shows `\opcode{2F}` as `-- / Undocumented`). Applied the finding's `fix` verbatim:
  changed the bullet to `Coprocessor (\opcode{0F}, \opcode{3F}): Coprocessor interface` and added a
  new sentence after the list, `\opcode{2F} is undocumented (see
  Appendix~\ref{appendix:undocumented}).`
- **Ruling-H consistency sweep**: Checked every other opcode-range statement in the chapter
  (opcode-count sentence at lines 6–7; ALU-range bullet at line 11; Memory and Address/Control-Flow
  bullets; the "Save vs. Test Variants" bullets at lines 122–124; the Complete Instruction List
  table rows for `0x27`/`0x2F`; the Opcode Organization nibble-pattern bullets; the Instruction
  Timing phase list). All already agree with ruling H (50 documented/assigned, 14 undocumented)
  and with the save (`+0x00`–`+0x07`) / test-only (`+0x08`–`+0x0E`) / coprocessor-call (`+0x0F`)
  split. No further edits were needed.

### Findings I could not apply as written
None.

### Open questions
None.
