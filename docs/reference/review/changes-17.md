# Changes -- chapters/17-meta-instructions.tex

## Findings applied

- **F-cop-1** (accepted, blocker): `NOOP` now assembles to `ADDI[N] sr, #0` (was
  wrongly `ADDI[N] r1, #0`). This is the only encoding that produces the all-zero
  instruction word `0x00000000`.
- **F-cop-2** (accepted, blocker): Rewrote the Operation line to describe a
  destination of `sr`, the write-back path preserving the `EA` bit, and no
  condition-flag update (status-register update source `None`). Applied as
  written; added the word "occur" to close a dangling clause ("...and no
  architectural state changes occur"), a grammar fix only, no meaning change.
- **F-cop-25** (accepted, minor): Added a `NOOP` row to the meta-instruction
  cross-reference table, citing this chapter as primary documentation and
  `ADDI[N] sr, #0` as the target encoding. Placed first, matching the order
  already used for the equivalent table in Chapter 12.
- **G1-T1-17** (accepted): Renamed `Condition codes:` to `Condition field:` in the
  `NOOP` entry.
- **G1-T2-17** (accepted): Added `Applicability: SIRC-1, all revisions` directly
  under the `NOOP` entry title.
- **G1-T3-17** (accepted): Added an Instruction Fields list after `Assembles to:`,
  covering the Register, Immediate, AF, and Condition field values, sourced from
  Chapter 7's Immediate Format description and `facts.md` (register field 0 = `sr`,
  immediate 0, AF = `00`/None).
- **G1-T4-17** (accepted): Numbered the entry's example `Example 17-1:`.

## Findings skipped (and why)

- **F-cop-3** (status: `code-wrong`): Per Gate 2 ruling I, `NOOP` is unprivileged
  and must not be described as faulting in protected mode. The manual's existing
  `Privilege: Available in protected mode and supervisor mode.` line already
  reflects this and required no change; the finding itself is not applied since
  its status is not `accepted`.
- **F-con-38** (status: `rejected`): Superseded by F-cop-25 (which correctly lowers
  `NOOP` to `ADDI[N] sr, #0` rather than the `r1` form this finding proposed). Left
  alone per the ruling.

No other findings in `triaged.md` target this file.

## Open questions

None. Ruling I and F-cop-2's fix text fully resolve the only ambiguity in this
chapter (the `NOOP`/`sr` privilege question); the `Privilege` line required no
edit under that ruling.

## Phase 4

- **F-con2-31** (accepted, minor): Wrapped the `NOOP` entry in
  `\begin{instructionbox}{NOOP -- No Operation}` / `\end{instructionbox}` so it
  renders like the other 32 instruction entries in the manual. Dropped the
  standalone `\section{NOOP -- No Operation}` heading and used the same text as
  the `instructionbox` title, per the fix's second option ("or dropping it as
  the other chapters do"), matching the convention in Chapters 13--16. All
  existing fields (Applicability, Assembles to, Instruction Fields, Syntax,
  Operands, Operation, Description, Write-back, Flags, Exceptions, Condition
  field, Timing, Privilege, Example) were preserved unchanged, as was Gate 2
  ruling I's unprivileged/`ADDI[N] sr, #0` framing. No copy-editing beyond the
  wrap was performed.

Open questions: none.
