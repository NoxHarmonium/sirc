# Changes: reference / 06-exceptions.tex

## Findings applied
The triaged file has no findings IDs in the extract for the quoted-claim items below, so they are listed by topic.
Findings whose file is this chapter: segment-overflow bus access type (LDEA/LDEL), HALT sampling at every phase 0,
traced-EXCP trap held in the CSR (with the Reset State row), canceled/canceling, `\hwmanual{}` (2 uses), reset-vector
exception to the vector-fetch-failure rule, privilege check before the "ignored inside a handler" rule (both places),
debugger single-step wording, L5 overrun on failed vector fetch, L5 always enabled in the acceptance rule, fourth
metadata exception (auto-update), executed-ALU/LOAD qualifier (both places), dispatch-fault link-register wording,
Double Fault quick-reference cell and "except as described" wording, second-L5-in-handler wording, 32-bit vector
pointer, CSR priority example, reserved-vector row note, ETFR link-register wording, one-digit fault codes.
Findings in other files that name this chapter: F-hw-14 (NMI held across reset, Reset State row), F-hw-17 (HALT after
reset, same wording as the HALT decision), F-hw-7 (triple-fault reset behaves as a software reset), F-con-15
(GP/address/link registers architecturally undefined after reset; three Reset State rows), F-exc-3 (privilege rules bind
undefined encodings, Invalid Opcode note), F-con-33 (bus transaction to bus cycle, 2 places), F-tim-11 (Privilege
Violation states phases 2 and 3).

## Not applied as written
- Reserved-vector row in the Quick Reference table: the Vector column is 1.2 cm wide, so the longer cell would overflow.
  Used the "table note" alternative: a sentence after the table.
- `\hwmanual{}`: used as the finding directs, but `docs/reference/main.tex` does not yet define it (only the assembly
  manual does). It must be added there or the chapter will not build.
- F-con-32 (active-low names in ARCH): no author decision and STYLE.md scopes `\activelow{}` to the hardware manual;
  left unchanged.
- "Fault type" codes: changed only the fault-type list and the double-fault codes in the metadata section; the vector
  table keeps two-digit vector numbers.

## Open questions
- Should `docs/reference/main.tex` define `\hwmanual`? (blocking build)
- HALT latency "at most 6 cycles" and "bus idle" come from the author's decision text; the Reset State halt row now says
  the CPU halts "at the first phase 0 after reset, before the reset-vector fetch", which is in tension with the
  Reset Output Hold section's statement that RSTO is held for a fixed countdown first. Confirm the intended order.
