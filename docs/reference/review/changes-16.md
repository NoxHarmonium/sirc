# Changes — chapters/16-coprocessor-instructions.tex

The file was already partway through this same editing pass (uncommitted work from an
interrupted run). I ran `git diff` first, verified each accepted finding against the current
text, and only touched what was still missing or half-finished.

## Findings applied

Already correctly applied by the interrupted run (verified against `claim`/`fix` text, left
alone):
- F-con-20, F-con-56, F-cop-10, F-cop-11, F-cop-12, F-cop-13 (ruling M), F-cop-15, F-cop-16
  (ruling H), F-cop-17, F-cop-20, F-cop-21, F-cop-22, F-cop-23, F-cop-24, F-cop-26 (ruling K/L),
  F-cop-4, F-cop-5, F-cop-6 (ruling K/L), F-cop-7 (ruling K/L), F-cop-8 (also covers ruling J via
  F-con-49's text), F-con-36, F-con-42, F-con-49, F-cop-19, G1-T2-16 (Applicability line, all 11
  entries), G1-T3-16 (Instruction Fields list, all 11 entries; bit ranges cross-checked against
  `07-instruction-formats.tex`).

Completed in this pass (the interrupted run had left these half-finished — one entry out of
eleven still had the old form):
- **G1-T1-16**: the COPI/COPR entry's `Condition codes:` label (line ~125) was the one entry
  still using the old name; every other entry already said `Condition field:`. Renamed to match.
- **G1-T4-16**: the COPI/COPR entry's example block was still the old unnumbered
  `\textbf{Examples:}` (plural), while the EXCP and ETFR/ETTR entries were already `Example
  16-2:` and `Example 16-3:`. Renumbered it to `Example 16-1:`, giving a consistent 16-1/16-2/16-3
  sequence.
- **F-cop-18**: the added sentence ("For COPR the privilege decision is made on the run-time
  value of rS sampled during decode...") was present in the Common Semantics table's Privilege
  row but missing from the COPI/COPR instruction entry's own `Privilege:` field, as the fix
  explicitly requires both. Added it there.

## Findings skipped (status not `accepted`)

- **F-cop-14** (status: not-implemented) — skipped as a finding, but its content is covered by
  Gate 2 ruling M, which the launcher asked me to apply directly: the DMA unit stays listed as
  Required in the compatibility table, and the chapter notes once (Section "DMA Unit
  (Coprocessor 2)") that the reference emulator has not implemented it yet. This note was already
  present from the interrupted run.
- **F-cop-9** (status: code-wrong) — skipped as a finding. Its underlying content (EXCP with a
  vector below 0x60 raising a privilege-violation fault, per ruling J) is already delivered
  through F-con-49's accepted fix, applied to the EXCP entry's Exceptions field.

## Findings applied with a deviation from the literal fix text

- **F-con-58** (Clobbers field): the fix offered two options — add a `Clobbers` field to Chapter
  11's canonical list (out of scope; that's a different chapter), or fold the clobbered-register
  information into the existing `Write-back` field of the three DMA entries. The interrupted run
  took the second option (e.g. DMAT's Write-back: "`r1`–`r7` are clobbered; after completion they
  contain the final transfer window."). I left this as-is rather than introduce a new field name
  that Chapter 11 doesn't yet define.
- **F-cop-19**: applied with `R1`, `R2` in upper case rather than the finding's literal lower-case
  suggestion, since STYLE.md reserves upper-case `R1`/`R2`/`R3` (no macro) for abstract
  field-position labels — exactly this context. This is the more faithful application of house
  style than the literal finding text.

## Open questions

- None raised by this pass beyond what the rulings already resolved. The one substantive
  ambiguity in this chapter (whether general-purpose/address registers hold "unchanged" or
  "undefined" values after power-on reset, flagged in F-cop-17) was already resolved in the
  applied text as "not affected by reset and hold undefined values after power-on," which the
  finding itself flagged as a decision the architecture had not yet recorded — flagging here in
  case the user wants to confirm that resolution explicitly.

## Verification

`grep -c 'begin{'` and `grep -c 'end{'` both return 51; a full brace-depth walk of the file
returns to 0. No `\label`, macro, or environment was removed.

## Phase 4

Applied the five continuity findings from `consistency-2.md`, per the Phase 4 task brief:

### Findings applied
- **F-con2-6** (blocker): replaced the base `COPI`/`COPR` entry's Timing line, which had reverted
  to a bare "6 cycles," with the settled 12-cycle model shared with Chapter 12 and Appendix B:
  "6 cycles for the processing-unit call, plus a further 6-cycle coprocessor dispatch slot (12
  cycles in total), plus any bus wait states." No other Timing statement in this chapter needed
  changing — the Common Semantics table and every other instruction entry (`EXCP`, `WAIT`,
  `RETE`, `RSET`, `ETFR`/`ETTR`, DMA and maths entries) already state the call-plus-dispatch-slot
  form or a chapter-specific variant not covered by this finding.
- **F-con2-12** (major): added an `\textbf{Operation:}` pseudocode block, placed before
  `\textbf{Description:}`, to the `EXCP`, `WAIT`, `RETE`, `RSET`, and `ETFR`/`ETTR` entries. Each
  block restates the normative effect already given in prose (in this chapter and in Chapter 6's
  exception-entry/return mechanism) using the variable names already established in this
  chapter's own pseudocode style (`condition_true`, `current_exception_level`,
  `pending_coprocessor_command`, `link[n]`, `pc`, `sr`, `vector_table`).
- **F-con2-13** (minor): swapped Operation and Description so Operation precedes Description, in
  the base `COPI`/`COPR` entry and in `DMAR`, `DMAW`, `DMAT`, `MULU`/`MULS`, and `DIVU`/`DIVS`, as
  the fix listed. No prose was changed, only the block order.
- **F-con2-21** (major): changed the Exception Unit row's Operations cell in
  `tab:standard-coprocessor-ids` from "`0x0`–`0x1`, `0x9`–`0xD` (software); `0xE`–`0xF`
  (internal)" to "`0x1`, `0x9`–`0xD` (software); `0x0`, `0xE`–`0xF` (internal)," and extended the
  following note to read "Exception-unit operation nibble `0x0` is the internal no-operation
  encoding and operation nibbles `0x2`–`0x8` are architecturally undefined," matching Chapter 6
  and Ruling K.
- **F-con2-25** (minor): replaced the single unexpanded `\texttt{HIE}` in the `WAIT` entry's
  Description with "the hardware interrupt enable bits (`SR.E1`–`SR.E4`)," matching the term used
  everywhere else in the manual.

### Findings applied with a deviation
- None. All five fixes applied as written, adapted only to current surrounding wording where the
  quoted `claim` text had shifted slightly (e.g. exact line wrap around the `HIE` sentence).

### Open questions
- None raised by this pass. The new `Operation` pseudocode blocks for `EXCP`, `RETE`, `RSET`, and
  `ETFR`/`ETTR` are restatements of prose already present in this chapter and in Chapter 6; no
  new technical claim was introduced.

Verification: `grep -c 'begin{'` and `grep -c 'end{'` both return 56 after the Phase 4 edits, and
a full open/close brace count over the file (741/741) matches.

## Phase 4 follow-up: F-con2-32 (numbered examples in every entry)

### Findings applied
- **F-con2-32**: added one short `Example:` block to each of the eight entries that lacked
  one — `WAIT`, `RETE`, `RSET`, `DMAR`, `DMAW`, `DMAT`, `MULU`/`MULS`, and `DIVU`/`DIVS` —
  numbered `Example 16-4:` through `Example 16-11:` in document order, following on from the
  pre-existing `Example 16-1:`–`16-3:` on `COPI`/`COPR`, `EXCP`, and `ETFR`/`ETTR`. Each block
  uses only syntax this chapter or Chapters 3/8/14/15 document as legal (`LOAD ah/al/lh/ll,
  #imm` to build a 32-bit address; `LOAD rN, #imm` for maths operands; `BRAN @label` for the
  wait loop) and stays inside the entry's own `instructionbox`. Example vectors/addresses avoid
  anything an existing ruling flags: no `EXCP` vector below 0x60 appears, no exception-unit
  opcode 0x2–0x8 is used, and DMA/maths examples use only the documented command forms and
  in-range operand counts.

### Findings applied with a deviation
- None. All eight examples are new content, not a rewrite of an existing fix.

### Open questions
- None. The examples restate legal syntax and semantics already normative in this chapter;
  no new technical claim was introduced.

Verification: `grep -c 'begin{'` and `grep -c 'end{'` both return 64 after this pass; a full
brace-depth walk of the file returns to 0 with no negative excursions. Each of the eight new
`Example 16-N:` blocks was confirmed to sit inside the correct entry's own `instructionbox`.
