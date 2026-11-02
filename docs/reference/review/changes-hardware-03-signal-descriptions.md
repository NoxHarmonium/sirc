# Changes: hardware/chapters/03-signal-descriptions.tex

## Findings applied
F-hw-1 (F-con-3 merged), F-hw-2, F-hw-4 (SYNC decision), F-hw-6, F-hw-7 (decision), F-hw-9, F-hw-12, F-hw-13, F-hw-14 (decision), F-hw-17 (HALT decision, F-exc-14 wording), F-hw-22 and F-con-7, F-hw-23, F-con-6 (superseded by F-hw-13 wording), F-con-32 (`\activelow{}` outside headings and Signal Type bullets), F-con-34, F-con-39, F-con-40. Also copy edits (second person, hedges, "e.g." style rewording, `\mnemonic{RSET}`/`\mnemonic{WAIT}`).

## Not applied as written
- F-hw-6: the six-cycle hold is stated in the RSTI and RSTO sections; the Ch.5 repeat belongs to that chapter's editor.
- F-hw-17/F-exc-14: the decision's "bus idle" is stated without the F-hw-19 A/BRW/BAT detail, which belongs to Ch.4.
- F-hw-2: "Instruction Timing appendix" is my name for Appendix B; check the exact title.
- Not touched (simulator/defer): F-hw-3, 5, 8, 10, 11, 15, 16, 25. F-hw-18/F-con-24 (DMA bus-cycle section) belongs to Ch.4/5; the BAT prefetch example is unchanged.

## Open questions
- Does "reset is requested until the reset-vector fetch completes" (F-hw-9) need a matching statement of what HALT does in that window?
- Should the BAS section state the F-hw-19 idle-bus values (A0--A23 driven, BRW high, BAT 000)? Left to Ch.4.
