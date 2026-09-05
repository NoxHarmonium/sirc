# LaTeX QA — Phase 5 (Build and Layout)

Reviewer: latex-qa (typesetter). Source: `make all` in `docs/reference/` (pdflatex,
3 passes, diagrams regenerated via `dot`), `main.log` from that build.

## Build status

`make format` (latexindent) and `make all` both succeed. Output: 246-page
`SIRC-1-reference-manual.pdf`. No `!` errors, zero undefined references/citations,
zero multiply-defined labels, zero "file not found" in `main.log`. One harmless
`pdfTeX warning (dest): name{lstnumber.-11.28} has been referenced but does not
exist` (a `listings` line-number anchor artifact, not a manual cross-reference) and
the usual per-pass `pdfTeX warning (ext4): destination with the same identifier`
duplicates from the 3-pass build — both expected, not defects.

## Fixed in this pass (chapters/ only)

- `chapters/06-exceptions.tex`: "Reset State" table (~line 610) and "Exception
  Coprocessor Instruction Summary" table (~line 698) used a plain `l` first/last
  column next to a wide fixed-width column, overflowing the text block by
  36.8pt and 10.6pt. Converted both to `tabularx` with a `>{\raggedright\arraybackslash}X`
  wrapping column so width is computed to fit `\textwidth`. No wording changed.
- `chapters/14-memory-instructions.tex`: "Legal Forms" table (~line 51) had an
  unwrapped `Notes` column, 93.0pt overfull. Same `tabularx`/`X`-column fix.
- `chapters/appendix-e-quick-reference.tex`: "Alphabetical Mnemonic Index" header
  row was 44.5pt overfull because of an unwrapped `Opcode(s)` column. Changed
  that column to a fixed `p{3.1cm}` and the table to `tabularx` with `X` for
  `Description`.
- Net effect: overfull `\hbox` count wider than 10pt went from 4 to **0**.
  Remaining overfull hboxes are all under 10pt (max 8.7pt, in prose, not tables)
  and are not visible defects. Underfull-hbox count moved from 18 to 20 (three
  low-badness loose lines, badness 1371–2035, inside the now-narrower
  `Opcode(s)` cells in appendix E) — cosmetic looseness only, no text overflow.

## Findings

### F-latexqa-1
- file: chapters/12-instruction-summary.tex
- lines: 21-104
- severity: major
- category: latex
- claim: "Complete Instruction List" table (Section 12.2, `\label{tab:complete-instruction-set}`), a 64-row `table[H]`/`tabular`, not a `longtable`.
- evidence: main.log:1993 "Overfull \vbox (58.24834pt too high) has occurred while \output is active []" on page 110.
- resolution: n/a
- fix: Named explicitly in manual-handover.md Workstream 13 as needing conversion to a multi-page `longtable`/`ltablex`. Confirmed still broken as of this build; requires a preamble package change plus table restructuring, out of scope for a chapter-only fix. Human/Workstream-13 owner should convert it.
- confidence: high

### F-latexqa-2
- file: chapters/13-alu-instructions.tex
- lines: 199-1231
- severity: blocker
- category: structure
- claim: All 12 `instructionbox` entries in the ALU chapter (SHFT, ADDI/ADDR, ADCI/ADCR, SUBI/SUBR, SBCI/SBCR, ANDI/ANDR, ORRI/ORRR, XORI/XORR, LOAD, CMPI/CMPR, TSAI/TSAR, TSXI/TSXR) now overflow the page.
- evidence: main.log overfull \vbox at pages 120/122/124/126/128/130/132/134/136/138/140/142, magnitudes 66pt to 427pt too high (worst: CMPI/CMPR page 138, 426.8pt — i.e. the box is roughly 1.6 pages tall).
- resolution: n/a
- fix: The three boxes named in manual-handover.md Workstream 13 ("XORI/XORR", "LOAD", "CMPI/CMPR") are confirmed still broken and now worse (they gained the three new per-entry fields plus example blocks). Every other entry in the chapter has the same problem. `tcolorbox` `instructionbox` is not `breakable`; fixing needs either enabling `breakable` in preamble.tex (out of scope here — preamble/STYLE only) or trimming entry content (a wording change, also out of scope). Recording for a human/Workstream-13 pass.
- confidence: high

### F-latexqa-3
- file: chapters/14-memory-instructions.tex
- lines: 145-350
- severity: blocker
- category: structure
- claim: "LOAD -- Load from Memory" and "STOR -- Store to Memory" instructionbox entries.
- evidence: main.log overfull \vbox on page 149 (637.6pt too high) and page 151 (616.8pt too high) — each box is roughly two pages tall.
- resolution: n/a
- fix: Worst overflow found anywhere in the manual. Same root cause and same non-chapter-fixable resolution as F-latexqa-2. These two boxes need to be split or made breakable before this chapter can be considered typeset correctly.
- confidence: high

### F-latexqa-4
- file: chapters/15-control-flow.tex
- lines: 138-645
- severity: major
- category: structure
- claim: 6 of the 7 instructionbox entries (BRAN, BRSR, RETS, LDEA, LJMP, LJSR) overflow the page; LDEL fits.
- evidence: main.log overfull \vbox on pages 159, 160, 162, 163, 165, 166 (9.3pt to 307.8pt too high; worst is LJSR, page 166).
- resolution: n/a
- fix: Same as F-latexqa-2.
- confidence: high

### F-latexqa-5
- file: chapters/16-coprocessor-instructions.tex
- lines: 73-1013
- severity: major
- category: structure
- claim: 8 of the 11 instructionbox entries (COPI/COPR, EXCP, and 6 others) overflow the page; chapter 16 gained 8 new examples per the handover, which is the direct cause.
- evidence: main.log overfull \vbox on pages 170, 172, 175, 179, 182, 183, 184, 186 (29.6pt to 308.1pt too high; worst around page 172, the EXCP entry).
- resolution: n/a
- fix: Same as F-latexqa-2.
- confidence: medium (exact mnemonic-to-page mapping beyond COPI/COPR and EXCP not individually re-verified against rendered pages)

### F-latexqa-6
- file: chapters/17-meta-instructions.tex
- lines: 43-92
- severity: minor
- category: structure
- claim: "NOOP -- No Operation" instructionbox, the only box in this chapter, restored with its own `\section{NOOP -- No Operation}` heading (line 41) matching the pattern used by every other instruction entry in chapters 13-17.
- evidence: main.log overfull \vbox on page 187 (109.6pt too high). Structural check: `\textbf{Applicability:}`, `\textbf{Instruction Fields:}`, and `\textbf{Condition field...}` lines are present and formatted identically (bold label + colon) to every other entry in chapters 13-17; field order matches the ch13-17 template.
- resolution: n/a
- fix: NOOP renders structurally like its siblings — the section heading and box wrapping requested in the handover are both present and correct. It shares the same non-breakable-box overflow as every other entry (same cause/fix as F-latexqa-2); no NOOP-specific defect.
- confidence: high

### F-latexqa-7
- file: chapters/appendix-a-opcode-map.tex
- lines: 8-92
- severity: major
- category: latex
- claim: "Complete SIRCIS Opcode Map" table (`\label` near line 92), a single `table[H]` covering all 64 opcodes.
- evidence: main.log:2388 "Overfull \vbox (407.58205pt too high)" on page 194 — nearly a full extra page of overflow.
- resolution: n/a
- fix: Same single-page-table problem as F-latexqa-1; needs `longtable`/`ltablex`, out of scope for a chapter-only fix.
- confidence: high

### F-latexqa-8
- file: chapters/appendix-e-quick-reference.tex
- lines: 20-104
- severity: major
- category: latex
- claim: "Alphabetical Mnemonic Index" table (`\label{tab:mnemonic-index}`), single `table[H]`, ~50 rows.
- evidence: main.log:2505 "Overfull \vbox (663.49947pt too high)" on page 228 — this is the worst single-page-table overflow in the manual (over 9 inches of excess height).
- resolution: n/a
- fix: Same as F-latexqa-1/7. This table is a stronger candidate for `longtable` than section 12.2 since it is purely tabular data with no other content sharing the page.
- confidence: high

### F-latexqa-9
- file: chapters/14-memory-instructions.tex
- lines: 166, 271
- severity: minor
- category: style
- claim: "\item \textbf{Condition (bits 3--0):} All 16 condition-field encodings are legal; see \textbf{Condition field} below."
- evidence: chapters/13-alu-instructions.tex:213 and every other Instruction Fields list in chapters 13, 15, 16, 17 use `\textbf{Condition field (bits 3--0):}` (with the word "field"); chapter 14's LOAD and STOR entries are the only two that drop it.
- resolution: unclear
- fix: Change both occurrences to `\textbf{Condition field (bits 3--0):}` for consistency with every other entry in chapters 13-17. Not applied here (task 4 asks to check and report this rendering-consistency item, and the fix is a two-word label/wording change).
- confidence: high

## Explicit checks (Workstream 13 / task item 3)

- "Complete Instruction List" (ch. 12, Section 12.2): checked — still a single-page
  table, still overfull (F-latexqa-1), unchanged in kind from the handover's report,
  slightly worse in degree.
- Named tall boxes "XORI/XORR", "LOAD" (ch. 13 ALU-context LOAD/move), "CMPI/CMPR":
  checked — all three still overflow and are now worse than the handover's report
  (see F-latexqa-2). The ch. 14 memory-context LOAD/STOR entries are worse still
  (F-latexqa-3), not previously called out by name but the same defect.
- NOOP (ch. 17): checked — renders identically to its siblings structurally
  (F-latexqa-6); no NOOP-specific defect beyond the systemic box-height issue.

## Formatting consistency (task item 4)

- `booktabs`: consistent everywhere — no `\hline` found in any chapter or
  `generated/*.tex`; every `tabular`/`tabularx` in the manual pairs `\toprule`/
  `\midrule`/`\bottomrule` correctly (verified by rule-count per table).
- `Applicability:`, `Instruction Fields:` labels: identical bold+colon rendering
  in every entry checked across chapters 13-17.
- `Condition field:` label: one drift found, see F-latexqa-9.

## generated/ (machine output, not edited)

`generated/immediate-format-encodings.tex`, `register-format-encodings.tex`,
`short-immediate-format-encodings.tex` — no overfull boxes; three low-badness
underfull `\vbox` warnings only (normal page-break looseness). No content
issues found.

## git diff --stat docs/reference

All modifications are confined to `docs/reference/chapters/*.tex` (22 files,
line-wrap/table-structure only) plus the rebuilt `SIRC-1-reference-manual.pdf`.
Nothing outside `chapters/` or `STYLE.md` changed; `STYLE.md` itself was not
touched. Nothing surprising.

## Pages a human should look at

- p.110 — Complete Instruction List (ch. 12) runs off the page.
- p.120-142 — every ALU instruction box (ch. 13); worst p.138 (CMPI/CMPR).
- p.146-151 — memory chapter Legal Forms + LOAD/STOR boxes; **p.149 and p.151
  are the worst layout breaks in the manual** (each box ~2 pages tall).
- p.159-166 — control-flow instruction boxes (ch. 15); worst p.166 (LJSR).
- p.169-186 — coprocessor/meta instruction boxes (ch. 16); worst p.172 (EXCP).
- p.187 — NOOP box (ch. 17), structurally fine but still overflows the page.
- p.194 — Complete SIRCIS Opcode Map (Appendix A).
- p.228 — Alphabetical Mnemonic Index (Appendix E), worst single-page-table
  overflow in the manual.

## Addendum: breakable instructionbox (applied after this report)

The orchestrator applied the fix F-latexqa-2 through F-latexqa-6 called for:
`preamble.tex` now loads `\tcbuselibrary{breakable, skins}` and the
`instructionbox` environment sets `breakable, enhanced jigsaw`. Rebuilt with
`make all`: overfull `\vbox` count dropped from 12 (spanning ch12-17 and
appendices A/E, worst case 663pt) to 4, and the page count fell from 246 to
242. Every instruction-box overflow (F-latexqa-2, F-latexqa-3, F-latexqa-4,
F-latexqa-5, F-latexqa-6) is resolved; boxes now break across a page instead
of forcing the page to grow. This was a preamble-level typesetting fix, not a
content change, so it was in scope for this phase.

The four remaining overfull vboxes are the single-page-table class
(F-latexqa-1, F-latexqa-7, F-latexqa-8) plus one newly-visible instance:

### F-latexqa-10
- file: chapters/16-coprocessor-instructions.tex
- lines: 42-61
- severity: minor
- category: latex
- claim: "Common Coprocessor Instruction Semantics" table (`\label{tab:coprocessor-common-semantics}`), a 9-row `table[H]` with a `p{9cm}` Definition column.
- evidence: main.log, page ~165, "Overfull \vbox (75.06796pt too high)". The table's content is genuinely long (9 rows of multi-sentence policy text at fixed 9cm width); this is a page-height problem, not a column-width problem, so the `tabularx` fix used elsewhere does not apply.
- resolution: n/a
- fix: Same class of defect and same resolution as F-latexqa-1/7/8: needs `longtable` (or `[H]` relaxed to `[htbp]`, if STYLE.md's convention allows a non-`[H]` float here) so the table can break or move rather than forcing page overflow. Out of scope for a chapter-only fix; fold into the Workstream 13 longtable-conversion item alongside the instruction-summary, opcode-map, and mnemonic-index tables.
- confidence: high

Updated finding count: 10 total (was 9). Remaining defects are all in the
single-page/oversized-table class now, tracked as one Workstream 13 item
(F-latexqa-1, F-latexqa-7, F-latexqa-8, F-latexqa-10) needing `longtable`.
