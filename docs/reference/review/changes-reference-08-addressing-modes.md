# Changes: reference/chapters/08-addressing-modes.tex

## Findings applied
- F-enc-5 (OPR register shorthand takes an optional shift)
- F-enc-9 (LJMP/LJSR encoding: p in bits 23--22, source pair in AF bits 5--4)
- F-enc-15 (Short Immediate usage text)
- F-con-35 (run time -> runtime)

## Not applied as written
- F-enc-4 (simulator), F-enc-8 (merged into F-alu-1; the decision keeps this text): left alone.
- F-con-2, F-enc-1 name this chapter only as evidence; the chapter text already agrees (lines about LJMP/LJSR `(src)` and "link to own address + 2").
- Copy edit: "Post-increment/Pre-decrement" bullet lowercased to "Post-increment/pre-decrement".

## Open questions
- STYLE.md reserves "displacement" for addressing-mode values, but this chapter uses "offset" throughout (tables, generation formulas). Not changed; a manual-wide decision is needed.
- F-mem-4 adds the auto-update wrap check to the Ch. 14 and 15 fault text; the Indirect Immediate box here says the fault fires "only if the low-word addition wraps". It may need the same clause for post-increment and pre-decrement.
