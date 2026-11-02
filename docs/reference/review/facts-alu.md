# Findings: G5 (ALU), reviewer tag `alu`

Chapter reviewed: `docs/reference/chapters/13-alu-instructions.tex`.

Checks that came out clean, so no findings were filed for them: all bytefield widths sum to 32 (the preamble sets
`endianness=big`); every opcode in Table `tab:alu-legal-forms` matches the opcode map; the C and V formulas in
Table `tab:alu-flag-formulas` are correct for ADD/ADC and SUB/SBC, including a carry or borrow in (checked against
`alu.rs:81-153,245-267`, where V uses `(a, !b, r)`, which is the same thing); every arithmetic comment in the listings
(XOR swap, `0x12<<8|0x34`, `CMNI r1,#5` with r1 = -5, the CPC 32-bit compare conditions) works out; every listing form
is accepted by `arithmetic_immediate.rs`, `arithmetic_register.rs` and `mov.rs`; the shorthand-with-shift semantics
(line 121) match `arithmetic_register.rs:142-164` and `assembly/chapters/03-operands.tex:69-80`; the timing (6 cycles)
matches `appendix-b-timing.tex:34-35,181`.

### F-alu-1
- file: reference/chapters/13-alu-instructions.tex
- lines: 93-111, 1385-1386, 1404-1405, 1520-1521, 1652-1653, 1671-1672, 1792-1793, 1924-1925, 2060-2061, 2188-2189
- severity: major
- category: contradiction
- claim: "ignored (R1, bits 25--22), register form only: no destination register is written, so the CPU ignores this field. The assembler encodes a copy of \texttt{rS1}."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:120-140,166-189 accept a three-register form for every test mnemonic, and tc/tests/assembler/arithmetic_register_test.rs:34-36 assembles `CMPR ll, r7, r1`, `TSAR ah, r6, r2` and `TSXR al, r5, r3`. That form puts the first register in R1, which the CPU ignores, so `CMPR r1, r2, r3` silently compares r2 with r3. The chapter lists only `CMPR rS1, rS2[, shift]`, and assembly/chapters/03-operands.tex:69-80 does not cover the test mnemonics. Also reported as a toolchain gap in review/facts-enc.md:96.
- resolution: design
- fix: Option (a), recommended: the assembler rejects a three-register operand list (with or without a shift) for CMNR/CNCR/CMPR/CPCR/TSAR/TSOR/TSXR, and the assembly manual's Register Shorthand section says that test mnemonics take exactly two registers. The chapter text then stands as written. Option (b): keep the toolchain as it is, add `CMPR rX, rS1, rS2[, shift]` rows to the table, and change "The assembler encodes a copy of \texttt{rS1}" to "In the two-register form the assembler encodes a copy of \texttt{rS1}; in the three-register form it encodes the first register written, which has no effect."
- confidence: high

### F-alu-2
- file: reference/chapters/13-alu-instructions.tex
- lines: 266, 276-277, 291, 317
- severity: minor
- category: contradiction
- claim: "\textbf{AF (bits 5--4):} fixed at \texttt{10} (Shift) by the meta-instruction." vs "\textbf{Flags:} Updated from the shifter result by default."
- evidence: tc/src/parsers/opcodes/arithmetic_immediate.rs:165-166 (`status_register_update_source.unwrap_or(Shift)`) means `SHFT[A]` and `SHFT[N]` are accepted and change AF. reference/chapters/11-reading-instructions.tex:143 says "SHFT defaults to [S] instead". The bytefield (line 277) hard-codes `10`.
- resolution: manual-wrong
- fix: Replace the AF bullet with "\textbf{AF (bits 5--4):} \texttt{10} (Shift) by default; an explicit \texttt{[A]} or \texttt{[N]} suffix encodes \texttt{01} or \texttt{00}." Relabel the bytefield AF box `SUS`. If the author instead wants SHFT to have a fixed AF, the assembler must reject a suffix on SHFT, and ch. 11:143 needs the same change.
- confidence: high

### F-alu-3
- file: reference/chapters/13-alu-instructions.tex
- lines: 152-153
- severity: major
- category: fact
- claim: "An ALU instruction whose register field names \reg{sr} is permitted in protected mode, but only bits 7--0 of \reg{sr} are written; bits 15--8 are left unchanged"
- evidence: reference/chapters/05-status-register.tex (sec:sr-manual-updates) says that an ALU instruction targeting sr writes its result and then overwrites bits 7-0 with its own flags unless `[N]` is used. Code agrees (write_back.rs:186-189, `set_register_value` then `update_status_flags`). In protected mode, then, `ORRI sr, #3` with the default `[A]` changes nothing except the flags. The ALU chapter never states this ordering, and the sentence quoted above suggests the low byte can be written. Related: general-todo.md item 2; facts.md Open question 5.
- resolution: manual-wrong
- fix: After the quoted sentence, add: "With the \texttt{[A]} or \texttt{[S]} status source, the instruction's own flag update then overwrites bits 7--0, so the written value survives only in bits 15--8 (supervisor mode) or not at all (protected mode). Use \texttt{[N]} to write the low byte (Section~\ref{sec:sr-manual-updates})."
- confidence: high

### F-alu-4
- file: reference/chapters/13-alu-instructions.tex
- lines: 247, 406-407 (and the identical SUS bullet in every instruction box)
- severity: major
- category: fact
- claim: "AF encoding \texttt{11} (binary) is reserved and has no assembly syntax."
- evidence: The chapter defines no effect for AF = 11 on an ALU opcode. reference/chapters/07-instruction-formats.tex:98,158,228 and 11-reading-instructions.tex:136 say only "Reserved for future use". appendix-c-undocumented.tex:113 lists it as a possible future flag source. The code leaves SR unchanged (write_back.rs:154, `_ => registers.sr`). By contrast, the chapter does give a rule (`U U U U`) for MOVI/MOVR with AF = 10 (lines 203-204, 1258-1260).
- resolution: design
- fix: Option (a), recommended: state in the Status Flag Updates section that "An ALU encoding with AF \texttt{11} leaves N, Z, C and V architecturally undefined (\texttt{U U U U}); the result write is unaffected." This keeps AF = 11 free for a future source and matches the MOVI/MOVR AF = 10 rule. Option (b): define AF = 11 as "no update", the same as `[N]`, which matches the simulator but uses up the encoding. In either case, also state that the register result is still written.
- confidence: medium

### F-alu-5
- file: reference/chapters/13-alu-instructions.tex
- lines: 208-211, 219-220
- severity: major
- category: fact
- claim: "in terms of the sign bit (bit 15) of the destination operand (\texttt{Dm}), the source operand (\texttt{Sm}), and the result (\texttt{Rm})"
- evidence: In the three-operand register form (`SUBR rD, rS1, rS2`) and in compare instructions, the destination register is not an input at all. The operands are the left operand, which is shift(rS1) or shift(Reg) per lines 134-136, and the right operand (rS2 or the immediate). The code computes V from (a, b, result), where a is the post-shift left operand (fetch_and_decode.rs:180-194; alu.rs:81-90,195-206). Read literally, "destination operand" points an implementer at the old value of rD and, for shifted forms, leaves open whether the pre-shift or post-shift value is meant.
- resolution: manual-wrong
- fix: Replace the sentence with: "in terms of the sign bit (bit 15) of the left operand after any shift (\texttt{Dm}), the right operand (\texttt{Sm}: \texttt{rS2}, imm16, or the zero-extended imm8), and the 16-bit result (\texttt{Rm}). For \mnemonic{ADC}/\mnemonic{SBC}, \texttt{Rm} includes the incoming carry." Alternatively, rename the symbols to \texttt{Am}/\texttt{Bm} so they do not suggest the destination register.
- confidence: high

### F-alu-6
- file: reference/chapters/13-alu-instructions.tex
- lines: 133
- severity: minor
- category: contradiction
- claim: "Condition false        & The instruction is skipped and has no side effects. Registers and status flags are preserved."
- evidence: reference/chapters/06-exceptions.tex (sec:segment-overflow-fault) says that an instruction in the last slot "and a branch whose condition is false" still advances the PC and raises segment overflow with SR.A set. A false-condition instruction is also still traced (06-exceptions.tex:121,321). Line 149-150 of this chapter itself says the PC-wrap fault applies to every ALU instruction that does not write pl, and a false-condition instruction does not write pl.
- resolution: manual-wrong
- fix: "The instruction writes no register and no status flag. The program counter still advances, so the segment-overflow check on the program counter and trace still apply."
- confidence: high

### F-alu-7
- file: reference/chapters/13-alu-instructions.tex
- lines: 149-150
- severity: major
- category: fact
- claim: "An ALU instruction in the last instruction slot of a segment still raises a segment-overflow fault when \texttt{SR.A} is set, because it advances the program counter, unless it writes \reg{pl}"
- evidence: The manual is consistent with 06-exceptions.tex (sec:segment-overflow-fault). The simulator raises the fault at the next fetch for every instruction whenever `pl + 2` carries, including one that writes pl, and records the new PC rather than the instruction's own address (pu-execution.rs:99-108,134-148; fetch_and_decode.rs:204). This is general-todo.md item 15, and is also noted in review/facts-arch.md:33.
- resolution: code-wrong
- fix: No manual change. Simulator todo 15 covers this; add a test case for an ALU instruction at `pl = 0xFFFE` that writes `pl` (no fault) and one that writes `r1` (fault, LR6 = 0xFFFE).
- confidence: high

### F-alu-8
- file: reference/chapters/13-alu-instructions.tex
- lines: 154-155
- severity: minor
- category: contradiction
- claim: "If trace mode was enabled when the instruction began, an instruction-trace fault is raised after the instruction commits."
- evidence: reference/chapters/06-exceptions.tex:121,307,321,763 say that no trace fault is raised at exception level 7. The simulator does not implement that exception yet (general-todo.md item 19).
- resolution: manual-wrong
- fix: "If trace mode was enabled when the instruction began, an instruction-trace fault is raised after the instruction commits, except at exception level 7 (Section~\ref{sec:instruction-trace-fault})."
- confidence: high

### F-alu-9
- file: reference/chapters/13-alu-instructions.tex
- lines: 474
- severity: minor
- category: fact
- claim: "Adding zero (\texttt{ADDI r1, \#0}) can be used to move values or test flags"
- evidence: The immediate form uses rD as both the source and the destination (lines 134, 420), so `ADDI rD, #0` cannot move a value to another register. A register-to-register copy with flags needs `ADDR rD, rS, rZ` with a zero register, and SIRC has none (facts.md, Registers; ho:21-22).
- resolution: manual-wrong
- fix: "Adding zero (\texttt{ADDI r1, \#0}) sets N and Z from \texttt{r1} and clears C and V without changing it; use \mnemonic{MOVR} to copy a register."
- confidence: high

### F-alu-10
- file: reference/chapters/13-alu-instructions.tex
- lines: 54-116, 266
- severity: minor
- category: improvement
- claim: "\textbf{Assembles to:} \texttt{ORRI[S] rD, \#0, shift}. See Table~\ref{tab:alu-legal-forms} for the full family of legal forms."
- evidence: Table tab:alu-legal-forms has no SHFT row. It also lists `MOVI rD, \#imm16` only, but mov.rs:98-140 accepts `@sym` (default `.l`) and `$placeholder`. The ALU immediates reject symbols (facts.md, Default symbol-reference types), and the table does not show that difference.
- resolution: n/a
- fix: Add a row `SHFT & \texttt{SHFT rD, shift} & \opcode{25} & Writes result; default \texttt{[S]}`, and change the MOVI row to `\texttt{MOVI rD, \#imm16 | @sym | \$name}`. Alternatively, point the SHFT box at Chapter 12 rather than at this table.
- confidence: medium
