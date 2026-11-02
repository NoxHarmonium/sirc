# Phase 5 layout fix brief

You own exactly one chapter file, named in your task prompt. Fix only the layout problems the prompt lists, which come
from `docs/reference/review/latex-qa-<manual>.md`. Do not change the meaning or wording of any sentence, except where a
line in a verbatim block or table cell has to be broken or shortened to fit; then keep every word or abbreviate only in
an obvious way, and log it.

Prefer, in order: letting a table column wrap (`p{}`/`X` columns in `tabularx`, smaller `\small` or `\footnotesize` only
if the table already uses a reduced size elsewhere in the set), breaking a long verbatim or listing line at a natural
point, splitting a long `\texttt` with `\allowbreak` or `\linebreak`. For TikZ or bytefield figures, shorten a label
across two lines (`\shortstack`) rather than shrinking the font.

Then, from the manual's directory: run latexindent on your file, run `make all`, and confirm in `main.log` that the
overfull box you were fixing is gone or under 10pt (grep for your chapter's file name and line). Render the page with
`pdftoppm -r 80 -f N -l N` and view it with Read to confirm. Append a "Layout" section to your change log
`docs/reference/review/changes-<manual>-<basename>.md`. Finish with a two-line summary.
