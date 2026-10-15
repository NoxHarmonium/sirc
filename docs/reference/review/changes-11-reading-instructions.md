# Changes — chapters/11-reading-instructions.tex

## Findings applied
- F-sum-15: reworded the Instruction Fields entry to drop the "facts digest"
  reference, replaced with a pointer to Chapter~\ref{ch:instruction-formats}.
- F-sum-16: reworded the `@label` notation-table row to say "a low-word
  address in other instruction operands," with the existing pointer to
  Section~\ref{sec:assembler-directives} kept for the full resolution rules.
- F-cons-5: reworded the LOAD status-override sentence to "its public syntax
  does not accept a status update override," matching Chapter 13's wording.

All three were applied as written; no faithful-approximation was needed.

## Could not apply as written
None.

## Open questions
- The Conditional Execution section (line ~108-110) states that a
  false-condition instruction leaves "the program counter... not modified."
  Chapter 10's own text makes the same claim and has an accepted fix
  (F-data-5, applied by that chapter's editor) noting that `p` still advances
  unconditionally at decode regardless of the condition, so "no register...
  changes" is misleading as written. F-data-5 is scoped only to
  chapters/10-condition-codes.tex in triaged.md, so I left this chapter's
  parallel sentence untouched rather than making an unscoped technical call.
  Worth a follow-up pass once the exact wording is settled in Chapter 10.

## Copy edit
Also standardized two internal-consistency issues while in the file (not
tied to a triaged finding): "Every normal instruction may be conditional"
now reads "Every real instruction may be conditional" (matching this
chapter's own "real CPU instruction" usage at line 23 and Chapters 16/17's
established real-vs-meta-instruction terminology), and "condition-code
field" was changed to "condition field" to match this chapter's own
dominant usage (used twice elsewhere in the same file, and matching the
STYLE.md-mandated field name "Condition field").
