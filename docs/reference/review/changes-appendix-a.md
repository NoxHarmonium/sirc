# Changes — chapters/appendix-a-opcode-map.tex

## Context

This file had already been partially edited by an interrupted run (112 lines added, 105
removed, uncommitted). I ran `git diff` first, checked each finding against the current
file state, and did not re-apply anything already present. No further edits to the file
were needed; all in-scope findings were already applied correctly. Verified: `grep -c
'begin{'` = 9, `grep -c 'end{'` = 9 (balanced), and open/close brace counts are equal
(205/205).

## Findings applied (already done by the interrupted run; verified correct, left as-is)

- **F-sum-6** (line ~126, "Bit 1" auto-update claim) — replaced with "Bit 1: Auto-update of
  the source address register (0 = no update, 1 = pre-decrement for STOR/LDEA, post-increment
  for LOAD/LDEL). Only the low word of the pair is adjusted, and always by one word regardless
  of the displacement." Matches the finding's fix text verbatim.
- **F-con-21** (same line, same defect) — this finding targets the same bullet as F-sum-6 with
  slightly different wording ("Auto-update form (0 = no address-register update, 1 =
  pre-decrement for STOR/LDEA or post-increment for LOAD/LDEL; only the low word of the pair
  changes)"). The two accepted findings conflict word-for-word on the same line; the applied
  F-sum-6 wording already satisfies F-con-21's underlying fact claim (same auto-update
  semantics, same exclusion of the high word), so no further edit was made. Treating both as
  applied via one wording rather than picking a second, redundant edit.
- **F-sum-16** (Format Identification bit-numbering ambiguity) — a scoping sentence, "Bit
  numbers in this section refer to the 6-bit opcode value (instruction bits 31--26), not to
  the instruction word," was added under `\subsection{Format Identification}`, and each bullet
  reworded to "Opcode bits 5--4 = ...".
- **F-sum-17** (Memory/Control format-selection gap) — bullet reworded to "Opcode bits 5--4 =
  01: Memory and control operations (\opcode{10}--\opcode{1F}). Format is selected by opcode
  bit 0: even opcodes use Immediate format, odd opcodes use Register format."
- **F-sum-24** (terminology: "implementation-defined" for undocumented opcodes) — replaced
  with "These opcodes execute in hardware, but their behavior is architecturally undefined and
  may change between CPU revisions," matching STYLE.md's terminology ruling and Gate 2 ruling H.
- **F-sum-33** (booktabs conversion) — table converted from `{|c|l|l|l|}` + `\hline` to
  `{clll}` + `\toprule`/`\midrule`/`\bottomrule`, no vertical rules; `\label{tab:opcode-map}`
  and `\label{tab:instruction-variant-quick-reference}` added to both tables in the file.
- **F-con-72 (appendix A part)** — same booktabs conversion as F-sum-33 above; this is the
  appendix-A half of a finding whose primary `file:` is `02-cpu-architecture.tex`, and whose
  ruling explicitly assigns the appendix-A part to this file's editor. Confirmed no `\hline`
  or vertical-rule (`{|`) remnants remain anywhere in the file.
- **F-con-10 bullet** — a fourth bullet, "\item \opcode{27}, \opcode{2F} (no public assembly
  syntax)", was added to the Undocumented Instructions list, per Gate 2 ruling H and the
  launch instructions calling this out explicitly. F-con-10's primary `file:` is
  `12-instruction-summary.tex`; only the appendix-A bullet addition was in scope here.
- **Gate 2 ruling H terminology** — opcodes \opcode{27} and \opcode{2F} are marked
  "Undocumented" in the main table (they already were) and are included in the Undocumented
  Instructions bullet list; Appendix C's count of 14 is untouched (out of scope — different
  file).

## Findings I could not apply, or applied only in part

- **G1-T4-ap** (Example numbering template) — not applicable: this file contains no
  `Example:` blocks to number.
- **F-sum-14** and **F-sum-5** — both have `file: chapters/12-instruction-summary.tex` as
  their primary location, but their `fix:` text also asks for edits inside this appendix
  (F-sum-14: rows/bullets at appendix-a lines 62, 70, 134--138, 159, 163, 169; F-sum-5: the
  "Bit 3 = 1" bullet, to exclude \opcode{0F}/\opcode{2F}/\opcode{3F}). Per my launch
  instructions, I applied only the findings whose `file:` field is this chapter, plus the two
  items explicitly named (F-con-10's bullet, F-con-72's appendix-A table). Since neither
  F-sum-14 nor F-sum-5 was named explicitly, I left both untouched here:
  - The "Bit 3 = 1: Test only, discard result (\opcode{08}--\opcode{0F}, \opcode{28}--\opcode{2F},
    \opcode{38}--\opcode{3F})" bullet (line 115) still includes \opcode{0F}/\opcode{2F}/\opcode{3F}
    in the "test only" set, even though those are coprocessor-call opcodes (COPI/COPR) that
    write neither a result nor flags. This is a real defect but is owned by the 12-instruction-
    summary.tex finding; flagging it here so it isn't missed.
  - The main table's mnemonic column keeps `LOAD`/`COPI` for \opcode{27}/\opcode{2F} rather than
    `--`. I left this as-is deliberately: unlike \opcode{08} etc. (which have no defined
    operation at all), \opcode{27} and \opcode{2F} do have a defined internal operation
    (LoadRegisterFromShortImmediate / CoprocessorCallShortImmediate per the evidence in
    F-sum-14) — they simply have no assembler syntax, a distinction the Quick Lookup Table's
    footnote already preserves. Changing the mnemonic to `--` would erase that true
    distinction, so I did not make this change without an explicit instruction to do so.

## Open questions

- Should the "Bit 3 = 1: Test only, discard result" bullet (line 115) be corrected here to
  exclude \opcode{0F}/\opcode{2F}/\opcode{3F} (coprocessor calls, which discard both result and
  flags rather than "testing"), matching the fix already asked for at
  `chapters/12-instruction-summary.tex` (F-sum-5)? I left it alone because F-sum-5's `file:`
  field points at chapter 12, but the identical defect lives in this appendix too and nobody
  else has been told to fix this half.
- Should the main opcode-map table's mnemonic column show `--` instead of `LOAD`/`COPI` for
  \opcode{27}/\opcode{2F}, to make the row visually match the other "same class" undocumented
  rows per Gate 2 ruling H? I preserved the existing `LOAD`/`COPI` values because they reflect
  a real, tested internal operation and the distinction ("no public assembly syntax" vs. "no
  defined operation") is already captured by the Quick Lookup Table's footnote — but this is a
  judgment call the author may want to confirm.

## Summary

No new edits were required: the interrupted run had already applied every accepted finding
in scope for this chapter, including the booktabs table conversion, the terminology fix, and
the F-con-10 bullet. Verified brace/environment balance and left two cross-file consistency
gaps (owned by chapter 12's findings) as open questions rather than guessing at them.

## Phase 4

Applied the three assigned continuity findings from `consistency-2.md`:

- **F-con2-3** (blocker): Rewrote the "Save vs. Test" bullet list (`\subsection{Save vs.
  Test}`) to state the settled split: bit 3 = 0 with low nibble 0--6 saves the result
  (`\opcode{00}--\opcode{06}`, `\opcode{20}--\opcode{26}`, `\opcode{30}--\opcode{36}`); bit 3 =
  1 with low nibble 8--E is test-only (`\opcode{08}--\opcode{0E}`, `\opcode{28}--\opcode{2E}`,
  `\opcode{38}--\opcode{3E}`); low nibble F is the coprocessor call, not a test form; low
  nibble 7 is LOAD; and `\opcode{27}`/`\opcode{2F}` are undocumented. This also resolves the
  open question left in the prior Phase 3 note about the bullet re-asserting the pre-Gate-2
  "add 0x8" pattern.
- **F-con2-10** (major): Set the Mnemonic column to `--` for `\opcode{27}` and `\opcode{2F}` in
  the main opcode table, moving the LOAD-like/COP-like description into the Description column
  ("Undocumented (LOAD-like)" / "Undocumented (COP-like)"), matching
  Table~\ref{tab:complete-instruction-set}. This also resolves the second open question from
  the prior Phase 3 note (whether the table should show `--` here).
- **F-con2-37** (nit): Changed `OR` to `ORR` in the Quick Lookup Table's Operation column to
  match the `\mnemonic{ORR}`/`ORRI`/`ORRR` family name used everywhere else.

Related to F-con2-9 (filed against Appendix C, not mine to fix there): narrowed the blanket
sentence after the "Undocumented Instructions" bullet list from "These opcodes execute in
hardware, but their behavior is architecturally undefined..." to "The twelve test-variant
opcodes above execute in hardware, but their behavior is architecturally undefined...", and
pointed the cross-reference to Appendix C for all fourteen, including `\opcode{27}` and
`\opcode{2F}`. This removes Appendix A's implicit claim that `\opcode{27}`/`\opcode{2F}` are
"architecturally undefined," leaving their behavioral classification to Appendix C per Gate 2
ruling H (count stays 14); no other file was touched.

### Findings applied
F-con2-3, F-con2-10, F-con2-37 (all applied as written, no deviation needed).

### Findings not applied as written
None — all three fixes applied verbatim.

### Open questions
None new. The two open questions logged in the Phase 3 section of this file are resolved by
the F-con2-3/F-con2-10 fixes above and can be considered closed.

Verified `\begin`/`\end` counts match (9/9) after the edits.
