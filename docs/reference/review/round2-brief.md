# Phase 4 fix round brief

You own exactly one chapter file, named in your task prompt. Apply, to that file only:

1. Every finding in `docs/reference/review/consistency-2.md` whose `file:` is your chapter (path relative to `docs/`). A finding with `resolution: design` is settled by the author decisions below; follow the decision, not the finding's options.
2. Every decision in `docs/reference/review/phase3-decisions.md` that concerns your chapter (including the "Orchestrator fixes" paragraph). Decision 9 replaces the "held in the CSR" wording for the traced-EXCP trap.
3. Signal names in the architecture manual were just changed from `\texttt{BERR}` to `\activelow{BERR}` by a global replacement; if a sentence still says these names are in typewriter font, correct it.

Locate text by the quoted `claim`, not line numbers. Keep edits minimal; do not re-copy-edit the chapter. Then run latexindent on your file from its manual's directory, confirm `begin{`/`end{` counts match, and append a "Round 2" section to your existing change log `docs/reference/review/changes-<manual>-<basename>.md` listing what you applied and any open question. Finish with a two-line summary.
