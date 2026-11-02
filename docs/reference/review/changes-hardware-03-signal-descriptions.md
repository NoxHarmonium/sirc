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

## Round 2

Applied: F-con2-15 (BERR and BPER: only fault-, double-fault- and reset-vector fetch failures differ; interrupt and trap vector fetches raise the bus or protection fault), F-con2-16 (RSTO hold wording, "six for a one-cycle assertion"), F-con2-17 (NMI heading singular), decision 2 (HALT after reset: RSTO hold runs first, then halt at first phase 0). Decision 1 overbars already present; no typewriter-font claims found.

Open questions: none.

## Improvements

- F-CRH-19: address-bus description now says A0-A23 carry a 24-bit word address, one 16-bit word per address, 16M words, no byte strobes.
- F-CRH-1: CLKI text links to Section sec:execution-phases.
- F-CRH-15: no statement about unconnected or unused pins exists in the manual (only that BACK may be held asserted). Left unaddressed; the topic belongs in Chapter 6, which is not in my scope. TODO(author): state pull-up/unconnected behavior of IRQ, NMI, HALT, TRCE, BAS-related inputs, and power-up pin states.
