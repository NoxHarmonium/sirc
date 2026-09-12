# SIRC-1 Reference Manual Handover

Date: 2026-06-20
Last updated: 2026-09-12

This handover tracks what's left to bring the SIRC-1 CPU Reference Manual up to the standard of a production CPU
programmer's reference manual, using the Motorola M68000 Family Programmer's Reference Manual as the comparison
point. Completed workstreams (internal consistency, machine-checked encodings, the instruction-reference upgrade,
addressing modes, data representation, exceptions/reset, bus timing, coprocessor compatibility, and quick-reference
appendices) have been trimmed out of this file entirely to keep it focused — see git history for the full record.
Two workstreams (ABI/calling-convention docs and a binary/object-format appendix) were deliberately ruled out of
scope for this manual and are also omitted here.

## Workstream 12: Documentation QA and Build Hygiene

Goal: prevent the manual from drifting away from the implementation.

Open tasks:

- Add a reference-manual QA target: build the PDF, verify generated examples, check references, check
  glossary/terminology, and optionally render selected pages for visual inspection, all from one command.
- Chapter 2's 24 MHz maximum clock rate and 5 V +/- 5% supply figures have no stated basis. Deliberately left
  unresolved for now (author's call, 2026-09-12) rather than fixed as part of this pass.
- Generate more tables from source where possible: opcode map, register map, condition codes, instruction
  encodings, coprocessor IDs.
- Add a "manual source of truth" note: decide whether implementation code, LaTeX tables, or a shared
  machine-readable spec is canonical.
- Add TODO tracking: `TODO(manual)` comments in LaTeX, or track via issues instead. Avoid prose TODOs in the
  rendered manual unless intentionally visible.
- Audit assembly examples by chapter: architecture/reset/bus/exception/memory-model chapters should generally
  prefer pseudocode, diagrams, timing tables, and state-transition descriptions over raw assembly; assembly
  listings should live mainly in the instruction-reference chapters, assembler-facing sections, and the examples
  appendix. Where architecture chapters currently use assembly to illustrate behavior, decide whether to replace
  it with pseudocode, move it to the relevant instruction chapter, or mark it as non-normative example code.

Acceptance criteria:

- Manual examples and tables are tested against implementation data.
- Manual build failures catch broken references and stale generated content.
- The manual distinguishes architectural specification from assembler usage examples.

## Workstream 13: Typesetting and Visual Design

Goal: make the rendered PDF look like a professional CPU reference manual rather than a research paper — readable,
navigable, and visually consistent throughout.

Open task:

- Add a notation and glossary section. Chapter 11 already defines instruction-notation symbols; expand this into
  a short standalone glossary appendix (or a "Notation and Conventions" section in the front matter) covering:
  - architectural terms (supervisor mode, protected mode, fault, exception, meta-instruction, word address)
  - register name conventions (rN, l/a/s/p, sr, lh/ll, etc.)
  - typographic conventions used in the manual (mnemonic style, register style, opcode style, pseudocode style,
    reserved/undefined vocabulary)
    This prevents readers having to hunt through chapters to understand notation.
- Add a front cover with some graphic with abstact shapes and a back cover with a blurb and fake publishing info.

Smaller, not-currently-a-known-problem items, not explicitly revisited during the 2026-09-12 visual-design pass:
a dedicated `lstlisting` restyle, and a review of chapter/part opening pages specifically for visual consistency.

Acceptance criteria:

- Every term used in the manual is defined the first time it appears or is listed in the glossary. Not yet met.
- (Page-overflow, box-styling, font, and color-palette criteria for this workstream are met as of 2026-09-12.)

## Workstream 14: Period Coverage Gaps

Identified by a period benchmark against the M68000 Family Programmer's Reference Manual (1992, `M68000PRM`) and
the MCS6500 Family Programming Manual (1976, `MCS6500`), run 2026-09-05. Accepted as backlog; new content, not
editing. (Two items from the original 11-item list are already substantially satisfied by later work and have
been dropped from this list: the exception vector table now has a full quick-reference table in Chapter 6, and
Appendix E's alphabetical mnemonic index already provides mnemonic-to-page access.)

1. **Instruction Format bit diagram in every entry.** A bit-numbered 16-bit word (both
   words for immediate format) showing opcode, condition, register, AF, shift and immediate
   fields. `M68000PRM:4-4`, `4-25`; `MCS6500:B-3`.
2. **Legal forms table in each entry.** Accepted operand/addressing forms with encodings,
   or a cross-reference line to the chapter "Legal Forms" table. `M68000PRM:4-5`, `4-108`.
3. **Condition-code computation table.** Boolean formulas for V, C, Z per instruction
   family in terms of Sm, Dm, Rm, plus condition-test formulas. `M68000PRM:3-18`, `3-19`.
4. **Preface / About This Manual.** Audience, assumed knowledge, companion documents,
   out-of-scope items, how to read entries, revision status. `MCS6500:p.1-2`;
   `M68000PRM:1-1`.
5. **Manual-wide notational conventions table.** Operators, register names, `#imm`,
   indirection, `SR.X`, assignment, literal prefixes, flag-symbol legend, in one place near
   the front; generalise Chapter 11's table. `M68000PRM:3-2`..`3-4`; `MCS6500:B-2`. (Overlaps
   with Workstream 13's glossary task above — probably worth doing together.)
6. **List of Examples in the front matter.** Follows from the Gate 1 decision to number
   examples `N-M` within each chapter. `MCS6500` front matter.
7. **Instruction format summary in opcode order.** Appendix listing every opcode
   0x00--0x3F with its full 32-bit layout as a bit diagram. `M68000PRM:8-1`..`8-5`;
   `MCS6500:D-1`.
8. **Programming model figures.** One figure of user-visible registers with bit widths,
   one for supervisor additions, and a table of privileged registers/bits.
   `M68000PRM:1-2`, `1-9`, `1-11`.
9. **Per-mode encoding box in Chapter 8.** GENERATION (EA formula), ASSEMBLER SYNTAX,
   field encoding and instruction word count for each addressing mode. `M68000PRM:2-6`,
   `2-7`.

## Definition of Done

The manual is "up to scratch" when, in addition to everything already met by the completed workstreams:

- There's a single repeatable command that builds the PDF, verifies generated examples, and checks
  references/terminology (Workstream 12).
- Every term used in the manual is defined the first time it appears or is listed in a glossary (Workstream 13).

#### REMEMBER TO WORK OUT WHAT IS WRONG WITH THE DIAGRAM AND FIX IT. I SWEAR I LEFT A NOTE SOMEWHERE
