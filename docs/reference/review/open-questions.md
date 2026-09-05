# Open questions raised by the chapter editors

Collected from the `## Open questions` section of every `changes-*.md` after Phase 3.

## 01

1. The Execution Model section (lines ~82–88) still frames DMA as "the main exception" to the
   fixed 6-cycle model. Per finding F-arch-2 (filed against chapter 2, not this chapter), every
   coprocessor call — not just DMA — costs a second six-cycle dispatch slot, and DMA additionally
   holds instruction fetch until the transfer completes. Since that finding's `file` field points
   at chapter 2 and its status/resolution weren't given to this editor to act on, I left chapter
   1's wording as-is. If the chapter-2 editor rewords the equivalent passage, chapter 1's
   "DMA commands are the main exception..." sentences (here and in the RISC Principles bullet at
   line 25) will need the same correction for consistency — flagging for the user/coordinator to
   route to whichever editor owns the cross-chapter consistency pass.
2. Table row "Reserved/model use" (coprocessor table) uses different wording from Chapter 16's
   equivalent row ("Reserved/model space"). Not a triaged finding and not clearly wrong, so left
   alone; flagging in case the user wants the two tables to use identical wording.

## 02

- F-arch-9: should the whole manual standardize on 0-indexed execution phases (matching Chapter
  6 and the implementation), including this chapter's `enumerate` list and the "Stage 4" mention
  in §Parallel Functional Units? This chapter's fix only removed the ambiguous bare number in
  one table cell; a full renumbering is a cross-chapter decision.
- F-arch-10 (deferred): the 24 MHz max clock rate and 5 V ±5% supply figures have no stated
  basis anywhere in the corpus. Left as-is per the defer status, but flagging again per the
  original finding in case the author's hardware-figure check is still pending.

## 03

- The Register Encoding table's `sr` row now holds a full sentence ("Reads: high byte
  redacted. Writes: fully privileged") in a centered (`c`) column alongside single-word "Yes"/
  "No" entries elsewhere in the same column. This is content-correct per F-arch-3 but may
  render as an overly wide table cell; a table-layout pass (e.g., a `p{}` column) may be
  wanted at typeset time. Left as prose text and did not change the column type, since that is
  a formatting decision outside this pass's scope.
- F-arch-15's fix offered two options: delete the Chapter 3 subsection, or narrow Chapter 1's
  scope statement. Chose deletion since this editor owns only Chapter 3; flagging in case the
  user prefers the Chapter 1 editor to instead narrow the scope sentence there (in which case
  this chapter's replacement text may want revisiting for redundancy).

## 04

- F-data-3's fix assumes the manual will formally define `system_ram_offset`
  in Chapter 6 (per F-con-13, whose resolution is an either/or left to the
  Chapter 6 editor: define the term, or delete it and use flat addresses).
  If the Chapter 6 editor instead deletes `system_ram_offset` and reverts to
  flat `V * 2` addressing, this chapter's wording will need to be revisited
  to match. Flagging for the user/coordinating editor to confirm consistency
  once Chapter 6 lands.

## 05

- None. All triaged findings for this chapter had unambiguous `accepted` resolutions or a
  Gate 2 ruling to follow, and no remaining ambiguity required a technical decision beyond
  the manual's existing text.

## 06

- The Exception Quick Reference table's Segment Overflow row now reads "Faulting instr. (PC
  wrap only), else next" to reflect that the same fault vector has two different
  return-address behaviors depending on cause. This is a faithful reading of Gate 2 ruling A
  ("Only Alignment and PC-wrap Segment Overflow ... are retryable"), but the ruling does not
  explicitly confirm that a *non*-PC-wrap Segment Overflow belongs in the "next instruction"
  group rather than being unspecified. If that's wrong, the fix is confined to this row and
  the corresponding sentence in the Segment Overflow Fault description and the Fault
  Categories list.

## 07

- None. All ambiguities encountered (opcode 0x2F's true behavior, LDEA/LDEL field layout) were
  resolved by the Gate 2 rulings or by the findings' own evidence; no open technical questions
  remain for this chapter.

## 08

### Addressing-mode count

Per Gate 2 Ruling N, the count and list below were derived from the ten operand forms the
assembler parses (`sirc-vm/toolchain/src/parsers/instruction.rs:85-106`, `AddressingMode` enum)
and cross-checked against `docs/reference/review/facts.md` (Open question 11 / line 206, 343).

The ten parser operand forms are: `Immediate`, `DirectRegister`, `DirectAddressRegister`,
`IndirectRegisterDisplacement`, `IndirectImmediateDisplacement`,
`IndirectRegisterDisplacementPostIncrement`, `IndirectImmediateDisplacementPostIncrement`,
`IndirectRegisterDisplacementPreDecrement`, `IndirectImmediateDisplacementPreDecrement`, and
`ShiftDefinition`.

**Final count: 7 addressing modes.**

| # | Mode                    | Parser form(s) it covers |
|---|-------------------------|---------------------------|
| 1 | Immediate               | `Immediate` (covers both the full 16-bit form and the 8-bit shift-qualified "short immediate" form; the assembler uses one parser rule for both, the width is an encoding choice made by the specific opcode, not a distinct syntax) |
| 2 | Register Direct         | `DirectRegister` |
| 3 | Address Register Direct | `DirectAddressRegister` (previously missing from the chapter's list; used by `LJMP`/`LJSR` src and `LDEA`/`LDEL` dest) |
| 4 | Indirect Immediate      | `IndirectImmediateDisplacement` |
| 5 | Indirect Register       | `IndirectRegisterDisplacement` |
| 6 | Post-Increment          | `IndirectImmediateDisplacementPostIncrement`, `IndirectRegisterDisplacementPostIncrement` (kept as one mode, matching the chapter's existing single "Post-Increment Addressing" section, which already documents both the immediate- and register-displacement syntax together) |
| 7 | Pre-Decrement           | `IndirectImmediateDisplacementPreDecrement`, `IndirectRegisterDisplacementPreDecrement` (same reasoning as Post-Increment) |

`ShiftDefinition` (the parser's tenth form) is **not counted as an addressing mode**. It is a
modifier attached to a register operand already selected by one of the seven modes above (per
Ruling O, the shift applies to the register operand, e.g. `ADDI r1, #10, LSL #2` is
`(r1 << 2) + 10`); it does not by itself select an operand's location, so grouping it as an
eighth mode would conflate "how is the operand found" with "how is it transformed afterward."

This keeps the total at seven, matching the existing chapter-1/handover claim of seven modes, so
no count change is strictly required elsewhere — but the *composition* of the seven changed:
"Short Immediate" is no longer counted as its own mode (it is now documented as a width variant
of Immediate) and "Address Register Direct" was added in its place. Chapters 1 and 11 should be
checked in Phase 4 against this specific list (not just the number seven), since they may still
name "Short Immediate" as the seventh mode instead of "Address Register Direct."

## 09

- None requiring a technical decision beyond what Gate 2 rulings E, F, G, and O already
  resolved. If the maintainers want the `\subsection{Examples}` sub-blocks under "Status Flag
  Update Control" (Default Behavior / Shift Flag Override / No Flag Update / Testing Shift
  Results / Preserving Flags / SHFT Meta-Instruction) folded into the `Example 9-N:` numbering
  scheme too, that's a structural decision (renaming/merging subsection headings) left for the
  user rather than guessed at here.

## 10

- None. The chapter's remaining HI/LO/CS/CC unsigned-comparison language is intentionally left as-is per Gate 2
  ruling B; it is not an open question, just a documented skip.

## 11

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

## 12

None.

## 13

1. Should `Table~\ref{tab:alu-legal-forms}` gain the equivalent `MNEMR rD, rS2[, shift]` /
   `MNEMR rS1, rS2[, shift]` shorthand+shift rows for ADC, SUB, SBC, AND, ORR, XOR, CMP, TSA,
   and TSX, matching the one row F-alu-9 added for ADD? The underlying encoding behaviour is
   identical across all register-form ALU ops.
2. F-con-16 is accepted but its fix text edits `chapters/11-reading-instructions.tex:97`,
   which belongs to another chapter's editor. Chapter 13's own wording already conforms to
   Gate 2 ruling C and does not need to change on this side; please route the Chapter 11 edit
   to that file's owner.

## 14

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

## 15

- **F-mem-18**: as implemented, any `LDEA` whose destination pair is `l` (register field 0)
  raises a privilege-violation fault in protected mode before write-back, regardless of the high
  words — this is a rule the chapter does not currently state, and no test exercises
  `LDEA l, (...)` in protected mode. Either the decode-time privilege check needs to exclude the
  effective-address opcode class (0x18-0x1F), in which case the current chapter text is correct
  and the emulator has a bug, or the manual needs an added sentence "`LDEA` with destination `l`
  is privileged." I did not guess; the Common Semantics `Privilege` row and the LDEA entry's
  `Privilege` field are unchanged pending a decision.

## 16

- None raised by this pass beyond what the rulings already resolved. The one substantive
  ambiguity in this chapter (whether general-purpose/address registers hold "unchanged" or
  "undefined" values after power-on reset, flagged in F-cop-17) was already resolved in the
  applied text as "not affected by reset and hold undefined values after power-on," which the
  finding itself flagged as a decision the architecture had not yet recorded — flagging here in
  case the user wants to confirm that resolution explicitly.

## 17

None.

## appendix-a

- Should the "Bit 3 = 1: Test only, discard result" bullet (line 115) be corrected here to
  exclude \opcode{0F}/\opcode{2F}/\opcode{3F} (coprocessor calls, which discard both result and
  flags rather than "testing"), matching the fix already asked for at
  `chapters/12-instruction-summary.tex` (F-sum-5)? I left it alone because F-sum-5's `file:`
  field points at chapter 12, but the identical defect lives in this appendix too and nobody
  else has been told to fix this half.
- Should the main opcode-map table's mnemonic column show `--` instead of `LOAD`/`COPI` for
  \opcode{27}/\opcode{2F}, to make the row visually match the other "same class" undocumented
  rows per Gate 2 ruling H? I preserved the existing `LOAD`/`COPI` values because they reflect
  a real, tested internal operation and the distinction ("no public assembly syntax" vs. "no
  defined operation") is already captured by the Quick Lookup Table's footnote — but this is a
  judgment call the author may want to confirm.

## appendix-b

- None. Every applicable, accepted finding for this file was already correctly applied by the
  interrupted run; I found no discrepancy requiring a judgment call.

## appendix-c

- None requiring author input. The two same-target finding pairs above (F-con-30/F-exc-21 and
  F-exc-26/F-con-47) were resolved editorially because both members of each pair agreed on the
  underlying fact and differed only in presentation, not in substance.

## appendix-d

- F-sum-31, F-sum-32, and F-sum-34 are marked `accepted` in triage but their fixes all live
  in `.sasm` files under `examples/`, which this pass is barred from touching. Someone with
  write access to `examples/` (or a future pass explicitly scoped to that directory) still
  needs to apply them; flagging so they aren't silently dropped.
- F-sum-29's fix text refers to "six listings D-1 through D-6," but the chapter has seven
  `\lstinputlisting` blocks. I numbered/left the existing D-1 through D-7 sequence as-is
  since it's the more literal application of the underlying "number every example" rule;
  confirming this reading would be useful if the discrepancy was intentional (e.g., two
  listings meant to share one number).

## appendix-e

- None. Every conflict between duplicate/overlapping findings targeting
  this file was resolved above by preferring the more code-grounded or
  more internally consistent wording; none required a technical decision
  only the user could make.

Braces and `\begin`/`\end` counts verified balanced (8/8) after edits.
