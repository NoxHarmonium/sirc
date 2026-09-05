# Changes — chapters/11-reading-instructions.tex

## Findings applied

- **F-con-28** / **F-sum-21**: Rewrote the "Instruction Entry Fields" description list into
  the canonical STYLE.md order and naming (Opcodes → Instruction Fields → Syntax → Operands →
  Operation → Description → Flags → Write-back → Exceptions → Condition field → Timing →
  Privilege → Example/Examples → Notes). Added the previously-missing `Description`, `Example
  / Examples`, and `Notes` items. Relabeled `Status Flags` to `Flags` and `Condition Codes` to
  `Condition field`.
- **G1-T1** (rename predicated-execution line): folded into the same edit — the item is now
  `\item[Condition field]`, described as the set of condition-field values the instruction
  accepts for predicated execution, distinct from `Flags`.
- **G1-T2** (applicability line): added a paragraph before the field list stating that every
  entry title is followed by one applicability line (`SIRC-1, all revisions` for CPU
  instructions, or the required coprocessor/revision, per Chapter 16's model and revision
  policy, for coprocessor instructions).
- **G1-T3** (Instruction Fields list): added a new `\item[Instruction Fields]` immediately
  after `Opcodes`, describing the per-encoding-field line (register fields, condition field,
  status-update-source field with legal encodings, shift type/count, immediate width), sourced
  from Chapter 7 and the facts digest.
- **G1-T4** (`Example N-M:` numbering): described the numbering rule inside the new
  `Example / Examples` item so Chapters 13–17 have the rule to point at. Chapter 11 itself has
  no worked examples to renumber.
- **F-sum-10** (shift notation, Ruling O): rewrote the `shift` row in the notation table to
  state that the shift applies to the register operand being shifted (first source register,
  or the offset register for register-displacement LOAD/STOR), never to an immediate, a
  second source register, or loaded data, and pointed to Chapter 9 for the full encoding.
- **F-sum-8** (SHFT default AF): appended "The one exception is `SHFT`, which defaults to
  `[S]`." to the Status Update Overrides paragraph.
- **F-sum-9** (undefined placeholders): added notation-table rows for `addr / src`, `dest`,
  `#offset / #disp`, `@label`, and `|cond`.
- **F-con-59** / **F-sum-20** / **G1-T5-11** (all three request the same fix): added the `U`
  row to the Status Flag Effect Symbols table.

## Findings skipped

- **F-sum-11** — status `phase4`. Left the `Operands` field description as-is (does not name a
  closed set of addressing-mode names); this is explicitly deferred until the Chapter 8 editor
  settles the addressing-mode count (Gate 2 ruling N).

## Applied with a deviation

- **F-sum-9**: the proposed fix names the new notation rows `A`/`src` and `B`/`dest`, but `A`
  and `B` do not appear anywhere else in the manual as address-register-pair placeholders (a
  grep across all chapters found zero uses), and STYLE.md's terminology table reserves bare
  upper-case `R1`/`R2`/`R3` for a different, already-defined purpose (abstract field-position
  labels in Chapter 7). Rather than introduce two new single-letter symbols with no basis in
  the actual entries, the added rows use the placeholder spellings that Chapters 14 and 15
  actually use (`addr`, `src`, `dest`, `#offset`, `#disp`, `@label`, `|cond`), which is closer
  to the finding's underlying goal — making the notation table define every placeholder the
  downstream chapters use.
- Widened the notation table's second column from a plain `l` to `p{10cm}` so the longer
  definitions added by F-sum-9 and F-sum-10 wrap instead of running off the page. This is a
  column-width change only; row content, order, and all labels are unchanged. Verified the
  chapter still compiles cleanly in a full `latexmk` build (page 101–102 of the output, no
  errors attributed to this file).

## Open questions

- F-sum-9's fix text also says "Alternatively pick one spelling and make Chapters 14 and 15
  use it." That is a cross-chapter consistency decision (renaming `addr`/`src`/`dest` to a
  single shared name) that touches files I do not own; I left it as an open question for the
  user/author rather than unilaterally renaming placeholders in Chapters 14–15.
- G1-T3's Instruction Fields item and G1-T2's Applicability line are now documented here, but
  no entry in Chapters 13–17 implements them yet (confirmed against `chapters/13-alu-instructions.tex`,
  which still lacks both). That rollout is explicitly out of scope for this file per the task,
  but the user should confirm the Chapter 16 cross-reference wording ("per the model and
  revision policy in Chapter~\ref{ch:coprocessor}") reads correctly once Chapter 16's own
  editing pass is done, since that subsection currently has no `\label` of its own to target
  more precisely.
