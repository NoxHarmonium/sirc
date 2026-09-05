# Change log: chapters/13-alu-instructions.tex

## Findings applied

- **F-alu-1** — Replaced the fault-exclusion sentence (Common ALU Semantics narrative) and
  the `Exceptions` row of Table~\ref{tab:alu-common-semantics} to state that ALU instruction
  execution does not raise privilege-violation faults *unconditionally*, but does raise one at
  decode when the register field names a privileged register (`sr`, `ah`, `lh`, `ph`, `sh`).
  Added the matching exception clause to the `Privilege:` line of all 12 instruction entries
  (SHFT, ADD, ADC, SUB, SBC, AND, ORR, XOR, LOAD, CMP, TSA, TSX).
- **F-alu-2** — Added the "privilege check is made on the register field even though the
  instruction writes no register" sentence, with the instruction-specific example
  (`CMPI sr, #0` / `TSAI sr, #0` / `TSXI sr, #0`), to the `Privilege:` lines of CMP, TSA, TSX.
- **F-alu-3** — Rewrote the LOAD `Operands:` sentence: dropped the false "no ... shift forms"
  claim; now states the register-to-register form rejects a shift (use SHFT) and cross-refers
  the indirect memory forms to Chapter 14.
- **F-alu-4 / F-con-2** — Fixed the CMP entry's signed/unsigned condition-mnemonic lists to
  `>=, <<, >>, <=` (signed) and `HI, LO, CS, CC` (unsigned), per Gate 2 ruling B (ARM-style
  glosses stand, not inverted).
- **F-alu-5** — Replaced the `ANDI r2, #~(1<<5)` example (not valid assembler syntax) with
  `ANDI r2, #0xFFDF`.
- **F-alu-6** (Gate 2 ruling C) — Added one clarifying sentence after the flag-update
  narrative: the public LOAD syntax always encodes AF=None (preserves flags); an AF encoding
  other than None on this opcode, unreachable from assembly syntax, leaves all four flags
  architecturally undefined (`U U U U`). The existing `- - - -` / default `[N]` row in
  Table~\ref{tab:alu-status-flag-effects} already matched the ruling and was left as-is.
- **F-alu-7** — Added the sentence explaining that the immediate format has no shift fields,
  so `[S]` on an immediate-format instruction clears all four flags; restricted `[S]` to
  short-immediate and register forms.
- **F-alu-8** (Gate 2 ruling E) — Replaced the vague "same shift field limits" SHFT note with
  an explicit statement: immediate counts 0–15, register counts above 15 architecturally
  undefined. Did not document the clamp/modulo behaviour, per the ruling.
- **F-alu-9** (Gate 2 ruling Q — apply as proposed) — Added an `ADDR rD, rS2[, shift]` row to
  Table~\ref{tab:alu-legal-forms} and extended the register-shorthand note with the
  shift-applies-to-the-left-operand example (`ADDR r1, r2, LSL #2` = `(r1 << 2) + r2`).
- **F-alu-10 / F-con-51** — Fixed the ORR "set bit N" example: `ADDI r4, #1` → `LOAD r4, #1`.
- **F-alu-11** — Fixed the TSX example: replaced the non-masking `TSAI r3, #0xFF` /
  `TSXI r3, #0xAA` pair with `ANDI r3, #0x00FF` / `TSXI r3, #0x00AA`.
- **F-alu-12** — Rewrote the LOAD Description/Operation to remove the "implemented as ADD
  with zero operand" claim; states the ALU passes operand2 straight through and operand1 is
  ignored.
- **F-alu-14** — Replaced "reserved or architecturally undefined ... not required to raise"
  with "every opcode decodes; none raises invalid-opcode; undocumented encodings execute with
  architecturally undefined results," referencing Appendix~\ref{appendix:undocumented}.
- **F-alu-15** — "single-cycle operations" → "single-instruction operations" in the Summary.
- **F-alu-16** — Added the fourth AF-encoding bullet (`11` reserved, status register
  unchanged) to the Manual Flag Update Control list.
- **F-alu-17** — Added NUL to the SHFT shift-type list, with the `SHFT rD, NUL #0` note.
- **F-alu-18** — Converted every `(rX << shift)` pseudo-comment in a Syntax block to
  `shift(rX)`, across all ten register/short-immediate instruction families (ADD, ADC, SUB,
  SBC, AND, ORR, XOR, CMP, TSA, TSX).
- **F-con-16** — The chapter 13 LOAD row (`- - - -`, default `[N]`) already matches Gate 2
  ruling C and needed no change on this side. See "Could not apply as written" below — the
  finding's actual fix target is `chapters/11-reading-instructions.tex:97`, outside this file.
- **G1-T1-13** — Renamed every `Condition codes:` field label to `Condition field:` (12
  instances, one per instruction entry).
- **G1-T2-13** — Added `\textbf{Applicability:} SIRC-1, all revisions` under every entry
  title (12 entries).
- **G1-T3-13** — Added an `Instruction Fields:` list after the opcode/assembles-to line of
  every entry, giving legal values for the register, immediate, shift, AF, and condition
  fields, drawn from `review/facts.md` and Chapter 7 only.
- **G1-T4-13** — Numbered all 13 `Example:`/`Examples:` blocks in the chapter sequentially as
  `Example 13-1:` through `Example 13-13:` (including the one standalone example in "Manual
  Flag Update Control", which is not inside an instruction box but is otherwise the same kind
  of block).

## Findings skipped or adapted

- **F-alu-13** — status `rejected` (superseded by Gate 2 ruling H: opcode 0x27 stays
  "undocumented"). Left the existing wording ("Opcode 0x27 has no public assembly syntax and
  is undocumented") untouched in all three locations.
- **F-con-16** — status `accepted`, but its `fix` text targets
  `chapters/11-reading-instructions.tex:97`, a file this editor does not own. No edit was made
  to Chapter 11. Confirmed Chapter 13's own LOAD row/text already agree with the ruling, so no
  contradiction originates on this side; flagged in Open Questions for the Chapter 11 editor.
- **F-alu-9**'s fix, taken literally, adds only one new legal-form row (`ADDR rD, rS2[,
  shift]`). The same shorthand-plus-shift capability applies to ADC/SUB/SBC/AND/ORR/XOR and to
  the CMP/TSA/TSX two-operand register forms, but the finding did not ask for those rows and
  none were added, to avoid inventing scope beyond the accepted finding. See Open Questions.

## Open questions

1. Should `Table~\ref{tab:alu-legal-forms}` gain the equivalent `MNEMR rD, rS2[, shift]` /
   `MNEMR rS1, rS2[, shift]` shorthand+shift rows for ADC, SUB, SBC, AND, ORR, XOR, CMP, TSA,
   and TSX, matching the one row F-alu-9 added for ADD? The underlying encoding behaviour is
   identical across all register-form ALU ops.
2. F-con-16 is accepted but its fix text edits `chapters/11-reading-instructions.tex:97`,
   which belongs to another chapter's editor. Chapter 13's own wording already conforms to
   Gate 2 ruling C and does not need to change on this side; please route the Chapter 11 edit
   to that file's owner.

## Summary

Applied all 21 accepted chapter-13 findings (skipping the one rejected finding, F-alu-13) and
the four Gate-1 template rules across all 12 instruction entries: renamed the predicated-
execution field, added an applicability line and an Instruction Fields list to every entry,
and numbered all 13 examples sequentially as `Example 13-N`.

## Phase 4

### Findings applied
- F-con2-29 (minor): applied `\opcode{}` to all bare `0xNN` opcode literals in the Encoding
  column of the Legal ALU Instruction Forms table (33 cells, lines ~57-89) and on every
  `\textbf{Opcodes:}` line in the instruction entries (11 lines, including the inline
  "Opcode 0x27 is undocumented" clause on the LOAD entry's Opcodes line).

### Findings not applied as written
- F-con2-16 (major): the finding's mechanical fix only touches
  `chapters/07-instruction-formats.tex` (Table~\ref{tab:shift-types}) and
  `chapters/09-shift-operations.tex` (Table~\ref{tab:shift-encoding} plus a new definition
  sentence) — neither file is owned by this pass. Checked chapters/13-alu-instructions.tex
  against the finding's own `claim` quotes ("Shift type and count: NUL, LSL, ..." and "All six
  shift types are supported, plus NUL ... SHFT rD, NUL #0 sets the flags from the unshifted
  value") and confirmed this chapter already documents `NUL` correctly and consistently
  (12 occurrences, all in Instruction Fields lists and the SHFT entry's Description). No edit
  was needed or made in this file for F-con2-16; the outstanding work is entirely in Chapters
  7 and 9, out of scope here.

### Open questions
- None.
