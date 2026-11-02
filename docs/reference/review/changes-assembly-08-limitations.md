# Changes: assembly/chapters/08-limitations.tex

## Findings applied
F-asm-5 (Table 8.1 row), F-asm-23, F-con-1 (new behavior bullet), F-con-10 (new behavior bullet), F-con-25 (behaviours to behaviors; count intro now "Four behaviors").

## Not applied as written
None. Pass two: caption changed to Title Case.

## Open questions
- Table row "Place code and data with .ORG" depends on the per-object .ORG rule (F-asm-7); consider a pointer to Section 5.1.
- F-asm-9 and F-asm-6 are marked simulator; no Table 8.1 rows were added.

## Round 2

- Table 8.1, ".ORG" row: added a pointer to Section~\ref{sec:org} (orchestrator fix).
- COPI #0x14FF bullet: now points to Chapter~\ref{ch:toolchain}, where the hook is documented.
- No consistency-2 findings concern this chapter. Open questions: none.
