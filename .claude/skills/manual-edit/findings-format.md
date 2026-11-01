# Findings format

Every reviewer writes findings to its own file under `docs/reference/review/` and never to
anyone else's file. One finding per block, in this exact shape so the orchestrator can
concatenate and sort them mechanically:

```
### F-<reviewer-tag>-<n>
- file: reference/chapters/13-alu-instructions.tex
- lines: 120-124
- severity: blocker | major | minor | nit
- category: fact | contradiction | terminology | style | grammar | latex | structure | improvement
- claim: "<quote the manual text verbatim, trimmed to one or two lines>"
- evidence: <path:line in code or tests, or another chapter's path:line, or "none">
- resolution: manual-wrong | code-wrong | design | unclear | n/a
- fix: <exact replacement text, or a one-sentence description if the fix is structural>
- confidence: high | medium | low
```

`file` is relative to `docs/`, so it always names the manual: `reference/chapters/…`,
`hardware/chapters/…` or `assembly/chapters/…`. Several manuals have a chapter with the same
file name (`01-introduction.tex`), so a bare `chapters/…` path is ambiguous.

Severity scale:

- blocker: a manual contradicts itself, another manual, or the architecture on a normative fact
  (an encoding, a flag effect, a vector number, a privilege rule, a count, a pin or signal
  behaviour, an accepted assembler form).
- major: normative information is missing, ambiguous, or misleading enough that an implementer
  could build the wrong thing.
- minor: correct but unclear, inconsistent with the style guide, or awkward.
- nit: typo, punctuation, spacing, macro misuse with no semantic effect.

Source of truth: the manuals are canonical (see "Source of Truth" in
`docs/reference/manual-handover.md`). When a manual and the `sirc-vm` implementation disagree,
the default is that the implementation is wrong. Within the set, the architecture manual
(`reference/`) is authoritative for behaviour inside the CPU, the hardware manual for what is
driven or sampled on each pin, and the assembly manual for what the toolchain accepts.

`resolution` is only meaningful for `fact` and `contradiction`:

- `code-wrong`: the manual is clear and consistent and the implementation does something else.
  The manual stays; the gap becomes a simulator or toolchain todo.
- `manual-wrong`: the claim does not hold up when traced to first principles, or it contradicts
  a clearer statement elsewhere in the set, and there is an obvious correct form.
- `design`: two readings are both defensible and choosing between them is an architectural
  decision (for example, two chapters give different rules and neither is clearly a slip). The
  author decides; put the options and your recommendation in `fix`.
- `unclear`: you could not tell which side is wrong. Give both sides in `evidence`.

Rules:

- Quote, do not paraphrase, in `claim`. The editor applying the fix needs to find the text.
- One finding per issue. Do not bundle "and also" items.
- Do not report the same issue twice because it appears in two places; report it once and list
  the other locations in `fix`.
- `improvement` findings suggest something that would make a manual better without anything
  being wrong today: a missing example, table or diagram, a passage that could be shorter, a
  cross-reference a reader needs. They are never `blocker`, and they are never applied
  without the author's approval.
- Do not edit any file under `docs/*/chapters/` or `docs/reference/generated/`. Reviewers only
  report; the chapter editors apply.
