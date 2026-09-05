# Changes — appendix-b-timing.tex

## Findings applied

All `accepted` findings for `chapters/appendix-b-timing.tex` were already present in the
file from the interrupted prior editing run. I verified each against `triaged.md` and the
Gate 2 rulings and confirmed the applied text matches the required fix (no further edits were
needed, and no finding was applied twice):

- F-tim-2 — `ADDI r7, #0, #10` replaced with `LOAD r7, #10`.
- F-tim-3 — "Combine operations with shifts" example corrected to `ADDR r1, r2, r2` / `ADDR r1,
  r1, r2` (2*r2 then 3*r2), matching the "Use" line's `LSL #1` result.
- F-tim-4 / F-tim-5 / F-con-39 — "post-increment/pre-decrement" example corrected to
  `LOAD r1, (#0, a)` / `ADDI al, #1`.
- F-tim-6 — all three trailing-colon labels (`loop:`, `skip:`, and the unrolled-loop `loop:`)
  converted to the leading-colon form (`:loop`, `:skip`).
- F-tim-7 / F-con-40 — "Use conditional execution" example's `BRAN|<= @skip` changed to
  `BRAN|LO @skip` so both halves of the example use the unsigned domain consistently with
  `ADDI|HI r3, #1`, per Gate 2 ruling B (glosses stand; only the signed/unsigned mismatch is
  fixed).
- F-tim-8 — Overview reworded from "DMA meta-instructions are the main exception..." to
  "Coprocessor calls are the exception: the dispatched coprocessor occupies at least one
  further slot after the six-phase call," and the Instruction Timing table gained rows for
  `COPI`/`COPR` (12 = 6+6), the exception-unit meta-instructions (12 = 6+6), and `WAIT`
  (12 + idle until an enabled interrupt or reset).
- F-tim-9 — new "Exception and Fault Timing" subsection added, stating the vector fetch/entry
  timing (phase 0 high word, phase 1 low word, entry committed at phase 3), interrupt latency
  (6 cycles to finish current instruction + 6 cycles to dispatch), and the 6-cycle `RSTO` hold
  before the reset-vector fetch.
- F-tim-11 — same new subsection also states, per Gate 2 ruling A, the phase at which each
  fault is detected (Alignment before phase 0; Privilege Violation at phase 2; Segment
  Overflow at phase 2 or 3; Bus/Bus Protection at the acknowledging phase) and that only
  Alignment and PC-wrap Segment Overflow save the faulting instruction's address and are
  retryable; Bus, Bus Protection, Invalid Opcode, and Privilege Violation save the next
  instruction's address and are not restartable.
- F-tim-10 — "Conditional Execution" bullets replaced to say all six phases still run one
  cycle each, but execute/memory-access/write-back are no-ops (no effective address, no
  `SR.A` check, no data bus cycle), and no register/memory/address-register update occurs.
- F-tim-12 — "Memory Access" bullets gained the bus-access accounting: two instruction-fetch
  accesses per instruction, one extra data access for load/store, two vector-fetch accesses
  for a coprocessor dispatch, giving `2w`/`3w` added cycles for `w` wait states per access.
- F-con-29 — "At 25 MHz" performance figure replaced with "At 24 MHz (maximum rated clock):
  ≈ 4.0 million instructions/second," matching the documented 24 MHz maximum clock rate.

No further changes were required from me on this pass; I only reviewed and confirmed the
above against the finding text and Gate 2 rulings.

## Findings skipped and why

- **F-tim-1** — status `code-wrong`. Per Gate 2 ruling B, the ARM-style unsigned glosses
  stand and the HI/LO comments in the "Conditional Block" example
  (`ADDR|HI r3, r4, r5 ; exec if r1 > r2`, `ADDR|LO r3, r6, r7 ; exec if r1 <= r2`) were left
  unchanged, as instructed.
- **F-tim-13** — status `defer`. The ARM6 and MIPS R2000 "Avg CPI" rows in the "Comparison to
  Other CPUs" table were left at `1.0`, untouched.
- **F-tim-14** — no `status` field in `triaged.md` (blank/unmarked, i.e. not `accepted`), so
  not in scope for this pass. Its text (en dashes for `2--7`/`4--158`, `\mnemonic{}` wrapping
  for `DMAR`/`DMAW`/`DMAT`) was nonetheless already present in the file from the interrupted
  run. I left it as-is: it is harmless, consistent with STYLE.md §3, and has no semantic
  effect, so reverting it would be counterproductive and is not required by my brief.
- **G1-T4-ap** (the `appendix-b-timing.tex` instance) — status `accepted`, but not applicable:
  this appendix contains no instruction-entry `Example:` fields to number (its
  "Program Timing Examples" section uses plain `\subsection` titles, e.g. "Simple Loop," not
  the `Example N-M:` template field used in Chapters 13–17). No action taken.
- All other findings in `triaged.md` belong to different files and are out of scope for this
  chapter.

## Open questions

- None. Every applicable, accepted finding for this file was already correctly applied by the
  interrupted run; I found no discrepancy requiring a judgment call.

## Phase 4

### Findings applied
- F-con2-4 (blocker): DMAR/DMAW and DMAT timing formulas converted to the 6-cycle-slot model: `12 + max(|n|-1, 0)` and `12 + max(2n-1, 0)`, replacing the stale `6 + max(...)` forms so a zero-count DMA operation now reads 12 cycles, matching Chapter 16.
- F-con2-5 (blocker): RSET removed from the shared "Exception-unit meta-instructions" row (which now lists only EXCP, RETE, ETFR, ETTR at 12 cycles) and given its own row: "RSET & 18 (6 call + 6 RSTO hold + 6 reset-vector fetch)", matching Chapter 16 and Chapter 6.
- F-con2-34 (minor): "a coprocessor dispatch adds two vector-fetch accesses" corrected to "an exception or fault dispatch adds two exception-vector-fetch accesses; a coprocessor dispatch performs no bus access of its own," since vector fetches belong to exception/fault dispatch, not coprocessor calls.

### Findings not applied as written
- None; all three fixes applied verbatim as specified.

### Open questions
- None.
