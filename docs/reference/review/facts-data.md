# Findings: chapters 04, 05, 10 (data, status, condition codes)

Reviewer tag: data. Digest: docs/reference/review/facts.md.

### F-data-1
- file: chapters/10-condition-codes.tex
- lines: 74-86
- severity: blocker
- category: fact
- claim: "HI (Unsigned Higher) Executes when C = 1 AND Z = 0. Tests if first operand $>$ second operand (unsigned)."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:195-206 (perform_subtract sets C from a.overflowing_sub(b), i.e. C = borrow = a < b unsigned); sirc-vm/peripheral-cpu/tests/instructions/arithmetic_immediate_test.rs:260-272 (SUBI 0x5FFF - 0xFFFF sets Carry); definitions.rs:128-136 (UnsignedHigher = C AND NOT Z, UnsignedLowerOrSame = NOT C OR Z); docs/reference/manual-handover.md:19 "Resolved: subtraction sets C when a borrow occurs"; docs/reference/chapters/13-alu-instructions.tex:147 "C is set when a borrow occurs."
- resolution: unclear
- fix: With C set on borrow, after CMP a, b the true mapping is CS = HI = unsigned lower (a < b) and CC = LO = unsigned higher or same (a >= b); there is no single condition code for strict unsigned greater-than. Either the chapter must be rewritten to state this (and the mnemonic glosses HI/LO relabelled), or definitions.rs:128-136 must invert the two predicates. Both sides: the code plus tests plus handover fix C = borrow; the DEF comment at definitions.rs:114 says the predicate set was copied from ARM, which defines C after subtraction as NOT borrow, so the ARM-derived HI/LO predicates were never re-derived for this convention. Same error occurs at lines 30-31 (table rows for HI and LO), 84-85 (CS/CC equivalences), 138 and 142 (unsigned examples), 181-184 (Unsigned Comparison Results table), 219-223 (conditional assignment example), 249 (loop example) and 263-266 (selection guide rows HI, CS, CC, LO), and in 05-status-register.tex:69-71. Fix once, then propagate.
- confidence: high

### F-data-2
- file: chapters/05-status-register.tex
- lines: 10
- severity: blocker
- category: contradiction
- claim: "\textbf{Lower Byte (bits 7--0):} Condition flags -- readable and writable in both supervisor and protected modes"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:21-27, 29-51 (sr is in PRIVILEGED_REGISTERS; any instruction whose destination is sr raises PrivilegeViolation in protected mode, whatever the bit); sirc-vm/peripheral-cpu/tests/instructions/protected_mode_test.rs:95-132; the same chapter contradicts itself at lines 207-209 ("any direct write to the status register raises a privilege violation fault")
- resolution: manual-wrong
- fix: Replace with: "\textbf{Lower Byte (bits 7--0):} Condition flags -- readable in both modes, and updated as a side effect of ALU and shift instructions in both modes; a direct write to \reg{sr} is privileged and raises a privilege violation fault in protected mode."
- confidence: high

### F-data-3
- file: chapters/04-data-representation.tex
- lines: 97-106
- severity: blocker
- category: contradiction
- claim: "Vector entry \texttt{V} starts at word address \texttt{V * 2}"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:304-306 (vector_address = registers.system_ram_offset | (vector * INSTRUCTION_SIZE_WORDS)); docs/reference/chapters/06-exceptions.tex:443 and 615-616 both add the system RAM offset
- resolution: manual-wrong
- fix: State the base: "Vector entry V starts at word address system_ram_offset + V * 2", and change the worked example to read "with a vector table base of 0, vector 0x06 occupies word addresses 0x00000C and 0x00000D". Note the internal vector-table base register is not program-visible (registers.rs:162-169).
- confidence: high

### F-data-4
- file: chapters/05-status-register.tex
- lines: 299
- severity: blocker
- category: fact
- claim: "ADDI r1, r2, r3"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_immediate.rs:169, 206, 271 (the only accepted operand shapes are register+immediate and register+immediate+shift; anything else fails with "The [ADDI] opcode only supports immediate->register addressing mode"); the three-register form belongs to ADDR, sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:114
- resolution: manual-wrong
- fix: Change the listing line to "ADDR r1, r2, r3" (or "ADDI r1, #1").
- confidence: high

### F-data-5
- file: chapters/10-condition-codes.tex
- lines: 245
- severity: blocker
- category: fact
- claim: "ADDI r1, #0, #10         ; i = 10"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_immediate.rs:206 (the third operand of the short-immediate form must be a shift definition, not an immediate) and :271 (all other shapes are rejected)
- resolution: manual-wrong
- fix: Replace with "LOAD r1, #10             ; i = 10".
- confidence: high

### F-data-6
- file: chapters/04-data-representation.tex
- lines: 90-92
- severity: blocker
- category: contradiction
- claim: "Instruction fetch reads the high word first, increments \reg{p} by one word, reads the low word, then increments \reg{p} by one word again."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:89-117 (phase 0 drives the address get_full_pc_address(), phase 1 drives get_full_pc_address() + 1; pl is not written between them) and :151 (registers.pl = npc_l_, that is PC + 2, at decode); execution.rs:83-86 raises an Alignment fault whenever pl is odd at any phase, so a pl that is odd during the low-word fetch would fault on every instruction
- resolution: manual-wrong
- fix: Replace with: "Instruction fetch reads the high word from the address in \reg{p} and the low word from that address plus one. \reg{p} itself is not modified during the fetch; it advances by two words during instruction decode, so an instruction that reads or uses \reg{p} as an address register sees the address of the instruction itself."
- confidence: medium

### F-data-7
- file: chapters/05-status-register.tex
- lines: 187-205
- severity: major
- category: fact
- claim: "In supervisor mode, the status register can be directly manipulated like any other register:"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:199-202 (AluToRegister writes the destination register and then calls update_status_flags) and :164-179 (the flag update replaces the whole low byte of sr from the ALU or shift status). For "ORRI sr, #0x0100" the low byte written by the OR is therefore discarded and replaced by the N/Z/C/V of the OR result; only the privileged high byte survives.
- resolution: manual-wrong
- fix: Add after the listing: "An ALU instruction that targets \reg{sr} writes its result to \reg{sr} and then overwrites bits 7--0 with the flags produced by that same instruction, so only bits 15--8 can be set this way. To write the low byte, suppress the flag update with the [N] status source (for example ORRI[N] sr, #0x0003) or use LOAD sr, rN." Verify the intended examples still hold.
- confidence: high

### F-data-8
- file: chapters/05-status-register.tex
- lines: 228-233
- severity: major
- category: fact
- claim: "Interrupts may be masked depending on exception type"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:231-271 (exception entry clears P, clears T, sets EA, and writes PC; the four hardware-interrupt-enable bits and the A bit are not modified); masking during a handler comes only from the level comparison at execution.rs:182-183
- resolution: manual-wrong
- fix: Replace item 4 with: "The hardware interrupt enable bits are not changed. A pending interrupt is serviced during a handler only if its priority level is higher than the current exception level."
- confidence: high

### F-data-9
- file: chapters/05-status-register.tex
- lines: 84-86
- severity: major
- category: fact
- claim: "Bits 4 through 7 of the lower byte are reserved for future use. They should not be relied upon by software."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:170-177 (every ALU or shift flag update writes (sr AND 0xFF00) OR (status AND 0x00FF), replacing the whole low byte) and stages/alu.rs:545-593 (set_alu_bits only ever writes Z, N, C, V into a status word that starts at zero, so bits 4-7 of that word are always 0)
- resolution: manual-wrong
- fix: Keep the advisory wording required by STYLE.md and add the observable rule: "These bits can be written by a supervisor-mode direct write to \reg{sr}, but any subsequent ALU or shift flag update clears them to zero. They read as zero after any flag-updating instruction."
- confidence: medium

### F-data-10
- file: chapters/04-data-representation.tex
- lines: 154-157
- severity: major
- category: fact
- claim: "If the low word overflows or underflows and \texttt{SR.A} (Trap on Address Overflow) is set, the instruction raises a segment-overflow fault."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:76-115. The only test is an unsigned 16-bit carry out of A.low + displacement and of (A.low + displacement) + addr_inc, where addr_inc for a pre-decrement is 0xFFFF. Consequences: (a) a pre-decrement whose displaced address is nonzero always sets the carry and therefore always faults when A is set, even though no underflow occurred; (b) a genuine underflow (displaced address 0, decrement to 0xFFFF) produces no carry and does not fault. No test covers pre-decrement or underflow with A set (sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:640-693 uses a positive displacement only).
- resolution: unclear
- fix: Either the manual must describe the actual rule (the trap fires on unsigned carry out of the low-word addition, which is not the same as underflow detection), or execution_effective_address.rs:88-115 must compute borrow separately for the decrement path. Present both sides until the author rules.
- confidence: medium

### F-data-11
- file: chapters/04-data-representation.tex
- lines: 119-124
- severity: major
- category: fact
- claim: "Memory displacements           & 16 bits        & Two's-complement offset added to the low address word"
- evidence: Open question 8 of docs/reference/review/facts.md; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:77-83 and :110-115. The displacement is added as an unsigned 16-bit value, so it wraps identically to a two-complement add only while SR.A is clear; with SR.A set, any negative displacement whose add wraps (for example LOAD r1, (#-2, a) with al >= 2) raises SegmentOverflow. Tests use positive displacements only.
- resolution: unclear
- fix: State explicitly whether negative displacements are architecturally usable while SR.A is set, and if the current behaviour is intended, add a sentence to this table row and to the Arithmetic and Address Wraparound section: "A displacement whose addition wraps the low word raises a segment-overflow fault when SR.A is set, including displacements intended as negative values."
- confidence: high

### F-data-12
- file: chapters/05-status-register.tex
- lines: 183
- severity: major
- category: fact
- claim: "\textbf{Load Instructions} (\mnemonic{LOAD}, \mnemonic{LDEA}): Does not update condition flags"
- evidence: Open question 9 of docs/reference/review/facts.md. Three observable behaviours: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:307-317 with AF=Alu sets N and Z from the loaded value and clears C and V (tests/instructions/arithmetic_register_test.rs:497-534); the assembler always emits AF=None for LOAD (toolchain/src/parsers/opcodes/load.rs:47, 128), so assembled LOAD never updates flags; memory loads never update flags whatever AF is (stages/write_back.rs:308-315). LDEA never updates flags (write_back.rs:203-236).
- resolution: unclear
- fix: Decide which behaviour is architectural and say so here. Suggested wording once ruled: "LDEA and memory-form LOAD never update the condition flags. Register and immediate LOAD update flags only when the status source field selects the ALU; the assembler emits None for every LOAD form, so assembled LOAD never updates flags."
- confidence: high

### F-data-13
- file: chapters/04-data-representation.tex
- lines: 130-133
- severity: major
- category: fact
- claim: "Assemblers must reject values that cannot be represented in the selected immediate field width unless they deliberately offer a documented truncation mode."
- evidence: sirc-vm/toolchain/src/parsers/instruction.rs:108-116 (parse_value casts the parsed 32-bit number straight to u16 with a TODO acknowledging the unchecked cast), so ADDI r1, #70000 assembles silently as 0x1170 and CMPI r1, #-10 assembles as 0xFFF6. Only the 8-bit short-immediate-with-shift form is range checked (toolchain/src/parsers/opcodes/arithmetic_immediate.rs:214-222).
- resolution: unclear
- fix: Either relax the sentence to describe what the reference assembler does (values are truncated to the field width; only short immediates with a shift are diagnosed) or keep it as a requirement and record that the reference assembler does not yet conform. Note that chapter 10 line 153 relies on the truncation behaviour with CMPI r1, #-10.
- confidence: high

### F-data-14
- file: chapters/05-status-register.tex
- lines: 154-161
- severity: major
- category: fact
- claim: "\item[When Set (1):] Memory address calculations that overflow or underflow will trigger an address overflow exception"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:92-97 and :127-137 (the A bit also gates the program-counter wrap trap, which is raised at the next instruction fetch); tests/exceptions/faults.rs:696-758. The fault raised in both cases is SEGMENT_OVERFLOW_FAULT, vector 0x03 (exception_unit/definitions.rs:37).
- resolution: manual-wrong
- fix: Extend the description to cover instruction fetch, and name the fault consistently with chapter 4 and chapter 6: "When set, an effective-address calculation or a program-counter advance that wraps the low word raises a segment-overflow fault (vector 0x03). For a program-counter wrap the fault is raised at the following instruction fetch."
- confidence: high

### F-data-15
- file: chapters/10-condition-codes.tex
- lines: 281-288
- severity: minor
- category: fact
- claim: "Conditional instructions do not branch, avoiding pipeline flushes"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:66-67 (CYCLES_PER_INSTRUCTION = 6) and coprocessors/shared.rs:6-14: every instruction of every class occupies exactly six cycles and there is no overlapped pipeline to flush, so a taken branch costs the same six cycles as any other instruction
- resolution: manual-wrong
- fix: Replace the first bullet with: "A predicated instruction costs the same six cycles whether or not it executes; the saving over a branch is the branch instruction itself, not a pipeline penalty, since the SIRC-1 has no overlapped pipeline to flush."
- confidence: high

### F-data-16
- file: chapters/05-status-register.tex
- lines: 101
- severity: minor
- category: fact
- claim: "Privileged coprocessor operations (opcodes > 0x07xx)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:44-48 (cop_opcode = command AND 0x0F00, privileged when greater than 0x0700), which is the opcode nibble in bits 11--8 of the coprocessor command word regardless of the coprocessor ID in bits 15--12
- resolution: manual-wrong
- fix: Replace with "Coprocessor commands whose opcode field (bits 11--8 of the command word) is greater than 7".
- confidence: high

### F-data-17
- file: chapters/05-status-register.tex
- lines: 124-127
- severity: minor
- category: fact
- claim: "Disabled interrupts are not queued -- they are completely ignored while the enable bit is 0"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:294-297 and :499-518 (the interrupt pins are sampled every cycle and latched only if enabled at that moment); docs/reference/manual-handover.md:473-479 records the pins as level sensitive
- resolution: manual-wrong
- fix: Add the level-sensitive consequence: "A line that is still asserted when its enable bit is set is latched at that point; only assertions that end while the bit is 0 are lost."
- confidence: medium

### F-data-18
- file: chapters/10-condition-codes.tex
- lines: 9-10
- severity: minor
- category: fact
- claim: "If the condition is not met, the instruction behaves as a \mnemonic{NOOP} (no operation)."
- evidence: Open question 2 of docs/reference/review/facts.md. A false condition suppresses every effect including the decode-time privilege check (processing_unit/execution.rs:35-37; tests/instructions/protected_mode_test.rs:135-150) and the pending coprocessor command (protected_mode_test.rs:497-509), whereas the NOOP meta-instruction itself assembles to ADDI with register field 0, which is \reg{sr} (toolchain/src/parsers/opcodes/meta.rs:80-95) and therefore raises PrivilegeViolation in protected mode (processing_unit/execution.rs:21-27). No test covers NOOP in protected mode.
- resolution: unclear
- fix: Avoid defining a false condition in terms of NOOP: "If the condition is not met the instruction has no architectural effect: no register, flag, memory, address-register or coprocessor state changes, and no privilege check is performed." The same substitution is needed at line 105 for the NV entry.
- confidence: medium

### F-data-19
- file: chapters/10-condition-codes.tex
- lines: 6
- severity: minor
- category: fact
- claim: "Most SIRCIS instructions can be executed conditionally based on the state of the condition flags in the status register."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/encoding.rs:47-59 (bits 3:0 are the condition code in all three instruction formats, so every encoding is predicated); toolchain/tests/assembler/coprocessor_test.rs:62-74 shows even COPI takes a condition suffix. Also "SIRCIS" is not the processor name used anywhere else in the manual.
- resolution: manual-wrong
- fix: "Every SIRC-1 instruction can be executed conditionally based on the state of the condition flags in the status register."
- confidence: high

### F-data-20
- file: chapters/05-status-register.tex
- lines: 202-203, 292-304
- severity: minor
- category: fact
- claim: "; Restore original status\nLOAD sr, r7"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:55-72 (a write to sr always preserves bit 13, ExceptionActive, even in supervisor mode)
- resolution: manual-wrong
- fix: Add a comment or note to both restore examples: bit 13 (EA) is preserved from the current value and is not restored by a direct write; only RETE restores it.
- confidence: high

### F-data-21
- file: chapters/04-data-representation.tex
- lines: 158-161
- severity: minor
- category: fact
- claim: "The program counter follows the same low-word wrap rule during normal instruction fetch."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:92-97 and :127-137: a program-counter wrap with SR.A set raises SegmentOverflow, but the fault is deferred to the next instruction fetch rather than raised on the instruction that wrapped
- resolution: manual-wrong
- fix: Add: "If the program counter wraps the low word while SR.A is set, a segment-overflow fault is raised at the next instruction fetch, not on the instruction that caused the wrap."
- confidence: high

