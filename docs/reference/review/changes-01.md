# Changes — chapters/01-introduction.tex

## Findings applied
- **F-con-70** (second person "you"): "Other coprocessors are optional and will vary depending
  on which SIRC-1 model you have" rewritten in third person: "Other coprocessors are optional
  and vary by SIRC-1 model."
- **F-con-71** / **F-arch-16** (duplicate findings, same fix): `$2^24$` corrected to `$2^{24}$`.
- **G1-T4-01** (Example numbering): checked — this chapter contains no `Example:` blocks (it is
  the narrative introduction, not an instruction-reference chapter), so there is nothing to
  number. No change made; noted here so the finding isn't silently dropped.

## Findings skipped (not mine / not accepted)
- **F-arch-6** (file: chapters/01-introduction.tex, status: not-implemented) — left untouched
  per instructions to skip `not-implemented` status. Gate 2 ruling M was not applied since the
  finding itself is out of scope for this pass.
- **F-arch-12** (file: chapters/01-introduction.tex, status: phase4, addressing-mode count) —
  left the "Addressing Modes: 7" table row and related prose untouched, per Gate 2 ruling N and
  explicit task instruction not to change the addressing-mode count.
- **F-arch-2** (file: chapters/02-cpu-architecture.tex) references "the same understatement"
  about DMA/coprocessor-call timing appearing at chapters/01-introduction.tex:25 and :82-85, but
  its `file` field is chapter 2, not chapter 1, so it is not mine to apply. Left the "DMA
  commands are the main exception..." wording in the Execution Model section unchanged; see Open
  questions.
- **F-con-13** (file: chapters/06-exceptions.tex, `system_ram_offset`) cites
  chapters/01-introduction.tex:157 only as corroborating evidence that chapter 1's flat `V * 2`
  vector-address statement is correct; it does not ask for a change to chapter 1, so none was
  made.
- **F-con-64** / **F-arch-15** (file: chapters/03-registers.tex) not applied as chapter-1
  findings, but see the copy-edit note below — the "upper 8 bits" instance in chapter 1 was
  brought into STYLE.md terminology conformance anyway (see below).

## Copy edit (STYLE.md conformance and clarity)
- Hyphenated compound adjectives before nouns: "general-purpose computing" (x2), "General-Purpose
  Registers" (table), "memory-constrained environments," "fixed-length instructions,"
  "cross-compatible," "6-cycle."
- Fixed a grammar error: "The SIRC-1 is designed for ideal for general purpose computing" →
  "The SIRC-1 is designed for general-purpose computing."
- Fixed subject–verb agreement / comma splice in the Coprocessors section ("Only one coprocessor
  can be active at one time and are usually activated...") by splitting into two sentences.
- Fixed a dangling conditional in "This means if the CPU model is missing an optional
  coprocessor, and a program requires it..." → "This means that if a CPU model is missing an
  optional coprocessor and a program requires it..."
- Fixed comma splice in "Each coprocessor can execute 16 opcodes, the first 8 ... can be called
  in any mode." → colon + "and" construction; also fixed misplaced "only" ("can only be
  called" → "can be called only").
- Applied the STYLE.md hex-literal macro pass: wrapped every bare coprocessor-ID and vector-ID
  hex literal in `\texttt{0x...}` (coprocessor table, opcode-nibble ranges, vector-ID table
  column). Addresses already used `\addr{}` and were left as-is.
- Applied STYLE.md terminology: "upper 8 bits" → "high 8 bits" (line ~150); "Protected Mode bit"
  → "protected mode bit" (not a heading, so not Title Case); "behaviour" → "behavior" to match
  the manual's dominant spelling.
- Terminology precision: "alignment fault exception" → "alignment fault" to match the formal
  name used in Chapter 6's exception table (avoids conflating "fault" and "exception").
- Tightened a hedge/redundancy: "can potentially mean larger program sizes" → "can result in
  larger program sizes."
- Removed the hedge opener "Note that" from the high-register-bits sentence.
- Untangled a stacked double-relative-clause sentence for clarity: "It should have powerful
  instruction encodings that include conditional execution and bit shifting that can help reduce
  the number of instructions and branches to keep track of." → "Powerful instruction encodings,
  including conditional execution and bit shifting, should reduce the number of instructions and
  branches a program must track."
- Added terminal periods to the four full-sentence items in the Alignment list (STYLE.md: full
  sentences in a list get a period; these were previously unpunctuated).
- Expanded contraction "don't" → "do not" in running prose (Execution Model section).

No content was restructured, no tables/examples were deleted, no labels or macros were removed,
and the addressing-mode count (7) was left untouched.

## Open questions
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
