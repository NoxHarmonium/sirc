# Changes — chapters/04-data-representation.tex

## Findings applied

- **F-data-6** (lines ~90-92, instruction fetch model): replaced the "increments
  \reg{p} by one word twice" description with the corrected model: \reg{p} is
  unchanged during fetch (high word at \reg{p}, low word at \reg{p}+1) and
  advances by two words during decode. Applied per Gate 2 ruling ("apply the
  fix as proposed").
- **F-data-3** (lines ~97-106, vector table base): reworded "Vector entry V
  starts at word address V * 2" to include `system_ram_offset`, with a note
  that it is the internal vector-table base and is not program-visible (per
  the finding's own note citing registers.rs). Updated the worked example to
  state a vector-table base of 0 explicitly, and converted the resulting
  concrete addresses to `\addr{}`.
- **F-data-13** (lines ~130-133, immediate truncation): changed the normative
  "Assemblers must reject..." claim to a descriptive statement of the
  reference assembler's actual truncation behavior (truncates silently except
  for the short-immediate-with-shift form, which is range-checked).
- **F-data-11** (lines ~119-125, ~157-162, signed displacements) — applied per
  Gate 2 ruling D: added a cross-reference from the immediate-field table to
  the wraparound section, and added a sentence there stating displacements
  are signed two's-complement and that the trap fires only on a low-word
  wrap, regardless of the displacement's sign. Did not describe the
  unsigned-carry behavior, per ruling D ("do not document it").
- **F-data-21** (lines ~164-167, PC wrap deferral): added the sentence that a
  program-counter wrap while `SR.A` is set raises the segment-overflow fault
  at the next instruction fetch, not on the wrapping instruction.

## Findings skipped (not this file, or not applicable)

- **F-data-10**: status `code-wrong`, not `accepted` — left alone (superseded
  by ruling D / F-data-11 for the signed-displacement wording, but the
  finding itself is not applied as its own item).
- **G1-T4-04** (Example numbering template): this chapter contains no
  `Example:` blocks, so there was nothing to renumber. No change made.
- Findings whose `file:` is a different chapter (F-con-13, F-con-45,
  F-exc-23, F-con-12, F-con-21, F-con-71, etc.) reference this chapter only
  as corroborating evidence; left untouched, out of scope for this file.

## Copy edit (STYLE.md conformance)

- Converted concrete address literals to `\addr{}` where the table/example
  is stating an actual memory or vector address, per the Gate-1 hex-literal
  macro pass: the "Address bus value" column in Table
  `tab:address-pair-interpretation`, the "Stored address" column in Table
  `tab:stored-address-format`, and the worked vector-address example. Left
  data-word contents (register/word values, not addresses) as `\texttt{0x...}`.
- Replaced two uses of "offset(s)" with "displacement(s)" in addressing-mode
  context (Section "Arithmetic and Address Wraparound" prose and the
  "Memory displacements" table row) to match STYLE.md §1's displacement/offset
  split; left the genuine structure/stack "offset" wording in the Software
  Layout Conventions section unchanged.
- Reworded "If it should jump to 0x123456..." to "To jump to 0x123456..." to
  avoid an ambiguous, non-normative "should" next to the manual's normative
  vocabulary.
- No "e.g.", hedge words ("basically", "in order to", "it should be noted"),
  bare `rN` registers, or `$`-prefixed hex literals were found in this
  chapter.
- Added `\label{sec:arithmetic-and-address-wraparound}` so the new
  cross-reference from the immediate-field table has a target.

## Open questions

- F-data-3's fix assumes the manual will formally define `system_ram_offset`
  in Chapter 6 (per F-con-13, whose resolution is an either/or left to the
  Chapter 6 editor: define the term, or delete it and use flat addresses).
  If the Chapter 6 editor instead deletes `system_ram_offset` and reverts to
  flat `V * 2` addressing, this chapter's wording will need to be revisited
  to match. Flagging for the user/coordinating editor to confirm consistency
  once Chapter 6 lands.
