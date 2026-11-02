# Open questions from the chapter editors (phase 3)

## assembly-03-operands

- F-con-2 evidence says `LJSR (src)+` is legal, while F-asm-4 says LJMP/LJSR take `addr` rather than `(addr)`. The F-asm-4 wording was used.

## assembly-06-toolchain

- The 64 K scratch example now uses length 10000 (hexadecimal) per F-asm-26.
- The table states PROGRAM as read-only, from `writeable: false` in main.rs. Please confirm the intent.

## assembly-07-diagnostics

- The toolchain todo (report errors without panicking) will change the surrounding output; the table text should be rechecked after it.
- The `Invalid addressing mode for MOVR:.` typo and the `Parsing Failure: Base {...}` wrapping are toolchain defects; if fixed, update the table.

## assembly-08-limitations

- Table row "Place code and data with .ORG" depends on the per-object .ORG rule (F-asm-7); consider a pointer to Section 5.1.
- F-asm-9 and F-asm-6 are marked simulator; no Table 8.1 rows were added.

## assembly-preface

- Define `\asmmanual` in assembly/main.tex so the Revision Status can use it.

## hardware-02-package-and-pinout

- Should the pinout figure's pin labels also take overbars? STYLE.md lists prose, tables and timing diagrams only.

## hardware-03-signal-descriptions

- Does "reset is requested until the reset-vector fetch completes" (F-hw-9) need a matching statement of what HALT does in that window?
- Should the BAS section state the F-hw-19 idle-bus values (A0--A23 driven, BRW high, BAT 000)? Left to Ch.4.

## hardware-04-bus-interface

- The value of A0--A23 while the bus is idle is written as "not yet specified". The author's decision fixed only BRW and BAT.
- The DMA section states that the DMA coprocessor is optional and that BAT 101 and 110 then never appear. This comes from the Coprocessor Instructions chapter; F-hw-18(f) left it as a question.
- F-hw-18 asked whether "burst" is the right name, since each access is a single-word request. The BAT table in Ch.3 keeps "Read Burst" and "Write Burst", and the new section follows it.

## hardware-05-bus-timing

- Value-band abbreviations used in figures (Mem, next, req, wait, done, Enter, Prev WB, active, addr, data) are not all defined in the opening paragraph, as STYLE.md section 6 requires. Defining them is a decision on wording, so left alone.

## reference-01-introduction

- Line 49 says the CPU "should not be used in memory-constrained environments"; a design-intent statement, left as advice.
- "32 megabytes of external byte storage" depends on a 2-byte word; left unchanged.

## reference-02-cpu-architecture

- Should the active-low overbar rule extend to ARCH (`\texttt{RSTI}` remains as is)?

## reference-03-registers

- Should the figure box text be shortened or split over two lines if it overflows?

Copy edit: "privilege exception" changed to "privilege violation fault" (two places) to match the rest of the chapter; "User programs" changed to "Programs".

## reference-04-data-representation

- Line 49--51 says a protected-mode write that changes "any of the 16 bits" of a high register faults, which includes bits 7--0 that are address bits. Confirm whether this should read bits 15--8.

## reference-05-status-register

- The accepted F-data-10 text writes "bits 7-0" with a hyphen inside a listing comment (comment, so allowed by STYLE).
- The line 5 NMI "Level 5" wording in the E1--E4 list uses interrupt-line levels, while the new sentence says line 5 runs at exception level 6. Author may want the list relabeled.

## reference-06-exceptions

- Should `docs/reference/main.tex` define `\hwmanual`? (blocking build)
- HALT latency "at most 6 cycles" and "bus idle" come from the author's decision text; the Reset State halt row now says
  the CPU halts "at the first phase 0 after reset, before the reset-vector fetch", which is in tension with the
  Reset Output Hold section's statement that RSTO is held for a fixed countdown first. Confirm the intended order.

## reference-07-instruction-formats

- Lines about LDEA/LDEL bits 25--24 say "always write them as zero", which STYLE.md says reserved fields should not require. Left unchanged because removing the requirement is a technical decision.
- The immediate-format notes list AF for ALU ops as 00/01/10 only; the 11 case is covered in the field descriptions, not repeated in the notes.

## reference-08-addressing-modes

- STYLE.md reserves "displacement" for addressing-mode values, but this chapter uses "offset" throughout (tables, generation formulas). Not changed; a manual-wide decision is needed.
- F-mem-4 adds the auto-update wrap check to the Ch. 14 and 15 fault text; the Indirect Immediate box here says the fault fires "only if the low-word addition wraps". It may need the same clause for post-increment and pre-decrement.

## reference-09-shift-operations

- Lines 55-56 say the shifted operand is "the second operand" of the three-operand form, while the table calls it the first source operand. Presumably it counts the destination as the first written operand; the wording needs the author's choice.
- Bare listings (Mode 0, Mode 1, Testing Shift Results, Preserving Flags, SHFT) have no `% qa:assemble` marker. I did not add one because the Testing Shift Results and Preserving Flags listings reference undefined labels and would fail.
- Examples in this chapter are not numbered `Example N-M`; that rule is stated for instruction entries only.

## reference-10-condition-codes

- The Loop Construction example uses BRAN|HI after SUBI, where HI tests C = 0 AND Z = 0 (a borrow-based test). Whether this is the intended i > 0 test is a technical point, left unchanged.

## reference-11-reading-instructions

- "SIRCIS" in the opening sentence: confirm the intended name (SIRC-1?).

## reference-12-instruction-summary

- The Flags column still uses "NZCV" and "NZ (C,V=0)" rather than the Chapter 11 symbols; converting the column to symbol columns is a structural change left to the author.
- The 18-cycle RSET figure and "WAIT takes longer than 12" are taken from Appendix B; confirm they match after its edit.

Also copy edited: "commonly-used" to "commonly used"; example list now has a full introducing sentence.

## reference-13-alu-instructions

- The new SHFT table row widens the Syntax column; check the rendered table for overflow (the MOVI row is also longer).
- The SHFT Operation block still reads `SR = flags_from_shift_result`, which no longer holds for an explicit `[A]` or `[N]` suffix.

## reference-14-memory-instructions

- Operation pseudocode and the Privilege field still say little about LOAD into `sr`/`pl`; confirm whether to cross-reference these from the Description.
- LOAD "Condition field" and Ch14 Common-semantics prose do not mention the segment-overflow check for a skipped instruction beyond the table row.

## reference-16-coprocessor-instructions

- DMA recovery bullets in "Retrying After a Fault" use imperatives ("correct", "do not return", "Use"); the
  STYLE.md rule on imperatives is ambiguous for ARCH. Left unchanged.
- Should the DMA Operands sentences "the assembler rejects a literal outside that range" move to the assembly manual
  (STYLE.md Scope)?
- `RSTO` stays `\texttt{}` pending the ruling on overbars for ARCH (F-con-32).

## reference-appendix-a-opcode-map

- The caption "Complete SIRCIS Opcode Map" and the intro use "SIRCIS", while the rest of the manual says SIRC-1. Left alone; confirm the intended name.

## reference-appendix-b-timing

- `\hwmanual{}` is not defined in `docs/reference/main.tex` (F-con-38 asks for it). The build fails until that macro is added.
- The table cells for COPI/COPR, EXCP, RSET and WAIT are now long and wrap in the 6.5 cm column. Consider widening the second column.

## reference-appendix-d-examples

- F-sum-11 suggested optionally noting in Program Conventions that the final `COPI #0x14FF` is a simulator exit hook. The author's decision on F-con-1 places that note in the assembly manual, so none was added here. Confirm that is intended.

## reference-notation-and-glossary

- Should `\activelow{}` extend to the ARCH signal names (Signals, Requests and Pins, and other ARCH chapters)? The chapter currently says they are in typewriter font.
- The `.EQU` entry says a constant is "usable anywhere a immediate operand is accepted". The toolchain rejects constants in shift counts, EXCP and ETFR/ETTR (F-asm-9, simulator status). Left unchanged.
