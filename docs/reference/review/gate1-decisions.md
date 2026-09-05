# Gate 1 decisions (2026-09-05)

## Style rulings
Recorded in `docs/reference/STYLE.md` under "Approved decisions".

## Template-field findings to apply in Phase 3
Each is a `major` finding against every chapter with instruction entries
(13-alu-instructions, 14-memory-instructions, 15-control-flow, 16-coprocessor-instructions,
17-meta-instructions). Merge into `triaged.md` as `status: accepted`, `resolution: manual-wrong`,
`confidence: high`, source `gate1`.

- G1-T1 (major): rename the `Condition codes:` line in every instruction entry to
  `Condition field:`. Source: period-gaps.md item 3.
- G1-T2 (major): add an applicability line under every instruction entry title
  (`SIRC-1, all revisions`, or the coprocessor revision per Chapter 16). Source:
  period-gaps.md item 13.
- G1-T3 (major): add an Instruction Fields list to every instruction entry after the opcode
  list, one line per encoding field with its legal values here. Source: period-gaps.md
  item 2. Values from `review/facts.md` and Chapter 7 only.
- G1-T4 (minor): number every example block `Example N-M:` per STYLE.md. Applies to all
  chapters, not only instruction chapters.
- G1-T5 (minor): Chapter 11's flag-symbol table must define `U` (undefined).

## Backlog
Section gaps and the two rejected template fields are recorded as Workstream 14 in
`docs/reference/manual-handover.md`.

## Facts-digest open questions
All sixteen in `review/facts.md` become `unclear` findings; none were resolved at Gate 1.
