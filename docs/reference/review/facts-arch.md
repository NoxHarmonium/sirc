# Findings: chapter group G1 (preface, introduction, CPU architecture, registers)

Reviewer tag: arch.

### F-arch-1
- file: reference/chapters/02-cpu-architecture.tex
- lines: 118-123
- severity: major
- category: fact
- claim: "The exception coprocessor never overwrites a pending command. It keeps its own record of the exception it wants to service, and loads the CSR with a hardware interrupt, software exception or trace fault only when the CSR is empty and no coprocessor is active."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:140-218 (selection order: pending fault, then enabled hardware line, and only then `pending_coprocessor_command`); execution.rs:476-477 (EU write-back sets `pending_coprocessor_command = 0x0` after every dispatch, with a TODO describing this exact loss); sirc-vm/peripheral-cpu/src/lib.rs:448-459 (a trace fault is raised at the COP call's own write-back, so it is selected ahead of the command the call just wrote). facts.md Open question 8.
- resolution: code-wrong
- fix: Keep the manual text. Simulator todo: at phase 0, when `pending_coprocessor_command` is non-zero, dispatch it before any hardware interrupt or trace fault (traced `EXCP` excepted, per Section sec:interrupt-latency). The EU write-back must clear the CSR only when the slot it just ran was that command. Add tests for a traced `RETE`/`ETFR` call and for an IRQ arriving during a `COPI` to an absent coprocessor.
- confidence: high

### F-arch-2
- file: reference/chapters/02-cpu-architecture.tex
- lines: 220-223
- severity: major
- category: fact
- claim: "The alignment fault is the exception: it is detected at phase 0, before the instruction starts, and its dispatch begins at that same phase 0"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:86-112 (the alignment `raise_fault` does not return early, so phase 0 still drives the odd address onto the bus as an instruction fetch); sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:722-758 (`expect_dummy_instruction(0x1, true)` takes a full 6-phase slot with address 0x1 on the bus, and only then `expect_fault`). The manual agrees with itself here: docs/reference/chapters/06-exceptions.tex:217-219 says "the odd address never reaches the bus ... the fault's dispatch slot begins at that same phase 0", and appendix-b-timing.tex:84-86 says "its dispatch slot takes the place of the instruction, so it costs no extra cycles".
- resolution: code-wrong
- fix: Keep the manual text. Simulator todo: check `pl` before the phase-0 strobe. On an odd `pl`, emit no bus cycle and start the exception-unit dispatch in the same phase 0 (recompute the cause register after the check). Update `test_alignment_fault` so it no longer expects the dummy slot.
- confidence: high

### F-arch-3
- file: reference/chapters/03-registers.tex
- lines: 129-134
- severity: major
- category: fact
- claim: "Such an instruction does not use the advanced address, so it is exempt from the program-counter wrap check in the same way as a taken branch (Section~\ref{sec:segment-overflow-fault})."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:204 (`npc_overflowed` comes only from `pl + 2`); execution.rs:134-148 (the flag is latched at decode for every instruction whenever SR bit 14 is set) and execution.rs:99-108 (it is raised at the next fetch whatever was written to `p`). Taken branches, `pl` writes and false-condition skips are all treated the same. The fault also lands at the next fetch, with the new PC as its address, rather than in phase 3 with the instruction's own address as 06-exceptions.tex:246-255 requires. The manual is consistent with itself (06-exceptions.tex:246-257; 15-control-flow.tex:109-113; 13-alu-instructions.tex:150).
- resolution: code-wrong
- fix: Keep the manual text. Simulator todo: raise the PC-wrap SegmentOverflow in phase 3 and only for instructions that use the advanced address (non-branching instructions, false conditions, and the link value of LDEL/BRSR/LJSR). Cancel the instruction and save its own address. Add tests: a taken BRAN in slot 0xFFFE must not fault, and `MOVI pl, #x` in slot 0xFFFE must not fault.
- confidence: high

### F-arch-4
- file: reference/chapters/01-introduction.tex
- lines: 179-196
- severity: major
- category: contradiction
- claim: "Hardware        & \texttt{0x10}, \texttt{0x20}, \texttt{0x30}, \texttt{0x40}, \texttt{0x50} & \addr{0x000020}--\addr{0x0000A1}" ... "Beyond these reserved regions, the memory layout is flexible and can be defined by system software."
- evidence: docs/reference/chapters/06-exceptions.tex:66-67 and :87 ("Vectors 0x00--0x5F (96 entries) are reserved for the CPU"; "0x11--0x5F (excluding 0x20, 0x30, 0x40, 0x50): Reserved"). Vectors 0x51-0x5F occupy word addresses 0x0000A2-0x0000BF. No row of the intro table covers them, and the sentence after the table hands them to system software. The Hardware row also silently includes the reserved vectors 0x11-0x4F.
- resolution: manual-wrong
- fix: Change the Hardware row to "\texttt{0x10}--\texttt{0x5F} & \addr{0x000020}--\addr{0x0000BF} & Hardware exception vectors at \texttt{0x10}, \texttt{0x20}, \texttt{0x30}, \texttt{0x40}, \texttt{0x50}; all other entries reserved", so the four rows tile 0x000000-0x0001FF with no gap.
- confidence: high

### F-arch-5
- file: reference/chapters/01-introduction.tex
- lines: 132-133
- severity: major
- category: contradiction
- claim: "Each coprocessor can execute 16 opcodes: the first 8 (\texttt{0x0}--\texttt{0x7}) can be called in any mode, and the last 8 (\texttt{0x8}--\texttt{0xF}) can be called only in supervisor mode."
- evidence: docs/reference/chapters/16-coprocessor-instructions.tex:311,324-325,340-343 and 06-exceptions.tex:286 (exception-unit operation 0x1, `EXCP`, raises a privilege-violation fault in protected mode when the vector is below 0x60); sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:438-448.
- resolution: manual-wrong
- fix: Append: "A coprocessor may add further protected-mode restrictions on individual operations; for example, the exception unit's \mnemonic{EXCP} (operation 0x1) faults in protected mode when its vector is below \texttt{0x60}."
- confidence: high

### F-arch-6
- file: reference/chapters/01-introduction.tex
- lines: 82-85
- severity: major
- category: contradiction
- claim: "Every coprocessor call (\mnemonic{COPI}, \mnemonic{COPR}, and every meta-instruction that assembles to one) takes a second complete 6-cycle dispatch slot, for 12 cycles in total, before the next instruction is fetched."
- evidence: docs/reference/chapters/appendix-b-timing.tex:59-60 (a call whose condition is false "does not dispatch ... and takes only 6 cycles"); 10-condition-codes.tex:8-11 (a false condition causes no coprocessor state change); 02-cpu-architecture.tex:226-227 says only "usually takes a further six".
- resolution: manual-wrong
- fix: "Every executed coprocessor call (\mnemonic{COPI}, \mnemonic{COPR}, and every meta-instruction that assembles to one) normally takes a second complete 6-cycle dispatch slot, for 12 cycles in total; a call whose condition is false takes only the 6 cycles of the call itself."
- confidence: high

### F-arch-7
- file: reference/chapters/01-introduction.tex
- lines: 135-139
- severity: major
- category: fact
- claim: "Using \mnemonic{COPI} or \mnemonic{COPR} with coprocessor 0x0 is technically valid, but any operand passed into \mnemonic{COPI} or \mnemonic{COPR} will be ignored. It is effectively a no-op"
- evidence: The set does not say whether an ID-0 call costs 6 or 12 cycles. Intro :82-85 says every call takes a further 6-cycle slot. 16-coprocessor-instructions.tex:55 gives 12 cycles only for "coprocessor 1". 16-coprocessor-instructions.tex:211-213 says the call "never delays exception selection". appendix-b-timing.tex:46 lists COPI/COPR as a flat 12. Code: a cause value with ID 0 just runs the processing unit at the next phase 0, so no extra slot is used (sirc-vm/peripheral-cpu/src/lib.rs:381-388).
- resolution: design
- fix: Options: (a) an ID-0 call takes 6 cycles and has no dispatch slot (matches the code and the "never delays exception selection" wording); (b) it takes a 6-cycle idle dispatch slot (12 total), the same as every other ID. Recommend (a). State it in the intro, the COPI/COPR timing row (16-coprocessor-instructions.tex:55) and appendix-b-timing.tex:46.
- confidence: medium

### F-arch-8
- file: reference/chapters/02-cpu-architecture.tex
- lines: 113-114
- severity: minor
- category: fact
- claim: "While nothing is pending the CSR is \texttt{0x0000}, which selects the processing unit, so the CPU carries on running the program."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:292 writes any COPI/COPR value, including an ID-0 command such as 0x0123. Nothing clears it except an exception-unit write-back (exception_unit/execution.rs:477) or an invalid-ID fault (lib.rs:442), so the CSR keeps a non-zero ID-0 value indefinitely. The manual (also 16-coprocessor-instructions.tex:212-213, "The CSR is 0x0000 again by the next instruction boundary") is clear.
- resolution: code-wrong
- fix: Keep the manual text. Simulator todo: clear `pending_coprocessor_command` when the selected coprocessor is ID 0 (the call is consumed as a no-op).
- confidence: high

### F-arch-9
- file: reference/chapters/01-introduction.tex
- lines: 98-100
- severity: minor
- category: fact
- claim: "The CPU will revert to this coprocessor at power on, after a reset and when another coprocessor is done and returns control to it."
- evidence: docs/reference/chapters/06-exceptions.tex:874 ("Coprocessor select register (CSR): Set internally to the reset operation until the reset vector sequence completes"); sirc-vm/peripheral-cpu/src/lib.rs:612-625 (reset seeds the cause register with 0x1B00, so the exception unit runs the reset-vector fetch before the processing unit gets control).
- resolution: manual-wrong
- fix: "At power on and after a reset the exception unit first fetches the reset vector; control then passes to the processing unit, which also regains control whenever another coprocessor finishes."
- confidence: high

### F-arch-10
- file: reference/chapters/03-registers.tex
- lines: 121-122
- severity: minor
- category: fact
- claim: "The high 8 bits (bits 31--24) of the high register are not used for addressing"
- evidence: The high register is 16 bits wide. Bits 31-24 are positions in the 32-bit pair figure (Figure fig:addr-pair, lines 106-119), and they are bits 15-8 of the high register, which is how 01-introduction.tex:168 states it. The figure itself labels only bits 23-16 as "High Register" and calls bits 31-24 "Unused", although those bits also belong to the high register.
- resolution: manual-wrong
- fix: "The high 8 bits of the high register (bits 15--8, shown as pair bits 31--24 in the figure) are not used for addressing". Relabel the figure boxes "High register [15:8] (not used for addressing)" and "High register [7:0]".
- confidence: high

### F-arch-11
- file: reference/chapters/03-registers.tex
- lines: 136-137
- severity: minor
- category: contradiction
- claim: "The program counter pair increments after every instruction."
- evidence: 03-registers.tex:219-221 and 02-cpu-architecture.tex:46 say only `pl` advances, by 2, during phase 2, and `ph` is never incremented. A jump or a write to `pl` replaces the advanced value (03-registers.tex:129-131).
- resolution: manual-wrong
- fix: "The low word \reg{pl} advances by 2 during the decode phase of every instruction; \reg{ph} is never incremented."
- confidence: high

### F-arch-12
- file: reference/chapters/03-registers.tex
- lines: 265-269
- severity: minor
- category: fact
- claim: "Instructions such as \mnemonic{LDEA}, \mnemonic{LDEL}, \mnemonic{BRAN}, \mnemonic{BRSR}, \mnemonic{LJMP}, \mnemonic{LJSR}, and \mnemonic{RETS} remain usable in protected mode when they stay within the current segment."
- evidence: The rule at :260-261 compares every written high word in full, including tag bits 15-8 (sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:62-95). `LDEL`/`BRSR`/`LJSR` also write `l` with high word = `ph`, so a protected-mode call faults whenever `lh` != `ph` (for example, after supervisor code left a different segment or tag in `lh`), even when the target is in the current segment. "Stay within the current segment" (the 8-bit segment) understates the condition.
- resolution: manual-wrong
- fix: "...remain usable in protected mode provided every high word they write (the destination pair's, and \reg{lh} for the linking forms, which receives \reg{ph}) is bit-for-bit equal to its current value, tag bits included."
- confidence: high

### F-arch-13
- file: reference/chapters/03-registers.tex
- lines: 252-263
- severity: minor
- category: improvement
- claim: "Direct writes trigger a privilege exception."
- evidence: The section is cited as "the full read/write privilege rules" (03-registers.tex:28,64) but never says that the checks apply only when the condition is true. 10-condition-codes.tex:8-11 ("no privilege check is performed"); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:23-54; tests protected_mode_test.rs:262.
- resolution: n/a
- fix: Add a bullet: "A privilege check is made only when the instruction's condition is true; a skipped instruction never faults on privilege (Chapter~\ref{ch:condition-codes})."
- confidence: high

### F-arch-14
- file: reference/chapters/03-registers.tex
- lines: 236-237
- severity: minor
- category: fact
- claim: "\textbf{Low Byte (bits 7--0):} Condition flags, readable and writable in both modes"
- evidence: docs/reference/chapters/05-status-register.tex:10,85-88 (bits 3-0 are flags; bits 7-4 are reserved and their value after any flag-updating instruction is undefined).
- resolution: manual-wrong
- fix: "\textbf{Low Byte (bits 7--0):} Condition flags (bits 3--0) and reserved bits (bits 7--4), readable and writable in both modes; ..."
- confidence: high

### F-arch-15
- file: reference/chapters/preface.tex
- lines: 55
- severity: minor
- category: fact
- claim: "SIRC Assembly Programmer's Manual  & Complete assembly syntax, directives, macros, pseudoinstructions"
- evidence: docs/assembly/chapters/08-limitations.tex:13 and assembly/chapters/preface.tex:24 (macros are listed as not supported); facts.md "Directives" (only .ORG, .DB/.DW/.DQ, .EQU exist in sirc-vm/toolchain/src/types/data.rs:5-13). The same row appears in docs/assembly/chapters/01-introduction.tex:15.
- resolution: manual-wrong
- fix: "Complete assembly syntax, directives, symbols, meta-instructions and toolchain usage". Apply the same change in assembly/chapters/01-introduction.tex:15.
- confidence: high

### F-arch-16
- file: reference/chapters/02-cpu-architecture.tex
- lines: 240-242
- severity: nit
- category: structure
- claim: "One architectural fact the rest of this chapter relies on: when the reset request (\texttt{RSTI}) is raised the CPU aborts the current instruction or bus wait and restarts at phase 0"
- evidence: This is the last paragraph of the chapter, so nothing after it relies on the fact. Reset then holds RSTO and runs the exception-unit reset-vector fetch (06-exceptions.tex:854-877; appendix-b-timing.tex:80) rather than an instruction at phase 0.
- resolution: n/a
- fix: "When the reset request (\texttt{RSTI}) is raised the CPU abandons the current instruction or bus wait and restarts the phase sequence at phase 0 with the reset-vector fetch (Section~\ref{sec:reset})." Check the label name before applying.
- confidence: medium
