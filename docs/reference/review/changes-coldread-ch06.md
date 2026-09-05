# Changes: 06-exceptions.tex (cold-read follow-up)

## Phase 6 (cold read)

### Finding 1 — undefined mnemonics used before their reference entries

Added a new "Instructions Used in This Chapter" subsection immediately after the chapter
Overview, before any use of `EXCP`, `RETE`, `ETFR`, or `ETTR` (including the `fault_handler`
worked example under "Link Register Preservation"). It gives a one-line gloss for each of
`EXCP`, `RETE`, `WAIT`, `RSET`, `ETFR`, and `ETTR`, states once that all six are
meta-instructions that assemble to `COPI` (coprocessor call) forms addressed to the exception
coprocessor (coprocessor `0x1`) — verified against Chapter 16's "Exception Unit
Meta-Instructions" section, which shows each one's "Assembles to: COPI #0x1X00" line — and
points forward to `Section~\ref{sec:exception-registers-coprocessor-summary-forward}` (the
existing "Exception Coprocessor Instructions" opcode table later in this chapter, now labelled)
and to `Chapter~\ref{ch:coprocessor}` for the full syntax/operation/flags/legal-forms entries.
No encoding table was duplicated; this is a glossary-style forward summary only.

Note (not fixed, flagged below): the chapter's existing line "For full syntax, operation
details, and worked examples, see Chapter~\ref{ch:meta-instructions}" (in the "Exception
Coprocessor Instructions" section) appears to point at the wrong chapter — the full
instruction-box entries for `EXCP`/`WAIT`/`RETE`/`RSET`/`ETFR`/`ETTR` actually live in Chapter 16
(`ch:coprocessor`), not Chapter 17 (`ch:meta-instructions`). This was out of scope for the two
authorized findings, so it was left untouched; see Open Questions.

### Finding 6 — "these two rules compose in a non-obvious way" paragraph

Replaced the single dense paragraph (fault raised inside an L5 handler while a second L5 is
pending) with one transition sentence plus a 6-step timeline table (`tab:l5-fault-timeline`,
columns: Step / Event / Exception Level / Link Register Used), followed by a short "in short"
summary sentence restating the outcome. No behavioural claim was changed — the same sequence
(fault preempts at level 7, second L5 queues, RETE drops to level 6, queued L5 collides with the
still-active L5 handler and raises a Level Five Hardware Exception Conflict instead of
dispatching normally) is preserved, just split into discrete steps.

## Findings applied
- Cold-read Finding 1 (glossary of EXCP/RETE/WAIT/RSET/ETFR/ETTR before first use)
- Cold-read Finding 6 (L5-vs-fault timeline table)

## Findings not applied as written
- None. Both findings were applied as specified (glossary list/table format, timeline
  table format) rather than as a mechanical text substitution, since both were explicitly
  presentation fixes with room for the editor's judgment.

## Open questions for the user
- The chapter's existing cross-reference "see Chapter~\ref{ch:meta-instructions}" (in
  "Exception Coprocessor Instructions") appears to name the wrong chapter for
  EXCP/WAIT/RETE/RSET/ETFR/ETTR's full instruction-box entries, which live in Chapter 16
  (`ch:coprocessor`, "Exception Unit Meta-Instructions" section), not Chapter 17. This is a
  pre-existing issue, not introduced by this pass. Recommend the Chapter 16/17 owner or a
  future pass fix it, since Chapter 17 is not this file's chapter to edit.

## Verification
- `grep -c 'begin{'` and `grep -c 'end{'` both return 35 (balanced).
- Full manual recompiled with `pdflatex` from `docs/reference/` after the edits; exit 0, no
  new errors (only pre-existing duplicate-destination warnings from other chapters and
  underfull-vbox warnings unrelated to this file). Build artifacts (`main.pdf`, `main.aux`,
  etc.) were removed/left untracked afterward; only `chapters/06-exceptions.tex` was modified.
