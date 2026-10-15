# Changes — appendix-d-examples.tex

## Findings applied
F-sum-1, F-sum-2, F-sum-3, F-sum-5, F-sum-7, F-sum-8, F-cons-10 (merges F-sum-9),
F-cons-11 (merges F-sum-13), F-sum-10, F-sum-11, F-sum-12.

All applied as written, with one label-name change carried through both the `.DQ`
comment line and the `:level2_handler` declaration in Example D-1's inline listing
(F-cons-10), verified to still assemble with `sirc-vm/target/debug/assembler`.

## Findings I could not apply as written
None — all eleven findings applied as specified with no textual drift from the quoted
claims.

## Additional copy-edit (STYLE.md conformance, not from triaged findings)
While in the file I also wrapped several bare hex literals in prose (outside
`lstlisting`) in the appropriate STYLE.md macro, since none of them were using the
macro form: `\addr{0x0000}` and `\addr{0xFFF0}` (Faults section, trampoline address),
`\addr{0x60}`/`\addr{0x61}` (exception vector numbers) and `\texttt{0xFF}`/`\texttt{0xBB}`
(register values) in the Software Exception section.

## Open questions for the user
- F-cons-15 and F-cons-16 in `triaged.md` also name `appendix-d-examples.tex` (missing
  return cross-reference to Chapter 11, and the `ch:`→`appendix:` label rename), but
  they were not in my task's finding list and both also touch
  `appendix-e-quick-reference.tex`, which I don't own. I left them untouched; flagging
  in case they were meant to be picked up by whoever edits Appendix E, or need explicit
  assignment.
