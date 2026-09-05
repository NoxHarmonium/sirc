# Changes: appendix-d-examples.tex

## Findings applied
None applied in this session. The interrupted prior run had already applied the two
findings that require changes inside this .tex file:

- **F-sum-29** — caption numbering. All seven `\lstinputlisting` captions already carry
  `Example D-1:` through `Example D-7:` prefixes (Byte Sieve, Faults x2, Hardware Exception
  x2, Software Exception, Store/Load), matching G1-T4/STYLE.md numbering. The finding text
  says "six listings D-1 through D-6," but the file has seven listings; the prior run
  numbered all seven sequentially, which is the correct application of the underlying rule.
  The fix also asks to "refer to them by number in the section text" — there is no such
  prose in this file (only chapter/section headings and listing captions), so no further
  action was needed for that clause.
- **G1-T4-ap** (appendix-d) — same numbering requirement as F-sum-29; already satisfied by
  the caption prefixes above.

No explanatory prose survives around the listings in this chapter (it is section headings
and `\lstinputlisting` blocks only), so Gate 2 rulings B (unsigned glosses), O (shift applies
to register operand), and F-mem-12 (PC-relative displacements) have no prose to apply to in
this file.

## Findings skipped, and why
- **F-sum-7** — status `code-wrong`; skipped per explicit instruction (the `EXCP #0x40`
  example is a code/runtime-guard issue, not a manual defect).
- **F-sum-30** — status `code-wrong`; skipped per explicit instruction (vector-count comment
  is a source-code fix, not a manual defect).
- **F-sum-31** — status `accepted`, but the fix text targets
  `examples/hardware-exception/hardware-exception.sasm:43` (a comment inside the assembly
  source), not this `.tex` file. Editing under `examples/` is explicitly out of scope for
  this pass, so left untouched.
- **F-sum-32** — status `accepted`, but the fix targets
  `examples/faults/faults.sasm` (renaming `.EQU` constants and comments in the source).
  Out of scope under `examples/`; left untouched.
- **F-sum-34** — status `accepted`, but the fix targets
  `examples/byte-sieve/byte-sieve.sasm:19` (a stray comment copied from the line above).
  Out of scope under `examples/`; left untouched.

## Open questions
- F-sum-31, F-sum-32, and F-sum-34 are marked `accepted` in triage but their fixes all live
  in `.sasm` files under `examples/`, which this pass is barred from touching. Someone with
  write access to `examples/` (or a future pass explicitly scoped to that directory) still
  needs to apply them; flagging so they aren't silently dropped.
- F-sum-29's fix text refers to "six listings D-1 through D-6," but the chapter has seven
  `\lstinputlisting` blocks. I numbered/left the existing D-1 through D-7 sequence as-is
  since it's the more literal application of the underlying "number every example" rule;
  confirming this reading would be useful if the discrepancy was intentional (e.g., two
  listings meant to share one number).
