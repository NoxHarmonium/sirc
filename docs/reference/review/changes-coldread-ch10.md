# Changes — Cold Read Follow-up — ch10 (condition-codes)

## Phase 6 (cold read)

Fixed the LO row in the "Unsigned Comparison (CMPR rA, rB)" truth table, which used
`\multicolumn{2}{c}{C = 0 OR Z = 1}` spanning the Z/C columns while every other row keeps them
separate. Replaced with per-column entries (`X` in Z, `X` in C, each marked with a footnote
symbol) plus a table note explaining that Z and C are not independently don't-care for this row:
LO is true when C = 0 OR Z = 1, and the excluded combination (Z = 0, C = 1) is the HI case.
This preserves the original predicate from finding F-con2-27 while matching the per-column shape
of the rest of the table.

Verified `\begin`/`\end` counts (24/24) and brace counts (158/158) balance.
