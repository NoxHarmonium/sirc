# Changes: reference / 05-status-register.tex

## Findings applied
F-data-7, F-data-8, F-data-9, F-data-10, F-data-11, F-data-12, F-data-13, F-con-19.

## Not applied as written
- F-data-12: the Memory and Address bullet reads "Never update the condition flags as a side effect" (words "as a side effect" added) so it agrees with the accepted LOAD-into-sr finding for the Memory Instructions chapter.
- F-data-4, F-data-5, F-data-6: status `simulator`, left alone.

## Copy edit
"will be serviced" to "is serviced"; "it is recommended to" to "software should"; "undefined" to "architecturally undefined" for bits 7--4; "user code" to "protected-mode code"; ranges `1-4` and `E1-E4` given en dashes; Purpose line in the A bit reworded.

## Open questions
- The accepted F-data-10 text writes "bits 7-0" with a hyphen inside a listing comment (comment, so allowed by STYLE).
- The line 5 NMI "Level 5" wording in the E1--E4 list uses interrupt-line levels, while the new sentence says line 5 runs at exception level 6. Author may want the list relabeled.

## Round 2
- Decision 5: E1--E4 list relabeled "Line n (exception level n+1)"; NMI line 5 labeled exception level 6; the now-redundant "line 5 runs at level 6" clause removed from the following sentence. "Priority level" in the interrupt-preemption bullet changed to "exception level".
- No consistency-2 findings or other decisions apply. Open question from round 1 (list relabeling) is resolved.

## Improvements
- F-CR-9: gloss for [N]/[A]/[S] suffixes at first use, ref ch:notation-glossary (AF already referenced ch:instruction-formats).
