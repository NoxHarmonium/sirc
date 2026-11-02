# Changes: hardware/chapters/01-introduction.tex

## Findings applied
F-con-4 (manual-set table scope, using the wording specified by the caller), the manual-title macro finding (`\archmanual`, `\hwmanual`, `\asmmanual`), the active-low overbar finding (signal-groups table), the American spelling finding (behaviour to behavior).

## Not applied as written
F-con-4 gives "symbols, and the toolchain"; the caller's wording "symbols, meta-instructions, and toolchain usage" was used instead. The `\activelow` rule was applied to the signal-groups table only; the chapter has no other signal names in prose.

## Open questions
None. Also changed without a finding: captions to Title Case, and "the architecture manual" to `\archmanual{}`.

## Improvements

- F-CRH-1: added Section "Execution Phases and Slots" (sec:execution-phases) with a six-phase table (bus access per phase) and brief definitions of phase 0, slot, dispatch slot and continuation slot, pointing to the CPU Architecture chapter of the Architecture Manual.
