# Changes — chapters/15-control-flow.tex

## Findings applied
- F-cons-17: Wrapped all 16 literal opcode-byte values in `\opcode{}`: the eight
  `Encoding` cells in Table~\ref{tab:control-flow-legal-forms} (0x18-0x1F) and the
  eight values in the LDEA/LDEL "Opcodes:" lines (0x18-0x1B, 0x1C-0x1F). Left the
  `0x0000` immediate/offset placeholder values (in bytefield diagrams, instruction
  fields, and lstlisting comments) as plain `\texttt{}` or listing text, since those
  are not opcode bytes.

## Findings not applied as written
- None. F-cons-17 applied cleanly as written for this file.

## Other STYLE.md conformance fixed while in the file
- Reordered the `Flags:`/`Write-back:` pair to `Flags:` then `Write-back:` in all
  seven instruction boxes (BRAN, BRSR, RETS, LDEA, LDEL, LJMP, LJSR). STYLE.md §4
  gives the canonical field order as `... Description -> Flags -> Write-back ->
  Exceptions ...`, matching the de facto order used throughout Chapter 13; this
  chapter had Write-back before Flags in every entry. No wording changed, only the
  order of the two one-line fields.

## Open questions
- None. The rest of the chapter's prose already conformed to STYLE.md (normative
  vocabulary, "for example" vs "e.g.", cross-reference form, register/opcode
  macros, list punctuation) on inspection, so no further copy-edit changes were
  made.
