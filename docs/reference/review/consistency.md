# Continuity review: SIRC-1 manual set (reviewer tag `con`)

Scope: `docs/reference` (all chapters, appendices, `generated/`), `docs/hardware`, `docs/assembly`, each read in
`main.tex` order. Claims were checked only against other manual text, never against code.

Mechanical LaTeX checks (all three manuals): every `\ref` target has a `\label`, no duplicate labels, every
`\input` in each `main.tex` exists and every chapter file is included, every `lstlisting`, `instructionbox` and
`example` is balanced and unnested, and no listing contains a backslash command. **No `latex` findings.**

---

## Cross-manual contradictions

### F-con-1
- file: assembly/chapters/06-toolchain.tex
- lines: 24
- severity: blocker
- category: contradiction
- claim: "    COPI #0x14FF                ; stop the simulator (test exit)"
- evidence: reference/chapters/06-exceptions.tex:950 ("Exception coprocessor opcodes \texttt{0x2}--\texttt{0x8} are reserved; issuing one raises an invalid-opcode fault."); reference/chapters/16-coprocessor-instructions.tex:217 (same rule); reference/chapters/16-coprocessor-instructions.tex:201 (exception unit operations are 0x1, 0x9--0xD, 0x0, 0xE--0xF only)
- resolution: design
- fix: The assembly manual's worked example (Chapter 6, the only complete program in the manual) relies on exception-unit operation 0x4, which the architecture defines as reserved and faulting. Options: (a) document 0x4 in the assembly manual as a simulator-only test hook (state in Chapter 6 and in Chapter 8 that on a conforming CPU it raises an invalid-opcode fault), (b) assign a simulator/debug coprocessor ID from the 0x4--0xF model space in the architecture manual and change the example to use it, or (c) end the example with `WAIT`/a `BRAN` loop. Recommend (a) now and (b) as a tracked decision.
- confidence: high

### F-con-2
- file: assembly/chapters/03-operands.tex
- lines: 4-32
- severity: blocker
- category: contradiction
- claim: "This chapter lists every operand form the assembler accepts."
- evidence: reference/chapters/08-addressing-modes.tex:9-10 and 106-129 (Table "Zero-Displacement Shorthand": `(addr)`, `(addr)+`, `-(addr)` are "exact assembler-level equivalents"); reference/chapters/14-memory-instructions.tex:71-74; reference/chapters/15-control-flow.tex:65-70 (`LJSR (src)+` "is legal"); the shorthand is used in assembled listings at reference/chapters/03-registers.tex:354-363 and reference/chapters/05-status-register.tex:233
- resolution: manual-wrong
- fix: Add three rows to Table 3.1 (`(addr)`, `(addr)+`, `-(addr)`, "zero displacement shorthand for `(#0, addr)` ...") and one sentence that the shorthand exists only for immediate displacements, and that `LJMP (src)`/`LJSR (src)` are not accepted (reference/chapters/08-addressing-modes.tex:98).
- confidence: high

### F-con-3
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 48
- severity: blocker
- category: contradiction
- claim: "\item \textbf{Max Rate:} 24 MHz"
- evidence: hardware/chapters/06-electrical-characteristics.tex:4-12 ("The only electrical requirement currently specified is the supply" ... "Clock frequency range" not yet specified); hardware/README.md:37-38; the architecture manual also asserts the rate: reference/chapters/appendix-b-timing.tex:76 ("22 microseconds at 24~MHz") and :243 ("At 24 MHz (maximum rated clock)")
- resolution: design
- fix: Known from the period benchmark. Either specify 24 MHz in Chapter 6 (and change "The only electrical requirement currently specified is the supply"), or delete the `Max Rate` bullet. In both cases the architecture manual should not state a rated clock: change reference/chapters/appendix-b-timing.tex:243 to "At 24 MHz (example)" or drop it, and phrase :76 as "(22 microseconds at an example 24~MHz clock)". Recommend removing it from HW Chapter 3 until Chapter 6 fixes a value, as STYLE.md §9 requires.
- confidence: high

### F-con-4
- file: assembly/chapters/01-introduction.tex
- lines: 15
- severity: major
- category: contradiction
- claim: "SIRC Assembly Programmer's Manual  & Complete assembly syntax, directives, macros, pseudoinstructions (this manual)"
- evidence: assembly/chapters/08-limitations.tex:13 and 20 (macros and multi-instruction pseudoinstructions not supported); assembly/chapters/04-instructions.tex:104-105; reference/chapters/17-meta-instructions.tex:7-8 and reference/chapters/16-coprocessor-instructions.tex:1207-1210 (architecture forbids multi-instruction meta-instructions)
- resolution: manual-wrong
- fix: Known from the period benchmark. Replace "Complete assembly syntax, directives, macros, pseudoinstructions" with "Complete assembly syntax, directives, symbols, and the toolchain" in all copies of the manual-set table: assembly/chapters/01-introduction.tex:15, hardware/chapters/01-introduction.tex:15, reference/chapters/preface.tex:55, hardware/README.md:8, assembly/README.md:8.
- confidence: high

### F-con-5
- file: hardware/chapters/04-bus-interface.tex
- lines: 28
- severity: major
- category: contradiction
- claim: "SYNC & Asserted while phase 0 (Instruction Fetch, high word) of an instruction or exception-unit dispatch is active."
- evidence: hardware/chapters/03-signal-descriptions.tex:63 (same wording); reference/chapters/12-instruction-summary.tex:185-186 and reference/chapters/16-coprocessor-instructions.tex:55, 730-736 (every coprocessor call, including DMA and maths commands, takes a further six-phase dispatch slot beginning at phase 0; DMA phase 0 is a setup cycle with no bus access); reference/chapters/16-coprocessor-instructions.tex:211-214 (coprocessor 0 dispatch)
- resolution: design
- fix: The hardware manual defines SYNC only for instructions and exception-unit dispatches, but the architecture has dispatch slots for every coprocessor and for DMA continuation slots. Also, HW labels phase 0 "Instruction Fetch, high word" for a dispatch that fetches no instruction. Options: (a) SYNC marks phase 0 of every six-phase slot (instruction, any coprocessor dispatch, DMA continuation), or (b) SYNC marks only instruction fetches and exception-vector fetches. State the chosen rule in both HW locations and drop "(Instruction Fetch, high word)". Recommend (a), which matches the architecture's slot model.
- confidence: medium

### F-con-6
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 253-254
- severity: major
- category: contradiction
- claim: "holding the pin asserted after its handler returns does not by itself cause it to fire again -- a fresh falling-then-rising transition is required."
- evidence: hardware/chapters/03-signal-descriptions.tex:243, 247-248 (NMI is Active Low; "the pin must be released and reasserted to fire again"); hardware/chapters/04-bus-interface.tex:97; reference/chapters/06-exceptions.tex:501-502 ("must be withdrawn and raised again")
- resolution: manual-wrong
- fix: For an active-low pin, release then reassert is a rising-then-falling transition, and the falling edge is what fires. Replace with "a fresh release and reassertion (a rising edge followed by a falling edge) is required."
- confidence: high

### F-con-7
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 149-151
- severity: minor
- category: contradiction
- claim: "A memory controller might trigger a bus error if BAT is 010 (data read), PROT is 1 (protected mode) and the address is in a protected memory region."
- evidence: hardware/chapters/03-signal-descriptions.tex:132-133 and 187-188 (protected regions are reported with BPER); reference/chapters/06-exceptions.tex:205-206 (accessing a protected memory region from protected mode is a Bus Protection Fault)
- resolution: manual-wrong
- fix: "A memory controller might assert BPER (bus protection error) if BAT is 010 (data read), PROT is 1 (protected mode) and the address is in a protected memory region."
- confidence: high

### F-con-8
- file: hardware/chapters/05-bus-timing.tex
- lines: 191-215
- severity: minor
- category: contradiction
- claim: "IRQ1--IRQ4  & L L H H H"
- evidence: hardware/chapters/03-signal-descriptions.tex:233-234 ("The requesting device must hold the line asserted until its handler acknowledges it"); hardware/chapters/04-bus-interface.tex:96; reference/chapters/06-exceptions.tex:462-463
- resolution: design
- fix: The figure shows the request withdrawn during the vector fetch, before any handler instruction runs, which the normative text forbids a device to do (the architecture tolerates it only because the bit was already latched). Either keep IRQ asserted (`L L L L L`) to the end of the figure, or add a sentence that the figure shows only the latch and dispatch and that a real device keeps the line asserted until its handler acknowledges it. Recommend the waveform change.
- confidence: medium

### F-con-9
- file: reference/chapters/preface.tex
- lines: 42-44
- severity: minor
- category: contradiction
- claim: "nor software conventions built on top of the architecture --- register usage conventions, calling conventions, stack frame layout, or object/binary file format --- which are the concern of the (forthcoming) SIRC-1 ABI and Toolchain documentation."
- evidence: assembly/chapters/01-introduction.tex:23-36 and assembly/chapters/06-toolchain.tex:47-63 (the Assembly Programmer's Manual now covers the toolchain, object files and linker); reference/chapters/03-registers.tex:94-95 repeats the reference
- resolution: design
- fix: The "SIRC-1 ABI and Toolchain documentation" does not exist under that name. Either point toolchain and object-file topics to the SIRC-1 Assembly Programmer's Manual and keep a separate "(forthcoming) SIRC-1 ABI" only for calling conventions, or name the Assembly Programmer's Manual for both. Apply the same change at reference/chapters/03-registers.tex:94-95.
- confidence: high

### F-con-10
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 629-630
- severity: minor
- category: contradiction
- claim: "Values above \imm{7} are architecturally undefined; the assembler currently masks \texttt{\#n} to four bits without diagnosing the error."
- evidence: STYLE.md Scope ("ASM owns what the tools accept and print"); assembly/chapters/07-diagnostics.tex and assembly/chapters/08-limitations.tex:30-35 do not mention it
- resolution: manual-wrong
- fix: Scope drift: toolchain behavior is stated only in the architecture manual. Move the sentence "the assembler masks the link-register index to four bits without a diagnostic" into the assembly manual's Chapter 8 behaviors list, and replace it here with "Values above \imm{7} are architecturally undefined." The same applies, at lower priority, to the DMA count range check at :958 and :1057, which the assembly manual does not list among its diagnostics.
- confidence: medium

### F-con-11
- file: assembly/chapters/03-operands.tex
- lines: 69-70
- severity: minor
- category: contradiction
- claim: "A register-form ALU instruction with two register operands is shorthand for the three-operand form with the destination repeated as the first source."
- evidence: reference/chapters/13-alu-instructions.tex:93-111 and 118-121 (test forms `CMPR rS1, rS2` have no destination and no three-operand form, so two operands are their full form); reference/chapters/08-addressing-modes.tex:91-92
- resolution: manual-wrong
- fix: "A save-result register-form ALU instruction (for example \mnemonic{ADDR}) written with two register operands is shorthand ... The test forms (\mnemonic{CMPR}, \mnemonic{TSAR} and so on) always take exactly two registers."
- confidence: medium

---

## Contradictions inside the architecture manual

### F-con-12
- file: reference/chapters/10-condition-codes.tex
- lines: 314-316
- severity: major
- category: contradiction
- claim: "A predicated instruction costs the same six cycles whether or not it executes"
- evidence: reference/chapters/16-coprocessor-instructions.tex:159-160, 738-740, 1007-1010 (a coprocessor call takes 12 cycles when its condition is true and 6 when false; DMA longer); reference/chapters/appendix-b-timing.tex:40-41 and 59-60
- resolution: manual-wrong
- fix: "An ordinary processing-unit instruction costs the same six cycles whether or not it executes. A coprocessor call whose condition is false skips its dispatch slot and takes 6 cycles instead of 12 or more (Chapter~\ref{ch:coprocessor})." In appendix-b-timing.tex:40-41 relabel the rows "Conditional processing-unit instruction (executed/not executed)".
- confidence: high

### F-con-13
- file: reference/chapters/03-registers.tex
- lines: 121-122
- severity: major
- category: contradiction
- claim: "The high 8 bits (bits 31--24) of the high register are not used for addressing"
- evidence: reference/chapters/01-introduction.tex:168 ("The high 8 bits of the high register (bits 15--8)"); reference/chapters/04-data-representation.tex:46
- resolution: manual-wrong
- fix: "The high 8 bits of the high register (bits 15--8 of the high register, bits 31--24 of the pair) are not used for addressing"
- confidence: high

### F-con-14
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 6-8
- severity: minor
- category: contradiction
- claim: "The primary coprocessor is the Exception Unit (Coprocessor 1), which handles exceptions, interrupts, and privileged operations."
- evidence: reference/chapters/01-introduction.tex:98-100 ("The primary coprocessor is the "processing unit" (coprocessor \texttt{0x0})"); reference/chapters/02-cpu-architecture.tex:104
- resolution: manual-wrong
- fix: "The processing unit (coprocessor 0) executes ordinary instructions; the principal coprocessor reached through coprocessor calls is the Exception Unit (Coprocessor 1), which handles exceptions, interrupts, and privileged operations."
- confidence: high

### F-con-15
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 553-554
- severity: major
- category: contradiction
- claim: "General-purpose registers, the \reg{a}, \reg{l}, and \reg{s} pairs, and the exception link registers are not affected by reset and hold undefined values after power-on."
- evidence: reference/chapters/03-registers.tex:17-18 ("all other registers are architecturally undefined" after reset); reference/chapters/06-exceptions.tex:869-871 ("Not architecturally cleared by reset. Software should treat their contents as undefined after reset.")
- resolution: design
- fix: "Not affected by reset" promises preservation across a software reset; the other two chapters promise nothing. Options: (a) architecture guarantees preservation across reset (state it in 03-registers and the 06 Reset State table), or (b) contents are architecturally undefined after any reset (change this box to "are not cleared by reset; software must treat their contents as architecturally undefined"). Recommend (b), the weaker contract.
- confidence: high

### F-con-16
- file: reference/chapters/appendix-a-opcode-map.tex
- lines: 173, 181, 188
- severity: minor
- category: contradiction
- claim: "MOV                & \opcode{07}        & \opcode{27}\textsuperscript{*} & \opcode{37}"
- evidence: reference/chapters/07-instruction-formats.tex:351-353 ("Opcodes \texttt{0x27} and \texttt{0x2F} ... are undocumented opcodes ..., not a "save result" operation or a coprocessor call"); reference/chapters/appendix-c-undocumented.tex:6-9; the same framing as a syntax-less MOV/COP variant appears at reference/chapters/13-alu-instructions.tex:123 ("Short-immediate move: Opcode \opcode{27} has no public assembly syntax") and reference/chapters/16-coprocessor-instructions.tex:38-39 and :80
- resolution: design
- fix: Decide whether 0x27/0x2F are "short-immediate MOVI/COPI with no syntax" or unassigned opcodes with architecturally undefined behavior. If the latter (as Chapter 7 and Appendix C say), move them out of the MOV and COP rows of Table A.3 into a footnote row "Undocumented", rename the ch13 paragraph from "Short-immediate move" to "Opcode 0x27", and in ch16 say "Opcode 0x2F is undocumented (Appendix C); it is not a coprocessor call".
- confidence: medium

### F-con-17
- file: reference/chapters/03-registers.tex
- lines: 136-137
- severity: minor
- category: contradiction
- claim: "The program counter pair increments after every instruction."
- evidence: reference/chapters/03-registers.tex:219-221 and reference/chapters/02-cpu-architecture.tex:46 (\reg{pl} advances by 2 during decode; \reg{ph} is never incremented)
- resolution: manual-wrong
- fix: "The low word of the program counter, \reg{pl}, advances by 2 during the decode phase of every instruction; \reg{ph} is never incremented."
- confidence: high

---

## Structure

### F-con-18
- file: reference/chapters/02-cpu-architecture.tex
- lines: 146-147
- severity: minor
- category: structure
- claim: "Detecting segment overflow in the address calculation, and when an instruction would use a program-counter advance that wraps the segment"
- evidence: no definition of "segment" in reference/chapters/notation-and-glossary.tex:222-258 or chapters 1--2; the meaning (the 64K-word range selected by the high word) is only implied at reference/chapters/03-registers.tex:265-266 and reference/chapters/15-control-flow.tex:177-178
- resolution: n/a
- fix: Add a glossary entry: "Segment: the 64K-word range of addresses that share one high-word value (bits 7--0 of an address register pair's high word). Displacement arithmetic wraps within a segment; see Chapter~\ref{ch:data-representation}."
- confidence: high

### F-con-19
- file: reference/chapters/05-status-register.tex
- lines: 206-208
- severity: nit
- category: structure
- claim: "an encoding of the opcode with AF \texttt{10}, not reachable from public assembly syntax"
- evidence: AF is first defined at reference/chapters/07-instruction-formats.tex:51-52; no forward reference here
- resolution: n/a
- fix: "an encoding of the opcode with the status-update field (AF, Chapter~\ref{ch:instruction-formats}) set to \texttt{10}"
- confidence: high

### F-con-20
- file: reference/chapters/07-instruction-formats.tex
- lines: 359-367
- severity: minor
- category: structure
- claim: "\textbf{Opcodes (Imm / Short / Reg)}" with rows such as "Add ... & \opcode{08} / \opcode{28} / \opcode{38}"
- evidence: reference/chapters/12-instruction-summary.tex:28 and 36 (ADDI is 0x00, CMNI is 0x08); the column gives only the test-only opcodes although the row also names the save-result mnemonics
- resolution: n/a
- fix: Split the column into "Save opcodes (Imm / Short / Reg)" (`00 / 20 / 30`) and "Test opcodes" (`08 / 28 / 38`), or rename it "Test-only opcodes (Imm / Short / Reg)".
- confidence: high

### F-con-21
- file: reference/chapters/12-instruction-summary.tex
- lines: 26-101
- severity: minor
- category: structure
- claim: "See Table~\ref{tab:flag-effect-symbols} for the flag-effect symbol definitions."
- evidence: the table above uses "NZCV" and "NZ (C,V=0)", not the `* 0 1 - S U` symbols of reference/chapters/11-reading-instructions.tex:96-112 used by reference/chapters/13-alu-instructions.tex:167-192 and reference/chapters/appendix-e-quick-reference.tex:12-17 (STYLE.md §3 "Status flags")
- resolution: n/a
- fix: Either convert the Flags column to four N/Z/C/V symbol columns, as in Table E.1, or replace the sentence with "Flags lists the condition flags each instruction updates; Table~\ref{tab:alu-status-flag-effects} gives the exact effect."
- confidence: high

### F-con-22
- file: reference/chapters/appendix-e-quick-reference.tex
- lines: 67, 74, 99, 120
- severity: nit
- category: structure
- claim: "\multicolumn{8}{l}{\textit{D -- Ch.\ref{ch:meta-instructions}}}"
- evidence: reference/chapters/17-meta-instructions.tex:27-39 lists Chapter 16 as the primary documentation for DIVS, DIVU, DMAR, DMAT, DMAW, ETFR, ETTR, EXCP, RETE, RSET, WAIT, MULS, MULU
- resolution: n/a
- fix: Add `/ Ch.\ref{ch:coprocessor}` to the D, E, M, R and W group headers (D and W: Ch.\ref{ch:coprocessor} / Ch.\ref{ch:meta-instructions}).
- confidence: high

### F-con-23
- file: assembly/chapters/02-source-language.tex
- lines: 61-62
- severity: minor
- category: structure
- claim: "Two directive operands are written without the hash: the address of \texttt{.ORG} is a bare decimal or \texttt{0x}-prefixed hexadecimal number"
- evidence: only one operand is named; assembly/chapters/05-directives-and-symbols.tex:52 shows the `.EQU` name is written `$NAME`, also without a hash
- resolution: n/a
- fix: "Two directive operands are written without the hash: the address of \texttt{.ORG}, a bare decimal or \texttt{0x}-prefixed hexadecimal number (Section~\ref{sec:org}), and the name of \texttt{.EQU}, written \texttt{\$NAME}." If only .ORG was meant, change "Two directive operands are" to "One directive operand is".
- confidence: medium

### F-con-24
- file: hardware/chapters/05-bus-timing.tex
- lines: 4-215
- severity: minor
- category: improvement
- claim: "The timing diagrams below show cycle-level logical timing."
- evidence: reference/chapters/16-coprocessor-instructions.tex:729-744 defines DMA bus behavior (setup phase 0 with no bus access, one access per cycle from phase 1, DMAT reads a chunk then writes it, BAT 101/110); reference/chapters/appendix-b-timing.tex:10 sends the reader to the hardware manual for bus timing; hardware/chapters/03-signal-descriptions.tex:146-147 only mentions DMA prefetch in passing
- resolution: n/a
- fix: Add a DMA burst figure (for example a 3-word DMAR: setup cycle with BAS deasserted, then three DR-type accesses with BAT 101, BAS per access, wait-state behavior), plus one sentence on what SYNC does during DMA continuation slots (see F-con-5). Needs author approval (STYLE.md approval 8 lists hardware additions not yet authorized).
- confidence: medium

---

## Terminology (STYLE.md)

### F-con-25
- file: hardware/chapters/preface.tex
- lines: n/a
- severity: minor
- category: terminology
- claim: "behaviour"
- evidence: STYLE.md §1 and §8 (American spelling set-wide)
- resolution: n/a
- fix: behaviour -> behavior. Files: hardware/chapters/preface.tex, hardware/chapters/01-introduction.tex, assembly/chapters/preface.tex, assembly/chapters/08-limitations.tex (also hardware/README.md). Pattern: `grep -rn 'behaviour' docs/*/chapters`
- confidence: high

### F-con-26
- file: hardware/chapters/preface.tex
- lines: n/a
- severity: minor
- category: terminology
- claim: "active-low signalling"
- evidence: STYLE.md §8
- resolution: n/a
- fix: signalling -> signaling. Files: hardware/chapters/preface.tex. Pattern: `grep -rn 'signalling' docs/*/chapters`
- confidence: high

### F-con-27
- file: assembly/chapters/04-instructions.tex
- lines: n/a
- severity: minor
- category: terminology
- claim: "The assembler recognises the mnemonics"
- evidence: STYLE.md §8
- resolution: n/a
- fix: recognise/recognised -> recognize/recognized. Files: assembly/chapters/04-instructions.tex (text and caption), reference/chapters/notation-and-glossary.tex. Pattern: `grep -rn 'recognis' docs/*/chapters`
- confidence: high

### F-con-28
- file: assembly/chapters/06-toolchain.tex
- lines: n/a
- severity: minor
- category: terminology
- claim: "the segment is initialised from it"
- evidence: STYLE.md §8
- resolution: n/a
- fix: initialised -> initialized. Files: assembly/chapters/06-toolchain.tex. Pattern: `grep -rn 'initialis' docs/*/chapters`
- confidence: high

### F-con-29
- file: reference/chapters/11-reading-instructions.tex
- lines: n/a
- severity: minor
- category: terminology
- claim: "A box labelled with an operand name"
- evidence: STYLE.md §8 and approval 2 (single-l labeled)
- resolution: n/a
- fix: labelled -> labeled. Files: reference/chapters/11-reading-instructions.tex, hardware/chapters/05-bus-timing.tex. Pattern: `grep -rn 'labelled' docs/*/chapters`
- confidence: high

### F-con-30
- file: reference/chapters/06-exceptions.tex
- lines: n/a
- severity: minor
- category: terminology
- claim: "The instruction is cancelled before it takes effect"
- evidence: STYLE.md §8 and approval 2 (single-l canceled)
- resolution: n/a
- fix: cancelled/cancelling -> canceled/canceling (12 hits). Files: reference/chapters/02-cpu-architecture.tex, reference/chapters/06-exceptions.tex, reference/chapters/14-memory-instructions.tex, reference/chapters/15-control-flow.tex, reference/chapters/16-coprocessor-instructions.tex, reference/chapters/appendix-b-timing.tex. Pattern: `grep -rn 'ancell' docs/*/chapters`
- confidence: high

### F-con-31
- file: reference/chapters/appendix-d-examples.tex
- lines: n/a
- severity: nit
- category: terminology
- claim: "The assembly syntax used in these programs is summarised"
- evidence: STYLE.md §8
- resolution: n/a
- fix: summarised -> summarized. Files: reference/chapters/appendix-d-examples.tex. Pattern: `grep -rn 'summaris' docs/*/chapters`
- confidence: high

### F-con-32
- file: hardware/chapters/03-signal-descriptions.tex
- lines: n/a
- severity: minor
- category: terminology
- claim: "BAS remains asserted until the bus responds with BACK, BERR, or BPER."
- evidence: STYLE.md §3 "Active-low names [HW]" and Gate 1 2026-11-02 approval 3 (`\activelow{}` in prose, tables and timing-diagram labels; headings keep the plain name); the hardware manual has 0 uses of `\activelow`
- resolution: n/a
- fix: Wrap every active-low signal name (BAS, BACK, BERR, BPER, HALT, TRCE, IRQ1--IRQ4, NMI, RSTI, RSTO) in `\activelow{}` outside section headings and `Signal Type:` bullets. Files: hardware/chapters/01-introduction.tex, 02-package-and-pinout.tex, 03-signal-descriptions.tex, 04-bus-interface.tex, 05-bus-timing.tex. Pattern: `grep -nE '\b(BAS|BACK|BERR|BPER|HALT|TRCE|IRQ[1-4]|NMI|RSTI|RSTO)\b' docs/hardware/chapters/*.tex | grep -v activelow`. Separately, the architecture manual writes the same signals as `\texttt{BERR}` etc. (reference/chapters/notation-and-glossary.tex, 02-cpu-architecture.tex, 06-exceptions.tex, 16-coprocessor-instructions.tex, appendix-b-timing.tex), which the glossary at notation-and-glossary.tex:200-202 declares as the convention; the author should decide whether the overbar rule extends to ARCH.
- confidence: high

### F-con-33
- file: hardware/chapters/04-bus-interface.tex
- lines: n/a
- severity: minor
- category: terminology
- claim: "\section{Bus Transaction Completion}"
- evidence: STYLE.md §1 and approval 11 (`bus cycle` over `bus transaction`; the heading changes)
- resolution: n/a
- fix: bus transaction -> bus cycle (heading becomes "Bus Cycle Completion"). Files: hardware/chapters/04-bus-interface.tex, reference/chapters/06-exceptions.tex, reference/chapters/appendix-e-quick-reference.tex. Pattern: `grep -rni 'bus transaction' docs/*/chapters`
- confidence: high

### F-con-34
- file: hardware/chapters/03-signal-descriptions.tex
- lines: n/a
- severity: nit
- category: terminology
- claim: "\item \textbf{Direction:} Bi-directional"
- evidence: STYLE.md §1 (`Direction:` value is `Bidirectional`)
- resolution: n/a
- fix: Bi-directional -> Bidirectional. Files: hardware/chapters/03-signal-descriptions.tex. Pattern: `grep -rni 'bi-directional' docs/*/chapters`
- confidence: high

### F-con-35
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: n/a
- severity: nit
- category: terminology
- claim: "the run-time value of \reg{rS}"
- evidence: STYLE.md §1 (runtime, not run-time/run time)
- resolution: n/a
- fix: run-time / run time -> runtime. Files: reference/chapters/08-addressing-modes.tex, reference/chapters/16-coprocessor-instructions.tex. Pattern: `grep -rnE 'run-time|run time' docs/*/chapters`
- confidence: high

### F-con-36
- file: assembly/chapters/preface.tex
- lines: n/a
- severity: minor
- category: terminology
- claim: "This manual assumes you have read the Assembly Language Summary"
- evidence: STYLE.md §5 ("you", "your" and "we" banned in all three manuals, back covers included; "Source" for the "You wrote" column; "This manual contains")
- resolution: n/a
- fix: Remove second person. Files: assembly/chapters/preface.tex ("assumes the reader has read"), assembly/chapters/07-diagnostics.tex (column "You wrote" -> "Source"), reference/chapters/back-cover.tex, hardware/chapters/back-cover.tex, assembly/chapters/back-cover.tex ("Inside you will find:" -> "This manual contains:"). Pattern: `grep -rnwiE 'you|your|we' docs/*/chapters`
- confidence: high

### F-con-37
- file: assembly/chapters/preface.tex
- lines: n/a
- severity: minor
- category: terminology
- claim: "The architecture manual defines what each instruction does"
- evidence: STYLE.md Scope ("Always use `\archmanual{}` or `\hwmanual{}` for the book title ..., not "the architecture manual"")
- resolution: n/a
- fix: "the architecture manual" -> `\archmanual{}` (10 hits). Files: assembly/chapters/preface.tex, 01-introduction.tex, 03-operands.tex, 04-instructions.tex (text and Table 4.2 header), 06-toolchain.tex; hardware/chapters/preface.tex, 01-introduction.tex. Pattern: `grep -rni 'architecture manual' docs/{hardware,assembly}/chapters`
- confidence: high

### F-con-38
- file: reference/chapters/06-exceptions.tex
- lines: n/a
- severity: minor
- category: terminology
- claim: "(see the Force Trace Mode Input in the \textit{SIRC-1 Hardware Reference Manual})"
- evidence: STYLE.md Scope (use `\hwmanual{}`); reference/main.tex defines neither `\hwmanual` nor an assembly-manual macro (hardware/main.tex:15 and assembly/main.tex:15-16 do), so ARCH types the titles by hand and drifts: "SIRC Assembly Programmer's Manual" at reference/chapters/notation-and-glossary.tex:59 and appendix-d-examples.tex:9 against the title "SIRC-1 Assembly Programmer's Manual"; the manual-set tables in all three manuals also drop the "-1"
- resolution: n/a
- fix: Define `\hwmanual` and an `\asmmanual` macro in reference/main.tex (and `\asmmanual` in hardware/main.tex) and use them for every title. Files: reference/chapters/notation-and-glossary.tex, 02-cpu-architecture.tex, 06-exceptions.tex, appendix-b-timing.tex, appendix-d-examples.tex, appendix-e-quick-reference.tex; manual-set tables in reference/chapters/preface.tex, hardware/chapters/01-introduction.tex, assembly/chapters/01-introduction.tex. Pattern: `grep -rnE 'textit\{SIRC|SIRC (Architecture|Assembly|Hardware)' docs/*/chapters`
- confidence: high

### F-con-39
- file: hardware/chapters/03-signal-descriptions.tex
- lines: n/a
- severity: nit
- category: terminology
- claim: "Unlike \texttt{IRQ1}--\texttt{IRQ4}, which are level-sensitive"
- evidence: STYLE.md §3 "Signal and pin names [HW]" (plain upper case; `\texttt` only for logic letters and cycle labels)
- resolution: n/a
- fix: Drop `\texttt{}` from signal and pin names. Files: hardware/chapters/03-signal-descriptions.tex, 04-bus-interface.tex, 05-bus-timing.tex, 06-electrical-characteristics.tex. Pattern: `grep -nE '\\texttt\{(BAS|BACK|BERR|BPER|HALT|TRCE|IRQ|NMI|RSTI|RSTO|VCC|GND|CLKI|BAT|PROT|SYNC)' docs/hardware/chapters/*.tex`
- confidence: high

### F-con-40
- file: hardware/chapters/03-signal-descriptions.tex
- lines: n/a
- severity: nit
- category: terminology
- claim: "\section{Power Pins (VCC1--2, GND1--2)}" and "connected to 5V +/- 5\%"
- evidence: STYLE.md §3 "Signal ranges [HW]" (not `VCC1--2`) and "Units [HW]" (`5~V $\pm$ 5\%`, not `5V`, not `+/-`); hardware/chapters/06-electrical-characteristics.tex:5 already uses the correct form
- resolution: n/a
- fix: "Power Pins (VCC1--VCC2, GND1--GND2)"; "the 5~V power"; "5~V $\pm$ 5\%". Files: hardware/chapters/03-signal-descriptions.tex. Pattern: `grep -nE 'VCC1--2|GND1--2|[0-9]V\b|\+/-' docs/hardware/chapters/*.tex`
- confidence: high

---

## Count

Counted by the manual named in each finding's `file`.

| Category | ARCH | HW | ASM | Total |
|---|---|---|---|---|
| contradiction | 8 (F9, F10, F12--F17) | 5 (F3, F5--F8) | 4 (F1, F2, F4, F11) | 17 |
| structure | 5 (F18--F22) | 0 | 1 (F23) | 6 |
| improvement | 0 | 1 (F24) | 0 | 1 |
| terminology | 5 (F29--F31, F35, F38) | 7 (F25, F26, F32--F34, F39, F40) | 4 (F27, F28, F36, F37) | 16 |
| latex | 0 | 0 | 0 | 0 |
| **Total** | 18 | 13 | 9 | 40 |

Eleven of the seventeen contradictions are cross-manual (F1--F11).

Most damaging: F1 (the assembly manual's worked example exits through a reserved, faulting exception-unit operation),
F2 (the assembly manual's "every operand form" table omits the zero-displacement shorthand that the architecture manual
calls legal and uses in assembled listings), F12 (the claim that a predicated instruction always costs six cycles
contradicts the 12-cycle and 6-cycle timing of coprocessor calls).
