# Changes: assembly/chapters/04-instructions.tex

## Findings applied
F-asm-25, F-con-27, F-con-37 (this chapter's hits: text and table header), Gate 1 decision 5 (syntax line rewritten as `<MNEMONIC>{[<SUS>]}{|<COND>} <OPERAND>, ...`), Gate 1 decision 4 (captions to Title Case), STYLE.md §3 (`|cond` in Conditions section changed to `|<COND>`).

## Not applied as written
F-asm-16: status `simulator`, left alone. The table header became `\archmanual{} Chapter`, the closest Title Case form of F-con-37.

## Open questions
None.

## Layout

Tables 4.1 and 4.2: Table 4.2 converted to tabularx (full width, wrapping Mnemonics column, 3.3cm chapter column); the shift conditions are written <{}< and >{}> so they print as two signs rather than T1 guillemets. Overfull boxes of 62, 115, 59 and 109pt in this chapter are gone (manual total 6 -> 2, remaining 7.3pt and 5.4pt are in other files).
