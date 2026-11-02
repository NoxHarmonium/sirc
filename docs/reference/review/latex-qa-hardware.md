# LaTeX QA: hardware reference manual

Build: pass. `make format` clean; `make cleanall` + `make all` (3 passes, 33 pages). 0 undefined references or
citations, 0 multiply-defined labels, 0 missing files, 0 LaTeX warnings. No chapter source changed. No listing check
(this manual has no listings).

Warnings: 0 overfull vbox. 1 overfull hbox at 7.28pt in chapters/01-introduction.tex lines 10-19 (below the 10pt
threshold). 4 underfull hboxes at chapters/04-bus-interface.tex line 87 (Table 4.3, "BACK samples data;" cell, p.12).

Literal LaTeX in the PDF text: none.

Visual: Figure 2.1 pinout (p.3), Figure 4.1 DMA read (p.13) and Figures 5.1-5.10 (pp.15-19) render correctly; every
active-low label has its overbar and nothing is clipped. No table or box is split across pages.

Pages for a human to look at:
- p.12-13: the Section 4.4 heading and two bullets fall in the last quarter of p.12, Figure 4.1 is pushed to p.13, and
  half of p.13 is blank.
- p.18-19: Figure 5.10 is separated from the text that introduces it.
- p.3-4: Table 2.1 is pushed off p.3.
