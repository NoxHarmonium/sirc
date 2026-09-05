# Changes — chapters/07-instruction-formats.tex

## Findings applied

All 12 findings triaged against this chapter are `status: accepted`; all were applied.

- **F-con-1 / F-enc-1** — Corrected the condition-code table (`tab:condition-code-encoding`):
  row `1100` mnemonic `<` → `<<`, row `1110` mnemonic `<<` → `<=`, and row `0000` `(none)` →
  `AL (or none)`, to match Chapter 10's table and every use site in the manual.
- **F-enc-2** — Corrected the "Test vs. Save" pattern: the test-only range is
  `0x_8`–`0x_E` (not `0x_8`–`0x_F`); `0x_F` is the coprocessor call, not a test-only
  form; the "add `0x8`" shortcut only holds for `0x_0`–`0x_6`. Applied with the **Gate 2 ruling
  H** adjustment: the fix's own evidence would have documented `0x2F` as a coprocessor call, but
  ruling H states `0x27` and `0x2F` are undocumented opcodes, same class as
  `0x08`/`0x09`/`0x0B`/`0x0D`. Added a sentence carving out `0x27`/`0x2F` as exceptions to the
  general pattern, with a cross-reference to Appendix C, rather than asserting `0x2F` is a
  coprocessor call.
- **F-enc-3** — Short Immediate Format "Immediate" field: changed "8-bit signed or unsigned" to
  "8-bit unsigned ... zero-extended to 16 bits before use."
- **F-enc-4 / F-con-9** — Fixed the register-operand-defaulting note and code comment: "If R3 is
  not specified" → "If only two register operands are given"; the two-operand `ADDR r1, r2`
  comment now reads "(R2 implicitly = R1)"; the parallel "Notes:" bullet under the Register
  Format encoding table now reads "R2 is encoded as a copy of R1 and the second operand is
  encoded in R3"; fixed the typo "the assembler with use" → "the assembler will use."
- **F-enc-5** — Rewrote the instruction-fetch steps: PC is not incremented twice during fetch;
  the high word is fetched from `pl`, the low word from `pl + 1`, and `pl` advances by 2 only
  after decode. Added the note that `pl` still holds the executing instruction's address during
  decode, since address calculations that select `p` use that value.
- **F-enc-6** — Added the LDEA/LDEL register-field exception (2-bit address-register-pair index
  in bits 23–22, bits 25–24 unused) to the "Register" field description in both the Immediate
  Format and the Register Format sections.
- **F-con-62** — `meta instruction` → `meta-instruction` (both occurrences, in the Short
  Immediate and Register Format notes).
- **F-enc-7** — Replaced the fragile row-number references ("rows 7 and 8", "SHFT row 9") in the
  Register Format notes with row names ("the `SUBR|>>` and `ANDR|CS` rows", "the `SHFT` row").
  Left the parallel Short Immediate Format notes ("rows 4 and 6", "SHFT row 7") unchanged: I
  checked those row numbers against the generated encoding table and they are currently
  correct, so this finding's underlying defect does not apply there and rewriting would be a
  cosmetic-only change outside the finding's scope.
- **F-enc-8** — Retitled "ALU Instructions" to "Opcode Groups" and added a sentence scoping the
  "Test vs. Save" pattern to the `0x0_`, `0x2_`, and `0x3_` groups only, so the `0x1_` memory
  group is no longer presented as an ALU group.
- **G1-T4-07** — Checked for `Example:` blocks to number per the Gate 1 template decision; this
  chapter has none (its "Encoding Examples" subsections are tables, and the "For example:" list
  under Opcode Groups is not an instruction-entry example block), so there was nothing to
  renumber here.

## Mechanical / copy-edit changes (not tied to a specific finding)

- Wrapped bare hex literals in `\texttt{0x...}` (condition-code examples, `R1=0x4` etc.), used
  `\opcode{25}` for the ORRI opcode literal, and used `\imm{-2}` for the negative-immediate
  example, per STYLE.md §3.
- Converted the two `0b10` binary literals (AF field examples) to the bare form `10`, per
  STYLE.md's explicit note that this file was the outlier.
- `e.g.` → "for example" (one instance).
- Bolded three bare `Note:` labels to `\textbf{Note:}` to match the convention used everywhere
  else in the manual.
- Rewrote "Only supported for ALU instructions - cannot be used with any of the branching, or
  load/store instructions" as two clauses joined by a semicolon, fixing the missing subject,
  the stray hyphen-as-dash, and the awkward parallelism.
- "Specifies under what conditions the instruction should execute" → "Specifies the conditions
  under which the instruction executes" (removed a non-normative "should" describing mandatory
  decode behavior).
- Aligned the Register Format's "Condition (bits 3–0)" field description to "4-bit condition
  code," matching the wording used in the other two format sections.
- "don't save result" → "does not save the result" for consistency with the adjacent normative
  phrasing.

## Open questions

- None. All ambiguities encountered (opcode 0x2F's true behavior, LDEA/LDEL field layout) were
  resolved by the Gate 2 rulings or by the findings' own evidence; no open technical questions
  remain for this chapter.

## Verification

`grep -c 'begin{'` and `grep -c 'end{'` both return 35; brace counts in the file are balanced
(262 `{` / 262 `}`). A single-pass `pdflatex` build of the full manual completed with no `!`
errors after these edits.
