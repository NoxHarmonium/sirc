# Changes: hardware/chapters/04-bus-interface.tex

## Findings applied
- F-hw-19: added the idle-bus paragraph (A0--A23, BRW high and BAT 000 driven while BAS is deasserted, including halted, waiting or in reset; no bus-release mechanism).
- F-hw-4 / F-con-5: SYNC table row now follows the author's decision; "(Instruction Fetch, high word)" dropped.
- F-hw-18 / F-con-24: new section "DMA Bus Cycles" (`sec:dma-bus-cycles`) with Figure `fig:dma-read-timing` (3-word DMA read), placed in this chapter.
- F-con-33: "Bus Transaction Completion" is now "Bus Cycle Completion"; every "transaction" is now "bus cycle".
- F-hw-2 / F-con-40 (Ch.4 lines 96--97): IRQ row now says "except during reset", with the reset-window sentence that names NMI.
- Active-low overbar finding (`\activelow` in prose and tables) and F-con-39 (`\texttt` dropped from signal names).
- Table captions and the Control Input column header changed to Title Case.

## Not applied as written
- F-hw-18 asked for a section and figure in "Ch.4 or Ch.5". I put them in Ch.4, so the figure's value bands needed no new abbreviation in the opening paragraph of Ch.5. BAT bands use the literal bit values (101, 110). Address and data bands use `start`, `start+1`, `start+2` and `d0`--`d2`, which the section text explains. If the Ch.5 editor prefers the figure there, it can be moved without changes.
- The figure's row labels use `\activelow{}`, as STYLE.md requires for timing-diagram labels. The existing Ch.5 figures do not yet use it.
- F-hw-10 and F-hw-11 (simulator findings) are not touched. The HALT row still says interrupt lines are sampled while halted.

## Open questions
- The value of A0--A23 while the bus is idle is written as "not yet specified". The author's decision fixed only BRW and BAT.
- The DMA section states that the DMA coprocessor is optional and that BAT 101 and 110 then never appear. This comes from the Coprocessor Instructions chapter; F-hw-18(f) left it as a question.
- F-hw-18 asked whether "burst" is the right name, since each access is a single-word request. The BAT table in Ch.3 keeps "Read Burst" and "Write Burst", and the new section follows it.
