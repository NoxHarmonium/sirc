# Changes: reference/chapters/13-alu-instructions.tex

## Findings applied
- F-alu-2 (SHFT AF bullet and bytefield box relabeled SUS)
- F-alu-3 (SR write ordering sentence added)
- F-alu-4 (AF = 11 rule added to Status Flag Updates and the Manual Flag Update Control bullet; all 15 SUS bullets now read "Reserved; N, Z, C and V are architecturally undefined.")
- F-alu-5 (flag formula operand definitions)
- F-alu-6 (Condition false row)
- F-alu-8 (trace fault exception at level 7)
- F-alu-9 (ADDI r1, #0 note)
- F-alu-10 (SHFT row added; MOVI row shows `@sym` and `$name`)
- F-con-16 (decision: "Short-immediate move" paragraph renamed "Opcode 0x27", described as undocumented with architecturally undefined behavior)
- F-alu-1 (decision: Ch.13 text stands; no change)

## Not applied as written
- F-alu-7 (status simulator): no manual change.
- F-alu-4: the decision also asks for a Ch.7 statement and an Appendix C pointer; those files belong to other editors.
- F-alu-4: in the box bullets the result-write clause is omitted because test instructions write no result; the clause appears in the Status Flag Updates section.

## Copy edit
- Bare hex literals in the AND, ORR and XOR notes wrapped in `\texttt{}`.
- "Use [S] only with ..." changed to "[S] should be used only with ...".

## Open questions
- The new SHFT table row widens the Syntax column; check the rendered table for overflow (the MOVI row is also longer).
- The SHFT Operation block still reads `SR = flags_from_shift_result`, which no longer holds for an explicit `[A]` or `[N]` suffix.

## Round 2
- F-con2-23: `\texttt{r1}` in the ADDI notes and CMNI description changed to `\reg{r1}`.
- Orchestrator fix: SHFT Operation block now says flags are updated per the status-update suffix ([S] shifter result, default for SHFT; [A] ALU result; [N] no update). This resolves the earlier open question.
- Open question: none.

## Layout

SHFT Operation verbatim block: broke the over-long status-update line after "(default for SHFT)," onto a continuation line with the same indent. No words changed; overfull box (10.1pt) gone.
