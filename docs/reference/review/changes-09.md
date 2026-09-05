# Change log: chapters/09-shift-operations.tex

## Findings applied

- **F-con-18** — Row `000` in `tab:shift-encoding` no longer invents the mnemonic `NUL`; changed
  to `-- / -- / No shift performed`, matching Chapter 7's `tab:shift-types`. Kept the table
  (did not delete it, per the "do not delete tables" instruction), which was one of the two
  options the finding offered.
- **F-con-5** — ASL is now documented as producing the *same data result* as LSL but setting V
  on signed overflow, at the shift-type table (line ~30), the ASL Operation line, and left the
  Overflow Flag section as-is (it already stated the correct rule; only the contradicting text
  needed to change).
- **F-enc-16** / **F-con-26** — Fixed the `ADDI r1, #1, LSL #2` example: removed the phantom
  `r2` operand, corrected the arithmetic to `(r1 << 2) + 1`, and reworded the section comment.
- **F-con-27** / **F-enc-23** — Rewrote the "Set bit 10" `ORRI` example, which didn't actually
  set bit 10 under the real shift semantics; replaced with a 16-bit immediate form that does.
  Both findings targeted the same two lines with compatible fixes; applied once.
- **F-enc-19** / **F-con-68** — Renamed "shift postamble" to "shift suffix" throughout, and
  restated which instruction forms accept a shift suffix (ALU short-immediate/register, SHFT,
  register-displacement LOAD/STOR) versus which reject it (`LOAD rD, rS`, LDEA/LDEL,
  coprocessor instructions).
- **F-enc-20** — Split the "where is the shift applied" rule by instruction format (see Open
  Questions/Skipped for the part of this finding not applied as written).
- **F-con-55** — `tab:shift-operand` Description cells now name both the Register-format (R2)
  and Short-Immediate-format (Reg field) location of the shifted operand.
- **F-enc-21** (Gate 2 ruling E) — Restricted "4-bit field, max 15" to the immediate-count case
  and stated that a register-supplied count above 15 is architecturally undefined, in both the
  Shift Count section and the Limitations section (lines ~439-440 in the pre-edit file).
- **F-enc-17** — Reworded the shift-count-0 carry-flag bullet: a count of 0 leaves the operand
  unchanged and clears carry; a shifter-sourced status update writes all four flags, so use
  `[N]` to preserve flags across a shift.
- **F-enc-22** — Removed the illegal/misleading `LOAD[N]` from the "Preserving Flags" example;
  replaced with plain `LOAD` and a comment stating LOAD never updates flags (see Open Questions
  for why the finding's proposed extra sentence was not added).
- **F-con-52** — ASR's sign-bit bullet now says the sign bit is replicated into *all* vacated
  bits (Gate 2 ruling F: ASR sign-fills every vacated bit; the manual's general statement was
  already consistent with this, only the single-bit-copy bullet was wrong).
- **F-con-53** (Gate 2 ruling: apply as proposed) — Both `SHFT[N] r2, LSR #1` example lines
  (32-bit unsigned and signed right-shift examples) changed to `ORRI[N] r2, #0, LSR #1`, since
  `SHFT` already implies `[S]` and cannot also carry `[N]`.
- **F-enc-24** — Rewrote the LSL, LSR, and ASR carry-flag bullets to state the general rule
  (last bit shifted out of bit 15/0, i.e. bit `16-n`/`n-1` for a count of `n`) instead of the
  single-position-only wording.
- **G1-T4-09** — Numbered the six `\textbf{Example:}` blocks in the Shift Type Details section
  as `Example 9-1` through `Example 9-6`, in order of appearance (LSL, LSR, ASL, ASR, RTL,
  RTR). The later `\subsection{Examples}` block (Status Flag Update Control) uses named
  sub-headings ("Default Behavior", "Shift Flag Override", "No Flag Update") rather than literal
  `Example:` labels, so it was left as-is; renumbering it would have meant inventing new
  section structure, which is out of scope.

## Findings skipped or applied differently

- **F-enc-18** — status `code-wrong`, not `accepted`; skipped per instructions. (This is the ASR
  vacated-bits finding; Gate 2 ruling F already covers the correct manual wording, applied via
  F-con-52 instead.)
- **F-enc-20** — the finding's fix claimed that in register-displacement LOAD forms "the shift
  is applied to the loaded memory word at write-back." This directly conflicts with Gate 2
  ruling G ("LOAD shift applies to the offset register (source), never to the loaded data";
  ruling explicitly says to correct Chapter 14 to match Chapters 2 and 9, i.e. Chapter 9's
  original phase/operand description was already correct). Applied the finding's operand-naming
  split for ALU register-format vs. short-immediate forms, but for LOAD/STOR wrote instead:
  shift applies to the offset register (address calculation) for LOAD, and to the source
  register before the store for STOR — consistent with ruling G and with the encoding evidence
  cited elsewhere in triaged.md for STOR's R2 field.
- **F-enc-22** — did not add the finding's proposed sentence "the `[A|S|N]` modifier is accepted
  only on ALU mnemonics... `LOAD`, `STOR`, `LDEA`, `LDEL`... never update the status register."
  Gate 2 ruling C states LOAD's default AF is `[N]` and an *explicit* AF suffix on LOAD leaves
  all four flags architecturally undefined (`U U U U`) rather than being rejected outright,
  which contradicts the finding's "accepted only on ALU mnemonics" (hard-rejection) framing.
  Fixed the concrete example (removed the explicit `[N]` on `LOAD`, corrected the comment) but
  left out the broader unqualified claim to avoid restating a rule that a Gate 2 ruling
  contradicts.

## Open questions

- None requiring a technical decision beyond what Gate 2 rulings E, F, G, and O already
  resolved. If the maintainers want the `\subsection{Examples}` sub-blocks under "Status Flag
  Update Control" (Default Behavior / Shift Flag Override / No Flag Update / Testing Shift
  Results / Preserving Flags / SHFT Meta-Instruction) folded into the `Example 9-N:` numbering
  scheme too, that's a structural decision (renaming/merging subsection headings) left for the
  user rather than guessed at here.

## Phase 4

### Findings applied
- **F-con2-16** (chapter-9 half) — Changed the Mnemonic cell for shift code `000` in
  Table~\ref{tab:shift-encoding} from `--` to `NUL`. Added one sentence after the table
  defining `\mnemonic{NUL}` as the no-shift form that still routes the operand through the
  shifter, which is what makes `SHFT rD, NUL #0` a legal way to drive the status flags from
  the unshifted value of `rD`. (Chapter 7's matching table was already fixed by that
  chapter's editor; not touched here.)
- **F-con2-33** — Numbered the ten previously-unnumbered listings in Common Use Cases
  (Multiplication by Constants, Division by Powers of 2, Bit Field Extraction, Bit
  Manipulation), Multi-Word Shifts (32-bit Left Shift, 32-bit Right Shift Unsigned/Signed),
  and the `\subsection{Examples}` block under Status Flag Update Control (Default Behavior,
  Shift Flag Override, No Flag Update) as `Example 9-7` through `Example 9-16`, continuing
  the chapter's existing 9-1..9-6 sequence with no gap or duplicate. This resolves the open
  question left in the Phase 3 log about that `\subsection{Examples}` block. The later
  "Use Cases" sub-subsections (Testing Shift Results, Preserving Flags, SHFT
  Meta-Instruction) were left unnumbered, since they fall outside the line range the
  finding cites and outside "Common Use Cases / Multi-Word Shifts / Status Flag Update
  Control" as scoped.

### Findings not applied as written
- None; both findings applied as written.

### Open questions
- None new. Whether the remaining unnumbered listings under "Use Cases" (Testing Shift
  Results, Preserving Flags, SHFT Meta-Instruction) should also get `Example 9-N:` labels
  is still open, as noted in the Phase 3 log; F-con2-33 did not scope those in.

Verification: `grep -c 'begin{'` and `grep -c 'end{'` both return 51 for this file after
these edits, and a per-environment breakdown (`bytefield`/`figure`/`itemize`/`lstlisting`/
`table`/`tabular`) confirms every pair balances.
