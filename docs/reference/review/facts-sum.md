# Facts review — chapter group G8 (summaries and appendices)

Reviewer tag: `sum`. Chapters reviewed: `chapters/11-reading-instructions.tex`,
`chapters/12-instruction-summary.tex`, `chapters/appendix-a-opcode-map.tex`,
`chapters/appendix-d-examples.tex`, `chapters/appendix-e-quick-reference.tex`.

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

### F-sum-7
- file: chapters/appendix-d-examples.tex
- lines: 39-42
- severity: blocker
- category: contradiction
- claim: "\lstinputlisting[ caption={Software exception example} ... ]{../../examples/software-exception/software-exception.sasm}" — the included listing executes "EXCP    #0x40"
- evidence: examples/software-exception/software-exception.sasm:18; chapters/06-exceptions.tex:140 "The vectors for user exceptions are in the 0x60-0xFF range" and :98 lists "Triggering a software exception with a vector below 0x60" as a violation; sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs:112-113 USER_EXCEPTION_VECTOR_START = 0x60. The runtime guard at execution.rs:330-333 compares `vector_address_high` (which is vector*2 = 0x80 here) against 0x60, so it does not fire for vector 0x40; on the manual reading of the rule the example is illegal, on the implementation reading any vector >= 0x30 is accepted.
- resolution: unclear
- fix: Either change the example (and its vector-table .ORG from 0x0080 to 0x00C0, vector 0x60) so the manual never shows EXCP with a reserved vector, or state in Chapter 6 and Appendix D that the implementation only checks vector*2 >= 0x60. Do not publish a listing that contradicts 06-exceptions.tex:140 without a note.
- confidence: high

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

### F-sum-10
- file: chapters/11-reading-instructions.tex
- lines: 41
- severity: major
- category: fact
- claim: "\texttt{shift} & Optional SIRCIS shift definition"
- evidence: facts.md Open question 16; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:180-192 applies the shift to operand `a` only, so `ADDI r1, #2, LSL #3` computes (r1 << 3) + 2 and `ADDR r1, r2, r3, LSL #2` computes (r2 << 2) + r3 (T-ASI:507-596; T-AR)
- resolution: unclear
- fix: The notation table is the one place that defines `shift` for all of Part III, so it should state which operand the shift applies to. Proposed row text: "shift & Optional shift definition. The shift is applied to the first (register) operand before the operation; it never applies to the immediate or to the second source register." Confirm the wording against Chapter 9 before applying.
- confidence: medium

### F-sum-11
- file: chapters/11-reading-instructions.tex
- lines: 14
- severity: major
- category: fact
- claim: "\item[Operands] The permitted operand kinds and addressing modes for the instruction."
- evidence: facts.md Open question 11: the handover (HO:17) fixes seven addressing modes, but the parser distinguishes ten syntactic operand forms (sirc-vm/toolchain/src/parsers/instruction.rs:85-106) and the seven-way grouping in the digest is inferred, not cited
- resolution: unclear
- fix: Question for the author: which fixed set of addressing-mode names may an Operands field use, and does the count of seven include the (A) and (A)+ shorthands as separate modes? Chapter 11 should name the closed set (or point at the Chapter 8 table by label) so that entries in Chapters 13-17 cannot invent mode names.
- confidence: low

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

### F-sum-14
- file: chapters/12-instruction-summary.tex
- lines: 66, 74
- severity: major
- category: fact
- claim: "0x27 & LOAD & Short Imm+Shift & -- & Undocumented" and "0x2F & COPI & Short Imm+Shift & -- & Undocumented"
- evidence: facts.md Open question 10. 0x27 LoadRegisterFromShortImmediate is fully implemented (T-ASI:464-500) and 0x2F CoprocessorCallShortImmediate delivers an 8-bit command that can never be privileged (T-PROT:530-547); neither has an assembler form (A-AI:31-47, A-LOAD:73, A-COP:49-140). They are not in the same class as 0x08/0x09/0x0B/0x0D, which have no defined mnemonic at all, and this appendix treats them inconsistently: chapters/appendix-a-opcode-map.tex:62,70 calls them Undocumented, :134-138 omits them from the list of undocumented opcodes, and :169 says they "have no public assembly syntax and are undocumented"
- resolution: unclear
- fix: Author decision needed: are 0x27 and 0x2F (a) undocumented like 0x08/0x09/0x0B/0x0D, (b) defined encodings with no assembler syntax, or (c) reserved? Whichever is chosen, make chapters/12-instruction-summary.tex:66,74, appendix-a-opcode-map.tex:62,70,134-138,159,163,169 and Appendix C agree, and drop the mnemonics LOAD/COPI from those rows if the encodings are declared undocumented.
- confidence: high

### F-sum-15
- file: chapters/12-instruction-summary.tex
- lines: 32, 83
- severity: major
- category: fact
- claim: "0x07 & LOAD & Immediate & -- & Load immediate (move)" and "0x37 & LOAD & Register & -- & Load from register (move)"
- evidence: facts.md Open question 9. The hardware LOAD ALU operation with AF=Alu sets N and Z from the loaded value and clears C and V (alu.rs:307-317; T-AR:497-534 shows 0xFACE setting N); the assembler always emits AF=None (A-LOAD:47,128), and the immediate LOAD test uses AF=Shift and expects all flags cleared (T-AI:29-33, 521-573)
- resolution: unclear
- fix: State which of the three behaviours is architectural. If "the LOAD opcodes update no flags" is a property of the assembler rather than of the encoding, the Flags cell should read "-- (AF=N as emitted by the assembler; see Chapter 13)". Same cell at chapters/appendix-e-quick-reference.tex:58.
- confidence: medium

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

### F-sum-30
- file: chapters/appendix-d-examples.tex
- lines: 8-11
- severity: minor
- category: fact
- claim: the included listing begins "; Reserved space for 128x32 bit exception vectors" followed by ".ORG 0x0200" for the first code
- evidence: examples/byte-sieve/byte-sieve.sasm:5-9, repeated at examples/faults/faults.sasm:19, examples/hardware-exception/hardware-exception.sasm:1, examples/software-exception/software-exception.sasm:1 and examples/store-load/store-load.sasm:1. The vector number is 8 bits and the vector address is vector*2 (EEX:304-325), so the table spans word addresses 0x000-0x1FF, which is 256 vectors of 32 bits, exactly what the .ORG 0x0200 in the listings reserves. facts.md Open question 3 records that EDEF:6-9 says 128 vectors while the encoding allows 256
- resolution: unclear
- fix: Settle the vector count once (facts.md Open question 3) and correct the comment in all five example sources to match; as written the listings state 128 while reserving room for 256.
- confidence: medium

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

