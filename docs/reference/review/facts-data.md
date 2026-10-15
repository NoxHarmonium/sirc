# Findings: chapter group G2 (data, status, condition codes)

Reviewer tag: `data`. Chapters reviewed: `chapters/04-data-representation.tex`,
`chapters/05-status-register.tex`, `chapters/10-condition-codes.tex`.

No facts digest existed for this pass, so every normative claim below was checked directly
against `sirc-vm/` and, where a listing was involved, by actually assembling it with
`sirc-vm/target/debug/assembler`. All 11 assembly listings in Chapters 5 and 10 assemble
cleanly, including `ORRI[N] sr, #0x0003`, `LOAD sr, (s)+`, `STOR|NS (#0, s), r6`,
`LJSR|>> a` and `LOAD|>= r3, r4`. Every hex value in Chapter 4's four tables, the bit
positions in Figure "Status Register Bit Layout", and all 16 rows of Chapter 10's condition
code encoding table were recomputed and match the implementation.

### F-data-1
- file: chapters/04-data-representation.tex
- lines: 162-163
- severity: blocker
- category: fact
- claim: "Displacements are signed two's-complement, and the trap fires only when the low-word calculation wraps, whether the displacement is positive or negative."
- evidence: `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:79-80` computes `let (displaced, displacement_overflowed) = decoded.ad_l_.overflowing_add(decoded.sr_b_);` -- an *unsigned* `u16` add, whose carry-out is the opposite of a signed wrap for a negative displacement. `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:189` sets `sr_b_` to the raw immediate, so `#-4` arrives as `0xFFFC`. Worked case: `al = 0x1000`, `STOR (#-4, s), r2` gives `0x1000.overflowing_add(0xFFFC) = (0x0FFC, true)`, so with `SR.A` set the CPU raises `Faults::SegmentOverflow` (line 128-133) even though `0x1000 - 4` does not wrap. The inverse case also fails: `al = 0x0000` with `#-1` gives `(0xFFFF, false)`, a genuine underflow that does *not* trap. The same file's lines 84-94 already fix exactly this class of bug for the pre-decrement `-1` case ("Pre-decrement's borrow must be computed with a real subtraction"), and `sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:900-980` tests it, but the displacement add itself was never converted. Manual side: `chapters/08-addressing-modes.tex:236-238` states the same rule even more explicitly ("a negative offset that keeps the sum within the low word's range does not fault"), and `chapters/14-memory-instructions.tex:104` agrees. No existing test covers a negative displacement with `SR.A` set, and `docs/reference/manual-handover.md` records no decision here.
- resolution: code-wrong
- fix: No manual change. Per the handover's "Source of Truth" section the manual is canonical and self-consistent across Ch. 4, Ch. 8 and Ch. 14, so this is an implementation gap: `execution_effective_address.rs:79-80` must detect a *signed* wrap of `ad_l_ + sr_b_` (e.g. sign-extend both to `i32`, or branch on `sr_b_`'s sign and use `overflowing_sub` for the negative case, mirroring the existing pre-decrement fix), and a regression test for a negative displacement with `SR.A` set should be added next to `test_segment_overflow_fault_with_predecrement_nonzero_does_not_fault`. If the manual is instead to be changed, the sentence in Ch. 4 plus `chapters/08-addressing-modes.tex:236-238` and `chapters/14-memory-instructions.tex:104` all have to change together.
- confidence: high

### F-data-2
- file: chapters/05-status-register.tex
- lines: 226-228
- severity: major
- category: fact
- claim: "\textbf{Important:} When running in protected mode, any direct write to the status register raises a privilege violation fault. Protected-mode programs can still affect condition flags through ordinary ALU and shift instruction execution; they cannot directly load or store \reg{sr}."
- evidence: `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:42-43` decides the privilege violation purely from the *destination operand field*: `let writing_to_privileged_registers = !is_cop_instruction && PRIVILEGED_REGISTERS.contains(&instruction.des);`, with `PRIVILEGED_REGISTERS` (lines 21-27) containing `Sr`. It does not consider whether the instruction actually writes its destination. So the flag-only instructions `CMPI`, `CMPR`, `TSAI`, `TSAR`, `TSXI`, `TSXR` (`definitions.rs:223-227`, write-back type `AluStatusOnly` at `stages/write_back.rs:143`, which never writes `des`) also fault in protected mode when `sr` is named. The manual already documents one instance of this: `chapters/13-alu-instructions.tex:1530` says "\texttt{TSAI sr, \#0} in protected mode raises a privilege-violation fault." Chapter 5's rule, phrased as "any direct write", does not cover those instructions, and the same understatement appears in the Overview bullet at line 10 ("a direct write to \reg{sr} is privileged").
- resolution: manual-wrong
- fix: Replace the sentence with: "\textbf{Important:} When running in protected mode, any instruction that names \reg{sr} as its destination operand raises a privilege violation fault -- including the flag-only comparison and test instructions (\mnemonic{CMPI}, \mnemonic{CMPR}, \mnemonic{TSAI}, \mnemonic{TSAR}, \mnemonic{TSXI}, \mnemonic{TSXR}), which fault even though they do not write their destination. Protected-mode programs can still affect condition flags through ordinary ALU and shift instruction execution on general-purpose registers; they cannot directly load or store \reg{sr}." Apply the same widening to the Overview bullet at line 10 ("a direct write to \reg{sr} is privileged" -> "naming \reg{sr} as a destination operand is privileged").
- confidence: high

### F-data-3
- file: chapters/05-status-register.tex
- lines: 192-193
- severity: minor
- category: contradiction
- claim: "An explicit status update override written on \mnemonic{LOAD} leaves all four flags architecturally undefined (\texttt{U U U U})."
- evidence: The reference assembler rejects the syntax outright rather than encoding anything: `sirc-vm/toolchain/src/parsers/opcodes/load.rs:29-37` returns a hard failure, "The [LOAD] opcode does not support an explicit status register update source. Only ALU instructions can update the status register as a side-effect." Confirmed by assembling `LOAD[A] r1, r2` (rejected) versus `LOAD r1, r2` (accepted). `chapters/13-alu-instructions.tex:178-180` states this correctly -- "Public \mnemonic{LOAD} forms do not support status update override syntax [...] not reachable from public assembly syntax, leaves all four status flags architecturally undefined" -- so Chapter 5 is the outlier. (`chapters/11-reading-instructions.tex:133` is a third, also-loose wording: "regardless of any override syntax" implies the override is accepted and ignored.)
- resolution: manual-wrong
- fix: Replace with: "A \mnemonic{LOAD} encoded with a non-\texttt{[N]} status update source is not reachable from the reference assembler's syntax, which rejects an override on \mnemonic{LOAD}; such an encoding leaves all four flags architecturally undefined (\texttt{U U U U})." The same loose phrasing appears at `chapters/11-reading-instructions.tex:133` and `chapters/12-instruction-summary.tex:102-103` and should be brought into line with `chapters/13-alu-instructions.tex:178-180`.
- confidence: high

### F-data-4
- file: chapters/05-status-register.tex
- lines: 100-105
- severity: minor
- category: fact
- claim: "\item[When Set (1):] CPU operates in protected mode with restricted access to: \begin{itemize} \item High address-register components (\reg{lh}, \reg{ah}, \reg{sh}, \reg{ph}) \item High byte of status register \item Privileged coprocessor operations (command-word operation nibble \texttt{0x8}--\texttt{0xF}, bits 11--8) \end{itemize}"
- evidence: The implementation has a fourth protected-mode restriction that this list omits: `sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:429-439` raises `Faults::PrivilegeViolation` for a software exception whose `vector < USER_EXCEPTION_VECTOR_START` (`= 0x60`, `exception_unit/definitions.rs:112`) while `ProtectedMode` is set. The authoritative list in the code (`exception_unit/definitions.rs:51-56`) has four items, and `chapters/06-exceptions.tex:159` documents it ("Triggering a software exception with a vector below \texttt{0x60}"). The three items Chapter 5 does list are all correct: `PRIVILEGED_REGISTERS` at `processing_unit/execution.rs:21-27`; SR redaction at `stages/write_back.rs:46-52`; and `cop_opcode = sr_b_ & 0x0F00` with `> 0x0700` at `processing_unit/execution.rs:46-48`, which is exactly nibble `0x8`--`0xF` of bits 11--8.
- resolution: manual-wrong
- fix: Add a fourth bullet: "\item Software exceptions targeting a vector below \texttt{0x60} (see Chapter~\ref{ch:exceptions})".
- confidence: high

### F-data-5
- file: chapters/10-condition-codes.tex
- lines: 9-11
- severity: minor
- category: fact
- claim: "If the condition is not met, the instruction has no architectural effect: no register, flag, memory, address-register or coprocessor state changes, and no privilege check is performed."
- evidence: The four gated behaviours are all confirmed -- `stages/execution_effective_address.rs:35-37`, `stages/memory_access.rs:26`, `stages/write_back.rs:131-133` all return `NoOp` when `!con_`, and `processing_unit/execution.rs:35-37` returns `false` from `check_privilege` when `!con_` (tested by `sirc-vm/peripheral-cpu/tests/instructions/protected_mode_test.rs:155` `conditional_false_privileged_writes_do_not_fault` and `:504` `privileged_coprocessor_calls_fault_only_when_executed`). But \reg{p} *is* a register and it still advances: `processing_unit/execution.rs:165` performs `registers.pl = self.decoded_instruction.npc_l_;` unconditionally at decode, outside any `con_` test. As written, "no register [...] changes" could be read as saying a condition-false instruction does not advance the program counter.
- resolution: manual-wrong
- fix: Amend to: "If the condition is not met, the instruction has no architectural effect beyond the normal two-word advance of \reg{p}: no general-purpose register, flag, memory, address-register or coprocessor state changes, and no privilege check is performed." The identical wording in the Never (NV) entry at lines 111-112 needs the same amendment.
- confidence: high

### F-data-6
- file: chapters/05-status-register.tex
- lines: 296-298
- severity: minor
- category: structure
- claim: "The restore above does not reproduce bit 13 (\texttt{EA}) as it stood at the time of the save: a direct write to \reg{sr} always preserves the register's current \texttt{EA} value rather than adopting the written one, so only \mnemonic{RETE} can restore \texttt{EA}."
- evidence: chapters/05-status-register.tex:222-224 already states exactly this, in the same chapter, about the `LOAD sr, (s)+` restore in the "Manual Updates" listing: "A direct write to \reg{sr} also preserves the current value of bit 13 (\texttt{EA}); only \mnemonic{RETE} restores it, so the \mnemonic{LOAD} \reg{sr}, (s)+ restore above does not reproduce the exception-active state saved at the time of the original write." Both statements are correct against `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:56-71` and `coprocessors/exception_unit/execution.rs:364-370`; the issue is that the rule is now stated twice in full, which is the sort of duplication this session's edit pass was removing.
- resolution: n/a
- fix: Shorten the trailing paragraph to a back-reference, e.g. "As with the stack restore in Section~\ref{sec:sr-manual-updates}, this restore does not reproduce bit 13 (\texttt{EA}); only \mnemonic{RETE} restores it."
- confidence: medium

### F-data-7
- file: chapters/05-status-register.tex
- lines: 281-294
- severity: minor
- category: fact
- claim: "; Save status register\nLOAD r7, sr\n\n; Do some operations that change flags\nADDR r1, r2, r3\nCMPI r1, #100\n\n; Restore original status\nLOAD sr, r7"
- evidence: This listing assembles, and works as shown in supervisor mode, but it is silently supervisor-only and the caption ("Saving and Restoring Status") does not say so. In protected mode `LOAD r7, sr` returns only bits 7--0 with bits 15--8 zeroed (`stages/fetch_and_decode.rs:40-48`, `SR_REDACTION_MASK`), and `LOAD sr, r7` raises a privilege violation (`processing_unit/execution.rs:42-43`; tested by `sirc-vm/peripheral-cpu/tests/instructions/protected_mode_test.rs:98` `direct_status_register_writes_fault_in_protected_mode`, which covers both the immediate and register forms). The immediately preceding example in the same section ("Enabling Protected Mode") is explicitly annotated "(from supervisor mode)", so a reader may reasonably assume this one is not mode-restricted.
- resolution: manual-wrong
- fix: Add a mode annotation to the first comment line, matching the neighbouring example: `; Save status register (supervisor mode only -- see Section~\ref{sec:sr-manual-updates})`.
- confidence: medium

### F-data-8
- file: chapters/05-status-register.tex
- lines: 9-12
- severity: nit
- category: terminology
- claim: "\item \textbf{Low Byte (bits 7--0):} Condition flags -- readable in both modes, and updated as a side effect of ALU and shift instructions in both modes"
- evidence: Only bits 3--0 are condition flags; chapters/05-status-register.tex:85-89 (this chapter's own "Reserved Flags -- Bits 4--7" subsection) says bits 4--7 are reserved. The reserved-bit claim itself is correct against the code: `stages/write_back.rs:171-181` rebuilds the low byte as `(registers.sr & SR_PRIVILEGED_MASK) | (alu_status & SR_REDACTION_MASK)`, and `stages/alu.rs:552-600` `set_alu_bits` only ever touches bits 0--3 of an `alu_status_register` that starts at zero, so bits 4--7 do read back as zero after any flag-updating instruction, exactly as the manual says.
- resolution: manual-wrong
- fix: "\item \textbf{Low Byte (bits 7--0):} Condition flags (bits 3--0) and reserved bits (bits 7--4) -- readable in both modes, and updated as a side effect of ALU and shift instructions in both modes"
- confidence: high
