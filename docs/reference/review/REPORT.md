# Editorial pass report (2026-11-02)

Scope: all three manuals (architecture reference, hardware reference, assembly programmer's manual). Fourteen commits on
`human-proofread`, from 4614c8ea (phase 1) to this report; 46 source files changed (+2,014 / -1,404 lines).

## Findings

| Phase | Source | Findings |
|---|---|---|
| 2 | 11 fact-check groups + cross-manual continuity check | 268 (7 blocker, 110 major, 126 minor, 25 nit) |
| 2 | Verify (117 blocker/major fact and contradiction findings) | 117 confirmed, 0 refuted |
| 4 | Continuity re-check after editing | 24 (16 contradiction, 7 terminology, 1 style) |
| 5 | LaTeX QA, one per manual | 8 overfull boxes over 10 pt, 1 figure overlap, 1 ligature |
| 6 | Cold read, one per manual | 60 reader-experience findings, 23 improvements |

Triage of the 268: 194 accepted and applied, 63 left to the simulator (the manual stays), 11 merged as duplicates.

By manual (phase 2, by the file each finding is filed against): architecture 192, hardware 40, assembly 36.

## What was applied

- Every accepted finding, in 41 chapter files, plus the author's design decisions (`triaged.md` `decision:` lines and
  `phase3-decisions.md`): coprocessor ID 0 = 6 cycles; discarded EXCP = 12; EXCP privilege check at phase 3; 24 MHz is an
  example only; three-register tests rejected; AF = 11 undefined flags; undefined encodings still privilege-bound;
  0x27/0x2F undocumented; DMA backward 0 = no-op; DMAT checks both advances; traced-EXCP trap held by the exception unit;
  L5 overrun on failed fetch; HALT at every phase 0 (after the RSTO hold); triple fault = software reset; held NMI does
  not fire after reset; registers undefined after reset; SYNC rule; idle bus driven (BAT 000, address undefined); DMA
  bus-cycle section; clock-edge model; overbars set-wide; `displacement`/`#disp`; .ORG per object; .EQU before use;
  labels declared once.
- STYLE.md extended to all three manuals; `\activelow`, `\archmanual`, `\hwmanual`, `\asmmanual` added to the preamble.
- Improvements the author chose: exception summary table, forward references, canonical copies of duplicated tables,
  phase/slot terms and legend table, clock-edges section with sample points on every timing figure, word-address and
  notation keys, assembly syntax-card appendix.

## Deferred

- 63 simulator findings, most matching existing `general-todo.md` items 1-34, plus 18 new items 35-52 for the
  remaining gaps and the toolchain/simulator changes the author's decisions require. Highlights: 37 (ID-0 CSR), 38 (SYNC), 39 (RSTI direction, triple fault), 40 (NMI across reset), 45
  (three-register tests), 46 (duplicate labels), 47 (linker 0x12/0x16 panic), 51 (read-only PROGRAM), 52 (LDEL panic).
- Period coverage backlog: handover Workstream 14, items 10-24 from the benchmark. Cold-read improvements not chosen
  (assembly setup steps and a second worked example, hardware wiring example, Ch.12 flags in Ch.11 symbols) are in
  `cold-read-*.md`.

## Pages needing a visual check

- Architecture: PDF p.58 (exception summary table, small type), p.178 (64 pt vbox, renders fine), p.119 and p.139
  (awkward tails), p.39 (Figure 3.2 label, fixed).
- Hardware: pp.12-13 (Figure 4.1 pushed, half-empty page), pp.18-19 (Figure 5.10 split from its text), pp.3-4 (Table 2.1
  pushed), and the reset figure 5.9 (last sample mark at the edge).
- Assembly: Appendix A syntax card (4 pages), Table 7.1.

## Open for the author

Hardware: what unconnected or unused pins do, and pin states at power-up (no source in the manuals). Hardware Ch.5: the
inferred meanings of the Enter, active, Mem and next figure labels.
