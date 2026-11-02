# Findings: G4 (formats, addressing, shifts), reviewer tag `enc`

Scope: `reference/chapters/07-instruction-formats.tex`, `08-addressing-modes.tex`, `09-shift-operations.tex`.

Checked and found consistent (no finding): all 33 worked encodings in `generated/immediate-format-encodings.tex`,
`generated/short-immediate-format-encodings.tex` and `generated/register-format-encodings.tex` (recomputed from the
field layout; every hex value matches its fields and every hex value has 8 digits); the decode-tree opcode bit splits
(op[5:4], op[3:2], op[1], op[0]) against the opcode map; the condition-code table against `definitions.rs:47-67`; the
shift-type table against `definitions.rs:69-81`; the `ADDR r4, r5` R1/R2/R3 note; PC-relative base = own address
(`fetch_and_decode.rs:233-234`); LSL/LSR/ASR carry rules and the >=16 clamp (`alu.rs:368-483`); post-increment and
pre-decrement worked comments; the zero-displacement shorthand (`instruction.rs:150-160`, `addressing_mode_test.rs:52-169`).

### F-enc-1
- file: reference/chapters/07-instruction-formats.tex
- lines: 608-610
- severity: blocker
- category: contradiction
- claim: "Every read of \reg{p}, every address calculation that selects \reg{p}, and every link value uses that latched address, so they all see the instruction's own address"
- evidence: reference/chapters/15-control-flow.tex:271-272 ("the BRSR link value is that address plus 2 (the address of the next instruction)"); reference/chapters/08-addressing-modes.tex:162-163 ("still link to the instruction's own address + 2"); sirc-vm/peripheral-cpu/tests/instructions/ljmp_test.rs:225,292 (pl 0xFAC0, link holds 0xFAC2)
- resolution: manual-wrong
- fix: "Every read of \reg{p} and every address calculation that selects \reg{p} uses that latched address, so they see the instruction's own address even though \reg{pl} has already advanced by the time the address is calculated in phase 3. A link value is the latched address plus 2, the address of the next instruction."
- confidence: high

### F-enc-2
- file: reference/chapters/09-shift-operations.tex
- lines: 357, 334-336
- severity: major
- category: fact
- claim: "A shift count of 0 leaves the operand unchanged and clears the carry flag." / "a count of 16 behaves as a count of 0"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:485-515 (RTL sets C = result bit 0 and RTR sets C = result bit 15 for every count, so `RTL #0` or a register count of 16 on 0x0001 sets C=1; LSL/LSR/ASL/ASR do give C=0 at count 0). No test exercises RTL/RTR at all (grep of peripheral-cpu/tests for RotateLeft/RotateRight finds nothing). Not covered by general-todo.md (29 covers ASL/ASR only); related to facts.md Open question 14.
- resolution: code-wrong
- fix: Keep the manual. New simulator todo: for RTL/RTR, when the effective count (immediate, or register value mod 16) is 0, clear C instead of copying bit 0/bit 15; add tests for RTL/RTR with counts 0, 1, 15, and register counts 16 and 17.
- confidence: high

### F-enc-3
- file: reference/chapters/09-shift-operations.tex
- lines: 374-376
- severity: major
- category: fact
- claim: "For \textbf{ASL}: Set if bit 15 changes at any point during the shift (signed overflow); that is, for a count $n$ from 1 to 15, set unless the original bits 15 down to $15 - n$ are all equal."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:417-443 (V from `set_alu_bits(..., Some((a, a, result)))`, which compares only the final sign with the original sign); general-todo.md item 29
- resolution: code-wrong
- fix: Keep the manual; tracked as general-todo 29. The same rule is restated at 09-shift-operations.tex:144 and :160 ("sets the overflow flag if the sign bit changes"); those are consistent with it.
- confidence: high

### F-enc-4
- file: reference/chapters/08-addressing-modes.tex
- lines: 431-433
- severity: major
- category: fact
- claim: "the auto-update forms must not name \reg{p} as the address-register pair ... Such an encoding is architecturally undefined and the assembler rejects it"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/load.rs:144,201 (rejects only a destination that aliases the pair; `LOAD r1, (#0, p)+` is accepted); only LJSR rejects p (ljsr.rs:161-162; control_flow_test.rs:302); general-todo.md item 27
- resolution: code-wrong
- fix: Keep the manual; tracked as general-todo 27 (also covers STOR -(#n, p), LDEA -(#n, p), LDEL (#n, p)+).
- confidence: high

### F-enc-5
- file: reference/chapters/08-addressing-modes.tex
- lines: 91
- severity: major
- category: contradiction
- claim: "\texttt{OPR rD, rS1, rS2[, shift]}; register shorthand \texttt{OPR rD, rS2}."
- evidence: reference/chapters/09-shift-operations.tex:67 uses `ADDR r1, r2, LSL #1` (shorthand with a shift); sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:142-165 accepts `[reg, reg, shift]`. The table claims to list "exactly which operand forms are legal" (08:79-82).
- resolution: manual-wrong
- fix: "\texttt{OPR rD, rS1, rS2[, shift]}; register shorthand \texttt{OPR rD, rS2[, shift]} (encoded with R2 = rD, so the shift applies to rD)."
- confidence: high

### F-enc-6
- file: reference/chapters/07-instruction-formats.tex
- lines: 98, 158, 228, 485, 513, 542
- severity: major
- category: fact
- claim: "11 = Reserved for future use"
- evidence: No chapter says what the CPU does when an ALU or COP opcode is executed with AF = 11 (reference/chapters/11-reading-instructions.tex:137 and appendix-c-undocumented.tex:113 only say "reserved"). Code leaves SR unchanged, i.e. behaves as [N] (sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:154; facts.md "Update source None (00), and the reserved value 11, leave SR unchanged").
- resolution: design
- fix: Options: (a) state that the effect on SR bits 7-0 is architecturally undefined and assemblers never emit it (parallels the reserved shift type, 09:497); (b) define it as identical to [N] (matches the simulator); (c) make it an invalid-opcode fault. Recommend (a), with one sentence in Ch. 7 under the AF description and a pointer from Appendix C, so an emulator author knows nothing is guaranteed.
- confidence: medium

### F-enc-7
- file: reference/chapters/07-instruction-formats.tex
- lines: 62, 485
- severity: minor
- category: improvement
- claim: "\textbf{Note:} Due to the large 16-bit operand, shift is not supported in this format." / "AF (5--4) & Status update source: \texttt{00} none, \texttt{01} ALU result, \texttt{10} shifter result"
- evidence: reference/chapters/13-alu-instructions.tex:198-201 defines that [S] on an immediate-format instruction clears N, Z, C and V; Ch. 7 lists AF=10 "shifter result" for the immediate group with no hint that there is no shifted operand. Same gap in 09-shift-operations.tex:603-608 (Assembler Syntax Notes).
- resolution: n/a
- fix: Append to the note at 07:62: "AF = 10 is still encodable; with no shift fields the shifter result is treated as zero flags, so it clears N, Z, C and V (Chapter~\ref{ch:alu-instructions})." Add a matching bullet at 09:603-608.
- confidence: high

### F-enc-8
- file: reference/chapters/08-addressing-modes.tex
- lines: 92
- severity: minor
- category: fact
- claim: "\texttt{CMNR/CNCR/CMPR/CPCR/TSAR/TSOR/TSXR rS1, rS2[, shift]}."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:120-140,166-189 accept a three-register form for the test mnemonics too (`CMPR r1, r2, r3` encodes R1=r1 and compares r2 with r3). reference/chapters/13-alu-instructions.tex:99,105,1671-1672 and assembly/chapters/04-instructions.tex list only the two-register form.
- resolution: code-wrong
- fix: Keep the manual; toolchain todo: reject a three-register operand list for CMNR..TSXR (or, if the author prefers to keep it, document the form in the assembly manual and here).
- confidence: medium

### F-enc-9
- file: reference/chapters/08-addressing-modes.tex
- lines: 221-225
- severity: minor
- category: fact
- claim: "\mnemonic{LJMP}/\mnemonic{LJSR} are meta-instructions that lower to this same encoding with \reg{p} as the named pair."
- evidence: The box says LJMP/LJSR use Address Register Direct for the *source* pair (08:212-213), but the field encoding only describes bits 23-22. The source pair goes in AF bits 5-4 (generated/register-format-encodings.tex `LJMP a, r2`: R1=0x3, AF=01; ljmp.rs:63-140).
- resolution: manual-wrong
- fix: "... lower to this same encoding with \reg{p} in bits 23--22; their source pair, also named directly, is encoded in AF, bits 5--4."
- confidence: high

### F-enc-10
- file: reference/chapters/09-shift-operations.tex
- lines: 486-487
- severity: minor
- category: contradiction
- claim: "Using shifts for multiplication/division by powers of 2 is much faster than using a multiply/divide instruction (if one existed)"
- evidence: reference/chapters/16-coprocessor-instructions.tex:1257,1289 define MULU/MULS/DIVU/DIVS on the optional maths coprocessor.
- resolution: manual-wrong
- fix: "Using shifts for multiplication or division by powers of 2 avoids a call to the optional maths coprocessor (\mnemonic{MULU}, \mnemonic{DIVU}, Chapter~\ref{ch:coprocessor})."
- confidence: high

### F-enc-11
- file: reference/chapters/09-shift-operations.tex
- lines: 198-200
- severity: minor
- category: fact
- claim: "ADDI r3, #0, ASR #1  ; r3 = (r3 >> 1) = (r4 + r5) / 2"
- evidence: First principles: the 16-bit ADDR drops the 17th bit, so r4 = r5 = 0x7000 gives r3 = 0xE000, then ASR gives 0xF000 (-4096), not 0x7000. The identity holds only if r4 + r5 does not overflow.
- resolution: manual-wrong
- fix: Change the comment to "; r3 = (r4 + r5) / 2, if r4 + r5 does not overflow" (or add a note that V from the ADDR must be clear).
- confidence: high

### F-enc-12
- file: reference/chapters/09-shift-operations.tex
- lines: 207-208, 222, 241-242, 256
- severity: minor
- category: fact
- claim: "Bit 15 wraps into bit 0 and is copied into the carry flag" / "Bit 0 wraps into bit 15 and is copied into the carry flag"
- evidence: For a count n > 1, C holds the last bit to wrap: original bit 16 - n for RTL, bit n - 1 for RTR (alu.rs:485-515; 09:355-356). Read literally, the bullets say C = original bit 15 / bit 0 for any count.
- resolution: manual-wrong
- fix: RTL: "The last bit to wrap from bit 15 to bit 0 (original bit 16 - n for a count of n) is copied into the carry flag." RTR: "The last bit to wrap from bit 0 to bit 15 (original bit n - 1) is copied into the carry flag." Match the LSL/LSR bullets at 09:93 and 09:125.
- confidence: high

### F-enc-13
- file: reference/chapters/09-shift-operations.tex
- lines: 569-573
- severity: minor
- category: fact
- claim: "; Check if r1 << 3 would be negative" / "ADDI[S] r1, #0, LSL #3        ; r1 = r1 << 3, flags from shift" / "BRAN|NS @overflow_detected"
- evidence: "would be" implies r1 is not changed, but ADDI writes r1. NS tests N, not V, so the label name misdescribes the condition.
- resolution: manual-wrong
- fix: Use "TSOI[S] r1, #0, LSL #3   ; flags from r1 << 3, r1 unchanged" and rename the label `@shifted_negative`.
- confidence: high

### F-enc-14
- file: reference/chapters/09-shift-operations.tex
- lines: 606
- severity: minor
- category: fact
- claim: "The override applies to the entire instruction, including any conditional execution"
- evidence: A false condition changes nothing, including SR (facts.md "Condition false"; write_back.rs:104-106). The sentence can be read as saying flags update even when the condition fails.
- resolution: manual-wrong
- fix: "The override selects the flag source for the whole instruction. If the condition is false, the instruction has no effect and no flags change, whatever the override."
- confidence: medium

### F-enc-15
- file: reference/chapters/08-addressing-modes.tex
- lines: 344
- severity: minor
- category: fact
- claim: "\textbf{Usage:} Loading constant values; arithmetic with known values; bit manipulation with masks."
- evidence: The short-immediate form cannot load a constant: there is no public short-immediate MOVI (08:93), and 0x27 is undocumented (07:351-353). Every short-immediate ALU op combines a shifted register with the immediate.
- resolution: manual-wrong
- fix: "\textbf{Usage:} Shift-and-add or shift-and-mask in one instruction, for example scaling a register and adding a small constant."
- confidence: high

### F-enc-16
- file: reference/chapters/07-instruction-formats.tex
- lines: 359-367
- severity: minor
- category: fact
- claim: "\textbf{Opcodes (Imm / Short / Reg)}" ... "Add & ADDI / ADDR & CMNI / CMNR & \opcode{08} / \opcode{28} / \opcode{38}"
- evidence: The column header implies the opcodes of the row, but only the test-only opcodes are listed; ADDI/ADDR are 0x00/0x20/0x30 (appendix-a-opcode-map; facts.md opcode map).
- resolution: manual-wrong
- fix: Rename the column "Test-only opcodes (Imm / Short / Reg)", or add a "Save opcodes" column with 00/20/30 ... 06/26/36.
- confidence: high

### F-enc-17
- file: reference/chapters/07-instruction-formats.tex
- lines: 172
- severity: nit
- category: terminology
- claim: "\texttt{0x5} = Negative (<0)"
- evidence: The condition table (07:282) names 0x5 "Negative Set (N = 1)"; "<0" reads like signed less-than, which is 0xC (`<<`, N != V) and differs when V = 1.
- resolution: n/a
- fix: "\texttt{0x5} = Negative Set (NS, N = 1)"
- confidence: high

### F-enc-18
- file: reference/chapters/09-shift-operations.tex
- lines: 588
- severity: nit
- category: fact
- claim: "BRAN|>= @r1_was_greater"
- evidence: `>=` (code 0xB, N = V) is greater-or-equal, so the branch is also taken when r1 = r2.
- resolution: manual-wrong
- fix: "BRAN|>= @r1_was_greater_or_equal"
- confidence: high
