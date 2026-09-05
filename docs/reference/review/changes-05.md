# Change log: chapters/05-status-register.tex

## Findings applied

All 14 findings triaged for this chapter are `status: accepted` and were applied:

- **F-data-2** (line 10, lower-byte privilege claim) — replaced with the corrected
  readable/updated-as-side-effect/privileged-direct-write wording.
- **F-con-11** (lines 45-46, condition-flag privilege contradiction) — reworded to match
  F-data-2; added a cross-reference to the "Manual Updates" subsection (which needed a new
  `\label{sec:sr-manual-updates}` to support the `Section~\ref{}` form required by STYLE.md).
- **F-data-4** / **F-con-43** (line 299, `ADDI r1, r2, r3`) — both findings target the same
  line with the same fix; changed to `ADDR r1, r2, r3` once.
- **F-data-12** (LOAD flags) — applied per Gate 2 ruling C rather than the finding's own
  suggested wording: LOAD is stated to never update flags (row `- - - -`, assembler always
  emits status source `[N]`), and an explicit AF suffix on LOAD is stated to leave all four
  flags architecturally undefined (`U U U U`). LDEA is stated separately as never updating
  flags.
- **F-data-14** (Trap on Address Overflow, "When Set" text) — replaced with the
  effective-address/PC-wrap wording naming the fault "segment overflow fault (vector 0x03)",
  consistent with Chapters 4 and 6 and with Gate 2 ruling D (trap fires only on low-word wrap).
- **F-data-7** (Manual Updates section) — added the paragraph explaining that an ALU
  instruction targeting `sr` overwrites bits 7-0 with its own flags, so only bits 15-8 can be
  set this way, and how to avoid it (`[N]` suffix or `LOAD`). Checked the existing listing's
  examples (`ORRI sr, #0x0100`, `#0x1E00`, `ANDI sr, #0xE1FF`) — all only ever target upper-byte
  bits, so they still hold under the corrected rule.
- **F-data-8** (Exception Handling Effects, item 4) — replaced the vague "interrupts may be
  masked" with the concrete rule: hardware interrupt enable bits are unchanged, and a pending
  interrupt is serviced only if its priority exceeds the current exception level.
- **F-data-9** (Reserved Flags) — kept the advisory wording required by STYLE.md's Approved
  decisions ("reserved for future use; software must not rely on them" — tightened the second
  clause from "should not be relied upon" to "must not rely on them" to match the approved
  phrase and normative vocabulary in STYLE.md §2), and added the observable rule that a
  supervisor write can set these bits but any later flag update clears them.
- **F-con-37** / **F-data-16** (line 101, coprocessor privilege bullet) — both findings target
  the same bullet; merged into a single fix using the "operation nibble" terminology from
  Chapters 1/16 for consistency and the bit-position detail from F-data-16: "command-word
  operation nibble `0x8`-`0xF`, bits 11-8".
- **F-data-17** (interrupt latch behavior) — added the level-sensitive latching rule to the
  first "Important behavior" bullet.
- **F-data-20** (EA bit not restored by direct write) — added a note to both restore examples
  (the `LOAD sr, (s)+` example in "Manual Updates" and the `LOAD sr, r7` example in "Example
  5-5: Saving and Restoring Status") stating that a direct write always preserves the current
  EA bit rather than adopting the written one, and that only RETE restores it.
- **G1-T4-05** (numbered example blocks) — see "Findings applied differently" below.

## Findings applied differently than written

- **G1-T4-05**: STYLE.md's `Example N-M:` numbering rule is written for `Example:`/`Examples:`
  fields inside Chapters 13-17 instruction entries. Chapter 5 has no instruction entries; its
  closest analogue is the five `\subsection`s under "Usage Examples". Applied the closest
  faithful version: renamed those subsections to `Example 5-1: Conditional Execution` through
  `Example 5-5: Saving and Restoring Status`, preserving their original descriptive titles.

## Other copy-edit changes (not tied to a triaged finding)

- Removed a stray quoted `"privilege violation" fault` in the Protected Mode bit note; changed
  to the unquoted `privilege violation fault` form used everywhere else in the chapter, and
  changed "will cause" to "raises" to match the manual's normative verb usage.
- Changed "will not be serviced" to "are not serviced" (present tense, per STYLE.md §5).
- Changed the one "it's" contraction to "it is" (contractions are effectively unused elsewhere
  in the manual).
- Changed one `e.g.,` to "for example" (Gate 1 ruling, manual-wide pass).
- No hex literals in running prose needed macro/`\texttt` treatment beyond the coprocessor
  nibble bullet above (`0x27xx`-style literal removed by F-con-37/F-data-16); all other hex
  literals in the chapter are inside `lstlisting` blocks and were left as written per
  STYLE.md.

## Open questions

- None. All triaged findings for this chapter had unambiguous `accepted` resolutions or a
  Gate 2 ruling to follow, and no remaining ambiguity required a technical decision beyond
  the manual's existing text.

## Verification

- `grep -c 'begin{'` and `grep -c 'end{'` both return 23 for this file.
- Brace count (`{` vs `}`) balances at 204/204.
- Full manual (`make quick`) compiles cleanly to `main.pdf` with no new errors introduced by
  this chapter.
