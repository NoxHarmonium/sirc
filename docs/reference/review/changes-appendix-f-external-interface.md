# Changes — chapters/appendix-f-external-interface.tex

## Findings applied
- F-cons-13: Replaced the latching guarantee in the IRQ1-IRQ4 pin description
  ("...is latched at the moment of assertion, even if it is released before the
  next instruction boundary; only an assertion that begins and ends while the
  enable bit is 0 is lost") with the author's wording: "The requesting device must
  hold the line asserted until it is sampled and serviced; an assertion that
  begins and ends between instruction boundaries, before the CPU samples it, is
  not recognized." Applied the same correction, condensed, to the IRQ1--IRQ4 row
  of Table~\ref{tab:external-input-sampling-rules}. Both passages now match the
  plain level-sensitive behavior already described in Chapters 5 and 6.

## Findings not applied as written
- None. F-cons-13 applied cleanly as written for this file.

## Other STYLE.md conformance fixed while in the file
- Numeric-range dashes: converted every pin-range and signal-range hyphen to an
  en dash for consistency with the rest of the chapter's timing tables/diagrams,
  which already used en dashes throughout (e.g. `1-8` -> `1--8`, `A0-A7` ->
  `A0--A7`, `D0-D15` -> `D0--D15`, `BAT0-BAT2` -> `BAT0--BAT2`, `IRQ1-IRQ4` ->
  `IRQ1--IRQ4`, `VCC1-2`/`GND1-2` -> `VCC1--2`/`GND1--2`, `E1-E4` -> `E1--E4`),
  in the pinout table, the five affected subsubsection headings, and two inline
  prose references. Also fixed "Data bus bit 0-15" to plural "bits" to match the
  other rows.
- Level 5 fault-name consistency: the NMI section used lowercase "level 5" and a
  quoted, lowercase "level five hardware exception conflict" fault name, which
  didn't match Chapter 6's canonical capitalization ("Level 5", "Level Five
  Hardware Exception Conflict"). Corrected both instances and dropped the scare
  quotes.
- Split the two-clause NMI sentence ("This interrupt cannot be masked, and
  if...") into two sentences; removed the "it is important that" hedge in the
  following sentence and reworded it with "must"/"should" per STYLE.md's
  normative vocabulary.
- Removed a redundant hedge ("It can be useful for..." -> "This pin is useful
  for...") to match the parallel sentences in the HALT and RSTO sections.
- Removed a redundant clause ("which disables them" after "can be masked") and
  dropped scare quotes around "masked".
- Fixed a grammatically awkward phrase ("the data it was driven" -> "the data
  driven to it") in the bus-abort paragraph.
- Added a missing Oxford comma ("BACK, BERR and BPER" -> "BACK, BERR, and
  BPER").

## Open questions
- None technical. The rest of the chapter (pin tables, timing diagrams, bus
  cycle tables, sampling-rules table) already conformed to STYLE.md on
  inspection, so no further changes were made there.
