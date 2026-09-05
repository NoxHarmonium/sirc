# Fact-check findings — chapter group G7 (coprocessor and meta)

Reviewer tag: `cop`.
Chapters reviewed: `docs/reference/chapters/16-coprocessor-instructions.tex`,
`docs/reference/chapters/17-meta-instructions.tex`.
Digest: `docs/reference/review/facts.md`.

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

### F-cop-3
- file: chapters/17-meta-instructions.tex
- lines: 67
- severity: major
- category: contradiction
- claim: "\textbf{Privilege:} Available in protected mode and supervisor mode."
- evidence: manual side — this line and line 58 ("Inherits \mnemonic{ADDI[N]} exception behavior"); code side — sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:21-27 and 44-51 (`PRIVILEGED_REGISTERS` contains `RegisterName::Sr`; `writing_to_privileged_registers` fires for a non-COP instruction whose destination is `sr`), with `NOOP`'s destination being register 0 = `sr` (toolchain meta.rs:88). No test covers `NOOP` in protected mode. Digest Open question 2 (facts.md:334).
- resolution: unclear
- fix: The architecture must rule on this. Either (a) `NOOP` is unprivileged, in which case the assembler must stop emitting register field 0 (or the decoder must exempt an all-zero destination with AF = None), or (b) the manual must state that `NOOP`, and therefore any all-zero instruction word, raises a PrivilegeViolation fault in protected mode and is a supervisor-only encoding. Until ruled, this line and line 58 must not claim protected-mode availability.
- confidence: high

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

### F-cop-6
- file: chapters/16-coprocessor-instructions.tex
- lines: 140
- severity: major
- category: fact
- claim: "0x1         & Exception Unit       & Yes               & 0x0--0xD            & Handles reset, faults, interrupts, traps, and links"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs:147-162 (`ExceptionUnitOpCodes` defines only 0x0 None, 0x1 SoftwareException, 0x9 WaitForException, 0xA ReturnFromException, 0xB Reset, 0xC TransferFromRegister, 0xD TransferToRegister, 0xE Fault, 0xF HardwareException); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:66-77 (opcodes 0x2--0x8 fall through `FromPrimitive` and `panic!("Unimplemented op code ... for exception co-processor")`, with a `TODO: Define what happens when exception CoP gets an unimplemented opcode`). Digest Open question 6 (facts.md:338).
- resolution: unclear
- fix: The contiguous range "0x0--0xD" is not the implemented set. It must become either an explicit list ("0x0--0x1, 0x9--0xD; 0xE--0xF internal") or the architecture must define opcodes 0x2--0x8 as reserved-and-invalid-opcode-faulting, which the implementation does not currently do (it panics). Line 155--156 of this chapter makes the stronger claim that reserved operation nibbles "must raise invalid-opcode faults"; both statements need the same ruling.
- confidence: high

### F-cop-7
- file: chapters/16-coprocessor-instructions.tex
- lines: 155-156
- severity: major
- category: contradiction
- claim: "Reserved coprocessor IDs, reserved operation nibbles, and reserved operand values must raise invalid-opcode faults when they are not implemented by the selected CPU model."
- evidence: manual side — this line, and lines 12--14 and 65--67 making the same promise; code side — sirc-vm/peripheral-cpu/src/lib.rs:375-394 raises `Faults::InvalidOpCode` only when the *coprocessor ID* is neither 0 nor 1; unimplemented *operation nibbles* on coprocessor 1 panic (exception_unit/execution.rs:69-77) and out-of-range ETFR/ETTR *operand values* panic (exception_unit/execution.rs:227). Digest facts.md:315, Open question 6.
- resolution: unclear
- fix: Either narrow the claim to "Reserved coprocessor IDs raise invalid-opcode faults" and mark reserved operation nibbles and operand values as architecturally undefined, or record this as a required implementation change. Do not leave a "must" that the reference implementation does not satisfy.
- confidence: high

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

### F-cop-9
- file: chapters/16-coprocessor-instructions.tex
- lines: 201-202
- severity: major
- category: fact
- claim: "Triggers a software exception at the specified user vector, normally 0x60--0xFF."
- evidence: manual side — "normally", which implies vectors below 0x60 are legal but unusual; code side — sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:330-333 asserts (panics) when a SoftwareException vector is below `USER_EXCEPTION_VECTOR_START` (0x60); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs:51-56 instead lists "Triggering a software exception below 0x60" as a cause of PRIVILEGE_VIOLATION_FAULT; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:46-48 only inspects the opcode nibble, so an `EXCP` can never be privileged. No test. Digest Open question 4 (facts.md:336).
- resolution: unclear
- fix: The architecture must choose one of: (a) `EXCP #v` with `v < 0x60` raises PrivilegeViolation in protected mode (matching the EDEF comment), (b) it raises InvalidOpCode, or (c) it is architecturally undefined. Then replace "normally 0x60--0xFF" with the ruled, unhedged statement. The assembler masks the vector to 8 bits and does not reject low vectors (toolchain/src/parsers/opcodes/exception.rs:73).
- confidence: high

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

### F-cop-13
- file: chapters/16-coprocessor-instructions.tex
- lines: 376-382
- severity: major
- category: fact
- claim: "The \mnemonic{COPI} dispatch takes the normal 6 cycles, then the DMA unit gets at least one post-dispatch cycle. ... Therefore \mnemonic{DMAR} and \mnemonic{DMAW} take $6 + \max(|n|, 1)$ cycles, and \mnemonic{DMAT} takes $6 + \max(2n, 1)$ cycles."
- evidence: manual side — this passage and the derived per-entry timings at lines 429--431, 477--479 and 523--527; code side — no DMA coprocessor exists in `sirc-vm/peripheral-cpu` (sirc-vm/peripheral-cpu/src/lib.rs:375-394 faults on coprocessor ID 2), and the only implemented coprocessor dispatch (the exception unit, exception_unit/execution.rs:302-446) always consumes a full 6-cycle slot rather than "at least one post-dispatch cycle". Digest facts.md:329 states DMA burst cycle counts are not derivable from code.
- resolution: unclear
- fix: Either state that coprocessor 2 dispatch, unlike coprocessor 1, does not consume a whole 6-cycle slot (and say why), or restate the formulas on the 6-cycle-slot model (for example $6 + 6 + \ldots$). As written the chapter uses two incompatible dispatch-cost models for the same COP mechanism, and an emulator author cannot tell which applies.
- confidence: medium

### F-cop-14
- file: chapters/16-coprocessor-instructions.tex
- lines: 141
- severity: major
- category: contradiction
- claim: "0x2         & DMA Unit             & Yes               & 0x8--0xA            & Required standard DMA transfer unit"
- evidence: manual side — this row, plus line 340 "The required DMA unit" and line 128 "A CPU model may omit an optional coprocessor"; code side — sirc-vm/peripheral-cpu/src/lib.rs:375-394 dispatches only coprocessor IDs 0 and 1 and raises InvalidOpCode for every other ID, so the reference implementation is a conforming SIRC-1 that omits a "required" coprocessor; digest facts.md:313
- resolution: unclear
- fix: Either mark the DMA unit "No" (optional, present on models that document it) to match the reference implementation, or record that the reference emulator is non-conformant. A "Required" coprocessor that the reference CPU faults on makes the probe-by-invalid-opcode-fault advice at lines 157--158 meaningless for DMA.
- confidence: medium

### F-cop-15
- file: chapters/16-coprocessor-instructions.tex
- lines: 61-67
- severity: major
- category: fact
- claim: "If trace mode was enabled when the instruction began, an instruction-trace fault is raised after the processing-unit call commits, unless an earlier fault prevents the instruction from completing."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:397-406 (the trace fault is raised at `WriteBackExecutor` when `pending_fault.is_none()`); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:146-160 (`get_cause_register_value` returns the *fault* cause in preference to `registers.pending_coprocessor_command`); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:444 (`registers.pending_coprocessor_command = 0x0` unconditionally at the dispatch's phase 5). The trace-fault dispatch therefore clears the just-latched coprocessor command and the coprocessor operation never runs. No test covers a coprocessor call in trace mode.
- resolution: unclear
- fix: The manual must state what happens to a pending coprocessor command when a fault or hardware exception is dispatched at the same phase 0. Add to the Common Semantics table: "A pending coprocessor command is discarded if a fault or hardware exception is dispatched before the coprocessor's dispatch slot begins; the command is not re-issued after the handler returns." If the intended architecture is that the command survives, this is a code defect.
- confidence: medium

### F-cop-16
- file: chapters/16-coprocessor-instructions.tex
- lines: 36
- severity: major
- category: fact
- claim: "Opcode 0x2F has no public assembly syntax and is undocumented."
- evidence: manual side — this line plus line 70 ("Opcode 0x2F is undocumented") and STYLE.md:25, which reserves "undocumented" for "a specific, currently-unassigned opcode with observed behavior"; code side — 0x2F is an assigned opcode `CoprocessorCallShortImmediate` with fully defined behaviour (sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs, `CoprocessorCallShortImmediate`), decoded like any short-immediate instruction and latching the zero-extended 8-bit immediate as the command (sirc-vm/peripheral-cpu/tests/instructions/protected_mode_test.rs:530-547 asserts `pending_coprocessor_command == 0x0019` for an encoded value of 0x19). Digest Open question 10 (facts.md:342).
- resolution: unclear
- fix: Rule whether 0x2F is "undocumented" (App. C style, unassigned) or an assigned-but-unassembled encoding. If the latter, replace with: "Opcode 0x2F (short-immediate coprocessor call) has no assembly syntax. It latches the zero-extended 8-bit immediate as the command operand; the shift fields do not apply to the immediate, so the coprocessor ID is always 0x0 and the operation nibble is always 0x0. It can therefore never address a coprocessor and can never be privileged." Note that as written, "undocumented" here conflicts with the same word used for opcodes 0x08/0x09/0x0B/0x0D at line 159.
- confidence: high

### F-cop-17
- file: chapters/16-coprocessor-instructions.tex
- lines: 276-277, 287
- severity: major
- category: fact
- claim: "Performs a software reset of the processor by clearing the status register and jumping to the reset vector." / "\textbf{Timing:} Same as \mnemonic{COPI}: 6 cycles before reset processing begins."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:422-431 (`reset_requested` asserted after write-back of the COPI); sirc-vm/peripheral-bus/src/reset_unit.rs:22-26 (RSTO held for the assertion cycle plus a 5-cycle countdown); sirc-vm/peripheral-cpu/tests/exceptions/reset.rs:84-147; sirc-vm/peripheral-cpu/src/lib.rs:524-536 (`reset()` clears CPU control state and seeds the Reset cause but leaves `r1`--`r7`, the `a`/`l`/`s` pairs and all link registers untouched); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:367-369 (`registers.sr = 0x0; registers.set_full_pc_address(self.vector_value);`). Digest Open question 12 (facts.md:344).
- resolution: unclear
- fix: The Description must say what \mnemonic{RSET} leaves alone: "Requests a full processor reset. The reset output is asserted and held for 6 cycles, after which the exception unit fetches the reset vector, sets \reg{sr} to \texttt{0x0000} and loads the program counter from the vector. General-purpose registers, the \reg{a}, \reg{l} and \reg{s} pairs, and the exception link registers are not affected by reset and hold undefined values after power-on." The Timing line must add the 6-cycle RSTO hold before the vector fetch (total 6 + 6 + 6 cycles from the \mnemonic{RSET} fetch to the first instruction of the reset handler). Whether general registers are "unchanged" or "undefined" after power-on is a decision the architecture has not yet recorded.
- confidence: high

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

### F-cop-26
- file: chapters/16-coprocessor-instructions.tex
- lines: 106-107
- severity: major
- category: fact
- claim: "\textbf{Privilege:} Available in protected mode and supervisor mode. In protected mode, coprocessor-operation nibbles 0x8--0xF are supervisor-only and raise privilege-violation faults when executed."
- evidence: manual side — this line covers nibbles 0xE and 0xF without exception; code side — sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs:158-161 marks 0xE `Fault` and 0xF `HardwareException` as the units' internal cause encodings, and sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:416-427 dispatches them through `handle_exception` exactly as if the hardware had raised them, so a supervisor-mode `COPI #0x1Fxx` fabricates a hardware exception at an arbitrary level and `COPI #0x1Exx` fabricates a fault. sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:1321-1323 marks this unresolved. Digest Open question 14 (facts.md:346).
- resolution: unclear
- fix: The architecture must state whether coprocessor-1 operations 0xE and 0xF are software-callable. If they are internal only, the manual must say so and the exception unit must reject them from a software-issued command; if they are callable, their operand semantics (vector and level) must be documented. Either way, the "Operations 0x0--0xD" row at line 140 and this privilege statement must agree.
- confidence: high
