# Triaged findings

Sources: facts-alu.md, facts-arch.md, facts-cop.md, facts-data.md, facts-enc.md, facts-exc.md, facts-mem.md, facts-sum.md, facts-tim.md, consistency.md + gate1-decisions.md. 312 raw, 306 after dedup.

## Merged duplicates
- F-exc-2 absorbs F-con-3
- F-exc-8 absorbs F-con-25
- F-enc-22 absorbs F-con-17
- F-cop-24 absorbs F-con-65
- F-sum-24 absorbs F-con-67
- F-exc-6 absorbs F-con-6

## Counts
- accepted blocker: 43
- accepted major: 125
- accepted minor: 115
- accepted nit: 6
- code-wrong blocker: 5
- code-wrong major: 4
- code-wrong minor: 1
- defer minor: 2
- not-implemented major: 2
- phase4 major: 1
- phase4 minor: 1

## Gate 2 rulings (binding on every editor)

Read these before applying any finding. Where a finding's `fix` text conflicts with a ruling,
the ruling wins.

- **A. Fault return address.** **Revised 2026-09-08** (after this ruling was already applied):
  the original ruling below was wrong and contradicted the CPU's own design comment
  (`exception_unit/definitions.rs:121-131`), which explicitly lists Bus Fault, Alignment Fault,
  Privilege Violation, and Invalid Opcode Fault as intended to be retryable. The author's final
  call: **Alignment, Bus, Bus Protection, Privilege Violation, and PC-wrap Segment Overflow are
  retryable** (link register holds the faulting instruction's address). **Invalid Opcode Fault and
  non-PC-wrap Segment Overflow are not retryable** (link register holds the next instruction's
  address) -- Invalid Opcode deliberately keeps the manual's existing "emulate then resume after
  the trap" pattern rather than the design comment's original intent. See
  `implementation-bugs.md` for the code-wrong routing; the original (superseded) ruling read:
  "Only Alignment and PC-wrap Segment Overflow save the faulting instruction's address and are
  retryable. Bus, Bus Protection, Invalid Opcode and Privilege Violation save the next
  instruction's address; the aborted instruction is not restartable."
- **B. Unsigned condition codes.** The ARM-style glosses stand (HI = unsigned higher, LO =
  unsigned lower or same, CS = unsigned >=). The implementation's predicates will be fixed.
  Do not invert any gloss, truth table, or example on the grounds that C is set on borrow.
  **Revised 2026-09-08** (after this ruling was already applied by every Phase 3/4 editor):
  `CS = unsigned >=` above was itself wrong — `CS` actually means unsigned lower on this
  CPU (the opposite of ARM), and `CC` means unsigned higher-or-same. `HI`/`LO` keep the
  glosses stated above; only two lines of `definitions.rs` need to change (flip the
  `Carry` term only) to make the hardware deliver them. See `implementation-bugs.md`,
  F-data-1, for the exact fix and the manual sites corrected on 2026-09-08 (chapters 7
  and 10).
- **C. LOAD flags.** LOAD never updates flags (`- - - -`, default AF `[N]`). Any explicit AF
  suffix on a LOAD leaves all four flags undefined (`U U U U`).
- **D. Displacements.** 16-bit displacements are signed two's-complement. The address-overflow
  trap fires only when the low-word calculation wraps. (The emulator's unsigned-carry test is
  a bug; do not document it.)
- **E. Shift counts.** Immediate counts are 0--15. A register count above 15 is architecturally
  undefined.
- **F. ASR** sign-fills every vacated bit (emulator bug; manual stands).
- **G. LOAD shift** applies to the offset register (source), never to the loaded data.
- **H. Opcodes 0x27 and 0x2F** are Undocumented, same class as 0x08/0x09/0x0B/0x0D. Appendix C
  keeps its count of 14.
- **I. NOOP** is unprivileged and assembles to `ADDI[N] sr, #0` (word 0x00000000). Do not
  mention a protected-mode fault.
- **J. EXCP** with a vector below 0x60 raises a privilege-violation fault in protected mode
  (emulator to be fixed).
- **K/L. Undefined.** Exception-unit opcodes 0x2--0x8, software-issued 0xE/0xF, RETE at
  exception level 0, and two fault conditions in one instruction are architecturally undefined.
  One sentence each; never describe a panic.
- **M. DMA unit** is Required (coprocessor 0x2); the reference emulator has not implemented it
  yet, which the manual may note once. DMA timing is restated on the 6-cycle-slot model. The
  DMA register-field encoding stays, with an explicit warning that it is not the AF field.
- **N. Addressing modes.** The chapter 8 editor derives the count and list from the ten
  assembler operand forms and records the result in its changes file. Chapters 1 and 11 are
  aligned in Phase 4; other editors must not change the count.
- **O. Shifts apply to the register operand:** `ADDI r1, #2, LSL #3` is `(r1 << 3) + 2`.
- **P. Vector table** has 256 entries: 0x00--0x5F reserved (96), 0x60--0xFF user (160).
- **Reserved SR bits** keep the advisory wording (STYLE.md).
- **Template fields (G1-T1..T5)** from Gate 1 apply to every instruction entry; see STYLE.md
  "Approved decisions".

## Findings

### F-arch-1
- file: chapters/02-cpu-architecture.tex
- lines: 17-36
- severity: blocker
- category: fact
- claim: "\item \textbf{Instruction Fetch (High Word)} -- 1 cycle \begin{itemize} \item Fetch bits 31--16 from memory address in PC \item Increment PC by 1"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:107-117 (the second fetch addresses `get_full_pc_address() + 1` without modifying PC); .../execution.rs:151 (`registers.pl = self.decoded_instruction.npc_l_` at the decode phase); .../stages/fetch_and_decode.rs:203-205 (`npc_l_ = pl + INSTRUCTION_SIZE_WORDS`, `npc_h_ = ph`); sirc-vm/peripheral-cpu/tests/instructions/ljmp_test.rs:225-232 (branch base is the branching instruction's own address, which is only true if `pl` has not yet advanced during fetch)
- resolution: manual-wrong
- fix: The PC is not incremented during either fetch phase; it advances by 2 once, during Decode, and only `pl` changes (`ph` is never incremented, so a wrap stays inside the segment). Replace the two "Increment PC by 1" bullets with "PC is unchanged; the low word is fetched from PC + 1", and add to the Decode phase bullet list "Advance \reg{pl} by 2 to the address of the next instruction (\reg{ph} is unchanged)". The same wrong model appears in chapters/03-registers.tex:171-173 ("It is automatically incremented by 1 after each 16-bit word fetch. Since instructions are 32 bits (two words), the PC is incremented twice during a complete instruction fetch") and must be replaced there too; this matters because branch and \mnemonic{LDEL} link values are defined relative to the branching instruction's own address, which the two-increment model would shift by 2.
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

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
- status: accepted

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
- status: accepted

### F-exc-1
- file: chapters/06-exceptions.tex
- lines: 528
- severity: blocker
- category: fact
- claim: "\textbf{0x60-0xFF:} User exception vectors (128 user-accessible trap vectors)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs:172-173 (`USER_EXCEPTION_VECTOR_START = 0x60`, `USER_EXCEPTION_VECTOR_END = 0xFF`); facts.md:274
- resolution: manual-wrong
- fix: Replace "(128 user-accessible trap vectors)" with "(160 user-accessible trap vectors)". 0xFF - 0x60 + 1 = 160. Note that the code comments at `exception_unit/definitions.rs:65-68` ("128 ... vector addresses ... first 48 ... remaining 80") and `:171` ("128 user exception vectors") are also wrong and are the likely source of this number; they are comments only and carry no behaviour. Same count is repeated nowhere else in Chapter 6, but check `appendix-e-quick-reference.tex` for a copy.
- confidence: high
- status: accepted

### F-exc-2
- file: chapters/06-exceptions.tex
- lines: 259-268
- severity: blocker
- category: contradiction
- claim: "\bitbox{3}{BAT} & \bitbox{1}{D} & \bitbox{4}{Current Fault} & \bitbox{4}{Original Fault} & \bitbox{4}{Reserved}"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:80-88 (`BUS_ACCESS_TYPE_MASK = 0x7`, `DOUBLE_FAULT_FLAG_MASK = 0x8`, `CURRENT_FAULT_MASK = 0xF0`, `PREVIOUS_FAULT_MASK = 0xF00`); chapters/06-exceptions.tex:271-286 (the itemised list, which is correct); chapters/05-status-register.tex:23-26 (house bytefield convention: highest-numbered bit drawn leftmost)
- resolution: manual-wrong
- fix: Reverse the field order in the figure so it reads left (MSB) to right (LSB): `\bitbox{4}{Reserved} & \bitbox{4}{Original Fault} & \bitbox{4}{Current Fault} & \bitbox{1}{D} & \bitbox{3}{BAT}`. As drawn, the figure places BAT at bits 15--13 and Reserved at bits 3--0, the exact opposite of the normative text immediately below it and of the masks in `lib.rs`.
- confidence: high
- status: accepted

### F-exc-3
- file: chapters/06-exceptions.tex
- lines: 33-38, 544, 545
- severity: blocker
- category: fact
- claim: "[Retryable faults] The instruction that triggered the fault is cancelled before it takes effect. The link register stores the address of the faulting instruction." ... "\textit{Bus Fault, Bus Protection Fault, Alignment Fault, Segment Overflow Fault, Invalid Opcode Fault, Privilege Violation Fault.}"
- evidence: sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:939-942 asserts `link_registers[6].return_address == 0x0000_0002` for an invalid-opcode fault raised by the COP instruction at 0x0000_0000 (and :989-992 asserts 0x00AB_CDE4 for the second one); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:141-151 advances `registers.pl = npc_l_` unconditionally, after the decode-time privilege check has already raised the fault. Against this: definitions.rs:120-125 and :19-20 state that abort exceptions save the faulting instruction's address and that "pl is not incremented for a bus fault". facts.md Open question 1.
- resolution: manual-wrong
- fix: The manual and the implementation disagree. Either (a) the manual is wrong and the Retryable category must be reduced to Alignment Fault and PC-wrap Segment Overflow only, with Bus, Bus Protection, Invalid Opcode and Privilege Violation moved to a category whose return address is the *next* instruction; or (b) the implementation is wrong and must suppress the `pl` advance when a fault is raised in the same instruction. Author must rule. Until then do not assert "so it can be retried" for the four faults with no test coverage (Bus, Bus Protection, Privilege Violation, Segment Overflow) and correct the Invalid Opcode Fault row of Table~\ref{tab:exception-quick-reference} (line 544) and the Privilege Violation row (line 545) to match whichever side wins. Also note that Alignment, the one fault that genuinely does preserve the faulting address, preserves an *odd* `pl`, so a bare RETE re-faults forever; the manual should say so.
- confidence: high
- status: accepted
- ruling: Gate 2 A: document actual behaviour. Retryable class is Alignment and PC-wrap Segment Overflow only; Bus, Bus Protection, Invalid Opcode and Privilege Violation save the next instruction address and are not restartable.

### F-con-1
- file: chapters/07-instruction-formats.tex
- lines: 277-280
- severity: blocker
- category: contradiction
- claim: "1100          & <                 & Signed Less Than (N $\neq$ V)              \\ ... 1110          & <<                & Signed Less or Equal (Z = 1 OR N $\neq$ V)"
- evidence: chapters/10-condition-codes.tex:33-35 gives `1100 = <<` (Signed Less Than) and `1110 = <=` (Signed Less or Equal); every use site in the manual follows Chapter 10, not Chapter 7 (chapters/10-condition-codes.tex:154 `BRAN|<<`, chapters/13-alu-instructions.tex:846 `BRAN|<<`, chapters/15-control-flow.tex:525,540 `BRAN|<=`, chapters/appendix-b-timing.tex:136 `BRAN|<=`). The mnemonic `<` is never used anywhere else in the manual (0 hits).
- resolution: manual-wrong
- fix: In `tab:condition-code-encoding` (07-instruction-formats.tex) change the `1100` row mnemonic from `<` to `<<` and the `1110` row mnemonic from `<<` to `<=`, matching `tab:condition-codes` in Chapter 10. Also reconcile row `0000`: Chapter 7 says `(none)`, Chapter 10 says `AL (or none)` — pick `AL (or none)`. Note that appendix-e-quick-reference.tex:185 sends readers to the Chapter 7 table as the authoritative one, so the two tables must either be merged or Chapter 7's must be corrected.
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: accepted

### F-con-5
- file: chapters/09-shift-operations.tex
- lines: 328-330
- severity: blocker
- category: contradiction
- claim: "For \textbf{ASL}: Set if the sign bit changes during the shift (signed overflow) \\ For other shifts: Cleared to 0"
- evidence: chapters/09-shift-operations.tex:30 "011 & ASL & Arithmetic Shift Left & Same as LSL (shifts in zeros)" and chapters/09-shift-operations.tex:132 "\textbf{Operation:} Identical to LSL. Shift bits to the left, filling with zeros." If ASL is identical to LSL it cannot have a different V effect. chapters/14-memory-instructions.tex:351 repeats "ASL -- Arithmetic Shift Left (same as LSL)".
- resolution: manual-wrong
- fix: ASL and LSL differ only in flag effect. Change chapters/09-shift-operations.tex:30 to "Same data result as LSL; sets V on signed overflow" and chapters/09-shift-operations.tex:132 to "Produces the same data result as LSL, but sets the overflow flag if the sign bit changes." Apply the same correction at chapters/14-memory-instructions.tex:351.
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted

### F-enc-18
- file: chapters/09-shift-operations.tex
- lines: 161-166
- severity: blocker
- category: fact
- claim: "\item Bit 15 (sign bit) is copied into bit 15 and bit 14 \item Equivalent to signed division by $2^n$ (rounds toward negative infinity)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:446-476 (`wide_result = extended_a.rotate_right(clamped_b) | sign_bit` — only bit 15 is forced; the vacated bits 14..(15-n) are filled from the rotated-out low bits, not from the sign); sirc-vm/peripheral-cpu/tests/instructions/arithmetic_register_test.rs:885-897 (`0b1100_1100_1100_1101` ASR 6 gives `0b1000_0011_0011_0011`, whereas a true arithmetic shift gives `0b1111_1111_0011_0011`)
- resolution: code-wrong
- fix: Either correct the implementation to sign-fill all vacated bits, or replace these bullets with the observed rule: "\mnemonic{ASR} shifts right and forces bit 15 to the original sign bit; bits 14 down to 16-n are filled with the bits rotated out of the low end, so \mnemonic{ASR} is equivalent to signed division by $2^n$ only for a shift count of 1." Both sides: the manual (and the mnemonic) describe a true arithmetic shift; the implementation and its test assert the rotate-plus-sign-bit result. The same claim must be fixed at chapters/09-shift-operations.tex:171 ("r1 / 8 (signed)"), :175, :356 and the sign-extension idiom at :367-370, which does not sign-extend under the current implementation.
- confidence: high
- status: code-wrong
- ruling: Gate 2 F: ASR must sign-fill every vacated bit. Manual text stands.

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
- status: accepted

### F-data-1
- file: chapters/10-condition-codes.tex
- lines: 74-86
- severity: blocker
- category: fact
- claim: "HI (Unsigned Higher) Executes when C = 1 AND Z = 0. Tests if first operand $>$ second operand (unsigned)."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:195-206 (perform_subtract sets C from a.overflowing_sub(b), i.e. C = borrow = a < b unsigned); sirc-vm/peripheral-cpu/tests/instructions/arithmetic_immediate_test.rs:260-272 (SUBI 0x5FFF - 0xFFFF sets Carry); definitions.rs:128-136 (UnsignedHigher = C AND NOT Z, UnsignedLowerOrSame = NOT C OR Z); docs/reference/manual-handover.md:19 "Resolved: subtraction sets C when a borrow occurs"; docs/reference/chapters/13-alu-instructions.tex:147 "C is set when a borrow occurs."
- resolution: code-wrong
- fix: With C set on borrow, after CMP a, b the true mapping is CS = HI = unsigned lower (a < b) and CC = LO = unsigned higher or same (a >= b); there is no single condition code for strict unsigned greater-than. Either the chapter must be rewritten to state this (and the mnemonic glosses HI/LO relabelled), or definitions.rs:128-136 must invert the two predicates. Both sides: the code plus tests plus handover fix C = borrow; the DEF comment at definitions.rs:114 says the predicate set was copied from ARM, which defines C after subtraction as NOT borrow, so the ARM-derived HI/LO predicates were never re-derived for this convention. Same error occurs at lines 30-31 (table rows for HI and LO), 84-85 (CS/CC equivalences), 138 and 142 (unsigned examples), 181-184 (Unsigned Comparison Results table), 219-223 (conditional assignment example), 249 (loop example) and 263-266 (selection guide rows HI, CS, CC, LO), and in 05-status-register.tex:69-71. Fix once, then propagate.
- confidence: high
- status: code-wrong
- ruling: Gate 2 B: the HI/LO predicates in definitions.rs are to be inverted so the ARM-style glosses stand. Manual text unchanged.

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
- status: accepted

### F-sum-1
- file: chapters/12-instruction-summary.tex
- lines: 29-31
- severity: blocker
- category: fact
- claim: "0x04 & ANDI & Immediate & NZ & AND immediate" (same NZ-only flag cell for ORRI, XORI, TSAI, TSXI and their short-immediate and register forms)
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:270-305 (perform_and/or/xor call set_alu_bits with carry=false and no overflow triple); alu.rs:571-590 (C and V are written unconditionally); facts.md flag table rows AND/OR/XOR; chapters/13-alu-instructions.tex:537,604,674 already says "N Z (C and V cleared)"
- resolution: manual-wrong
- fix: Change the Flags cell for 0x04, 0x05, 0x06, 0x0C, 0x0E, 0x24, 0x25, 0x26, 0x2C, 0x2E, 0x34, 0x35, 0x36, 0x3C, 0x3E from "NZ" to "NZ (C,V=0)" so the column matches Chapter 13. Same defect in chapters/appendix-e-quick-reference.tex lines 28, 29, 68, 69, 85-88, 94, 95, whose header defines Flags as "flags written by the instruction".
- confidence: high
- status: accepted

### F-sum-2
- file: chapters/12-instruction-summary.tex
- lines: 137
- severity: blocker
- category: fact
- claim: "\mnemonic{NOOP} & \texttt{ADDI[N] r1, \#0} & No operation"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/meta.rs:82-95 emits AddImmediate with `register: 0x0`; register index 0 is `sr` (sirc-vm/peripheral-cpu/src/registers.rs:531, facts.md register table); the source comment "add zero to register 1" is itself wrong
- resolution: manual-wrong
- fix: Replace with "ADDI[N] sr, #0" and add a Note that because the destination field is 0 (sr, a privileged destination, PU:21-27) the architectural behaviour of NOOP in protected mode is unresolved (facts.md Open question 2). Same defect at chapters/appendix-e-quick-reference.tex:65.
- confidence: high
- status: accepted
- ruling: Apply the encoding correction (`ADDI[N] sr, #0`) only. Do NOT add a note about protected-mode behaviour: per Gate 2 I, NOOP is unprivileged and the emulator will be fixed.

### F-sum-5
- file: chapters/12-instruction-summary.tex
- lines: 116-118
- severity: blocker
- category: fact
- claim: "\textbf{+0x08 to +0x0F}: Test only (update flags, discard result)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:132-151 maps alu_code 0x8-0xE to AluStatusOnly and 0xF to CoprocessorCall; execution_effective_address.rs:61-67; opcodes 0x0F/0x2F/0x3F are COPI/COPR and update no flags at all
- resolution: manual-wrong
- fix: Change to "+0x08 to +0x0E: Test only (update flags, discard result)" and add "+0x0F: Coprocessor call (neither result nor flags are written)". Identical defect at chapters/appendix-a-opcode-map.tex:110, whose bullet "Bit 3 = 1: Test only, discard result (0x08--0x0F, 0x28--0x2F, 0x38--0x3F)" must exclude 0x0F, 0x2F and 0x3F the same way.
- confidence: high
- status: accepted

### F-alu-1
- file: chapters/13-alu-instructions.tex
- lines: 123-125
- severity: blocker
- category: fact
- claim: "ALU instruction execution does not perform data-memory access and does not raise data bus, data bus-protection, data alignment, segment-overflow, privilege-violation, or invalid-opcode faults."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:21-27 (`PRIVILEGED_REGISTERS` = sr, ah, lh, ph, sh), :29-51 (`check_privilege` fires for any non-COP instruction whose destination field names one of them), :139-144 (`raise_fault(..., Faults::PrivilegeViolation, ...)` at decode); sirc-vm/peripheral-cpu/tests/instructions/protected_mode_test.rs:94-132 (`LoadRegisterFromImmediate`/`LoadRegisterFromRegister` into `sr` and `ph` in protected mode both `assert_privilege_fault`); sirc-vm/toolchain/src/parsers/instruction.rs:88 (`sr`, `lh`..`pl` are ordinary `DirectRegister` operands, so `ADDI ph, #1` assembles)
- resolution: manual-wrong
- fix: Replace the clause with: "ALU instruction execution does not perform data-memory access and does not raise data bus, data bus-protection, data alignment, segment-overflow, or invalid-opcode faults. In protected mode, an ALU instruction whose register field names \reg{sr}, \reg{ah}, \reg{lh}, \reg{ph}, or \reg{sh} raises a privilege-violation fault at decode and has no other effect." The same correction is needed in the `Privilege:` line of every entry in this chapter (lines 225, 290, 358, 420, 485, 547, 614, 684, 751, 828, 905, 979 all say "Available in protected mode and supervisor mode"), which should read "Available in protected mode and supervisor mode, except when the register field names a privileged register (\reg{sr}, \reg{ah}, \reg{lh}, \reg{ph}, \reg{sh}); see Chapter~\ref{ch:exceptions}", and in the `Exceptions` row of Table~\ref{tab:alu-common-semantics} (line 116).
- confidence: high
- status: accepted

### F-alu-3
- file: chapters/13-alu-instructions.tex
- lines: 725-726
- severity: blocker
- category: contradiction
- claim: "No short-immediate, memory, or shift forms are public assembly syntax."
- evidence: docs/reference/chapters/14-memory-instructions.tex:52-55 and :130-139 document `LOAD rD, (#offset, addr)` (0x14), `LOAD rD, (rO, addr)[, shift]` (0x15), `LOAD rD, (#offset, addr)+` (0x16), `LOAD rD, (rO, addr)+[, shift]` (0x17) as public syntax; sirc-vm/toolchain/src/parsers/opcodes/load.rs:150-200 parses those forms; only the register-to-register shift form is rejected (load.rs:134-144)
- resolution: manual-wrong
- fix: Replace with: "No short-immediate form is public assembly syntax, and the register-to-register form does not accept a shift definition (use \mnemonic{SHFT}). The indirect memory forms of \mnemonic{LOAD} are documented in Chapter~\ref{ch:memory-instructions}."
- confidence: high
- status: accepted

### F-alu-4
- file: chapters/13-alu-instructions.tex
- lines: 857-858
- severity: blocker
- category: fact
- claim: "\item For signed comparisons, use \texttt{>=}, \texttt{<}, \texttt{>>}, \texttt{<<} \item For unsigned comparisons, use \texttt{>}, \texttt{CS}, \texttt{CC}, \texttt{<=}"
- evidence: sirc-vm/toolchain/src/parsers/instruction.rs:317-356 — the complete condition-code tag set is `AL == != CS CC NS NC OS OC HI LO >= << >> <= NV`; there is no `<` and no `>`; `<=` is `LessThanOrEqual` (`Z || N != V`, signed) per sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:115-157; the unsigned pair is `HI`/`LO`
- resolution: manual-wrong
- fix: Replace the two items with: "\item For signed comparisons, use \texttt{>=}, \texttt{<<}, \texttt{>>}, \texttt{<=} \item For unsigned comparisons, use \texttt{HI}, \texttt{LO}, \texttt{CS}, \texttt{CC}"
- confidence: high
- status: accepted
- ruling: Per Gate 2 B the ARM-style glosses stand: HI = unsigned higher, LO = unsigned lower or same. Do not invert.

### F-con-2
- file: chapters/13-alu-instructions.tex
- lines: 857-858
- severity: blocker
- category: contradiction
- claim: "For signed comparisons, use \texttt{>=}, \texttt{<}, \texttt{>>}, \texttt{<<} \\ For unsigned comparisons, use \texttt{>}, \texttt{CS}, \texttt{CC}, \texttt{<=}"
- evidence: chapters/10-condition-codes.tex:30-35 and chapters/10-condition-codes.tex:263-270 — the unsigned conditions are `HI`, `LO`, `CS`, `CC`; `<=` and `>>` are *signed* conditions; `>` is not a condition mnemonic in any table.
- resolution: manual-wrong
- fix: Replace both lines with "For signed comparisons, use \texttt{>=}, \texttt{<<}, \texttt{>>}, \texttt{<=}" and "For unsigned comparisons, use \texttt{HI}, \texttt{LO}, \texttt{CS}, \texttt{CC}".
- confidence: high
- status: accepted
- ruling: Per Gate 2 B the ARM-style glosses stand: HI = unsigned higher. Do not invert.

### F-con-4
- file: chapters/14-memory-instructions.tex
- lines: 325-326
- severity: blocker
- category: contradiction
- claim: "When loading from memory, the data read from memory is shifted \textbf{after} being loaded but \textbf{before} being written to the destination register"
- evidence: chapters/09-shift-operations.tex:12-13 "Shift operations are only applied to source operands. A shift cannot be applied to the result of an operation before register writeback."; chapters/09-shift-operations.tex:43-44 "It is applied in the \"Decode and Register Fetch\" phase of instruction execution, before any ALU operations or memory address calculation occurs."; chapters/02-cpu-architecture.tex:136-137 "The shift is applied to the first source operand when fetching the source registers for an instruction. Shifts cannot be applied to the result or any other source operand." Also chapters/14-memory-instructions.tex:23, :53, :55, :144, :151, :156. The generated encodings confirm the conflict: `LOAD r1, (r2, a)` encodes R2=0x0 (generated/register-format-encodings.tex:14), so there is no first source operand for the Chapter 9 rule to shift, while `STOR -(r2, s), r1` encodes R2=0x1 (the stored source) and is consistent with Chapter 9.
- resolution: manual-wrong
- fix: Decide whether the register-offset LOAD shift is a load-result shift (then Chapter 9 line 12-13, Chapter 9 line 43-44 and Chapter 2 line 136-137 must be amended to carve out memory loads and to state that the load-result shift happens in the Memory Access / Write Back phase), or a source-operand shift (then Chapter 14 lines 23, 53, 55, 144, 151, 156, 325-326 must be rewritten). Whichever way it resolves, one sentence must state the phase in which the memory shift is applied.
- confidence: high
- status: accepted
- ruling: Gate 2 G: the shift applies to the offset register (source operand), never to the loaded data. Correct ch14 to match ch2 and ch9.

### F-con-8
- file: chapters/14-memory-instructions.tex
- lines: 132-139
- severity: blocker
- category: contradiction
- claim: "LOAD rD, (rS, addr)           ; rD = memory[addr + rS]" and (lines 213-220) "STOR (rD, addr), rS           ; memory[addr + rD] = rS"
- evidence: The displacement register is named `rO` in every table: chapters/08-addressing-modes.tex:41 "\texttt{(rO, addr)}", chapters/08-addressing-modes.tex:62-63, chapters/14-memory-instructions.tex:53,55,57,59 ("\texttt{LOAD rD, (rO, addr)[, shift]}"). `rD` is defined at chapters/11-reading-instructions.tex:35 as "Destination general-purpose register" and `rS` at :36 as "Source general-purpose register", so `STOR (rD, addr), rS` names the *displacement* register with the *destination* placeholder while the actual source is `rS`. Chapter 15 has the same drift: chapters/15-control-flow.tex:34,36,38,40,48,51,53,292,295,300,354,357,362,418,458,461,465 all use `rS` for the displacement register, against chapters/08-addressing-modes.tex:66 which uses `rO`.
- resolution: manual-wrong
- fix: Standardize on `rO` (displacement/offset register) in every syntax line and Operands paragraph in Chapters 14 and 15, and add `\texttt{rO}` to the notation table `tab:instruction-description-notation` (chapters/11-reading-instructions.tex:29-47) with the meaning "Register supplying a memory or effective-address displacement". Affected: 14-memory-instructions.tex:132,135,138,139,144,213,216,219,220,225; 15-control-flow.tex:34,36,38,40,48,51,53,292,295,300,354,357,362,418,458,461,465.
- confidence: high
- status: accepted

### F-mem-1
- file: chapters/14-memory-instructions.tex
- lines: 95
- severity: blocker
- category: fact
- claim: "Documented memory forms do not raise privilege-violation or invalid-opcode faults."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:21-27 and :42-43 (`PRIVILEGED_REGISTERS.contains(&instruction.des)` for every non-COP opcode); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:158 (`des` is the register field of *all three* formats, so it is the LOAD destination and the STOR source register)
- resolution: manual-wrong
- fix: Replace with: "A memory instruction raises a privilege-violation fault in protected mode when its register field names a privileged register (\reg{sr}, \reg{ah}, \reg{lh}, \reg{ph}, or \reg{sh}). This applies to the \mnemonic{LOAD} destination register and to the \mnemonic{STOR} source register, even though \mnemonic{STOR} only reads it. Documented memory forms do not raise invalid-opcode faults." The `Privilege:` fields of the LOAD entry (line 180) and the STOR entry (line 260) need the same qualification.
- confidence: high
- status: accepted

### F-mem-5
- file: chapters/14-memory-instructions.tex
- lines: 94-95
- severity: blocker
- category: contradiction
- claim: "Effective-address calculation can raise a segment-overflow fault when \texttt{SR.A} (Trap on Address Overflow) is set and the 16-bit low-word address calculation wraps."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:81-83 and :110-115. For pre-decrement forms `addr_inc` is `-1`, which is added as `0xFFFF`: `displaced.overflowing_add(0xFFFF)` reports overflow for every `displaced != 0` and reports *no* overflow for `displaced == 0`. With SR.A set, `STOR -(#0, s), r1` with `sl = 0x1000` therefore faults although 0x1000-1 does not wrap, while `sl = 0x0000` (the genuine wrap) does not fault. No test covers a pre-decrement with SR.A set (only sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:640-693, a positive-displacement LOAD).
- resolution: code-wrong
- fix: Manual side: the stated rule ("raise only when the calculation wraps") is the sane architectural rule. Code side: the borrow out of the `-1` is being fed into the same overflow term as the carry out of the displacement add, inverting the test for every pre-decrement form. Either the implementation must stop treating the decrement borrow as an overflow, or the manual must say that with SR.A set every pre-decrement memory or effective-address instruction faults unless the computed low word is zero. Do not publish the current wording until this is decided. The same sentence appears at chapters/15-control-flow.tex:103-105 and covers \mnemonic{LDEA} pre-decrement (opcodes 0x1A/0x1B), which uses the same code path.
- confidence: medium
- status: code-wrong
- ruling: Gate 2 D: the address-overflow trap must detect signed wrap of the low word, not unsigned carry. Manual rule stands.

### F-mem-12
- file: chapters/15-control-flow.tex
- lines: 144-145
- severity: blocker
- category: fact
- claim: "\texttt{@label} is resolved by the linker to a PC-relative displacement from the next instruction address."
- evidence: sirc-vm/toolchain/src/bin/linker.rs:86-102 (`full_offset = target_offset_words - program_offset_words`, where `program_offset_words` is the address of the branch instruction itself); sirc-vm/peripheral-cpu/tests/instructions/ljmp_test.rs:224-232 (pl = 0xFAC0, displacement 0x000E lands at 0xFACE, not 0xFACC); execution.rs:151 (pl advances only after the decoded `npc_l_` is captured)
- resolution: manual-wrong
- fix: Replace with "resolved by the linker to a word displacement from the address of the branch instruction itself." The same wrong statement appears at 15-control-flow.tex:194-195 (BRSR). Also add to the BRAN/BRSR Description blocks: the displacement base is the branching instruction's own address, so `BRAN #0` branches to itself and `BRAN #2` falls through to the next instruction, while the \mnemonic{BRSR} link value is that address plus 2 (ljmp_test.rs:292-300).
- confidence: high
- status: accepted

### F-cop-1
- file: chapters/17-meta-instructions.tex
- lines: 44
- severity: blocker
- category: fact
- claim: "\textbf{Assembles to:} \texttt{ADDI[N] r1, \#0}"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/meta.rs:82-95 (`"NOOP" => ... op_code: Instruction::AddImmediate, register: 0x0, value: 0x0, additional_flags: 0x0`); sirc-vm/peripheral-cpu/src/registers.rs:47-48 (`RegisterName { Sr = 0, R1, ... }`); facts.md:157, facts.md:100
- resolution: manual-wrong
- fix: Replace with `\textbf{Assembles to:} \texttt{ADDI[N] sr, \#0}` (register field 0, value 0, AF = None, condition Always; the whole 32-bit word is `0x00000000`). The `r1` claim also changes the encoding: `ADDI[N] r1, #0` would encode as `0x00400000`, not `0x00000000`, which breaks the architectural property that an all-zero instruction word decodes as `NOOP` (sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/encoding.rs:741-755). Same wrong register name appears at lines 48-49 of this file.
- confidence: high
- status: accepted
- ruling: Apply as written. Do not mention any protected-mode fault: per Gate 2 I, NOOP is unprivileged.

### F-cop-2
- file: chapters/17-meta-instructions.tex
- lines: 48-49
- severity: blocker
- category: fact
- claim: "Executes an add-immediate-zero instruction that writes the original value of \reg{r1} back to \reg{r1} and preserves status flags."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/meta.rs:88 (`register: 0x0`); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:55-71 (a supervisor-mode `sr` destination write preserves only bit 13 EA and rewrites the rest)
- resolution: manual-wrong
- fix: Replace with "Executes an add-immediate-zero instruction whose destination is \reg{sr}. In supervisor mode the status register is rewritten with its own value, the \texttt{EA} bit is preserved by the write-back path, and no architectural state changes. Because the status-register update source is \texttt{None}, no condition flags are written."
- confidence: high
- status: accepted

### F-sum-6
- file: chapters/appendix-a-opcode-map.tex
- lines: 126
- severity: blocker
- category: fact
- claim: "Bit 1: Determines if both registers in address pair are updated"
- evidence: opcode bit 1 selects the auto-update forms (0x12/0x13 store pre-decrement, 0x16/0x17 load post-increment, 0x1A/0x1B LDEA pre-decrement, 0x1E/0x1F LDEL post-increment); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:88-105 updates only the low word of the pair by exactly 1 word, never the high word
- resolution: manual-wrong
- fix: Replace with "Bit 1: Auto-update of the source address register (0 = no update, 1 = pre-decrement for stores and LDEA, post-increment for loads and LDEL). Only the low word of the pair is adjusted, and always by one word regardless of the displacement."
- confidence: high
- status: accepted

### F-tim-1
- file: chapters/appendix-b-timing.tex
- lines: 99-102
- severity: blocker
- category: fact
- claim: "ADDR|HI r3, r4, r5            ; 6 cycles (exec if r1 > r2)" / "ADDR|LO r3, r6, r7            ; 6 cycles (exec if r1 <= r2)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:195-206 (`a.overflowing_sub(b)` — C is set on *borrow*, i.e. when `a < b`); sirc-vm/peripheral-cpu/tests/instructions/arithmetic_immediate_test.rs:260-273; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:129-135 (`UnsignedHigher` = C && !Z, `UnsignedLowerOrSame` = !C || Z); docs/reference/manual-handover.md:19 ("subtraction sets C when a borrow occurs"); facts.md:171, 248
- resolution: code-wrong
- fix: After `CMPR r1, r2` the carry is a borrow, so `HI` (C && !Z) is true exactly when r1 < r2 and `LO` (!C || Z) is true exactly when r1 >= r2 — the comments are inverted. Replace with "ADDR|HI r3, r4, r5            ; 6 cycles (exec if r1 < r2)" and "ADDR|LO r3, r6, r7            ; 6 cycles (exec if r1 >= r2)". The same inversion drives line 176 (`SUBI r7, #4` / `BRAN|HI @loop` loops only while r7 < 4, so the unrolled-loop example never iterates); change that branch to `BRAN|!= @loop`. Note that `10-condition-codes.tex:74,76,82,138,181-184,249,263` states the ARM reading ("HI tests if first > second") throughout, so the fix must be coordinated with that chapter's reviewer; if the architects instead intend the ARM polarity, the defect is in the ALU carry output and this becomes `code-wrong`.
- confidence: high
- status: code-wrong
- ruling: Gate 2 B: depends on the HI/LO predicate inversion, which is being fixed in the code. Manual text stands.

### F-tim-2
- file: chapters/appendix-b-timing.tex
- lines: 74
- severity: blocker
- category: fact
- claim: "ADDI r7, #0, #10              ; 6 cycles"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_immediate.rs:274-279 (only `[DirectRegister, Immediate]`, `[DirectRegister, Immediate, ShiftDefinition]` accepted); assembling this line with `sirc-vm/target/debug/assembler` fails: "The [ADDI] opcode only supports immediate->register addressing mode (e.g. ADDI y1, #1) at line 1, column 6"
- resolution: manual-wrong
- fix: The third operand of the short-immediate form must be a shift definition, not a second immediate. Replace the line with "LOAD r7, #10                  ; 6 cycles" (opcode 0x07, `A-LOAD:41-49`), which is what the surrounding comment ("Process 10 elements") intends.
- confidence: high
- status: accepted

### F-tim-3
- file: chapters/appendix-b-timing.tex
- lines: 146-153
- severity: blocker
- category: fact
- claim: "; Instead of:\nADDR r1, r2, r2\nADDI r1, #0, LSL #1\n\n; Use:\nADDR r1, r2, r2, LSL #1    ; Multiply by 3 in one instruction"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:180-192 (register form computes `shift(S1) op S2`; short-immediate form computes `shift(D) op imm8`); facts.md:153, 348
- resolution: manual-wrong
- fix: The two sequences are not equivalent. The "Instead of" block computes `r1 = (r2 + r2) << 1 = 4 * r2`; the "Use" line computes `(r2 << 1) + r2 = 3 * r2`. Replace the "Instead of" block with the two-instruction sequence that really does yield 3*r2:
  "ADDR r1, r2, r2            ; r1 = 2 * r2" / "ADDR r1, r1, r2            ; r1 = 3 * r2".
- confidence: high
- status: accepted

### F-tim-4
- file: chapters/appendix-b-timing.tex
- lines: 159
- severity: blocker
- category: fact
- claim: "ADDI a, #2"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_immediate.rs:274-279 with sirc-vm/toolchain/src/parsers/instruction.rs:85-106 (an address-register *pair* name `a` parses as `DirectAddressRegister`, which no `ADDI` arm accepts); assembling the line fails with the same "only supports immediate->register addressing mode" error; facts.md:211-212
- resolution: manual-wrong
- fix: ALU instructions take a 4-bit register field, so the operand must be a half of the pair: write "ADDI al, #1" (see also F-tim-5 for the count).
- confidence: high
- status: accepted

### F-tim-5
- file: chapters/appendix-b-timing.tex
- lines: 156-163
- severity: blocker
- category: fact
- claim: "LOAD r1, (#0, a)+          ; Same result, fewer cycles"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:81, 100-104 (`addr_inc` is exactly 1); sirc-vm/peripheral-cpu/tests/instructions/load_test.rs:250-294; memory is word-addressed (sirc-vm/peripheral-cpu/tests/instructions/store_test.rs:105-107); facts.md:155, 326
- resolution: manual-wrong
- fix: Post-increment advances the address register's low word by one *word*, not by two, so `ADDI a, #2` is not the same result. Change the "Instead of" block to "LOAD r1, (#0, a)" / "ADDI al, #1". The same word-vs-byte error appears at line 78, where `LOAD r1, (#0, a)+` followed by `STOR (#-2, a), r1` writes two words below the word just read; that store must be `STOR (#-1, a), r1`.
- confidence: high
- status: accepted

### F-exc-4
- file: chapters/appendix-c-undocumented.tex
- lines: 48, 53, 58, 97-101, 204
- severity: blocker
- category: fact
- claim: "0x0B            & Immediate       & SUB-like + 0x8   & Test variant (no save)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:27-38 (`AluOp`: 0x2 = Subtract, 0x3 = SubtractWithCarry); ALU decode rule `operation = alu_code & 0x7` (execution_effective_address.rs:61-67), so 0xB -> 0x3 = SubtractWithCarry; sirc-vm/peripheral-cpu/tests/instructions/arithmetic_immediate_test.rs:826-828 ("UNDOCUMENTED 0x0B (SBCI no status)")
- resolution: manual-wrong
- fix: 0x0B/0x2B/0x3B are SBC-like, not SUB-like. Change the Pattern cells on lines 48, 53 and 58 to "SBC-like + 0x8"; retitle the subsection at line 97 from "0x0B, 0x2B, 0x3B (SUB Test)" to "0x0B, 0x2B, 0x3B (SBC Test)"; delete the body claim at lines 99-101 ("Probably identical to \mnemonic{CMP} ... Utility: None") because the SUB-test slot is 0x0A/0x2A/0x3A, which is the documented \mnemonic{CMP}; and correct line 204 ("Others (SUB Test, which duplicates CMP)").
- confidence: high
- status: accepted

### F-exc-5
- file: chapters/appendix-c-undocumented.tex
- lines: 49, 54, 59, 103-114, 201
- severity: blocker
- category: fact
- claim: "0x0D            & Immediate       & SBC-like + 0x8   & Test variant (no save)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:27-38 (`AluOp` 0x5 = Or), decode `alu_code & 0x7` -> 0xD maps to 0x5; sirc-vm/peripheral-cpu/tests/instructions/arithmetic_immediate_test.rs:966-968 ("UNDOCUMENTED 0x0D (ORRI no status)") and :970-996, which assert the OR result flags
- resolution: manual-wrong
- fix: 0x0D/0x2D/0x3D are OR-like (flags-only OR), not SBC-like. Change the Pattern cells on lines 49, 54 and 59 to "OR-like + 0x8"; retitle the subsection at line 103 to "0x0D, 0x2D, 0x3D (OR Test)" and replace its body with the OR behaviour (result discarded, N and Z from `a | b`, C and V cleared); correct the Future Standardization bullet at line 201, which currently offers 0x0D/0x2D/0x3D as an SBC test for multi-precision borrow. The genuine SBC-test slot is 0x0B/0x2B/0x3B (see F-exc-4).
- confidence: high
- status: accepted

### F-exc-6
- file: chapters/appendix-c-undocumented.tex
- lines: 107-112
- severity: blocker
- category: contradiction
- claim: "result = operand1 - operand2 - (1 - C)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:246-266 (`perform_subtract_with_carry`: `let (r1, c1) = a.overflowing_sub(b); let (r2, c2) = r1.overflowing_sub(carry_from_previous);` — the incoming C is subtracted directly, not inverted); chapters/13-alu-instructions.tex:148, 457, 494 ("Subtracts incoming C as borrow input"; "r1 = r3 - r5 - borrow"); facts.md:172, HO:19
- resolution: manual-wrong
- fix: Replace the pseudocode line with `result = operand1 - operand2 - C  ; C is the borrow input`. SIRC-1 uses the borrow (not inverted-carry) convention throughout; the `(1 - C)` form contradicts Chapter 13 and the ALU. This block moves to the 0x0B/0x2B/0x3B subsection once F-exc-4 is applied.
- confidence: high
- status: accepted

### F-sum-7
- file: chapters/appendix-d-examples.tex
- lines: 39-42
- severity: blocker
- category: contradiction
- claim: "\lstinputlisting[ caption={Software exception example} ... ]{../../examples/software-exception/software-exception.sasm}" — the included listing executes "EXCP    #0x40"
- evidence: examples/software-exception/software-exception.sasm:18; chapters/06-exceptions.tex:140 "The vectors for user exceptions are in the 0x60-0xFF range" and :98 lists "Triggering a software exception with a vector below 0x60" as a violation; sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs:112-113 USER_EXCEPTION_VECTOR_START = 0x60. The runtime guard at execution.rs:330-333 compares `vector_address_high` (which is vector*2 = 0x80 here) against 0x60, so it does not fire for vector 0x40; on the manual reading of the rule the example is illegal, on the implementation reading any vector >= 0x30 is accepted.
- resolution: code-wrong
- fix: Either change the example (and its vector-table .ORG from 0x0080 to 0x00C0, vector 0x60) so the manual never shows EXCP with a reserved vector, or state in Chapter 6 and Appendix D that the implementation only checks vector*2 >= 0x60. Do not publish a listing that contradicts 06-exceptions.tex:140 without a note.
- confidence: high
- status: code-wrong
- ruling: Gate 2 J: EXCP with a vector below 0x60 must raise a fault (privilege violation), not panic; the guard must compare vector numbers, not doubled addresses. The appendix D example must move to vector 0x60 when the check lands. Manual stands.

### F-con-7
- file: chapters/appendix-e-quick-reference.tex
- lines: 25
- severity: blocker
- category: contradiction
- claim: "\mnemonic{ADCI}   & ALU           & 0x01/0x21/0x31       & NZCV           & Add with carry (immediate, short, or register)"
- evidence: `ADCR` is the register-form mnemonic for opcode 0x31 (chapters/12-instruction-summary.tex:77, chapters/appendix-a-opcode-map.tex:75, chapters/13-alu-instructions.tex:62 "\texttt{ADCR rD, rS1, rS2[, shift]} & 0x31"). Appendix E introduces itself at line 12 as "All instruction mnemonics in alphabetical order", and every other family is split (ADDI/ADDR, ANDI/ANDR, SUBI/SUBR, SBCI/SBCR, ORRI/ORRR, XORI/XORR, CMPI/CMPR, TSAI/TSAR, TSXI/TSXR). `ADCR` is the only mnemonic missing from the index.
- resolution: manual-wrong
- fix: Change the ADCI row opcodes to `0x01/0x21` and insert a new row after it: `\mnemonic{ADCR}   & ALU           & 0x31                 & NZCV           & Add with carry register`.
- confidence: high
- status: accepted

### F-sum-3
- file: chapters/appendix-e-quick-reference.tex
- lines: 25
- severity: blocker
- category: fact
- claim: "\mnemonic{ADCI} & ALU & 0x01/0x21/0x31 & NZCV & Add with carry (immediate, short, or register)"
- evidence: 0x31 is AddRegisterWithCarry, whose mnemonic is ADCR (sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:17,72); ADCI is only 0x01/0x21 (arithmetic_immediate.rs:18,34); chapters/appendix-a-opcode-map.tex:75 lists 0x31 as ADCR
- resolution: manual-wrong
- fix: Change the ADCI row to "0x01/0x21 ... Add with carry, immediate or short immediate" and insert a missing ADCR row after it: "ADCR & ALU & 0x31 & NZCV & Add with carry, register". ADCR is currently absent from the alphabetical index entirely, so the index is not the complete mnemonic list it claims to be.
- confidence: high
- status: accepted

### F-sum-4
- file: chapters/appendix-e-quick-reference.tex
- lines: 72-74
- severity: blocker
- category: fact
- claim: "\mnemonic{RETE} & Meta & COPI \#0x1A00 & -- & Return from exception handler" and "\mnemonic{RSET} & Meta & COPI \#0x1B00 & -- & System reset"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:348-366 (RTE assigns registers.sr = return_status_register, restoring all four condition flags) and :367-370 (Reset sets registers.sr = 0x0); the table header defines "--" as "flags written by the instruction ... none"
- resolution: manual-wrong
- fix: Set the Flags cell for RETE to "all (SR restored from the link register)" and for RSET to "all (SR cleared to 0)".
- confidence: high
- status: accepted

### F-arch-6
- file: chapters/01-introduction.tex
- lines: 95-100, 110
- severity: major
- category: contradiction
- claim: "All implementations of the SIRC-1 will have the \"processing unit\" as coprocessor 0x0, the \"exception unit\" as coprocessor 0x1, and the DMA unit as coprocessor 0x2." / "0x2         & DMA Unit           & No                & Transfers blocks of data via the bus        \\"
- evidence: manual side: chapters/16-coprocessor-instructions.tex:142 also marks DMA "Required: Yes". Code side: sirc-vm/peripheral-cpu/src/lib.rs:375-394 dispatches only coprocessor IDs 0 and 1 and raises `Faults::InvalidOpCode` for every other ID, including 0x2; facts digest "Coprocessors" table records DMA as "not implemented in peripheral-cpu"
- resolution: code-wrong
- fix: Either the reference implementation is deliberately an incomplete model (in which case say so once, for example "the reference simulator model omits the DMA unit and raises an invalid-opcode fault for coprocessor 0x2") or DMA is optional like the maths unit. An emulator author cannot tell from the manual whether coprocessor 0x2 absence is conformant. The chapter needs one sentence settling it; the DMA timing figures in Chapter 16 depend on the same decision.
- confidence: medium
- status: not-implemented
- ruling: Gate 2 M: the DMA unit is Required and planned; the reference emulator has not implemented it yet. Manual stands.

### F-arch-2
- file: chapters/02-cpu-architecture.tex
- lines: 155-159
- severity: major
- category: fact
- claim: "DMA commands are the main exception: after the six-phase coprocessor-call dispatch, the DMA unit keeps instruction fetch stopped until its command and bus-transfer cycles have completed."
- evidence: sirc-vm/peripheral-cpu/tests/exceptions/software_exceptions.rs:41-51 and sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:781-791 (the coprocessor dispatch occupies a second full six-cycle slot after the \mnemonic{COPI} that issued it); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:302-446; chapters/16-coprocessor-instructions.tex:213 ("6 cycles for the coprocessor call, plus exception-unit follow-up work")
- resolution: manual-wrong
- fix: DMA is not the only exception. Every coprocessor call (\mnemonic{COPI}, \mnemonic{COPR}, and every exception-unit meta-instruction: \mnemonic{EXCP}, \mnemonic{WAIT}, \mnemonic{RETE}, \mnemonic{RSET}, \mnemonic{ETFR}, \mnemonic{ETTR}) costs the six cycles of the processing-unit instruction plus a second six-cycle coprocessor dispatch slot, and every exception entry costs an additional six-cycle dispatch. Reword to "Coprocessor calls are the exception: the processing-unit instruction takes six cycles and the coprocessor dispatch that follows takes a further six; DMA commands additionally hold instruction fetch until the transfer completes." The same understatement appears at chapters/01-introduction.tex:25 and chapters/01-introduction.tex:82-85.
- confidence: high
- status: accepted

### F-arch-5
- file: chapters/02-cpu-architecture.tex
- lines: 136-137
- severity: major
- category: fact
- claim: "The shift is applied to the first source operand when fetching the source registers for an instruction. Shifts cannot be applied to the result or any other source operand."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:308-315 (`MemoryLoad` and `AddressWriteLoadPostIncrement` apply `do_shift` to the word returned by the bus before writing the destination register); sirc-vm/peripheral-cpu/tests/instructions/load_test.rs:112-175
- resolution: manual-wrong
- fix: Add the memory-load exception: "For register-displacement \mnemonic{LOAD} forms the shift is applied to the word returned from memory during write-back, not to a register source." Keep the rest of the sentence (the shift never applies to the immediate operand or to the second register source).
- confidence: high
- status: accepted

### F-arch-7
- file: chapters/02-cpu-architecture.tex
- lines: 695-731
- severity: major
- category: contradiction
- claim: "Phase       & D{Mem} D{next} D{}        \\ ... BAS         & H L H                     \\ ... BAT0--BAT2  & D{None} D{DR} D{None}"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/memory_access.rs:56-74 (the data read/write request is asserted during the Memory Access phase); chapters/02-cpu-architecture.tex:809-826 (Figure "Complete LOAD instruction" shows BAS and BAT=DR asserted *during* the Memory column, not the one after)
- resolution: manual-wrong
- fix: In Figures \ref{fig:data-read-timing} and \ref{fig:data-write-timing} the bus request is shown one column later than the "Mem" phase label, contradicting the full-instruction figures. Shift the BAS/BRW/BAT/A/D/BACK columns left by one so the request is asserted during the Memory Access phase, or relabel the Phase row so the asserting column is the one named "Mem".
- confidence: medium
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-arch-3
- file: chapters/03-registers.tex
- lines: 238
- severity: major
- category: fact
- claim: "0x0         & \reg{sr}          & Upper byte only     \\"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:21-27, 42-43 (`PRIVILEGED_REGISTERS` contains `Sr`; any non-COP instruction whose destination field is `sr` fails the protected-mode check); sirc-vm/peripheral-cpu/tests/instructions/protected_mode_test.rs:95-132; chapters/03-registers.tex:192-193 ("Direct writes to \reg{sr} raise a privilege violation fault")
- resolution: manual-wrong
- fix: The "Upper byte only" qualifier is true for reads only. As a destination, `sr` is wholly privileged: in protected mode *any* instruction that names register 0x0 as its destination raises a privilege violation before write-back, including one that would only alter the unprivileged low byte. Change the cell to "Reads: upper byte redacted. Writes: fully privileged" (or split the column into Read/Write). Note also that this makes any instruction encoded with register field 0x0 privileged in protected mode.
- confidence: high
- status: accepted

### F-arch-4
- file: chapters/03-registers.tex
- lines: 285-289
- severity: major
- category: fact
- claim: "; Push r1 onto stack (pre-decrement)\nSTOR -(#2, s), r1\n\n; Pop stack into r2 (post-increment)\nLOAD r2, (s)+"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:81-93 (pre-decrement stores to `A.low + displacement - 1` and writes back `A.low - 1`; the auto-decrement is always exactly one word regardless of the displacement); sirc-vm/peripheral-cpu/tests/instructions/store_test.rs:250-293
- resolution: manual-wrong
- fix: `STOR -(#2, s), r1` writes to word address `sl + 1` and then sets `sl = sl - 1`, so the matching `LOAD r2, (s)+` reads `sl - 1` and does not recover the pushed value. Use the zero-displacement shorthand for a push: `STOR -(s), r1` (equivalently `STOR -(#0, s), r1`), which writes `sl - 1` and sets `sl = sl - 1`, pairing correctly with `LOAD r2, (s)+`.
- confidence: high
- status: accepted

### F-data-10
- file: chapters/04-data-representation.tex
- lines: 154-157
- severity: major
- category: fact
- claim: "If the low word overflows or underflows and \texttt{SR.A} (Trap on Address Overflow) is set, the instruction raises a segment-overflow fault."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:76-115. The only test is an unsigned 16-bit carry out of A.low + displacement and of (A.low + displacement) + addr_inc, where addr_inc for a pre-decrement is 0xFFFF. Consequences: (a) a pre-decrement whose displaced address is nonzero always sets the carry and therefore always faults when A is set, even though no underflow occurred; (b) a genuine underflow (displaced address 0, decrement to 0xFFFF) produces no carry and does not fault. No test covers pre-decrement or underflow with A set (sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:640-693 uses a positive displacement only).
- resolution: code-wrong
- fix: Either the manual must describe the actual rule (the trap fires on unsigned carry out of the low-word addition, which is not the same as underflow detection), or execution_effective_address.rs:88-115 must compute borrow separately for the decrement path. Present both sides until the author rules.
- confidence: medium
- status: code-wrong
- ruling: Gate 2 D: the address-overflow trap must detect signed wrap of the low word, not unsigned carry. Manual rule stands.

### F-data-11
- file: chapters/04-data-representation.tex
- lines: 119-124
- severity: major
- category: fact
- claim: "Memory displacements           & 16 bits        & Two's-complement offset added to the low address word"
- evidence: Open question 8 of docs/reference/review/facts.md; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:77-83 and :110-115. The displacement is added as an unsigned 16-bit value, so it wraps identically to a two-complement add only while SR.A is clear; with SR.A set, any negative displacement whose add wraps (for example LOAD r1, (#-2, a) with al >= 2) raises SegmentOverflow. Tests use positive displacements only.
- resolution: manual-wrong
- fix: State explicitly whether negative displacements are architecturally usable while SR.A is set, and if the current behaviour is intended, add a sentence to this table row and to the Arithmetic and Address Wraparound section: "A displacement whose addition wraps the low word raises a segment-overflow fault when SR.A is set, including displacements intended as negative values."
- confidence: high
- status: accepted
- ruling: Gate 2 D: add one sentence that displacements are signed two's-complement and that the trap fires only when the low-word calculation wraps (the reference implementation currently traps on unsigned carry; that is an implementation bug, do not document it).

### F-data-13
- file: chapters/04-data-representation.tex
- lines: 130-133
- severity: major
- category: fact
- claim: "Assemblers must reject values that cannot be represented in the selected immediate field width unless they deliberately offer a documented truncation mode."
- evidence: sirc-vm/toolchain/src/parsers/instruction.rs:108-116 (parse_value casts the parsed 32-bit number straight to u16 with a TODO acknowledging the unchecked cast), so ADDI r1, #70000 assembles silently as 0x1170 and CMPI r1, #-10 assembles as 0xFFF6. Only the 8-bit short-immediate-with-shift form is range checked (toolchain/src/parsers/opcodes/arithmetic_immediate.rs:214-222).
- resolution: manual-wrong
- fix: Either relax the sentence to describe what the reference assembler does (values are truncated to the field width; only short immediates with a shift are diagnosed) or keep it as a requirement and record that the reference assembler does not yet conform. Note that chapter 10 line 153 relies on the truncation behaviour with CMPI r1, #-10.
- confidence: high
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-con-11
- file: chapters/05-status-register.tex
- lines: 45-46
- severity: major
- category: contradiction
- claim: "These flags reflect the results of ALU operations and comparisons. They can be read and written by code running in any privilege level."
- evidence: Contradicted three times in the same chapter and once in Chapter 3: chapters/05-status-register.tex:10 "Condition flags -- readable and writable in both supervisor and protected modes"; chapters/05-status-register.tex:90-91 "These flags control CPU behavior and can only be modified directly in supervisor mode. Attempts to write \reg{sr} in protected mode raise a privilege violation fault."; chapters/05-status-register.tex:207-209 "When running in protected mode, any direct write to the status register raises a privilege violation fault ... they cannot directly load or store \reg{sr}."; chapters/03-registers.tex:192-193 "Direct writes to \reg{sr} raise a privilege violation fault."
- resolution: manual-wrong
- fix: Change line 10 to "Condition flags -- readable in both supervisor and protected modes; modifiable in protected mode only as a side effect of ALU and shift instruction execution" and lines 45-46 to "They can be read at any privilege level and are updated by ordinary instruction execution at any privilege level, but a direct write to \reg{sr} is a privileged operation (see Section \"Manual Updates\")."
- confidence: high
- status: accepted

### F-data-12
- file: chapters/05-status-register.tex
- lines: 183
- severity: major
- category: fact
- claim: "\textbf{Load Instructions} (\mnemonic{LOAD}, \mnemonic{LDEA}): Does not update condition flags"
- evidence: Open question 9 of docs/reference/review/facts.md. Three observable behaviours: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:307-317 with AF=Alu sets N and Z from the loaded value and clears C and V (tests/instructions/arithmetic_register_test.rs:497-534); the assembler always emits AF=None for LOAD (toolchain/src/parsers/opcodes/load.rs:47, 128), so assembled LOAD never updates flags; memory loads never update flags whatever AF is (stages/write_back.rs:308-315). LDEA never updates flags (write_back.rs:203-236).
- resolution: manual-wrong
- fix: Decide which behaviour is architectural and say so here. Suggested wording once ruled: "LDEA and memory-form LOAD never update the condition flags. Register and immediate LOAD update flags only when the status source field selects the ALU; the assembler emits None for every LOAD form, so assembled LOAD never updates flags."
- confidence: high
- status: accepted
- ruling: Gate 2 C: LOAD never updates flags (row `- - - -`, default AF [N]); any explicit AF suffix on LOAD leaves all four flags undefined (`U U U U`). Say so once in ch5 and ch13 and cross-reference from ch12 and ch14.

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
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-con-13
- file: chapters/06-exceptions.tex
- lines: 442-443
- severity: major
- category: contradiction
- claim: "The vector-table address is computed by multiplying the vector ID by 2 (since vectors are 32-bit addresses stored as two 16-bit words), then adding the system RAM offset."
- evidence: `system_ram_offset` / "system RAM offset" appears only at chapters/06-exceptions.tex:443, :615 and :616 and is never defined anywhere in the manual (grep across chapters/ and generated/ returns only those three hits). Every other statement of the vector address is a flat V*2 with no offset: chapters/01-introduction.tex:157 "The CPU expects the exception vector table to begin at word address \addr{0x000000}"; chapters/04-data-representation.tex:98 "Vector entry \texttt{V} starts at word address \texttt{V * 2}"; chapters/06-exceptions.tex:506 "The exception vectors are stored starting at address 0x0"; chapters/06-exceptions.tex:508 "The high word is stored first at \texttt{vector * 2}".
- resolution: manual-wrong
- fix: Either define `system_ram_offset` (a normative subsection in Chapter 6 stating what it is, its reset value, and how software sets it — and then correct Chapter 1 and Chapter 4 to include it), or delete it from 06-exceptions.tex:443, :615 and :616 and write "\texttt{0x000000}" / "\texttt{0x000001}". Do not leave it undefined.
- confidence: high
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-con-24
- file: chapters/06-exceptions.tex
- lines: 631-637
- severity: major
- category: contradiction
- claim: "Reads the link register corresponding to the current interrupt mask level (minus 1, since the mask is set to the exception level) \\ Restores the status register from the link register (including the interrupt mask and protected mode bit)"
- evidence: The same chapter describes RETE differently 130 lines earlier: chapters/06-exceptions.tex:494-499 "The return address is loaded from the link register into the program counter \\ The saved status register value is restored (including interrupt enable bits) \\ The saved exception level is restored to current\_exception\_level". The term "interrupt mask" / "interrupt mask level" appears nowhere else in the manual; the status register has four *interrupt enable* bits E1--E4 (chapters/05-status-register.tex:107-116) and the exception unit tracks `current_exception_level` (chapters/06-exceptions.tex:377). The second version also omits restoring the exception level.
- resolution: manual-wrong
- fix: Delete the duplicate enumeration at chapters/06-exceptions.tex:631-637 and replace it with a cross-reference to the "Returning from Exceptions" list at chapters/06-exceptions.tex:494-499, or rewrite it using `current_exception_level` and "interrupt enable bits". Do not use "interrupt mask level" anywhere.
- confidence: high
- status: accepted

### F-exc-10
- file: chapters/06-exceptions.tex
- lines: 494-499, 631-637
- severity: major
- category: fact
- claim: "\item The saved status register value is restored (including interrupt enable bits) \item The saved exception level is restored to current\_exception\_level"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:361-365 — after restoring SR, `if saved_exception_level == 0 { clear_sr_bit(ExceptionActive) }`; verified sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:864-869
- resolution: manual-wrong
- fix: Both RETE step lists omit the SR.EA rule. Add a step after the SR restore: "\item If the restored exception level is 0, \texttt{SR.EA} is cleared; otherwise the restored value of \texttt{SR.EA} stands." Without this an implementer restoring the saved SR verbatim gets EA wrong on every nested return. Same omission at lines 631-637 (the "Return from Exception" section) — fix both.
- confidence: high
- status: accepted

### F-exc-11
- file: chapters/06-exceptions.tex
- lines: 425-448
- severity: major
- category: contradiction
- claim: "\item The current program counter, status register, and current exception level are stored in the appropriate link register based on the exception priority level." ... "\item The vector-table address is computed ... \item The exception unit reads the high word and then the low word of the vector target address"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:302-334 — the vector high word is fetched at phase 0 and the low word at phase 1; the link-register write, SR updates and PC load all happen at phase 3 (`ExecutionEffectiveAddressExecutor`, :335-262). chapters/06-exceptions.tex:459-461 states the correct order ("Architectural entry side effects commit only after the vector target has been fetched")
- resolution: manual-wrong
- fix: Reorder the numbered list so the vector-address computation and the two vector-word reads (current steps 7 and 8) come before the link-register write, SR updates and level update (current steps 4, 5, 6). As written the list contradicts the "Exception Entry Side Effects" section two pages later and the vector-fetch-fault rule at lines 485-488, which only makes sense if nothing has been committed when the fetch fails.
- confidence: high
- status: accepted

### F-exc-12
- file: chapters/06-exceptions.tex
- lines: 101-107, 546
- severity: major
- category: fact
- claim: "\textbf{Instruction Trace Fault} \textit{(Post-instruction)} — Raised after every instruction completes when the \texttt{TraceMode} SR bit is set."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:349-353 — `self.trace_mode_sampled = coprocessor_id == ProcessingUnitExecutor::COPROCESSOR_ID && (sr_bit_is_set(TraceMode, ...) || bus_assertions.force_trace_mode)`; the TRCE pin is documented at chapters/02-cpu-architecture.tex:302, 507, 891
- resolution: manual-wrong
- fix: Add "or when the \texttt{TRCE} input is asserted at the instruction boundary (see Section~\ref{...} in Chapter~\ref{ch:architecture})" to the fault description, and change the Maskability cell for vector 0x06 in Table~\ref{tab:exception-quick-reference} (line 546) from "SR.T gated" to "SR.T or TRCE gated". Also add the sampling rule from lib.rs:344-353: T is sampled once at phase 0 before the instruction can modify SR, and exception-unit dispatches are never traced.
- confidence: high
- status: accepted

### F-exc-13
- file: chapters/06-exceptions.tex
- lines: 442-443, 506, 615-616
- severity: major
- category: fact
- claim: "The vector-table address is computed by multiplying the vector ID by 2 ... then adding the system RAM offset."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:303-305 — `self.vector_address = registers.system_ram_offset | vector_address_offset` (bitwise OR, not addition); sirc-vm/peripheral-cpu/src/registers.rs:162-167 defines `system_ram_offset` as a CPU-internal register "not directly accessible to programs"; it is set once at construction (`new_cpu_peripheral(0x0)` in sirc-vm/sirc-vm/src/main.rs:186 and in every test). Chapter 6 line 506 says the vectors "are stored starting at address 0x0" and Table~\ref{tab:exception-quick-reference} gives absolute addresses 0x0000--0x01FF, both of which assume the offset is 0.
- resolution: manual-wrong
- fix: `system_ram_offset` is named in three places in Chapter 6 and defined nowhere in the manual. Either (a) drop it and state flatly that the vector table starts at word address 0x000000, matching lines 506 and the quick-reference table and the only value the implementation is ever given, or (b) add a normative definition (width, reset value, how it is programmed, and that the vector address is formed by OR, not add, so a non-zero offset must be aligned to a 512-word boundary). As written an implementer cannot build either behaviour. If (a) is chosen, replace the two `system\_ram\_offset + 0x000N` references at lines 615-616 with `0x000000` and `0x000001`.
- confidence: high
- status: accepted

### F-exc-14
- file: chapters/06-exceptions.tex
- lines: 571-593
- severity: major
- category: fact
- claim: "\textbf{State} & \textbf{Reset Behavior}" (Table~\ref{tab:reset-state})
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:524-536 — `reset()` also sets `eu_registers.current_exception_level = 0`; the table has no row for it. The current exception level is architecturally visible: it gates software-exception acceptance (lib.rs:326-337) and selects the link register on entry (execution.rs:258).
- resolution: manual-wrong
- fix: Add a row: "Current exception level & Cleared to 0. The CPU leaves reset outside any exception handler." The rest of the table matches lib.rs:524-536 and facts.md Open question 12 is satisfied by the "Not architecturally cleared ... treat as undefined" wording already present.
- confidence: high
- status: accepted

### F-exc-15
- file: chapters/06-exceptions.tex
- lines: 215-247
- severity: major
- category: fact
- claim: "For robust fault handling that can recover from double faults, fault handlers should save the link register contents early in the handler:"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:371-415 — `ETFR`/`ETTR` move only `return_address` and `return_status_register`; `saved_exception_level` cannot be read or written. On a double fault, execution.rs:255-262 overwrites `link_registers[6].saved_exception_level` with the level current at that moment (7), so after the `ETTR #6` in this listing RETE restores level 7, not the original level (execution.rs:356-362).
- resolution: manual-wrong
- fix: Add a sentence after the listing: "This sequence restores the return address and status register of link register 6 but not its saved exception level, which software cannot access. A handler that has taken a double fault therefore cannot fully reconstruct the pre-fault exception level; treat the CPU state as unrecoverable and reset." Alternatively demote the listing to an illustration of *capturing* fault state for a register dump, which is what the surrounding prose (lines 208-213) actually recommends.
- confidence: medium
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-exc-16
- file: chapters/06-exceptions.tex
- lines: 682-688
- severity: major
- category: fact
- claim: "The following opcodes are used internally by the exception unit and are not directly accessible via user instructions:"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:416-427 — `Fault`, `HardwareException` and `SoftwareException` share one dispatch arm, reached from any cause value, including a `pending_coprocessor_command` written by a privileged `COPI #0x1Exx` / `COPI #0x1Fxx`; the privilege check (processing_unit/execution.rs:46-48) only rejects opcode nibbles > 7 in protected mode, so a supervisor-mode COPI reaches the EU unmodified. sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:1321-1323 records this as unresolved. facts.md Open question 14.
- resolution: manual-wrong
- fix: The claim "not directly accessible via user instructions" is true for protected mode but false for supervisor mode. Either state the restriction precisely ("cannot be issued from protected mode; issuing them from supervisor mode via \mnemonic{COPI} is architecturally undefined") or have the implementation reject them. Author must rule.
- confidence: medium
- status: accepted
- ruling: Gate 2 K/L: exception-unit opcodes 0x2--0x8, software-issued 0xE/0xF, RETE at exception level 0, and two fault conditions detected in one instruction are all architecturally undefined. Say so in one sentence each; do not describe the panic.

### F-exc-17
- file: chapters/06-exceptions.tex
- lines: 660-676
- severity: major
- category: fact
- claim: "\mnemonic{EXCP}      & 0x1             & User               & Trigger a software exception (trap)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs:148-162 defines only opcodes 0x0, 0x1, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF; sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:69-77 panics on any other value ("Unimplemented op code ... for exception co-processor"), with a TODO saying hardware behaviour is undecided. facts.md Open question 6.
- resolution: manual-wrong
- fix: The table plus the "Internal Instructions" list covers 9 of the 16 exception-unit opcodes and never says what 0x2--0x8 are. Add one line after the table: "Exception coprocessor opcodes 0x2--0x8 are reserved. Issuing one is architecturally undefined." Confirm with the author that "reserved/undefined" (not "no-op" and not "invalid opcode fault") is the intended ruling, since the implementation currently halts.
- confidence: medium
- status: accepted
- ruling: Gate 2 K/L: exception-unit opcodes 0x2--0x8, software-issued 0xE/0xF, RETE at exception level 0, and two fault conditions detected in one instruction are all architecturally undefined. Say so in one sentence each; do not describe the panic.

### F-exc-18
- file: chapters/06-exceptions.tex
- lines: 626-641
- severity: major
- category: fact
- claim: "\item Reads the link register corresponding to the current interrupt mask level (minus 1, since the mask is set to the exception level)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:350-356 — `eu_registers.link_registers[(current_exception_level - 1) as usize]` with a TODO: "Overflow error when `current_exception_level` is zero. What should actually happen on hardware if you try to RTE when you're not in an exception". facts.md Open question 5.
- resolution: manual-wrong
- fix: The manual never says what \mnemonic{RETE} does outside a handler. Add a normative sentence, for example "Executing \mnemonic{RETE} while the current exception level is 0 is architecturally undefined", and confirm with the author. Separately, "current interrupt mask level" is not a register this manual defines anywhere — the code field is `current_exception_level` (registers.rs:590-603) and the chapter uses that name at line 377; make the two agree.
- confidence: medium
- status: accepted
- ruling: Gate 2 K/L: exception-unit opcodes 0x2--0x8, software-issued 0xE/0xF, RETE at exception level 0, and two fault conditions detected in one instruction are all architecturally undefined. Say so in one sentence each; do not describe the panic.

### F-exc-28
- file: chapters/06-exceptions.tex
- lines: 116-127
- severity: major
- category: fact
- claim: "\textbf{Double Fault} \textit{(Exception dispatch)} — Raised when a fault occurs while another fault is already being handled."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:147-157 — `raise_fault` panics ("Cannot raise fault when one is pending") when a second fault condition is detected while `pending_fault` is still set, i.e. in the window between raising a fault and dispatching it, with a TODO asking what hardware should do. The documented double-fault path (lib.rs:164-171) only covers a fault raised once `current_exception_level >= 7`. facts.md Open question 7.
- resolution: manual-wrong
- fix: The manual describes only the level-7 case. It does not say what happens when two fault conditions are detected in the same instruction before either is dispatched (for example an alignment fault and a bus error on the same fetch), and the implementation halts there. Add a normative sentence covering the pre-dispatch case, or confirm with the author that it is architecturally undefined. Note that the priority order BERR > BPER > BACK is defined (exception_unit/definitions.rs:93) but the fault-vs-fault order is not.
- confidence: medium
- status: accepted
- ruling: Gate 2 K/L: exception-unit opcodes 0x2--0x8, software-issued 0xE/0xF, RETE at exception level 0, and two fault conditions detected in one instruction are all architecturally undefined. Say so in one sentence each; do not describe the panic.

### F-exc-7
- file: chapters/06-exceptions.tex
- lines: 93-99
- severity: major
- category: fact
- claim: "\item Triggering a software exception with a vector below 0x60"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:330-333 — the exception unit `assert!`s (panics) when a SoftwareException opcode carries a vector below `USER_EXCEPTION_VECTOR_START`, it does not raise PrivilegeViolation; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:46-48 makes the privilege decision purely on the COP opcode nibble (EXCP is opcode 0x1, so never privileged); sirc-vm/toolchain/src/parsers/opcodes/exception.rs:73 masks the vector to 8 bits with no range check. Against this: `exception_unit/definitions.rs:51-55` lists it as a privilege violation. No test covers it. facts.md Open question 4.
- resolution: code-wrong
- fix: The manual asserts a fault the implementation cannot raise. Either the implementation must add the check (fault instead of panic), or item 5 of this list must be deleted and Section "User Exceptions (Traps)" must state what an `EXCP` below 0x60 does (currently: architecturally undefined). Author must rule; do not leave the claim as written, since an emulator author following the manual would raise vector 0x05 where the reference implementation halts.
- confidence: high
- status: code-wrong
- ruling: Gate 2 J: EXCP with a vector below 0x60 must raise a fault (privilege violation), not panic; the guard must compare vector numbers, not doubled addresses. The appendix D example must move to vector 0x60 when the check lands. Manual stands.

### F-exc-8
- file: chapters/06-exceptions.tex
- lines: 163-168
- severity: major
- category: contradiction
- claim: "Each link register stores two pieces of information:"
- evidence: sirc-vm/peripheral-cpu/src/registers.rs:581-588 (`ExceptionLinkRegister { return_address, return_status_register, saved_exception_level }`); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:255-262 writes all three on entry and :356-361 restores `current_exception_level` from `saved_exception_level` on RETE; the figure at 06-exceptions.tex:170-179 itself shows three fields
- resolution: manual-wrong
- fix: Change to "Each link register stores three pieces of information:" and add a third bullet: "\textbf{Saved Exception Level:} The exception level that was active when the exception was taken; \mnemonic{RETE} restores it into \texttt{current\_exception\_level}." This is normative — without it an implementer cannot build RETE's level restore or the SR.EA rule (see F-exc-10).
- confidence: high
- status: accepted

### F-exc-9
- file: chapters/06-exceptions.tex
- lines: 170-179
- severity: major
- category: fact
- claim: "\bitbox{16}{Return Status Register} & \bitbox{8}{Saved Level} & \bitbox{8}{Reserved}"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:371-415 — `ETFR`/`ETTR` `register_select` covers only 1 (`return_address` <-> `a`), 2 (`return_status_register` <-> `r7`) and 3 (both); there is no encoding that reads or writes `saved_exception_level`. The struct at registers.rs:581-588 is three separate fields with no packed bit layout anywhere in the code.
- resolution: manual-wrong
- fix: The figure invents a 64-bit packed layout (Saved Level at bits 15--8 of the second word, Reserved at 7--0) that no code defines and that software cannot observe. Either state that the layout is illustrative only and add a sentence "The saved exception level is not accessible to software; \mnemonic{ETFR} and \mnemonic{ETTR} transfer only the return address and the return status register", or drop the packed figure. Author must confirm whether the packing is architectural.
- confidence: medium
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-con-9
- file: chapters/07-instruction-formats.tex
- lines: 232-238
- severity: major
- category: contradiction
- claim: "Note: If R3 is not specified, the assembler with use R1 as both destination and first source operand when encoding the instruction: ... ADDR r1, r2      ; r1 = r1 + r2 (R3 implicitly = R1)"
- evidence: The same chapter's own note says the opposite field defaults: chapters/07-instruction-formats.tex:246 "When R3 is not specified in assembly, it defaults to R1 (e.g., \texttt{ADDR r4, r5} uses R1=0x4, R2=0x4, R3=0x5)" — that example shows **R2** taking R1's value and the second operand landing in R3. generated/register-format-encodings.tex:38 confirms: `CMPR r4, r5` encodes R1=0x4, R2=0x4, R3=0x5. chapters/13-alu-instructions.tex:95-97 states it correctly: "When a register-form ALU instruction omits \texttt{rS1}, the assembler encodes \texttt{rD} as both the destination and first source operand. For example, \texttt{ADDR r1, r2} is encoded as \texttt{ADDR r1, r1, r2}."
- resolution: manual-wrong
- fix: At line 232 change "If R3 is not specified" to "If only two register operands are given"; at line 237 change the comment to "; r1 = r1 + r2 (R2 implicitly = R1)"; at line 246 change "When R3 is not specified in assembly, it defaults to R1" to "When only two register operands are given, R2 is encoded as a copy of R1 and the second operand is encoded in R3". Also fix the typo "the assembler with use" -> "the assembler will use".
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted

### F-enc-10
- file: chapters/08-addressing-modes.tex
- lines: 6-26
- severity: major
- category: fact
- claim: "The SIRC-1 CPU supports seven addressing modes ... Immediate / Register Direct / Indirect Immediate / Indirect Register / Post-Increment / Pre-Decrement / Short Immediate"
- evidence: docs/reference/manual-handover.md:17 fixes the count at seven but does not enumerate them; sirc-vm/toolchain/src/parsers/instruction.rs:85-106 distinguishes ten operand forms including `DirectAddressRegister` (`l`, `a`, `s`, `p`), which this list does not cover although the chapter's own family table uses it (`LDEA dest`, `LJMP src`, chapters/08-addressing-modes.tex:64-66); facts.md Open question 11
- resolution: manual-wrong
- fix: Either (a) keep seven and replace "Short Immediate" (an immediate-width variant of Immediate, not a distinct operand access path) with "Address Register Direct" so that `LDEA a, ...` and `LJMP a` are covered, or (b) keep the current seven and add an explicit sentence that naming an address-register pair as a destination/source operand is not counted as an addressing mode. Both sides: the handover fixes the count at seven; the parser has ten operand forms and the current list leaves one legal operand form unclassified.
- confidence: medium
- status: accepted
- ruling: Gate 2 N: the ch8 editor derives the addressing-mode count and list from the ten assembler operand forms, adds a mapping table, and records the final count in the changes file's open-questions section. Ch1 and ch11 are updated to that count in Phase 4.

### F-enc-11
- file: chapters/08-addressing-modes.tex
- lines: 186-197
- severity: major
- category: fact
- claim: "The effective address is computed by adding a 16-bit signed immediate offset to an address register pair low word." ... "STOR (#-4, s), r2     ; Store to address (s - 4)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:77-83 and 110-115 (the effective address is an unsigned 16-bit add and the SegmentOverflow trap fires on unsigned wrap when SR.A is set, so a negative displacement traps whenever `A.low >= |displacement|`); tests only exercise positive displacements (sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:640-693); facts.md Open question 8
- resolution: manual-wrong
- fix: State in this section whether negative displacements are architecturally usable while SR.A (TrapOnAddressOverflow) is set. Both sides: the encoding and the wrapping add make `(#-4, s)` behave as `s - 4`, but the implementation's overflow check is unsigned, so with SR.A set the same instruction raises SegmentOverflow. Either document the trap as intended (and mark negative displacements as unusable with the trap enabled) or record the check as a code defect.
- confidence: high
- status: accepted
- ruling: Gate 2 D: add one sentence that displacements are signed two's-complement and that the trap fires only when the low-word calculation wraps (the reference implementation currently traps on unsigned carry; that is an implementation bug, do not document it).

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
- status: accepted

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
- status: accepted

### F-con-18
- file: chapters/09-shift-operations.tex
- lines: 27
- severity: major
- category: contradiction
- claim: "000           & NUL (or none)     & None                   & No shift performed"
- evidence: chapters/07-instruction-formats.tex:298 gives the same encoding with mnemonic "--": "000           & None (no shift)        & --". The two tables (`tab:shift-types` at 07-instruction-formats.tex:309 and `tab:shift-encoding` at 09-shift-operations.tex:38) are duplicates of the same normative encoding that have drifted; the mnemonic `NUL` occurs nowhere else in the manual, and `\ref{tab:shift-types}` is what both instruction-format sections cite (07-instruction-formats.tex:145, :204) while appendix-e-quick-reference.tex:186 also points at `tab:shift-types`.
- resolution: manual-wrong
- fix: Delete `tab:shift-encoding` from Chapter 9 and replace it with a reference to `Table~\ref{tab:shift-types}` plus the Description column merged into Chapter 7's table; or, if Chapter 9 keeps its table, make row 000 read "-- (no shift)" in both and decide whether `NUL` is a real assembler mnemonic. Do not ship two encoding tables that disagree.
- confidence: high
- status: accepted

### F-con-26
- file: chapters/09-shift-operations.tex
- lines: 265-267
- severity: major
- category: contradiction
- claim: "; Add (r1 * 4) + 1 to r1 \\ ADDI r1, #1, LSL #2  ; r1 = r1 + (r2 << 2) + 1 \\                           ; SO=0: r2 is shifted by literal 2"
- evidence: chapters/09-shift-operations.tex:60 (`tab:source-operand-examples`) states that `ADDI r1, #2, LSL #1` is interpreted as `ADDI r1, r1, #2, LSL #1` with first source operand **r1**; chapters/09-shift-operations.tex:46-48 "If using the two operand form of instruction, the second operand is inferred to be the same as the destination"; chapters/13-alu-instructions.tex:254 gives the semantics as `ADDI rD, #imm8, shift ; rD = (rD << shift) + imm8`. The example names `r2`, which does not appear in the instruction, and the arithmetic (`r1 + (...) + 1`) double-counts the source.
- resolution: manual-wrong
- fix: Change the comment block to `ADDI r1, #1, LSL #2  ; r1 = (r1 << 2) + 1` / `; SO=0: r1 is shifted by literal 2`, and change the section comment at line 265 to "; Multiply r1 by 4 and add 1".
- confidence: high
- status: accepted

### F-con-27
- file: chapters/09-shift-operations.tex
- lines: 95-96
- severity: major
- category: contradiction
- claim: "; Set bit 10 \\ ORRI r3, #1, LSL #10     ; r3 |= (r3 << 10) with immediate 1 = 0x0400"
- evidence: The shift is applied to the register source, never to the immediate: chapters/09-shift-operations.tex:46 "It is applied to the first source operand of an instruction"; chapters/08-addressing-modes.tex:325 "The shift is applied to the source operand, not the constant"; chapters/13-alu-instructions.tex:580 "\texttt{ORRI rD, \#imm8, shift          ; rD = (rD << shift) | imm8}". `ORRI r3, #1, LSL #10` computes `(r3 << 10) | 1`, which does not set bit 10 and is not `0x0400`.
- resolution: manual-wrong
- fix: Replace both lines with `; Set bit 10 (16-bit immediate form)` / `ORRI r3, #0x0400         ; r3 |= 0x0400`.
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted

### F-enc-21
- file: chapters/09-shift-operations.tex
- lines: 286-294
- severity: major
- category: fact
- claim: "The shift count is a 4-bit field, allowing shifts of 0--15 positions. \item Maximum shift count is 15 \item Shift counts $\geq$ 16 would be meaningless for 16-bit values"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:85-89 and stages/fetch_and_decode.rs:60-69 (with SO = 1 the count is the full 16-bit value of the named register, not a 4-bit field); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:372, 395, 425, 457 (LSL/LSR/ASL/ASR clamp the count to 16) versus :479, 495 (rotates take the count modulo 16); no test covers register counts above 15 (sirc-vm/peripheral-cpu/tests/instructions/arithmetic_register_test.rs:953-959); facts.md Open question 13
- resolution: manual-wrong
- fix: Restrict "4-bit field, maximum 15" to the SO = 0 (immediate count) case — the assembler enforces it (sirc-vm/toolchain/src/parsers/instruction.rs:225-231) and the encoder masks the field to 4 bits — and add a normative statement for SO = 1: say what a register count greater than 15 does. Both sides: the manual says counts above 15 are impossible; the implementation accepts a 16-bit register count and gives shifts a zero (or sign-only) result while rotates wrap modulo 16. The same claim appears at chapters/09-shift-operations.tex:439-440.
- confidence: high
- status: accepted
- ruling: Gate 2 E: immediate shift counts are 0--15; a register shift count above 15 is architecturally undefined. Do not document the clamp/modulo behaviour.

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
- status: accepted

### F-con-19
- file: chapters/10-condition-codes.tex
- lines: 245
- severity: major
- category: contradiction
- claim: "ADDI r1, #0, #10         ; i = 10"
- evidence: No `ADDI` form takes two immediates. chapters/13-alu-instructions.tex:57-58 and :253-254 give the only immediate forms: `ADDI rD, #imm16` and `ADDI rD, #imm8, shift`; chapters/08-addressing-modes.tex:59 confirms. The idiom used everywhere else to load a constant is `LOAD` (chapters/13-alu-instructions.tex:721, chapters/09-shift-operations.tex:377). The same broken line appears at chapters/appendix-b-timing.tex:74 "ADDI r7, #0, #10              ; 6 cycles".
- resolution: manual-wrong
- fix: Change chapters/10-condition-codes.tex:245 to `LOAD r1, #10             ; i = 10` and chapters/appendix-b-timing.tex:74 to `LOAD r7, #10                  ; 6 cycles`.
- confidence: high
- status: accepted

### F-con-28
- file: chapters/11-reading-instructions.tex
- lines: 9-25
- severity: major
- category: contradiction
- claim: "\item[Write-back] ... \item[Status Flags] The effect on the condition flags in the status register. \item[Condition Codes] All normal instructions may be conditional."
- evidence: Every actual entry in Chapters 13--17 uses the labels `Flags:` and `Condition codes:` (sentence case), and orders the fields Operation -> Description -> Flags -> Write-back -> Exceptions -> Condition codes -> Timing -> Privilege (chapters/13-alu-instructions.tex:263-292 is representative; also :332-360, :459-486). Chapter 11 lists Write-back before Status Flags, Condition Codes before Exceptions, and omits `Description`, `Example`, and `Notes` entirely. docs/reference/STYLE.md:58-69 rules that the entries win: "Do not use \"Status Flags\" or \"Condition Codes\" (title case) as field labels — use \"Flags\" and \"Condition codes.\""
- resolution: manual-wrong
- fix: Rewrite the Chapter 11 field list in the canonical order and names from STYLE.md:63-65: Opcodes (or Assembles to) -> Syntax -> Operands -> Operation -> Description -> Flags -> Write-back -> Exceptions -> Condition codes -> Timing -> Privilege -> Example/Examples -> Notes. Related but separate drift to fix at the same time: chapters/14-memory-instructions.tex:166-180 puts Write-back before Flags and Timing before Condition codes, and chapters/16-coprocessor-instructions.tex:395-433 puts Description before Operation — bring all boxes into the canonical order.
- confidence: high
- status: accepted

### F-sum-10
- file: chapters/11-reading-instructions.tex
- lines: 41
- severity: major
- category: fact
- claim: "\texttt{shift} & Optional SIRCIS shift definition"
- evidence: facts.md Open question 16; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:180-192 applies the shift to operand `a` only, so `ADDI r1, #2, LSL #3` computes (r1 << 3) + 2 and `ADDR r1, r2, r3, LSL #2` computes (r2 << 2) + r3 (T-ASI:507-596; T-AR)
- resolution: manual-wrong
- fix: The notation table is the one place that defines `shift` for all of Part III, so it should state which operand the shift applies to. Proposed row text: "shift & Optional shift definition. The shift is applied to the first (register) operand before the operation; it never applies to the immediate or to the second source register." Confirm the wording against Chapter 9 before applying.
- confidence: medium
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-sum-11
- file: chapters/11-reading-instructions.tex
- lines: 14
- severity: major
- category: fact
- claim: "\item[Operands] The permitted operand kinds and addressing modes for the instruction."
- evidence: facts.md Open question 11: the handover (HO:17) fixes seven addressing modes, but the parser distinguishes ten syntactic operand forms (sirc-vm/toolchain/src/parsers/instruction.rs:85-106) and the seven-way grouping in the digest is inferred, not cited
- resolution: manual-wrong
- fix: Question for the author: which fixed set of addressing-mode names may an Operands field use, and does the count of seven include the (A) and (A)+ shorthands as separate modes? Chapter 11 should name the closed set (or point at the Chapter 8 table by label) so that entries in Chapters 13-17 cannot invent mode names.
- confidence: low
- status: phase4
- ruling: Gate 2 N: apply after the ch8 editor settles the addressing-mode count.

### F-sum-8
- file: chapters/11-reading-instructions.tex
- lines: 97-98
- severity: major
- category: contradiction
- claim: "When no override is written, ALU instructions use \texttt{[A]}."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_immediate.rs:97-98 defaults to StatusRegisterUpdateSource::Alu, but :152-155 defaults SHFT to StatusRegisterUpdateSource::Shift; SHFT is an ALU-class encoding (OrShortImmediate 0x25, arithmetic_immediate.rs:40); chapters/13-alu-instructions.tex:215 says SHFT flags are "Updated from the shifter result by default"
- resolution: manual-wrong
- fix: Append "The one exception is \mnemonic{SHFT}, which defaults to \texttt{[S]}." after the sentence.
- confidence: high
- status: accepted

### F-sum-9
- file: chapters/11-reading-instructions.tex
- lines: 31-47
- severity: major
- category: fact
- claim: "\texttt{rD} & Destination general-purpose register ... \texttt{shift} & Optional SIRCIS shift definition"
- evidence: the notation table defines only rD, rS, rS1, rS2, #imm16, #imm8, shift and SR.X, but the instruction entries it governs use undefined placeholders: chapters/14-memory-instructions.tex:130-131 uses `addr` and `#offset`; chapters/15-control-flow.tex:139-141 uses `#disp`, `@label` and `|cond`; :290-293 uses `dest`, `src`, `rS`. None of these names appears in the table, and nothing maps `addr`/`src` to the 2-bit address-register-pair field (facts.md Registers section; FD:198-201)
- resolution: manual-wrong
- fix: Add rows for `A`/`src` (source address register pair l, a, s, p, encoded in the AF field for memory-class opcodes), `B`/`dest` (destination address register pair, encoded in the register field), `#offset`/`#disp` (displacement added to the low word of the pair), `@label` (symbol reference; PC-relative for BRAN/BRSR, lower word otherwise) and `|cond` (condition suffix). Alternatively pick one spelling and make Chapters 14 and 15 use it.
- confidence: high
- status: accepted

### F-con-10
- file: chapters/12-instruction-summary.tex
- lines: 6-7
- severity: major
- category: contradiction
- claim: "The SIRCIS instruction set consists of 64 possible opcodes (6-bit opcode field), of which approximately 52 are documented and assigned."
- evidence: chapters/appendix-c-undocumented.tex:6 "The SIRCIS instruction set includes 14 undocumented instruction opcodes" and appendix-c-undocumented.tex:211 "14 undocumented opcodes"; its table (appendix-c-undocumented.tex:46-59) lists 14 rows. 64 - 14 = 50, not 52. Separately, chapters/appendix-a-opcode-map.tex:134-138 lists only **12** undocumented opcodes (0x08, 0x09, 0x0B, 0x0D, 0x28, 0x29, 0x2B, 0x2D, 0x38, 0x39, 0x3B, 0x3D), omitting 0x27 and 0x2F even though its own table marks both "Undocumented" (appendix-a-opcode-map.tex:62, :70) and its quick-lookup footnote (appendix-a-opcode-map.tex:169) calls them undocumented.
- resolution: manual-wrong
- fix: Change 12-instruction-summary.tex:6-7 to "of which 50 are documented and assigned". Add a fourth bullet to appendix-a-opcode-map.tex:134-138: "\item 0x27, 0x2F (no public assembly syntax)" so the appendix agrees with the 14 listed in Appendix C.
- confidence: high
- status: accepted

### F-con-15
- file: chapters/12-instruction-summary.tex
- lines: 29-31
- severity: major
- category: contradiction
- claim: "0x04            & ANDI              & Immediate       & NZ             & AND immediate"
- evidence: The logical instructions also *clear* C and V, which the "NZ" column hides: chapters/13-alu-instructions.tex:150-154 "AND & * & * & 0 & 0"; chapters/13-alu-instructions.tex:135 "Logical (AND, ORR, XOR, TSA, TSX)] Update N and Z; clear C and V"; chapters/05-status-register.tex:180 "Logical Instructions (ANDI, ORRI, XORI): Update Z and N flags; C and V flags are cleared." The same understatement is repeated in chapters/appendix-e-quick-reference.tex:28,29,68,69,85,86,87,88,94,95.
- resolution: manual-wrong
- fix: Change the Flags cell for every logical/test-logical row to `NZ (C,V=0)` in 12-instruction-summary.tex:29,30,31,37,39,63,64,65,71,73,80,81,82,88,90 and in appendix-e-quick-reference.tex:28,29,68,69,85,86,87,88,94,95, and add a legend line under each table pointing at `tab:flag-effect-symbols`.
- confidence: high
- status: accepted

### F-sum-12
- file: chapters/12-instruction-summary.tex
- lines: 6-7
- severity: major
- category: fact
- claim: "The SIRCIS instruction set consists of 64 possible opcodes (6-bit opcode field), of which approximately 52 are documented and assigned."
- evidence: exactly 50 opcodes have an assembler mnemonic (0x00-0x07, 0x0A, 0x0C, 0x0E, 0x0F, 0x10-0x1F, 0x20-0x26, 0x2A, 0x2C, 0x2E, 0x30-0x37, 0x3A, 0x3C, 0x3E, 0x3F); the tables in this chapter mark 14 rows Undocumented (0x08, 0x09, 0x0B, 0x0D, 0x27, 0x28, 0x29, 0x2B, 0x2D, 0x2F, 0x38, 0x39, 0x3B, 0x3D), which gives 50, not 52
- resolution: manual-wrong
- fix: Replace with "of which 50 are documented and assigned; the remaining 14 are undocumented (see Appendix~\ref{appendix:undocumented})". A reference manual should not hedge a count with "approximately".
- confidence: high
- status: accepted

### F-sum-13
- file: chapters/12-instruction-summary.tex
- lines: 165-179
- severity: major
- category: fact
- claim: "Ordinary processing-unit instructions use exactly 6 execution phases and complete in 6 clock cycles when no bus wait states are required"
- evidence: every coprocessor call costs a second 6-cycle slot (the PU instruction, then the coprocessor dispatch at the following phase 0) — T-FAULT:781-791, T-SW:41-51, EEX:302-446; facts.md Coprocessors section. The chapter mentions extra cycles only for DMA, so a reader concludes that COPI, COPR, EXCP, WAIT, RETE, RSET, ETFR and ETTR are 6-cycle instructions
- resolution: manual-wrong
- fix: Add after the phase list: "Coprocessor calls (\mnemonic{COPI}, \mnemonic{COPR} and every meta-instruction that assembles to one) take a second 6-cycle slot for the coprocessor dispatch that begins at the next phase 0, for 12 cycles in total before the next instruction is fetched."
- confidence: high
- status: accepted

### F-sum-14
- file: chapters/12-instruction-summary.tex
- lines: 66, 74
- severity: major
- category: fact
- claim: "0x27 & LOAD & Short Imm+Shift & -- & Undocumented" and "0x2F & COPI & Short Imm+Shift & -- & Undocumented"
- evidence: facts.md Open question 10. 0x27 LoadRegisterFromShortImmediate is fully implemented (T-ASI:464-500) and 0x2F CoprocessorCallShortImmediate delivers an 8-bit command that can never be privileged (T-PROT:530-547); neither has an assembler form (A-AI:31-47, A-LOAD:73, A-COP:49-140). They are not in the same class as 0x08/0x09/0x0B/0x0D, which have no defined mnemonic at all, and this appendix treats them inconsistently: chapters/appendix-a-opcode-map.tex:62,70 calls them Undocumented, :134-138 omits them from the list of undocumented opcodes, and :169 says they "have no public assembly syntax and are undocumented"
- resolution: manual-wrong
- fix: Author decision needed: are 0x27 and 0x2F (a) undocumented like 0x08/0x09/0x0B/0x0D, (b) defined encodings with no assembler syntax, or (c) reserved? Whichever is chosen, make chapters/12-instruction-summary.tex:66,74, appendix-a-opcode-map.tex:62,70,134-138,159,163,169 and Appendix C agree, and drop the mnemonics LOAD/COPI from those rows if the encodings are declared undocumented.
- confidence: high
- status: accepted
- ruling: Gate 2 H: 0x27 and 0x2F are Undocumented, the same class as 0x08/0x09/0x0B/0x0D. Appendix C keeps them and its count of 14; ch12 and ch16 say "Undocumented" consistently.

### F-sum-15
- file: chapters/12-instruction-summary.tex
- lines: 32, 83
- severity: major
- category: fact
- claim: "0x07 & LOAD & Immediate & -- & Load immediate (move)" and "0x37 & LOAD & Register & -- & Load from register (move)"
- evidence: facts.md Open question 9. The hardware LOAD ALU operation with AF=Alu sets N and Z from the loaded value and clears C and V (alu.rs:307-317; T-AR:497-534 shows 0xFACE setting N); the assembler always emits AF=None (A-LOAD:47,128), and the immediate LOAD test uses AF=Shift and expects all flags cleared (T-AI:29-33, 521-573)
- resolution: manual-wrong
- fix: State which of the three behaviours is architectural. If "the LOAD opcodes update no flags" is a property of the assembler rather than of the encoding, the Flags cell should read "-- (AF=N as emitted by the assembler; see Chapter 13)". Same cell at chapters/appendix-e-quick-reference.tex:58.
- confidence: medium
- status: accepted
- ruling: Gate 2 C: LOAD never updates flags (row `- - - -`, default AF [N]); any explicit AF suffix on LOAD leaves all four flags undefined (`U U U U`). Say so once in ch5 and ch13 and cross-reference from ch12 and ch14.

### F-alu-2
- file: chapters/13-alu-instructions.tex
- lines: 820-828
- severity: major
- category: fact
- claim: "\textbf{Write-back:} None. The subtraction result is discarded after status flags are updated." ... "\textbf{Privilege:} Available in protected mode and supervisor mode."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:159 (`let des = immediate_representation.register;` — the destination field is decoded identically for every format and opcode); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:42-44 (privilege test is on `instruction.des` alone, with no reference to the write-back class); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:132-151 (0x08-0x0E are `AluStatusOnly`, no register write)
- resolution: manual-wrong
- fix: Add to the `Privilege:` line of the \mnemonic{CMP}, \mnemonic{TSA}, and \mnemonic{TSX} entries (lines 828, 905, 979): "The privilege check is made on the register field even though these instructions write no register, so \texttt{CMPI sr, \#0} in protected mode raises a privilege-violation fault." No test covers a flags-only opcode with a privileged register field, but the decode path is shared with the write-back forms.
- confidence: high
- status: accepted

### F-alu-5
- file: chapters/13-alu-instructions.tex
- lines: 554-555
- severity: major
- category: fact
- claim: "; Clear bit 5\nANDI r2, #~(1<<5)          ; r2 = r2 & 0xFFDF"
- evidence: sirc-vm/toolchain/src/parsers/shared.rs:99-100 (`parse_number_` = `#` followed by `parse_hex | parse_bin | parse_dec`), :44-96 (hex is `0x`+digits, binary is `0b`+digits, decimal is optional sign + digits). The assembler has no expression grammar, so `~`, `(`, and `<<` in an immediate are rejected.
- resolution: manual-wrong
- fix: Replace the listing line with `ANDI r2, #0xFFDF           ; r2 = r2 & 0xFFDF (clear bit 5)`.
- confidence: high
- status: accepted

### F-alu-6
- file: chapters/13-alu-instructions.tex
- lines: 155, 741
- severity: major
- category: fact
- claim: "LOAD            & -          & -          & -          & -          & \texttt{[N]}        & Status flags are preserved." / "\textbf{Flags:} Preserved."
- evidence: manual side — sirc-vm/toolchain/src/parsers/opcodes/load.rs:29-37 (the assembler rejects `[A|S|N]` for LOAD) and load.rs:47, :128 (`additional_flags: 0x0`, i.e. AF = None), so every assembled LOAD preserves flags. Code side — sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:307-317 (`perform_load` calls `set_alu_bits` with `Some((a, b, result))`) and sirc-vm/peripheral-cpu/tests/instructions/arithmetic_register_test.rs:497-534 (opcode 0x37 with `StatusRegisterUpdateSource::Alu` sets N from 0xFACE and clears C, V, Z), so the LOAD *encoding* with AF = Alu does update all four flags. A third behaviour is in sirc-vm/peripheral-cpu/tests/instructions/arithmetic_immediate_test.rs:521-573 (0x07 with AF = Shift clears all four). Digest Open question 9.
- resolution: manual-wrong
- fix: Decide which of the three is architectural and say so. If the flag behaviour of the LOAD encoding under a non-default AF is not architecturally fixed, the row must read `U U U U` with default AF `[N]` and a note: "Public \mnemonic{LOAD} syntax always encodes AF = None and preserves the flags. The result of encoding AF = Alu or AF = Shift on a LOAD opcode is architecturally undefined." If AF = Alu is architectural, the row must read `*  *  0  0` for that encoding. The same choice must be applied at line 114 (Table~\ref{tab:alu-common-semantics}), lines 78-79 (Table~\ref{tab:alu-legal-forms} "preserves flags"), line 732, and line 776.
- confidence: high
- status: accepted
- ruling: Gate 2 C: LOAD never updates flags (row `- - - -`, default AF [N]); any explicit AF suffix on LOAD leaves all four flags undefined (`U U U U`). Say so once in ch5 and ch13 and cross-reference from ch12 and ch14.

### F-alu-7
- file: chapters/13-alu-instructions.tex
- lines: 164-166
- severity: major
- category: fact
- claim: "If \texttt{[S]} is encoded, flags are updated from the shifter result instead of the ALU result; see Chapter~\ref{ch:shift-operations} for the shifter flag rules."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:172-176 and :188 — in the 16-bit immediate format the shift fields do not exist, decode forces `ShiftType::None` and `sr_shift = 0x0`; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:175-177 writes `sr_shift & SR_REDACTION_MASK` into the low byte, so `ADDI[S] r1, #100` clears N, Z, C, and V unconditionally. Confirmed by sirc-vm/peripheral-cpu/tests/instructions/arithmetic_immediate_test.rs:521-573.
- resolution: manual-wrong
- fix: Append to the sentence: "The 16-bit immediate format has no shift fields, so \texttt{[S]} on an immediate-format instruction writes a constant zero into the flag byte: N, Z, C, and V are all cleared. Use \texttt{[S]} only with short-immediate and register forms." The same qualification is missing from `09-shift-operations.tex:462` (the `[S]` row of Table~\ref{tab:status-override}).
- confidence: high
- status: accepted

### F-alu-8
- file: chapters/13-alu-instructions.tex
- lines: 239-240
- severity: major
- category: fact
- claim: "Because the lowering uses the short-immediate format, \mnemonic{SHFT} has the same shift field limits as other short-immediate ALU instructions."
- evidence: manual side — the limit is never stated numerically anywhere in this chapter. Code side — sirc-vm/toolchain/src/parsers/instruction.rs:224-236 rejects an immediate shift count above `MAX_SHIFT_COUNT` (15, definitions.rs:43); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/encoding.rs:339-342 masks an immediate count to 4 bits at encode time; a *register* shift count is not limited at all (definitions.rs:85-89; fetch_and_decode.rs:60-69), and the shift functions clamp to 16 for LSL/LSR/ASL/ASR (alu.rs:372, 395, 425, 457) while rotates take the count modulo 16 (alu.rs:479, 495). Digest Open question 13 records that register counts above 15 are untested (arithmetic_register_test.rs:953-959).
- resolution: manual-wrong
- fix: State the limit explicitly and rule on register counts, for example: "Immediate shift counts are 0--15; the assembler rejects a larger literal. A register shift count is the full 16-bit register value; counts above 15 produce a fully-shifted result (zero, or the replicated sign bit for \mnemonic{ASR}) for the shift types and are taken modulo 16 for \mnemonic{RTL} and \mnemonic{RTR}." If the author does not wish to fix the rotate/shift asymmetry in the architecture, the manual must instead say that a register shift count greater than 15 is architecturally undefined. The same gap exists at `09-shift-operations.tex:440`.
- confidence: high
- status: accepted
- ruling: Gate 2 E: immediate shift counts are 0--15; a register shift count above 15 is architecturally undefined. Do not document the clamp/modulo behaviour.

### F-alu-9
- file: chapters/13-alu-instructions.tex
- lines: 95-97
- severity: major
- category: fact
- claim: "When a register-form ALU instruction omits \texttt{rS1}, the assembler encodes \texttt{rD} as both the destination and first source operand. For example, \texttt{ADDR r1, r2} is encoded as \texttt{ADDR r1, r1, r2}."
- evidence: manual side — the chapter correctly states elsewhere (lines 110-112) that the shift applies to the left operand, but the shorthand note never says what a shift does when combined with the shorthand. Code side — sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:135-155 parses `ADDR rD, rS, shift` as r1 = r2 = rD, r3 = rS, so `ADDR r1, r2, LSL #2` computes `(r1 << 2) + r2`: the shift applies to the *destination* register, not to the named source. Digest Open question 16 asks proofreaders to confirm the operand-order wording; the shorthand-plus-shift case is the one form the chapter does not cover, and Table~\ref{tab:alu-legal-forms} lists only `ADDR rD, rS1, rS2[, shift]`.
- resolution: manual-wrong
- fix: Add a row `ADDR rD, rS2[, shift]` to Table~\ref{tab:alu-legal-forms} and extend the shorthand note: "A shift definition may be combined with the shorthand; because the shift always applies to the left operand, \texttt{ADDR r1, r2, LSL \#2} computes \texttt{(r1 << 2) + r2}, not \texttt{r1 + (r2 << 2)}." Confirm with the author that this is the intended reading before applying.
- confidence: medium
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-con-16
- file: chapters/13-alu-instructions.tex
- lines: 155
- severity: major
- category: contradiction
- claim: "LOAD            & -          & -          & -          & -          & \texttt{[N]}        & Status flags are preserved."
- evidence: chapters/11-reading-instructions.tex:97 "When no override is written, ALU instructions use \texttt{[A]}." LOAD is an ALU-family instruction (chapters/13-alu-instructions.tex:33-36 lists it under "Data Movement"; opcodes 0x07/0x37 sit inside the ALU opcode ranges named at chapters/12-instruction-summary.tex:10).
- resolution: manual-wrong
- fix: Amend chapters/11-reading-instructions.tex:97 to "When no override is written, ALU instructions use \texttt{[A]}, except \mnemonic{LOAD}, which always encodes \texttt{[N]} and never updates the status flags." (Chapter 13 line 166 already states the LOAD exception; Chapter 11 must not contradict it.)
- confidence: high
- status: accepted

### G1-T1-13
- file: chapters/13-alu-instructions.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Rename the `Condition codes:` line in every instruction entry to `Condition field:` (period-gaps item 3).
- confidence: high
- status: accepted

### G1-T2-13
- file: chapters/13-alu-instructions.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Add an applicability line under every instruction entry title: `SIRC-1, all revisions`, or the coprocessor revision per Chapter 16 (period-gaps item 13).
- confidence: high
- status: accepted

### G1-T3-13
- file: chapters/13-alu-instructions.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Add an Instruction Fields list after the opcode list in every entry: one line per encoding field with its legal values here; values from review/facts.md and Chapter 7 only (period-gaps item 2).
- confidence: high
- status: accepted

### F-con-12
- file: chapters/14-memory-instructions.tex
- lines: 31-41
- severity: major
- category: contradiction
- claim: "\text{EA} = \text{AddressRegister} + \text{Offset} ... The resulting 24-bit address is used to access memory."
- evidence: The same chapter's own table says the addition is low-word-only: chapters/14-memory-instructions.tex:80 "Immediate-offset forms add the 16-bit encoded offset to the selected address register low word ... The selected address register high word supplies the segment." chapters/08-addressing-modes.tex:201-203 gives the correct form "EA = AddressRegister.high : (AddressRegister.low + Offset)"; chapters/04-data-representation.tex:153-155 "Address calculation ... adds offsets to the low word of the selected address register pair. The high address-register word is not changed"; chapters/08-addressing-modes.tex:120-122.
- resolution: manual-wrong
- fix: Replace the equation with `\text{EA} = \text{AddressRegister.high} : (\text{AddressRegister.low} + \text{Displacement})` and change "The resulting 24-bit address is used to access memory." to "The high word supplies the segment and is not changed by the displacement; only the low word participates in the addition."
- confidence: high
- status: accepted

### F-con-22
- file: chapters/14-memory-instructions.tex
- lines: 15-21
- severity: major
- category: contradiction
- claim: "Both instructions support four memory addressing forms: ... Post-increment load: \texttt{(\#offset, addr)+} or \texttt{(reg, addr)+} ... Pre-decrement store: \texttt{-(\#offset, addr)} or \texttt{-(reg, addr)}"
- evidence: The same chapter says the opposite 50 lines later: chapters/14-memory-instructions.tex:68-69 "\mnemonic{LOAD} has no pre-decrement form and \mnemonic{STOR} has no post-increment form."; the legal-forms table (chapters/14-memory-instructions.tex:52-59) lists four forms for LOAD and four for STOR, not the same four; chapters/08-addressing-modes.tex:95-97 states the same asymmetry.
- resolution: manual-wrong
- fix: Change the lead-in to "\mnemonic{LOAD} and \mnemonic{STOR} each support four memory addressing forms. Both support indirect immediate-displacement and indirect register-displacement forms; \mnemonic{LOAD} additionally supports post-increment and \mnemonic{STOR} additionally supports pre-decrement." and re-label the last two bullets accordingly.
- confidence: high
- status: accepted

### F-con-23
- file: chapters/14-memory-instructions.tex
- lines: 95
- severity: major
- category: contradiction
- claim: "Documented memory forms do not raise privilege-violation or invalid-opcode faults."
- evidence: chapters/06-exceptions.tex:96 lists "Address-register-pair write-back that would change a high address-register word" as a privilege-violation trigger; chapters/08-addressing-modes.tex:412-413 "\textbf{Address-register-pair write-back}: Allowed only when the high word would remain unchanged; otherwise triggers a privilege violation fault \\ \textbf{Post-increment/Pre-decrement}: Allowed in protected mode only when the high word would remain unchanged"; chapters/03-registers.tex:209-211. Post-increment LOAD and pre-decrement STOR are address-register-pair write-backs. Chapter 15 states the rule explicitly for its instructions (chapters/15-control-flow.tex:94); Chapter 14 denies it.
- resolution: manual-wrong
- fix: Either add the privilege rule to Chapter 14 (a `Privilege` row in `tab:memory-common-semantics` mirroring chapters/15-control-flow.tex:94, and an amended line 95), or state in one place why auto-update on memory instructions can never change the high word and therefore never faults. The current flat denial cannot stand alongside Chapter 6, 8 and 3.
- confidence: medium
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-mem-2
- file: chapters/14-memory-instructions.tex
- lines: 104-105
- severity: major
- category: contradiction
- claim: "Memory instructions do not support status update override syntax; the status override field is ignored."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:198-199 and :217 (the AF field is decoded simultaneously as `sr_src` and as the address-register pair index); sirc-vm/toolchain/src/parsers/opcodes/store.rs:48 and load.rs:58 (the assembler puts the address register in `additional_flags`)
- resolution: manual-wrong
- fix: Replace the second clause with: "the AF field (bits 5--4) is not a status update source for these opcodes; it selects the address-register pair used to form the effective address." As written, an implementer or assembler author could conclude that AF is a don't-care for memory opcodes and encode zero, which silently retargets every memory access to \reg{l}.
- confidence: high
- status: accepted

### F-mem-3
- file: chapters/14-memory-instructions.tex
- lines: 29-41
- severity: major
- category: fact
- claim: "\text{EA} = \text{AddressRegister} + \text{Offset}" ... "The resulting 24-bit address is used to access memory."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:77-78 (`decoded.ad_l_.overflowing_add(decoded.sr_b_)`, a 16-bit add); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/memory_access.rs:58 and :67 (`(decoded.ad_h_, alu_output).to_full_address()`, the high word is taken unmodified)
- resolution: manual-wrong
- fix: The displacement is added to the low word only, modulo 2^16; the high word of the address register is concatenated unchanged and carry never propagates into it. Replace the equation with two lines, e.g. `EA[23:16] = AddressRegister.high` and `EA[15:0] = AddressRegister.low + Offset (mod 2^16)`, and state that a memory access can never leave the 64K segment selected by the high word. The table row at line 80 says this correctly; the equation contradicts it.
- confidence: high
- status: accepted

### F-mem-4
- file: chapters/14-memory-instructions.tex
- lines: 142-144
- severity: major
- category: fact
- claim: "\texttt{rD} is the destination general-purpose register. \texttt{addr} is the address-register pair that supplies the segment and base low word."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/load.rs:240-246, :297-303, :331-337 and store.rs:160-166, :217-223 (`reject_aliased_address_register_write`); sirc-vm/toolchain/tests/assembler/control_flow_test.rs:305-308 (`LOAD al, (#0, a)+`, `LOAD ah, (r3, a)+`, `STOR -(#0, a), al`, `STOR -(r3, a), ah` all fail to assemble)
- resolution: manual-wrong
- fix: Chapter 14 never states the aliased-write restriction that Chapter 15 states for control flow. Add a normative sentence to the Legal Forms section: "In post-increment \mnemonic{LOAD} forms the destination register must not be either half of the auto-updated address-register pair; in pre-decrement \mnemonic{STOR} forms the source register must not be either half of the auto-updated pair. The result of such an encoding is architecturally undefined and the assembler rejects it."
- confidence: high
- status: accepted

### F-mem-6
- file: chapters/14-memory-instructions.tex
- lines: 94-95
- severity: major
- category: fact
- claim: "Effective-address calculation can raise a segment-overflow fault when \texttt{SR.A} (Trap on Address Overflow) is set and the 16-bit low-word address calculation wraps."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:77-78 and :110-115 (the overflow test is an unsigned 16-bit add of the raw displacement field); docs/reference/review/facts.md Open question 8
- resolution: manual-wrong
- fix: The manual never says whether the 16-bit displacement is signed. Because the trap fires on unsigned carry, a "negative" displacement such as `LOAD r1, (#-2, a)` faults with SR.A set whenever `al >= 2` -- that is, negative displacements are unusable with the trap enabled. State this explicitly (either "displacements are unsigned and SR.A traps any carry out of bit 15" or "negative displacements must not be used while SR.A is set"), and give the same warning for backward \mnemonic{BRAN}/\mnemonic{BRSR} displacements in Chapter 15.
- confidence: medium
- status: accepted
- ruling: Gate 2 D: add one sentence that displacements are signed two's-complement and that the trap fires only when the low-word calculation wraps (the reference implementation currently traps on unsigned carry; that is an implementation bug, do not document it).

### F-mem-7
- file: chapters/14-memory-instructions.tex
- lines: 86
- severity: major
- category: fact
- claim: "If a fault aborts execution before write-back, those write-back effects do not occur."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:73-76 (the whole phase is skipped once a fault is pending) confirms the quoted sentence; but sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:684-693 shows the fault return address for a faulting LOAD is the *following* instruction (`0x00AB_CDE2` for an instruction at `0x00AB_CDE0`)
- resolution: manual-wrong
- fix: The statement is correct but incomplete in the way an implementer needs. Add: "A memory instruction aborted by a fault is not restartable: the saved return address is the address of the next instruction, so returning from the handler resumes after the faulting access." (See docs/reference/review/facts.md Open question 1 for the general abort-fault return-address question, which the exception chapter must settle.)
- confidence: medium
- status: accepted
- ruling: Gate 2 A: document actual behaviour. Retryable class is Alignment and PC-wrap Segment Overflow only; Bus, Bus Protection, Invalid Opcode and Privilege Violation save the next instruction address and are not restartable.

### F-mem-8
- file: chapters/14-memory-instructions.tex
- lines: 103-114
- severity: major
- category: fact
- claim: "Memory instructions do not update status flags. All four condition flags are preserved regardless of addressing form, shift result, or auto-update side effect." (table row: "LOAD & - & - & - & -")
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:310-316 (memory loads never call `update_status_flags`) supports the claim for opcodes 0x14--0x17; but sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:307-317 and tests/instructions/arithmetic_register_test.rs:497-534 show the non-memory `LOAD` op with AF=Alu sets N and Z; docs/reference/review/facts.md Open question 9
- resolution: manual-wrong
- fix: The table row is labelled only "LOAD", which a reader will apply to `LOAD rD, #imm16` (0x07) and `LOAD rD, rS` (0x37) as well. Relabel the rows "LOAD (memory forms 0x14--0x17)" and "STOR (0x10--0x13)", and add a cross-reference to whichever chapter rules on LOAD-immediate/LOAD-register flag behaviour. The three observable behaviours (hardware AF=Alu sets N/Z; assembler always emits AF=None; the immediate test uses AF=Shift and clears everything) must be settled once, manual-wide.
- confidence: medium
- status: accepted
- ruling: Gate 2 C: LOAD never updates flags (row `- - - -`, default AF [N]); any explicit AF suffix on LOAD leaves all four flags undefined (`U U U U`). Say so once in ch5 and ch13 and cross-reference from ch12 and ch14.

### F-mem-9
- file: chapters/14-memory-instructions.tex
- lines: 296
- severity: major
- category: fact
- claim: "loop:"
- evidence: sirc-vm/toolchain/src/parsers/shared.rs:112 (`parse_label_` is `preceded(char(':'), parse_label_name_)`, i.e. the colon is a *prefix*); examples/byte-sieve/byte-sieve.sasm:10 (`:main`)
- resolution: manual-wrong
- fix: Label definitions in SIRC assembly are written `:loop`, not `loop:`. Every trailing-colon label in these chapters is rejected by the assembler. Occurrences: 14-memory-instructions.tex:296; 15-control-flow.tex:177, 276, 529, 532, 538, 544, 554.
- confidence: high
- status: accepted

### G1-T1-14
- file: chapters/14-memory-instructions.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Rename the `Condition codes:` line in every instruction entry to `Condition field:` (period-gaps item 3).
- confidence: high
- status: accepted

### G1-T2-14
- file: chapters/14-memory-instructions.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Add an applicability line under every instruction entry title: `SIRC-1, all revisions`, or the coprocessor revision per Chapter 16 (period-gaps item 13).
- confidence: high
- status: accepted

### G1-T3-14
- file: chapters/14-memory-instructions.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Add an Instruction Fields list after the opcode list in every entry: one line per encoding field with its legal values here; values from review/facts.md and Chapter 7 only (period-gaps item 2).
- confidence: high
- status: accepted

### F-mem-13
- file: chapters/15-control-flow.tex
- lines: 60-62
- severity: major
- category: fact
- claim: "\texttt{src} and \texttt{dest} are address-register pairs: \reg{l}, \reg{a}, \reg{s}, or \reg{p}."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/ldel.rs:51-57, :106-113, :135-142, :206-213 (every \mnemonic{LDEL} form rejects `l` as the destination, not just the post-increment forms); sirc-vm/toolchain/tests/assembler/control_flow_test.rs:299 (`LDEL l, (#0, a)` is an error)
- resolution: manual-wrong
- fix: Add: "\reg{l} is never a legal \mnemonic{LDEL} destination, because \mnemonic{LDEL} always writes the return address to \reg{l}; the destination and the link write would alias. \mnemonic{LDEL} destinations are \reg{a}, \reg{s}, or \reg{p}." The Legal Forms rows at lines 37-40 need the same restriction, since as written they admit `LDEL l, (#offset, src)`.
- confidence: high
- status: accepted

### F-mem-14
- file: chapters/15-control-flow.tex
- lines: 71-73
- severity: major
- category: fact
- claim: "Instructions that write the same address-register pair through more than one write-back path have architecturally undefined behavior. Avoid forms such as \texttt{LDEA a, -(\#0, a)} and \texttt{LDEL l, (\#0, l)+}. The assembler rejects known hazardous aliased forms."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/ldea.rs:135-141, :191-197; ldel.rs:51-57, :135-157, :206-228; ljsr.rs:154-167, :214-227; load.rs:240-246; store.rs:160-166; tests/assembler/control_flow_test.rs:295-310; docs/reference/review/facts.md Open question 15
- resolution: manual-wrong
- fix: "Known hazardous" is not implementable: an assembler author cannot derive the rejection list, and a CPU implementer cannot tell which encodings must be diagnosed. Replace with the exhaustive list the toolchain enforces: \mnemonic{LDEA} pre-decrement with destination equal to source; \mnemonic{LDEL} with destination \reg{l} (any form); \mnemonic{LDEL} post-increment with source \reg{l} or with destination equal to source; \mnemonic{LJSR} post-increment from \reg{l} or \reg{p}; \mnemonic{LOAD} post-increment whose destination is a half of the auto-updated pair; \mnemonic{STOR} pre-decrement whose source is a half of the auto-updated pair. Keep the "architecturally undefined" label rather than describing the order the current implementation happens to use (write_back.rs:216-236 applies the source update first and the destination write last).
- confidence: high
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-mem-15
- file: chapters/15-control-flow.tex
- lines: 506
- severity: major
- category: fact
- claim: "LJSR a, #function_offset      ; p = a + function_offset"
- evidence: sirc-vm/toolchain/src/parsers/shared.rs:99 (`parse_number_` = `#` followed by a hex, binary, or decimal literal only), :138 (symbol references are `@name`), :103 (placeholders are `$name`)
- resolution: manual-wrong
- fix: `#identifier` is not assembler syntax; the listing does not assemble. Use `LJSR a, @function_offset` (a symbol reference, resolved as the symbol's lower word by ljsr.rs:108-119) or define the constant with `.EQU $function_offset #N` and write `LJSR a, $function_offset`.
- confidence: high
- status: accepted

### F-mem-16
- file: chapters/15-control-flow.tex
- lines: 550-552
- severity: major
- category: fact
- claim: "ADDI r2, #0, LSL #1           ; r2 = (r2 << 1) = r2 * 2 (word size)" / "LDEA p, (r2, p)               ; Jump via table"
- evidence: sirc-vm/peripheral-cpu/tests/instructions/ljmp_test.rs:224-232 (the PC-relative base is the \mnemonic{LDEA}'s own address, not the following word)
- resolution: manual-wrong
- fix: The example does not work. `LDEA p, (r2, p)` computes `p = address_of_this_LDEA + r2`, and `jump_table` begins two words later, so index 0 jumps back to the \mnemonic{LDEA} itself (an infinite loop) and every other index lands two words short. The scaling is also wrong for the table content: the entries are one-word data words, so an index must be scaled by 1 to address them and by 2 only if the table holds branch instructions. Rewrite the example so the table holds two-word branch instructions (`ADDI r2, #2, LSL #1` to skip the \mnemonic{LDEA}, table of `BRAN @caseN`), or load the displacement first with `LOAD r3, (r2, p)` and then branch.
- confidence: high
- status: accepted

### F-mem-17
- file: chapters/15-control-flow.tex
- lines: 554-558
- severity: major
- category: fact
- claim: "    DW case0 - jump_table"
- evidence: sirc-vm/toolchain/src/types/data.rs:5-7 (`DB_TOKEN = ".DB"`, `DW_TOKEN = ".DW"`, `DQ_TOKEN = ".DQ"`); sirc-vm/toolchain/src/parsers/data.rs:36-47, :49-51 (a data directive operand is a `#`-prefixed number or an `@` symbol reference -- there is no expression grammar)
- resolution: manual-wrong
- fix: The directive is `.DW` (leading dot) and `case0 - jump_table` is not parseable: the assembler has no arithmetic on symbols. Use `.DW @case0` (which the linker resolves) or hand-written literals, and note that a `.DW` symbol reference resolves as a full address, not a difference.
- confidence: high
- status: accepted

### F-mem-18
- file: chapters/15-control-flow.tex
- lines: 94, 330-331
- severity: major
- category: fact
- claim: "In protected mode, address-register-pair write-back is allowed only when every written high word would remain unchanged. If any high word would change, the instruction raises a privilege-violation fault instead of completing write-back."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:21-27 and :42-43 (the decode-time check tests `instruction.des` against register indexes 0, 8, 10, 12, 14); fetch_and_decode.rs:158 and :200-201 (for \mnemonic{LDEA}/\mnemonic{LDEL} the register field is the 2-bit destination *pair* index, 0 = \reg{l}, so `des == 0` collides with the \reg{sr} index used by the privilege check). No test covers `LDEA l, (...)` in protected mode (tests/instructions/protected_mode_test.rs:152-438 uses destinations \reg{a} and \reg{p} only).
- resolution: manual-wrong
- fix: As implemented, any \mnemonic{LDEA} whose destination pair is \reg{l} (register field 0) raises a privilege-violation fault in protected mode before write-back, regardless of the high words -- an extra rule the chapter does not state. Either the decode-time check must exclude the effective-address opcode class (0x18--0x1F), in which case the chapter is right and the code is wrong, or the chapter must add "\mnemonic{LDEA} with destination \reg{l} is privileged." Present both sides to the author; do not publish either wording until it is settled.
- confidence: medium
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### G1-T1-15
- file: chapters/15-control-flow.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Rename the `Condition codes:` line in every instruction entry to `Condition field:` (period-gaps item 3).
- confidence: high
- status: accepted

### G1-T2-15
- file: chapters/15-control-flow.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Add an applicability line under every instruction entry title: `SIRC-1, all revisions`, or the coprocessor revision per Chapter 16 (period-gaps item 13).
- confidence: high
- status: accepted

### G1-T3-15
- file: chapters/15-control-flow.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Add an Instruction Fields list after the opcode list in every entry: one line per encoding field with its legal values here; values from review/facts.md and Chapter 7 only (period-gaps item 2).
- confidence: high
- status: accepted

### F-con-20
- file: chapters/16-coprocessor-instructions.tex
- lines: 366-368
- severity: major
- category: contradiction
- claim: "bits 7--6 select the address register (\texttt{00} = \reg{a}, \texttt{01} = \reg{l}, \texttt{10} = \reg{s}, \texttt{11} reserved)"
- evidence: The architectural 2-bit address-register-pair encoding is `00 = l, 01 = a, 10 = s, 11 = p` — chapters/08-addressing-modes.tex:345-348 (`tab:addr-reg-encoding`), chapters/07-instruction-formats.tex:84-87 and :208-211. The DMA operand byte swaps `l` and `a` with no note that it is a different field with a different assignment, which is an easy trap for an implementer.
- resolution: manual-wrong
- fix: If the DMA operand really uses a different assignment, add an explicit warning sentence after line 368: "Note: this DMA operand field is not the AF address-register field; its \texttt{00}/\texttt{01} assignment differs from Table~\ref{tab:addr-reg-encoding}." If it does not, change the encoding to `00 = l, 01 = a, 10 = s, 11 = reserved`.
- confidence: medium
- status: accepted
- ruling: Gate 2 M: keep the DMA register-field encoding; add the explicit warning that this field is not the AF address-register field.

### F-con-56
- file: chapters/16-coprocessor-instructions.tex
- lines: 293-306
- severity: major
- category: structure
- claim: "\textbf{Assembles to:} \texttt{COPI \#0x1Cxy} or \texttt{COPI \#0x1Dxy} ... \texttt{\#n} selects an exception link register. Optional operands select whether to transfer the saved return address, saved status register, or both."
- evidence: The `xy` operand byte of the ETFR/ETTR command word is never defined anywhere in the manual. Contrast the DMA operand byte, which is fully specified at chapters/16-coprocessor-instructions.tex:366-370. chapters/16-coprocessor-instructions.tex:185-186 gives the ranges 0x1C00--0x1C7F and 0x1D00--0x1D7F without a field breakdown, and chapters/06-exceptions.tex:308 and :645-650 describe the instructions only in prose. An implementer cannot encode `ETFR a, #6` from this manual.
- resolution: manual-wrong
- fix: Add a paragraph to the ETFR/ETTR entry, in the same shape as the DMA operand paragraph, specifying which bits of the low byte hold the link-register index (0--7) and which bits select address-only / status-only / both.
- confidence: high
- status: accepted

### F-cop-10
- file: chapters/16-coprocessor-instructions.tex
- lines: 231
- severity: major
- category: fact
- claim: "Puts the CPU into wait state until an exception occurs."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:295-302 (the wake-up path is `raise_hardware_interrupt` followed by an early return while `waiting_for_exception` is set); sirc-vm/peripheral-cpu/src/lib.rs:499-518 (`raise_hardware_interrupt` clears `waiting_for_exception` only for lines that pass the `HIE` enable mask); sirc-vm/peripheral-cpu/src/lib.rs:530 (`reset()` clears it); sirc-vm/peripheral-cpu/tests/exceptions/reset.rs:227-248. While waiting, the CPU issues no bus cycles, so no instruction, fault, or software exception can occur.
- resolution: manual-wrong
- fix: Replace with "Stops instruction fetch and places the CPU in a low-power wait state. The CPU issues no bus cycles while waiting. It leaves the wait state only when an enabled hardware interrupt line is asserted (NMI is always enabled) or when the CPU is reset. Interrupts masked by the \texttt{HIE} bits do not wake the CPU, and faults and software exceptions cannot occur while waiting."
- confidence: high
- status: accepted

### F-cop-11
- file: chapters/16-coprocessor-instructions.tex
- lines: 46
- severity: major
- category: fact
- claim: "The instruction is skipped and has no side effects. The pending coprocessor command, registers, memory, address registers, program counter, and status flags are preserved."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:151 (`registers.pl = self.decoded_instruction.npc_l_;` runs at decode unconditionally, before and independently of the condition check); digest facts.md:182 lists the condition-gated effects as "register, flag, memory, or address-register", not the PC
- resolution: manual-wrong
- fix: Remove "program counter" from the preserved list and replace the sentence with "The instruction is skipped and has no side effects. The pending coprocessor command, registers, memory, address registers, and status flags are preserved. The program counter still advances to the following instruction." The same over-broad wording ("all CPU state is preserved") appears at lines 374, 427, 475 and 521 and should be narrowed the same way.
- confidence: high
- status: accepted

### F-cop-12
- file: chapters/16-coprocessor-instructions.tex
- lines: 52
- severity: major
- category: fact
- claim: "The coprocessor-call instruction itself uses the normal 6 execution phases and performs no data-memory bus access. The selected coprocessor may perform follow-up work after the command is latched."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:302-446 (the exception unit's dispatch runs through all six `ExecutionPhase` values: vector fetch at phase 0, second word at phase 1, decode at phase 2, execute at phase 3, cleanup at phase 5); sirc-vm/peripheral-cpu/tests/exceptions/common.rs:125-167; digest facts.md:286, 317, 327 ("A software exception therefore starts its handler 12 cycles after the EXCP fetch began")
- resolution: manual-wrong
- fix: State the fixed cost normatively: "The coprocessor-call instruction itself uses the normal 6 execution phases and performs no data-memory bus access. Dispatch to coprocessor 1 then occupies one further complete 6-cycle slot beginning at the next phase 0, so a coprocessor-1 operation completes 12 cycles after the call's own fetch began." Then replace the vague "6 cycles before ..." timings at lines 213, 241, 264, 287 and 320 with "6 cycles for the call plus a 6-cycle exception-unit dispatch slot".
- confidence: high
- status: accepted

### F-cop-13
- file: chapters/16-coprocessor-instructions.tex
- lines: 376-382
- severity: major
- category: fact
- claim: "The \mnemonic{COPI} dispatch takes the normal 6 cycles, then the DMA unit gets at least one post-dispatch cycle. ... Therefore \mnemonic{DMAR} and \mnemonic{DMAW} take $6 + \max(|n|, 1)$ cycles, and \mnemonic{DMAT} takes $6 + \max(2n, 1)$ cycles."
- evidence: manual side — this passage and the derived per-entry timings at lines 429--431, 477--479 and 523--527; code side — no DMA coprocessor exists in `sirc-vm/peripheral-cpu` (sirc-vm/peripheral-cpu/src/lib.rs:375-394 faults on coprocessor ID 2), and the only implemented coprocessor dispatch (the exception unit, exception_unit/execution.rs:302-446) always consumes a full 6-cycle slot rather than "at least one post-dispatch cycle". Digest facts.md:329 states DMA burst cycle counts are not derivable from code.
- resolution: manual-wrong
- fix: Either state that coprocessor 2 dispatch, unlike coprocessor 1, does not consume a whole 6-cycle slot (and say why), or restate the formulas on the 6-cycle-slot model (for example $6 + 6 + \ldots$). As written the chapter uses two incompatible dispatch-cost models for the same COP mechanism, and an emulator author cannot tell which applies.
- confidence: medium
- status: accepted
- ruling: Gate 2 M: restate DMA dispatch timing on the 6-cycle-slot model used for coprocessor 1.

### F-cop-14
- file: chapters/16-coprocessor-instructions.tex
- lines: 141
- severity: major
- category: contradiction
- claim: "0x2         & DMA Unit             & Yes               & 0x8--0xA            & Required standard DMA transfer unit"
- evidence: manual side — this row, plus line 340 "The required DMA unit" and line 128 "A CPU model may omit an optional coprocessor"; code side — sirc-vm/peripheral-cpu/src/lib.rs:375-394 dispatches only coprocessor IDs 0 and 1 and raises InvalidOpCode for every other ID, so the reference implementation is a conforming SIRC-1 that omits a "required" coprocessor; digest facts.md:313
- resolution: code-wrong
- fix: Either mark the DMA unit "No" (optional, present on models that document it) to match the reference implementation, or record that the reference emulator is non-conformant. A "Required" coprocessor that the reference CPU faults on makes the probe-by-invalid-opcode-fault advice at lines 157--158 meaningless for DMA.
- confidence: medium
- status: not-implemented
- ruling: Gate 2 M: the DMA unit is Required and planned; the reference emulator has not implemented it yet. Manual stands.

### F-cop-15
- file: chapters/16-coprocessor-instructions.tex
- lines: 61-67
- severity: major
- category: fact
- claim: "If trace mode was enabled when the instruction began, an instruction-trace fault is raised after the processing-unit call commits, unless an earlier fault prevents the instruction from completing."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:397-406 (the trace fault is raised at `WriteBackExecutor` when `pending_fault.is_none()`); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:146-160 (`get_cause_register_value` returns the *fault* cause in preference to `registers.pending_coprocessor_command`); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:444 (`registers.pending_coprocessor_command = 0x0` unconditionally at the dispatch's phase 5). The trace-fault dispatch therefore clears the just-latched coprocessor command and the coprocessor operation never runs. No test covers a coprocessor call in trace mode.
- resolution: manual-wrong
- fix: The manual must state what happens to a pending coprocessor command when a fault or hardware exception is dispatched at the same phase 0. Add to the Common Semantics table: "A pending coprocessor command is discarded if a fault or hardware exception is dispatched before the coprocessor's dispatch slot begins; the command is not re-issued after the handler returns." If the intended architecture is that the command survives, this is a code defect.
- confidence: medium
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-cop-16
- file: chapters/16-coprocessor-instructions.tex
- lines: 36
- severity: major
- category: fact
- claim: "Opcode 0x2F has no public assembly syntax and is undocumented."
- evidence: manual side — this line plus line 70 ("Opcode 0x2F is undocumented") and STYLE.md:25, which reserves "undocumented" for "a specific, currently-unassigned opcode with observed behavior"; code side — 0x2F is an assigned opcode `CoprocessorCallShortImmediate` with fully defined behaviour (sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs, `CoprocessorCallShortImmediate`), decoded like any short-immediate instruction and latching the zero-extended 8-bit immediate as the command (sirc-vm/peripheral-cpu/tests/instructions/protected_mode_test.rs:530-547 asserts `pending_coprocessor_command == 0x0019` for an encoded value of 0x19). Digest Open question 10 (facts.md:342).
- resolution: manual-wrong
- fix: Rule whether 0x2F is "undocumented" (App. C style, unassigned) or an assigned-but-unassembled encoding. If the latter, replace with: "Opcode 0x2F (short-immediate coprocessor call) has no assembly syntax. It latches the zero-extended 8-bit immediate as the command operand; the shift fields do not apply to the immediate, so the coprocessor ID is always 0x0 and the operation nibble is always 0x0. It can therefore never address a coprocessor and can never be privileged." Note that as written, "undocumented" here conflicts with the same word used for opcodes 0x08/0x09/0x0B/0x0D at line 159.
- confidence: high
- status: accepted
- ruling: Gate 2 H: 0x27 and 0x2F are Undocumented, the same class as 0x08/0x09/0x0B/0x0D. Appendix C keeps them and its count of 14; ch12 and ch16 say "Undocumented" consistently.

### F-cop-17
- file: chapters/16-coprocessor-instructions.tex
- lines: 276-277, 287
- severity: major
- category: fact
- claim: "Performs a software reset of the processor by clearing the status register and jumping to the reset vector." / "\textbf{Timing:} Same as \mnemonic{COPI}: 6 cycles before reset processing begins."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:422-431 (`reset_requested` asserted after write-back of the COPI); sirc-vm/peripheral-bus/src/reset_unit.rs:22-26 (RSTO held for the assertion cycle plus a 5-cycle countdown); sirc-vm/peripheral-cpu/tests/exceptions/reset.rs:84-147; sirc-vm/peripheral-cpu/src/lib.rs:524-536 (`reset()` clears CPU control state and seeds the Reset cause but leaves `r1`--`r7`, the `a`/`l`/`s` pairs and all link registers untouched); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:367-369 (`registers.sr = 0x0; registers.set_full_pc_address(self.vector_value);`). Digest Open question 12 (facts.md:344).
- resolution: manual-wrong
- fix: The Description must say what \mnemonic{RSET} leaves alone: "Requests a full processor reset. The reset output is asserted and held for 6 cycles, after which the exception unit fetches the reset vector, sets \reg{sr} to \texttt{0x0000} and loads the program counter from the vector. General-purpose registers, the \reg{a}, \reg{l} and \reg{s} pairs, and the exception link registers are not affected by reset and hold undefined values after power-on." The Timing line must add the 6-cycle RSTO hold before the vector fetch (total 6 + 6 + 6 cycles from the \mnemonic{RSET} fetch to the first instruction of the reset handler). Whether general registers are "unchanged" or "undefined" after power-on is a decision the architecture has not yet recorded.
- confidence: high
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-cop-18
- file: chapters/16-coprocessor-instructions.tex
- lines: 51, 106-107
- severity: major
- category: fact
- claim: "In protected mode, coprocessor-operation nibbles 0x0--0x7 are user-callable and 0x8--0xF are supervisor-only."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:46-48 (`let cop_opcode = instruction.sr_b_ & 0x0F00; let calling_privileged_cop_opcode = is_cop_instruction && (cop_opcode > 0x0700);`) — the rule is correct, but the check reads operand *b*: the 16-bit immediate for `COPI` and the value of \reg{rS} for `COPR`, sampled at decode. sirc-vm/peripheral-cpu/tests/instructions/protected_mode_test.rs:510-526 (a `COPR` whose source register holds `0x1900` faults).
- resolution: manual-wrong
- fix: Add the missing normative sentence to the Privilege row and to the COPI/COPR Privilege field: "For \mnemonic{COPR} the privilege decision is made on the run-time value of \reg{rS} sampled during decode, not on any encoded field, so whether a given \mnemonic{COPR} instruction faults depends on register contents." Without this an implementer may check the privilege rule against the encoding only.
- confidence: high
- status: accepted

### F-cop-26
- file: chapters/16-coprocessor-instructions.tex
- lines: 106-107
- severity: major
- category: fact
- claim: "\textbf{Privilege:} Available in protected mode and supervisor mode. In protected mode, coprocessor-operation nibbles 0x8--0xF are supervisor-only and raise privilege-violation faults when executed."
- evidence: manual side — this line covers nibbles 0xE and 0xF without exception; code side — sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs:158-161 marks 0xE `Fault` and 0xF `HardwareException` as the units' internal cause encodings, and sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:416-427 dispatches them through `handle_exception` exactly as if the hardware had raised them, so a supervisor-mode `COPI #0x1Fxx` fabricates a hardware exception at an arbitrary level and `COPI #0x1Exx` fabricates a fault. sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:1321-1323 marks this unresolved. Digest Open question 14 (facts.md:346).
- resolution: manual-wrong
- fix: The architecture must state whether coprocessor-1 operations 0xE and 0xF are software-callable. If they are internal only, the manual must say so and the exception unit must reject them from a software-issued command; if they are callable, their operand semantics (vector and level) must be documented. Either way, the "Operations 0x0--0xD" row at line 140 and this privilege statement must agree.
- confidence: high
- status: accepted
- ruling: Gate 2 K/L: exception-unit opcodes 0x2--0x8, software-issued 0xE/0xF, RETE at exception level 0, and two fault conditions detected in one instruction are all architecturally undefined. Say so in one sentence each; do not describe the panic.

### F-cop-4
- file: chapters/16-coprocessor-instructions.tex
- lines: 185-186
- severity: major
- category: fact
- claim: "0x1C00-0x1C7F   & ETFR              & Transfer from exception link register     \\ 0x1D00-0x1D7F   & ETTR              & Transfer to exception link register"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs:1-3 (`EXCEPTION_UNIT_TRANSFER_EU_REGISTER_MASK = 0xF`, `EXCEPTION_UNIT_TRANSFER_EU_REGISTER_LENGTH = 4`, `EXCEPTION_UNIT_TRANSFER_REGISTER_SELECT_MASK = 0x03`); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:213-229 (`register_select = (value >> 4) & 0x03`, `eu_register = value & 0xF`); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:227 (`link_registers.get_mut(eu_register)` over an 8-entry array)
- resolution: manual-wrong
- fix: Replace the two ranges with `0x1C00--0x1C37` and `0x1D00--0x1D37`, and add a note that command bits 7--6 are not decoded, bits 5--4 are the register select (0 none, 1 saved address, 2 saved status, 3 both) and bits 3--0 are the link-register index 0--7. Values with bit 6 or bit 7 set are not distinct operations.
- confidence: high
- status: accepted

### F-cop-5
- file: chapters/16-coprocessor-instructions.tex
- lines: 305-306
- severity: major
- category: fact
- claim: "\texttt{\#n} selects an exception link register. Optional operands select whether to transfer the saved return address, saved status register, or both."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:225-229 (`.expect("ETFR Expected target register value between 0-7")` on an 8-entry `link_registers` array); sirc-vm/toolchain/src/parsers/opcodes/exception.rs:148, 180, 223, 265, 296, 338 (`value & 0x000F`, with a `TODO: Throw error if value doesn't fit?` — the assembler silently accepts `#8`--`#15`)
- resolution: manual-wrong
- fix: Add the range to the Operands line: "\texttt{\#n} selects one of the eight exception link registers and must be in the range \imm{0}--\imm{7}. Link register \imm{7} holds fault metadata; link register $L-1$ holds the state saved on entry to an exception at level $L$. Values above \imm{7} are architecturally undefined; the assembler currently masks \texttt{\#n} to four bits without diagnosing the error."
- confidence: high
- status: accepted

### F-cop-6
- file: chapters/16-coprocessor-instructions.tex
- lines: 140
- severity: major
- category: fact
- claim: "0x1         & Exception Unit       & Yes               & 0x0--0xD            & Handles reset, faults, interrupts, traps, and links"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs:147-162 (`ExceptionUnitOpCodes` defines only 0x0 None, 0x1 SoftwareException, 0x9 WaitForException, 0xA ReturnFromException, 0xB Reset, 0xC TransferFromRegister, 0xD TransferToRegister, 0xE Fault, 0xF HardwareException); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:66-77 (opcodes 0x2--0x8 fall through `FromPrimitive` and `panic!("Unimplemented op code ... for exception co-processor")`, with a `TODO: Define what happens when exception CoP gets an unimplemented opcode`). Digest Open question 6 (facts.md:338).
- resolution: manual-wrong
- fix: The contiguous range "0x0--0xD" is not the implemented set. It must become either an explicit list ("0x0--0x1, 0x9--0xD; 0xE--0xF internal") or the architecture must define opcodes 0x2--0x8 as reserved-and-invalid-opcode-faulting, which the implementation does not currently do (it panics). Line 155--156 of this chapter makes the stronger claim that reserved operation nibbles "must raise invalid-opcode faults"; both statements need the same ruling.
- confidence: high
- status: accepted
- ruling: Gate 2 K/L: exception-unit opcodes 0x2--0x8, software-issued 0xE/0xF, RETE at exception level 0, and two fault conditions detected in one instruction are all architecturally undefined. Say so in one sentence each; do not describe the panic.

### F-cop-7
- file: chapters/16-coprocessor-instructions.tex
- lines: 155-156
- severity: major
- category: contradiction
- claim: "Reserved coprocessor IDs, reserved operation nibbles, and reserved operand values must raise invalid-opcode faults when they are not implemented by the selected CPU model."
- evidence: manual side — this line, and lines 12--14 and 65--67 making the same promise; code side — sirc-vm/peripheral-cpu/src/lib.rs:375-394 raises `Faults::InvalidOpCode` only when the *coprocessor ID* is neither 0 nor 1; unimplemented *operation nibbles* on coprocessor 1 panic (exception_unit/execution.rs:69-77) and out-of-range ETFR/ETTR *operand values* panic (exception_unit/execution.rs:227). Digest facts.md:315, Open question 6.
- resolution: manual-wrong
- fix: Either narrow the claim to "Reserved coprocessor IDs raise invalid-opcode faults" and mark reserved operation nibbles and operand values as architecturally undefined, or record this as a required implementation change. Do not leave a "must" that the reference implementation does not satisfy.
- confidence: high
- status: accepted
- ruling: Gate 2 K/L: exception-unit opcodes 0x2--0x8, software-issued 0xE/0xF, RETE at exception level 0, and two fault conditions detected in one instruction are all architecturally undefined. Say so in one sentence each; do not describe the panic.

### F-cop-8
- file: chapters/16-coprocessor-instructions.tex
- lines: 209
- severity: major
- category: fact
- claim: "\textbf{Exceptions:} Dispatches a software exception through the exception unit."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:326-337 (`ignored_software_exception` — when `current_exception_level != 0` and the pending command is a coprocessor-1 SoftwareException, the command register is cleared and no vector is fetched); sirc-vm/peripheral-cpu/tests/exceptions/software_exceptions.rs:74-143; digest facts.md:291 and HO:392-394
- resolution: manual-wrong
- fix: Append: "Software exceptions are accepted only at exception level 0. An \mnemonic{EXCP} executed inside any exception handler is discarded: the pending coprocessor command is cleared, no vector is fetched, and no fault is raised. Nested system calls from a handler are therefore not possible."
- confidence: high
- status: accepted

### F-cop-9
- file: chapters/16-coprocessor-instructions.tex
- lines: 201-202
- severity: major
- category: fact
- claim: "Triggers a software exception at the specified user vector, normally 0x60--0xFF."
- evidence: manual side — "normally", which implies vectors below 0x60 are legal but unusual; code side — sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:330-333 asserts (panics) when a SoftwareException vector is below `USER_EXCEPTION_VECTOR_START` (0x60); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs:51-56 instead lists "Triggering a software exception below 0x60" as a cause of PRIVILEGE_VIOLATION_FAULT; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:46-48 only inspects the opcode nibble, so an `EXCP` can never be privileged. No test. Digest Open question 4 (facts.md:336).
- resolution: code-wrong
- fix: The architecture must choose one of: (a) `EXCP #v` with `v < 0x60` raises PrivilegeViolation in protected mode (matching the EDEF comment), (b) it raises InvalidOpCode, or (c) it is architecturally undefined. Then replace "normally 0x60--0xFF" with the ruled, unhedged statement. The assembler masks the vector to 8 bits and does not reject low vectors (toolchain/src/parsers/opcodes/exception.rs:73).
- confidence: high
- status: code-wrong
- ruling: Gate 2 J: EXCP with a vector below 0x60 must raise a fault (privilege violation), not panic; the guard must compare vector numbers, not doubled addresses. The appendix D example must move to vector 0x60 when the check lands. Manual stands.

### G1-T1-16
- file: chapters/16-coprocessor-instructions.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Rename the `Condition codes:` line in every instruction entry to `Condition field:` (period-gaps item 3).
- confidence: high
- status: accepted

### G1-T2-16
- file: chapters/16-coprocessor-instructions.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Add an applicability line under every instruction entry title: `SIRC-1, all revisions`, or the coprocessor revision per Chapter 16 (period-gaps item 13).
- confidence: high
- status: accepted

### G1-T3-16
- file: chapters/16-coprocessor-instructions.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Add an Instruction Fields list after the opcode list in every entry: one line per encoding field with its legal values here; values from review/facts.md and Chapter 7 only (period-gaps item 2).
- confidence: high
- status: accepted

### F-cop-3
- file: chapters/17-meta-instructions.tex
- lines: 67
- severity: major
- category: contradiction
- claim: "\textbf{Privilege:} Available in protected mode and supervisor mode."
- evidence: manual side — this line and line 58 ("Inherits \mnemonic{ADDI[N]} exception behavior"); code side — sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:21-27 and 44-51 (`PRIVILEGED_REGISTERS` contains `RegisterName::Sr`; `writing_to_privileged_registers` fires for a non-COP instruction whose destination is `sr`), with `NOOP`'s destination being register 0 = `sr` (toolchain meta.rs:88). No test covers `NOOP` in protected mode. Digest Open question 2 (facts.md:334).
- resolution: code-wrong
- fix: The architecture must rule on this. Either (a) `NOOP` is unprivileged, in which case the assembler must stop emitting register field 0 (or the decoder must exempt an all-zero destination with AF = None), or (b) the manual must state that `NOOP`, and therefore any all-zero instruction word, raises a PrivilegeViolation fault in protected mode and is a supervisor-only encoding. Until ruled, this line and line 58 must not claim protected-mode availability.
- confidence: high
- status: code-wrong
- ruling: Gate 2 I: NOOP must not fault in protected mode. Manual stands (unprivileged).

### G1-T1-17
- file: chapters/17-meta-instructions.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Rename the `Condition codes:` line in every instruction entry to `Condition field:` (period-gaps item 3).
- confidence: high
- status: accepted

### G1-T2-17
- file: chapters/17-meta-instructions.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Add an applicability line under every instruction entry title: `SIRC-1, all revisions`, or the coprocessor revision per Chapter 16 (period-gaps item 13).
- confidence: high
- status: accepted

### G1-T3-17
- file: chapters/17-meta-instructions.tex
- lines: entry template
- severity: major
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Add an Instruction Fields list after the opcode list in every entry: one line per encoding field with its legal values here; values from review/facts.md and Chapter 7 only (period-gaps item 2).
- confidence: high
- status: accepted

### F-con-21
- file: chapters/appendix-a-opcode-map.tex
- lines: 126
- severity: major
- category: contradiction
- claim: "\item Bit 1: Determines if both registers in address pair are updated"
- evidence: No auto-update ever writes both registers of a pair. chapters/08-addressing-modes.tex:124-125 "Post-increment and pre-decrement change the selected address register by exactly one word"; chapters/08-addressing-modes.tex:121-122 "The high word supplies the segment and is preserved"; chapters/04-data-representation.tex:153-155; chapters/14-memory-instructions.tex:81-82 ("The address register low word is incremented by one word"). Bit 1 in fact selects the auto-update form (0x10/0x11 vs 0x12/0x13; 0x14/0x15 vs 0x16/0x17).
- resolution: manual-wrong
- fix: Replace with "\item Bit 1: Auto-update form (0 = no address-register update, 1 = pre-decrement for STOR/LDEA or post-increment for LOAD/LDEL; only the low word of the pair changes)".
- confidence: high
- status: accepted

### F-sum-16
- file: chapters/appendix-a-opcode-map.tex
- lines: 99-104
- severity: major
- category: fact
- claim: "Bits 5--4 = 00: Immediate format (0x00--0x0F)"
- evidence: bits 5:4 of the 32-bit instruction word are the AF field (ENC:47-59, 115-137; facts.md Instruction formats table); the opcode occupies bits 31:26. The bullets mean bits 5:4 of the 6-bit opcode value, that is instruction bits 31:30, but nothing on the page says so, and the same section then uses "Bit 3", "Bit 4", "Bits 3--2", "Bit 1", "Bit 0" with the same unstated convention
- resolution: manual-wrong
- fix: Prefix the section with "Bit numbers in this section refer to the 6-bit opcode value (instruction bits 31--26), not to the instruction word." and change each bullet to "Opcode bits 5--4 = 00", and so on.
- confidence: high
- status: accepted

### F-sum-17
- file: chapters/appendix-a-opcode-map.tex
- lines: 101
- severity: major
- category: fact
- claim: "Bits 5--4 = 01: Memory/Control operations (0x10--0x1F)"
- evidence: ENC:272-290 and FD:23-38: in the 0x10-0x1F block even opcodes decode as Immediate format and odd opcodes decode as Register format. The section is headed "Format Identification" but gives no format for a quarter of the opcode space, so a decoder built from this list cannot decode 0x10--0x1F
- resolution: manual-wrong
- fix: Change the bullet to "Opcode bits 5--4 = 01: Memory and control operations (0x10--0x1F). Format is selected by opcode bit 0: even opcodes use Immediate format, odd opcodes use Register format."
- confidence: high
- status: accepted

### F-con-29
- file: chapters/appendix-b-timing.tex
- lines: 187
- severity: major
- category: contradiction
- claim: "\textbf{At 25 MHz:} $\approx$ 4.17 million instructions/second"
- evidence: chapters/02-cpu-architecture.tex:354 specifies the clock input as "\textbf{Max Rate:} 24 Mhz". A performance figure quoted above the documented maximum clock rate is misleading.
- resolution: manual-wrong
- fix: Change to "\textbf{At 24 MHz (maximum rated clock):} $\approx$ 4.0 million instructions/second". (Also fix the unit spelling at 02-cpu-architecture.tex:354: "Mhz" -> "MHz".)
- confidence: high
- status: accepted

### F-tim-10
- file: chapters/appendix-b-timing.tex
- lines: 58-64
- severity: major
- category: contradiction
- claim: "All execution phases run normally" / "The write-back stage is disabled (no register or memory modification)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:33-35 (false condition -> `ExecutionStepInstructionType::NoOp`, no effective address and no `SR.A` overflow check); stages/memory_access.rs:26-28 (false condition -> `MemoryAccessInstructionType::NoOp`, so no `BAS`/`DataRead`/`DataWrite` cycle is driven at phase 4); stages/write_back.rs:128-130; facts.md:182
- resolution: manual-wrong
- fix: The phases still occur, but three of them are suppressed, and the suppression of the memory-access phase is externally visible (a not-executed `LOAD`/`STOR` drives no bus cycle and therefore cannot incur a data wait state). Replace the bullets with: "All six phases still occur and consume one cycle each"; "The execute, memory-access, and write-back phases perform no operation: no effective address is computed, no `SR.A` overflow check is made, and no data bus cycle is driven"; "No register, memory, or address-register update occurs"; "Status flags are not updated".
- confidence: high
- status: accepted

### F-tim-11
- file: chapters/appendix-b-timing.tex
- lines: 26-54
- severity: major
- category: fact
- claim: "The cycle counts in this appendix assume that every bus operation is acknowledged without a wait state."
- evidence: facts.md Open question 1 (EDEF:121-131 says abort faults save the faulting instruction's address for retry, but pl is advanced unconditionally at decode, PU:151, even when the decode-time privilege check has just faulted at PU:141-145; data bus faults arrive after decode, LIB:312-324; only alignment, PU:83-86, and PC-wrap segment overflow, PU:92-97, act before decode); no test asserts the link register for PrivilegeViolation, Bus, BusProtection, or Alignment
- resolution: manual-wrong
- fix: The appendix never says what a faulting instruction costs — whether it runs all six phases and then dispatches, or is aborted at the phase that detects the fault. The digest's open question 1 shows the code and the definitions comment disagree on the closely related question of which address is saved. State explicitly, once the architects rule: at which phase each fault is detected (alignment before phase 0's fetch; privilege violation at phase 2; segment overflow at phase 2/3; bus faults at the acknowledging phase), whether the remaining phases still run, and how many cycles elapse before the exception-unit dispatch slot begins.
- confidence: medium
- status: accepted
- ruling: Gate 2 A: document actual behaviour. Retryable class is Alignment and PC-wrap Segment Overflow only; Bus, Bus Protection, Invalid Opcode and Privilege Violation save the next instruction address and are not restartable.

### F-tim-6
- file: chapters/appendix-b-timing.tex
- lines: 75
- severity: major
- category: fact
- claim: "loop:"
- evidence: sirc-vm/toolchain/src/parsers/shared.rs:112-114 (`parse_label_` = `preceded(char(':'), cut(parse_label_name_))`); assembling a file containing `loop:` fails at line 1 column 1 with "expected ';' ... in section \"instruction\""; the `:label` form assembles
- resolution: manual-wrong
- fix: Label definitions take a leading colon in this assembler. Replace `loop:` with `:loop` here and at line 169, and `skip:` with `:skip` at line 138. (The trailing-colon form also appears in `08-addressing-modes.tex:376`, `10-condition-codes.tex:246`, `15-control-flow.tex:177,276,529,532,538,544,554`, and `16-coprocessor-instructions.tex:326`; a manual-wide sweep is needed, but only the three occurrences in this appendix are mine to report.)
- confidence: high
- status: accepted

### F-tim-7
- file: chapters/appendix-b-timing.tex
- lines: 133-143
- severity: major
- category: fact
- claim: "CMPR r1, r2\nBRAN|<= @skip\nADDI r3, #1\nskip:\n\n; Use:\nCMPR r1, r2\nADDI|HI r3, #1             ; Saves branch overhead"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:129-135, 144-152 (`LessThanOrEqual` = Z || (N != V) — signed; `UnsignedHigher` = C && !Z — unsigned); facts.md:240-245
- resolution: manual-wrong
- fix: The rewrite is not equivalent even before F-tim-1 is applied: `<=` is the signed condition, so its complement is `>>` (GreaterThan), not the unsigned `HI`. Use one domain consistently — with the borrow convention (F-tim-1) the unsigned pair is `BRAN|HI @skip` / `ADDI|LO r3, #1`, or keep the signed test as `BRAN|<= @skip` / `ADDI|>> r3, #1`.
- confidence: high
- status: accepted
- ruling: Per Gate 2 B the glosses stand; fix only the signed/unsigned mismatch between the two halves of the example.

### F-tim-8
- file: chapters/appendix-b-timing.tex
- lines: 6-9, 28-48
- severity: major
- category: fact
- claim: "DMA meta-instructions are the main exception: after the normal coprocessor dispatch they hold instruction fetch until the DMA unit has completed its bus transfers."
- evidence: facts.md:317 ("Every COP instruction costs two 6-cycle slots: the PU instruction, then the coprocessor dispatch at the following phase 0"), citing sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:781-791 and sirc-vm/peripheral-cpu/tests/exceptions/software_exceptions.rs:41-51; dispatch slot structure at sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:302-446 and tests/exceptions/common.rs:125-167
- resolution: manual-wrong
- fix: DMA is not the only exception to "6 cycles". Every coprocessor call that actually dispatches — `COPI`, `COPR`, and the exception-unit meta-instructions `EXCP`, `WAIT`, `RETE`, `RSET`, `ETFR`, `ETTR` — costs the 6-cycle processing-unit instruction plus a further 6-cycle coprocessor dispatch slot beginning at the next phase 0, i.e. 12 cycles before the next instruction fetch (and unbounded for `WAIT`). Add rows to Table "Instruction Timing" for `COPI`/`COPR` (6 + 6), the exception-unit meta-instructions (6 + 6), and `WAIT` (6 + 6 + idle until an enabled interrupt or reset), and reword the Overview sentence to "Coprocessor calls are the exception: the dispatched coprocessor occupies at least one further slot after the six-phase call." Chapter 16's per-entry Timing fields (`16-coprocessor-instructions.tex:213,241,264,287,320`) say only "6 cycles ... plus exception-unit follow-up work" and should be given the same number.
- confidence: high
- status: accepted

### F-tim-9
- file: chapters/appendix-b-timing.tex
- lines: 26-54
- severity: major
- category: fact
- claim: "\section{Timing Characteristics}" (table lists only ALU/load/store/branch/conditional/NOOP/DMA rows)
- evidence: facts.md:286, 327 (EU dispatch = 6 cycles: vector high word at phase 0, low word at phase 1, entry at phase 3; "a software exception therefore starts its handler 12 cycles after the EXCP fetch began; a data bus fault detected at phase 4/5 starts its handler 6 cycles later"), citing sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:303-334, tests/exceptions/common.rs:133-153, tests/exceptions/faults.rs:124-184; reset hold at sirc-vm/peripheral-bus/src/reset_unit.rs:22-26 and tests/exceptions/reset.rs:84-147
- resolution: manual-wrong
- fix: The appendix gives no exception-entry or interrupt-latency figures, and no other chapter quantifies them (`06-exceptions.tex` mentions cycles only for the reset hold). Add a subsection "Exception Timing" stating: a hardware-exception or fault dispatch occupies one 6-cycle slot (vector high word at phase 0, low word at phase 1, entry committed at phase 3), a latched interrupt is dispatched at the next phase 0 so worst-case latency from assertion to the handler's first instruction fetch is 6 (finish current instruction) + 6 (dispatch) cycles, and reset holds RSTO for 6 cycles before the reset-vector fetch.
- confidence: high
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-con-30
- file: chapters/appendix-c-undocumented.tex
- lines: 213
- severity: major
- category: contradiction
- claim: "\item Reserved address register encoding could enable more address modes"
- evidence: There is no reserved value in the address-register field. chapters/08-addressing-modes.tex:345-348 assigns all four: `00 = l, 01 = a, 10 = s, 11 = p`; chapters/07-instruction-formats.tex:84-87 and :208-211 repeat the full assignment. The only reserved 2-bit value in the architecture is AF `11` when the field is used as a status-update selector (chapters/07-instruction-formats.tex:94, :152, :218; chapters/11-reading-instructions.tex:90).
- resolution: manual-wrong
- fix: Replace with "\item The reserved status-update source encoding (AF = 11) could select a new flag-update source".
- confidence: high
- status: accepted

### F-exc-19
- file: chapters/appendix-c-undocumented.tex
- lines: 6, 40-63, 211
- severity: major
- category: contradiction
- claim: "The SIRCIS instruction set includes 14 undocumented instruction opcodes."
- evidence: chapters/appendix-a-opcode-map.tex:130-138 lists exactly 12 undocumented opcodes (0x08, 0x09, 0x0B, 0x0D; 0x28, 0x29, 0x2B, 0x2D; 0x38, 0x39, 0x3B, 0x3D), excluding 0x27 and 0x2F. Only those 12 are named `_Undocumented*` in sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:218-277; 0x27 is `LoadRegisterFromShortImmediate` and 0x2F is `CoprocessorCallShortImmediate`, both with defined, tested behaviour (T-ASI:464-500; T-PROT:530-547). facts.md Open question 10.
- resolution: manual-wrong
- fix: The two appendices give different counts because they disagree on whether an opcode with defined behaviour but no assembler syntax is "undocumented". Author must rule. If 0x27/0x2F stay in this table, the Overview's blanket "their architectural behaviour is undefined" must be qualified for them, since their behaviour is fully specified (0x27: rD = zero-extended 8-bit immediate; 0x2F: coprocessor command = 8-bit immediate, coprocessor ID always 0, therefore never privileged). If they leave, change 14 to 12 here and at line 211 and delete their two rows.
- confidence: high
- status: accepted
- ruling: Gate 2 H: 0x27 and 0x2F are Undocumented, the same class as 0x08/0x09/0x0B/0x0D. Appendix C keeps them and its count of 14; ch12 and ch16 say "Undocumented" consistently.

### F-exc-20
- file: chapters/appendix-c-undocumented.tex
- lines: 65-114, 122-127
- severity: major
- category: fact
- claim: "Based on the systematic opcode organization, the undocumented opcodes with bit 3 set in the ALU operation field likely behave as \"test\" variants of arithmetic instructions" ... "Flag updates may or may not be implemented correctly"
- evidence: sirc-vm/peripheral-cpu/tests/instructions/arithmetic_immediate_test.rs:579-645 (0x08), :651-750 (0x09), :830-930 (0x0B), :966-998 (0x0D) assert exact result-discard and exact N/Z/C/V values for every one of these opcodes; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:132-151 classifies 0x8--0xE as `AluStatusOnly` unconditionally. docs/reference/STYLE.md:25 rules that App. C describes "a specific, currently-unassigned opcode with observed behavior worth describing as a caution, not a promise."
- resolution: manual-wrong
- fix: Replace the speculative framing ("likely", "Probably", "Hypothetical behavior", "may or may not be implemented correctly") with a statement of the observed reference behaviour plus the standing caution: the opcodes compute `alu_code & 0x7` as the operation, discard the result, and update all four flags exactly as the corresponding documented operation does; this is observed behaviour of the current implementation and is architecturally undefined, so software must not rely on it. In particular "Flag updates may or may not be implemented correctly" is false — the flag behaviour is fully covered by tests.
- confidence: high
- status: accepted

### F-exc-21
- file: chapters/appendix-c-undocumented.tex
- lines: 213
- severity: major
- category: fact
- claim: "\item Reserved address register encoding could enable more address modes"
- evidence: sirc-vm/peripheral-cpu/src/registers.rs:75-85 — `AddressRegisterName` has exactly four variants, LinkRegister = 0, Address = 1, StackPointer = 2, ProgramCounter = 3, filling the whole 2-bit field; facts.md:59
- resolution: manual-wrong
- fix: Delete this bullet. All four encodings of the 2-bit address-register-pair field are assigned (l, a, s, p); there is no spare encoding. If a genuine spare is wanted here, the reserved `StatusRegisterUpdateSource` value 0b11 (definitions.rs:100-101) is one — AF = 3 is decoded as "Reserved" and leaves the status register unchanged.
- confidence: high
- status: accepted

### F-con-14
- file: chapters/appendix-e-quick-reference.tex
- lines: 13-14
- severity: major
- category: contradiction
- claim: "\emph{Flags}: flags written by the instruction (\texttt{--} = none; \texttt{*} = determined by the coprocessor)."
- evidence: chapters/11-reading-instructions.tex:60-64 defines the manual-wide symbol set as `*` = "Updated from the instruction result", `-` = "Preserved", `S` = "Updated from the shifter result when \texttt{[S]} is used". Appendix E's own cross-reference table sends readers to Chapter 11 for these symbols (appendix-e-quick-reference.tex:192). Appendix E then uses `*` for SHFT (appendix-e-quick-reference.tex:79), which is not a coprocessor instruction at all — its flags come from the shifter, i.e. symbol `S`.
- resolution: manual-wrong
- fix: Change the legend to "\emph{Flags}: uses the symbols defined in Table~\ref{tab:flag-effect-symbols} (Chapter~\ref{ch:reading-instructions}); \texttt{--} means all four flags preserved and \texttt{COP} means the effect is defined by the selected coprocessor." Replace the `*` entries at lines 38, 39, 42, 43, 61, 62 with `COP`, and change the SHFT row (line 79) from `*` to `S`.
- confidence: high
- status: accepted

### F-con-31
- file: chapters/appendix-e-quick-reference.tex
- lines: 126
- severity: major
- category: contradiction
- claim: "7--4         & --            & Either          & 0              & Reserved; reads as 0"
- evidence: chapters/05-status-register.tex:84-86 "Bits 4 through 7 of the lower byte are reserved for future use. They should not be relied upon by software." — it does not promise a read value. docs/reference/STYLE.md:127-129 (Gate 1, approved) says "Reserved fields stay advisory. Keep the Chapter 5 wording ... Do *not* adopt the M68000 PRM \"must be written as zeros\" form; the implementation does not promise it." Chapter 6 also lists reserved lower status bits as merely "preserved" on exception entry (chapters/06-exceptions.tex:471).
- resolution: manual-wrong
- fix: Change the Function cell to "Reserved for future use; software must not rely on the value read" and the Reset cell to "--".
- confidence: high
- status: accepted

### F-sum-18
- file: chapters/appendix-e-quick-reference.tex
- lines: 126
- severity: major
- category: fact
- claim: "7--4 & -- & Either & 0 & Reserved; reads as 0"
- evidence: write_back.rs:54-71 lets a supervisor-mode direct write to sr set every bit except bit 13, so bits 7--4 can be written non-zero and read back non-zero (protected-mode reads mask with SR_REDACTION_MASK 0x00FF, which preserves bits 7--4: FD:40-48, WB:45-53). Only a subsequent ALU or shift flag update clears them (WB:170-177). STYLE.md "Approved decisions" also rules that reserved fields stay advisory rather than promising a read value
- resolution: manual-wrong
- fix: Replace the Function cell with "Reserved for future use; software must not rely on the value read" and the Reset cell with "0".
- confidence: high
- status: accepted

### F-sum-19
- file: chapters/appendix-e-quick-reference.tex
- lines: 79
- severity: major
- category: fact
- claim: "\mnemonic{SHFT} & Meta & ORRI[S] rD,\#0 & * & Shift with status-register update"
- evidence: the table header at lines 13-14 defines "*" as "determined by the coprocessor"; SHFT is not a coprocessor instruction. Its flags come from the shifter status (WB:175-177; ALU:322-508; facts.md Shift types table), which Chapter 11 denotes with the symbol S
- resolution: manual-wrong
- fix: Change the Flags cell for SHFT to "NZCV (from the shifter)".
- confidence: high
- status: accepted

### F-con-32
- file: generated/immediate-format-encodings.tex
- lines: 20-36
- severity: major
- category: contradiction
- claim: "\texttt{LDEA a, (\#4, s)}   & 0x18            & 0x1          & 0x0004                & 0b10        & 0x0           & \texttt{0x60400120}"
- evidence: The Reg field (instruction bits 25--22) is documented as a 4-bit *register identifier* (chapters/07-instruction-formats.tex:80, :138, :196) using the IDs in `tab:register-encoding` (chapters/03-registers.tex:238-253), where `0x1 = r1` and `0x3 = r3`. The generated rows instead put the 2-bit address-register-pair code there (a=0x1, p=0x3): see also generated/immediate-format-encodings.tex:23,26,29,32,35 and generated/register-format-encodings.tex:20,23,26,29,32,35. Nowhere does the manual say that LDEA/LDEL/BRAN/BRSR/LJMP/LJSR encode their destination pair in the Reg/R1 field using the pair code rather than the register ID.
- resolution: manual-wrong
- fix: Add a subsection to Chapter 7 (after `\subsection{Register Operand Semantics}`) stating: "For the effective-address opcodes 0x18--0x1F, the Reg/R1 field holds the *destination address-register pair* code from Table~\ref{tab:addr-reg-encoding} (zero-extended to four bits), not a general-purpose register identifier from Table~\ref{tab:register-encoding}." Reference it from `tab:control-flow-legal-forms` in Chapter 15.
- confidence: high
- status: accepted

### F-arch-12
- file: chapters/01-introduction.tex
- lines: 69
- severity: minor
- category: fact
- claim: "Addressing Modes            & 7                      \\"
- evidence: docs/reference/manual-handover.md:17 ("Resolved: the CPU has seven addressing modes; operandless meta-instructions ... are not an addressing mode"); sirc-vm/toolchain/src/parsers/instruction.rs:85-106 (ten distinct syntactic operand forms); facts digest Open question 11
- resolution: manual-wrong
- fix: The count of seven is a ruled decision, but no chapter derives it from the ten operand forms the assembler parses, so the number cannot be checked. Add a cross-reference from this row to the addressing-mode chapter's numbered list, and make that list contain exactly seven entries with the immediate/register displacement variants grouped explicitly.
- confidence: medium
- status: phase4
- ruling: Gate 2 N: apply after the ch8 editor settles the addressing-mode count.

### F-con-70
- file: chapters/01-introduction.tex
- lines: multiple
- severity: minor
- category: terminology
- claim: "vary depending on which SIRC-1 model you have"
- evidence: docs/reference/STYLE.md:75-77 "Voice/tense: third person, present tense ... No first person (\"we\"), no second person (\"you\"). This already matches the manual; treat any surviving \"you\" as a defect to flag."
- resolution: manual-wrong
- fix: Rewrite each site in third person; there is no single mechanical substitution. Sites: chapters/01-introduction.tex:97, chapters/02-cpu-architecture.tex:569, chapters/03-registers.tex:85, chapters/06-exceptions.tex:250, chapters/07-instruction-formats.tex:336, chapters/09-shift-operations.tex:15 and :506, chapters/14-memory-instructions.tex:407, chapters/appendix-c-undocumented.tex:178, :185, :188, :189. Locate with `grep -rnw 'you\|your\|You\|Your' docs/reference/chapters/`.
- confidence: high

---
- status: accepted

### F-con-71
- file: chapters/01-introduction.tex
- lines: 141-142
- severity: minor
- category: latex
- claim: "With a 24-bit address space, this allows access to $2^24$ = 16,777,216 words, or 32 megabytes of external byte storage."
- evidence: `$2^24$` renders as $2^2 4$ (superscript 2 followed by a literal 4), not $2^{24}$. The manual gets this right elsewhere: chapters/04-data-representation.tex:150 "$2^{16}$", :157, chapters/16-coprocessor-instructions.tex:386.
- resolution: manual-wrong
- fix: Change `$2^24$` to `$2^{24}$`.
- confidence: high
- status: accepted

### G1-T4-01
- file: chapters/01-introduction.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-arch-10
- file: chapters/02-cpu-architecture.tex
- lines: 349-355
- severity: minor
- category: contradiction
- claim: "\item \textbf{Max Rate:} 24 Mhz"
- evidence: chapters/02-cpu-architecture.tex:899-901 ("Electrical setup, hold, propagation delay, voltage, fanout, and maximum-board-loading requirements are outside the architectural specification"); docs/reference/manual-handover.md:481-485 ("Electrical assumptions are intentionally deferred: ... maximum clock rate basis"); no clock-rate figure exists anywhere in the implementation
- resolution: unclear
- fix: The chapter defers electrical characteristics but still specifies a 24 MHz maximum clock rate (and a 5 V +/- 5% supply at lines 329--333) with no stated basis. Either mark these as informative, model-specific examples ("a typical SIRC-1 model is specified for up to 24 MHz; see the model datasheet") or remove them and leave the rate to the deferred datasheet.
- confidence: medium
- status: defer
- ruling: Gate 2 Q: author to check the hardware figures.

### F-arch-11
- file: chapters/02-cpu-architecture.tex
- lines: 530-532, 889
- severity: minor
- category: fact
- claim: "Interrupt inputs are level-sensitive and are sampled at instruction boundaries." / "Inputs should remain asserted until an instruction boundary if service is required."
- evidence: manual/handover side: docs/reference/manual-handover.md:472-479 resolves IRQ1--IRQ4 and NMI as level-sensitive, instruction-boundary sampled. Code side: sirc-vm/peripheral-cpu/src/lib.rs:295-298 calls `raise_hardware_interrupt` on *every* cycle (so a pulse asserted mid-instruction is latched and later dispatched), while only the dispatch is gated to phase 0 (lib.rs:326-341)
- resolution: manual-wrong
- fix: The handover fixes the architectural rule, but the reference implementation latches an enabled line on any cycle, so it services pulses the manual says may be missed. State explicitly whether a line asserted and released entirely within an instruction is architecturally guaranteed to be lost (manual as written) or latched (implementation), because an emulator must choose.
- confidence: medium
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-arch-8
- file: chapters/02-cpu-architecture.tex
- lines: 133-134
- severity: minor
- category: fact
- claim: "The barrel shifters support logical shift left (LSL), logical shift right (LSR), arithmetic shift right (ASR), and rotate operations."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:71-81 (shift types 0 None, 1 LSL, 2 LSR, 3 ASL, 4 ASR, 5 RTL, 6 RTR, 7 Reserved); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:417-443 (ASL is a distinct operation that sets V when the sign changes)
- resolution: manual-wrong
- fix: Add arithmetic shift left: "logical shift left (\mnemonic{LSL}), logical shift right (\mnemonic{LSR}), arithmetic shift left (\mnemonic{ASL}), arithmetic shift right (\mnemonic{ASR}), rotate left (\mnemonic{RTL}), and rotate right (\mnemonic{RTR})". ASL is not a synonym for LSL here: it is the only shift that sets the overflow flag.
- confidence: high
- status: accepted

### F-arch-9
- file: chapters/02-cpu-architecture.tex
- lines: 616
- severity: minor
- category: contradiction
- claim: "SYNC            & Asserted while phase 1 of an instruction or exception-unit dispatch is active."
- evidence: chapters/06-exceptions.tex:577 ("Instruction phase & Set to phase 0."); sirc-vm/peripheral-cpu/src/coprocessors/shared.rs:6-14 (phases are numbered 0--5, `InstructionFetchLow` = 0)
- resolution: manual-wrong
- fix: Chapter 2 numbers the six phases 1--6 (the `enumerate` at lines 17--57 and this table cell) while Chapter 6 and the implementation number them 0--5. Pick one numbering manual-wide; the recommended fix is to state the phases as 0--5 in Chapter 2 and write this cell as "Asserted while phase 0 (Instruction Fetch, high word) of an instruction or exception-unit dispatch is active."
- confidence: high
- status: accepted

### F-con-34
- file: chapters/02-cpu-architecture.tex
- lines: 133-134
- severity: minor
- category: contradiction
- claim: "The barrel shifters support logical shift left (LSL), logical shift right (LSR), arithmetic shift right (ASR), and rotate operations."
- evidence: Arithmetic Shift Left (ASL, encoding 011) is a documented shift type: chapters/07-instruction-formats.tex:301, chapters/09-shift-operations.tex:30 and §"Arithmetic Shift Left (ASL)" at :130-144, chapters/13-alu-instructions.tex:241 "All shift types are supported: LSL, LSR, ASL, ASR, RTL, and RTR.", chapters/14-memory-instructions.tex:351.
- resolution: manual-wrong
- fix: Change to "logical shift left (LSL), logical shift right (LSR), arithmetic shift left (ASL), arithmetic shift right (ASR), and rotate left/right (RTL, RTR)".
- confidence: high
- status: accepted

### F-con-35
- file: chapters/02-cpu-architecture.tex
- lines: 131-137
- severity: minor
- category: contradiction
- claim: "Two barrel shifter units sit between the Register File and the CU ... The shift is applied to the first source operand when fetching the source registers for an instruction. Shifts cannot be applied to the result or any other source operand."
- evidence: If only one operand can ever be shifted, the count "Two barrel shifter units" is unexplained; no other chapter mentions a second shifter (chapters/09-shift-operations.tex:12-13, :46-48 describe a single shift path).
- resolution: manual-wrong
- fix: Either state what the second shifter is for (for example, one per read port with only one enabled per instruction) or change "Two barrel shifter units sit" to "A barrel shifter sits".
- confidence: medium
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-con-60
- file: chapters/02-cpu-architecture.tex
- lines: multiple
- severity: minor
- category: terminology
- claim: "DMA Co-processor Read Burst"
- evidence: docs/reference/STYLE.md:22 "coprocessor | co-processor | 155 vs 7; drop the hyphen everywhere, including headings."
- resolution: manual-wrong
- fix: `grep -rl 'co-processor\|Co-processor' docs/reference/chapters/` then `s/co-processor/coprocessor/g; s/Co-processor/Coprocessor/g`. Files: chapters/02-cpu-architecture.tex (4), chapters/06-exceptions.tex (1), chapters/appendix-e-quick-reference.tex (2).
- confidence: high
- status: accepted

### F-con-72
- file: chapters/02-cpu-architecture.tex
- lines: 280-309
- severity: minor
- category: latex
- claim: "\begin{table}[] \\ \centering \\ \begin{tabular}{rll} \\ \textbf{Pin} & \textbf{Name} & \textbf{Description}          \\ \hline"
- evidence: docs/reference/STYLE.md:94-95 "Use `booktabs` (`\toprule`/`\midrule`/`\bottomrule`) for every table; no vertical rules, no hand-drawn `\hline`." Three tables in the manual violate this: chapters/02-cpu-architecture.tex:280-309 and :459-474 use `\hline` with no rules, and chapters/appendix-a-opcode-map.tex:11-91 uses both `\hline` and vertical rules (`{|c|l|l|l|}`). Every other table in the corpus uses booktabs. The two Chapter 2 tables also use an empty float specifier `\begin{table}[]` where every other table uses `[H]`.
- resolution: manual-wrong
- fix: Convert all three tables to `booktabs`: replace the header `\hline` with `\midrule`, add `\toprule` after `\begin{tabular}` and `\bottomrule` before `\end{tabular}`, delete the interior `\hline`s, and drop the vertical rules from the Appendix A column spec (`{|c|l|l|l|}` -> `{cll l}`). Change `\begin{table}[]` to `\begin{table}[H]` at 02-cpu-architecture.tex:280 and :459. The two Chapter 2 tables also lack `\label`s while their captions are cross-referenced only informally; add `\label{tab:pinout}` and `\label{tab:bat-encoding}` if they are to be referenced.
- confidence: high
- status: accepted
- ruling: Chapter 2 part applied in Phase 4 by the orchestrator (the ch2 editor had already finished); appendix A part applied by the appendix A editor.

## Summary

Findings by category:

| Category | blocker | major | minor | nit | total |
|---|---|---|---|---|---|
| contradiction | 8 | 24 | 22 | 1 | 55 |
| structure | 0 | 1 | 3 | 0 | 4 |
| terminology | 0 | 0 | 11 | 0 | 11 |
| latex | 0 | 0 | 2 | 0 | 2 |
| **total** | **8** | **25** | **38** | **1** | **72** |

Mechanical checks all pass: every `\ref` target has a matching `\label` (0 dangling), every `\label`
is unique (0 duplicates), every `\input` in `main.tex` resolves and every file in
`chapters/` is included, and every `lstlisting`, `instructionbox`, `table`, `tabular`, `figure`,
`bytefield` and `verbatim` environment is balanced across `chapters/` and `generated/`.

### The three most damaging contradictions

1. **F-con-1 — the condition-code table in Chapter 7 disagrees with the one in Chapter 10 on two of
   sixteen encodings.** Chapter 7 calls `1100` = `<` and `1110` = `<<`; Chapter 10 calls them `<<`
   and `<=`. Appendix E's "where to find other tables" points implementers at the Chapter 7 version.
   Every example in the manual follows Chapter 10, so an assembler written from Chapter 7 would
   silently emit signed-less-or-equal where the programmer wrote signed-less-than. This is a
   normative encoding disagreement between two tables that both claim to be complete, and it
   propagates into F-con-2 (Chapter 13's comparison guidance, which invents `<` and `>`).

2. **F-con-4 — the manual cannot decide what a register-offset memory shift shifts.** Chapter 14
   says four times that the *loaded value* is shifted after the memory read and before write-back.
   Chapter 9 and Chapter 2 both state flatly that shifts apply only to the first source operand and
   never to a result, and Chapter 9 pins the shift to the Decode phase, three phases before the data
   exists. The generated encoding tables settle nothing: `LOAD r1, (r2, a)` encodes R2 = 0, so under
   the Chapter 9 rule there is no operand to shift. Hardware built from Chapter 9 and hardware built
   from Chapter 14 will not agree on the result of `LOAD rD, (rO, addr), LSL #2`.

3. **F-con-3 — the fault metadata register bit diagram is drawn backwards.** The figure places BAT
   in bits 15--13 and Reserved in bits 3--0; the prose immediately beneath it places BAT in bits
   2--0 and Reserved in bits 15--12. `bytefield` MSB-left is the manual's own convention (the status
   register figure in Chapter 5 follows it), so the figure is unambiguously the wrong one. A fault
   handler written from the figure decodes every field of every fault at the wrong offset — and
   this is the register a double-fault last-chance handler reads first.
- status: accepted

### G1-T4-02
- file: chapters/02-cpu-architecture.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-arch-13
- file: chapters/03-registers.tex
- lines: 6-16
- severity: minor
- category: fact
- claim: "The SIRC-1 CPU provides sixteen 16-bit registers that serve various purposes."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:524-536 (`reset()` does not touch r1--r7 or the l/a/s pairs); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:367-370 (only SR = 0 and PC = reset vector); chapters/06-exceptions.tex:583 states the rule
- resolution: n/a
- fix: The register-model chapter never states power-on/reset state, so a reader building from Chapter 3 alone has no reset values. Add one sentence: "Only \reg{sr} (cleared to zero) and the \reg{p} pair (loaded from the reset vector) have defined values after reset; all other registers are architecturally undefined. See Chapter~\ref{ch:exceptions}."
- confidence: high
- status: accepted

### F-arch-14
- file: chapters/03-registers.tex
- lines: 192-193
- severity: minor
- category: fact
- claim: "In protected mode, reads of \reg{sr} are redacted to the lower byte (upper byte reads as zero). Direct writes to \reg{sr} raise a privilege violation fault."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:55-72 (a supervisor-mode direct write to `sr` is permitted but cannot change bit 13, EA); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:29-33 (the privilege check is skipped entirely outside protected mode)
- resolution: manual-wrong
- fix: The second sentence reads as unconditional. Write "In protected mode ... and direct writes to \reg{sr} raise a privilege violation fault. In supervisor mode a direct write to \reg{sr} is permitted, except that it cannot change the Exception Active (EA) bit."
- confidence: high
- status: accepted

### F-arch-15
- file: chapters/03-registers.tex
- lines: 43-52
- severity: minor
- category: contradiction
- claim: "\subsection{Usage Conventions} While there are no hardware-enforced conventions, typical usage patterns include: \item \reg{r5}--\reg{r7}: Preserved across function calls (by convention)"
- evidence: chapters/01-introduction.tex:207-210 ("Software conventions --- including register usage, calling conventions, stack frame layout ... are outside the scope of this document. These topics are addressed in the SIRC-1 ABI and Toolchain documentation.")
- resolution: manual-wrong
- fix: Chapter 1 declares register usage and calling conventions out of scope, then Chapter 3 prescribes a caller/callee-saved split that no other chapter or tool enforces. Either delete this subsection and point at the ABI document, or narrow Chapter 1's scope statement to exclude this one informative note.
- confidence: high
- status: accepted

### F-con-64
- file: chapters/03-registers.tex
- lines: multiple
- severity: minor
- category: terminology
- claim: "\item \textbf{Upper Byte (bits 15--8):} Control and privileged state"
- evidence: docs/reference/STYLE.md:21 "high word / high byte / low word / low byte | upper word / upper byte | 43 \"high\" vs 12 \"upper\"; standardize on \"high\"/\"low\"."
- resolution: manual-wrong
- fix: `s/[Uu]pper [Bb]yte/high byte/g; s/[Uu]pper word/high word/g; s/[Uu]pper nibble/high nibble/g; s/upper 8 bits/high 8 bits/g` (capitalize where the original was a heading or table cell). Files: chapters/01-introduction.tex:149, chapters/02-cpu-architecture.tex:488, chapters/03-registers.tex:77,189,192,238, chapters/05-status-register.tex:11,100,220,221, chapters/09-shift-operations.tex:126, chapters/13-alu-instructions.tex:622,913,922, chapters/14-memory-instructions.tex:386, chapters/appendix-e-quick-reference.tex:107.
- confidence: high
- status: accepted

### G1-T4-03
- file: chapters/03-registers.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

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
- status: accepted

### G1-T4-04
- file: chapters/04-data-representation.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-con-37
- file: chapters/05-status-register.tex
- lines: 101
- severity: minor
- category: contradiction
- claim: "\item Privileged coprocessor operations (opcodes > 0x07xx)"
- evidence: The rule is stated as an operation-nibble range everywhere else: chapters/01-introduction.tex:119-120 "the first 8 (0x0-0x7) can be called in any mode. The last 8 (0x8-0xF) can only be called in supervisor mode"; chapters/16-coprocessor-instructions.tex:51 "coprocessor-operation nibbles 0x0--0x7 are user-callable and 0x8--0xF are supervisor-only"; chapters/16-coprocessor-instructions.tex:106-107. The notation "opcodes > 0x07xx" is a numeric comparison against a command word and is wrong for any coprocessor other than 0x0.
- resolution: manual-wrong
- fix: Change to "\item Privileged coprocessor operations (operation nibble 0x8--0xF)".
- confidence: high
- status: accepted

### F-con-43
- file: chapters/05-status-register.tex
- lines: 299
- severity: minor
- category: contradiction
- claim: "ADDI r1, r2, r3"
- evidence: `ADDI` is the immediate form and takes at most `rD, #imm16` or `rD, #imm8, shift` (chapters/13-alu-instructions.tex:57-58, :253-254). The three-register form is `ADDR` (chapters/13-alu-instructions.tex:59, :255).
- resolution: manual-wrong
- fix: Change to `ADDR r1, r2, r3`.
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: accepted

### G1-T4-05
- file: chapters/05-status-register.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-con-45
- file: chapters/06-exceptions.tex
- lines: 442
- severity: minor
- category: contradiction
- claim: "(since vectors are 32-bit addresses stored as two 16-bit words)"
- evidence: Vectors hold 24-bit addresses stored in 32 bits: chapters/06-exceptions.tex:506 "Each vector is a 24-bit target address stored as two consecutive 16-bit words"; chapters/04-data-representation.tex:97 "Each exception vector table entry is a stored 24-bit address"; chapters/04-data-representation.tex:64-65 "Only bits 7--0 of the high word are significant when the value is loaded as an address." The same slip appears at chapters/06-exceptions.tex:166 "\textbf{Return Address:} The 32-bit address to return to".
- resolution: manual-wrong
- fix: Change line 442 to "(since each vector is a 24-bit address stored as two 16-bit words)" and line 166 to "The 24-bit return address, stored in a 32-bit field".
- confidence: high
- status: accepted

### F-con-46
- file: chapters/06-exceptions.tex
- lines: 222-226
- severity: minor
- category: contradiction
- claim: "ETFR #6                   ; Save level 6 exception registers to a and r7 ... ETFR #7                   ; Save level 7 (metadata) exception registers to a and r7"
- evidence: 6 and 7 are *register indices*, not exception levels. chapters/06-exceptions.tex:192-199 (`tab:exception-link-register-assignment`) gives register 6 = Faults at priority level 7 and register 7 = Fault metadata at priority level "n/a". Calling register 7 "level 7" collides with priority level 7, which is the fault priority stored in register 6.
- resolution: manual-wrong
- fix: Change the comments to "; Save fault link register (register 6) to a and r7" and "; Save fault metadata register (register 7) to a and r7". Apply the same change to the matching `ETTR` comments at lines 238 and 242.
- confidence: high
- status: accepted

### F-con-61
- file: chapters/06-exceptions.tex
- lines: multiple
- severity: minor
- category: terminology
- claim: "e.g.\ mapping a page in response to a bus fault"
- evidence: docs/reference/STYLE.md:125-126 (Gate 1, approved) "**\"for example\", never \"e.g.\"** Spell it out in prose and tables alike. Replace every `e.g.` manual-wide."
- resolution: manual-wrong
- fix: `s/(e\.g\.,? ?/(for example, /g` and `s/e\.g\.\\? /for example, /g`, then re-read each site for comma placement. Files: chapters/02-cpu-architecture.tex (1), chapters/05-status-register.tex (1), chapters/06-exceptions.tex (4), chapters/07-instruction-formats.tex (1).
- confidence: high
- status: accepted

### F-con-63
- file: chapters/06-exceptions.tex
- lines: multiple
- severity: minor
- category: terminology
- claim: "For abort exceptions (faults), the return address is the address of the faulting instruction so it can be retried."
- evidence: docs/reference/STYLE.md:17 "fault | abort exception | \"Fault\" dominant (207 vs 4); \"abort exception\" survives only as a stray label, already flagged fixed in handover."
- resolution: manual-wrong
- fix: `s/abort exceptions (faults)/faults/g; s/Faults (abort exceptions)/Faults/g; s/Faults (abort exceptions, highest priority)/Faults (highest priority)/g`. Files: chapters/06-exceptions.tex:182,412,639, chapters/16-coprocessor-instructions.tex:167.
- confidence: high
- status: accepted

### F-con-69
- file: chapters/06-exceptions.tex
- lines: 648-649
- severity: minor
- category: terminology
- claim: "\textbf{ETFR (Exception Transfer From Register):} Copies the return address and/or status register from a link register to the address register and/or R7"
- evidence: docs/reference/STYLE.md:28 names these exact lines: "`06-exceptions.tex:648–649` uses bare \"R7\" for the real register `r7`; correct to `\reg{r7}` to match the 89-instance convention." docs/reference/STYLE.md:27 reserves bare `R1/R2/R3` for abstract field-position labels only. chapters/16-coprocessor-instructions.tex:311-312 uses `\reg{r7}` correctly.
- resolution: manual-wrong
- fix: `s/\bR7\b/\\reg{r7}/g` on chapters/06-exceptions.tex:648-649 only. While in the same paragraph, wrap the bare mnemonics `ETFR`/`ETTR` (lines 648, 649) in `\mnemonic{}`; the same bare-mnemonic problem occurs at chapters/02-cpu-architecture.tex:585 (RSET), chapters/03-registers.tex:83 (STOR/LOAD), chapters/06-exceptions.tex:138 (EXCP), :308 (\texttt{ETFR}), :492 and :628 (RETE).
- confidence: high
- status: accepted

### F-exc-22
- file: chapters/06-exceptions.tex
- lines: 18-21, 543, 546
- severity: minor
- category: contradiction
- claim: "Unlike hardware and software exceptions, faults cannot be masked or deferred: when a fault condition is detected, the fault is dispatched at the next instruction boundary regardless of the current exception level."
- evidence: chapters/06-exceptions.tex:543 and :546 give Maskability "SR.A gated" for Segment Overflow and "SR.T gated" for Trace; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:110-115 and sirc-vm/peripheral-cpu/src/lib.rs:349-353 confirm those two faults are conditioned on SR bits
- resolution: manual-wrong
- fix: Narrow the sentence to the property that is actually true: "Once a fault condition is *raised*, the fault cannot be masked or deferred; it is dispatched at the next instruction boundary regardless of the current exception level. Two faults are conditional on whether they are raised at all: Segment Overflow requires \texttt{SR.A}, and Instruction Trace requires \texttt{SR.T} or the \texttt{TRCE} input."
- confidence: high
- status: accepted

### F-exc-23
- file: chapters/06-exceptions.tex
- lines: 166, 174
- severity: minor
- category: fact
- claim: "\textbf{Return Address:} The 32-bit address to return to when the exception handler completes"
- evidence: sirc-vm/peripheral-cpu/src/registers.rs:290-296 — `to_full_address` masks with `ADDRESS_MASK = 0x00FF_FFFF`, so `get_full_pc_address()` (the value stored on entry, execution.rs:259) is always 24-bit; sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:939-942, 989-992 show stored values with a zero top byte; chapters/04-data-representation.tex:95-104 calls the vector "a stored 24-bit address"
- resolution: manual-wrong
- fix: "\textbf{Return Address:} The 24-bit return address, held in a 32-bit field whose top 8 bits are always zero." Apply the same wording to the `\bitbox{32}{Return Address}` caption context in Figure~\ref{fig:exception-link-register-layout}.
- confidence: high
- status: accepted

### F-exc-24
- file: chapters/06-exceptions.tex
- lines: 222, 226, 238, 242
- severity: minor
- category: fact
- claim: "ETFR #6                   ; Save level 6 exception registers to a and r7"
- evidence: chapters/06-exceptions.tex:155-161 and Table~\ref{tab:exception-link-register-assignment} — link register 6 belongs to priority *level 7* (faults) and link register 7 is fault metadata, not a level; sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:213-229 shows the value's low nibble is a link-register index, not a level
- resolution: manual-wrong
- fix: The immediate operand of \mnemonic{ETFR}/\mnemonic{ETTR} is a link-register index, not an exception level. Change the four comments to "; Save link register 6 (fault) to a and r7", "; Save link register 7 (fault metadata) to a and r7", "; Restore link register 7 (fault metadata) from a and r7", "; Restore link register 6 (fault) from a and r7". The instructions themselves assemble correctly (toolchain/src/parsers/opcodes/exception.rs:250-278, 374-390).
- confidence: high
- status: accepted

### F-exc-25
- file: chapters/06-exceptions.tex
- lines: 667
- severity: minor
- category: fact
- claim: "\mnemonic{WAIT}      & 0x9             & Supervisor         & Enter wait state until an exception occurs"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:299-302 (the CPU returns immediately while `waiting_for_exception`, so no instruction executes and no fault can be raised) and :499-518 (`raise_hardware_interrupt` clears `waiting_for_exception` only for interrupt lines that are *enabled*); sirc-vm/peripheral-cpu/tests/exceptions/reset.rs:227-248 (reset also wakes it)
- resolution: manual-wrong
- fix: "Enter wait state until an enabled hardware interrupt or a reset occurs". A disabled interrupt line does not wake the CPU, and no fault or software exception can occur while waiting because no instruction is executing. Chapter~\ref{ch:meta-instructions} should carry the same wording.
- confidence: high
- status: accepted

### G1-T4-06
- file: chapters/06-exceptions.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-con-62
- file: chapters/07-instruction-formats.tex
- lines: multiple
- severity: minor
- category: terminology
- claim: "SHFT is a meta instruction using ORRI opcode (0x25) with AF=0b10"
- evidence: docs/reference/STYLE.md:19 "meta-instruction | meta instruction, alias, convenience instruction | Resolved in handover; hyphenated form dominant (73 vs 4)."
- resolution: manual-wrong
- fix: `s/meta instruction/meta-instruction/g`. Files: chapters/02-cpu-architecture.tex:585, chapters/07-instruction-formats.tex:167,252, chapters/09-shift-operations.tex:15.
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted

### G1-T4-07
- file: chapters/07-instruction-formats.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-con-41
- file: chapters/08-addressing-modes.tex
- lines: 66
- severity: minor
- category: contradiction
- claim: "\texttt{BRAN/BRSR \#disp}; \texttt{BRAN/BRSR @label}; \texttt{RETS}; \texttt{LJMP src[, \#offset|rO]}; \texttt{LJSR src[, \#offset|rO]}; \texttt{LJSR (\#offset, src)+}."
- evidence: Chapter 15 documents a further legal form that this legality table omits: chapters/15-control-flow.tex:53 "LJSR & \texttt{LJSR (rS, src)+}" and chapters/15-control-flow.tex:461, :514. The register-displacement post-increment call form is missing from the Chapter 8 legality table.
- resolution: manual-wrong
- fix: Append `; \texttt{LJSR (rO, src)+}` to the Legal operand forms cell.
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted

### G1-T4-08
- file: chapters/08-addressing-modes.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-con-52
- file: chapters/09-shift-operations.tex
- lines: 163
- severity: minor
- category: contradiction
- claim: "\item Bit 15 (sign bit) is copied into bit 15 and bit 14"
- evidence: Only true for a one-position shift. The same list says the shift is by $n$ positions: chapters/09-shift-operations.tex:164 "Equivalent to signed division by $2^n$", and chapters/09-shift-operations.tex:288 "The shift count is a 4-bit field, allowing shifts of 0--15 positions." chapters/09-shift-operations.tex:148 states the general rule correctly: "Shift bits to the right, preserving the sign bit (bit 15)."
- resolution: manual-wrong
- fix: Change to "\item Bit 15 (sign bit) is replicated into all bits vacated by the shift".
- confidence: high
- status: accepted

### F-con-53
- file: chapters/09-shift-operations.tex
- lines: 413
- severity: minor
- category: contradiction
- claim: "SHFT[N] r2, LSR #1    ; Shift low while preserving carry from high word"
- evidence: \mnemonic{SHFT} is defined as exactly `ORRI[S]` — the `[S]` is part of the meta-instruction: chapters/09-shift-operations.tex:535 "SHFT r1, LSL #3               ; ORRI[S] r1, #0, LSL #3"; chapters/13-alu-instructions.tex:192 "\textbf{Assembles to:} \texttt{ORRI[S] rD, \#0, shift}"; chapters/12-instruction-summary.tex:139; chapters/17-meta-instructions.tex:15 "\mnemonic{SHFT} ... \mnemonic{ORRI[S]}". `SHFT[N]` therefore asks for `[S]` and `[N]` at once. The same line is repeated at chapters/09-shift-operations.tex:422.
- resolution: manual-wrong
- fix: Either state explicitly in the \mnemonic{SHFT} entry (chapters/13-alu-instructions.tex:191-243) that an explicit status override suffix on \mnemonic{SHFT} overrides the implied `[S]`, or change both example lines to `ORRI[N] r2, #0, LSR #1`.
- confidence: medium
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

### F-con-55
- file: chapters/09-shift-operations.tex
- lines: 248-253
- severity: minor
- category: contradiction
- claim: "0               & Shift by Immediate      & R2 (first source operand) is shifted by a literal amount \\ 1               & Shift Count in Register & R2 is shifted by the amount in the shift count register"
- evidence: `tab:shift-operand` describes SO purely in terms of the Register-format field R2, but SO also exists in the Short Immediate format, where there is no R2 field — the shifted operand is the Reg field (chapters/07-instruction-formats.tex:138-146). chapters/09-shift-operations.tex:60 (`tab:source-operand-examples`) shows the short-immediate case shifting `r1` (the Reg field), not R2.
- resolution: manual-wrong
- fix: Change both Description cells to "The first source operand (Register format: R2; Short Immediate format: the Reg field) is shifted ...".
- confidence: high

---
- status: accepted

### F-con-68
- file: chapters/09-shift-operations.tex
- lines: 43
- severity: minor
- category: terminology
- claim: "A shift postamble can come after any short immediate or register instruction."
- evidence: The same construct is called a "shift suffix" in Chapter 8 and Chapter 15 (chapters/08-addressing-modes.tex:62 "Shift suffixes are only available on register-displacement memory forms", :63, :64, :65; chapters/15-control-flow.tex:75-76 "\textbf{Shift suffixes:} ... Register-offset forms do not accept shift suffixes"), and "shift" or "shift definition" in Chapter 11 (chapters/11-reading-instructions.tex:41). "Postamble" appears once in the whole manual.
- resolution: manual-wrong
- fix: `s/shift postamble/shift suffix/`. One file: chapters/09-shift-operations.tex:43.
- confidence: high
- status: accepted

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
- status: accepted

### G1-T4-09
- file: chapters/09-shift-operations.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

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
- status: accepted

### F-data-18
- file: chapters/10-condition-codes.tex
- lines: 9-10
- severity: minor
- category: fact
- claim: "If the condition is not met, the instruction behaves as a \mnemonic{NOOP} (no operation)."
- evidence: Open question 2 of docs/reference/review/facts.md. A false condition suppresses every effect including the decode-time privilege check (processing_unit/execution.rs:35-37; tests/instructions/protected_mode_test.rs:135-150) and the pending coprocessor command (protected_mode_test.rs:497-509), whereas the NOOP meta-instruction itself assembles to ADDI with register field 0, which is \reg{sr} (toolchain/src/parsers/opcodes/meta.rs:80-95) and therefore raises PrivilegeViolation in protected mode (processing_unit/execution.rs:21-27). No test covers NOOP in protected mode.
- resolution: manual-wrong
- fix: Avoid defining a false condition in terms of NOOP: "If the condition is not met the instruction has no architectural effect: no register, flag, memory, address-register or coprocessor state changes, and no privilege check is performed." The same substitution is needed at line 105 for the NV entry.
- confidence: medium
- status: accepted
- ruling: Gate 2 Q: apply the fix as proposed.

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
- status: accepted

### G1-T4-10
- file: chapters/10-condition-codes.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-con-59
- file: chapters/11-reading-instructions.tex
- lines: 54-69
- severity: minor
- category: structure
- claim: "\texttt{S}      & Updated from the shifter result when \texttt{[S]} is used"
- evidence: `tab:flag-effect-symbols` defines five symbols and omits `U`. docs/reference/STYLE.md:136-139 (Gate 1, approved) requires: "The symbol set is `*` (updated), `0`, `1`, `-` (preserved), `S` (from shifter), and `U` (undefined: the implementation leaves the flag in an unspecified state). Chapter 11's notation table must define `U`." Chapter 11 is the table that every flag table in Chapters 13--15 cites (chapters/13-alu-instructions.tex:162, chapters/14-memory-instructions.tex:121, chapters/15-control-flow.tex:128).
- resolution: manual-wrong
- fix: Add a row `\texttt{U}      & Architecturally undefined; the flag is left in an unspecified state \\` to `tab:flag-effect-symbols`.
- confidence: high

---
- status: accepted

### F-sum-20
- file: chapters/11-reading-instructions.tex
- lines: 54-69
- severity: minor
- category: structure
- claim: "\texttt{S} & Updated from the shifter result when \texttt{[S]} is used"
- evidence: STYLE.md "Approved decisions" (Notation): "The symbol set is * (updated), 0, 1, - (preserved), S (from shifter), and U (undefined: the implementation leaves the flag in an unspecified state). Chapter 11 notation table must define U."
- resolution: n/a
- fix: Add a final row before \bottomrule: "\texttt{U} & Architecturally undefined; the implementation leaves the flag in an unspecified state and software must not rely on it".
- confidence: high
- status: accepted

### F-sum-21
- file: chapters/11-reading-instructions.tex
- lines: 9-25
- severity: minor
- category: structure
- claim: "\item[Write-back] Whether the computed result is written to a register ... \item[Status Flags] The effect on the condition flags in the status register. \item[Condition Codes] All normal instructions may be conditional."
- evidence: STYLE.md section 4 fixes the canonical order and names: Opcodes, Syntax, Operands, Operation, Description, Flags, Write-back, Exceptions, Condition codes, Timing, Privilege, Example, Notes, with the predicated-execution line relabelled "Condition field" per the approved decisions; the entries in Chapters 13-17 already use Flags before Write-back and include Description, Example and Notes
- resolution: n/a
- fix: Reorder the description list to the canonical order, rename "Status Flags" to "Flags" and "Condition Codes" to "Condition field", and add the missing Description, Example and Notes items plus the approved Applicability and Instruction Fields lines.
- confidence: high
- status: accepted

### G1-T4-11
- file: chapters/11-reading-instructions.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### G1-T5-11
- file: chapters/11-reading-instructions.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Define the `U` (undefined) flag symbol in the flag-symbol table.
- confidence: high
- status: accepted

### F-con-50
- file: chapters/12-instruction-summary.tex
- lines: 168-175
- severity: minor
- category: contradiction
- claim: "\item Execute/Address Calculate \\ \item Memory Access (or NOP)"
- evidence: The canonical phase names are given identically in two other places and differ here: chapters/02-cpu-architecture.tex:38 "\textbf{Execute and Address Calculation}" and :44 "\textbf{Memory Access}"; chapters/appendix-b-timing.tex:21-22 repeats Chapter 2 verbatim.
- resolution: manual-wrong
- fix: Change to "Execute and Address Calculation" and "Memory Access" to match Chapter 2 and Appendix B.
- confidence: high
- status: accepted

### F-sum-22
- file: chapters/12-instruction-summary.tex
- lines: 33-38
- severity: minor
- category: fact
- claim: "0x08 & -- & -- & -- & Undocumented"
- evidence: undocumented opcodes are valid encodings that decode and execute: 0x08 is an Immediate-format ADD that updates flags and discards the result (T-AI:579-645); ENC:726-739 decodes all 64 opcodes and there is no invalid-opcode detection for CPU instructions (EDEF:45-48). A Format cell of "--" suggests these words do not decode
- resolution: manual-wrong
- fix: Fill in the Format cell for every undocumented row (Immediate for 0x08/0x09/0x0B/0x0D, Short Imm+Shift for 0x28/0x29/0x2B/0x2D, Register for 0x38/0x39/0x3B/0x3D) and leave only the Mnemonic cell as "--".
- confidence: medium
- status: accepted

### F-sum-23
- file: chapters/12-instruction-summary.tex
- lines: 13
- severity: minor
- category: contradiction
- claim: "\textbf{Coprocessor (0x0F, 0x3F)}: Coprocessor interface"
- evidence: 0x2F is also a coprocessor call (CoprocessorCallShortImmediate, DEF:207-280; T-PROT:530-547) and the same chapter lists it as COPI at line 74; the ALU bullet at line 10 meanwhile claims 0x00--0x0F, 0x20--0x2F and 0x30--0x3F are all ALU instructions, which double-counts 0x0F, 0x2F and 0x3F
- resolution: manual-wrong
- fix: Change the ALU bullet ranges to "0x00--0x0E, 0x20--0x2E, 0x30--0x3E" and the coprocessor bullet to "(0x0F, 0x2F, 0x3F)".
- confidence: high
- status: accepted

### G1-T4-12
- file: chapters/12-instruction-summary.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-alu-10
- file: chapters/13-alu-instructions.tex
- lines: 627-630
- severity: minor
- category: fact
- claim: "; Set bit N (variable)\nADDI r4, #1\nSHFT r4, LSL r5            ; r4 = 1 << r5"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:188 — immediate-format ADDI computes `a = rD`, so `ADDI r4, #1` yields `r4 + 1`, not 1. The comment `r4 = 1 << r5` only holds if r4 was zero.
- resolution: manual-wrong
- fix: Replace `ADDI r4, #1` with `LOAD r4, #1`.
- confidence: high
- status: accepted

### F-alu-11
- file: chapters/13-alu-instructions.tex
- lines: 992-994
- severity: minor
- category: contradiction
- claim: "TSAI r3, #0xFF             ; Mask to lower byte\nTSXI r3, #0xAA             ; Test if lower byte == 0xAA"
- evidence: docs/reference/chapters/13-alu-instructions.tex:897 ("\textbf{Write-back:} None. The AND result is discarded") and sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:132-151 (0x0C is `AluStatusOnly`). \mnemonic{TSAI} does not mask anything, so the following \mnemonic{TSXI} tests the whole 16-bit register, not the low byte.
- resolution: manual-wrong
- fix: Replace the two lines with `ANDI r3, #0x00FF           ; Mask to low byte` followed by `TSXI r3, #0x00AA           ; Test if low byte == 0xAA`, or delete the first line and change the comment to "Test whether the whole word equals 0x00AA".
- confidence: high
- status: accepted

### F-alu-12
- file: chapters/13-alu-instructions.tex
- lines: 721-722, 730, 737
- severity: minor
- category: fact
- claim: "LOAD rD, #imm16                ; rD = imm16 (via add)" ... "; Implemented as: rD = 0 + operand" ... "This is actually implemented as an ADD with an implicit zero operand, but flags are not updated."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:307-317 — the Load ALU code is a distinct operation (`let result = b;`), not an add with a zero operand; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:61-67 selects it as `alu_code & 0x7 == 7`. Under AF = Alu it also produces different flags from an add (arithmetic_register_test.rs:497-519).
- resolution: manual-wrong
- fix: Drop the "(via add)" comments and replace the Description sentence with "The ALU passes the second operand straight through to the destination; the first operand is ignored." Remove the `; Implemented as: rD = 0 + operand` line from the Operation block.
- confidence: high
- status: accepted

### F-alu-13
- file: chapters/13-alu-instructions.tex
- lines: 99, 717, 739
- severity: minor
- category: terminology
- claim: "\noindent\textbf{Short-immediate LOAD:} Opcode \texttt{0x27} has no public assembly syntax and is undocumented."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:253 assigns 0x27 to `LoadRegisterFromShortImmediate`, and sirc-vm/peripheral-cpu/tests/instructions/arithmetic_short_immediate_test.rs:464-500 fixes its behaviour (rD = the zero-extended 8-bit immediate). `docs/reference/STYLE.md:25` reserves "undocumented" for "a specific, currently-unassigned opcode"; 0x27 is assigned and its behaviour is described in `appendix-c-undocumented.tex:50`.
- resolution: manual-wrong
- fix: Replace with "\noindent\textbf{Short-immediate LOAD:} Opcode \texttt{0x27} is a defined encoding (\texttt{rD = \#imm8}, zero-extended) that has no public assembly syntax; see Appendix~\ref{app:undocumented}." Apply the same wording at lines 717 and 739.
- confidence: medium
- status: rejected
- ruling: Superseded by Gate 2 H: 0x27 stays "undocumented". Do not apply.

### F-alu-14
- file: chapters/13-alu-instructions.tex
- lines: 126-127
- severity: minor
- category: fact
- claim: "Processing-unit encodings outside the documented ALU forms are reserved or architecturally undefined; they are not required to raise an invalid-opcode fault."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/encoding.rs:726-739 (all 64 opcodes decode); sirc-vm/peripheral-cpu/src/lib.rs:375-394 (invalid-opcode faults are raised only for coprocessor IDs with no attached unit, never for a processing-unit encoding). "Reserved or architecturally undefined" also mixes two terms that `docs/reference/STYLE.md:25` requires be kept distinct.
- resolution: manual-wrong
- fix: Replace with "Every 6-bit opcode decodes; no processing-unit encoding raises an invalid-opcode fault. Encodings outside the documented ALU forms execute with architecturally undefined results (see Appendix~\ref{app:undocumented})."
- confidence: high
- status: accepted

### F-alu-15
- file: chapters/13-alu-instructions.tex
- lines: 1014
- severity: minor
- category: fact
- claim: "\item Shift operations can be combined with ALU ops for powerful single-cycle operations"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:66-67 (`CYCLES_PER_INSTRUCTION = 6`); contradicted by this chapter's own `Timing:` lines (for example line 288, "6 cycles, plus instruction-fetch wait states").
- resolution: manual-wrong
- fix: Replace "single-cycle operations" with "single-instruction operations".
- confidence: high
- status: accepted

### F-alu-16
- file: chapters/13-alu-instructions.tex
- lines: 174-178
- severity: minor
- category: fact
- claim: "\item \texttt{INSTR[A]} -- Update flags from ALU result (default behavior) \item \texttt{INSTR[S]} -- Update flags from shift result instead of ALU \item \texttt{INSTR[N]} -- Do not update status flags"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:96-102 — the AF field has four encodings: 0 None, 1 Alu, 2 Shift, 3 Reserved; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:164-179 leaves the SR unchanged for the reserved encoding. The chapter never states that a fourth AF encoding exists, so an emulator writer working from this chapter has no rule for it.
- resolution: manual-wrong
- fix: Add a fourth item: "\item AF encoding 11 (binary) is reserved and has no assembly syntax; the status register is left unchanged." (This is also the "Instruction Fields" AF entry required by `docs/reference/STYLE.md:157-161`.)
- confidence: medium
- status: accepted

### F-alu-17
- file: chapters/13-alu-instructions.tex
- lines: 241
- severity: minor
- category: fact
- claim: "\item All shift types are supported: LSL, LSR, ASL, ASR, RTL, and RTR."
- evidence: sirc-vm/toolchain/src/parsers/instruction.rs:359-380 — the assembler's shift-type tag set is `NUL LSL LSR ASL ASR RTL RTR`; `NUL` (encoding 0) is accepted, so `SHFT r1, NUL #0` assembles. Encoding 7 is Reserved (definitions.rs:71-81) and has no mnemonic.
- resolution: manual-wrong
- fix: Replace with "\item All six shift types are supported, plus \mnemonic{NUL}: NUL, LSL, LSR, ASL, ASR, RTL, and RTR. \texttt{SHFT rD, NUL \#0} sets the flags from the unshifted value of \texttt{rD}."
- confidence: medium
- status: accepted

### F-alu-18
- file: chapters/13-alu-instructions.tex
- lines: 254, 256
- severity: minor
- category: fact
- claim: "ADDI rD, #imm8, shift          ; rD = (rD << shift) + imm8"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:322-508 — `shift` may be LSR, ASR, RTL, or RTR, none of which is a left shift, so the `<<` in the pseudo-comment is wrong for five of the seven shift types. The chapter states the correct general rule at lines 110-112 but every Syntax block contradicts it notationally.
- resolution: manual-wrong
- fix: Change every `(rX << shift)` in a Syntax comment to `shift(rX)`. Affected lines: 254, 256, 324, 326, 387, 389, 452, 454, 513, 515, 580, 582, 650, 652, 793, 795, 871, 873, 945, 947.
- confidence: high
- status: accepted

### F-con-51
- file: chapters/13-alu-instructions.tex
- lines: 627-630
- severity: minor
- category: contradiction
- claim: "; Set bit N (variable) \\ ADDI r4, #1 \\ SHFT r4, LSL r5            ; r4 = 1 << r5"
- evidence: `ADDI r4, #1` adds 1 to whatever r4 already holds; the comment on the next line assumes r4 == 1. The equivalent example in Chapter 9 uses the correct instruction: chapters/09-shift-operations.tex:377 "LOAD r3, #1               ; r3 = 1" and :382, :389.
- resolution: manual-wrong
- fix: Change `ADDI r4, #1` to `LOAD r4, #1                ; r4 = 1`.
- confidence: high
- status: accepted

### G1-T4-13
- file: chapters/13-alu-instructions.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-mem-10
- file: chapters/14-memory-instructions.tex
- lines: 15-21
- severity: minor
- category: fact
- claim: "Both instructions support four memory addressing forms:" followed by a list including "Post-increment load" and "Pre-decrement store"
- evidence: docs/reference/review/facts.md opcode map 0x10--0x17; chapters/14-memory-instructions.tex:68-69 states the correct rule
- resolution: manual-wrong
- fix: Rewrite as "Each instruction supports four addressing forms: indirect with immediate displacement, indirect with register displacement, and one auto-update form -- post-increment for \mnemonic{LOAD}, pre-decrement for \mnemonic{STOR} -- with immediate or register displacement." As written, "Both instructions support four" contradicts the list, in which two of the four items are single-instruction forms.
- confidence: high
- status: accepted

### F-mem-11
- file: chapters/14-memory-instructions.tex
- lines: 53-59
- severity: minor
- category: terminology
- claim: "LOAD & \texttt{LOAD rD, (rO, addr)[, shift]} & 0x15 & Register offset; optional load-result shift"
- evidence: chapters/14-memory-instructions.tex:132 (`LOAD rD, (rS, addr)`), :213 (`STOR (rD, addr), rS`)
- resolution: manual-wrong
- fix: One placeholder name per operand role, chapter-wide. The Legal Forms table calls the displacement register `rO`, the LOAD syntax block calls it `rS`, and the STOR syntax block calls it `rD` while using `rS` for the store source -- so `rD` denotes a destination in one entry and a displacement in the next. Pick one (suggest `rO` for the displacement register everywhere) and apply it at lines 53, 55, 57, 59, 132, 135, 138-139, 213, 216, 219-220, 225.
- confidence: high
- status: accepted

### G1-T4-14
- file: chapters/14-memory-instructions.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-con-54
- file: chapters/15-control-flow.tex
- lines: 481-482
- severity: minor
- category: contradiction
- claim: "Because \mnemonic{LJSR} implicitly writes \reg{l} and \reg{p}, post-increment source registers cannot be \reg{l} or \reg{p}."
- evidence: The same chapter and Appendix C classify the same situation as architecturally undefined behaviour that the assembler merely rejects, not as an architectural prohibition: chapters/15-control-flow.tex:71-73 "Instructions that write the same address-register pair through more than one write-back path have architecturally undefined behavior. Avoid forms such as ... The assembler rejects known hazardous aliased forms."; chapters/appendix-c-undocumented.tex:20-22, :35-36.
- resolution: manual-wrong
- fix: Change "cannot be" to "must not be \reg{l} or \reg{p}; such forms are aliased address-register writes with architecturally undefined behaviour and are rejected by the assembler (see Appendix~\ref{appendix:undocumented})".
- confidence: medium
- status: accepted

### F-con-57
- file: chapters/15-control-flow.tex
- lines: 554-558
- severity: minor
- category: structure
- claim: "jump_table: \\     DW case0 - jump_table \\     DW case1 - jump_table"
- evidence: `DW` is used here and nowhere else in the manual; no chapter defines assembler directives. The surrounding example is also misleading: `LDEA p, (r2, p)` (chapters/15-control-flow.tex:552) computes `p = p + r2` and never reads the table, so the `DW` entries are unused.
- resolution: manual-wrong
- fix: Either drop the `DW` lines and re-comment the example as a computed branch into a run of branch instructions, or add a short "Assembler directives used in examples" note to Chapter 11 defining `DW`, `:label` and `@label`.
- confidence: high
- status: accepted

### F-mem-19
- file: chapters/15-control-flow.tex
- lines: 144
- severity: minor
- category: fact
- claim: "\texttt{\#disp} is a 16-bit PC-relative word displacement."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/branching.rs:41-52 (`BRAN|!= #-4` encodes 0xFFFC); sirc-vm/toolchain/src/bin/linker.rs:105-114 (the linker range-checks the label displacement as `i16`); sirc-vm/peripheral-cpu/tests/instructions/ljmp_test.rs:271-286 (the add wraps within the segment)
- resolution: manual-wrong
- fix: State the signedness and the wrap: "\texttt{\#disp} is a signed 16-bit word displacement (--32768 to +32767) added to the low word of \reg{p} modulo 2^16. A branch cannot change the program segment; the displacement wraps within the current 64K segment." The same sentence is repeated for \mnemonic{BRSR} at line 194.
- confidence: high
- status: accepted

### F-mem-20
- file: chapters/15-control-flow.tex
- lines: 501-503
- severity: minor
- category: contradiction
- claim: "LOAD ah, #0x0040" / "LOAD al, #0x0000              ; a = 0x00400000"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:21-27 (`ah` is a privileged destination); chapters/15-control-flow.tex:94 ("Direct writes to high address registers also raise a privilege-violation fault")
- resolution: manual-wrong
- fix: The example contradicts the chapter's own privilege rule when read in protected mode. Add a comment to the listing, for example `; supervisor mode only -- writing ah is privileged`, or build the segment through a supervisor-provided pointer.
- confidence: high
- status: accepted

### G1-T4-15
- file: chapters/15-control-flow.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-con-36
- file: chapters/16-coprocessor-instructions.tex
- lines: 140
- severity: minor
- category: contradiction
- claim: "0x1         & Exception Unit       & Yes               & 0x0--0xD            & Handles reset, faults, interrupts, traps, and links"
- evidence: chapters/06-exceptions.tex:684-688 documents exception-unit operations 0xE (Fault) and 0xF (HardwareException) as real, internally-invoked operations of the same coprocessor, plus 0x0 (None).
- resolution: manual-wrong
- fix: Change the Operations cell to "0x0--0xD (software); 0xE--0xF internal" and add a footnote pointing at chapters/06-exceptions.tex §"Internal Instructions".
- confidence: high
- status: accepted

### F-con-42
- file: chapters/16-coprocessor-instructions.tex
- lines: 328
- severity: minor
- category: contradiction
- claim: "LOAD al, @fixed_address   ; Correct the address"
- evidence: `@label` is defined only as a branch/call operand resolved by the linker to a PC-relative displacement (chapters/15-control-flow.tex:42, :144-145, :194-195; chapters/08-addressing-modes.tex:37 lists `@label` only under "branch meta-instructions"). The legal LOAD forms are `LOAD rD, #imm16` and `LOAD rD, rS` (chapters/08-addressing-modes.tex:61, chapters/13-alu-instructions.tex:78-79).
- resolution: manual-wrong
- fix: Change to `LOAD al, #0x0100          ; Correct the address`, or define an absolute-label immediate form for `LOAD` in Chapter 8 and Chapter 13 if the assembler really supports one.
- confidence: medium
- status: accepted

### F-con-49
- file: chapters/16-coprocessor-instructions.tex
- lines: 209-215
- severity: minor
- category: contradiction
- claim: "\textbf{Exceptions:} Dispatches a software exception through the exception unit. ... \textbf{Privilege:} User-callable in protected mode and supervisor mode."
- evidence: Two normative rules from Chapter 6 are absent from the EXCP entry: chapters/06-exceptions.tex:98 lists "Triggering a software exception with a vector below 0x60" as a privilege-violation fault, and chapters/06-exceptions.tex:142-144 "If an \mnemonic{EXCP} instruction is executed while any exception or fault handler is already active, the software exception is ignored, is not queued, and does not fetch its vector."
- resolution: manual-wrong
- fix: Add to the Exceptions field: "Raises a privilege-violation fault in protected mode if the selected vector is below 0x60. If any exception or fault handler is already active, the software exception is ignored and no vector is fetched (see Chapter~\ref{ch:exceptions})."
- confidence: high
- status: accepted

### F-con-58
- file: chapters/16-coprocessor-instructions.tex
- lines: 419
- severity: minor
- category: structure
- claim: "\textbf{Clobbers:} Writes \reg{r1} through \reg{rn}."
- evidence: `Clobbers` is used as an instruction-entry field at chapters/16-coprocessor-instructions.tex:419, :467 and :510 but is not among the fields defined in chapters/11-reading-instructions.tex:9-25 and is not in the canonical field order at docs/reference/STYLE.md:63-65.
- resolution: manual-wrong
- fix: Add `\item[Clobbers] Registers that the instruction overwrites beyond its named destination.` to the Chapter 11 field list and place it in the canonical order (after `Write-back`), or fold the information into the `Write-back` field of the three DMA entries.
- confidence: high
- status: accepted

### F-cop-19
- file: chapters/16-coprocessor-instructions.tex
- lines: 33-34
- severity: minor
- category: fact
- claim: "The CPU encoding reserves register and shift fields inherited from the immediate and register instruction formats."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:172-176, 188 (in Immediate format decode forces shift type None and count 0; there are no shift fields to reserve); digest facts.md:98
- resolution: manual-wrong
- fix: "The CPU encoding reserves the register field of the immediate format (opcode 0x0F) and the r1, r2 and shift fields of the register format (opcode 0x3F). The immediate format has no shift fields."
- confidence: high
- status: accepted

### F-cop-20
- file: chapters/16-coprocessor-instructions.tex
- lines: 78-80
- severity: minor
- category: fact
- claim: "\mnemonic{COPR} takes a general-purpose source register whose 16-bit value is used as the command operand."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/coprocessor.rs:108-127 (the `COPR` arm accepts any `AddressingMode::DirectRegister`, which covers all 16 register names including `sr`, `lh`, `ph`); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:45-52 (a protected-mode read of `sr` is redacted to its low byte)
- resolution: manual-wrong
- fix: "\mnemonic{COPR} takes any register name as its source; its 16-bit value is used as the command operand. A protected-mode \mnemonic{COPR sr} reads only the low byte of \reg{sr} and therefore always builds a coprocessor-0 command." Or, if only \reg{r1}--\reg{r7} are intended to be legal, the assembler must reject the other names.
- confidence: medium
- status: accepted

### F-cop-21
- file: chapters/16-coprocessor-instructions.tex
- lines: 61-62
- severity: minor
- category: fact
- claim: "Coprocessor-call instructions do not perform data-memory access and do not raise data bus, data bus-protection, data alignment, or segment-overflow faults in documented forms."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:92-97, 127-137 (a program-counter wrap with \texttt{SR.A} set raises a SegmentOverflow fault at the *next* fetch, for every instruction class including coprocessor calls); digest facts.md:261
- resolution: manual-wrong
- fix: Qualify the exclusion: "...do not raise data bus, data bus-protection, data alignment, or effective-address segment-overflow faults. A program-counter wrap with \texttt{SR.A} set still raises a segment-overflow fault at the following instruction fetch, as it does for every instruction."
- confidence: high
- status: accepted

### F-cop-22
- file: chapters/16-coprocessor-instructions.tex
- lines: 157-158
- severity: minor
- category: fact
- claim: "Software may probe optional coprocessors by executing the documented operation and handling an invalid-opcode fault, or by using a model-specific capability table if one is provided by system firmware."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:375-394 (the offending 16-bit coprocessor command word is recorded in the fault-metadata link register's `return_address` field before the command is cleared); sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:819-822, 943-947; digest facts.md:288
- resolution: manual-wrong
- fix: Add: "The invalid-opcode fault records the offending 16-bit coprocessor command word in the \texttt{return\_address} field of the fault-metadata link register (link register \imm{7}), which an emulation handler reads with \mnemonic{ETFR}. The pending command is cleared before the handler runs." Without this, the recommended emulation strategy is not implementable.
- confidence: high
- status: accepted

### F-cop-23
- file: chapters/16-coprocessor-instructions.tex
- lines: 370
- severity: minor
- category: fact
- claim: "A zero count is encoded with the forward direction bit."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/coprocessor_meta.rs:152-158 (`let direction = if signed_count < 0 { DMA_REGISTER_DIRECTION_BIT } else { 0 };`, so a zero count clears bit 5); sirc-vm/toolchain/tests/assembler/coprocessor_test.rs:135 (`assert_copi_immediate("DMAR a, #0\n", 0x2800, ...)`)
- resolution: manual-wrong
- fix: "A zero count is encoded with the direction bit clear (forward)." The current wording reads as if a bit is set.
- confidence: high
- status: accepted

### F-cop-24
- file: chapters/16-coprocessor-instructions.tex
- lines: 202
- severity: minor
- category: terminology
- claim: "Used by user-mode programs to invoke system calls or request supervisor-mode services."
- evidence: docs/reference/STYLE.md:15 ("protected mode" preferred; "user mode" is now unused, 0 hits, resolved in handover Workstream 1)
- resolution: manual-wrong
- fix: "Used by programs running in protected mode to invoke system calls or request supervisor-mode services."
- confidence: high
- status: accepted

### G1-T4-16
- file: chapters/16-coprocessor-instructions.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-con-38
- file: chapters/17-meta-instructions.tex
- lines: 14-33
- severity: minor
- category: contradiction
- claim: "\textbf{Meta-Instruction} & \textbf{Primary Documentation}    & \textbf{Lowers To}"
- evidence: `tab:meta-instruction-cross-reference` lists 19 meta-instructions but omits \mnemonic{NOOP}, which is documented in the very next section of the same chapter (chapters/17-meta-instructions.tex:40-44) and is listed in Chapter 12's meta-instruction table (chapters/12-instruction-summary.tex:137) and in Appendix E (chapters/appendix-e-quick-reference.tex:65).
- resolution: manual-wrong
- fix: Insert a row `\mnemonic{NOOP}           & Chapter~\ref{ch:meta-instructions} & \mnemonic{ADDI[N] r1, \#0}  \\` in alphabetical position.
- confidence: high
- status: rejected
- ruling: Superseded by F-cop-25 (NOOP lowers to ADDI[N] sr, #0, not r1). Do not apply.

### F-cop-25
- file: chapters/17-meta-instructions.tex
- lines: 13-33
- severity: minor
- category: structure
- claim: "\textbf{Meta-Instruction} & \textbf{Primary Documentation}    & \textbf{Lowers To}"
- evidence: docs/reference/chapters/17-meta-instructions.tex:40-74 (\mnemonic{NOOP} is a meta-instruction documented in this chapter but is absent from the cross-reference table); digest facts.md:157 lists `NOOP` among the assembler's meta-instruction aliases
- resolution: manual-wrong
- fix: Add a `\mnemonic{NOOP}` row citing Chapter~\ref{ch:meta-instructions} as its primary documentation and `\mnemonic{ADDI[N] sr, \#0}` as what it lowers to, so the table is a complete list of the 20 meta-instructions the assembler accepts.
- confidence: high
- status: accepted

### G1-T4-17
- file: chapters/17-meta-instructions.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-sum-24
- file: chapters/appendix-a-opcode-map.tex
- lines: 140-141
- severity: minor
- category: terminology
- claim: "These opcodes may execute in hardware but their behavior is implementation-defined and may change between CPU revisions."
- evidence: STYLE.md section 1 names this exact line: undocumented opcodes are valid encodings with architecturally undefined behaviour, and "implementation-defined" is reserved for genuine per-model timing variance
- resolution: manual-wrong
- fix: Replace with "These opcodes execute in hardware, but their behavior is architecturally undefined and may change between CPU revisions."
- confidence: high
- status: accepted

### G1-T4-ap
- file: chapters/appendix-a-opcode-map.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-con-39
- file: chapters/appendix-b-timing.tex
- lines: 156-163
- severity: minor
- category: contradiction
- claim: "; Instead of: \\ LOAD r1, (#0, a) \\ ADDI a, #2 \\ \\ ; Use: \\ LOAD r1, (#0, a)+          ; Same result, fewer cycles"
- evidence: Two problems. (1) `a` is an address-register *pair*, not an ALU-addressable register; the 4-bit register IDs are `al` = 0xB and `ah` = 0xA (chapters/03-registers.tex:248-249), so `ADDI a, #2` has no encoding. (2) Post-increment advances by exactly one word, not two: chapters/08-addressing-modes.tex:124-125 "Post-increment and pre-decrement change the selected address register by exactly one word"; chapters/14-memory-instructions.tex:81. The two sequences therefore do not produce the "Same result".
- resolution: manual-wrong
- fix: Change the "Instead of" block to `LOAD r1, (#0, a)` / `ADDI al, #1` and keep the "Use" block as-is.
- confidence: high
- status: accepted

### F-con-40
- file: chapters/appendix-b-timing.tex
- lines: 133-143
- severity: minor
- category: contradiction
- claim: "CMPR r1, r2 \\ BRAN|<= @skip \\ ADDI r3, #1 \\ skip: \\ \\ ; Use: \\ CMPR r1, r2 \\ ADDI|HI r3, #1             ; Saves branch overhead"
- evidence: `<=` is the *signed* less-or-equal condition and `HI` is the *unsigned* higher condition (chapters/10-condition-codes.tex:30-35, :263-270), so the two sequences are not equivalent. chapters/10-condition-codes.tex:293 warns about exactly this: "\textbf{Signed vs. Unsigned}: Using unsigned conditions (\texttt{HI}, \texttt{LO}) for signed values or vice versa produces incorrect results".
- resolution: manual-wrong
- fix: Change `BRAN|<= @skip` to `BRAN|LO @skip` so both halves use unsigned conditions.
- confidence: high
- status: accepted
- ruling: Per Gate 2 B the glosses stand; make both halves unsigned as proposed.

### F-tim-12
- file: chapters/appendix-b-timing.tex
- lines: 121-127
- severity: minor
- category: fact
- claim: "Each wait state adds one clock cycle to the instruction"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:89-117 (two instruction-fetch bus cycles per instruction, phases 0 and 1); stages/memory_access.rs:56-74 (one further bus cycle only for opcodes 0x10--0x17); facts.md:322-324
- resolution: manual-wrong
- fix: True per wait state but not enough to compute an instruction's cost. Add the bus-access count per class: every instruction performs two instruction-fetch accesses; loads and stores add one data access; a coprocessor dispatch adds two vector-fetch accesses. So a system inserting *w* wait states per access adds 2w cycles to an ALU instruction and 3w to a load or store.
- confidence: high
- status: accepted

### F-tim-13
- file: chapters/appendix-b-timing.tex
- lines: 204-205
- severity: minor
- category: fact
- claim: "ARM6           & 1.0              & Single-cycle with pipeline" / "MIPS R2000     & 1.0              & Single-cycle with pipeline"
- evidence: none (external historical claim; not checkable against this repository)
- resolution: unclear
- fix: An average CPI of exactly 1.0 is not attainable on either part: ARM6 loads take 3 cycles and taken branches 3 cycles, and R2000 loads/branches cost extra cycles on a cache or interlock stall. Either give a range in the same style as the 6502 and 68000 rows (for example "1--3" and "1--2" with the note "1 cycle for register ALU operations; loads and taken branches cost more"), or mark the column "Best-case CPI" so the SIRC-1 row (a true fixed 6.0) is not compared against best cases.
- confidence: medium
- status: defer
- ruling: Gate 2 Q: author to check the hardware figures.

### G1-T4-ap
- file: chapters/appendix-b-timing.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-exc-26
- file: chapters/appendix-c-undocumented.tex
- lines: 212
- severity: minor
- category: fact
- claim: "\item Reserved shift type (0x111) could enable additional shift modes"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:69-81 — `ShiftType::Reserved` is the eighth variant, value 7, i.e. binary 111 in the 3-bit shift-type field; docs/reference/STYLE.md:49 requires bare digit groups for binary literals and forbids a `0x` prefix on them
- resolution: manual-wrong
- fix: "Reserved shift type (encoding 111) could enable additional shift modes". As written, `0x111` reads as hexadecimal 273, which is not a legal value of a 3-bit field.
- confidence: high
- status: accepted

### F-exc-27
- file: chapters/appendix-c-undocumented.tex
- lines: 120
- severity: minor
- category: fact
- claim: "In the current SIRC-VM simulator (as of version 1.0):"
- evidence: sirc-vm/peripheral-cpu/Cargo.toml:3 — `version = "0.1.0"`
- resolution: manual-wrong
- fix: Drop the version number ("In the current SIRC-VM simulator:") or replace it with the crate version actually shipped. A version pin that does not match any released artefact tells a reader nothing.
- confidence: medium
- status: accepted

### G1-T4-ap
- file: chapters/appendix-c-undocumented.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-sum-29
- file: chapters/appendix-d-examples.tex
- lines: 8-49
- severity: minor
- category: structure
- claim: "\lstinputlisting[ caption={Byte sieve example}, label={lst:example-byte-sieve} ]"
- evidence: STYLE.md "Approved decisions" (Prose): "Examples are numbered. Each Example: block becomes Example N-M where N is ... the appendix letter and M counts from 1 within the chapter."
- resolution: n/a
- fix: Number the six listings D-1 through D-6 in their captions (for example caption={Example D-1: Byte sieve}) and refer to them by number in the section text.
- confidence: medium
- status: accepted

### F-sum-30
- file: chapters/appendix-d-examples.tex
- lines: 8-11
- severity: minor
- category: fact
- claim: the included listing begins "; Reserved space for 128x32 bit exception vectors" followed by ".ORG 0x0200" for the first code
- evidence: examples/byte-sieve/byte-sieve.sasm:5-9, repeated at examples/faults/faults.sasm:19, examples/hardware-exception/hardware-exception.sasm:1, examples/software-exception/software-exception.sasm:1 and examples/store-load/store-load.sasm:1. The vector number is 8 bits and the vector address is vector*2 (EEX:304-325), so the table spans word addresses 0x000-0x1FF, which is 256 vectors of 32 bits, exactly what the .ORG 0x0200 in the listings reserves. facts.md Open question 3 records that EDEF:6-9 says 128 vectors while the encoding allows 256
- resolution: code-wrong
- fix: Settle the vector count once (facts.md Open question 3) and correct the comment in all five example sources to match; as written the listings state 128 while reserving room for 256.
- confidence: medium
- status: code-wrong
- ruling: Gate 2 P: vector table is 256 entries (8-bit field; 0x00--0x5F reserved, 0x60--0xFF user). The comment lives in examples/*.sasm sources, outside the chapters.

### F-sum-31
- file: chapters/appendix-d-examples.tex
- lines: 27-30
- severity: minor
- category: fact
- claim: the included listing states "; Enable all hardware interrupts (set bits 9-13 of SR)" above "ORRI sr, #0b0001_1110_0000_0000"
- evidence: examples/hardware-exception/hardware-exception.sasm:43-44. The constant 0x1E00 sets bits 9-12 only. Bit 13 is EA, which is set and cleared by the exception unit and cannot be changed by a direct SR write at all (WB:55-72; REG:33-35); the enable bits are 9-12 (REG:25-32, 507-516)
- resolution: manual-wrong
- fix: Correct the comment in examples/hardware-exception/hardware-exception.sasm:43 to "; Enable all four maskable hardware interrupt lines (bits 9-12 of SR)".
- confidence: high
- status: accepted

### F-sum-32
- file: chapters/appendix-d-examples.tex
- lines: 15-18
- severity: minor
- category: fact
- claim: the included listing defines ".EQU $CPU_PHASE_INSTRUCTION_FETCH  #0x0000" and ".EQU $CPU_PHASE_EFFECTIVE_ADDRESS  #0x0003" and then comments "; Mask off the phase section of the status register"
- evidence: examples/faults/faults.sasm:15-16, 168-181. Bits 2:0 of the fault metadata register hold the bus access type, not a CPU phase (LIB:81-88, 120-145; BUS:19-29: 0 None, 1 InstructionFetch, 2 DataRead, 3 DataWrite), and the register read by ETFR r7, #7 is the fault metadata link register, not the status register. Under the real encoding the constant named INSTRUCTION_FETCH holds the code for None and the one named EFFECTIVE_ADDRESS holds the code for DataWrite
- resolution: manual-wrong
- fix: Rename the constants in examples/faults/faults.sasm to $BAT_NONE / $BAT_DATA_WRITE (or to whatever the fault-metadata chapter calls them) and change the comments to say "fault metadata register" and "bus access type", so the published example does not teach a field that does not exist.
- confidence: medium
- status: accepted

### G1-T4-ap
- file: chapters/appendix-d-examples.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-con-44
- file: chapters/appendix-e-quick-reference.tex
- lines: 91
- severity: minor
- category: contradiction
- claim: "\mnemonic{WAIT}   & Meta          & COPI \#0x1900        & --             & Halt until an interrupt fires"
- evidence: chapters/16-coprocessor-instructions.tex:231 "Puts the CPU into wait state until an exception occurs."; chapters/06-exceptions.tex:667 "Enter wait state until an exception occurs". Faults and traps are exceptions but not interrupts, so "interrupt" is narrower than the normative definition.
- resolution: manual-wrong
- fix: Change the description to "Wait until an exception occurs".
- confidence: high
- status: accepted

### F-con-48
- file: chapters/appendix-e-quick-reference.tex
- lines: 169
- severity: minor
- category: contradiction
- claim: "See Chapter~\ref{ch:cpu-architecture} (section on External Bus Signals) for usage examples including the PROT pin interaction."
- evidence: Chapter 2 has no section named "External Bus Signals". Its sections are "External Interface" -> "Chip Layout", "Pin Descriptions", "Timing Notes" (chapters/02-cpu-architecture.tex:167, :169, :315, :590). The BAT description lives at chapters/02-cpu-architecture.tex:441-474.
- resolution: manual-wrong
- fix: Change to "See Chapter~\ref{ch:cpu-architecture}, Section \"Pin Descriptions\", for usage examples including the PROT pin interaction."
- confidence: high
- status: accepted

### F-sum-25
- file: chapters/appendix-e-quick-reference.tex
- lines: 13-14
- severity: minor
- category: terminology
- claim: "\emph{Flags}: flags written by the instruction (\texttt{--} = none; \texttt{*} = determined by the coprocessor)"
- evidence: chapters/11-reading-instructions.tex:60 defines "*" as "Updated from the instruction result", and STYLE.md section 3 requires the Chapter 11 symbol set to be used exactly, with no new symbols invented
- resolution: manual-wrong
- fix: Use a different marker for coprocessor-determined flags (for example a dagger with a footnote) or spell the cell as "COP" so that "*" keeps its Chapter 11 meaning.
- confidence: high
- status: accepted

### F-sum-26
- file: chapters/appendix-e-quick-reference.tex
- lines: 58
- severity: minor
- category: fact
- claim: "\mnemonic{LOAD} & Mem & 0x07/0x14--0x17/0x37 & -- & Load from memory or register"
- evidence: 0x07 (LoadRegisterFromImmediate) and 0x37 (LoadRegisterFromRegister) never assert the bus: they are ALU-class encodings whose write-back class is AluToRegister (WB:132-151; ALU:307-317). Only 0x14--0x17 perform a data read (MEM:56-74)
- resolution: manual-wrong
- fix: Split the row, or change the Type cell to "Mem/ALU" and the Description to "Load from memory (0x14--0x17) or move an immediate or register value (0x07, 0x37)".
- confidence: high
- status: accepted

### F-sum-27
- file: chapters/appendix-e-quick-reference.tex
- lines: 91
- severity: minor
- category: fact
- claim: "\mnemonic{WAIT} & Meta & COPI \#0x1900 & -- & Halt until an interrupt fires"
- evidence: LIB:300-302, 514-515 and T-RESET:227-248: WAIT idles until an interrupt that is enabled in the SR is asserted, or until reset. A disabled line does not wake the CPU, and "halt" is the manual name for the HALT pin state (LIB:283-293), which is a different mechanism
- resolution: manual-wrong
- fix: Replace the Description with "Idle until an enabled hardware exception or a reset occurs".
- confidence: high
- status: accepted

### F-sum-28
- file: chapters/appendix-e-quick-reference.tex
- lines: 107-108
- severity: minor
- category: fact
- claim: "The upper byte (bits 15--8) is privileged: reads in protected mode return zero for those bits; direct writes in protected mode raise a privilege violation fault."
- evidence: PU:21-27 and 29-51 with T-PROT:95-132: any instruction whose destination is sr raises PrivilegeViolation in protected mode, whether or not the value would change the high byte. The sentence attaches the write rule to the upper byte only. STYLE.md section 1 also prefers "high byte" to "upper byte"
- resolution: manual-wrong
- fix: Replace with "The high byte (bits 15--8) is privileged: reads in protected mode return zero for those bits. In protected mode any instruction that names \reg{sr} as its destination raises a privilege violation fault, regardless of which bits the write would change."
- confidence: high
- status: accepted

### G1-T4-ap
- file: chapters/appendix-e-quick-reference.tex
- lines: entry template
- severity: minor
- category: template
- claim: (Gate 1 template decision)
- evidence: review/gate1-decisions.md
- resolution: manual-wrong
- fix: Number every example block `Example N-M:` per STYLE.md (N = chapter number or appendix letter, M counts from 1 within the chapter).
- confidence: high
- status: accepted

### F-con-66
- file: generated/immediate-format-encodings.tex
- lines: multiple
- severity: minor
- category: terminology
- claim: "0b01"
- evidence: docs/reference/STYLE.md:49 "Binary literals: bare digit groups in tables (`00`, `10`, `111`), no `0b` prefix. The two existing `0b10` instances in `07-instruction-formats.tex` are the outliers; convert to the bare form used everywhere else." The AF column of all three generated tables uses `0b01`/`0b10`/`0b11` while the parenthetical decode row directly beneath each one uses the bare form `(01)`/`(10)`/`(11)`.
- resolution: manual-wrong
- fix: `s/0b\([01][01]\)/\1/g` across generated/immediate-format-encodings.tex, generated/short-immediate-format-encodings.tex, generated/register-format-encodings.tex and chapters/07-instruction-formats.tex:167,252. Note the three `generated/` files are machine-produced, so the generator must be changed rather than the output patched.
- confidence: high
- status: accepted

### F-con-33
- file: generated/register-format-encodings.tex
- lines: 50-51
- severity: minor
- category: contradiction
- claim: "\texttt{SHFT r1, ASL \#3}           & 0x25        & 0x1         & 0x0         & 0x0         & 0           & 011         & 0x3         & 0b10        & 0x0           & \texttt{0x94400CE0}"
- evidence: This row appears in `tab:register-format-examples` (caption "Register Format Encoding Examples", generated/register-format-encodings.tex:54) with R1/R2/R3 column headings, but 0x25 is the *short immediate* opcode (chapters/12-instruction-summary.tex:64, chapters/appendix-a-opcode-map.tex:60) and the hex decodes as a short-immediate word (bits 21--14 are the 8-bit immediate 0x00, not R2/R3). Chapter 7's own note under the register table concedes this: chapters/07-instruction-formats.tex:252 "SHFT is a meta instruction using ORRI opcode (0x25) with AF=0b10". The same instruction is already correctly shown in generated/short-immediate-format-encodings.tex:26.
- resolution: manual-wrong
- fix: Remove the `SHFT r1, ASL #3` row from the register-format table (and note 07-instruction-formats.tex:252, which only exists to explain it); the short-immediate table already carries the `SHFT` example.
- confidence: high
- status: accepted

### F-arch-16
- file: chapters/01-introduction.tex
- lines: 141
- severity: nit
- category: latex
- claim: "this allows access to $2^24$ = 16,777,216 words, or 32 megabytes of external byte storage"
- evidence: none
- resolution: manual-wrong
- fix: `$2^24$` typesets as 2 squared followed by a 4. Use `$2^{24}$`. (The arithmetic itself is correct: 16,777,216 words = 32 MiB of byte storage.)
- confidence: high
- status: accepted

### F-arch-17
- file: chapters/02-cpu-architecture.tex
- lines: 452, 469-470
- severity: nit
- category: terminology
- claim: "1             & 0             & 1             & DMA Co-processor Read Burst  \\"
- evidence: docs/reference/STYLE.md:22 ("coprocessor | co-processor | 155 vs 7; drop the hyphen everywhere, including headings")
- resolution: manual-wrong
- fix: Replace "Co-processor" with "coprocessor" in the BAT table rows (lines 469--470) and in the BAT example prose at line 452 ("the DMA coprocessor will be reading").
- confidence: high
- status: accepted

### F-sum-33
- file: chapters/appendix-a-opcode-map.tex
- lines: 11-14, 90-91
- severity: nit
- category: latex
- claim: "\begin{tabular}{|c|l|l|l|} \hline"
- evidence: STYLE.md section 6 requires booktabs rules for every table and forbids vertical rules and hand-drawn \hline; the two tables in this appendix are also the only ones in the group without a \label
- resolution: n/a
- fix: Convert the opcode map to booktabs (\toprule/\midrule/\bottomrule, no vertical rules) and add \label{tab:opcode-map} and \label{tab:instruction-variant-quick-reference} so Appendix E can cite them by number.
- confidence: high
- status: accepted

### F-tim-14
- file: chapters/appendix-b-timing.tex
- lines: 43-44, 202-203
- severity: nit
- category: style
- claim: "DMAR/DMAW                  & $6 + \\max(|n|, 1)$" / "MOS 6502       & 2-7" / "Motorola 68000 & 4-158"
- evidence: docs/reference/STYLE.md:51 (numeric ranges take an en dash `--`), STYLE.md:52 (mnemonics always inside `\mnemonic{}`)
- resolution: n/a
- fix: Write `2--7` and `4--158`, and set the mnemonics in the timing table as `\mnemonic{DMAR}/\mnemonic{DMAW}` and `\mnemonic{DMAT}` (the prose at lines 51--52 already uses the macro). No semantic effect.
- confidence: high

## Checks performed that found no defect

- Six-phase list (lines 17-24): matches `CSH:6-14` and `LIB:66-67` (`CYCLES_PER_INSTRUCTION = 6`) and Chapter 2's list. The appendix does **not** repeat Chapter 2's claim that the PC increments by 1 after each instruction-word fetch — it makes no PC statement at all, so the sibling reviewer's finding does not extend here. If Chapter 2's phase list is corrected, no corresponding change is needed in this appendix.
- Fetch order "High Word" then "Low Word": correct in behaviour — phase 0 reads the word at PC into instruction bits 31:16 and phase 1 reads PC+1 into bits 15:0 (`PU:89-108`), even though the code's enum names those phases `InstructionFetchLow` and `InstructionFetchHigh`.
- DMA rows `6 + max(|n|, 1)` and `6 + max(2n, 1)`, the "at least one post-dispatch cycle" prose, and the false-condition rule are exactly the ruled decision at `docs/reference/manual-handover.md:555-559` and match `16-coprocessor-instructions.tex:429,477,523`. `DMAR`/`DMAW` counts are 0--7 and `DMAT` counts 0--255 (facts.md:313), so both formulas are in range.
- Arithmetic in the loop example: 5 instructions x 6 = 30 cycles per iteration, 6 + (10 x 30) = 306. Correct.
- `LJSR a` and `RETS` are accepted by the assembler (`A-LJSR:83-92`; `A-META`), and the 12-cycle call overhead is right.
- `HI`/`LO` are exact complements (`C && !Z` vs `!C || Z`), so "one conditional executes, one doesn't" holds — only the direction is wrong (F-tim-1).
- `f_clock / 6`: 10 MHz -> 1.67 MIPS, 25 MHz -> 4.17 MIPS. Both correct to the stated precision.
- Branch taken and not taken both 6 cycles: branches are `LDEA`/`LDEL` with `p` as destination, no extra phases (facts.md:157-158).
- "one execution phase per 16-bit word transferred" and the BACK/BERR/BPER hold rule match `LIB:106-110, 259-276` and facts.md:324.
- 6502 (2--7) and 68000 (4--158) CPI ranges are consistent with the published figures for those parts.
- status: accepted

### F-con-47
- file: chapters/appendix-c-undocumented.tex
- lines: 212
- severity: nit
- category: contradiction
- claim: "\item Reserved shift type (0x111) could enable additional shift modes"
- evidence: 111 is the *binary* reserved shift-type code, written bare in both encoding tables (chapters/07-instruction-formats.tex:305 "111 & Reserved"; chapters/09-shift-operations.tex:34 "111 & -- & Reserved"). `0x111` is a hex literal meaning 273. docs/reference/STYLE.md:49 requires bare digit groups for binary literals.
- resolution: manual-wrong
- fix: Change to "\item Reserved shift type (111) could enable additional shift modes".
- confidence: high
- status: accepted

### F-sum-34
- file: chapters/appendix-d-examples.tex
- lines: 8-11
- severity: nit
- category: fact
- claim: the included listing comments the stack segment setup as "; Array Start = 0x00F0_0000"
- evidence: examples/byte-sieve/byte-sieve.sasm:17-20: the comment is copied from the array setup above it and describes the wrong pointer; the value loaded is sh=0x00F0, sl=0xFFFF
- resolution: manual-wrong
- fix: Correct the comment in examples/byte-sieve/byte-sieve.sasm:19 to "; Stack top = 0x00F0_FFFF".
- confidence: high
- status: accepted
