# Changes — chapters/08-addressing-modes.tex

## Findings applied

- **F-enc-9** (127–129 orig.) — Corrected the PC-relative addressing-value claim: `\reg{p}`
  supplies the address of the currently executing instruction, not the next instruction's
  address. Added the displacement-0/displacement-2 clarification and a note that
  `\mnemonic{BRSR}`/`\mnemonic{LDEL}` still link to `\reg{pl} + 2`, per the fix text.
- **F-enc-10** (6–26 orig.) + **Ruling N** — Rewrote the addressing-mode list and derived the
  final count/list from the ten assembler operand forms; see `## Open questions` below for the
  full derivation. Added a new operand-form-to-mode mapping table
  (`tab:addr-mode-forms`) and a new "Address Register Direct Addressing" section
  (Description/Syntax/Effective Value/Usage) matching the depth of the other mode sections.
  Updated the Addressing Mode Matrix table to add the Address Register Direct row and to fold
  the old separate Short Immediate row into the Immediate row as a width variant.
- **F-enc-11** (186–197 orig.) + **Ruling D** — Added one paragraph to Indirect Immediate
  Addressing stating that offsets are signed two's-complement and that the Segment Overflow
  Fault (SR.A) fires only on a low-word wrap, not merely because the offset is negative. Did
  not document the emulator's unsigned-carry behavior, per the ruling.
- **F-enc-12** (376 orig.) — Changed `loop:` to `:loop` in the Array Iteration example to match
  the assembler's label-definition syntax (leading colon).
- **F-enc-13** and **F-con-41** (both target line 66, same fix) — Added
  `\texttt{LJSR (rO, src)+}` to the Control-flow meta-instructions row's legal operand forms in
  the "Legal Modes by Instruction Family" table. One edit satisfies both findings.
- **F-enc-14** (247–249, 280 orig.) — Corrected both the Post-Increment and Pre-Decrement
  Description paragraphs to say the *low word* of the address-register pair is updated (not the
  whole pair), and that Post-Increment covers both indirect-immediate and indirect-register
  displacement forms.
- **F-enc-15** (411–414 orig.) — Corrected the Post-increment/Pre-decrement privilege-mode
  bullet: the auto-update is always allowed in protected mode because it only ever touches the
  low word.
- **G1-T4-08** (entry-template, Example numbering) — The chapter has no `Example:` fields inside
  instruction entries (that template applies to Chapters 13–17); the closest faithful
  application is the "Addressing Mode Examples" section's three worked examples, which were
  renumbered `Example 8-1`, `Example 8-2`, `Example 8-3` in their subsection titles.

## Could not apply as written

- **G1-T4-08** — see note above: applied to the nearest equivalent content (the three worked
  examples) rather than to instruction-entry `Example:` fields, since this chapter has none.
- All other findings for this file were applied close to the wording given in `triaged.md`; no
  other deviations were needed.

## Copy edits (not tied to a finding)

- Fixed a duplicated-word typo in Immediate Addressing: "a constant value 16-bit value" →
  "a constant 16-bit value".
- Reworded the Short Immediate shift sentence for precision/terminology consistency with Ruling
  O ("The shift applies to the register operand, not to the immediate constant.") and added a
  cross-reference clarifying it is a width variant, not a separate mode.
- No hex literals, `e.g.`, or other STYLE.md mechanical-pass violations were found in this
  chapter (checked with grep for `e\.g\.`, `i\.e\.`, and `0x`).

## Open questions

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

### Other open questions

None beyond the above. All other findings were unambiguous and applied as specified.
