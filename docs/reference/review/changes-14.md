# Changes — chapters/14-memory-instructions.tex

## Findings applied

- **F-con-4** (Gate 2 ruling G) — The register-offset shift for `LOAD` applies to the offset
  register `rO` before the effective-address calculation, never to the value read from memory.
  Rewrote the Overview "Important" note, the LOAD Operands/Operation/Syntax text, the LOAD
  Examples, and the whole "Shift Operations with Memory Instructions" section (LOAD-with-shift
  subsection, the "Use Cases" examples, all of which previously showed the loaded *value* being
  shifted). `STOR`'s shift (applies to the source register before the store) was already
  correct per Chapter 9 and was left unchanged.
- **F-con-8 / F-mem-11** — Standardized the register-displacement placeholder name to `rO`
  everywhere in this chapter (Syntax blocks, Operands paragraphs, and comments for both
  `LOAD` and `STOR`); the Legal Forms table already used `rO`.
- **F-mem-1 / F-con-23** — Replaced the "Documented memory forms do not raise
  privilege-violation ... faults" claim with the correct rule: a memory instruction raises a
  privilege-violation fault in protected mode when its register field names a privileged
  register (`sr`, `ah`, `lh`, `ph`, `sh`), applying to the `LOAD` destination and the `STOR`
  source. Added that post-increment/pre-decrement auto-update never changes the high word and
  so never itself raises a privilege-violation fault. Added matching qualification to the
  `Privilege:` and `Exceptions:` fields of both entries.
- **F-con-12 / F-mem-3** — Corrected the effective-address equation to
  `EA = AddressRegister.high : (AddressRegister.low + Displacement)`, with prose stating the
  high word is unchanged and only the low word participates (mod 2^16).
- **F-con-22 / F-mem-10** — Rewrote the Overview's addressing-forms lead-in: both instructions
  share indirect immediate/register-displacement forms; only `LOAD` has post-increment and only
  `STOR` has pre-decrement.
- **F-mem-2** — Replaced "the status override field is ignored" with the correct statement that
  the AF field (bits 5–4) selects the address-register pair for these opcodes and is not a
  status-update source, with a cross-reference to Chapter 13 for the LOAD-immediate/register
  flag behavior it does not cover.
- **F-mem-4** — Added an "Aliased register writes" paragraph to the Legal Forms section: the
  `LOAD` post-increment destination and the `STOR` pre-decrement source must not be either half
  of the auto-updated address-register pair; the assembler rejects such encodings.
- **F-mem-6** (Gate 2 ruling D) — Stated that the 16-bit displacement is signed two's-complement
  and that the SR.A trap fires only on a wrapping low-word calculation; did not document the
  emulator's unsigned-carry behavior.
- **F-mem-7** (Gate 2 ruling A) — Added that a memory instruction aborted by any of its faults is
  not restartable (the saved return address is the next instruction's), since the
  segment-overflow fault this chapter's instructions can raise is not the PC-wraparound case and
  so is not retryable.
- **F-mem-8** (Gate 2 ruling C) — Relabeled the Status Flag Effects table rows
  "LOAD (0x14–0x17)" / "STOR (0x10–0x13)" and added a cross-reference to Chapter 13 for the
  LOAD-immediate (0x07) / LOAD-register (0x37) flag behavior, which this table does not cover.
- **F-mem-9** — Changed the `loop:` label to `:loop` (SIRC-1 label syntax is a leading colon).
- **G1-T1-14** — Renamed `Condition codes:` to `Condition field:` in both entries.
- **G1-T2-14** — Added `SIRC-1, all revisions` under both entry titles.
- **G1-T3-14** — Added an `Instruction fields:` list to both entries (register/immediate/shift/
  AF/condition fields and their legal values), sourced from `facts.md` and Chapter 7.
- **G1-T4-14** — Numbered the two `Examples:` blocks `Example 14-1:` (LOAD) and
  `Example 14-2:` (STOR).

## Findings skipped

- **F-mem-5** — status `code-wrong`; per instructions, left alone (the Gate 2 D ruling text is
  already folded into the F-mem-6 fix above, which is the accepted half of the same issue).
- All other statuses (`defer`, `not-implemented`, `phase4`, `rejected`) — none applied to this
  chapter; none were found with those statuses for `chapters/14-memory-instructions.tex` besides
  F-mem-5.

## Copy edit (pass two)

- Applied the Gate 1 hex-literal macro pass within this chapter: `\opcode{XX}` for the opcode
  values in the Legal Forms table's Encoding column, both entries' `Opcodes:` lines, and the
  Status Flag Effects table row labels.
- Removed a second-person "you" in the closing Shift Operations note (STYLE.md: no second
  person).
- Minor terminology tightening tied directly to the findings above (`rO`, "signed
  displacement", "not a status-update source") — no unrelated rewrites.

## Open questions

- The "Shift Limitations" bullet list (`LOAD r1, \#0x1234`, `LOAD r1, (\#0, a)`, etc.) still
  uses bare `\texttt{}` code snippets with literal hex/register content rather than the
  `\opcode`/`\imm`/`\reg` macros. This matches the surrounding style of quick inline syntax
  examples throughout the chapter and was left as-is rather than converting only this one list;
  flagging for the user in case a manual-wide sweep should also touch these short inline
  snippets.
- F-mem-8's Gate 2 ruling C also directs that "any explicit AF suffix on LOAD leaves all four
  flags undefined" be documented once in Chapter 5 and Chapter 13. This chapter only
  cross-references that material (per the ruling's own instruction to "cross-reference from
  ch12 and ch14"); no explanation was duplicated here. Please confirm Chapter 13's editor has
  applied the corresponding fix so the cross-reference resolves to correct content.

## Phase 4

Continuity re-check fixes from `docs/reference/review/consistency-2.md`.

### Findings applied
- F-con2-11 (major): Replaced `\textit{SIRC-1, all revisions}` with
  `\textbf{Applicability:} SIRC-1, all revisions` and `\textbf{Instruction fields:}` with
  `\textbf{Instruction Fields:}` in both the LOAD and STOR entries, matching the template form
  used in Chapters 13, 15, 16, and 17.
- F-con2-14 (minor): Swapped the `Write-back:` and `Flags:` blocks in both entries so `Flags`
  precedes `Write-back`, per the canonical STYLE.md §4 order. (Ignored the Chapter 15 sites
  named in this finding; that chapter has its own editor.)
- F-con2-15 (minor): Swapped the `Timing:` and `Condition field:` blocks in both entries so
  `Condition field` precedes `Timing`, per the canonical order.
- F-con2-18 (major): Changed the SHFT cross-reference in the Status Flags note from
  `Chapter~\ref{ch:meta-instructions}` to `Chapter~\ref{ch:alu-instructions}`, since SHFT is
  documented in the ALU chapter.
- F-con2-38 (nit): Changed the ASL shift-type bullet from "(same as LSL)" to "(same data
  result as LSL; sets V on signed overflow)" to restore the one behavioral difference.

All five fixes applied exactly as specified in the `fix` lines; no deviations were needed.

### Findings not applied as written
None.

### Open questions
None new for this phase.
