---
name: manual-edit
description: Run a publisher-style editorial pass over the SIRC-1 reference manual under docs/reference using a pipeline of specialised subagents (facts digest, period benchmark against real 1990s CPU manuals, style guide, fact-checkers, continuity checker, per-chapter copy editors, LaTeX QA, cold reader). Invoke with /manual-edit. Optional arguments: "phase N" to run a single phase, "chapters <glob>" to restrict scope.
---

# Manual editing pass

You are the orchestrator. Your job is to dispatch subagents, keep your own context small, and
present decisions to the user at the two gates. **Never read a chapter file yourself.** You
read only files under `docs/reference/review/`, `docs/reference/STYLE.md`, and agent reports.
Everything else is delegated. If you catch yourself opening `docs/reference/chapters/*.tex`,
stop and dispatch an agent instead.

Working directory for all outputs: `docs/reference/review/`. Create it if missing. The pass is
resumable: at the start of each phase, if that phase's output files already exist and the user
has not asked to redo it, skip it and say so.

Commit after every phase on the current branch with the message
`docs(manual): editorial pass phase N - <name>` so that a partial run is never lost and each
phase's diff is reviewable on its own. Do not push unless the user asks.

## Chapter groups

Fact-checking runs per group so that one agent holds all the chapters that share an area of
the implementation. Copy editing runs per chapter file.

| Group | Chapters | Tag |
|---|---|---|
| G1 architecture and registers | 01-introduction, 02-cpu-architecture, 03-registers | arch |
| G2 data, status, condition codes | 04-data-representation, 05-status-register, 10-condition-codes | data |
| G3 exceptions and undocumented behaviour | 06-exceptions, appendix-c-undocumented | exc |
| G4 formats, addressing, shifts | 07-instruction-formats, 08-addressing-modes, 09-shift-operations | enc |
| G5 ALU | 13-alu-instructions | alu |
| G6 memory and control flow | 14-memory-instructions, 15-control-flow | mem |
| G7 coprocessor and meta | 16-coprocessor-instructions, 17-meta-instructions | cop |
| G8 summaries and appendices | 11-reading-instructions, 12-instruction-summary, appendix-a-opcode-map, appendix-d-examples, appendix-e-quick-reference | sum |
| G9 timing | appendix-b-timing | tim |

Each group is under about 8,000 words of LaTeX, so a fact-checker's context holds its chapters,
the facts digest, and the code it needs to open with room to spare. Run at most five agents at
once.

## Phase 0: preflight

Run `git status --porcelain docs/reference` and stop if there are uncommitted changes; the user
must commit or stash first so each phase's diff is clean. Check whether `pdflatex` or
`latexmk` is on PATH and remember the answer for phase 5.

## Phase 1: ground truth (three agents)

Check `docs/reference/review/corpus/` for period reference-manual PDFs. If it is empty, tell
the user which manuals to drop in (the Motorola M68000 Family Programmer's Reference Manual
`M68000PRM.pdf` and MC68000 User's Manual `MC68000UM.pdf` at minimum; both are distributed
free by NXP) and ask whether to proceed without the period benchmark. The directory is
gitignored; never commit the PDFs.

Dispatch `manual-facts-digest` and `manual-period-benchmark` together. When both return,
dispatch `manual-style-editor`; it reads `review/period-style.md` so it must run after the
benchmark. Then read `review/facts.md`'s `## Open questions`, `STYLE.md`'s "Decisions needing
human approval", and the ranked gap list at the end of `review/period-gaps.md`.

**Gate 1.** Present three things to the user with `AskUserQuestion`:

1. The style decisions, grouped, for batch approval with per-item override. Apply overrides to
   `STYLE.md` yourself with `Edit`.
2. The facts-digest open questions, for information; they become `unclear` findings later.
3. The period coverage gaps. These are new content and out of scope for an editing pass. Ask
   which to accept as backlog, and append the accepted ones to
   `docs/reference/manual-handover.md` as a new `## Workstream 14: Period Coverage Gaps`
   section, each with the corpus citation. Gaps of the kind "add a field to the instruction
   template" are the one exception: if the user accepts one, it becomes a `major` finding
   against every chapter with instruction entries and is applied in Phase 3.

Do not proceed until the style guide is approved; every later agent depends on it.

## Phase 2: review (ten agents, batches of five)

Dispatch one `manual-fact-checker` per group with a prompt of this form:

```
Chapter group G5 (ALU). Files: docs/reference/chapters/13-alu-instructions.tex.
Output file: docs/reference/review/facts-alu.md. Reviewer tag: alu.
```

Dispatch `manual-consistency-checker` in the first batch since it is the longest running.
Output file `review/consistency.md`.

When all return, build `review/triaged.md`:

1. Concatenate all findings files. Sort by severity then file.
2. Deduplicate obvious repeats (same file, overlapping lines, same claim) keeping the one with
   the stronger evidence, and note the merged IDs.
3. Mark every finding with a `status:` line. Default rules: `blocker` and `major` with
   `resolution: manual-wrong` and `confidence: high` are `accepted`; `nit` and `minor` are
   `accepted`; anything with `resolution: unclear` or `code-wrong` or `confidence: low` is
   `defer`.

**Gate 2.** Show the user the deferred list in full (these are the cases where the manual and
the implementation disagree, or the reviewer could not tell) and a count of the accepted ones
by severity with a sample of five. The user can flip any status. Write their answers back into
`review/triaged.md`. Items resolved as `code-wrong` are out of scope for this pass; collect
them into `review/implementation-bugs.md` for the user to fix in the Rust separately.

## Phase 3: edit (one agent per chapter, batches of five)

Dispatch `manual-chapter-editor` for every file in `docs/reference/chapters/*.tex` except
`title.tex`, with a prompt naming the one file it owns. Chapters are disjoint files, so parallel
editors cannot conflict. Do not dispatch an editor for `generated/*`; those are machine output.

Cross-chapter terminology fixes from the consistency checker (findings whose `fix` is a grep
pattern spanning many files) are yours to apply after all editors return, with a single
`sed`-style replacement per term, so they are not applied twenty-two times inconsistently.
Show the command and its match count before running it.

When all editors return, concatenate the `changes-*.md` files' open-question sections into
`review/open-questions.md` and show it to the user. Commit.

## Phase 4: continuity re-check (one agent)

Run `manual-consistency-checker` again with output `review/consistency-2.md`. Twenty-two
independent editors will have introduced a few new inconsistencies. Apply the fixes yourself if
they are one-line terminology or reference fixes; otherwise dispatch a `manual-chapter-editor`
for just the affected chapters with a fresh `triaged.md` slice. Commit.

## Phase 5: build and layout (one agent, local only)

If TeX is available, dispatch `manual-latex-qa`. If it reports layout problems that need
wording changes to fix, dispatch a `manual-chapter-editor` for that chapter with the finding.
If TeX is not available, write `review/latex-qa.md` containing one line saying the phase was
skipped and must be run on a machine with MacTeX, and tell the user. Commit.

## Phase 6: cold read (one agent)

Dispatch `manual-cold-reader` with output `review/cold-read.md`. Do not act on its findings
automatically; they are judgement calls about structure. Present the top items to the user and
let them pick which to act on, then dispatch editors for those chapters. Commit.

## Final report

Write `review/REPORT.md`: findings counts by phase and severity, what was applied, what was
deferred and why, implementation bugs found, and the pages needing a visual check. Keep it
under a page. Tell the user the branch is ready for `git diff main` review and that
`review/implementation-bugs.md` needs a separate pass on the Rust.

## Context and cost discipline

- You never hold chapter text. Agents do. Your context should stay under about 60k tokens for
  the whole run.
- An agent that returns a report longer than a screen has failed the brief; ask it via
  `SendMessage` for the three-line summary rather than reading the whole thing.
- Batches of five, not twenty-two at once: the machine and the rate limit both prefer it, and a
  failed batch is cheaper to redo.
- If a phase's agent fails or times out, re-dispatch only that agent, not the phase.
