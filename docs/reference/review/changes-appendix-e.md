# Changes — Appendix E (Quick Reference)

File owned: `docs/reference/chapters/appendix-e-quick-reference.tex`

## Findings applied

- **F-con-7** / **F-sum-3** (line 25, ADCI/ADCR split): split the merged
  `0x01/0x21/0x31` opcode list into an `ADCI` row (`0x01`/`0x21`, "Add with
  carry, immediate or short immediate") and a new `ADCR` row (`0x31`, "Add
  with carry, register"), so the alphabetical index is complete. Both
  findings targeted the same cell with the same intent; applied once.
- **F-sum-4** (lines 72–74, RETE/RSET flags): `RETE` Flags cell changed to
  `all` with description "Return from exception handler (SR restored from
  the link register)"; `RSET` Flags cell changed to `all` with description
  "System reset (SR cleared to 0)".
- **F-con-14** / **F-sum-25** (lines 13–14 legend, and the `*` cells):
  rewrote the Flags legend to point at Chapter 11's
  `tab:flag-effect-symbols` and to define `--` and `COP` instead of
  overloading `*`. Changed the `*` cells for `COPI`, `COPR`, `DIVS`, `DIVU`,
  `MULS`, `MULU` to `COP`.
- **F-sum-19** / **F-con-14** (SHFT flags, line 79): SHFT's flags come from
  the shifter, not a coprocessor, so its cell is now `S` (Chapter 11's
  shifter symbol) rather than `*` or `COP`. This follows F-con-14's explicit
  instruction to use `S` for this row over F-sum-19's alternative wording
  "NZCV (from the shifter)" — see "Findings I could not apply as written."
- **F-con-31** / **F-sum-18** (reserved SR bits, line 126): Function cell
  changed to "Reserved for future use; software must not rely on the value
  read," matching the advisory wording ruled in Gate 2 and STYLE.md. Reset
  cell kept at `0` (see note below).
- **F-con-44** / **F-sum-27** (WAIT description, line 91): description
  changed to "Idle until an enabled hardware interrupt or a reset occurs,"
  matching the more precise, code-grounded wording used in the accepted
  Chapter 6 finding F-exc-25 for the same fact.
- **F-con-48** (line 169, BAT cross-reference): "(section on External Bus
  Signals)" replaced with the real section title, `Section "Pin
  Descriptions"`.
- **F-sum-1** (NZ flags, applied here per its "Same defect in
  appendix-e-quick-reference.tex" note): changed the Flags cell from `NZ` to
  `NZ (C,V=0)` for ANDI, ANDR, ORRI, ORRR, TSAI, TSAR, TSXI, TSXR, XORI,
  XORR (lines 28/29, 68/69, 85–88, 94/95 pre-edit).
- **F-sum-2** (NOOP encoding, applied here per its "Same defect... :65"
  note): `ADDI[N] r1,#0` corrected to `ADDI[N] sr,#0`. Per the finding's own
  ruling and Gate 2 ruling I, no protected-mode note was added.
- **F-sum-26** (LOAD row, line 58): Type changed to `Mem/ALU`; description
  changed to "Load from memory (0x14–0x17) or move an immediate or register
  value (0x07, 0x37)" since 0x07/0x37 never assert the bus.
- **F-sum-28** (status-register paragraph, lines 107–108): "upper byte"
  corrected to "high byte" (also satisfies the manual-wide
  upper→high mechanical pass for this file); the write-privilege rule was
  corrected to attach to any instruction naming `sr` as destination, not
  just to writes that touch the high byte.
- **2422-line mechanical pass** ("co-processor" → "coprocessor"): fixed the
  two BAT-table occurrences ("DMA Co-processor Read/Write Burst").
- **G1-T4-ap**: Example numbering — not applicable; this appendix contains
  no `Example:` blocks.
- **Gate 1 hex-literal macro pass**: wrapped literal opcode-byte values in
  the Mnemonic Index table's Opcode(s) column with `\opcode{XX}` (single
  values, slash-separated lists, and en-dash ranges).

## Findings I could not apply as written

- **F-con-31 vs. F-sum-18** (reserved SR bits Reset cell): F-con-31 asked
  for the Reset cell to become `--`; F-sum-18 asked for it to stay `0`.
  Kept `0` for consistency with every other row in the same table (all of
  which show a concrete reset value of 0) and because the whole status
  register clears to 0 on reset; only the Function-cell wording (identical
  in both findings) was changed.
- **F-con-14 vs. F-sum-19** (SHFT Flags cell): F-con-14 said change the
  cell to `S`; F-sum-19 said change it to `NZCV (from the shifter)`. Used
  `S`, per the explicit instruction to align this appendix's flag notation
  exactly with Chapter 11's symbol set (`*`, `0`, `1`, `-`, `S`, `U`).
- **F-con-44 vs. F-sum-27** (WAIT description): F-con-44 said "Wait until
  an exception occurs"; F-sum-27 said "Idle until an enabled hardware
  exception or a reset occurs." Used the more precise, code-grounded
  wording ("Idle until an enabled hardware interrupt or a reset occurs"),
  matching the wording independently accepted for the same fact in Chapter
  6 (F-exc-25), for cross-chapter consistency.
- **COPI-immediate opcode cells** (e.g. `COPI #0x1900`, `COPI #0x1Cxy`):
  left unwrapped by `\opcode`/`\imm`, matching the established treatment of
  identical cells in Chapter 16's "Assembles to" fields and Chapter 17's
  "Lowers To" column (both already edited in this pass and left bare).
  STYLE.md's texttt exception for "a syntax cell where the whole cell is
  already a code example" covers this case; decomposing a `COPI #0x1Cxy`
  cell into a bare opcode plus a hybrid literal/placeholder immediate would
  create a style inconsistent with the sibling chapters.

## Open questions

- None. Every conflict between duplicate/overlapping findings targeting
  this file was resolved above by preferring the more code-grounded or
  more internally consistent wording; none required a technical decision
  only the user could make.

Braces and `\begin`/`\end` counts verified balanced (8/8) after edits.

## Phase 4

### Findings applied

- **F-con2-7** (blocker): converted the Instruction Mnemonic Index Flags column
  to Chapter 11's symbol set. `NZCV` -> `* * * *`; `NZ (C,V=0)` -> `* * 0 0`;
  `--` -> `- - - -`; `S` -> `S S S S` (Chapter 9 confirms all four flags come
  from the shifter when `[S]` updates the SR). Split the single Flags column
  into four columns headed N/Z/C/V, matching Chapters 13-15's per-flag table
  convention, and updated the table's colspec and every `\multicolumn` group
  header from 5 to 8 columns. `COP` (COPI, COPR, DIVS, DIVU, MULS, MULU)
  becomes `- - - -` with a `\dag` marker on the V cell, since Chapter 11's
  `*` specifically means "updated" and these rows are only preserved by the
  call instruction itself (per Chapter 16's per-entry Flags lines); a
  footnote below the table explains the coprocessor may define further
  effects once dispatched. `all` (RETE, RSET) becomes `* * * *` with a
  `\ddag` marker on the V cell plus a footnote naming the restore/clear
  source (link register / `sr` cleared to 0), since these values are fully
  specified rather than coprocessor-dependent. Updated the legend paragraph
  above the table to describe the new column layout and both footnote
  markers.
- **F-con2-36** (nit): filled in the Label column of the "Where to Find
  Other Tables" table for the three rows that do have labels:
  `\ref{tab:opcode-map}` (opcode map), `\ref{tab:addr-modes}` (addressing
  mode summary), `\ref{tab:meta-instruction-cross-reference}`
  (meta-instructions cross-reference). Verified all three labels exist in
  their respective chapter files before inserting. Left the remaining
  `---` rows (exception vector table, instruction timing) unchanged, since
  neither has a labelled table.

### Could not apply as written

- None; both fixes applied as specified.

### Open questions

- None.

`\begin`/`\end` count: 8/8. Brace count balanced (280 open / 280 close, 0 net depth).
