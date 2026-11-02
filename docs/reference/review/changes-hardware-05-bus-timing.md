# Changes: hardware/chapters/05-bus-timing.tex

## Findings applied
F-con-8, F-hw-20 (IRQ row held `L L L L L`; sentence added saying why), F-hw-21 (dispatch at the next phase 0 with no coprocessor call or command pending or active; edge sentence added), F-hw-24 (address bands `2n` / `2n+1`; EV legend extended), F-con-29 (labeled), F-hw-6 (reset-output hold length and n + 5 rule repeated in the reset paragraph; also consistent with F-tim-8). Also applied: `\activelow{}` on row labels, prose and two captions; Title Case captions.

## Not applied as written
- F-hw-24: the reset-vector fetch is vector 0, so its address bands read `0` and `1` rather than `2n` / `2n+1`.
- F-hw-18 / F-con-24 (DMA section): left to the 04-bus-interface editor, as instructed. F-hw-19: figures already drive BRW high and BAT 000 while BAS is deasserted; no change needed.

## Open questions
- Value-band abbreviations used in figures (Mem, next, req, wait, done, Enter, Prev WB, active, addr, data) are not all defined in the opening paragraph, as STYLE.md section 6 requires. Defining them is a decision on wording, so left alone.
