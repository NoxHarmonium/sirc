# Phase 6 improvement brief

The author picked these cold-read improvements to act on now. The full findings are in
`docs/reference/review/cold-read-<manual>.md`; read the ones your prompt names. Follow `docs/reference/STYLE.md`
(American spelling, no second person, `\activelow{}` for active-low signals, Title Case captions, `\archmanual{}`,
`\hwmanual{}`, `\asmmanual{}` for references between manuals, `% qa:assemble` before real assembly listings).

You own only the files your prompt names; other agents are editing other files at the same time. Do not change the
architecture: every fact you write must already be stated somewhere in the manual set (cite it to yourself as you go),
or come from the author's decisions in `review/triaged.md` (`decision:` lines) and `review/phase3-decisions.md`. If a
fact you need is not stated anywhere, leave a `% TODO(author):` comment in the source and list it in your change log
instead of inventing it.

Do not run `make`: other agents share the build directory. Run latexindent on each file you edit, from its manual's
directory, and confirm `begin{`/`end{` counts match. Append an "Improvements" section to each edited file's change log
`docs/reference/review/changes-<manual>-<basename>.md` (create it if missing). Finish with at most five lines.
