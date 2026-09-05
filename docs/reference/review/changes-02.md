# Changes — chapters/02-cpu-architecture.tex

## Findings applied

- **F-arch-1** (blocker) — PC/pl/ph fetch model was wrong. Removed the two "Increment PC by 1"
  bullets from the Instruction Fetch phases (PC/`\reg{p}` does not change during either fetch;
  the low word is read from `\reg{p} + 1`), and added "Advance `\reg{pl}` by 2 to the address of
  the next instruction (`\reg{ph}` is unchanged)" to the Decode phase bullet list.
- **F-arch-2** (major) — Reworded the "Fixed Execution Sequence" paragraph so DMA is no longer
  described as the sole cost exception: every coprocessor call now costs six cycles for the
  processing-unit instruction plus a further six for the coprocessor dispatch, with DMA
  additionally holding instruction fetch until the transfer completes.
- **F-arch-7** (major) — Figures `fig:data-read-timing` and `fig:data-write-timing` had the bus
  request asserted one column after the "Mem" phase label. Shifted BAS/BRW/BAT/A0--A23/D0--D15/
  BACK one column left in both figures so the request is asserted during the phase labelled
  "Mem," matching the full-instruction figures (`fig:load-instruction-cycle-timing`, etc.).
- **F-arch-8** / **F-con-34** (same location) — Added arithmetic shift left (`\mnemonic{ASL}`)
  to the barrel-shifter description, and converted the shift-type list to `\mnemonic{}` macros:
  LSL, LSR, ASL, ASR, RTL, RTR.
- **F-arch-9** — Chapter 6/implementation number phases 0--5; this chapter's SYNC timing-rule
  cell said "phase 1." Reworded the cell to name the phase instead of relying on a number:
  "Asserted while phase 0 (Instruction Fetch, high word) ... is active." (see "Findings applied
  with a narrower fix" below for why a full manual-wide renumbering was not attempted here).
- **F-arch-11** — Added the missing latch-vs-lose rule for interrupt pulses shorter than an
  instruction boundary, both in the IRQ pin-description prose and in the "Asynchronous and
  Boundary-Sampled Inputs" table: a line asserted while its enable bit is set is latched at
  that moment even if released before the next boundary; only an assertion that begins and ends
  while the enable bit is 0 is lost. This mirrors the resolution already accepted for the
  equivalent finding in Chapter 5 (F-data-17), so it is not a guess.
- **F-con-35** — "Two barrel shifter units sit..." changed to "A barrel shifter sits..." (the
  non-inventing option offered by the finding, since no other chapter documents a second
  shifter or its purpose).
- **F-con-60** / **F-arch-17** (same co-processor→coprocessor fix, same lines) — Fixed all four
  "co-processor"/"Co-processor" spellings in this chapter (BAT table rows, BAT example prose,
  reset-via-RSET prose).
- **G1-T4-02** — Checked for `Example:`/`Examples:` blocks to number per the Gate 1 template
  decision; this chapter has none (it uses only informal "for example" prose, not instruction-
  entry Example blocks), so there was nothing to renumber.

## Findings applied with a narrower fix (or not applied) and why

- **F-arch-5** — Not applied. The finding's fix ("For register-displacement LOAD forms the
  shift is applied to the word returned from memory during write-back") directly contradicts
  Gate 2 ruling G ("LOAD shift applies to the offset register (source), never to the loaded
  data"). Per the ruling, the ruling wins. The existing sentence — "The shift is applied to the
  first source operand ... Shifts cannot be applied to the result or any other source operand"
  — already matches ruling G, so it was left unchanged.
- **F-arch-9** — Applied the cell-level fix (name the phase instead of a bare number) rather
  than renumbering the whole Six-Stage Execution Phases `enumerate` (and the "Stage 4" mention
  in §Parallel Functional Units) from 1–6 to 0–5. A full renumbering is a manual-wide decision
  affecting cross-references from other chapters/owners and was out of scope for a single-file
  fix; the naming-based fix removes the ambiguity the finding flagged without that risk.

## Findings skipped (not accepted / deferred)

- **F-arch-10** — status `defer` (Gate 2: "author to check the hardware figures"). Left the
  24 MHz max-rate and 5V ±5% supply content untouched.
- **F-con-72** — no `status` field recorded (booktabs/`\hline`/`\label` conversion for the two
  Chapter 2 tables). Treated as not accepted and left the two tables as `\hline`, `\begin{table}[]`,
  no `\label`.

## Copy edit (pass two)

- Fixed typos: "SIRCIS" → "SIRC-1"; "conjuction" → "conjunction".
- Fixed "e.g." → "for example" (one instance) and "less types of instructions" → "fewer types
  of instructions"; also normalized "upper word" → "high word" per STYLE.md terminology in the
  same sentence.
- Converted six subsubsection headings to Title Case for consistency with the rest of the
  chapter: Protected-Mode Output, External CPU Halt Input, Force Trace Mode Input, Maskable
  Interrupt Inputs, Non-Maskable Interrupt Inputs, Reset Input, Reset Output.
- Replaced a bare cross-reference ("See the exceptions section for more details.") with
  `Chapter~\ref{ch:exceptions}`.
- Removed second person ("your system") from the RSTI note; reworded in third person with
  "must" for the two hardware requirements bundled in that sentence.
- Converted future-tense description prose to present tense (11 instances of "will") to match
  STYLE.md's third-person/present-tense rule, e.g. "will be driven" → "are driven", "will cause"
  → "causes", "will disable them" → "disables them." Left one genuine future/conditional
  example ("if a memory controller knows that the DMA coprocessor will be reading...") since it
  describes a hypothetical external device's future action, not an architectural fact.
- Fixed a "No selected device should drive D0--D15" table cell — "should" implied a mere
  recommendation for what is actually a bus-contention requirement; reworded as declarative
  "No device drives D0--D15" to match the style of the sibling cells in the same column.
- Minor grammar/units fixes: "24 Mhz" → "24 MHz"; "acts very similar to" → "behaves much like";
  "hardware driven debugging" → "hardware-driven debugging" (hyphenated compound adjective);
  "CPU internal six stage sequence" → "CPU's internal six-stage sequence" (possessive and
  compound-adjective hyphen), with the surrounding sentence reworded to present tense.
- No hex literals, `0b` binary literals, or `e.g.` remained after the pass; no `Example:` blocks
  needed numbering (see G1-T4-02 above).

## Open questions

- F-arch-9: should the whole manual standardize on 0-indexed execution phases (matching Chapter
  6 and the implementation), including this chapter's `enumerate` list and the "Stage 4" mention
  in §Parallel Functional Units? This chapter's fix only removed the ambiguous bare number in
  one table cell; a full renumbering is a cross-chapter decision.
- F-arch-10 (deferred): the 24 MHz max clock rate and 5 V ±5% supply figures have no stated
  basis anywhere in the corpus. Left as-is per the defer status, but flagging again per the
  original finding in case the author's hardware-figure check is still pending.

## Phase 4

Applied F-con2-26 (minor, verbatim `fix`): renumbered the execution-phase list in
`chapters/02-cpu-architecture.tex` from 1-based to 0-based, matching the fault-detection prose
and the SYNC definition (`phase 0` = Instruction Fetch, High Word) elsewhere in this chapter.

- Converted the six-item `enumerate` (§Six-Stage Execution Phases) to a `description` list with
  labels `Phase 0: Instruction Fetch (High Word)` through `Phase 5: Write Back`, per the fix's
  instruction to use a `description` list rather than `enumerate`.
- Changed "The ALU and AU operate in parallel during Stage 4" to "...during phase 3 (Execute and
  Address Calculation)" to match the new numbering.
- Verified the phase list still agrees with the settled program-counter model already in this
  chapter (Phase 3 editor's F-arch-1 fix): `\reg{p}` is unchanged during both fetch phases, and
  `\reg{pl}` advances by 2 at decode; no change was needed there, since it already reflects that
  model.
- Checked the rest of the chapter for other bare phase-number references that this renumbering
  could invalidate: only line 623 ("SYNC ... phase 0 (Instruction Fetch, high word)") and the
  reset-input description ("resets to phase 0") use a numeric phase, and both already say
  "phase 0" for Instruction Fetch (High Word), so they are unaffected by (and consistent with)
  the new 0-based numbering. No other numeric phase references exist in this chapter.
- This resolves the open question raised in the Phase 3 changes log entry above (whether to
  0-index this chapter's phase list).

Findings applied: F-con2-26.

Findings not applied as written: none — the fix applied cleanly to this file. (Note: F-con2-26's
`fix` also names `appendix-b-timing.tex` and `12-instruction-summary.tex` as containing copies
of the same list; those files are outside this editor's ownership and are not touched here.)

Open questions: none new. The prior open question about 0-indexing this chapter's phase list is
now resolved by this pass.

Summary: Renumbered the chapter's execution-phase list and its one cross-reference from 1-based
to 0-based to match the chapter's own fault-detection and SYNC-signal prose.
