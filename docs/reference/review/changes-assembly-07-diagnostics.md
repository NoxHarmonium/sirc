# Changes: assembly/chapters/07-diagnostics.tex

## Findings applied
F-asm-18, F-asm-19, F-con-36 (column "You wrote" renamed "Source"). Decision 7 (verbatim diagnostics) applied to Table 7.1.

## Applied differently, or not applied
- F-asm-17 is status `simulator`: left alone. The listing is unchanged.
- F-asm-19: the row text is moved into a paragraph after the table, because the aliased-write diagnostic carries a per-form detail phrase. The table row quotes one real message (`LOAD ah, (#0, a)+`). The paragraph points to the Memory Access Instructions and Control Flow Instructions chapters of the architecture manual.
- Table 7.1 now has three columns (Source, What the Assembler Says, Meaning). Messages were verified by assembling each example. The ADDR shift-range message is wrapped in a `Parsing Failure: Base { ... }` string today; the table quotes the inner message and a note says so. The `MOVR` message contains the typo `MOVR:.`, quoted as printed. The assembler appends "at line N, column M"; a sentence says so.
- Caption changed to Title Case.

## Open questions
- The toolchain todo (report errors without panicking) will change the surrounding output; the table text should be rechecked after it.
- The `Invalid addressing mode for MOVR:.` typo and the `Parsing Failure: Base {...}` wrapping are toolchain defects; if fixed, update the table.
