# Changes: reference/chapters/10-condition-codes.tex

## Findings applied
F-con-12 and F-data-14 (merged into the Performance Considerations bullets), F-data-15, F-data-16, F-data-17, F-data-18 (applied to every bare CMP mention, including the selection-table header and the Note, not only the listed lines).

## Not applied as written
- F-con-12 also asks for a relabel in appendix-b-timing.tex. That file belongs to another editor and was not touched.
- F-data-16: the selection table's Intent column now uses r1/r2; the column header "After CMP" became "After Compare".
- F-data-14/F-con-12 were split into three bullets (wait states, processing-unit versus coprocessor, branch saving) to keep one fact per bullet. The old duplicate "still takes 6 cycles" bullet was removed.

## Open questions
- The Loop Construction example uses BRAN|HI after SUBI, where HI tests C = 0 AND Z = 0 (a borrow-based test). Whether this is the intended i > 0 test is a technical point, left unchanged.

## Improvements

- F-CR-12: Table 10.1 is now the canonical condition-code table; added the legacy label tab:condition-code-encoding to it (no content change).
