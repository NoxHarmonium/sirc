# Change log: chapters/15-control-flow.tex

## Findings applied

- **F-mem-12** (blocker) — PC-relative displacement base. Corrected the `@label` gloss for
  `BRAN`/`BRSR` from "PC-relative displacement from the next instruction address" to "a word
  displacement from the address of the branch instruction itself," and added the same fact to
  each Description: `BRAN #0` branches to itself, `BRAN #2` falls through, and the `BRSR` link
  value is the branch instruction's own address plus 2.
- **F-mem-13** (major) — `LDEL` never accepts `l` as a destination (aliases the implicit link
  write). Added the rule to the "Address registers" note, and annotated the four `LDEL` rows in
  the Legal Forms table (`dest` excludes `l`).
- **F-mem-14** (major) — Replaced "avoid forms such as ... known hazardous aliased forms" with
  the exhaustive list the toolchain actually enforces (LDEA pre-decrement dest==src; LDEL dest
  `l`; LDEL post-increment src `l` or dest==src; LJSR post-increment from `l`/`p`; LOAD
  post-increment into a half of the auto-updated pair; STOR pre-decrement from a half of the
  auto-updated pair).
- **F-mem-15** (major) — `LJSR a, #function_offset` is not assembler syntax (`#identifier` isn't
  parseable). Changed to `LJSR a, @function_offset` in Example 15-6.
- **F-mem-16** (major) — The jump-table example did not work: the `LDEA` base is its own
  address, not the following word, so index 0 looped back to the `LDEA`. Rewrote the example to
  scale the index for a 2-word `BRAN` entry (`ADDI r2, #2, LSL #1`) and to skip over the `LDEA`,
  with the table holding `BRAN` instructions.
- **F-mem-17** / **F-con-57** (major/minor) — `.DW`/`DW` is undefined manual-wide and
  `case0 - jump_table` isn't parseable (no symbol arithmetic). Removed the `DW` lines; the
  rewritten table (see F-mem-16) holds real `BRAN` instructions instead of data words.
- **F-mem-18** (major) — Not applied as a text change; see Open questions.
- **F-mem-19** (minor) — Stated the signedness and wrap explicitly for `#disp` in both `BRAN`
  and `BRSR`: signed 16-bit word displacement (-32768 to +32767), added modulo $2^{16}$, wraps
  within the segment.
- **F-mem-20** (minor) — Added `; supervisor mode only -- writing ah is privileged` to the
  `LOAD ah, #0x0040` line in Example 15-6, since the chapter's own privilege rule (direct writes
  to high address registers are privileged) otherwise contradicts the example under protected
  mode.
- **F-con-54** (minor) — Changed "post-increment source registers cannot be l or p" to "must not
  be l or p; such forms are aliased address-register writes with architecturally undefined
  behavior and are rejected by the assembler," with a cross-reference to Appendix C, matching
  the classification already used at lines 71-77 and in Appendix C.
- **G1-T1-15** — Renamed every `Condition codes:` field label to `Condition field:` (7
  instances: BRAN, BRSR, RETS, LDEA, LDEL, LJMP, LJSR).
- **G1-T2-15** — Added `\textbf{Applicability:} SIRC-1, all revisions` immediately under each of
  the 7 instruction-entry titles.
- **G1-T3-15** — Added an `Instruction Fields` list after the opcode/`Assembles to` line in each
  of the 7 entries (register field, AF, immediate width, r3/`rS` field where applicable,
  condition field), using values from `facts.md` and Chapter 7's format tables. Followed the
  template already established in `13-alu-instructions.tex` for label wording and structure.
- **G1-T4-15** — Numbered the six `Examples:` blocks `Example 15-1` through `Example 15-6` in
  order of appearance (BRAN, BRSR, RETS, LDEA, LDEL, LJSR); LJMP has no example block.

## Findings not applied as written / skipped

- **F-mem-18** — The fix text itself says "present both sides to the author; do not publish
  either wording until it is settled." I left the Common Semantics `Privilege` row and the LDEA
  `Privilege` field unchanged and logged this as an open question instead of picking a wording.
- **F-con-8** — `file:` is `chapters/14-memory-instructions.tex` (the `rD`/`rS`/`rO` naming
  finding), even though its evidence lists `rS` occurrences in this chapter. Left this chapter's
  `rS` naming untouched; renaming is that chapter's editor's responsibility, and touching it here
  would risk an inconsistent partial rename if the other editor's fix differs in scope.
- All other findings whose `file` field was a different chapter (F-mem-5, F-sum-9, F-con-23,
  F-con-41, F-con-68, F-con-59, F-con-42) were left untouched, per scope.

## Open questions

- **F-mem-18**: as implemented, any `LDEA` whose destination pair is `l` (register field 0)
  raises a privilege-violation fault in protected mode before write-back, regardless of the high
  words — this is a rule the chapter does not currently state, and no test exercises
  `LDEA l, (...)` in protected mode. Either the decode-time privilege check needs to exclude the
  effective-address opcode class (0x18-0x1F), in which case the current chapter text is correct
  and the emulator has a bug, or the manual needs an added sentence "`LDEA` with destination `l`
  is privileged." I did not guess; the Common Semantics `Privilege` row and the LDEA entry's
  `Privilege` field are unchanged pending a decision.

## Phase 4

Continuity re-check follow-ups from `consistency-2.md`.

### Findings applied

- **F-con2-17** (major, chapter-15 half) — Renamed the displacement-register operand `rS` to
  `rO` everywhere it names the register supplying a register displacement in the `LDEA`,
  `LDEL`, `LJMP`, and `LJSR` entries (Legal Forms table, Instruction Fields, Syntax,
  Operands, and the matching `lstlisting` comments) — 23 sites total. Every `rS` occurrence
  in this chapter was that same displacement-register role, not a "source register" use, so
  no site was left as `rS`. This reverses the earlier Phase-3 decision (logged under F-con-8
  in this file) to leave the rename to another chapter's editor, per this Phase 4 task's
  explicit instruction and the `rO` row now defined in Chapter 11's notation table.
- **F-con2-35** (minor) — Capitalized `\textbf{r3 (bits 17--14` to `\textbf{R3 (bits 17--14`
  at all four sites (the `Instruction Fields` bullets in `LDEA`, `LDEL`, `LJMP`, `LJSR`),
  since these label the abstract field position, not the concrete register `\reg{r3}`.
- **F-con2-32** (minor) — Added `Example 15-7:` to the `LJMP` entry, the only chapter-15
  instruction lacking one, continuing the chapter's sequence (BRAN=15-1 ... LJSR=15-6). The
  example shows the plain, `#offset`, and register-displacement `LJMP` forms. Because `LJMP`
  appears earlier in the chapter than `LJSR` (whose example is already numbered 15-6), the
  example numbers are not monotonic in document order; the finding's fix specifies the
  number to use, and renumbering `LJSR`'s existing example was outside this finding's scope.

### Findings not applied as written / skipped

- None. All three findings applied as specified above.

### Open questions

- None new. The LJMP/LJSR example-numbering order note above (15-7 preceding 15-6 in the
  document) is worth flagging to the user if a strictly ascending example sequence is wanted;
  fixing it would require renumbering `LJSR`'s Example 15-6, which is outside this task's
  three findings.
