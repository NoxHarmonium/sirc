# Fact-check findings — chapter group G3 (exceptions, undocumented behaviour)

Reviewer tag: `exc`. Digest: `docs/reference/review/facts.md`. Chapters reviewed:
`chapters/06-exceptions.tex`, `chapters/appendix-c-undocumented.tex`.

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

### F-exc-3
- file: chapters/06-exceptions.tex
- lines: 33-38, 544, 545
- severity: blocker
- category: fact
- claim: "[Retryable faults] The instruction that triggered the fault is cancelled before it takes effect. The link register stores the address of the faulting instruction." ... "\textit{Bus Fault, Bus Protection Fault, Alignment Fault, Segment Overflow Fault, Invalid Opcode Fault, Privilege Violation Fault.}"
- evidence: sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:939-942 asserts `link_registers[6].return_address == 0x0000_0002` for an invalid-opcode fault raised by the COP instruction at 0x0000_0000 (and :989-992 asserts 0x00AB_CDE4 for the second one); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:141-151 advances `registers.pl = npc_l_` unconditionally, after the decode-time privilege check has already raised the fault. Against this: definitions.rs:120-125 and :19-20 state that abort exceptions save the faulting instruction's address and that "pl is not incremented for a bus fault". facts.md Open question 1.
- resolution: unclear
- fix: The manual and the implementation disagree. Either (a) the manual is wrong and the Retryable category must be reduced to Alignment Fault and PC-wrap Segment Overflow only, with Bus, Bus Protection, Invalid Opcode and Privilege Violation moved to a category whose return address is the *next* instruction; or (b) the implementation is wrong and must suppress the `pl` advance when a fault is raised in the same instruction. Author must rule. Until then do not assert "so it can be retried" for the four faults with no test coverage (Bus, Bus Protection, Privilege Violation, Segment Overflow) and correct the Invalid Opcode Fault row of Table~\ref{tab:exception-quick-reference} (line 544) and the Privilege Violation row (line 545) to match whichever side wins. Also note that Alignment, the one fault that genuinely does preserve the faulting address, preserves an *odd* `pl`, so a bare RETE re-faults forever; the manual should say so.
- confidence: high

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

### F-exc-7
- file: chapters/06-exceptions.tex
- lines: 93-99
- severity: major
- category: fact
- claim: "\item Triggering a software exception with a vector below 0x60"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:330-333 — the exception unit `assert!`s (panics) when a SoftwareException opcode carries a vector below `USER_EXCEPTION_VECTOR_START`, it does not raise PrivilegeViolation; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:46-48 makes the privilege decision purely on the COP opcode nibble (EXCP is opcode 0x1, so never privileged); sirc-vm/toolchain/src/parsers/opcodes/exception.rs:73 masks the vector to 8 bits with no range check. Against this: `exception_unit/definitions.rs:51-55` lists it as a privilege violation. No test covers it. facts.md Open question 4.
- resolution: unclear
- fix: The manual asserts a fault the implementation cannot raise. Either the implementation must add the check (fault instead of panic), or item 5 of this list must be deleted and Section "User Exceptions (Traps)" must state what an `EXCP` below 0x60 does (currently: architecturally undefined). Author must rule; do not leave the claim as written, since an emulator author following the manual would raise vector 0x05 where the reference implementation halts.
- confidence: high

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

### F-exc-9
- file: chapters/06-exceptions.tex
- lines: 170-179
- severity: major
- category: fact
- claim: "\bitbox{16}{Return Status Register} & \bitbox{8}{Saved Level} & \bitbox{8}{Reserved}"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:371-415 — `ETFR`/`ETTR` `register_select` covers only 1 (`return_address` <-> `a`), 2 (`return_status_register` <-> `r7`) and 3 (both); there is no encoding that reads or writes `saved_exception_level`. The struct at registers.rs:581-588 is three separate fields with no packed bit layout anywhere in the code.
- resolution: unclear
- fix: The figure invents a 64-bit packed layout (Saved Level at bits 15--8 of the second word, Reserved at 7--0) that no code defines and that software cannot observe. Either state that the layout is illustrative only and add a sentence "The saved exception level is not accessible to software; \mnemonic{ETFR} and \mnemonic{ETTR} transfer only the return address and the return status register", or drop the packed figure. Author must confirm whether the packing is architectural.
- confidence: medium

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

### F-exc-16
- file: chapters/06-exceptions.tex
- lines: 682-688
- severity: major
- category: fact
- claim: "The following opcodes are used internally by the exception unit and are not directly accessible via user instructions:"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:416-427 — `Fault`, `HardwareException` and `SoftwareException` share one dispatch arm, reached from any cause value, including a `pending_coprocessor_command` written by a privileged `COPI #0x1Exx` / `COPI #0x1Fxx`; the privilege check (processing_unit/execution.rs:46-48) only rejects opcode nibbles > 7 in protected mode, so a supervisor-mode COPI reaches the EU unmodified. sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:1321-1323 records this as unresolved. facts.md Open question 14.
- resolution: unclear
- fix: The claim "not directly accessible via user instructions" is true for protected mode but false for supervisor mode. Either state the restriction precisely ("cannot be issued from protected mode; issuing them from supervisor mode via \mnemonic{COPI} is architecturally undefined") or have the implementation reject them. Author must rule.
- confidence: medium

### F-exc-17
- file: chapters/06-exceptions.tex
- lines: 660-676
- severity: major
- category: fact
- claim: "\mnemonic{EXCP}      & 0x1             & User               & Trigger a software exception (trap)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs:148-162 defines only opcodes 0x0, 0x1, 0x9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF; sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:69-77 panics on any other value ("Unimplemented op code ... for exception co-processor"), with a TODO saying hardware behaviour is undecided. facts.md Open question 6.
- resolution: unclear
- fix: The table plus the "Internal Instructions" list covers 9 of the 16 exception-unit opcodes and never says what 0x2--0x8 are. Add one line after the table: "Exception coprocessor opcodes 0x2--0x8 are reserved. Issuing one is architecturally undefined." Confirm with the author that "reserved/undefined" (not "no-op" and not "invalid opcode fault") is the intended ruling, since the implementation currently halts.
- confidence: medium

### F-exc-18
- file: chapters/06-exceptions.tex
- lines: 626-641
- severity: major
- category: fact
- claim: "\item Reads the link register corresponding to the current interrupt mask level (minus 1, since the mask is set to the exception level)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:350-356 — `eu_registers.link_registers[(current_exception_level - 1) as usize]` with a TODO: "Overflow error when `current_exception_level` is zero. What should actually happen on hardware if you try to RTE when you're not in an exception". facts.md Open question 5.
- resolution: unclear
- fix: The manual never says what \mnemonic{RETE} does outside a handler. Add a normative sentence, for example "Executing \mnemonic{RETE} while the current exception level is 0 is architecturally undefined", and confirm with the author. Separately, "current interrupt mask level" is not a register this manual defines anywhere — the code field is `current_exception_level` (registers.rs:590-603) and the chapter uses that name at line 377; make the two agree.
- confidence: medium

### F-exc-19
- file: chapters/appendix-c-undocumented.tex
- lines: 6, 40-63, 211
- severity: major
- category: contradiction
- claim: "The SIRCIS instruction set includes 14 undocumented instruction opcodes."
- evidence: chapters/appendix-a-opcode-map.tex:130-138 lists exactly 12 undocumented opcodes (0x08, 0x09, 0x0B, 0x0D; 0x28, 0x29, 0x2B, 0x2D; 0x38, 0x39, 0x3B, 0x3D), excluding 0x27 and 0x2F. Only those 12 are named `_Undocumented*` in sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:218-277; 0x27 is `LoadRegisterFromShortImmediate` and 0x2F is `CoprocessorCallShortImmediate`, both with defined, tested behaviour (T-ASI:464-500; T-PROT:530-547). facts.md Open question 10.
- resolution: unclear
- fix: The two appendices give different counts because they disagree on whether an opcode with defined behaviour but no assembler syntax is "undocumented". Author must rule. If 0x27/0x2F stay in this table, the Overview's blanket "their architectural behaviour is undefined" must be qualified for them, since their behaviour is fully specified (0x27: rD = zero-extended 8-bit immediate; 0x2F: coprocessor command = 8-bit immediate, coprocessor ID always 0, therefore never privileged). If they leave, change 14 to 12 here and at line 211 and delete their two rows.
- confidence: high

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

### F-exc-28
- file: chapters/06-exceptions.tex
- lines: 116-127
- severity: major
- category: fact
- claim: "\textbf{Double Fault} \textit{(Exception dispatch)} — Raised when a fault occurs while another fault is already being handled."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:147-157 — `raise_fault` panics ("Cannot raise fault when one is pending") when a second fault condition is detected while `pending_fault` is still set, i.e. in the window between raising a fault and dispatching it, with a TODO asking what hardware should do. The documented double-fault path (lib.rs:164-171) only covers a fault raised once `current_exception_level >= 7`. facts.md Open question 7.
- resolution: unclear
- fix: The manual describes only the level-7 case. It does not say what happens when two fault conditions are detected in the same instruction before either is dispatched (for example an alignment fault and a bus error on the same fetch), and the implementation halts there. Add a normative sentence covering the pre-dispatch case, or confirm with the author that it is architecturally undefined. Note that the priority order BERR > BPER > BACK is defined (exception_unit/definitions.rs:93) but the fault-vs-fault order is not.
- confidence: medium
