# Fact-check findings — chapter group G4 (formats, addressing, shifts)

Reviewer tag: `enc`. Chapters reviewed: `chapters/07-instruction-formats.tex`,
`chapters/08-addressing-modes.tex`, `chapters/09-shift-operations.tex`.
Digest used: `docs/reference/review/facts.md`. Code paths below are relative to the repo root.

### F-enc-1
- file: chapters/07-instruction-formats.tex
- lines: 278-280
- severity: blocker
- category: fact
- claim: "1100          & <                 & Signed Less Than (N $\neq$ V)              \\ ... 1110          & <<                & Signed Less or Equal (Z = 1 OR N $\neq$ V) \\"
- evidence: sirc-vm/toolchain/src/parsers/instruction.rs:317-357 (`"<<" => ConditionFlags::LessThan`, `"<=" => ConditionFlags::LessThanOrEqual`); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:60-64
- resolution: manual-wrong
- fix: Row 1100 mnemonic becomes `<<` and row 1110 mnemonic becomes `<=`. The assembler has no `<` tag, and `<<` is Less Than, not Less or Equal. (Row 1101 `>>` and row 1011 `>=` are already correct.)
- confidence: high

### F-enc-2
- file: chapters/07-instruction-formats.tex
- lines: 329-336
- severity: blocker
- category: fact
- claim: "All ALU operations have both a \"save result\" and \"test only\" variant: ... \textbf{0x\_8-0x\_F}: Test only (update flags, don't save result) ... To turn a \"save result\" instruction to a \"test only\" instruction, you can simply add 0x8 to the opcode."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:132-151 (`0x08..=0x0E => AluStatusOnly`, `0x0F => CoprocessorCall`, same for 0x2F and 0x3F); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:61-67 (`simulate = alu_code != 0xF && alu_code & 0x8 == 0x8`)
- resolution: manual-wrong
- fix: Change the range to `0x_8--0x_E`: test only (update flags, do not save the result); `0x_F` is the coprocessor call (`COPI`/`COPR`), not a test-only form. Also restrict the "add 0x8" rule: it holds for `0x_0--0x_6`; `0x_7` (LOAD) + 0x8 gives the coprocessor call, not a test form.
- confidence: high

### F-enc-3
- file: chapters/07-instruction-formats.tex
- lines: 139
- severity: blocker
- category: fact
- claim: "\item[Immediate (bits 21--14)] 8-bit signed or unsigned immediate value"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:189-192 (`short_immediate_representation.value as u16`, i.e. zero-extended); sirc-vm/toolchain/src/parsers/opcodes/arithmetic_immediate.rs:214-222 (values that do not fit `u8` are rejected: "Immediate values must fit into 8 bits when using a shift definition"); docs/reference/manual-handover.md:54 ("Short immediates are documented as zero-extended ALU operands only")
- resolution: manual-wrong
- fix: "\item[Immediate (bits 21--14)] 8-bit unsigned immediate value, zero-extended to 16 bits before use"
- confidence: high

### F-enc-4
- file: chapters/07-instruction-formats.tex
- lines: 232-238
- severity: blocker
- category: contradiction
- claim: "ADDR r1, r2      ; r1 = r1 + r2 (R3 implicitly = R1)"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:91-113 (two-operand form sets `r1 = dest`, `r2 = dest`, `r3 = src`); docs/reference/generated/register-format-encodings.tex:38-39 (`CMPR r4, r5` -> R1=0x4, R2=0x4, R3=0x5); contradicts this chapter's own note at chapters/07-instruction-formats.tex:246
- resolution: manual-wrong
- fix: "ADDR r1, r2      ; r1 = r1 + r2 (R2 implicitly = R1)"
- confidence: high

### F-enc-5
- file: chapters/07-instruction-formats.tex
- lines: 349-356
- severity: major
- category: fact
- claim: "\item Fetch high word (bits 31--16) from address in PC \item Increment PC by 1 \item Fetch low word (bits 15--0) from address in PC \item Increment PC by 1"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:89-117 (phase 0 drives `registers.get_full_pc_address()`, phase 1 drives `get_full_pc_address() + 1`; `pl` is not modified in either phase) and :151 (`registers.pl = self.decoded_instruction.npc_l_` at decode); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:203 (`npc_l_ = pl + INSTRUCTION_SIZE_WORDS`)
- resolution: manual-wrong
- fix: Replace the four steps with: (1) fetch the high word from the address in \reg{pl}; (2) fetch the low word from \reg{pl} + 1; (3) after decode, \reg{pl} advances by 2. State explicitly that \reg{pl} still holds the address of the instruction being executed while it is decoded, because address calculations that select \reg{p} use that value (see F-enc-9).
- confidence: high

### F-enc-6
- file: chapters/07-instruction-formats.tex
- lines: 79-81, 194-198
- severity: major
- category: fact
- claim: "\item[Register (bits 25--22)] 4-bit destination/source register identifier"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:198-201 (`des_ad_l = 0x9 | immediate_representation.register << 1`, i.e. the low 2 bits of the register field select the destination address-register pair); docs/reference/generated/immediate-format-encodings.tex:20-30 (`LDEA a, (#4, s)` encodes Reg = 0x1, `LDEL p, (#0, a)` encodes Reg = 0x3)
- resolution: manual-wrong
- fix: Add to the Register field description in both the Immediate and Register format sections: for \mnemonic{LDEA}/\mnemonic{LDEL} (opcodes 0x18--0x1F) the field holds a 2-bit address-register-pair index in bits 23--22 (00 = l, 01 = a, 10 = s, 11 = p) naming the destination pair, using the same encoding as the AF field; bits 25--24 are unused in that case. Without this, the generated `LDEA`/`LDEL` examples cannot be reproduced from the field table.
- confidence: high

### F-enc-7
- file: chapters/07-instruction-formats.tex
- lines: 247-252
- severity: minor
- category: fact
- claim: "SO (Shift Operand): 0 = literal shift amount, 1 = register contains shift amount (see rows 7 and 8) ... AF for memory ops: ... (see SHFT row 9)"
- evidence: docs/reference/generated/register-format-encodings.tex:44-51 (SO = 1 appears in the 13th and 14th data rows, `SUBR|>>` and `ANDR|CS`; `SHFT r1, ASL #3` is the 15th row)
- resolution: manual-wrong
- fix: Change "rows 7 and 8" to "the \texttt{SUBR|>>} and \texttt{ANDR|CS} rows" and "SHFT row 9" to "the \texttt{SHFT} row"; row numbers drift whenever the generated table is regenerated, so name the rows by instruction instead.
- confidence: high

### F-enc-8
- file: chapters/07-instruction-formats.tex
- lines: 318-325
- severity: minor
- category: structure
- claim: "\subsection{ALU Instructions} ... \item \textbf{0x1\_}: Memory operations"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:136-143 (0x10--0x1F are store/load/LDEA/LDEL classes, none of which are ALU write-back classes)
- resolution: manual-wrong
- fix: Retitle the subsection "Opcode Groups" (or similar) so that the 0x1\_ memory group is not presented as an ALU group, and keep the "Test vs. Save" subsection scoped to 0x0\_, 0x2\_ and 0x3\_ only.
- confidence: high

### F-enc-9
- file: chapters/08-addressing-modes.tex
- lines: 127-129
- severity: blocker
- category: fact
- claim: "When \reg{p} is used as the source address register, the value used for address calculation is the program counter after the current instruction has been fetched. Since every instruction is two words, PC-relative branch displacements are relative to the next instruction address, not to the address of the branch instruction itself."
- evidence: sirc-vm/peripheral-cpu/tests/instructions/ljmp_test.rs:224-232 (`pl = 0xFAC0`, displacement 0x000E, expected `pl = 0xFACE`, i.e. base = the branch instruction's own address); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:151 (`pl` is advanced only after decode has already computed the effective address); facts.md:158
- resolution: manual-wrong
- fix: "When \reg{p} is used as the source address register, the value used for address calculation is the address of the instruction currently executing. PC-relative displacements are therefore relative to the address of the branch instruction itself, not to the next instruction; a displacement of 2 targets the following instruction and a displacement of 0 re-executes the branch." Note that \mnemonic{BRSR}/\mnemonic{LDEL} still link to \reg{pl} + 2.
- confidence: high

### F-enc-10
- file: chapters/08-addressing-modes.tex
- lines: 6-26
- severity: major
- category: fact
- claim: "The SIRC-1 CPU supports seven addressing modes ... Immediate / Register Direct / Indirect Immediate / Indirect Register / Post-Increment / Pre-Decrement / Short Immediate"
- evidence: docs/reference/manual-handover.md:17 fixes the count at seven but does not enumerate them; sirc-vm/toolchain/src/parsers/instruction.rs:85-106 distinguishes ten operand forms including `DirectAddressRegister` (`l`, `a`, `s`, `p`), which this list does not cover although the chapter's own family table uses it (`LDEA dest`, `LJMP src`, chapters/08-addressing-modes.tex:64-66); facts.md Open question 11
- resolution: unclear
- fix: Either (a) keep seven and replace "Short Immediate" (an immediate-width variant of Immediate, not a distinct operand access path) with "Address Register Direct" so that `LDEA a, ...` and `LJMP a` are covered, or (b) keep the current seven and add an explicit sentence that naming an address-register pair as a destination/source operand is not counted as an addressing mode. Both sides: the handover fixes the count at seven; the parser has ten operand forms and the current list leaves one legal operand form unclassified.
- confidence: medium

### F-enc-11
- file: chapters/08-addressing-modes.tex
- lines: 186-197
- severity: major
- category: fact
- claim: "The effective address is computed by adding a 16-bit signed immediate offset to an address register pair low word." ... "STOR (#-4, s), r2     ; Store to address (s - 4)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:77-83 and 110-115 (the effective address is an unsigned 16-bit add and the SegmentOverflow trap fires on unsigned wrap when SR.A is set, so a negative displacement traps whenever `A.low >= |displacement|`); tests only exercise positive displacements (sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:640-693); facts.md Open question 8
- resolution: unclear
- fix: State in this section whether negative displacements are architecturally usable while SR.A (TrapOnAddressOverflow) is set. Both sides: the encoding and the wrapping add make `(#-4, s)` behave as `s - 4`, but the implementation's overflow check is unsigned, so with SR.A set the same instruction raises SegmentOverflow. Either document the trap as intended (and mark negative displacements as unusable with the trap enabled) or record the check as a code defect.
- confidence: high

### F-enc-12
- file: chapters/08-addressing-modes.tex
- lines: 376
- severity: major
- category: fact
- claim: "loop:\n  LOAD r1, (#0, a)+   ; Load element, advance by 1"
- evidence: sirc-vm/toolchain/src/parsers/shared.rs:113-115 (`parse_label_` = `preceded(char(':'), cut(parse_label_name_))`, so a label definition is `:name`, not `name:`); the chapter's own stack-frame listing uses the correct form at chapters/08-addressing-modes.tex:386
- resolution: manual-wrong
- fix: Change `loop:` to `:loop` (label definitions take a leading colon; references take `@loop`). Check other chapters for the trailing-colon form.
- confidence: high

### F-enc-13
- file: chapters/08-addressing-modes.tex
- lines: 66
- severity: major
- category: fact
- claim: "\texttt{LJMP src[, \#offset|rO]}; \texttt{LJSR src[, \#offset|rO]}; \texttt{LJSR (\#offset, src)+}."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/ljsr.rs:210-247 (`IndirectRegisterDisplacementPostIncrement` is accepted and lowers to `LoadEffectiveAddressAndLinkFromIndirectRegisterPostIncrement`, opcode 0x1F); docs/reference/manual-handover.md:29-31 lists `LJSR (r3, a)+` among the verified forms
- resolution: manual-wrong
- fix: Add `\texttt{LJSR (rO, src)+}` to the legal operand forms for the control-flow meta-instruction row (opcode 0x1F).
- confidence: high

### F-enc-14
- file: chapters/08-addressing-modes.tex
- lines: 247-249
- severity: minor
- category: fact
- claim: "The effective address is computed using indirect register addressing, then the address register pair is incremented after the memory operation."
- evidence: chapters/08-addressing-modes.tex:257 (`LOAD r1, (#2, s)+`, an immediate-displacement post-increment); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:138-139 (opcodes 0x16 immediate and 0x17 register both post-increment)
- resolution: manual-wrong
- fix: "The effective address is computed using indirect immediate or indirect register addressing, then the low word of the address register pair is incremented by one after the memory operation." Apply the same correction to the Pre-Decrement description at line 280 ("The address register pair is decremented..." should say the low word only).
- confidence: high

### F-enc-15
- file: chapters/08-addressing-modes.tex
- lines: 411-414
- severity: minor
- category: fact
- claim: "\item \textbf{Post-increment/Pre-decrement}: Allowed in protected mode only when the high word would remain unchanged"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:81-105 (the auto-update adds or subtracts 1 from the low word only; the high word is never part of the update) and stages/write_back.rs:74-98
- resolution: manual-wrong
- fix: "\item \textbf{Post-increment/Pre-decrement}: Always allowed in protected mode; the auto-update writes only the low word of the pair and never carries into the high word." The condition as written implies the auto-update can change the high word, which it cannot.
- confidence: high

### F-enc-16
- file: chapters/09-shift-operations.tex
- lines: 264-268
- severity: blocker
- category: contradiction
- claim: "ADDI r1, #1, LSL #2  ; r1 = r1 + (r2 << 2) + 1 ; SO=0: r2 is shifted by literal 2"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:189-192 (short immediate: operand `a` = shift(destination register), operand `b` = the zero-extended immediate; no second source register exists); contradicts this chapter's own table at chapters/09-shift-operations.tex:60
- resolution: manual-wrong
- fix: "ADDI r1, #1, LSL #2  ; r1 = (r1 << 2) + 1 ; SO=0: r1 is shifted by literal 2". There is no `r2` operand in the short-immediate format.
- confidence: high

### F-enc-17
- file: chapters/09-shift-operations.tex
- lines: 314
- severity: blocker
- category: fact
- claim: "\item If shift count is 0, carry flag is not modified"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:370-390 (with `clamped_b = 0` the computed carry is `false`) and stages/write_back.rs:170-177 (with AF = Shift the whole low byte of the SR is replaced by the shift status); sirc-vm/peripheral-cpu/tests/instructions/arithmetic_register_test.rs:853-871 (C, N, V, Z set before the instruction; only N set afterwards) and :924-936 (shift count 0 yields no carry)
- resolution: manual-wrong
- fix: "\item A shift count of 0 leaves the operand unchanged and clears the carry flag; when the status register is updated from the shifter, all four condition flags are written, so no flag survives a shift-sourced update." (If the flags must be preserved, use the `[N]` status override.)
- confidence: high

### F-enc-18
- file: chapters/09-shift-operations.tex
- lines: 161-166
- severity: blocker
- category: fact
- claim: "\item Bit 15 (sign bit) is copied into bit 15 and bit 14 \item Equivalent to signed division by $2^n$ (rounds toward negative infinity)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:446-476 (`wide_result = extended_a.rotate_right(clamped_b) | sign_bit` — only bit 15 is forced; the vacated bits 14..(15-n) are filled from the rotated-out low bits, not from the sign); sirc-vm/peripheral-cpu/tests/instructions/arithmetic_register_test.rs:885-897 (`0b1100_1100_1100_1101` ASR 6 gives `0b1000_0011_0011_0011`, whereas a true arithmetic shift gives `0b1111_1111_0011_0011`)
- resolution: unclear
- fix: Either correct the implementation to sign-fill all vacated bits, or replace these bullets with the observed rule: "\mnemonic{ASR} shifts right and forces bit 15 to the original sign bit; bits 14 down to 16-n are filled with the bits rotated out of the low end, so \mnemonic{ASR} is equivalent to signed division by $2^n$ only for a shift count of 1." Both sides: the manual (and the mnemonic) describe a true arithmetic shift; the implementation and its test assert the rotate-plus-sign-bit result. The same claim must be fixed at chapters/09-shift-operations.tex:171 ("r1 / 8 (signed)"), :175, :356 and the sign-extension idiom at :367-370, which does not sign-extend under the current implementation.
- confidence: high

### F-enc-19
- file: chapters/09-shift-operations.tex
- lines: 43-44
- severity: major
- category: fact
- claim: "A shift postamble can come after any short immediate or register instruction."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/load.rs:112-144 (register-copy `LOAD rD, rS` rejects a shift definition and directs the user to `SHFT`); sirc-vm/toolchain/src/parsers/opcodes/ldea.rs and ldel.rs (register forms are built with `ShiftType::None` and reject a shift operand); chapters/08-addressing-modes.tex:64-65 ("No shift suffixes" for LDEA and LDEL)
- resolution: manual-wrong
- fix: "A shift postamble may follow the ALU short-immediate and ALU register forms, the \mnemonic{SHFT} meta-instruction, and the register-displacement \mnemonic{LOAD}/\mnemonic{STOR} forms. It is rejected on \mnemonic{LOAD rD, rS}, on \mnemonic{LDEA}/\mnemonic{LDEL} and on coprocessor instructions."
- confidence: high

### F-enc-20
- file: chapters/09-shift-operations.tex
- lines: 43-48, 250-254
- severity: major
- category: fact
- claim: "It is applied in the \"Decode and Register Fetch\" phase of instruction execution, before any ALU operations or memory address calculation occurs." ... "0 & Shift by Immediate & R2 (first source operand) is shifted by a literal amount"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:310-316 (for memory loads the shift is applied to the word returned by the bus at write-back, and the status register is never updated); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:179-193 (short-immediate format shifts the destination register, which is the Reg field, not an R2 field)
- resolution: manual-wrong
- fix: Split the rule by format: in register-format ALU instructions the shifted operand is R2 (first source); in short-immediate ALU instructions it is the register named by the Reg field (destination = first source); in register-displacement \mnemonic{LOAD} forms the shift is applied to the loaded memory word at write-back (and never updates flags); in register-displacement \mnemonic{STOR} forms it is applied to the stored data. Do not describe the shift as always occurring in the decode phase.
- confidence: high

### F-enc-21
- file: chapters/09-shift-operations.tex
- lines: 286-294
- severity: major
- category: fact
- claim: "The shift count is a 4-bit field, allowing shifts of 0--15 positions. \item Maximum shift count is 15 \item Shift counts $\geq$ 16 would be meaningless for 16-bit values"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:85-89 and stages/fetch_and_decode.rs:60-69 (with SO = 1 the count is the full 16-bit value of the named register, not a 4-bit field); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:372, 395, 425, 457 (LSL/LSR/ASL/ASR clamp the count to 16) versus :479, 495 (rotates take the count modulo 16); no test covers register counts above 15 (sirc-vm/peripheral-cpu/tests/instructions/arithmetic_register_test.rs:953-959); facts.md Open question 13
- resolution: unclear
- fix: Restrict "4-bit field, maximum 15" to the SO = 0 (immediate count) case — the assembler enforces it (sirc-vm/toolchain/src/parsers/instruction.rs:225-231) and the encoder masks the field to 4 bits — and add a normative statement for SO = 1: say what a register count greater than 15 does. Both sides: the manual says counts above 15 are impossible; the implementation accepts a 16-bit register count and gives shifts a zero (or sign-only) result while rotates wrap modulo 16. The same claim appears at chapters/09-shift-operations.tex:439-440.
- confidence: high

### F-enc-22
- file: chapters/09-shift-operations.tex
- lines: 524
- severity: blocker
- category: contradiction
- claim: "LOAD[N] r4, (#0, a)+          ; Load next value (flags unchanged)"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/load.rs:29-37 (an explicit status-register update source on `LOAD` is a hard parse failure: "The [LOAD] opcode does not support an explicit status register update source"); chapters/08-addressing-modes.tex:67 already states that coprocessor and non-ALU forms take no status-update override
- resolution: manual-wrong
- fix: Replace with `LOAD r4, (#0, a)+          ; Load next value (LOAD never updates flags)` and add a sentence that the `[A|S|N]` modifier is accepted only on ALU mnemonics, because `LOAD`, `STOR`, `LDEA`, `LDEL` and the coprocessor mnemonics never update the status register.
- confidence: high

### F-enc-23
- file: chapters/09-shift-operations.tex
- lines: 95-96
- severity: major
- category: fact
- claim: "; Set bit 10\nORRI r3, #1, LSL #10     ; r3 |= (r3 << 10) with immediate 1 = 0x0400"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:189-192 (short immediate computes `shift(D) op imm8`, so the result is `(r3 << 10) | 1`); the shift never applies to the immediate (facts.md Open question 16)
- resolution: manual-wrong
- fix: Delete this example or replace it with one that matches the semantics, for example `ORRI r3, #1, LSL #10  ; r3 = (r3 << 10) | 1`. There is no single instruction that ORs a shifted immediate into a register, so the "Set bit 10" comment must go.
- confidence: high

### F-enc-24
- file: chapters/09-shift-operations.tex
- lines: 84-88, 114-119
- severity: minor
- category: fact
- claim: "\item Bit 15 is shifted into the carry flag" (LSL) / "\item Bit 0 is shifted into the carry flag" (LSR)
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:370-414 (the carry is the last bit shifted out, which is bit 16-n for LSL and bit n-1 for LSR); the chapter states the general rule correctly at chapters/09-shift-operations.tex:312
- resolution: manual-wrong
- fix: "\item The last bit shifted out of bit 15 (bit 16-n for a count of n) is placed in the carry flag" and, for LSR, "\item The last bit shifted out of bit 0 (bit n-1 for a count of n) is placed in the carry flag". Same wording applies to the ASR bullet at line 162.
- confidence: high
