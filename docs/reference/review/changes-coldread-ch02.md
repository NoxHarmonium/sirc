# Changes: cold-read phase 6 follow-up (Chapter 2 external interface move)

## What was moved

The entire `\section{External Interface}` from `chapters/02-cpu-architecture.tex`
(originally lines 169-908: the Chip Layout subsection with the DIP-64 pinout figure
and pin table, the Pin Descriptions subsection with all pin-by-pin prose, and the
Timing Notes subsection with the bus-timing tables and all nine timing diagrams) was
moved verbatim into a new file `chapters/appendix-f-external-interface.tex`, as
`\chapter{External Interface}` with `\label{appendix:external-interface}`. No wording,
tables, figures, or labels inside the moved content were changed. This appendix is
741 lines.

`docs/reference/main.tex` now has
`\input{chapters/appendix-f-external-interface}` immediately after
`\input{chapters/appendix-e-quick-reference}`, so it becomes Appendix F after the
existing appendices A-E; none of the existing appendices were renumbered or moved.

## What was kept in Chapter 2

Checked lines 1-168 of Chapter 2 (everything before the deleted section) for
references to specific pin names or to the reset/interrupt sampling rule: found
none. No other part of Chapter 2 depends on the deleted material by pin name.

The deleted section was replaced with a short `External Interface` section
containing one paragraph: a pointer to Appendix F (`\ref{appendix:external-interface}`)
for the chip package, pin-by-pin descriptions, and timing diagrams, plus one sentence
retaining the architectural fact that on `RSTI` assertion the CPU aborts the current
instruction/bus wait and restarts at phase 0 (tying back to the Six-Stage Execution
Phases section earlier in the chapter). This is a judgment call: no existing sentence
in Chapter 2 explicitly depended on this fact, but it was the one architectural
consequence of the deleted section most relevant to material Chapter 2 keeps, so it
was retained rather than dropped silently. Kept the section heading (rather than a
bare unheaded paragraph) so the table of contents still shows readers where to find
the external-interface material; this was a literal-vs-intent judgment call on "replace
it with one paragraph" and is flagged here for review.

Also added, per cold-read finding 3, one sentence after the intro paragraph of
"Six-Stage Execution Phases" noting that each address-register pair (including `p`)
splits into a high half and low half, written `Xh`/`Xl` (e.g. `ph`/`pl`), with a
forward reference to Chapter 3 (`\ref{ch:registers}`), since the phase descriptions
use `\reg{pl}`/`\reg{ph}` before Chapter 3 defines the split.

## Cross-reference fixes made

Searched all labels defined inside the moved section
(`tab:bus-output-timing-rules`, `tab:bus-response-timing-rules`,
`tab:bus-cycle-ownership`, `tab:external-input-sampling-rules`, and all nine
`fig:*-timing` labels) against every chapter file. None of these labels are
referenced from any other chapter, so no `\ref{}` fixes were needed elsewhere.

## Cross-reference fixes found but NOT made (belong to other editors' files)

- `chapters/appendix-b-timing.tex`, line 10: "See Chapter~\ref{ch:cpu-architecture}
  for the bus signal timing, wait-state, and response-priority rules." This is now
  about content that lives in Appendix F, not Chapter 2. Should become
  `Appendix~\ref{appendix:external-interface}`.
- `chapters/06-exceptions.tex`, line 118: "...or when the `TRCE` input is asserted at
  the instruction boundary (see Chapter~\ref{ch:cpu-architecture}, Force Trace Mode
  Input)." The Force Trace Mode Input pin description is now in Appendix F, not
  Chapter 2. Should become `Appendix~\ref{appendix:external-interface}`.

(Not changed: `chapters/appendix-b-timing.tex` line 14, "For a detailed description of
each phase, see Chapter~\ref{ch:cpu-architecture}." — this is about the six-stage
phase model, which stayed in Chapter 2, so it is still correct as-is.)

## Verification

- `grep -c 'begin{'` / `grep -c 'end{'` match in both
  `chapters/02-cpu-architecture.tex` (13/13) and
  `chapters/appendix-f-external-interface.tex` (51/51).
- Brace counts (`{` vs `}`) balance in both files (78/78 and 427/427).
- `make quick` (run twice, to let cross-references resolve) succeeds with no errors.
  `appendix:external-interface` resolves correctly to `{F}{217}{External Interface}`.
  One pre-existing unrelated undefined-reference warning remains
  (`tab:l5-fault-timeline` in `chapters/06-exceptions.tex`), not introduced by this
  change and out of scope (not my file).
