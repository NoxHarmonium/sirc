# Changes — chapters/10-condition-codes.tex

## Findings applied
- F-data-5: amended both occurrences ("If the condition is not met..." near the top, and the
  Never (NV) entry) to say the instruction has no architectural effect beyond the normal
  two-word advance of `\reg{p}`, and changed "no register" to "no general-purpose register"
  to match the accepted wording.

## Findings applied with a variation
- None. Both fixes were applied close to the wording given in triaged.md; the NV entry needed
  light rephrasing to fit the existing sentence structure (splitting into "no architectural
  effect beyond..." plus the existing "Can be used for..." sentence), not a substitution of
  meaning.

## Open questions
- None.

## Other copy-edit
- Added terminal periods to the six bullet items under "For unsigned comparisons" — all are
  full sentences and were inconsistently punctuated (five without periods, one with).
