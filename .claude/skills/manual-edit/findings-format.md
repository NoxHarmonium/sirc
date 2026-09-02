# Findings format

Every reviewer writes findings to its own file under `docs/reference/review/` and never to
anyone else's file. One finding per block, in this exact shape so the orchestrator can
concatenate and sort them mechanically:

```
### F-<reviewer-tag>-<n>
- file: chapters/13-alu-instructions.tex
- lines: 120-124
- severity: blocker | major | minor | nit
- category: fact | contradiction | terminology | style | grammar | latex | structure
- claim: "<quote the manual text verbatim, trimmed to one or two lines>"
- evidence: <path:line in code or tests, or another chapter's path:line, or "none">
- resolution: manual-wrong | code-wrong | unclear | n/a
- fix: <exact replacement text, or a one-sentence description if the fix is structural>
- confidence: high | medium | low
```

Severity scale:

- blocker: the manual contradicts the implementation, the tests, or itself on a normative fact
  (an encoding, a flag effect, a vector number, a privilege rule, a count).
- major: normative information is missing, ambiguous, or misleading enough that an implementer
  could build the wrong thing.
- minor: correct but unclear, inconsistent with the style guide, or awkward.
- nit: typo, punctuation, spacing, macro misuse with no semantic effect.

Rules:

- Quote, do not paraphrase, in `claim`. The editor applying the fix needs to find the text.
- `resolution` is only meaningful for `fact` and `contradiction`. When the manual and the code
  disagree, do not assume the code is right. Tests are stronger evidence than implementation
  code, and decisions recorded in `docs/reference/manual-handover.md` override both. If you
  cannot tell which side is wrong, say `unclear` and give both sides in `evidence`.
- One finding per issue. Do not bundle "and also" items.
- Do not report the same issue twice because it appears in two places; report it once and list
  the other locations in `fix`.
- Do not edit any file under `docs/reference/chapters/` or `docs/reference/generated/`.
  Reviewers only report; the chapter editors apply.
