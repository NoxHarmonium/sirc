# Changes: appendix-c-undocumented.tex

## Findings applied

- **F-exc-4** — Corrected 0x0B/0x2B/0x3B from "SUB-like" to "SBC-like" in the opcode table
  (Pattern column) and moved the SBC pseudocode/utility text into the `0x0B, 0x2B, 0x3B (SBC
  Test)` subsection. The old "Probably identical to CMP ... Utility: None" body (which
  described the wrong opcode family) was replaced by the corrected SBC-Test content. Also
  corrected the "Future Standardization" closing sentence, which used to read "Others (SUB
  Test, which duplicates CMP)"; there is no SUB-Test family among the undocumented opcodes, so
  it now reads "Others (the ADD Test and OR Test variants)".
- **F-exc-5** — Corrected 0x0D/0x2D/0x3D from "SBC-like" to "OR-like" in the opcode table and
  rewrote the `0x0D, 0x2D, 0x3D` subsection as "(OR Test)" with OR semantics (result discarded,
  N/Z from `a | b`, C and V cleared). Updated the Future Standardization bullet that used to
  offer 0x0D/0x2D/0x3D as the SBC-test candidate; it now correctly names 0x0B/0x2B/0x3B.
- **F-exc-6** — Replaced the SBC pseudocode `result = operand1 - operand2 - (1 - C)` with
  `result = operand1 - operand2 - C  ; C is the borrow input`, matching the borrow convention
  used in Chapter 13 and the ALU. This block now lives under the 0x0B/0x2B/0x3B subsection
  per F-exc-4.
- **F-con-30** and **F-exc-21** (reconciled — see below) — Replaced "Reserved address register
  encoding could enable more address modes" (false; all four 2-bit address-register encodings
  are assigned) with "The reserved status-update source encoding (AF = 11) could select a new
  flag-update source".
- **F-exc-19** (per Gate 2 ruling H) — 0x27 and 0x2F stay in the appendix and the count of 14
  stands. Qualified the Overview's blanket "their architectural behaviour is undefined" so it
  no longer misdescribes 0x27/0x2F, which have fully defined, tested behaviour and are listed
  here only because they have no public assembly syntax.
- **F-exc-20** — Removed the speculative framing ("likely", "Probably", "Hypothetical
  behavior", "may or may not be implemented correctly") from the section formerly titled
  "Likely Behavior" (renamed "Observed Behavior") and its four subsections, replacing it with
  a statement of the reference implementation's observed, tested behaviour plus the standing
  caution that it is architecturally undefined and must not be relied on.
- **F-exc-26** and **F-con-47** (reconciled — see below) — Changed "Reserved shift type
  (0x111)" to "Reserved shift type (111)"; the reserved code is the 3-bit binary value 111,
  not the hex literal 0x111 (=273).
- **F-exc-27** — Dropped the stale simulator version pin; "In the current SIRC-VM simulator
  (as of version 1.0):" is now "In the current SIRC-VM simulator:".
- **G1-T4-ap** — Checked for `Example:` blocks to number per the `Example N-M:` convention;
  this appendix contains none, so there was nothing to apply.

## Findings reconciled (two accepted findings targeted the same text with different fixes)

- **F-con-30 vs. F-exc-21** (both target the same bullet, line ~213, "Reserved address
  register encoding..."): F-exc-21's fix was to delete the bullet outright, optionally
  substituting the AF=11 spare encoding; F-con-30's fix was to replace the bullet's content
  with the AF=11 wording directly. I applied F-con-30's replacement text, which also satisfies
  F-exc-21's core requirement (the false claim about a spare address-register encoding is
  gone) and its suggested alternative (AF=11 is exactly what now appears).
- **F-exc-26 vs. F-con-47** (both target the same bullet, "Reserved shift type (0x111)..."):
  F-exc-26's fix text was "Reserved shift type (encoding 111) could enable additional shift
  modes"; F-con-47's fix text was "Reserved shift type (111) could enable additional shift
  modes". Both agree on the underlying defect (the `0x` prefix on a binary field value). I
  applied F-con-47's shorter wording, which also satisfies F-exc-26.

## Findings skipped

None. Every finding in `triaged.md` with `file: chapters/appendix-c-undocumented.tex` had
`status: accepted`; none were `rejected`, `defer`, or otherwise out of scope.

## Other STYLE.md conformance fixes (copy-edit pass)

- **Voice/tense ("you"):** STYLE.md ("no second person") flags four surviving instances of
  "you"/"your" in this file (also called out via `F-con-70`, filed against
  `01-introduction.tex` but listing these four sites). Rewrote all four in third person: "For
  Hardware Implementers" ("You may implement..." -> "These opcodes may be implemented...")
  and "For Researchers/Hackers" (the lead-in sentence and two bullets using "your").
- **Hex-literal macro pass:** Converted every opcode-byte hex literal in this file (table
  cells, subsection headings, "Future Standardization" bullets) to `\opcode{XX}`, and the
  non-opcode hex literal `0x8`/`0x7` in the Pattern column and the "Observed Behavior" prose
  to `\texttt{0x...}`, per STYLE.md's Approved Decisions.
- No `e.g.` or `i.e.` were present. No `Example:` blocks were present, so the numbering
  convention did not apply. No "implementation-defined" language was used for the undocumented
  opcodes' behavior; "architecturally undefined" is used throughout, per STYLE.md.

## Open questions

- None requiring author input. The two same-target finding pairs above (F-con-30/F-exc-21 and
  F-exc-26/F-con-47) were resolved editorially because both members of each pair agreed on the
  underlying fact and differed only in presentation, not in substance.

## Verification

- `grep -c 'begin{'` and `grep -c 'end{'` both report 16.
- Brace count balances (126 `{` / 126 `}`).
- Rebuilt the full manual (`make all`) and visually inspected the rendered Appendix C pages
  (199-203); the opcode-colored `\opcode{}` macro renders correctly inside `\subsection`
  headings and table cells, and the table, Overview, and body text all read correctly.

## Phase 4

- **F-con2-9** (major, continuity re-check) — Reviewed the Overview (lines 6-13) against the
  finding. Phase 3's fix for F-exc-19 had already established the exact distinction the
  finding requires: "Most of these opcodes are valid encodings with architecturally undefined
  behaviour" (the twelve test-variant opcodes) versus "Two of the fourteen, \opcode{27} and
  \opcode{2F}, are the exception -- each has fully defined, tested behaviour ... but no public
  assembly syntax." No sentence in this file describes 0x27/0x2F as members of the
  test-variant family or as architecturally undefined; the opcode table's Notes column also
  already reads "No public assembly syntax" for both rows, not "Test variant." Per Gate 2
  ruling H the count of 14 stands. No text changed in this file — Overview already internally
  consistent with keeping 0x27 and 0x2F under this resolution direction. Appendix A's blanket
  sentence and Chapters 13/16 are being narrowed separately by their own editors, per the
  task's instructions; not touched here.

## Open questions (Phase 4)

- None.

## Verification (Phase 4)

- `grep -c 'begin{'` / `grep -c 'end{'`: 16 / 16 (unchanged, no edits made).
- Brace count: 126 `{` / 126 `}` (unchanged).
