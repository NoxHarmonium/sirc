# SIRC-1 Reference Manual — Editorial Pass Report

Six phases, run 2026-09-05 to 2026-09-06, plus a follow-up correction round
on 2026-09-08 where the author reviewed every Gate 1/2 decision against the
actual source and revised five of them. Branch `main`. PDF: 244 pages,
builds clean (0 errors, 0 undefined references).

## Findings by phase and severity

| Phase | Source                        | Blocker | Major | Minor | Nit | Total                        |
| ----- | ----------------------------- | ------- | ----- | ----- | --- | ---------------------------- |
| 1     | Period benchmark gaps         | —       | —     | —     | —   | 13 (backlog, see below)      |
| 2     | 9 fact-checkers + consistency | 48      | 129   | 122   | 6   | 306 (raw, before dedup: 312) |
| 4     | Continuity re-check           | 7       | 16    | 12    | 5   | 40                           |
| 5     | LaTeX/layout QA               | 2       | 6     | 2     | —   | 10                           |
| 6     | Cold read                     | —       | 2     | 7     | —   | 10 (author selected 7 of 10) |

Phase 2's 306 counts by disposition: 288 **accepted** and applied, 10
**code-wrong** (routed to the Rust, not the manual), 2 **deferred**, 2
**rejected** as superseded, 2 held for Phase 4 alignment. All 40 Phase 4
findings and all 7 selected Phase 6 findings were applied.

## What was applied

- **22 chapters and appendices** each got one dedicated editing pass (Phase
  3), applying every accepted finding plus the Gate 1 template additions
  (an applicability line, a renamed condition-field line, and an
  instruction-fields list on every instruction entry; numbered examples
  throughout).
- **A continuity re-check** (Phase 4) caught what 22 parallel editors missed
  each other doing: chapter 3 still described the old two-increment
  program-counter model; coprocessor cycle counts disagreed three ways
  between chapters 12, 16, and the timing appendix; the save-versus-test
  opcode split had reverted to its pre-ruling form in two places; and the
  quick-reference appendix used its own flag shorthand instead of chapter
  11's symbol set.
- **Typesetting** (Phase 5): the `instructionbox` container was not
  breakable, so once the new per-entry fields and 8 new chapter-16 examples
  were added, most instruction entries overflowed their page — the worst by
  two full pages. Made the box breakable across pages; this alone resolved
  12 of the phase's 16 layout defects. The remaining 4 are single-page
  tables (chapter 12's instruction list, chapter 16's common-semantics
  table, and the two big appendix-A/E tables) that need conversion to
  `longtable` — a bigger, still-mechanical job left for a follow-up pass.
- **Cold read** (Phase 6): the author selected 7 of 10 findings. Chapter 2's
  external-interface section (chip layout, pins, timing notes — 741 lines)
  moved to a new Appendix F, since neither period reference manual keeps
  electrical/pin material in the same book as the architecture, and both
  introduce interrupt handling only after the programmer's model is
  established. Three chapter-2 cross-references left dangling by that move
  were fixed. The remaining fixes glossed notation and mnemonics at their
  first use instead of only where formally defined, added a timeline table
  for chapter 6's hardest paragraph, bridged chapter 8's two overlapping
  tables, and added orienting prose to every appendix D example.

## Author decisions of note (Gate 1 and Gate 2)

Five rulings were revisited and revised on 2026-09-08, after the author
reviewed the original decisions against the actual `sirc-vm` source and
the CPU's own design comments:

- **Fault return addresses (revised).** The original ruling (only Alignment
  and PC-wrap Segment Overflow retryable) contradicted the CPU's own design
  comment (`exception_unit/definitions.rs:121-131`), which names Bus Fault,
  Alignment, Privilege Violation, and Invalid Opcode as intended to be
  retryable. Final set: **Alignment, Bus, Bus Protection, Privilege
  Violation, and PC-wrap Segment Overflow are retryable; Invalid Opcode and
  non-PC-wrap Segment Overflow are not.** Root cause identified: the
  decode stage advances the program counter unconditionally in the same
  step it raises a privilege fault, so the link register captures the next
  instruction's address instead of the faulting one (`implementation-bugs.md`,
  F-exc-3).
- **Unsigned condition codes (revised).** `HI`/`LO` keep their ARM-style
  English meaning (higher / lower-or-same), but two lines of
  `definitions.rs` need to change so the hardware actually delivers it,
  given this CPU's correct and intentional C-on-borrow convention — see
  `implementation-bugs.md`, F-data-1. `CS`/`CC` need no code change; their
  real meaning (`CS` = lower, `CC` = higher-or-same, the opposite of ARM's
  `CS`/`CC`) is now documented correctly in chapters 7 and 10.
- **Register shift counts above 15 (revised).** Verified by running the
  actual code: this is well-defined, not undefined. `LSL`/`LSR`/`ASL` clamp
  a register-sourced count to 16 (full clear); `RTL`/`RTR` use the count
  modulo 16. Both documented as-is now.
- **`ASR` fill (elaborated).** Confirmed by running the code against a
  worked example and cross-checking an existing passing test: `ASR` only
  forces bit 15 to the sign; bits 14 down to `16-n` are zero-filled, not
  sign-extended, for any count above 1 — the test asserts the buggy value.
  True arithmetic-shift semantics (the manual's stated intent, unchanged)
  needs a code fix; see `implementation-bugs.md`, F-enc-18.
- **Register-offset `LOAD` shift (resolved as dead code, removed).** Traced
  end to end: the offset register is always used unshifted for the address;
  the shift suffix is computed on a hardcoded, discarded register and has
  zero effect. `STOR`'s shift, by contrast, is fully functional (applies to
  the value being stored) and needed no change. The shift suffix is now
  removed from `LOAD`'s legal syntax in chapters 8 and 14.
- **`NOOP` (resolved via register renumbering, not reassignment).** Rather
  than move `NOOP` to a non-zero-encoded register (which would lose the
  property that erased/uninitialized memory, all-zero words, safely decodes
  as a no-op), the author chose to renumber the register file: `r1`-`r7`
  now occupy encoding `0x0`-`0x6`, and `sr` moves to `0x7`. `NOOP` becomes
  `ADDI[N] r1, #0`, still the all-zero word, and is genuinely unprivileged
  since `r1` is an ordinary register — no privilege-check special case
  needed. Verified nothing else in the codebase depends on `sr` being index
  0 (every reference goes through the `RegisterName::Sr` enum symbol).
  Chapters 3, 12, 17, and appendix E updated.
- Only Alignment, Bus, Bus Protection, Privilege Violation, and PC-wrap
  Segment Overflow are retryable (see above; supersedes the original
  narrower ruling).
- LOAD never updates flags under its default form; an explicit status-flag
  suffix on LOAD leaves all four flags undefined.
- Displacements are signed two's-complement; the emulator's unsigned-carry
  overflow trap is a bug, not documented behavior.
- Opcodes 0x27 and 0x2F are Undocumented, in the same class as the four
  opcodes Appendix C already covered (count stays 14).
- The exception vector table has 256 entries (96 reserved, 160 user).

## Implementation bugs found (not fixed here)

23 items in `implementation-bugs.md`, for a separate Rust pass. The
seven most consequential:

1. The decode stage advances the program counter before a privilege fault's
   return address is captured, so five fault types that should be retryable
   currently save the wrong address. Fix: don't advance `pl` in the same
   step a decode-time fault is raised (F-exc-3).
2. `HI`/`LO`'s compound C-and-Z predicates in `definitions.rs` were copied
   directly from ARM without re-deriving them for this CPU's inverted
   (borrow-on-subtract) Carry polarity; flip only the `Carry` term in each
   (exact diff in `implementation-bugs.md`, F-data-1). `CS`/`CC` need no
   code change.
3. The address-overflow trap fires on unsigned carry instead of signed
   overflow, so negative displacements fault when they should not.
4. `ASR` fills vacated bits with zero instead of sign-extending all of them
   for any shift count above 1; confirmed with a worked numeric example
   (F-enc-18).
5. The register file needs renumbering (`r1`-`r7` to `0x0`-`0x6`, `sr` to
   `0x7`) so `NOOP`'s all-zero encoding is genuinely unprivileged without
   losing the erased-memory-is-safe property.
6. `NOOP` currently raises a privilege-violation fault in protected mode
   because it decodes as a write to `sr` under the old register numbering
   — resolved by item 5, not by changing what `NOOP` assembles to.
7. `EXCP` with a vector below `0x60` panics instead of raising a fault,
   because the guard compares a doubled vector address against the
   raw vector-number threshold.

Also flagged: register-offset `LOAD`'s shift suffix is dead code (computed
on a discarded operand, no effect) — resolved in the manual by removing the
syntax, so no code fix is required unless the author later prefers to wire
it up instead; three vector-count comments in `examples/*.sasm` sources
that still say "128" against the real 256-entry table; one example that
teaches a nonexistent status-register field; and one comment with a wrong
stack-pointer value — none of the `examples/` items are manual content, so
they're listed for a separate fix to the example sources.

## Backlog (Workstream 14, in `manual-handover.md`)

13 period-benchmark gaps the author accepted as new content, not editing:
a preface, a manual-wide notation table, per-instruction bit-diagrams,
a condition-code computation table, programmer's-model figures, an
opcode-order format appendix, a proper exception vector table, per-mode
encoding boxes in chapter 8, and an alphabetical index — plus the two
template-field gaps (bit diagrams, legal-forms tables) the author declined
to add to every entry this pass.

## Deferred (author's call, left as-is)

7 blocker/major-adjacent items where the manual and the implementation
still disagree and the author chose not to rule this pass: the four
electrical/timing figures in chapter 2 (now Appendix F), the ARM6/R2000 CPI
comparison row in the timing appendix, and two low-confidence findings
about supervisor-mode coprocessor-1 operations 0xE/0xF.

## Pages needing a visual check

p.110 (chapter 12 instruction list), p.169 (chapter 16 common-semantics
table), and the two big single-page tables in Appendix A and Appendix E —
all four still overflow their page and need the `longtable` conversion
noted above before this manual is print-ready.

## Next steps

- `git diff main` on this branch is ready for review; all commits are
  unsigned (`--no-gpg-sign`) per your request — rebase to sign before
  merging if that matters to you.
- `review/implementation-bugs.md` needs a separate pass on the Rust
  (`sirc-vm`), independent of this manual work. 23 items, of which 7 are
  worth doing first: the fault-return-address timing bug, the `HI`/`LO`
  predicate fix, the address-overflow trap polarity, the `ASR` sign-fill
  bug, the register renumbering (and its one hardcoded string table), and
  `EXCP`'s vector-threshold guard.
- The register renumbering is the one item with real migration cost:
  every existing assembled binary needs reassembling once it lands.
- The 4 remaining `longtable` conversions and the Workstream 14 backlog are
  natural candidates for a follow-up `/manual-edit` run once you're ready.
