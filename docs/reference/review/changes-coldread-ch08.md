# Changes: cold-read follow-up, ch. 08 (Addressing Modes)

## Issue
The cold reader flagged that the two addressing-mode legality tables — "Addressing
Mode Side Effects and Legal Instruction Families" (Table~\ref{tab:addressing-mode-matrix},
mode-first) and "Addressing Mode Legality by Instruction Family"
(Table~\ref{tab:legal-modes-by-family}, instruction-first) — sit back to back with no
guidance on which one to consult for a given question, or that they must agree.

## Fix
Added a two-sentence bridging paragraph immediately after the "Legal Modes by
Instruction Family" section heading and before its table, explaining that the
mode-first table answers "what does this syntax mean, and which families accept it
in general," the instruction-first table answers "for this instruction, exactly which
forms are legal," and that the two must agree. No table content was changed or
restated.

## Verification
`grep -c 'begin{'` and `grep -c 'end{'` both return 38; brace counts in the file are
balanced (320 open / 320 close).
