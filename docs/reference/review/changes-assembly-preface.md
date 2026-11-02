# Changes: assembly/chapters/preface.tex

## Findings applied
- F-con-36 (second person removed: "assumes the reader has read ... and is comfortable")
- F-con-37 ("the architecture manual" replaced by `\archmanual{}`, both occurrences)
- Behaviour spelling finding (behaviour to behavior)

## Not applied as written
- Manual-set table instruction: this preface has no manual-set table, so nothing to change.
- `\asmmanual` is not defined in assembly/main.tex (only `\archmanual`, `\hwmanual`), so the own-title mention in Revision Status stays plain text.
- Copy edit: added Oxford comma in "the assembler, the linker, and the simulator".

## Open questions
- Define `\asmmanual` in assembly/main.tex so the Revision Status can use it.
