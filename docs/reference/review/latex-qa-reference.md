# LaTeX QA: architecture reference manual

Build: pass. `make format` changed no chapters; `make all` exits 0, 269 pages, no undefined references or citations,
multiply-defined labels or missing files. `qa.sh`: pass (74 listings assemble; the Appendix D byte-sieve output matches
expected.hex).

Warnings: 278 overfull (277 hbox, 1 vbox), unchanged since f5a19d0d; 273 before phase 4 (84dd4547). The five added are
three 1.3pt hboxes plus items (a) and (b) below.

Over 10pt:
- (a) New: Ch.13 line 315, SHFT Operation verbatim "[S] shifter result (default for SHFT)", 10.1pt, PDF p.138.
- (b) New: Ch.15 lines 30-59, Legal Control-Flow table, 17.5pt, PDF p.184.
- (c) Pre-existing: Ch.16 line 322, EXCP pseudocode "link[0] = { address of next instruction, sr,
  current_exception_level }", 43.0pt, PDF p.208; visibly past the right margin.
- (d) Pre-existing: overfull vbox 64.3pt after page [177], most likely PDF p.178 (STOR page); not visibly clipped.

Under the threshold but visible: Ch.3 Figure 3.2 (PDF p.39), "High register [15:8] (not used for addressing)" overflows
its cell and overlaps the bit ruler, the "[7:0]" label and the "bits 23-16" caption.

Named tables render cleanly: ALU legal forms (p.134), Appendix B Table B.1 including the COPI/EXCP/RSET/WAIT rows
(p.238), Complete Instruction List (p.128). No instructionbox starts at a page bottom. Awkward page tails: p.119 (a lone
bullet from 10.10), p.139 (SHFT notes spill), p.36 (2.5 spill). No literal LaTeX or "??" in the PDF text.

Pages for a human to look at: PDF 39, 208, 178, 138, 184, 119, 139.
