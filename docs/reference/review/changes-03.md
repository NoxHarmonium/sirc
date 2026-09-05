# Changes — chapters/03-registers.tex

## Findings applied

All findings in `triaged.md` with `file: chapters/03-registers.tex` and `status: accepted`:

- **F-arch-3** — Register Encoding table: the `sr` row's "Privileged" cell said "Upper byte
  only," which is true for reads only. Changed to "Reads: high byte redacted. Writes: fully
  privileged" (wording adapted to house "high"/"low" terminology per F-con-64), and added a
  sentence after the table noting that any instruction whose destination field encodes
  `0x0` is therefore privileged in protected mode, even if the operation would only alter the
  unprivileged low byte.
- **F-arch-4** — Fixed the broken push/pop example: `STOR -(#2, s), r1` (which does not pair
  correctly with `LOAD r2, (s)+`) replaced with the zero-displacement form `STOR -(s), r1`.
- **F-arch-13** — Added a sentence to the Overview stating post-reset register state: only
  `sr` (cleared to zero) and the `p` pair (loaded from the reset vector) are defined after
  reset; all other registers are architecturally undefined, with a cross-reference to
  Chapter 6 (Exceptions).
- **F-arch-14** — The unconditional "Direct writes to sr raise a privilege violation fault"
  is qualified: that sentence now applies to protected mode only, and a new sentence states
  that in supervisor mode a direct write to `sr` is permitted except that it cannot change
  the Exception Active (EA) bit.
- **F-arch-15** — The "Usage Conventions" subsection prescribed a caller/callee-saved
  register split that contradicts Chapter 1's statement that calling conventions are out of
  scope. Replaced the subsection with a statement that the CPU enforces no register-usage
  convention and a pointer to the SIRC-1 ABI and Toolchain documentation (chosen over editing
  Chapter 1's scope statement, since this editor does not own that chapter).
- **F-con-64** — "Upper"/"Lower" byte and word terminology standardized to "high"/"low"
  throughout the chapter (heading labels in the Status Register section, the address-pair
  Overview paragraph, and the register-encoding table, per F-arch-3 above).
- **G1-T4-03** — Numbered the three code examples in the "Usage Examples" section as
  `Example 3-1`, `Example 3-2`, and `Example 3-3`, per the Gate 1 numbering decision.

## Findings skipped (and why)

None of the file's findings had status `defer`, `code-wrong`, `not-implemented`, `phase4`, or
`rejected` — all seven findings scoped to this chapter were `accepted` and applied above.

## Mechanical passes applied

- Wrapped all sixteen bare hex register-ID literals (`0x0`–`0xF`) in the Register Encoding
  table with `\texttt{}`, per the manual-wide hex-literal pass (Gate 1 Approved Decisions;
  these are field-position values, not opcodes/immediates/addresses, so `\texttt{}` is the
  correct macro).
- Hyphenated "general-purpose" as a compound adjective before a noun (section heading,
  Overview bullet, table cells) per STYLE.md §1. Left the one predicate-adjective use
  ("all seven registers are equally general purpose") unhyphenated, since it does not precede
  a noun.
- No `e.g.` instances were present in this chapter.

## Copy edits (grammar / clarity / STYLE conformance)

- Removed vague filler ("that serve various purposes," "it is worth noting that," "In fact,"
  "simply") and a stray second-person "you" (STYLE.md §5 flags surviving "you" as a defect).
- Converted the one future-tense normative statement ("will be updated automatically") to
  present tense per STYLE.md's voice/tense rule, and split the compound sentence it was part
  of into two single-fact sentences.
- Changed "can be used freely" / "can be used as source or destination operands" to "may," per
  the normative-vocabulary table (permission, not requirement).
- Wrapped the previously bare mnemonics "STOR/LOAD" in `\mnemonic{}`, matching the macro
  convention used for every other mnemonic in the chapter.
- Fixed a sentence fragment ("A general-purpose address register pair used for...") to a full
  sentence for parallelism with the other three register-pair subsections.
- Fixed "cannot be modified directly from protected mode writes" (garbled) to "direct writes
  in protected mode cannot modify it."
- Added terminal periods to three list items in "High Address Register Restrictions" that are
  full sentences but lacked them, for consistency with the full-sentence list items later in
  the same section (STYLE.md §5, lists).
- Minor wording: "is considered privileged" → "is privileged" (more direct); "segmentation/
  protection" → "segmentation and protection" (avoid slash construction in prose).

## Open questions

- The Register Encoding table's `sr` row now holds a full sentence ("Reads: high byte
  redacted. Writes: fully privileged") in a centered (`c`) column alongside single-word "Yes"/
  "No" entries elsewhere in the same column. This is content-correct per F-arch-3 but may
  render as an overly wide table cell; a table-layout pass (e.g., a `p{}` column) may be
  wanted at typeset time. Left as prose text and did not change the column type, since that is
  a formatting decision outside this pass's scope.
- F-arch-15's fix offered two options: delete the Chapter 3 subsection, or narrow Chapter 1's
  scope statement. Chose deletion since this editor owns only Chapter 3; flagging in case the
  user prefers the Chapter 1 editor to instead narrow the scope sentence there (in which case
  this chapter's replacement text may want revisiting for redundancy).

## Phase 4

Continuity re-check findings applied.

### Findings applied
- F-con2-1 (blocker): Replaced the two-increment program-counter description in
  "Program Counter (p)" with the settled model — the \reg{p} pair points at the instruction
  being fetched, is unmodified during either fetch phase, and \reg{pl} advances by 2 once
  during Decode and Register Fetch (\reg{ph} never increments, so a wrap stays inside the
  segment). Applied the fix text verbatim.
- F-con2-39 (nit): Replaced "accessible in both supervisor and protected modes" in the
  Status Register overview's Low Byte bullet with "readable in both modes and updated as a
  side effect of ALU and shift instructions in both modes; direct writes to \reg{sr} are
  privileged," matching Chapter 5's corrected wording and the sentence a few lines below it.
  Applied the fix text verbatim.

### Findings not applied as written
- None; both fixes applied as given.

### Other contradictions checked
- Line 85 ("The program counter pair increments after every instruction.") was reviewed
  against the settled model. It does not assert a specific increment count or timing that
  contradicts the settled model (it is compatible with a single two-word advance), so it was
  left unchanged to keep this pass scoped to the two findings.

### Open questions
- None.
