# Fact-check findings — chapter group G3 (exceptions and undocumented behaviour)

Reviewer tag: `exc`. Files reviewed: `docs/reference/chapters/06-exceptions.tex`,
`docs/reference/chapters/appendix-c-undocumented.tex`.

No `docs/reference/review/facts.md` digest existed for this pass, so every normative claim below
was checked directly against `sirc-vm/` (implementation plus `peripheral-cpu/tests/` and
`toolchain/` tests) and against the other manual chapters.

### F-exc-1
- file: chapters/06-exceptions.tex
- lines: 636
- severity: blocker
- category: contradiction
- claim: "\texttt{0x01}                & \addr{0x0002}--\addr{0x0003} & 7                 & Bus fault        & Not maskable         & Next instruction        & Fault metadata    & Yes             \\"
- evidence: chapters/06-exceptions.tex:91-93 and :273-275 say the bus-fault link register holds the *faulting* instruction's address; sirc-vm/peripheral-cpu/src/lib.rs:344-363 computes `fault_pc` as the faulting instruction's own fetch address; sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:298-300 ("Bus Fault is retryable: the link register holds the faulting LOAD's own address.")
- resolution: manual-wrong
- fix: Change the Return Address cell for vector `0x01` from `Next instruction` to `Faulting instruction`. The same error appears twice more in the same table and must be fixed in all three places: line 640 (vector `0x05`, Privilege fault — see tests/exceptions/faults.rs:1458-1461) and line 644 (vector `0x09`, Bus protection — see tests/exceptions/faults.rs:453-456). All six faults listed as Retryable in the "Fault Categories" section (lines 68-69) must read `Faulting instruction` in this table; only vector `0x06` (Trace) is `Next instruction`, and `0x07`/`0x08` are `Dispatch point`. The table predates the retryable-fault classification work (added in commit 4a1526b, 2026-06-24) and was never updated.
- confidence: high

### F-exc-2
- file: chapters/06-exceptions.tex
- lines: 729-732
- severity: blocker
- category: contradiction
- claim: "The reset-vector fetch therefore takes four clock cycles in total, separate from and following the reset-output hold described in Section~\ref{sec:reset-output-hold}."
- evidence: chapters/appendix-b-timing.tex:47 gives `RSET` as "18 (6 call + 6 \texttt{RSTO} hold + 6 reset-vector fetch)" and appendix-b-timing.tex:62-64 states "A hardware-exception or fault dispatch occupies one 6-cycle slot ... entry to the handler is committed at phase 3"; sirc-vm/peripheral-cpu/src/lib.rs:71 (`CYCLES_PER_INSTRUCTION: u8 = 6`) and lib.rs:606 (`advance_phase` is mod 6) make every exception-unit slot six cycles; sirc-vm/peripheral-cpu/tests/exceptions/reset.rs:189-191 polls `CYCLES_PER_INSTRUCTION - 1` further cycles after the vector fetch begins to complete the slot.
- resolution: manual-wrong
- fix: Replace with: "The reset-vector fetch therefore occupies one full six-cycle exception-unit slot, separate from and following the reset-output hold described in Section~\ref{sec:reset-output-hold}. The status-register clear and program-counter load both commit on the fourth cycle of that slot; the two remaining cycles complete the slot before the first instruction fetch." This also keeps the 18-cycle `RSET` total in Appendix~\ref{appendix:timing} arithmetically correct.
- confidence: high

### F-exc-3
- file: chapters/06-exceptions.tex
- lines: 358-359
- severity: blocker
- category: fact
- claim: "\textbf{Bits 8--11:} Original fault type (4 bits) — only meaningful for double faults, indicates the fault type that was being handled when the double fault occurred"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:185-203 sets `fault: resolved_fault` (always `DoubleFault` when escalating) and `original_fault: fault`, where `fault` is the *newly raised* second fault, not the one already being serviced; sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs:134 documents the field as "If double_fault: holds the fault that caused the double fault"; sirc-vm/peripheral-cpu/src/lib.rs:530-540 does the same on the vector-fetch escalation path. Neither side is settled by a test: the only end-to-end double-fault test (sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:1165-1237) uses `InvalidOpCode` for *both* faults, so it cannot distinguish the two readings, and docs/reference/manual-handover.md records no decision on this field.
- resolution: unclear
- fix: Decide which fault the field holds and make both sides agree. Manual side: bits 4--7 hold the fault "that occurred" (which for a double fault is always `0x08`) and bits 8--11 hold the *first* fault. Code side: bits 4--7 are always `0x08` and bits 8--11 hold the *second* (triggering) fault, which makes bits 4--7 carry no information and bits 8--11 the only useful field. If the manual's reading is kept, `raise_fault` must be changed to carry the in-flight fault forward; if the code's is kept, the text should read "indicates the fault type that triggered the double fault (the second fault), not the fault already being handled."
- confidence: high

### F-exc-4
- file: chapters/06-exceptions.tex
- lines: 438-441
- severity: major
- category: fact
- claim: "At an instruction boundary, the exception unit selects the highest-priority pending hardware interrupt whose priority is higher than the current exception level. The selected bit is cleared as the handler is dispatched. Pending lower-priority interrupts remain latched and are considered again after the current handler returns with \mnemonic{RETE}."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:161-204 re-tests the enable mask at *dispatch* time (`let is_enabled = (hw_interrupt_enable & (1 << interrupt_line)) != 0;`) using only the single highest pending line; if that line has since been disabled, the function returns without dispatching, and no lower-priority enabled pending interrupt is considered either. The manual describes masking only at the sampling point (lines 430-432) and says nothing about a re-check at dispatch or about a masked high-priority pending bit blocking lower ones.
- resolution: unclear
- fix: State explicitly what happens to an interrupt that was latched while enabled and then disabled before dispatch. Either (a) document the implementation's behaviour -- "the enable bit is re-tested at dispatch; a pending interrupt on a line that has since been disabled is not dispatched and, because only the highest pending line is considered each boundary, it also blocks lower-priority pending interrupts until its line is re-enabled or the pending bit is cleared" -- or (b) rule that latched interrupts are dispatched regardless of a later mask change and fix the code. An implementer cannot build either behaviour from the current text.
- confidence: medium

### F-exc-5
- file: chapters/06-exceptions.tex
- lines: 139-140
- severity: major
- category: contradiction
- claim: "\textbf{Invalid Opcode Fault} \textit{(Retryable)} — Raised when a coprocessor call targets a non-existent coprocessor or requests an operation that the selected coprocessor does not implement."
- evidence: the same chapter at line 804 says "Exception coprocessor opcodes \texttt{0x2}--\texttt{0x8} are reserved; issuing one is architecturally undefined" -- i.e. an unimplemented operation on an *existing* coprocessor that is explicitly *not* an invalid-opcode fault. Chapter 16 takes the first reading for the DMA unit (chapters/16-coprocessor-instructions.tex:702-703, "Missing DMA hardware, unsupported DMA operations, and reserved operand encodings raise invalid-opcode faults"). The implementation raises `InvalidOpCode` only for an unmapped coprocessor ID (sirc-vm/peripheral-cpu/src/lib.rs:415-440) and panics on an unimplemented exception-unit opcode (sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:69-77, a `TODO: Define what happens when exception CoP gets an unimplemented opcode`).
- resolution: unclear
- fix: Reconcile the two statements. Simplest reading that keeps both chapters consistent: make the fault description cover coprocessor IDs and *implemented-but-unsupported* operations, and change line 804 to "Exception coprocessor opcodes \texttt{0x2}--\texttt{0x8} are reserved; issuing one raises an Invalid Opcode Fault" -- or, if reserved exception-unit opcodes really are undefined rather than trapping, say so in the Invalid Opcode Fault description as a named exception to the general rule.
- confidence: medium

### F-exc-6
- file: chapters/06-exceptions.tex
- lines: 54-56
- severity: minor
- category: contradiction
- claim: "\item[Retryable faults] The operation that triggered the fault does not take effect. For most faults this means the instruction is cancelled before it commits"
- evidence: the DMA-burst caveats added later in the same chapter (lines 100-104, 114-116, 135-137) state the opposite for `DMAR`/`DMAW`/`DMAT`, and chapters/16-coprocessor-instructions.tex:713-738 says the retryable classification "does not hold uniformly" for those three instructions. The same unqualified promise is repeated at lines 763-766 ("\mnemonic{RETE} returns to the faulting instruction, allowing it to be retried") and at lines 273-275.
- resolution: manual-wrong
- fix: Add one sentence to the Retryable-faults bullet after "...the instruction is cancelled before it commits": "The exception is a DMA burst (\mnemonic{DMAR}, \mnemonic{DMAW}, \mnemonic{DMAT}), which performs several bus accesses per instruction; see the individual fault descriptions below and Section~\ref{sec:dma-fault-retry}." Consider the same pointer at lines 763-766, which currently restates the blanket guarantee with no caveat.
- confidence: high

### F-exc-7
- file: chapters/06-exceptions.tex
- lines: 402-404
- severity: minor
- category: contradiction
- claim: "A cause value of \texttt{0x0000} -- the value left in place at reset and whenever no exception is pending -- selects coprocessor ID \texttt{0x0}, the processing unit"
- evidence: Table~\ref{tab:reset-state} in the same chapter (line 686) says "Pending coprocessor command | Set internally to the reset operation until the reset vector sequence completes", and sirc-vm/peripheral-cpu/src/lib.rs:622-624 seeds `pending_coprocessor_command` with `construct_cause_value(&ExceptionUnitOpCodes::Reset, 0x0)` = `0x1B00` on reset, which is only cleared to `0x0` by the exception unit's write-back phase (coprocessors/exception_unit/execution.rs:469).
- resolution: manual-wrong
- fix: Replace "the value left in place at reset and whenever no exception is pending" with "the value left in place once the reset sequence has completed, and whenever no exception is pending".
- confidence: high

### F-exc-8
- file: chapters/06-exceptions.tex
- lines: 528-551
- severity: minor
- category: fact
- claim: "\item The Protected Mode bit in the status register is cleared, switching the CPU to supervisor mode."
- evidence: the numbered "Exception Handling Flow" omits two entry side effects that Section~\ref{sec:exception-entry-side-effects} (lines 569-579) lists as part of the same architectural action and that sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:276-279 performs together: `clear_sr_bit(TraceMode)` and `set_sr_bit(ExceptionActive)`.
- resolution: manual-wrong
- fix: Add two steps to the numbered list after the Protected Mode step: "The TraceMode bit is cleared, so trace faults do not fire inside the handler." and "The Exception Active bit is set." An implementer following only the numbered flow would omit both.
- confidence: high

### F-exc-9
- file: chapters/appendix-c-undocumented.tex
- lines: 71-73
- severity: minor
- category: fact
- claim: "The undocumented opcodes with bit 3 set in the ALU operation field compute \texttt{alu\_code \& 0x7} as the operation, discard the result, and update all four flags exactly as the corresponding documented operation does."
- evidence: as literally written this covers \opcode{2F}, which this appendix's own table (line 59) lists as an undocumented opcode and whose ALU-operation field is `0xF` -- bit 3 set. The implementation explicitly excludes `0xF` from that rule: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:69, `let simulate = alu_code != 0xF && alu_code & 0x8 == 0x8;`, and sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:146-153 routes `0x0F`/`0x2F`/`0x3F` to `CoprocessorCall`, which *does* write back (to `pending_coprocessor_command`). The remainder of the claim is correct for operation fields `0x8`--`0xE`: `perform_alu_operation` runs the masked operation and `set_alu_bits` writes all four flags (alu.rs:517-548, 551-598).
- resolution: manual-wrong
- fix: Replace "with bit 3 set in the ALU operation field" with "whose ALU operation field is \texttt{0x8}--\texttt{0xE}". The identical over-broad phrasing appears in the "Implementation Behavior" subsection at lines 131-133 ("the decoder treats bit 3 of the operation field as a \"don't write back\" bit"), which should gain the same exclusion, e.g. "...as a \"don't write back\" bit for operation fields \texttt{0x8}--\texttt{0xE}; \texttt{0xF} is the coprocessor call and is not affected".
- confidence: high

## Claims checked and found correct (no finding)

Recorded because the task asked specifically about the two edits made this session.

- **DMA-burst caveats (06-exceptions.tex:100-104, 114-116, 135-137).** The cross-reference target
  exists and says what is claimed: `\label{sec:dma-fault-retry}` is at
  chapters/16-coprocessor-instructions.tex:711 ("Retrying After a Fault"), and that section states
  the same non-atomicity for `DMAR`/`DMAW`/`DMAT` (lines 713-738). The BAT codes cited for
  recognising the case (`0x5` DMA read burst, `0x6` DMA write burst) match
  sirc-vm/peripheral-bus/src/device.rs:26-27 and the fault-metadata bit layout at
  sirc-vm/peripheral-cpu/src/lib.rs:85-92. Note that these claims cannot be checked against the
  emulator: no DMA coprocessor exists in `sirc-vm/peripheral-cpu` (only IDs 0 and 1 are
  implemented -- processing_unit/execution.rs:62 and exception_unit/execution.rs:292), which is
  consistent with the DMA unit being an optional coprocessor per
  16-coprocessor-instructions.tex:655.
- **Appendix C "Implementation Behavior" (lines 129-136).** All three technical claims survive the
  merge, subject to F-exc-9's `0xF` exclusion. Decode succeeds normally: the undocumented opcodes
  are real enum variants (`_Undocumented0x08` etc.,
  processing_unit/definitions.rs:221-280), so `FromPrimitive` decoding does not reject them. The
  ALU executes and sets flags as the documented operation would
  (execution_effective_address.rs:65 masks with `& 0x7`; alu.rs:525-540). Only write-back is
  suppressed: `decode_write_back_step_instruction_type` maps `0x08..=0x0E`, `0x28..=0x2E` and
  `0x38..=0x3E` to `AluStatusOnly`, which calls `update_status_flags` and no `set_register_value`
  (write_back.rs:140-160, 205-212).
- **Appendix C overview claims for \opcode{27} and \opcode{2F}.** `0x27` zero-extends the 8-bit
  immediate into `rD`: tests/instructions/arithmetic_short_immediate_test.rs:464-484 loads `0xFA`
  and expects `0xFA` with Negative cleared. `0x2F`'s coprocessor ID is always 0 and it is never
  privileged: tests/instructions/protected_mode_test.rs:549-566
  (`short_copi_cannot_encode_privileged_coprocessor_command`) asserts no fault and
  `pending_coprocessor_command == 0x0019` in protected mode. Neither has assembler syntax: no
  toolchain parser emits `LoadRegisterFromShortImmediate` or `CoprocessorCallShortImmediate`.
- **Appendix C aliased-write listings (lines 28-37).** All three forms are rejected by the
  assembler as claimed: `LDEA a, -(#0, a)` by toolchain/src/parsers/opcodes/ldea.rs:133-136,
  `LDEL p, (#0, l)+` and `LDEL l, (#0, a)` by ldel.rs:50-56 and ldel.rs:137-142
  (`reject_aliased_address_register_write`, opcodes/mod.rs:36-48).
- **Fault vector numbers, fault-type codes, BAT codes, fault-metadata bit layout, exception-unit
  opcodes, link-register assignment, SR enable-bit positions, vector-table arithmetic, reset-state
  table, RSTO hold length, and the L5-inside-fault timeline table** all match the implementation
  and its tests; each was checked individually.
