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

**Done (2026-09-12):** notation and glossary section. Added `chapters/preface.tex` (unnumbered front-matter chapter:
About This Manual, Assumed Knowledge, How to Read This Manual, Scope, Revision Status) and
`chapters/notation-and-glossary.tex` (unnumbered front-matter chapter: manual-wide typographic conventions,
normative vocabulary -- must/may/should/reserved/architecturally undefined/implementation-defined -- register name
conventions, and a glossary of architectural terms). This merges what was originally two separate asks (this
workstream's glossary task, and Workstream 14 item 5's notational-conventions table below) into one deliverable, and
absorbed/expanded the old thin "About This Manual"/"Document Conventions" sections that were previously stuck at the
end of `chapters/title.tex`. Placed in `\frontmatter`, after the title page and before the table of contents.

**Done (2026-09-12):** front and back cover. Added `chapters/front-cover.tex` (a TikZ abstract circuit/die-motif
graphic in the manual's red/gray/black palette, plus title text) and `chapters/back-cover.tex` (a marketing-style
blurb, a decorative barcode graphic, and a fake publishing block), wired into `main.tex` as the very first and very
last pages of the document. Reused the existing fictional publishing identity from `chapters/title.tex` (Silicon
Integrated Research Corporation, copyright 1989, Version 1.0) rather than inventing a competing one.

Smaller, not-currently-a-known-problem items, not explicitly revisited during the 2026-09-12 visual-design pass:
a dedicated `lstlisting` restyle, and a review of chapter/part opening pages specifically for visual consistency.

Acceptance criteria:

- Every term used in the manual is defined the first time it appears or is listed in the glossary. Met (2026-09-12).
- (Page-overflow, box-styling, font, and color-palette criteria for this workstream are met as of 2026-09-12.)

## Workstream 14: Period Coverage Gaps

Identified by a period benchmark against the M68000 Family Programmer's Reference Manual (1992, `M68000PRM`) and
the MCS6500 Family Programming Manual (1976, `MCS6500`), run 2026-09-05. Accepted as backlog; new content, not
editing. Three items from the original 11-item list have been dropped since they're already substantially covered:
the exception vector table now has a full quick-reference table in Chapter 6; Appendix E's alphabetical mnemonic
index already provides mnemonic-to-page access; and the opcode-order bit-diagram appendix (item 7, below) was
judged (2026-09-12, author's call) to duplicate Chapter 7's per-format diagrams and Appendix A's opcode table
closely enough not to be worth building separately.

**Done (2026-09-12):** items 4 (Preface) and 5 (manual-wide notational conventions table) -- see Workstream 13
above; both were folded into the new `chapters/preface.tex` and `chapters/notation-and-glossary.tex`.

Open, in original numbering (2, 3, 6, 8, 9 remain; 1 stays open but is now scoped -- see below):

1. **Instruction Format bit diagram in every entry.** A bit-numbered 16-bit word (both words for immediate format)
   showing opcode, condition, register, AF, shift and immediate fields. `M68000PRM:4-4`, `4-25`; `MCS6500:B-3`.
   Scoping decision (2026-09-12, author's call): one diagram per encoding form an entry documents, not one per
   entry overall -- so an entry like `ADDI`/`ADDR` (Immediate, Short-Immediate, and Register forms) gets three
   diagrams. This is roughly 70-90 diagrams total across all 33 instruction entries; most will look near-identical
   within a format family, with only the opcode value differing, but that repetition matches the M68000 PRM's own
   practice and the manual's "flip through and find it" design goal. Not started.
2. **Legal forms table in each entry.** `M68000PRM:4-5`, `4-108`. **Done (2026-09-12):** added a one-line
   cross-reference to the relevant chapter-level Legal Forms table to all 33 instruction entries (appended to each
   entry's `Opcodes:`/`Assembles to:` line), per the scoping decision to reference rather than duplicate.
3. **Condition-code computation table.** `M68000PRM:3-18`, `3-19`. **Done (2026-09-12):** added a "Flag
   Computation Formulas" subsection to Chapter 13's "Status Flag Updates" section with Boolean C/V formulas per
   family in terms of `Dm`/`Sm`/`Rm`, verified by hand-tracing signed overflow and borrow cases.
6. **List of Examples in the front matter.** `MCS6500` front matter. **Done (2026-09-12):** retrofitted all 75
   examples across the manual (68 short inline examples in Chapters 3, 5, 8, 9, 10, 13-17, plus 7 full-program
   listings in Appendix D) into a real numbered/captioned/listed mechanism, matching the existing List of
   Figures/Tables. Two mechanisms were needed:
   - A new `example` float (`\newfloat` in `preamble.tex`, numbered `N-M` like the existing convention, `[H]`
     placement so it never actually floats away) for the ~68 short examples, each given a real caption (mostly
     derived from a pre-existing subsection title or the leading code comment where neither existed yet).
   - Appendix D's 7 example programs are full multi-hundred-line `.sasm` listings (up to 268 lines) that cannot
     fit a non-breaking `[H]` float without massive overflow, so they keep `lstinputlisting`'s own page-breaking
     caption, but now also manually step the shared `example` counter and inject a matching "List of Examples"
     entry via `\addcontentsline{loe}{...}`, so they appear seamlessly alongside the float-based ones in one
     unified list.
   - This incidentally fixed a real numbering bug: Chapter 16's examples were hand-numbered out of file order
     (16-4 appeared before 16-3 in the source); auto-numbering now reflects true document order.
   - Also fixed two stale "this fault is not retryable" references in Chapter 14 (`LOAD`/`STOR` segment-overflow
     exceptions) left over from before the fault-category unification earlier in this session.
8. **Programming model figures.** `M68000PRM:1-2`, `1-9`, `1-11`. **Done (2026-09-12):** added a "Programming
   Model" section to Chapter 3 with a "User Programming Model" figure (`r1`-`r7`, the low word of each address
   register pair, `sr`'s unprivileged low byte) and a "Supervisor Programming Model Supplement" figure (the high
   word of each address register pair, `sr`'s privileged high byte). The privileged-registers table this item also
   asked for was already covered by the existing Register Encoding table (Table 3.1).
9. **Per-mode encoding box in Chapter 8.** `M68000PRM:2-6`, `2-7`. **Done (2026-09-12):** replaced the eight
   per-mode prose subsections (Description/Syntax/Effective Value/Usage) with a new `addrmodebox` environment --
   a compact, non-page-forcing bordered box (deliberately *not* the page-per-entry `instructionbox` pattern, since
   there are only 7 addressing modes and several fit per page) stating GENERATION, ASSEMBLER SYNTAX, FIELD
   ENCODING (with exact bit positions, e.g. Register format's R1/R2/R3 fields, the memory-format address-register
   and offset fields, or which opcode bit selects auto-update), and INSTRUCTION WORD COUNT (always 2, fixed --
   explicitly called out since that's what makes SIRC-1 different from architectures with variable extension
   words per mode). All existing prose content (formulas, syntax examples, usage bullets) was preserved, just
   reorganized.

## Definition of Done

The manual is "up to scratch" when, in addition to everything already met by the completed workstreams:

- There's a single repeatable command that builds the PDF, verifies generated examples, and checks
  references/terminology (Workstream 12).
- Every term used in the manual is defined the first time it appears or is listed in a glossary (Workstream 13).

#### REMEMBER TO WORK OUT WHAT IS WRONG WITH THE DIAGRAM AND FIX IT. I SWEAR I LEFT A NOTE SOMEWHERE
