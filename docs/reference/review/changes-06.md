# Change log: chapters/06-exceptions.tex

## Findings applied

- **F-exc-1** (line 528) — "128 user-accessible trap vectors" corrected to 160
  (`0x60`-`0xFF` inclusive), matching Gate 2 ruling P.
- **F-exc-2** (fault-metadata bitfield figure) — reversed the `bytefield` order to
  `Reserved | Original Fault | Current Fault | D | BAT` (MSB left), matching the itemized
  bit list and the house bytefield convention.
- **F-exc-3** (fault categories/return-address contradiction) — applied Gate 2 ruling A.
  Split the old "Retryable" category into **Retryable** (Alignment Fault; Segment Overflow
  Fault only when raised by program-counter wraparound) and a new **Aborted, non-retryable**
  category (Bus Fault, Bus Protection Fault, Invalid Opcode Fault, Privilege Violation Fault;
  Segment Overflow Fault for any other address-computation overflow). Updated each affected
  fault's `\textit{(...)}` tag, rewrote the Bus Fault description (it previously claimed the
  PC is "not incremented" and the operation "retried", which is now false), added the
  odd-`pl` re-fault warning to the Alignment Fault description, and corrected the "Return
  Address" column for vectors `0x01`, `0x03` (annotated), `0x04`, `0x05`, `0x09` in the
  Exception Quick Reference table.
- **F-con-13** / **F-exc-13** (`system_ram_offset`, lines 442-443, 615-616) — dropped the
  undefined `system_ram_offset` register entirely (option (a) in both findings); the
  vector-table address computation and the reset-vector fetch now state flat addresses
  (`0x000000`/`0x000001` equivalents), matching Chapter 1, Chapter 4, and the quick-reference
  table, all of which already assume a zero base.
- **F-con-24** / **F-exc-10** (duplicate/inconsistent RETE step lists) — deleted the
  duplicate enumeration under "Return from Exception" and replaced it with a cross-reference
  to the corrected "Returning from Exceptions" list (added a `\label` to support the
  `Section~\ref{}`). Added the missing `SR.EA` restore step to the surviving list. Removed
  "interrupt mask level" everywhere; the manual now says `current_exception_level`
  throughout.
- **F-exc-11** (exception handling flow order) — reordered the numbered list so vector-table
  address computation and the two vector-word reads happen before the link-register write,
  `SR.P` clear, and exception-level update, matching "Exception Entry Side Effects".
- **F-exc-12** (Instruction Trace Fault / TRCE) — added "or when the `TRCE` input is
  asserted" to the fault description, added the phase-0 sampling rule and the
  "exception-unit dispatches are never traced" rule, and changed the Maskability cell for
  vector `0x06` from "SR.T gated" to "SR.T or TRCE gated".
- **F-exc-14** (reset table missing current exception level) — added a "Current exception
  level" row to the Reset State table.
- **F-exc-15** (double-fault ETFR/ETTR listing) — added a sentence after the listing stating
  that the saved exception level is not restored by this sequence and that the CPU state
  should be treated as unrecoverable.
- **F-exc-16** / **F-exc-17** / **F-exc-18** / **F-exc-28** — applied Gate 2 ruling K/L. Added
  one sentence each, with no description of implementation panics: opcodes `0x2`-`0x8` are
  reserved and architecturally undefined; issuing `0xE`/`0xF` directly (e.g. via a
  supervisor-mode `COPI`) is architecturally undefined; executing RETE at exception level 0
  is architecturally undefined; two fault conditions detected in the same instruction before
  either is dispatched is architecturally undefined.
- **F-exc-8** (link register field count) — "two pieces of information" corrected to
  "three"; added the "Saved Exception Level" bullet.
- **F-exc-9** (fault-metadata... link-register figure packing) — added a note that the Saved
  Level/Reserved fields are illustrative only and that `ETFR`/`ETTR` transfer only the return
  address and status register, per Gate 2 ruling Q.
- **F-con-45** / **F-exc-23** (24-bit vs 32-bit address) — "Return Address" bullet and the
  vector-table-address sentence now say "24-bit address ... stored as two 16-bit words".
- **F-con-46** / **F-exc-24** (ETFR/ETTR listing comments using "level 6/7") — both findings
  target the same four comment lines with slightly different wording; applied one
  reconciled wording ("Save/Restore fault link register (register 6)" and "... fault
  metadata register (register 7)") that satisfies both findings' intent of not calling a
  register index a "level".
- **F-con-61** (e.g. → for example) — all 4 instances converted.
- **F-con-63** (abort exceptions (faults) → faults) — all 3 instances (Fault Categories
  intro paragraph, Exception Priority list, old Return-from-Exception paragraph) resolved;
  two were rewritten more substantially to also fix the ruling-A content, one was a
  one-word deletion.
- **F-con-69** (bare `R7` / bare `ETFR`/`ETTR`) — `R7` → `\reg{r7}` and `ETFR`/`ETTR` wrapped
  in `\mnemonic{}` in the "Accessing Link Registers" bullets.
- **F-exc-22** (faults "cannot be masked or deferred") — narrowed to "once raised, cannot be
  masked or deferred", with the two conditionally-raised faults (Segment Overflow on
  `SR.A`, Instruction Trace on `SR.T`/`TRCE`) called out explicitly.
- **F-exc-25** (WAIT description) — "until an exception occurs" → "until an enabled
  interrupt or reset occurs".
- **G1-T4-06** (numbered `Example N-M:` blocks) — this chapter has no `Example:`/`Examples:`
  blocks (no instruction entries); nothing to number.

## Findings skipped (not accepted, or not mine)

- **F-exc-7** — `status: code-wrong`, not `accepted`; left the "vector below `0x60`" bullet
  in Privilege Violation Fault untouched per the ruling ("manual stands").
- Findings whose `file:` field is a different chapter (e.g. F-con-60 "co-processor" pass,
  F-con-70 "you/your" pass, F-con-23, F-con-56, F-tim-9, F-con-31/36/44/49) were left for
  their owning editors, even where their evidence or fix text touches a line in this file.

## Other copy-edit changes (not tied to a triaged finding)

- Applied the manual-wide hex-literal macro pass to every remaining bare `0x...` literal in
  this chapter: vector IDs and encoded field values → `\texttt{0x..}`, concrete addresses →
  `\addr{0x......}` (matching the convention already used in Chapter 1's vector table).
- Converted remaining plain-hyphen numeric ranges (bit ranges, register-index ranges,
  vector-ID ranges) to en dashes per STYLE.md §3.
- Fixed "co-processor" → "coprocessor" at line 85 (the one instance in this chapter) and
  "You could provide..." → third person at the end of "Link Register Preservation" (the one
  instance in this chapter) — both are chapter-local STYLE.md terminology/voice conformance
  independent of the cross-file findings that also flagged them.
- Fixed subject/verb agreement ("... is overwritten" → "are overwritten"), removed the
  filler "Keep in mind that" and "just" fillers, added a missing comma after "When a
  hardware exception occurs", added an Oxford comma in the Overview, and changed "which will
  take control" to present tense "takes control".
- Wrapped a handful of additional bare mnemonics immediately adjacent to findings already
  being edited (`EXCP` at line 158, `ETFR` at line 340, `RETE` at line 527, `RETE`/`ETFR`
  wrap in Accessing Link Registers) for local consistency; did not attempt a chapter-wide
  sweep of the dozens of other bare `RETE`/`ETFR`/`ETTR` mentions in narrative prose, since
  that is a much larger normalization not called for by any triaged finding or by the task's
  named mechanical passes.

## Open questions

- The Exception Quick Reference table's Segment Overflow row now reads "Faulting instr. (PC
  wrap only), else next" to reflect that the same fault vector has two different
  return-address behaviors depending on cause. This is a faithful reading of Gate 2 ruling A
  ("Only Alignment and PC-wrap Segment Overflow ... are retryable"), but the ruling does not
  explicitly confirm that a *non*-PC-wrap Segment Overflow belongs in the "next instruction"
  group rather than being unspecified. If that's wrong, the fix is confined to this row and
  the corresponding sentence in the Segment Overflow Fault description and the Fault
  Categories list.

## Phase 4

Applied F-con2-8 (major, consistency-2.md) per Gate 2 ruling P.

- Added a single statement of the vector-table total in the "Vector Table" intro paragraph:
  "The vector table has 256 entries. Vectors 0x00--0x5F (96 entries) are reserved for the CPU;
  vectors 0x60--0xFF (160 entries) are user trap vectors."
- Added a catch-all "0x11--0x5F (excluding 0x20, 0x30, 0x40, 0x50): Reserved" item to the vector
  table itemize list, closing the gap between the named privileged/hardware vectors and 0x60.
- Added a matching "0x11--0x5F" Reserved row (address range 0x0022--0x00BF) to
  Table~\ref{tab:exception-quick-reference}.
- Did not touch chapters/01-introduction.tex; the fix's request to add the same reserved band to
  Table~\ref{tab:intro-vector-table-regions} is out of scope (owned by another editor).
- No base-offset term was present or reintroduced; vector-table addressing in this chapter
  remains `vector * 2` from address 0x0.

Findings applied: F-con2-8.
No findings could not be applied as written.
No new open questions.
