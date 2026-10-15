# SIRC-1 Reference Manual Handover

Date: 2026-06-20
Last updated: 2026-09-16

This handover tracks what's left to bring the SIRC-1 CPU Reference Manual up to the standard of a production CPU
programmer's reference manual, using the Motorola M68000 Family Programmer's Reference Manual as the comparison
point. Completed workstreams (internal consistency, machine-checked encodings, the instruction-reference upgrade,
addressing modes, data representation, exceptions/reset, bus timing, coprocessor compatibility, and quick-reference
appendices) have been trimmed out of this file entirely to keep it focused — see git history for the full record.
Two workstreams (ABI/calling-convention docs and a binary/object-format appendix) were deliberately ruled out of
scope for this manual and are also omitted here.

## Source of Truth

Decided 2026-09-16, since this had been resolved case-by-case up to this point without ever being written down: the
manual is canonical. When the manual and the `sirc-vm` Rust implementation disagree, the default assumption is that
the implementation has a bug, not that the manual is wrong -- fix the code to match the manual, and log the
implementation gap (see `examples/*/NOTES.md` and past `implementation-bugs.md` entries for the established format)
rather than editing the manual to match what the code currently does. This already matches how every such conflict
in this project's history has actually been resolved: `SHFR` was rejected as infeasible because the manual's
original design intentionally has no zero register (Ch. 3), the DMA coprocessor was made optional because the
manual's own optional-coprocessor model called for it, several commits are literally titled "fix: match fault
behaviour with the spec in the manual," and the `crc-report` reset-vector bug was fixed in the example program, not
the manual, because the manual's Ch. 1/Ch. 6 prose was already correct.

The manual is only wrong when a normative claim is traced back to source (the Rust implementation, its tests, or
first-principles architectural reasoning) and found to not hold up architecturally -- not merely because the
implementation currently does something else. That's a genuine manual bug, fixed like any other, with the reasoning
recorded in the commit. There is no separate machine-readable spec, and none is planned; if one is ever introduced,
this section should be revisited, since it would become the more natural canonical source over free-form LaTeX
prose.

## Workstream 12: Documentation QA and Build Hygiene

Goal: prevent the manual from drifting away from the implementation.

Open tasks:

- **Done (2026-09-16):** added a reference-manual QA target -- `make qa` / `./qa.sh` in `docs/reference/`, see the
  README's "QA" section for full detail. Deliberately built as plain `make` targets plus a thin bash runner, not a
  wrapper script with its own logic, so a CI pipeline can call the same `make` targets directly: `make format-check`,
  a full `make all` build (which now also runs the new `check-refs` target as its last step, failing if `main.log`
  has any unresolved reference or citation), `make -C ../../examples/manual-examples objects` (assembles every
  Usage Example listing pulled out of the manual chapters -- see `examples/manual-examples/`, new this pass: an
  `extract-listings.awk` script plus a `Makefile` using an ordinary pattern rule, so re-running only reassembles
  listings that actually changed, the same as any C-like build), and `make -C <dir> clean check` for every
  `examples/<name>/` directory an Appendix D listing points at (discovered from the manual source, not hardcoded).
  Bare `Syntax:`/`Operation:` pseudocode blocks (placeholder operands like `#opcode`, not real code) are skipped by
  default rather than reported as failures, with an opt-in `% qa:assemble` comment marker for the few bare blocks
  that are real code. An earlier version of this pass built the same checks as a Python script; reworked into pure
  make + bash on request, to get make's native incremental-rebuild behavior for free and keep every check callable
  directly from CI without a language runtime beyond what's already required. Deep prose/terminology review is left
  to the `manual-cold-reader` agent rather than approximated with regexes.

  First run found 3 real problems (all now fixed), which is a reasonable signal the check is pulling its weight
  rather than just passing trivially:
  - `16-coprocessor-instructions.tex`'s `EXCP` example used a character-literal immediate, `LOAD r1, #'A'` --
    character literals aren't part of the assembler's grammar (confirmed against `parsers/shared.rs`'s
    `parse_number`, and this was the only place in the manual using that syntax, so not an intended-but-unbuilt
    feature). Fixed to `LOAD r1, #0x41`.
  - Two Usage Examples (`15-control-flow.tex`'s `RETS` example, `17-meta-instructions.tex`'s `NOOP` timing-delay
    example) put an inline comment on the same line as a zero-operand meta-instruction. That's a genuine toolchain
    parser gap, not a manual mistake: confirmed with a minimal repro (`NOOP ; hello` alone fails to parse, but bare
    `NOOP` or an operand-bearing instruction with the identical trailing comment both parse fine) that zero-operand
    instructions' grammar doesn't consume a trailing same-line comment the way every other instruction's grammar
    does. Worked around in both examples by moving the comment to its own line so the manual's examples match what
    the assembler actually accepts today. **Logged (2026-09-17)** as `IB-6` in
    `docs/reference/review/implementation-bugs.md` (recreated as the standing home for findings like
    this one) once that file existed again; not fixed.
- **Done (2026-09-16):** wired the QA target into CI (`.github/workflows/manual.yml`, new). Runs on push/PR to
  `main`, path-filtered to `docs/reference/**`, `examples/manual-examples/**`, and `examples/**/*.sasm` (Appendix D
  pulls several of those in via `\lstinputlisting`) so it doesn't fire on unrelated Rust-only changes: installs
  Graphviz via apt and TeX Live via `zauguin/install-texlive` (`packages: scheme-full` -- deliberately the full
  scheme rather than a hand-picked list or apt's texlive-* metapackages, since the manual uses several less-common
  packages -- tcolorbox, bytefield, eso-pic, tikz-timing -- and hand-picking risks silently missing a transitive
  dependency that a real local TeX Live/MacTeX install would never surface; the action caches `~/texlive`, so
  scheme-full's slower install is paid once, not every run), then runs `make format-check`, `make all` (build +
  `check-refs`), and `make -C examples/manual-examples objects` as separate steps, uploading the built PDF as a
  build artifact. Deliberately does *not* re-run `make check` for the Appendix D example programs
  (byte-sieve/faults/hardware-exception/software-exception/store-load) -- `sirc-vm.yml`'s existing `examples` matrix
  job already does that on every push via the `coverage.yml` chain, so repeating it here would just double CI time
  for the same signal. Not yet verified against a real GitHub Actions run (built and tested every underlying `make`
  target locally, but the authoring session had no way to trigger CI itself) -- watch the first run on this branch.
  (Picking the TeX Live action took two wrong turns worth knowing about: `paolobrasolin/setup-texlive-action` looked
  right first but hasn't been touched since 2021, and `TeX-Live/setup-texlive-action` -- despite the org name --
  deprecated itself in favour of `zauguin/install-texlive`, the one actually used here.)
- **Done (2026-09-16):** gave `examples/manual-examples/Makefile` a `check` target (an alias for `objects`) after
  noticing it would otherwise break `examples/check-all.sh`, a local dev script (not used by any CI workflow) that
  discovers every `examples/*/Makefile` and runs `make clean check` in each -- it would have hit "no rule to make
  target 'check'" the first time someone ran it after this pass. Confirmed `check-all.sh`'s `find` picks up
  `manual-examples/` and that `make -C examples/manual-examples clean check` now succeeds. Also confirmed this
  target needs no TeX Live at all -- it only reads the manual chapters as plain text via `awk` (any POSIX awk;
  tested against BSD awk and gawk, reasoned but not tested against Ubuntu's default `mawk` since installing it
  locally hit an unrelated Homebrew permissions issue) plus a Rust toolchain for the assembler, both already
  present wherever `sirc-vm.yml`'s existing `examples` job runs.
- **Idea, not started:** publish the built PDF to GitHub Releases automatically on a version bump, using something
  like `release-please` to manage the versioning/changelog and a follow-up job to attach
  `docs/reference/SIRC-1-reference-manual.pdf` from `manual.yml`'s build artifact to the created release. Long-term
  goal per the author (2026-09-16); explicitly not part of this pass.
- Chapter 2's 24 MHz maximum clock rate and 5 V +/- 5% supply figures have no stated basis. Deliberately left
  unresolved for now (author's call, 2026-09-12) rather than fixed as part of this pass.
- Generate more tables from source where possible: opcode map, register map, condition codes, instruction
  encodings, coprocessor IDs.
- **Done (2026-09-16):** added a "manual source of truth" note -- see the new "Source of Truth" section near the
  top of this file. The manual is canonical.
- **Done (2026-09-16):** decided on TODO tracking -- `% TODO: <description>` followed by `% category=Manual` on
  the next line, reusing the exact convention already used across the Rust codebase (`// TODO: ...` /
  `// category=<Category>`, e.g. `Toolchain`/`Refactoring`/`Bugs`) rather than inventing a new one. LaTeX `%`
  comments are invisible in the rendered PDF, satisfying "avoid prose TODOs in the rendered manual," and
  `.github/workflows/todo.yml` (`ribtoks/tdg-github-action`) already scans the whole repository -- `.tex` files
  aren't excluded -- and files a GitHub issue per TODO on every push to `main`, so this gets issue tracking for
  free with no new tooling. Documented in `docs/reference/README.md`'s new "Tracking Open Questions" section.
  Retrofitted the two pre-existing bare `% TODO:` comments (`02-cpu-architecture.tex`, one about the block diagram;
  `appendix-f-external-interface.tex`, one about decoupling-capacitor guidance) with the missing
  `% category=Manual` line so they match going forward.
- **Done (2026-09-16):** audited assembly examples by chapter. Surveyed every architecture-side chapter (1
  Introduction, 2 CPU Architecture, 4 Data Representation, 5 Status Register, 6 Exceptions) plus Appendix F
  (external interface/bus) for raw SIRC-1 listings. Chapters 1, 2, 4, 6, and Appendix F had none, inline or
  external -- already clean, nothing to do. Chapter 5 had 7 (2 bare, under "Manual Updates" and "Reading the
  Status Register"; 5 in a "Usage Examples" section), and Appendix B (Timing) had 7 more, but Appendix B's are
  cycle-count optimization tips ("instead of X, use Y -- saves N cycles"), which is exactly what a
  timing/optimization appendix should contain, not a candidate for this item.
  - Chapter 5's two bare listings ("Manual Updates": `ORRI sr`/`ANDI sr`/`LOAD sr, (s)+`; "Reading the Status
    Register": `LOAD r1, sr` in both modes) directly illustrate the mechanism their own section just described, the
    same way a real CPU manual's status-register chapter would show `ORI #imm,SR` -- legitimate, kept. Verified
    both actually assemble (they'd never been checked, being bare blocks outside an `example` float) and opted
    them into the QA target with `% qa:assemble`.
  - Three of the five "Usage Examples" were straight duplicates of content already covered better elsewhere, so
    removed rather than kept or moved: "Conditional Execution" (comparison/branch idioms) duplicated Chapter 10's
    "Unsigned Comparison"/"Signed Comparison"/"Equality Testing" examples; "Manual Status Flag Control" duplicated
    Chapter 9's own "Status Flag Update Control" section (`sec:status-override`), which Chapter 5's surrounding
    prose already pointed to as the complete reference; "Testing Bits" was a strict subset of Chapter 13's
    `TSAI`/`TSAR` entry's own "Bit Testing with TSA" example. Replaced with a short paragraph pointing to Chapters
    9, 10, and 13 instead of restating their content.
  - The remaining two ("Enabling Protected Mode," "Saving and Restoring Status") are unique to this chapter --
    tied to the P bit and the EA-bit-not-restored caveat specifically documented here -- and were kept as-is.
  - Net effect: Chapter 5 went from 7 unverified/partly-duplicate listings to 4, all either newly verified or
    already example-float-verified, and the manual has three fewer places a future edit to condition-code or
    status-override behavior could drift out of sync. Rebuilt clean (0 undefined references, 69 verified listings
    down from 70 -- 3 removed, 2 newly opted in) and re-ran the QA target.
- **Done (2026-09-16):** audit the whole manual for anachronistic references that break the 1989 "in-universe"
  framing established by the title page, preface, and front/back covers. Two instances found on 2026-09-15 as a
  starting point, both fixed (one on 2026-09-15, the other -- `appendix-c-undocumented.tex` -- on 2026-09-16, see
  below), then followed by a full sweep of every chapter/appendix/front-matter file on 2026-09-16 rather than
  leaving the rest unaudited:
  - A sentence in `16-coprocessor-instructions.tex`'s DMA Unit section claiming "the reference SIRC-1 emulator has
    not yet implemented the DMA unit" (fixed 2026-09-15 -- removed, since the optional-coprocessor framing already
    covers the point without the anachronism).
  - `appendix-c-undocumented.tex` (fixed 2026-09-16): named the real "SIRC-VM" simulator directly, had a subsection
    literally titled "Current Simulator" (citing "the reference test suite" as its evidence basis), and a "For
    Researchers/Hackers" section that read like modern engineering documentation. Per the author's guidance:
    referring to a general toolchain SIRC itself provides is fine, but no command-line specifics and no references
    to what the current Rust simulator does or doesn't implement. Fixed: "current reference implementation" (in
    the Observed Behavior section) and "current simulator or hardware behaviour" (Overview) reworded to the
    manual's existing generic phrasing ("current implementations," "current behaviour"); the "Current Simulator"
    and "Hardware Implementation" subsections merged into one "Implementation Behavior" subsection describing what
    any implementation does, since once the software one couldn't be named, they were describing the same
    behaviour from two angles that didn't need separating; "For Researchers/Hackers" removed outright rather than
    reworded, since a 1989 vendor manual wouldn't carve out a hobbyist/reverse-engineer audience and its content
    (test thoroughly, expect behavior changes) was already redundant with "For Application Developers."
  - **Full sweep (2026-09-16):** grepped every `chapters/*.tex` file, `preface.tex`, and `notation-and-glossary.tex`
    for a broad set of anachronism-risk terms (`simulator`, `emulat-`, `SIRC-VM`, `reference implementation`, `test
    suite`, `rust`, `cargo`, `github`, `repository`, `CI`/`continuous integration`, `not yet implement-`, `hacker`,
    `debugg-`, `makefile`, `commit`, `pull request`, `http`/`www`/`.com`, `python`, `json`, `git`, `license`,
    `pipeline`, and several more) and read every hit in context. Everything else was a false positive (`.ORG` the
    assembler directive matching a `\.org` pattern, "make" the English verb matching `\bmake\b`) or already
    legitimate period-appropriate architecture prose: extensive "software emulation of a missing/optional
    coprocessor via an invalid-opcode trap" language throughout Chapter 6 and Chapter 16 is exactly how real 1980s
    CPU manuals described FPU-less models trapping to software float emulation (the M68000 family did this for the
    68881/68882), not an anachronism; "commits"/"committed" is the manual's own write-back/bus-transaction
    terminology, unrelated to version control; "pipeline" is the CPU execution pipeline, unrelated to CI/CD;
    generic "debuggers"/"emulator and simulator authors" plurals already match the established vague-implementation
    convention. One pleasant find while sweeping: `05-status-register.tex`'s Trace Mode entry already names
    "SIRCIT (SIRC Instruction Tracer)" as the in-universe debugging tool -- a good example of the pattern this item
    asks for (a fictional SIRC-branded tool, no real specifics), not something to fix. Rebuilt clean (0 undefined
    references) and re-ran the QA target after the `appendix-c-undocumented.tex` fixes; the sweep itself found
    nothing further to change.
- **Done (2026-09-16):** label syntax in the manual's worked examples. Audited every labeled code sample across
  `chapters/*.tex` (18 labels in `10-condition-codes.tex`, `15-control-flow.tex`, and
  `16-coprocessor-instructions.tex`) and converted every trailing-colon label (`while_start:`) to the leading-colon
  form (`:while_start`) the assembler actually accepts. The same pass over `16-coprocessor-instructions.tex`'s
  "Retry-Safe DMA Copy Routine" example (added earlier this pass) also turned up three branch targets missing
  their `@` prefix (`BRAN|== dma_fault_is_dma` instead of `BRAN|== @dma_fault_is_dma`) and one address load using
  `#label` instead of `@label` (`LOAD al, #dma_copy_failed`) -- fixed alongside the label syntax. Every fixed
  listing was re-verified by actually assembling it (`cargo run --bin assembler`), not just read for correct
  syntax. Added a rule to `STYLE.md` §7 (Code listings) documenting both conventions (leading-colon declarations,
  `@label` references with no `#`) so this doesn't reappear.
- **Done (2026-09-16):** documented the assembler's directives (`.ORG`, `:label`/`@label`, `.DB`/`.DW`/`.DQ`,
  `.EQU`) in a new Appendix D.1 "Assembler Directives" section, sourced directly from the toolchain parser
  (`sirc-vm/toolchain/src/parsers/shared.rs`, `parsers/data.rs`, `data/object.rs`) rather than inferred from
  examples, so it captures the non-obvious behavior too: `.DB`/`.DW`/`.DQ` all occupy the same 4-byte program slot
  regardless of name (the difference is only where a numeric value lands within it), and `@label`'s resolution
  (PC-relative offset, low word, or full address) depends on where it's used, with `.r`/`.l`/`.u` suffixes to
  override the default explicitly. Includes a worked example (Example D-1) assembled and linked with the real
  toolchain to confirm it's valid, showing a `.ORG 0x0000`/`.DQ @label` reset vector and an exception vector table
  entry side by side -- covering the previously-missing worked reset-vector example too (see
  `examples/crc-report/NOTES.md`, "Leading NOOP requirement (resolved...)", for the incident that motivated this).
  Cross-referenced from Chapter 11's notation table, which had a simplified, now-linked version of the same
  `@label` entry.
- **Done (2026-09-16):** ran a scoped `/manual-edit` pass to fact-check and consistency-check everything changed
  this session, since none of it had been through the systematic pipeline before. Full detail in
  `docs/reference/review/REPORT.md` and `docs/reference/review/triaged.md`; summary: 65 raw findings from 4
  targeted fact-checkers (data/status/condition, exceptions/undocumented, coprocessor/meta, summaries/appendices)
  plus a full-manual consistency check, deduplicated to 56, all resolved with the author at Gate 2 and applied
  across 16 chapter files. Three standouts: Chapter 6's exception quick-reference table had the wrong return
  address for three of six retryable faults; Chapter 7's worked register-encoding example was off by one on every
  field; and this session's own new "Retry-Safe DMA Copy Routine" example (Chapter 16) had a real logic bug --
  `l` was doing double duty as both the DMA destination register and the subroutine link register, so `RETS`
  would jump into copied data instead of returning -- rewritten with a new calling convention and verified against
  the real assembler, hand-traced for stack-arithmetic correctness, and (after the rewrite overflowed its
  non-breaking float) split into two parts to fix a genuine page-layout break. Also found and fixed two real Rust
  bugs unrelated to any manual error (see `implementation-bugs.md`: `IB-3`, interrupt dispatch blocking on a
  masked pending line, fixed this pass with two new unit tests; `IB-1`, a still-incomplete segment-overflow
  displacement fix, logged only) plus three more logged-only implementation gaps (`IB-2`, `IB-4`, `IB-5`). One
  item is still open: the reset-vector-fetch cycle count (Chapter 6 says 4, Appendix B/Chapter 16 say 6) has two
  reviewers disagreeing and the author wants to dig in further before deciding -- not touched this pass.

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
   **Done (2026-09-12):** added one 32-bit bytefield diagram per encoding form to all 33 instruction entries across
   Chapters 13-17 (81 diagrams total), per the one-diagram-per-form scoping decision. Deliberately plain, uncaptioned
   `bytefield` blocks rather than a captioned `figure`/`example` float, so they don't bloat the List of Figures or
   List of Examples. Two mechanics were needed:
   - Ordinary instructions (`ADDI`/`ADDR`, `LOAD`, `STOR`, `LDEA`, `LDEL`, `COPI`/`COPR`, etc.) get one diagram per
     opcode/format pair, with generic field labels matching Chapter 7's canonical field names (`Reg`, `R1`-`R3`,
     `Imm`, `AF`, `Cond`, etc.), and the entry's real opcode filled into the Opcode/Op field. `LDEA`/`LDEL` use their
     own operand names (`dest`, `src`, `rO`) instead of generic `Reg`/`R1`-`R3`, matching their prose.
   - Meta-instructions (`SHFT`, `BRAN`, `BRSR`, `RETS`, `LJMP`, `LJSR`, `EXCP`, `WAIT`, `RETE`, `RSET`, `ETFR`/`ETTR`,
     `DMAR`, `DMAW`, `DMAT`, `MULU`/`MULS`, `DIVU`/`DIVS`, `NOOP`) don't have their own opcode, so their diagram
     instead shows the real instruction they assemble to, with the fields the meta-instruction fixes shown as
     literal values (e.g. `NOOP`'s diagram shows `ADDI` opcode `0x00` with `Reg` fixed to `0x0`, `Immediate Value`
     fixed to `0x0000`, and `AF` fixed to `00`) rather than generic field names.
   - A macro-based approach (three reusable `\newcommand`s wrapping the `bytefield` environment, taking the opcode
     and field labels as arguments) was tried first but abandoned: the `bytefield` package's `\bitbox`/`&` machinery
     does not work when its row content is supplied through macro parameter substitution -- confirmed with a minimal
     reproduction outside the manual (`Extra alignment tab has been changed to \cr`, "more \span or & marks than
     were in the preamble"). All 81 diagrams are written out literally instead, matching the pattern Chapter 7's own
     three format diagrams already used.
   - While working through these, also resolved the standing "REMEMBER TO WORK OUT WHAT IS WRONG WITH THE DIAGRAM"
     note at the bottom of this file: Chapter 9's Arithmetic Shift Left (ASL) subsection was the only one of the six
     shift types (LSL, LSR, ASL, ASR, RTL, RTR) missing its bit-movement figure -- LSL, LSR, ASR, RTL, and RTR each
     had one, ASL didn't. Added the missing figure, identical in shape to LSL's (same data movement; ASL only
     differs from LSL in overflow-flag semantics, which the directional figure doesn't depict anyway).
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

All Workstream 13 and 14 backlog items above are now done. What remains open is Workstream 12 (documentation QA and
build hygiene) and the two smaller, not-currently-a-known-problem Workstream 13 items noted above (`lstlisting`
restyle, chapter/part opening-page review) -- neither was part of the agreed backlog for this pass.

## Definition of Done

The manual is "up to scratch" when, in addition to everything already met by the completed workstreams:

- There's a single repeatable command that builds the PDF, verifies generated examples, and checks
  references/terminology (Workstream 12).
- Every term used in the manual is defined the first time it appears or is listed in a glossary (Workstream 13).
