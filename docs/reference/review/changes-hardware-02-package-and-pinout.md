# Changes: hardware/chapters/02-package-and-pinout.tex

## Findings applied
- F-hw-26 (package sentence; no other packages described, so no extra sentence added)
- F-hw-27 (BPER description now "Bus protection error input")
- F-con-32 (`\activelow{}` on active-low names in the pin table; IRQ1--IRQ4 wrapped per name)
- STYLE.md captions: both captions now Title Case

## Not applied as written
- None. The TikZ pinout figure keeps plain signal names: they are node labels in a package diagram, not prose, tables or timing-diagram labels.

## Open questions
- Should the pinout figure's pin labels also take overbars? STYLE.md lists prose, tables and timing diagrams only.

## Round 2

- Applied decision 1 (STYLE.md active-low rule): the TikZ pinout figure labels for the 13 active-low pins (BAS, BACK, BERR, BPER, HALT, TRCE, IRQ1--IRQ4, NMI, RSTI, RSTO) now use `\activelow{}`, matching the pin table. No other round-2 findings or decisions concern this chapter.
- Built with `make all` and checked the rendered figure (page 13 at 80 dpi): overbars render and labels fit.
- Open questions: none.
