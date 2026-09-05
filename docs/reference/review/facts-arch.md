# Fact-check findings — chapter group G1 (architecture and registers)

Reviewer tag: `arch`. Chapters reviewed: `chapters/01-introduction.tex`,
`chapters/02-cpu-architecture.tex`, `chapters/03-registers.tex`.
Digest: `docs/reference/review/facts.md`. Conventions: `docs/reference/STYLE.md` (Approved decisions).

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

### F-arch-6
- file: chapters/01-introduction.tex
- lines: 95-100, 110
- severity: major
- category: contradiction
- claim: "All implementations of the SIRC-1 will have the \"processing unit\" as coprocessor 0x0, the \"exception unit\" as coprocessor 0x1, and the DMA unit as coprocessor 0x2." / "0x2         & DMA Unit           & No                & Transfers blocks of data via the bus        \\"
- evidence: manual side: chapters/16-coprocessor-instructions.tex:142 also marks DMA "Required: Yes". Code side: sirc-vm/peripheral-cpu/src/lib.rs:375-394 dispatches only coprocessor IDs 0 and 1 and raises `Faults::InvalidOpCode` for every other ID, including 0x2; facts digest "Coprocessors" table records DMA as "not implemented in peripheral-cpu"
- resolution: unclear
- fix: Either the reference implementation is deliberately an incomplete model (in which case say so once, for example "the reference simulator model omits the DMA unit and raises an invalid-opcode fault for coprocessor 0x2") or DMA is optional like the maths unit. An emulator author cannot tell from the manual whether coprocessor 0x2 absence is conformant. The chapter needs one sentence settling it; the DMA timing figures in Chapter 16 depend on the same decision.
- confidence: medium

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

### F-arch-11
- file: chapters/02-cpu-architecture.tex
- lines: 530-532, 889
- severity: minor
- category: fact
- claim: "Interrupt inputs are level-sensitive and are sampled at instruction boundaries." / "Inputs should remain asserted until an instruction boundary if service is required."
- evidence: manual/handover side: docs/reference/manual-handover.md:472-479 resolves IRQ1--IRQ4 and NMI as level-sensitive, instruction-boundary sampled. Code side: sirc-vm/peripheral-cpu/src/lib.rs:295-298 calls `raise_hardware_interrupt` on *every* cycle (so a pulse asserted mid-instruction is latched and later dispatched), while only the dispatch is gated to phase 0 (lib.rs:326-341)
- resolution: unclear
- fix: The handover fixes the architectural rule, but the reference implementation latches an enabled line on any cycle, so it services pulses the manual says may be missed. State explicitly whether a line asserted and released entirely within an instruction is architecturally guaranteed to be lost (manual as written) or latched (implementation), because an emulator must choose.
- confidence: medium

### F-arch-12
- file: chapters/01-introduction.tex
- lines: 69
- severity: minor
- category: fact
- claim: "Addressing Modes            & 7                      \\"
- evidence: docs/reference/manual-handover.md:17 ("Resolved: the CPU has seven addressing modes; operandless meta-instructions ... are not an addressing mode"); sirc-vm/toolchain/src/parsers/instruction.rs:85-106 (ten distinct syntactic operand forms); facts digest Open question 11
- resolution: unclear
- fix: The count of seven is a ruled decision, but no chapter derives it from the ten operand forms the assembler parses, so the number cannot be checked. Add a cross-reference from this row to the addressing-mode chapter's numbered list, and make that list contain exactly seven entries with the immediate/register displacement variants grouped explicitly.
- confidence: medium

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
