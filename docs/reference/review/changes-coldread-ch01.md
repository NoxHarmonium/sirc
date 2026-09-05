## Phase 6 (cold read)

- Coprocessors section: added a forward reference to `\mnemonic{COPI}`/`\mnemonic{COPR}`'s first
  mention, pointing to Chapter~\ref{ch:coprocessor} (the coprocessor instructions chapter),
  so readers know where these instructions are fully defined.
- Key Features table: expanded the "Address Register Pairs" row from `4 (l, a, s, p)` to
  `4 (l, a, s, p -- link, address, stack pointer, program counter)`. Names were verified against
  Chapter 3 (`03-registers.tex`), which defines the Link Register (l), Address Register (a),
  Stack Pointer (s), and Program Counter (p) register pairs, so no new facts were introduced.
- Verified `\begin{}`/`\end{}` counts (12/12) and overall brace counts (110/110) remain balanced
  after both edits.
