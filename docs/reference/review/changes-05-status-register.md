# Changes — 05-status-register.tex

## Findings applied
- F-data-2 (widened protected-mode privilege-violation wording, both the Overview
  bullet and the "Manual Updates" `\textbf{Important:}` paragraph, to cover
  flag-only instructions naming `sr` — CMPI/CMPR/TSAI/TSAR/TSXI/TSXR)
- F-data-4 (added fourth restricted-access bullet: software exceptions targeting
  a vector below `0x60`)
- F-data-6 (shortened the duplicated EA-bit-not-restored paragraph to a
  back-reference to Section~\ref{sec:sr-manual-updates})
- F-data-7 (added supervisor-mode-only annotation to the "Saving and Restoring
  Status" example's opening comment)
- F-data-8 (Overview's Low Byte bullet now states only bits 3–0 are condition
  flags and bits 7–4 are reserved)
- F-cons-5 / F-data-3 (rewrote the LOAD status-override wording: public syntax
  never accepts an override; a non-`[N]` AF encoding, unreachable from
  assembly, is the only path to `U U U U`)
- F-cons-19 (replaced the one "e.g." with "for example," line ~270)

## Findings applied with adjustment
None — all applied as specified, adapted only to the current line-wrapped text
(quoted `claim` text located and matched; no LaTeX structure was at risk).

## Not applicable
- F-data-5: checked this chapter for the "no architectural effect" / `p`-advance
  wording from Chapter 10; it does not appear here, so no change was made (the
  finding's `file` is `10-condition-codes.tex` only).

## Open questions
None. `grep -c 'begin{'` / `grep -c 'end{'` both return 22.
