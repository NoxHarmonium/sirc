# Findings: data, status, condition codes (reviewer tag: data)

Chapters: reference/chapters/04-data-representation.tex, 05-status-register.tex, 10-condition-codes.tex.

### F-data-1
- file: reference/chapters/04-data-representation.tex
- lines: 158-161
- severity: major
- category: fact
- claim: "If \texttt{SR.A} is set and advancing the program counter past an instruction would wrap the low word, that instruction raises a segment-overflow fault before it commits, unless it is a taken branch or jump that does not use the advanced address, or an ALU or \mnemonic{LOAD} instruction that writes \reg{pl}"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:99-108,134-148 (the wrap is recorded at decode in `next_instruction_fetch_is_overflow` and only raised at the *next* instruction's phase 0, after the last-slot instruction has fully committed; nothing clears the flag when write-back writes `pl`, so a taken branch/jump or an ALU/LOAD write to `pl` from the last slot still faults at the target); sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:855-935 (test expects the link register to hold the wrapped address 0x0000, not the instruction's own 0xFFFE). Agrees with manual: reference/chapters/06-exceptions.tex:249-257 and 10-condition-codes.tex:12-13.
- resolution: code-wrong
- fix: Keep the manual. Simulator todo: detect the PC-advance wrap in phase 3 of the same instruction, cancel it and save its own address; suppress the fault for taken branches/jumps that do not save a return address and for ALU/LOAD writes to `pl`; still fault for a skipped (condition-false) instruction.
- confidence: high

### F-data-2
- file: reference/chapters/04-data-representation.tex
- lines: 152-154
- severity: major
- category: fact
- claim: "For pre-decrement and post-increment forms, the address-register auto-update and the effective-address calculation are checked separately: either one wrapping raises the fault, even if the other brings the result back into range."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:93 (auto-update value `ad_l_ + addr_inc` computed with its overflow discarded), :98-103 and :130-133 (checks only the displacement and `displaced ± 1`). So an auto-update wrap is never trapped (e.g. `-(#5,a)` with al=0x0000: EA 0x0004 fine, al wraps to 0xFFFF, no fault), and post-increment checks `displaced+1`, which is not an architectural value (al=0x0010, disp 0x7FEF -> EA 0xFFFF, al->0x0011, yet the code faults). Matches facts.md Open question 11.
- resolution: code-wrong
- fix: Keep the manual. Simulator todo: check the overflow of `ad_l_ + addr_inc` (the written-back register) and of the EA (`displaced` for post-increment, `displaced - 1` for pre-decrement) separately, and drop the `displaced + 1` check.
- confidence: high

### F-data-3
- file: reference/chapters/04-data-representation.tex
- lines: 46-48, 87-88
- severity: major
- category: fact
- claim: "They are ignored by address calculation and are carried unchanged when an address pair is copied, loaded or stored. They will be preserved unless a direct write to the high register of an address register pair overwrites them." / "Each exception vector table entry is a stored 32-bit pointer with contains a 24-bit address and eight bits of metadata"
- evidence: sirc-vm/peripheral-cpu/src/registers.rs:289-295,348-356 (`to_full_address` and `to_segmented_address` both mask with ADDRESS_MASK); eu-execution.rs:273-275 (saved return address is `get_full_pc_address()`, masked, so `ph` bits 15-8 are lost), :289 and :382 (vector loaded into `p` through the mask, so vector metadata never reaches `ph`), :363-372 (RETE restores the masked value), :391,:417 (ETFR/ETTR move `a` through the mask). The manual agrees with itself: reference/chapters/06-exceptions.tex:603-604 ("with bits 31--24 carried unchanged from \reg{p}").
- resolution: code-wrong
- fix: Keep the manual. Simulator todo: keep the full 32-bit value for vector loads, link-register return addresses, RETE, and ETFR/ETTR; apply the 24-bit mask only when driving the address bus.
- confidence: medium

### F-data-4
- file: reference/chapters/05-status-register.tex
- lines: 159-169
- severity: major
- category: fact
- claim: "\mnemonic{RETE}, which sets \texttt{EA} from the restored exception level rather than from the saved status register, so editing bit 13 of a saved status register with \mnemonic{ETTR} has no effect."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:372-378 (RETE copies all 16 saved bits into `sr` and only clears bit 13 when the saved level is 0; when the saved level is non-zero, bit 13 comes from the saved SR); :414-421 (ETTR writes all 16 bits of r7 into the saved SR). A nested handler that clears bit 13 via ETTR therefore returns with EA=0 at a non-zero level.
- resolution: code-wrong
- fix: Keep the manual. Simulator todo: after restoring SR in RETE, set or clear bit 13 according to `saved_exception_level != 0`.
- confidence: high

### F-data-5
- file: reference/chapters/05-status-register.tex
- lines: 186-187
- severity: major
- category: fact
- claim: "An instruction trace fault (vector \texttt{0x06}) is generated after every instruction, except at exception level 7"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:389-393,450-460 (trace is sampled from SR.T or TRCE and raised with no exception-level check); lib.rs:183-190 (a fault raised at level >= 7 becomes a DoubleFault). So a level-7 handler that sets T, or runs while TRCE is asserted, double-faults. The manual is consistent: reference/chapters/06-exceptions.tex:121,307,321,763.
- resolution: code-wrong
- fix: Keep the manual. Simulator todo: gate `trace_mode_sampled` on `current_exception_level < 7`.
- confidence: high

### F-data-6
- file: reference/chapters/05-status-register.tex
- lines: 142-145
- severity: major
- category: fact
- claim: "The line is not latched while its own handler or a higher-priority one is running, so a request withdrawn before then is not seen"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:576-596 (`raise_hardware_interrupt` latches every enabled asserted line 1-4 on every poll and never compares against `current_exception_level`). The manual is consistent: reference/chapters/06-exceptions.tex:459-465. In the code, a line still asserted when its handler is entered is re-latched and dispatched again after RETE, which is the failure the Ch. 6 rule exists to prevent.
- resolution: code-wrong
- fix: Keep the manual. Simulator todo: mask out lines 1-4 whose dispatch level (line + 1) is <= `current_exception_level` before latching.
- confidence: high

### F-data-7
- file: reference/chapters/05-status-register.tex
- lines: 119-123, 139-144
- severity: major
- category: fact
- claim: "\textbf{E1 (Bit 9):} Enable Hardware Interrupt Line 1 (Level 1, lowest maskable priority)" ... "A lower-priority request held during a higher-priority handler is serviced as soon as the exception level drops below its level."
- evidence: reference/chapters/06-exceptions.tex:29-33,640-643 (line n is dispatched at exception level n+1; level 1 is software exceptions); eu/definitions.rs and eu-execution.rs:77-82 (level = 7 - (vector>>4), so line 1 -> level 2). The chapter uses "Level n" for the line number and "exception level" in the next bullets without giving the n+1 mapping. Read literally, "drops below its level" for line 2 means exception level <= 1, which is wrong: a line 2 request must preempt a line 1 handler running at exception level 2.
- resolution: manual-wrong
- fix: After the E1-E4 list, add: "Hardware interrupt line n is dispatched at exception level n+1 (Chapter~\ref{ch:exceptions}); line 5 runs at level 6." Replace "as soon as the exception level drops below its level" with "as soon as the exception level drops to n or lower, where n is the line number".
- confidence: medium

### F-data-8
- file: reference/chapters/05-status-register.tex
- lines: 248-252
- severity: minor
- category: contradiction
- claim: "This applies to every instruction that names \reg{sr} as its destination, including the flag-only comparison and test instructions (...), which do not write their destination and are therefore unaffected."
- evidence: none (internal: the sentence says the rule applies to these instructions, then says they are unaffected)
- resolution: manual-wrong
- fix: "This applies to every instruction that writes \reg{sr} as its destination. The flag-only comparison and test instructions (\mnemonic{CMNI} ... \mnemonic{TSXR}) do not write their destination register, so naming \reg{sr} there only affects the flags in bits 3--0."
- confidence: high

### F-data-9
- file: reference/chapters/05-status-register.tex
- lines: 281
- severity: minor
- category: contradiction
- claim: "When returning from an exception with \mnemonic{RETE}, the saved status register is restored."
- evidence: reference/chapters/05-status-register.tex:159,166-169 (EA is set from the restored level, not from the saved SR)
- resolution: manual-wrong
- fix: "When returning from an exception with \mnemonic{RETE}, the saved status register is restored, except bit 13 (\texttt{EA}), which is set from the restored exception level."
- confidence: high

### F-data-10
- file: reference/chapters/05-status-register.tex
- lines: 308-316
- severity: minor
- category: fact
- claim: "; Save status register (supervisor mode only -- see the manual updates section)"
- evidence: reference/chapters/05-status-register.tex:244-247,256-257 (in protected mode the read returns 0x00XX and the write updates only bits 7-0); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:40-48, write_back.rs:45-49. The sequence `MOVR r7, sr` / `MOVR[N] sr, r7` correctly saves and restores the flags in protected mode too; only the high byte is out of reach.
- resolution: manual-wrong
- fix: "; Save status register (in protected mode only bits 7-0 are saved and restored)"
- confidence: high

### F-data-11
- file: reference/chapters/05-status-register.tex
- lines: 81-82
- severity: minor
- category: fact
- claim: "Overflow occurs when adding two positive numbers yields a negative result, or adding two negative numbers yields a positive result"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:195-267 (SUB/CMP/SBC/CPC compute V on (a, !b, result)); facts.md "Shift types" (ASL sets V on a sign change). The note gives only the addition rule, but V is also set by subtraction and ASL.
- resolution: manual-wrong
- fix: Append: "For subtraction (a - b), overflow occurs when a and b have different signs and the result's sign differs from a. Shift operations define V separately (Chapter~\ref{ch:shift-operations})."
- confidence: high

### F-data-12
- file: reference/chapters/05-status-register.tex
- lines: 200-210
- severity: minor
- category: fact
- claim: "\textbf{Arithmetic Instructions} (\mnemonic{ADDI}, \mnemonic{SUBI}, \mnemonic{ADCI}, \mnemonic{SBCI})" ... "\textbf{Load Instructions} (\mnemonic{LOAD}, \mnemonic{LDEA}): Never update the condition flags"
- evidence: facts.md Opcode map (ADDR/SUBR/ADCR/SBCR and ANDR/ORRR/XORR update flags in the same way); write_back.rs:300-307 (memory/EA opcodes 0x10-0x1F never update SR, including STOR and LDEL). The compare and test bullets list both I and R forms, but the arithmetic and logical bullets list only I forms, and the no-flag bullet omits STOR, LDEL and the branch/jump aliases.
- resolution: manual-wrong
- fix: List the R forms in the arithmetic and logical bullets (ADDR, SUBR, ADCR, SBCR; ANDR, ORRR, XORR). Replace the load bullet with "\textbf{Memory and Address Instructions} (\mnemonic{LOAD}, \mnemonic{STOR}, \mnemonic{LDEA}, \mnemonic{LDEL} and their aliases): Never update the condition flags; AF selects the address register for these opcodes."
- confidence: high

### F-data-13
- file: reference/chapters/05-status-register.tex
- lines: 54
- severity: nit
- category: fact
- claim: "\item[Tested by:] Equal (\texttt{==}), Not Equal (\texttt{!=}) condition codes"
- evidence: reference/chapters/10-condition-codes.tex:33-38 (HI, LO, >> and <= also test Z). The N, C and V entries list their compound users; Z does not.
- resolution: manual-wrong
- fix: "\item[Tested by:] Equal (\texttt{==}), Not Equal (\texttt{!=}), and the compound conditions \texttt{HI}, \texttt{LO}, \texttt{>>}, \texttt{<=}"
- confidence: high

### F-data-14
- file: reference/chapters/10-condition-codes.tex
- lines: 314-316
- severity: minor
- category: fact
- claim: "A predicated instruction costs the same six cycles whether or not it executes"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/memory_access.rs:25-27 (a condition-false LOAD/STOR makes no phase-4 bus access); reference/chapters/14-memory-instructions.tex:95,264,421 (wait states extend the memory-access phase). An executed LOAD/STOR with data wait states takes longer than a skipped one, and both are subject to fetch wait states.
- resolution: manual-wrong
- fix: "A predicated instruction takes six cycles, plus instruction-fetch wait states, whether or not it executes. A skipped \mnemonic{LOAD} or \mnemonic{STOR} performs no data access, so it does not incur the data wait states it would incur when executed."
- confidence: high

### F-data-15
- file: reference/chapters/10-condition-codes.tex
- lines: 92-93
- severity: minor
- category: fact
- claim: "Prefer \texttt{HI}/\texttt{LO} over \texttt{CS}/\texttt{CC} for unsigned comparisons if that is easier to remember, since their names match their meaning directly."
- evidence: reference/chapters/10-condition-codes.tex:88-91 (HI is >, LO is <=, CS is <, CC is >=). They test different relations, so neither pair can replace the other. Unsigned < and >= have no condition other than CS and CC.
- resolution: manual-wrong
- fix: "\texttt{HI}/\texttt{LO} and \texttt{CS}/\texttt{CC} are not interchangeable: use \texttt{HI} for $>$, \texttt{LO} for $\leq$, \texttt{CS} for $<$ and \texttt{CC} for $\geq$."
- confidence: high

### F-data-16
- file: reference/chapters/10-condition-codes.tex
- lines: 289-302
- severity: minor
- category: fact
- claim: "If equal                 & \texttt{CMPR a, b}            & \texttt{==}" / "If negative              & \texttt{CMPI a, \#0}          & \texttt{NS}"
- evidence: facts.md Addressing modes (`a` is an address-register name, not a register-direct operand; `b` is not a register); sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:99-199. Written in assembler syntax, these rows would be rejected by the assembler.
- resolution: manual-wrong
- fix: Use real registers, for example `CMPR r1, r2` and `CMPI r1, \#0`, and write the Intent column as "If r1 $>$ r2 (unsigned)" and so on.
- confidence: high

### F-data-17
- file: reference/chapters/10-condition-codes.tex
- lines: 334-337
- severity: minor
- category: fact
- claim: "; CORRECT - flags preserved\nCMPR r1, r2\nBRAN|HI @target\nADDI r3, #100         ; After the branch"
- evidence: none (internal). The "correct" version is not equivalent to the "wrong" one: the add no longer runs when the branch is taken. The way to keep both the add and the flags is to suppress the flag update (reference/chapters/05-status-register.tex:239 uses `[N]` for this).
- resolution: manual-wrong
- fix: Replace the CORRECT listing with "CMPR r1, r2 / ADDI[N] r3, #100 ; [N]: flags untouched / BRAN|HI @target".
- confidence: high

### F-data-18
- file: reference/chapters/10-condition-codes.tex
- lines: 55, 140, 340
- severity: nit
- category: terminology
- claim: "Typically used after \mnemonic{CMP} to test equality."
- evidence: facts.md Opcode map (the mnemonics are CMPI and CMPR; there is no CMP)
- resolution: n/a
- fix: Write "\mnemonic{CMPI}/\mnemonic{CMPR}" (or "a compare instruction") at lines 55, 59-64, 140 and 340.
- confidence: high

### F-data-19
- file: reference/chapters/04-data-representation.tex
- lines: 126-127
- severity: minor
- category: fact
- claim: "Those forms use 16-bit immediate displacements, register displacements, or assembler-resolved full-width branch offsets."
- evidence: facts.md Opcode map (BRAN = LDEA p,(#n,p) with a 16-bit imm16 displacement) and Linker (`.r` resolves to an i16 and panics if out of range; sirc-vm/toolchain/src/bin/linker.rs:40-86). "Full-width" suggests a 24-bit offset; branch offsets are 16-bit signed and reach only within the current segment.
- resolution: manual-wrong
- fix: "Those forms use 16-bit signed immediate displacements (including assembler-resolved PC-relative branch offsets) or register displacements."
- confidence: high
