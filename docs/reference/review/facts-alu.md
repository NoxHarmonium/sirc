# Fact-check findings: Chapter 13 (ALU Instructions)

Reviewer tag: `alu`. Digest: `docs/reference/review/facts.md`.

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

### F-alu-6
- file: chapters/13-alu-instructions.tex
- lines: 155, 741
- severity: major
- category: fact
- claim: "LOAD            & -          & -          & -          & -          & \texttt{[N]}        & Status flags are preserved." / "\textbf{Flags:} Preserved."
- evidence: manual side — sirc-vm/toolchain/src/parsers/opcodes/load.rs:29-37 (the assembler rejects `[A|S|N]` for LOAD) and load.rs:47, :128 (`additional_flags: 0x0`, i.e. AF = None), so every assembled LOAD preserves flags. Code side — sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:307-317 (`perform_load` calls `set_alu_bits` with `Some((a, b, result))`) and sirc-vm/peripheral-cpu/tests/instructions/arithmetic_register_test.rs:497-534 (opcode 0x37 with `StatusRegisterUpdateSource::Alu` sets N from 0xFACE and clears C, V, Z), so the LOAD *encoding* with AF = Alu does update all four flags. A third behaviour is in sirc-vm/peripheral-cpu/tests/instructions/arithmetic_immediate_test.rs:521-573 (0x07 with AF = Shift clears all four). Digest Open question 9.
- resolution: unclear
- fix: Decide which of the three is architectural and say so. If the flag behaviour of the LOAD encoding under a non-default AF is not architecturally fixed, the row must read `U U U U` with default AF `[N]` and a note: "Public \mnemonic{LOAD} syntax always encodes AF = None and preserves the flags. The result of encoding AF = Alu or AF = Shift on a LOAD opcode is architecturally undefined." If AF = Alu is architectural, the row must read `*  *  0  0` for that encoding. The same choice must be applied at line 114 (Table~\ref{tab:alu-common-semantics}), lines 78-79 (Table~\ref{tab:alu-legal-forms} "preserves flags"), line 732, and line 776.
- confidence: high

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

### F-alu-8
- file: chapters/13-alu-instructions.tex
- lines: 239-240
- severity: major
- category: fact
- claim: "Because the lowering uses the short-immediate format, \mnemonic{SHFT} has the same shift field limits as other short-immediate ALU instructions."
- evidence: manual side — the limit is never stated numerically anywhere in this chapter. Code side — sirc-vm/toolchain/src/parsers/instruction.rs:224-236 rejects an immediate shift count above `MAX_SHIFT_COUNT` (15, definitions.rs:43); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/encoding.rs:339-342 masks an immediate count to 4 bits at encode time; a *register* shift count is not limited at all (definitions.rs:85-89; fetch_and_decode.rs:60-69), and the shift functions clamp to 16 for LSL/LSR/ASL/ASR (alu.rs:372, 395, 425, 457) while rotates take the count modulo 16 (alu.rs:479, 495). Digest Open question 13 records that register counts above 15 are untested (arithmetic_register_test.rs:953-959).
- resolution: unclear
- fix: State the limit explicitly and rule on register counts, for example: "Immediate shift counts are 0--15; the assembler rejects a larger literal. A register shift count is the full 16-bit register value; counts above 15 produce a fully-shifted result (zero, or the replicated sign bit for \mnemonic{ASR}) for the shift types and are taken modulo 16 for \mnemonic{RTL} and \mnemonic{RTR}." If the author does not wish to fix the rotate/shift asymmetry in the architecture, the manual must instead say that a register shift count greater than 15 is architecturally undefined. The same gap exists at `09-shift-operations.tex:440`.
- confidence: high

### F-alu-9
- file: chapters/13-alu-instructions.tex
- lines: 95-97
- severity: major
- category: fact
- claim: "When a register-form ALU instruction omits \texttt{rS1}, the assembler encodes \texttt{rD} as both the destination and first source operand. For example, \texttt{ADDR r1, r2} is encoded as \texttt{ADDR r1, r1, r2}."
- evidence: manual side — the chapter correctly states elsewhere (lines 110-112) that the shift applies to the left operand, but the shorthand note never says what a shift does when combined with the shorthand. Code side — sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:135-155 parses `ADDR rD, rS, shift` as r1 = r2 = rD, r3 = rS, so `ADDR r1, r2, LSL #2` computes `(r1 << 2) + r2`: the shift applies to the *destination* register, not to the named source. Digest Open question 16 asks proofreaders to confirm the operand-order wording; the shorthand-plus-shift case is the one form the chapter does not cover, and Table~\ref{tab:alu-legal-forms} lists only `ADDR rD, rS1, rS2[, shift]`.
- resolution: unclear
- fix: Add a row `ADDR rD, rS2[, shift]` to Table~\ref{tab:alu-legal-forms} and extend the shorthand note: "A shift definition may be combined with the shorthand; because the shift always applies to the left operand, \texttt{ADDR r1, r2, LSL \#2} computes \texttt{(r1 << 2) + r2}, not \texttt{r1 + (r2 << 2)}." Confirm with the author that this is the intended reading before applying.
- confidence: medium

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
