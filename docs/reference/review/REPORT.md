# Editorial pass report — 2026-09-16 (scoped run)

Scoped re-run of `/manual-edit`, not a full pass: skipped Phase 1 (period benchmark/style guide,
already settled) and the full Phase 3 copy-edit sweep (only chapters with an actual finding got a
dedicated editor). Ran Phase 2 fact-checking on groups G2 (data/status/condition), G3
(exceptions/undocumented), G7 (coprocessor/meta), and G8 (summaries/appendices) — the chapters
heavily edited earlier this session — plus a full-manual consistency check regardless of scope,
since cross-file drift can appear anywhere.

## Findings

65 raw findings across 5 reports (`facts-data.md`, `facts-exc.md`, `facts-cop.md`, `facts-sum.md`,
`consistency.md`), deduplicated to 56 in `triaged.md`.

| Severity | Count |
|---|---|
| blocker | 6 |
| major | 5 |
| minor | 32 |
| nit | 3 |
| logged as implementation bugs, no manual change | 5 |
| author decided "no change needed" | 1 |
| still open, author investigating | 1 (disputed pair) |

All content findings were resolved with the author in conversation at Gate 2 and applied. See
`triaged.md` for the full list with evidence and exact fix text.

## Notable findings

- **Chapter 6's exception quick-reference table** listed the wrong return address for three of six
  retryable faults (Bus, Privilege, Bus Protection all said "next instruction" instead of
  "faulting instruction") — the single artifact an implementer would build the exception unit
  from, and the one place in the manual that disagreed with itself. Fixed.
- **Chapter 7's worked register-encoding example** was off by one on every field (`ADDR r4, r5`
  shown with R1=R2=0x4 instead of 0x3/0x3/0x4) — reads register names as if they were the field
  values, sitting directly below the correct field diagram. Fixed.
- **The "Retry-Safe DMA Copy Routine" example** (Chapter 16, written earlier this session) used
  `l` as both the DMAT destination register and the subroutine link register, so `RETS` would
  jump into copied data instead of returning to the caller. Rewritten with a new calling
  convention (destination passed via r1/r2, return address saved off `l` before it's repurposed)
  and verified against the real assembler. Also had to be split into two parts and font-reduced to
  fit its non-breaking `[H]` float after the rewrite grew it past one page (see "Layout" below).
- **This session's own new Appendix D.1** ("Assembler Directives," added earlier in this session)
  had a blocker-severity error: it called the address space 32-bit in three places, when the rest
  of the manual consistently says 24-bit. Also had a self-contradiction (says `@label` "references
  a previously declared label," while its own worked example forward-references one) and an
  overclaim about `.EQU` matching a dead code path in the toolchain. All fixed or logged.
- **Two genuine Rust bugs** surfaced by fact-checking, unrelated to any manual error: a
  segment-overflow trap that still uses an unsigned overflow check for general (non-pre-decrement)
  negative displacements, spuriously faulting or missing genuine underflows depending on the exact
  value (`IB-1`, logged only); and an interrupt-dispatch routine that gave up on the whole cycle if
  the single highest-priority pending line had been masked after being latched, silently blocking
  every lower-priority pending interrupt too (`IB-3`, **fixed this pass** with two new unit tests,
  confirmed a genuine oversight by the author rather than a deliberate simplification).

## Author decisions at Gate 2

Nine deferred findings plus one disputed pair were resolved in conversation with the author:

- Double-fault "original fault type" field: keep the current Rust behavior (records the
  *triggering* fault), fix the manual to match — the triggering fault is genuinely unrecoverable
  any other way once inside the handler, while the fault already being serviced is always cheaply
  recoverable by the handler itself.
- Interrupt masking at dispatch: confirmed an oversight, fixed in Rust (see `IB-3` above).
- Invalid Opcode Fault scope: narrowed to "coprocessor ID not implemented" only, matching the
  manual's own pre-existing (but until now contradicted) Model and Revision Policy. Reserved
  operations/operands within an *implemented* coprocessor are architecturally undefined, not this
  fault — applied consistently across Chapters 6 and 16, including the maths-unit entries.
- DMA fault-timing details with no DMA implementation to check against: acceptable as spec-ahead-
  of-code, no change.
- `.EQU` on data directives and the `.r`/`.l`/`.u` suffix data-directive gap: both logged as
  unfinished toolchain features (`IB-4`, `IB-5`) rather than narrowing the manual's claims.
- `.ORG` alignment: manual now states the even-address rule explicitly.
- Appendix F interrupt latching: resolved on period-appropriate grounds (real level-sensitive IRQ
  lines on simple 1980s CPUs put the burden of holding the line on the requesting device, not on
  the CPU to remember a sub-instruction pulse) — matches Chapters 5 and 6, both already correct.
- **Still open**: the reset-vector-fetch cycle count (Chapter 6 says 4 cycles; Appendix B and
  Chapter 16 both say 6, an 18-cycle `RSET` total). Two reviewers disagreed — one traced it to a
  hard-coded `CYCLES_PER_INSTRUCTION = 6` constant and a matching test and recommended fixing
  Chapter 6; the other flagged the mismatch without checking the code. Author's lean is "probably
  6" but wants to dig in further before committing. **Not touched this pass.**

## Implementation bugs (`implementation-bugs.md`)

- `IB-1` segment-overflow trap, negative displacements — logged only.
- `IB-2` exception-unit reserved-opcode panic — logged only.
- `IB-3` interrupt dispatch masked-line blocking — **fixed this pass**, two new unit tests, full
  `peripheral_cpu` suite (171 tests) and the whole workspace pre-commit suite (230 tests) pass.
- `IB-4` `.EQU` unusable on data directives (dead `PlaceHolder` code path) — logged only.
- `IB-5` linker only supports `FullAddress` refs on data directives — logged only.

## Layout

One page-overflow found and fixed during final verification: the rewritten DMA example (Chapter
16, Example 16-7) grew past what its non-breaking `[H]` float could hold on one page after the
`l`-aliasing rewrite, overlapping the page footer. Split into two parts (routine / fault handler)
under the same example number, with `\footnotesize`/`\scriptsize` overrides on the two listings.
Confirmed via `main.log`: the only remaining `Overfull \vbox` in the whole manual is a pre-existing
9.9pt one in Chapter 16's own intro section, unrelated to this pass's edits.

## Verification

- Full 3-pass `make all`: 0 undefined references/citations.
- `make qa` / `./qa.sh`: 70 extracted inline listings assemble cleanly (up from 69 -- the DMA
  example split counts as two now), all 5 Appendix D worked-example programs build/run/match their
  expected output via their own `make check`.
- Chapter 16's rewritten DMA example additionally hand-traced for stack-arithmetic correctness
  across all three exit paths (success, retry, give-up), not just syntax.
- Visual spot-checks (rendered to PNG): Chapter 6's fault table, the Chapter 16 DMA example
  (before and after the layout fix), Appendix D.1.

## Pages needing a human visual check

None flagged beyond what's already covered above -- the layout pass found and fixed the one real
issue. Worth a skim of Chapter 16 in full given the scale of edits there (130 insertions, 81
deletions in one file).

Branch is ready for `git diff main` review once committed. `implementation-bugs.md` needs a
separate pass on the Rust for `IB-1`, `IB-2`, `IB-4`, `IB-5`.
