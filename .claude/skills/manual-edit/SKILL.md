---
name: manual-edit
description: Run a publisher-style editorial pass over the SIRC-1 manual set -- the architecture reference (docs/reference), the hardware reference (docs/hardware) and the assembly programmer's manual (docs/assembly) -- using a pipeline of specialised subagents (facts digest, period benchmark against real 1990s CPU manuals, style guide, fact-checkers, cross-manual continuity checker, verifier, per-chapter copy editors, LaTeX QA, cold reader with improvement suggestions). Invoke with /manual-edit. Optional arguments: "manuals reference|hardware|assembly" (comma-separated, default all three), "phase N" to run a single phase, "chapters <glob>" to restrict scope.
---

# Manual editing pass

You are the orchestrator. Your job is to dispatch subagents, keep your own context small, and
present decisions to the user at the gates. **Never read a chapter file yourself.** You read
only files under `docs/reference/review/`, `docs/reference/STYLE.md`, and agent reports.
Everything else is delegated. If you catch yourself opening `docs/*/chapters/*.tex`, stop and
dispatch an agent instead.

## The manual set

| Manual | Directory | Authoritative for | Size |
|---|---|---|---|
| Architecture reference | `docs/reference` | behaviour inside the CPU: ISA, encodings, exceptions, timing | ~70k words, 22 chapter files |
| Hardware reference | `docs/hardware` | what is driven or sampled on each pin: pinout, bus, signals | ~5k words, 6 chapter files |
| Assembly programmer's manual | `docs/assembly` | what the toolchain accepts and emits | ~4k words, 8 chapter files |

The three share `docs/reference/preamble.tex`, `docs/reference/STYLE.md` and the latexindent
configuration. They refer to each other by chapter name, not number (`\archmanual{}`,
`\hwmanual{}`). The `manuals` argument restricts every phase to the named manuals, except the
continuity checker, which always reads all three because cross-manual drift is what it is for.

Working directory for all outputs: `docs/reference/review/` (the corpus lives there and is
gitignored, so the review files stay alongside it). Create it if missing. The pass is
resumable: at the start of each phase, if that phase's output files already exist and the user
has not asked to redo it, skip it and say so.

## Commits

Commit after every phase on the current branch with the message
`docs(manual): editorial pass phase N - <name>` so a partial run is never lost and each
phase's diff is reviewable on its own. Do not push unless the user asks.

- Commit with `git commit --no-gpg-sign`; the signing key needs Touch ID, which a background
  commit cannot reach.
- The pre-commit hook runs `latexindent --check` over every architecture-manual file in the
  working tree, not only the staged ones. Format every edited `.tex` file first, from its own
  manual's directory: `latexindent -m -l --cruft=.cruft/ --silent -w -s <file>`. Otherwise one
  unformatted chapter blocks every commit.
- The PDFs are tracked. After any phase that edits chapters, run `make all` in each edited
  manual's directory and commit the PDF in its own commit.
- Never commit `docs/reference/chapters/general-todo.md`; it is the author's own untracked list.

## Chapter groups

Fact-checking runs per group so that one agent holds all the chapters that share an area of
the implementation. Copy editing runs per chapter file.

| Group | Files (relative to `docs/`) | Tag |
|---|---|---|
| G1 architecture and registers | reference: preface, 01-introduction, 02-cpu-architecture, 03-registers | arch |
| G2 data, status, condition codes | reference: 04-data-representation, 05-status-register, 10-condition-codes | data |
| G3 exceptions and undocumented behaviour | reference: 06-exceptions, appendix-c-undocumented | exc |
| G4 formats, addressing, shifts | reference: 07-instruction-formats, 08-addressing-modes, 09-shift-operations | enc |
| G5 ALU | reference: 13-alu-instructions | alu |
| G6 memory and control flow | reference: 14-memory-instructions, 15-control-flow | mem |
| G7 coprocessor and meta | reference: 16-coprocessor-instructions, 17-meta-instructions | cop |
| G8 summaries and appendices | reference: 11-reading-instructions, 12-instruction-summary, appendix-a-opcode-map, appendix-d-examples, appendix-e-quick-reference, notation-and-glossary | sum |
| G9 timing | reference: appendix-b-timing | tim |
| G10 hardware | hardware: every chapter | hw |
| G11 assembly | assembly: every chapter | asm |

Each group is under about 8,000 words of LaTeX, so a fact-checker's context holds its chapters,
the facts digest, and the code it needs to open with room to spare. Skip the groups of manuals
not in scope. Run at most five agents at once.

## Phase 0: preflight

Run `git status --porcelain docs/` and stop if there are uncommitted changes other than
untracked files the author keeps there (`general-todo.md`, `interaction-review.md`); the user
must commit or stash first so each phase's diff is clean. Check whether `pdflatex`,
`latexindent` and `cargo` are on PATH and remember the answers for phase 5.

## Phase 1: ground truth (three agents)

Check `docs/reference/review/corpus/` for period reference-manual PDFs. If it is empty, tell
the user which manuals to drop in (the Motorola M68000 Family Programmer's Reference Manual
`M68000PRM.pdf` and MC68000 User's Manual `MC68000UM.pdf` at minimum; both are distributed
free by NXP) and ask whether to proceed without the period benchmark. The directory is
gitignored; never commit the PDFs.

Dispatch `manual-facts-digest` and `manual-period-benchmark` together, naming the manuals in
scope. When both return, dispatch `manual-style-editor`; it reads `review/period-style.md` so
it must run after the benchmark. If `STYLE.md` already exists, the style editor updates it
rather than rewriting it, and only surfaces new decisions. Then read `review/facts.md`'s
`## Open questions`, `STYLE.md`'s "Decisions needing human approval", and the ranked gap list
at the end of `review/period-gaps.md`.

**Gate 1.** Present three things to the user with `AskUserQuestion`:

1. The new style decisions, grouped, for batch approval with per-item override. Apply
   overrides to `STYLE.md` yourself with `Edit`.
2. The facts-digest open questions, for information; they become `unclear` findings later.
3. The period coverage gaps. These are new content and out of scope for an editing pass. Ask
   which to accept as backlog, and append the accepted ones to the "Period Coverage Gaps"
   workstream in `docs/reference/manual-handover.md`, each with the corpus citation. Gaps of
   the kind "add a field to the instruction template" are the one exception: if the user
   accepts one, it becomes a `major` finding against every chapter with instruction entries
   and is applied in Phase 3.

Do not proceed until the style guide is approved; every later agent depends on it.

## Phase 2: review (fact-checkers and the continuity checker, batches of five)

Dispatch one `manual-fact-checker` per group in scope with a prompt of this form:

```
Chapter group G5 (ALU). Files: docs/reference/chapters/13-alu-instructions.tex.
Output file: docs/reference/review/facts-alu.md. Reviewer tag: alu.
```

Dispatch `manual-consistency-checker` in the first batch since it is the longest running.
Output file `review/consistency.md`.

### Verify

Reviewers are sometimes wrong, and a wrong `blocker` costs an editor's time and the author's
trust. When all reviewers return, collect every `blocker` and every `major` with category
`fact` or `contradiction`. Dispatch a `general-purpose` agent per batch of about fifteen with
this brief: for each finding, open the quoted text and the cited evidence, and return
`confirmed`, `refuted` (with the reason) or `reclassify` (with the corrected severity or
resolution). It reads only what the findings cite. Drop refuted findings, noting their IDs.

### Triage

Build `review/triaged.md`:

1. Concatenate all findings files. Sort by severity then file.
2. Deduplicate obvious repeats (same file, overlapping lines, same claim) keeping the one with
   the stronger evidence, and note the merged IDs.
3. Mark every finding with a `status:` line. Default rules:
   - `manual-wrong` with `confidence: high`, and every `nit` and `minor` that is not an
     `improvement`: `accepted`.
   - `code-wrong`: `simulator`. The manual stays as it is.
   - `design`, `unclear`, `confidence: low`, and every `improvement`: `defer`.

**Gate 2.** Show the user, in this order:

1. Every `design` finding, one at a time or in small related groups, with the options and the
   reviewer's recommendation. These are the author's calls; record each answer as the finding's
   `fix` and mark it `accepted`. Do not treat a partial answer or a follow-up question as a
   decision.
2. The remaining deferred list in full.
3. The `improvement` findings, grouped by manual, for the author to accept or drop.
4. A count of the accepted findings by severity, with a sample of five.

The user can flip any status. Write their answers back into `review/triaged.md`. Append each
`simulator` finding to `docs/reference/chapters/general-todo.md` as the next numbered item,
naming the manual section, the expected behaviour, the code location if known, and the tests
to add; this file is never committed.

## Phase 3: edit (one agent per chapter, batches of five)

Dispatch `manual-chapter-editor` for every chapter file of the manuals in scope
(`docs/<manual>/chapters/*.tex`) that has accepted findings or has not been copy edited since
`STYLE.md` last changed, with a prompt naming the one file it owns. Chapters are disjoint
files, so parallel editors cannot conflict. Do not dispatch an editor for `generated/*`; those
are machine output.

Cross-chapter terminology fixes from the consistency checker (findings whose `fix` is a grep
pattern spanning many files) are yours to apply after all editors return, with a single
`sed`-style replacement per term, so they are not applied many times inconsistently.
Show the command and its match count before running it.

When all editors return, concatenate the `changes-*.md` files' open-question sections into
`review/open-questions.md` and show it to the user. Format, then commit one commit per manual.

## Phase 4: continuity re-check (one agent)

Run `manual-consistency-checker` again with output `review/consistency-2.md`. Independent
editors will have introduced a few new inconsistencies. Apply the fixes yourself if they are
one-line terminology or reference fixes; otherwise dispatch a `manual-chapter-editor` for just
the affected chapters with a fresh `triaged.md` slice. Commit.

## Phase 5: build and layout (one agent per manual, local only)

If TeX is available, dispatch `manual-latex-qa` once per manual in scope, naming its
directory and output file `review/latex-qa-<manual>.md`. It also runs that manual's code
checks: `make listings` for the assembly manual and `./qa.sh` for the architecture manual
(which assembles the extracted listings and the Appendix D examples), when `cargo` is
available. If it reports layout problems that need wording changes to fix, dispatch a
`manual-chapter-editor` for that chapter with the finding. If TeX is not available, write
`review/latex-qa.md` containing one line saying the phase was skipped and must be run on a
machine with MacTeX, and tell the user. Commit the rebuilt PDFs.

## Phase 6: cold read and improvements (one agent per manual)

Dispatch `manual-cold-reader` once per manual in scope, with output
`review/cold-read-<manual>.md`; the architecture-manual reader also gets the other two
manuals' tables of contents so it can judge the set as a whole. Do not act on its findings
automatically; they are judgement calls about structure and suggestions, not errors. Present
the top items from each manual to the user and let them pick which to act on, then dispatch
editors for those chapters. Commit.

## Final report

Write `review/REPORT.md`: findings counts by manual, phase and severity, what was applied,
what was deferred and why, the design decisions the author made, the simulator todos added to
`general-todo.md` (by number), the accepted improvements still to do, and the pages needing a
visual check. Keep it under a page. Tell the user the branch is ready for `git diff main`
review.

## Context and cost discipline

- You never hold chapter text. Agents do. Your context should stay under about 60k tokens for
  the whole run.
- An agent that returns a report longer than a screen has failed the brief; ask it via
  `SendMessage` for the three-line summary rather than reading the whole thing.
- Batches of five: the machine and the rate limit both prefer it, and a failed batch is
  cheaper to redo.
- If a phase's agent fails or times out, re-dispatch only that agent, not the phase.
