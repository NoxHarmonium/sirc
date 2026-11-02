# Triaged findings

Verified: all 117 blocker/major fact and contradiction findings confirmed (verify-1..8.md).

### F-asm-1
- file: assembly/chapters/03-operands.tex
- lines: 69-76
- severity: blocker
- category: contradiction
- claim: "These two lines assemble to the same instruction: ... ADDR r1, r2, r3 ; r1 = r2 + r3 / ADDR r1, r2 ; r1 = r1 + r2"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:102-121 (two-register form sets r2 = destination). Assembled and linked: `ADDR r1, r2, r3` = 0xC0048010, `ADDR r1, r2` = 0xC0004010, `ADDR r1, r1, r2` = 0xC0004010. The listing's own comments give two different operations.
- resolution: manual-wrong
- fix: Change the first listing line to `    ADDR r1, r1, r2         ; r1 = r1 + r2` (keep the second line as is).
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted
- decision: Document COPI #0x14FF in assembly Ch.6 and Ch.8 as a simulator-only test hook that raises an invalid-opcode fault on a conforming CPU; todo 25's debug exit register replaces it later.

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
- status: merged into F-hw-1

### F-hw-1
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 48
- severity: blocker
- category: contradiction
- claim: "\item \textbf{Max Rate:} 24 MHz"
- evidence: hardware/chapters/06-electrical-characteristics.tex:7-13,19 ("Clock frequency range" is listed as not yet specified, and "designs must not assume any value"); reference/chapters/appendix-b-timing.tex:76 ("22 microseconds at 24~MHz") and :243 ("At 24 MHz (maximum rated clock)")
- resolution: design
- fix: Already-known item, reported once. Locations: hw Ch.3 line 48, hw Ch.6 lines 13 and 19, arch Appendix B lines 76 and 243. Option A: keep 24 MHz as the rated maximum, move it into Ch.6 as the one specified clock characteristic (and remove "Clock frequency range" from the unspecified list). Option B: delete the Max Rate bullet from Ch.3, keep Ch.6, and change Appendix B to "at a 24 MHz clock (example)". Recommend B: nothing else in the set backs a rated figure, and Ch.6 already forbids assuming one.
- confidence: high
- status: accepted
- decision: 24 MHz is an example clock only, not a rating. Delete the CLKI 'Max Rate' bullet in hardware Ch.3; hardware Ch.6 keeps the clock range unspecified; Appendix B says 'at an example 24 MHz clock' (both places) and drops 'maximum rated clock'.

### F-enc-1
- file: reference/chapters/07-instruction-formats.tex
- lines: 608-610
- severity: blocker
- category: contradiction
- claim: "Every read of \reg{p}, every address calculation that selects \reg{p}, and every link value uses that latched address, so they all see the instruction's own address"
- evidence: reference/chapters/15-control-flow.tex:271-272 ("the BRSR link value is that address plus 2 (the address of the next instruction)"); reference/chapters/08-addressing-modes.tex:162-163 ("still link to the instruction's own address + 2"); sirc-vm/peripheral-cpu/tests/instructions/ljmp_test.rs:225,292 (pl 0xFAC0, link holds 0xFAC2)
- resolution: manual-wrong
- fix: "Every read of \reg{p} and every address calculation that selects \reg{p} uses that latched address, so they see the instruction's own address even though \reg{pl} has already advanced by the time the address is calculated in phase 3. A link value is the latched address plus 2, the address of the next instruction."
- confidence: high
- status: accepted

### F-sum-1
- file: reference/chapters/12-instruction-summary.tex
- lines: 185-188
- severity: blocker
- category: contradiction
- claim: "Coprocessor calls (\mnemonic{COPI}, \mnemonic{COPR} and every meta-instruction that assembles to one) take a second 6-cycle slot for the coprocessor dispatch that begins at the next phase 0, for 12 cycles in total before the next instruction is fetched."
- evidence: reference/chapters/appendix-b-timing.tex:47-49 (RSET = 18 cycles, WAIT = 12 + idle); appendix-b-timing.tex:58-59 and 16-coprocessor-instructions.tex:739,1009 (a false condition does not dispatch and takes 6 cycles)
- resolution: manual-wrong
- fix: "A coprocessor call whose condition is true (\mnemonic{COPI}, \mnemonic{COPR} and every meta-instruction that assembles to one) takes a second 6-cycle slot for the coprocessor dispatch that begins at the next phase 0, so most calls take 12 cycles before the next instruction is fetched. \mnemonic{RSET}, \mnemonic{WAIT}, DMA transfers and maths-unit operations take longer, and a call whose condition is false takes 6 cycles; Appendix~\ref{appendix:timing} gives every case."
- confidence: high
- status: accepted

### F-asm-2
- file: assembly/chapters/01-introduction.tex
- lines: 15
- severity: major
- category: contradiction
- claim: "SIRC Assembly Programmer's Manual  & Complete assembly syntax, directives, macros, pseudoinstructions (this manual)"
- evidence: assembly/chapters/08-limitations.tex:13,20 and 04-instructions.tex:104-105 (no macros, no multi-instruction pseudoinstructions). Known; already reported in review/consistency.md:52-55 and review/facts-arch.md:164-165.
- resolution: manual-wrong
- fix: Report once. Apply the consistency.md fix ("Complete assembly syntax, directives, symbols, and the toolchain") at assembly/chapters/01-introduction.tex:15, hardware/chapters/01-introduction.tex:15, reference/chapters/preface.tex:55 and both READMEs.
- confidence: high
- status: merged into F-con-4

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
- status: accepted

### F-asm-10
- file: assembly/chapters/02-source-language.tex
- lines: 71-80
- severity: major
- category: fact
- claim: "\texttt{.EQU} defines a named constant. The name starts with a dollar sign at the definition and at every use"
- evidence: Placeholders resolve in token order, so a use before its .EQU panics with "Could not find a value for placeholder name [X]" (object.rs:22-23,93-94,192-194; reproduced with `.DQ $X` ahead of its definition). A second .EQU of the same name silently replaces the value for later uses (HashMap insert, object.rs:193). Labels, by contrast, may be used before declaration (02-source-language.tex:35-36). The manual states neither rule.
- resolution: design
- fix: Recommended: document the current behaviour. "A constant must be defined before its first use in the file. Defining the same name again changes its value for the lines that follow." Alternative: make the assembler resolve constants in a second pass and reject redefinition.
- confidence: high
- status: accepted
- decision: Document: a constant must be defined before its first use in the file; redefining a name changes its value for the lines that follow.

### F-asm-11
- file: assembly/chapters/02-source-language.tex
- lines: 35-36
- severity: major
- category: fact
- claim: "A label may be referred to before or after its declaration, and from any object file linked into the same program."
- evidence: Duplicate labels are not diagnosed. Symbols are collected into one list (types/object.rs:59-79), and the linker resolves a reference to the first match (linker.rs:120-130), whether the duplicate is in the same file or a different object. 08-limitations.tex:18 says "Give every label a unique name" but not what happens otherwise.
- resolution: design
- fix: Recommended: make a duplicate label an assembler error (within a file) and a linker error (across files), and add "A label name may be declared only once in a program" here. Minimum alternative: document "If a name is declared more than once, every reference resolves to the first declaration in link order."
- confidence: high
- status: accepted
- decision: A duplicate label will be an error (assembler within a file, linker across files; toolchain todo). The manual says 'A label name may be declared only once in a program'.

### F-asm-13
- file: assembly/chapters/02-source-language.tex
- lines: 58-59
- severity: major
- category: fact
- claim: "for example an immediate operand of a 16-bit instruction is 16 bits wide."
- evidence: Every SIRC instruction is 32 bits (facts.md "Instruction formats"; reference/chapters/07-instruction-formats.tex). The 16-bit field is the immediate format's, and the short immediate format's is 8 bits (03-operands.tex:43-44).
- resolution: manual-wrong
- fix: "for example the immediate operand of an immediate-format instruction is 16 bits wide, and that of a short-immediate (shifted) instruction is 8 bits."
- confidence: high
- status: accepted

### F-asm-9
- file: assembly/chapters/02-source-language.tex
- lines: 71-83
- severity: major
- category: fact
- claim: "A constant stands in for a numeric value in an immediate operand or in a data directive."
- evidence: Ran these cases. (1) `.EQU $N #-1` then `ADDI r1, $N` panics with "Immediate value (resolved from [N] placeholder) can only be up to 16 bits (4294967295 > 0xFFFF)" (object.rs:25-29). `ADDI r1, #-1` assembles, so a negative constant behaves differently from the same literal. (2) A constant is rejected as a shift count (`LSL $S`; instruction.rs:221-249 uses parse_number only). (3) EXCP and ETFR/ETTR reject constants ("does not support symbol refs or placeholders at this time"). Same claim at 05-directives-and-symbols.tex:52-53 and reference/chapters/notation-and-glossary.tex:132-133 ("usable anywhere a #immediate operand is accepted").
- resolution: code-wrong
- fix: Keep the manual's intent. Toolchain todo: resolve placeholders with the same u16 truncation as literals (or accept values in -32768..65535), and accept them in shift counts, EXCP and ETFR/ETTR. If the author prefers to keep the code, add to Table 8.1: "Named constants in shift counts, EXCP, ETFR and ETTR, and negative named constants in instructions".
- confidence: high
- status: simulator

### F-asm-3
- file: assembly/chapters/03-operands.tex
- lines: 69-79
- severity: major
- category: fact
- claim: "A register-form ALU instruction with two register operands is shorthand for the three-operand form with the destination repeated as the first source."
- evidence: Known from other reviewers: review/facts-alu.md F-alu-1 (line 20-22) and review/facts-enc.md:96-98. The assembler accepts `CMPR r1, r2, r3` / `TSAR ...` three-register forms (arithmetic_register.rs:122-141,167-190) that put an ignored register in R1. Neither manual documents them, and this section does not say how the test mnemonics (CMNR..TSXR), which have no destination, fit the rule.
- resolution: design
- fix: Follow the option chosen for F-alu-1. Recommended: the toolchain rejects three-register test forms, and this section adds "The test instructions (CMNR, CNCR, CMPR, CPCR, TSAR, TSOR, TSXR) have no destination and take exactly two registers, `rS1, rS2[, shift]`."
- confidence: high
- status: merged into F-alu-1

### F-asm-4
- file: assembly/chapters/03-operands.tex
- lines: 4-6, 10-32
- severity: major
- category: fact
- claim: "This chapter lists every operand form the assembler accepts."
- evidence: The zero-displacement shorthand `(addr)`, `(addr)+` and `-(addr)` is accepted (sirc-vm/toolchain/src/parsers/instruction.rs:150-160; `LOAD r1, (a)+`, `STOR -(s), r1` and `LJSR (a)+` all assemble). The architecture manual defines it normatively (reference/chapters/08-addressing-modes.tex:105-132; 14-memory-instructions.tex:71-74; 15-control-flow.tex:64-70, including "LJMP (src) and LJSR (src) are not public syntax"). Table 3.1 omits all three forms.
- resolution: manual-wrong
- fix: Add a paragraph after Table 3.1: "An immediate displacement of zero may be omitted: `(addr)`, `(addr)+` and `-(addr)` are the same as `(#0, addr)`, `(#0, addr)+` and `-(#0, addr)`. The shorthand is accepted only where the full form is, and `LJMP`/`LJSR` take `addr` rather than `(addr)`. Register displacements have no shorthand."
- confidence: high
- status: accepted

### F-asm-5
- file: assembly/chapters/03-operands.tex
- lines: 38-41
- severity: major
- category: fact
- claim: "An immediate operand may be a number, a label reference, or a named constant."
- evidence: ALU immediates (ADDI..TSXI), COPI, EXCP, ETFR/ETTR and the DMA/maths meta-instructions reject label references: arithmetic_immediate.rs:206-214,270-278 ("The [ADDI] opcode does not support symbol refs at this time"); facts.md "Default symbol-reference types". Ran `ADDI r1, @x`: rejected. The linker can patch only opcodes 0x07, 0x10, 0x14, 0x18, 0x1A, 0x1C, 0x1E (sirc-vm/toolchain/src/bin/linker.rs:170-177). Ch.5 Table 5.2 (05-directives-and-symbols.tex:76-78) lists only MOVI, LOAD, STOR, LDEA, LDEL, LJMP, LJSR, BRAN, BRSR, which hints at the restriction but never states it. reference/chapters/notation-and-glossary.tex:97 makes the same general claim.
- resolution: design
- fix: Option (a), recommended: document the limitation. Ch.3: "A label reference is accepted only by MOVI, LOAD, STOR, LDEA, LDEL, LJMP, LJSR, BRAN and BRSR; other instructions with an immediate operand accept a number or a named constant only." Add a row to Table 8.1 ("Label references in ALU and coprocessor immediates: load the address with MOVI first"). Option (b): extend the assembler and linker to patch every immediate-format opcode.
- confidence: high
- status: accepted
- decision: Document the limitation (recommended option a).

### F-asm-6
- file: assembly/chapters/03-operands.tex
- lines: 34-36
- severity: major
- category: fact
- claim: "The offset in parentheses may be a number, a label reference or a named constant."
- evidence: The parser accepts `@sym` in `LOAD rD, (@sym, A)+` (opcode 0x16) and `STOR -(@sym, A), rS` (0x12) (load.rs:164-177; store.rs:181-194; both assemble here). The linker has no patch arm for them and panics with "Can't patch address/offset for instruction" (linker.rs:170-177,203-208). facts.md Open question 3. The same gap makes 07-diagnostics.tex:50 "The linker reports two problems" incomplete, because this panic can be reached from source the assembler accepts.
- resolution: code-wrong
- fix: Keep the manual. Toolchain todo: add LoadRegisterFromIndirectImmediatePostIncrement (0x16) and StoreRegisterToIndirectImmediatePreDecrement (0x12) to the linker's patchable list. Until that lands, either note the gap in Table 8.1 or reject `@sym` in those two forms at assemble time.
- confidence: high
- status: simulator

### F-asm-7
- file: assembly/chapters/05-directives-and-symbols.tex
- lines: 7-10
- severity: major
- category: fact
- claim: "\texttt{.ORG addr} sets the word address at which the following instructions and data are assembled."
- evidence: .ORG sets an offset within the current object only (sirc-vm/toolchain/src/data/object.rs:177-180). The linker then shifts every object by the total size of the objects before it (sirc-vm/toolchain/src/types/object.rs:53-80). Ran two objects: m1 (.ORG 0, .DQ, .ORG 0x10, one instruction; 36 bytes) and m2 (`.ORG 0x0040` then `NOOP`). The NOOP lands at byte 0xA4 (word 0x52), not word 0x40. So .ORG is absolute only in the first object, and "put code or data at fixed addresses such as the reset and exception vector table" works only there. 06-toolchain.tex:56 and 08-limitations.tex:17 ("Place code and data with .ORG") depend on the same rule.
- resolution: design
- fix: Option (a), recommended for now: state the rule. Add to Section 5.1: "In a program linked from several objects, .ORG is relative to the start of its own object, and each object starts where the previous one ends. Put the vector table and any other fixed-address code in the first object." Option (b): make the linker place objects at absolute .ORG addresses and diagnose overlaps.
- confidence: high
- status: accepted
- decision: Document the current per-object .ORG rule (recommended option a).

### F-asm-8
- file: assembly/chapters/05-directives-and-symbols.tex
- lines: 46-48
- severity: major
- category: fact
- claim: "Each may also take a label reference or a named constant instead of a number."
- evidence: For a label reference the linker ignores the directive's size and writes the full 32-bit word address into all 4 bytes for .DB, .DW and .DQ alike (sirc-vm/toolchain/src/bin/linker.rs:64-84,150-158; object.rs:81-89 records no size). A named constant, by contrast, is truncated to the directive's size (object.rs:90-105). reference/chapters/notation-and-glossary.tex:130-131 states this: "A label reference always fills the whole slot". Table 5.1 implies that `.DB @label` puts the value in the low byte.
- resolution: manual-wrong
- fix: Replace the sentence with: "Each may also take a named constant, which is placed and truncated like a number, or a label reference. A label reference with no suffix always fills the whole slot with the label's address, whichever of the three directives is used; with .l, .u or .r the 16-bit value goes in the low word."
- confidence: high
- status: accepted

### F-asm-12
- file: assembly/chapters/06-toolchain.tex
- lines: 85-87
- severity: major
- category: fact
- claim: "With a file, the segment is initialised from it."
- evidence: The file is memory-mapped read-write (sirc-vm/device-ram/src/lib.rs:61-69, `OpenOptions::new().read(true).write(true)`, `map_mut`), and segments from --segment are writable by default (sirc-vm/sirc-vm/src/main.rs:62-83). Every STOR into the segment therefore changes the file on disk. The file must already exist and be writable, or the VM panics (`.unwrap()` at lib.rs:68-69).
- resolution: manual-wrong
- fix: "With a file, the segment is mapped onto it: the program reads the file's contents, and every store to the segment writes through to the file. The file must exist and be writable."
- confidence: high
- status: accepted

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
- status: accepted

### F-hw-10
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 201
- severity: major
- category: fact
- claim: "The interrupt lines are still sampled and a Level 5 edge is still latched, but nothing is dispatched until the CPU resumes."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:301-318 (the halt and stalled-bus returns happen before `raise_hardware_interrupt`, so no line is sampled while halted or during a wait state); facts.md Open question 7. general-todo.md 17.
- resolution: code-wrong
- fix: Manual stays (also Ch.4 line 98). Simulator todo 17; note in that todo that it covers the level-sensitive lines while halted, not only the L5 edge.
- confidence: high
- status: simulator

### F-hw-11
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 232-233
- severity: major
- category: fact
- claim: "and a line is not latched while its own handler or a higher-priority one is running."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:584-595 (every enabled, asserted line 1-4 is ORed into the pending register on every poll, with no level check). general-todo.md 18.
- resolution: code-wrong
- fix: Manual stays (also Ch.4 line 96). Simulator todo 18.
- confidence: high
- status: simulator

### F-hw-12
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 250-251
- severity: major
- category: fact
- claim: "If this pin is asserted while another Level 5 exception is being handled, the CPU raises a Level Five Hardware Exception Conflict fault."
- evidence: hw Ch.3 lines 247-248 and 253-254 (edge-triggered; holding the pin does not fire again); reference/chapters/06-exceptions.tex:506-515 (a second edge while the L5 pending bit is set sets the overrun flag and raises the conflict fault when L5 is next selected), :524-526, :534-536
- resolution: manual-wrong
- fix: "A new assertion of this pin (a fresh edge) while a Level 5 handler is running raises a Level Five Hardware Exception Conflict fault; holding the pin asserted from the edge that started the handler does not. A further edge while a Level 5 request is still pending (for example behind a fault handler) is lost and reported by a single conflict fault when Level 5 is next selected." Simulator: the overrun case is general-todo.md 11.
- confidence: high
- status: accepted

### F-hw-13
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 253-254
- severity: major
- category: fact
- claim: "holding the pin asserted after its handler returns does not by itself cause it to fire again -- a fresh falling-then-rising transition is required."
- evidence: hw Ch.3 line 243 (NMI is Active Low); hw Ch.5 lines 7-8 (physical levels); hw Ch.3 lines 247-248 ("fires once on assertion ... must be released and reasserted"). For an active-low pin, release is a rising transition and assertion is a falling one, so the required sequence is rising-then-falling, and the request fires on the falling edge.
- resolution: manual-wrong
- fix: "-- the pin must be released (driven high) for at least one CLKI cycle and asserted (driven low) again; the request fires on the clock edge at which the assertion is first sampled." This also states the minimum pulse width, which arch 06-exceptions.tex:505 implies ("active for at least one clock cycle is never lost") but the hardware manual does not.
- confidence: high
- status: accepted

### F-hw-14
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 246-248
- severity: major
- category: fact
- claim: "NMI is edge-triggered: it fires once on assertion regardless of how long the pin is held, and the pin must be released and reasserted to fire again."
- evidence: reference/chapters/06-exceptions.tex:874-875 ("requests that are still active then are sampled according to the normal interrupt rules"); sirc-vm/peripheral-cpu/src/lib.rs:612-616 (`reset()` clears `l5_interrupt_pin_previously_asserted`, so an NMI held asserted through reset fires once sampling resumes)
- resolution: design
- fix: Neither manual says whether an NMI held asserted across a reset counts as an edge when sampling resumes after the reset-vector fetch. Options: (A) yes, the edge detector is cleared by reset, so a held NMI fires once after reset (simulator behaviour); (B) no, the CPU must see the pin deasserted first. Recommend A (a held NMI after reset is otherwise silently lost) and state it in both the NMI section and the arch Reset State table.
- confidence: medium
- status: accepted
- decision: An NMI held asserted across a reset does not fire afterwards: reset does not clear the edge detector's last sampled level, so there is no transition; the line must be released and reasserted. State it in the NMI section and the architecture Reset State table.

### F-hw-15
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 214-216
- severity: major
- category: fact
- claim: "No trace fault is raised while a fault handler runs (exception level 7), since it would escalate to a double fault."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:389-393 (trace sampled from SR.T or `force_trace_mode` with no exception-level check); reference/chapters/06-exceptions.tex:307,322 agrees with the manual. general-todo.md 19 (last sentence).
- resolution: code-wrong
- fix: Manual stays (also Ch.4 line 99). Simulator todo 19.
- confidence: high
- status: simulator

### F-hw-16
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 199-200
- severity: major
- category: fact
- claim: "If asserted, the CPU completes the current instruction, including the dispatch slot of a coprocessor call, and halts before the next instruction fetch."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:301-313 (HALT is checked at every phase 0 before the CSR/cause is read at :366-380, so the CPU halts between a COPI's processing-unit slot and its exception-unit dispatch slot); reference/chapters/06-exceptions.tex:836-838 agrees with the manual. Not in general-todo.md (related to 12).
- resolution: code-wrong
- fix: Manual stays. New simulator todo: do not take HALT at a phase 0 whose slot dispatches a pending coprocessor command; add a test asserting HALT during a COPI and expecting the dispatch slot to run before the halt.
- confidence: medium
- status: simulator

### F-hw-17
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 199-203
- severity: major
- category: fact
- claim: "HALT is sampled at instruction boundaries. ... A reset ends the halt, but if HALT is still asserted the CPU halts again at the first instruction boundary after reset."
- evidence: reference/chapters/06-exceptions.tex:489-491 (interrupts are recognized only at a boundary when the CSR is empty and no coprocessor is active), :836-838, :864; reference/chapters/16-coprocessor-instructions.tex:736-737 (DMA runs into further whole slots, each with a phase 0); sirc-vm/peripheral-cpu/src/lib.rs:301-313 (halts at any phase 0, including before the reset-vector fetch slot and before a fault dispatch slot)
- resolution: design
- fix: Define "instruction boundary" for HALT in three cases the text leaves open: (1) the phase 0 between DMA continuation slots; (2) the phase 0 that would dispatch a fault raised by the instruction just finished; (3) after reset, whether the CPU halts before the reset-vector fetch or after it, before the first instruction fetch. Recommend: (1) no halt until the DMA command completes, matching interrupts; (2) the fault dispatch slot runs first, as the fault is part of the instruction; (3) the reset-vector fetch completes and the CPU halts before the first instruction fetch, consistent with "halts before the next instruction fetch". Mirror in arch 06-exceptions.tex:836-844 and Ch.4 line 98.
- confidence: medium
- status: accepted
- decision: Same decision as F-exc-14: HALT is sampled at every phase 0, not at instruction boundaries; apply the F-exc-14 wording to the hardware HALT description, covering DMA slots, fault dispatch and reset.

### F-hw-18
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 142-147
- severity: major
- category: fact
- claim: "For example, if a memory controller knows that the DMA coprocessor will be reading more than one sequential address, it can prefetch the next address."
- evidence: reference/chapters/16-coprocessor-instructions.tex:729-746 (phase 0 setup cycle, one access per cycle from phase 1, DMAT read chunk of up to 7 then write chunk, BAT 101/110), :741-743 (wait states); reference/chapters/appendix-b-timing.tex:10 (defers bus signal timing to the hardware manual); hw Ch.5 has no DMA diagram
- resolution: design
- fix: The hardware manual defines BAT 101/110 but never describes a DMA bus cycle. Add a section and a timing figure stating: (a) whether BAS is deasserted between back-to-back DMA accesses or stays asserted with A0-A23 changing each cycle (recommend: BAS is re-asserted per access and every access is a separate request completed by its own BACK, as for any other cycle); (b) that addresses ascend by one per access for both positive and negative counts; (c) BRW and BAT for each half of a DMAT chunk; (d) that PROT follows SR.P during DMA accesses; (e) the setup cycle has no bus access; (f) whether the DMA coprocessor is optional (handover ho:22-23) so BAT 101/110 may never appear. Also check whether "burst" is the right name when each access is an ordinary single-word request.
- confidence: medium
- status: accepted
- decision: Add a DMA bus-cycle section with a timing figure to the hardware manual (Ch.4 or Ch.5; also covers F-con-24): the setup cycle performs no bus access; each DMA access is a separate cycle with BAS asserted and completed by its own BACK; addresses ascend by one per access for positive and negative counts; BRW and BAT per half of a DMAT chunk (read burst 101, write burst 110); PROT follows SR.P; a wait state holds the phase and adds one cycle; SYNC is low in DMA slots. Example: a 3-word DMAR.

### F-hw-2
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 53-54
- severity: major
- category: fact
- claim: "Each transition from low to high advances the CPU's internal six-stage sequence by one stage, so every six clock transitions complete one full instruction execution when no bus wait states occur."
- evidence: reference/chapters/appendix-b-timing.tex:8-9,43-49 (coprocessor calls take 12 cycles, DMA 6 + 6*ceil((k+1)/6), RSET 18, WAIT 12 + idle); reference/chapters/06-exceptions.tex:892-899 (no advance during the reset hold); 06-exceptions.tex:836-839 (no advance while halted)
- resolution: manual-wrong
- fix: "Each rising edge of CLKI advances the CPU by one execution phase, except while a bus wait state holds the phase, while the CPU is halted, waiting (\mnemonic{WAIT}) or held in reset. With no wait states an ordinary instruction takes six cycles; coprocessor calls take longer (see the Instruction Timing appendix of the \archmanual{})."
- confidence: high
- status: accepted

### F-hw-3
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 63-64
- severity: major
- category: fact
- claim: "It remains asserted if a wait state holds that phase, and is deasserted during every other phase."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:286-288 (stall path reports SYNC as `self.phase == InstructionFetchLow` after `advance_phase()` has already moved the phase to 1, so SYNC is low in every wait-stated cycle of phase 0); lib.rs:469-475; facts.md Open question 6. Not in general-todo.md.
- resolution: code-wrong
- fix: Manual stays. New simulator todo: on the stalled-request path, report SYNC from the phase that issued the pending request (phase 0) rather than the advanced phase; add a test with a wait-stated phase-0 fetch expecting SYNC high in every held cycle. Same rule appears in hw Ch.4 line 28.
- confidence: high
- status: simulator

### F-hw-4
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 63-64
- severity: major
- category: fact
- claim: "Asserted while the CPU is in the first execution phase of an instruction or exception-unit dispatch."
- evidence: reference/chapters/16-coprocessor-instructions.tex:729-737 (the DMA slot is a coprocessor slot whose phase 0 is a setup cycle, and the transfer runs into further 6-cycle slots); reference/chapters/06-exceptions.tex:836-839 (halt), 892 (reset hold); hw Ch.4 line 28 (same rule)
- resolution: design
- fix: State SYNC for the cases the rule does not cover: (1) phase 0 of a DMA (coprocessor 2) or maths (coprocessor 3) slot and of each continuation slot of a long DMA transfer; (2) while halted; (3) while in WAIT; (4) during the RSTO hold. Recommend: SYNC is asserted only at phase 0 of a slot that begins an instruction (processing-unit fetch) or an exception-unit dispatch, deasserted in DMA/maths slots and continuation slots, and deasserted while halted, waiting or in reset. The simulator today drives SYNC at every phase 0 it runs and low while halted or waiting (lib.rs:301-322,471-475).
- confidence: medium
- status: accepted
- decision: SYNC is asserted only at phase 0 of an instruction fetch or an exception-unit dispatch slot. It is deasserted in every other coprocessor slot (DMA, maths) and DMA continuation slot, and while halted, waiting or in reset. Drop '(Instruction Fetch, high word)' from the dispatch case. Apply in hardware Ch.3 and Ch.4.

### F-hw-5
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 181-182
- severity: major
- category: fact
- claim: "Follows the status register Protected Mode (P) bit, except that it is deasserted for every exception-vector fetch (BAT 100)"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:236-244 (PROT is SR.P on every poll output, with no BAT 100 exception); reference/chapters/06-exceptions.tex:778-780 agrees with the manual. general-todo.md 22 ("Also deassert PROT for every exception-vector fetch").
- resolution: code-wrong
- fix: Manual stays (also hw Ch.4 line 26). Simulator todo 22.
- confidence: high
- status: simulator

### F-hw-6
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 271-275, 284-287
- severity: major
- category: fact
- claim: "When this pin is deasserted, the CPU completes the reset-output hold, stops asserting the RSTO pin, and the exception unit fetches the reset vector."
- evidence: reference/chapters/06-exceptions.tex:897-903 (assertion cycle counts as the first RSTO cycle, then five more after release; RSET gives six RSTO cycles; the retry hold is six cycles at :923); appendix-b-timing.tex:48,80. The hardware manual never gives the length of the "reset-output hold" in text; only the Ch.5 figure (lines 171-189) implies it.
- resolution: manual-wrong
- fix: Add after line 272: "The reset-output hold is six CLKI cycles: the cycle in which RSTI is first sampled asserted counts as the first, and RSTO stays asserted for five further cycles after RSTI is sampled deasserted, so an RSTI assertion of n cycles gives n + 5 cycles of RSTO. A software reset and a reset-vector fetch retry each give a six-cycle hold." Repeat the figure in the RSTO section and in Ch.5 line 167-169.
- confidence: high
- status: accepted

### F-hw-7
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 284-290
- severity: major
- category: fact
- claim: "It is also asserted for the reset-output hold after a software reset, and for a further hold before the CPU retries a reset-vector fetch that failed."
- evidence: reference/chapters/06-exceptions.tex:365-366,769 ("If the double-fault vector fetch fails, the CPU resets"); sirc-vm/peripheral-cpu/src/lib.rs:272-277,520-528 (triple fault re-asserts the reset request every poll, so the reset unit drives RSTO); facts.md Exceptions "Double and triple faults"
- resolution: design
- fix: The RSTO list omits the triple-fault reset. Options: (A) it behaves as a software reset: six-cycle RSTO hold, then the reset-vector fetch; (B) the CPU holds RSTO asserted until an external RSTI (a lock-up state). The simulator does neither cleanly (it requests reset every poll until `reset()` runs). Recommend A, and add "and for the reset-output hold after a reset caused by a failed double-fault vector fetch" to this paragraph and to the arch reset section.
- confidence: medium
- status: accepted
- decision: A reset caused by a failed double-fault vector fetch behaves as a software reset: six-cycle RSTO hold, then the reset-vector fetch. Add it to the RSTO paragraph and the architecture reset section.

### F-hw-8
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 286-287
- severity: major
- category: fact
- claim: "and for a further hold before the CPU retries a reset-vector fetch that failed."
- evidence: facts.md Exceptions ("A bus error while fetching a hardware, software or reset vector raises an ordinary Bus fault", lib.rs:561-571; faults.rs:471,532); reference/chapters/06-exceptions.tex:921-924 agrees with the manual. general-todo.md 23.
- resolution: code-wrong
- fix: Manual stays. Simulator todo 23.
- confidence: high
- status: simulator

### F-hw-9
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 231
- severity: major
- category: contradiction
- claim: "Interrupt inputs are level-sensitive and are sampled on every clock cycle."
- evidence: reference/chapters/06-exceptions.tex:503-504 (L5 "latched in every CPU state except reset"), :874-875 and :882-884 ("no interrupt line is sampled during the reset hold or the reset-vector fetch"); hw Ch.4 line 96 ("sampled every cycle")
- resolution: manual-wrong
- fix: "Interrupt inputs are sampled on every clock cycle except during reset: from the cycle in which reset is requested until the reset-vector fetch completes, no interrupt line, including NMI, is sampled." Apply the same qualification to Ch.4 line 96 and the NMI row at line 97, and to Ch.5 line 192.
- confidence: high
- status: accepted

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
- status: merged into F-hw-4

### F-hw-19
- file: hardware/chapters/04-bus-interface.tex
- lines: 35-36, 79
- severity: major
- category: fact
- claim: "When BAS is deasserted there is no active bus transaction. External devices must ignore A0--A23, D0--D15, BRW, and BAT0--BAT2 unless BAS is asserted."
- evidence: hw Ch.4 line 79 ("No device drives D0--D15") says nothing about A0-A23, BRW or BAT; hw Ch.5 figures (e.g. lines 19-22, 136) show BRW driven H and BAT "None" (000) while BAS is deasserted; hw Ch.3 line 159 defines 000 as "No I/O"; hw Ch.3 line 205 (HALT for hardware-driven debugging implies another master may need the bus)
- resolution: design
- fix: State what the CPU does with A0-A23, BRW and BAT0-BAT2 while BAS is deasserted, including while halted, waiting or in reset. Options: (A) driven, with BAT = 000 and BRW high, so 000 "No I/O" is the idle value; (B) high-impedance, so another bus master can use the bus while the CPU is halted. Recommend A for this revision (it matches the figures and the simulator's default assertions) and say explicitly that the SIRC-1 has no bus-release mechanism, so HALT does not free the bus for another master.
- confidence: medium
- status: accepted
- decision: While BAS is deasserted, including while halted, waiting or in reset, the CPU still drives A0-A23, BRW (high) and BAT0-BAT2 (000, the idle value). The SIRC-1 has no bus-release mechanism, so HALT does not free the bus for another master.

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
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: accepted
- decision: A COPI/COPR naming coprocessor 0x0 takes 6 cycles and has no dispatch slot. State it in Ch.1 intro, Ch.12 timing paragraph, the Ch.16 COPI/COPR Timing row and common-semantics row, and Appendix B's COPI/COPR row.

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
- status: simulator

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
- status: simulator

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
- status: simulator

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
- status: accepted

### F-data-1
- file: reference/chapters/04-data-representation.tex
- lines: 158-161
- severity: major
- category: fact
- claim: "If \texttt{SR.A} is set and advancing the program counter past an instruction would wrap the low word, that instruction raises a segment-overflow fault before it commits, unless it is a taken branch or jump that does not use the advanced address, or an ALU or \mnemonic{LOAD} instruction that writes \reg{pl}"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:99-108,134-148 (the wrap is recorded at decode in `next_instruction_fetch_is_overflow` and only raised at the *next* instruction's phase 0, after the last-slot instruction has fully committed; nothing clears the flag when write-back writes `pl`, so a taken branch/jump or an ALU/LOAD write to `pl` from the last slot still faults at the target); sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:855-935 (test expects the link register to hold the wrapped address 0x0000, not the instruction's own 0xFFFE). Agrees with manual: reference/chapters/06-exceptions.tex:249-257 and 10-condition-codes.tex:12-13.
- resolution: code-wrong
- fix: Keep the manual. Simulator todo: detect the PC-advance wrap in phase 3 of the same instruction, cancel it and save its own address; suppress the fault for taken branches/jumps that do not save a return address and for ALU/LOAD writes to `pl`; still fault for a skipped (condition-false) instruction.
- confidence: high
- status: simulator

### F-data-2
- file: reference/chapters/04-data-representation.tex
- lines: 152-154
- severity: major
- category: fact
- claim: "For pre-decrement and post-increment forms, the address-register auto-update and the effective-address calculation are checked separately: either one wrapping raises the fault, even if the other brings the result back into range."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:93 (auto-update value `ad_l_ + addr_inc` computed with its overflow discarded), :98-103 and :130-133 (checks only the displacement and `displaced ± 1`). So an auto-update wrap is never trapped (e.g. `-(#5,a)` with al=0x0000: EA 0x0004 fine, al wraps to 0xFFFF, no fault), and post-increment checks `displaced+1`, which is not an architectural value (al=0x0010, disp 0x7FEF -> EA 0xFFFF, al->0x0011, yet the code faults). Matches facts.md Open question 11.
- resolution: code-wrong
- fix: Keep the manual. Simulator todo: check the overflow of `ad_l_ + addr_inc` (the written-back register) and of the EA (`displaced` for post-increment, `displaced - 1` for pre-decrement) separately, and drop the `displaced + 1` check.
- confidence: high
- status: simulator

### F-data-3
- file: reference/chapters/04-data-representation.tex
- lines: 46-48, 87-88
- severity: major
- category: fact
- claim: "They are ignored by address calculation and are carried unchanged when an address pair is copied, loaded or stored. They will be preserved unless a direct write to the high register of an address register pair overwrites them." / "Each exception vector table entry is a stored 32-bit pointer with contains a 24-bit address and eight bits of metadata"
- evidence: sirc-vm/peripheral-cpu/src/registers.rs:289-295,348-356 (`to_full_address` and `to_segmented_address` both mask with ADDRESS_MASK); eu-execution.rs:273-275 (saved return address is `get_full_pc_address()`, masked, so `ph` bits 15-8 are lost), :289 and :382 (vector loaded into `p` through the mask, so vector metadata never reaches `ph`), :363-372 (RETE restores the masked value), :391,:417 (ETFR/ETTR move `a` through the mask). The manual agrees with itself: reference/chapters/06-exceptions.tex:603-604 ("with bits 31--24 carried unchanged from \reg{p}").
- resolution: code-wrong
- fix: Keep the manual. Simulator todo: keep the full 32-bit value for vector loads, link-register return addresses, RETE, and ETFR/ETTR; apply the 24-bit mask only when driving the address bus.
- confidence: medium
- status: simulator

### F-data-4
- file: reference/chapters/05-status-register.tex
- lines: 159-169
- severity: major
- category: fact
- claim: "\mnemonic{RETE}, which sets \texttt{EA} from the restored exception level rather than from the saved status register, so editing bit 13 of a saved status register with \mnemonic{ETTR} has no effect."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:372-378 (RETE copies all 16 saved bits into `sr` and only clears bit 13 when the saved level is 0; when the saved level is non-zero, bit 13 comes from the saved SR); :414-421 (ETTR writes all 16 bits of r7 into the saved SR). A nested handler that clears bit 13 via ETTR therefore returns with EA=0 at a non-zero level.
- resolution: code-wrong
- fix: Keep the manual. Simulator todo: after restoring SR in RETE, set or clear bit 13 according to `saved_exception_level != 0`.
- confidence: high
- status: simulator

### F-data-5
- file: reference/chapters/05-status-register.tex
- lines: 186-187
- severity: major
- category: fact
- claim: "An instruction trace fault (vector \texttt{0x06}) is generated after every instruction, except at exception level 7"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:389-393,450-460 (trace is sampled from SR.T or TRCE and raised with no exception-level check); lib.rs:183-190 (a fault raised at level >= 7 becomes a DoubleFault). So a level-7 handler that sets T, or runs while TRCE is asserted, double-faults. The manual is consistent: reference/chapters/06-exceptions.tex:121,307,321,763.
- resolution: code-wrong
- fix: Keep the manual. Simulator todo: gate `trace_mode_sampled` on `current_exception_level < 7`.
- confidence: high
- status: simulator

### F-data-6
- file: reference/chapters/05-status-register.tex
- lines: 142-145
- severity: major
- category: fact
- claim: "The line is not latched while its own handler or a higher-priority one is running, so a request withdrawn before then is not seen"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:576-596 (`raise_hardware_interrupt` latches every enabled asserted line 1-4 on every poll and never compares against `current_exception_level`). The manual is consistent: reference/chapters/06-exceptions.tex:459-465. In the code, a line still asserted when its handler is entered is re-latched and dispatched again after RETE, which is the failure the Ch. 6 rule exists to prevent.
- resolution: code-wrong
- fix: Keep the manual. Simulator todo: mask out lines 1-4 whose dispatch level (line + 1) is <= `current_exception_level` before latching.
- confidence: high
- status: simulator

### F-data-7
- file: reference/chapters/05-status-register.tex
- lines: 119-123, 139-144
- severity: major
- category: fact
- claim: "\textbf{E1 (Bit 9):} Enable Hardware Interrupt Line 1 (Level 1, lowest maskable priority)" ... "A lower-priority request held during a higher-priority handler is serviced as soon as the exception level drops below its level."
- evidence: reference/chapters/06-exceptions.tex:29-33,640-643 (line n is dispatched at exception level n+1; level 1 is software exceptions); eu/definitions.rs and eu-execution.rs:77-82 (level = 7 - (vector>>4), so line 1 -> level 2). The chapter uses "Level n" for the line number and "exception level" in the next bullets without giving the n+1 mapping. Read literally, "drops below its level" for line 2 means exception level <= 1, which is wrong: a line 2 request must preempt a line 1 handler running at exception level 2.
- resolution: manual-wrong
- fix: After the E1-E4 list, add: "Hardware interrupt line n is dispatched at exception level n+1 (Chapter~\ref{ch:exceptions}); line 5 runs at level 6." Replace "as soon as the exception level drops below its level" with "as soon as the exception level drops to n or lower, where n is the line number".
- confidence: medium
- status: accepted

### F-exc-1
- file: reference/chapters/06-exceptions.tex
- lines: 239-242
- severity: major
- category: fact
- claim: "\texttt{0x1} (instruction fetch) for the target of a branch, \mnemonic{LDEL}, \mnemonic{BRSR}, or \mnemonic{LJSR}, and \texttt{0x0} (none) for \mnemonic{LDEA}, which makes no access with it"
- evidence: reference/chapters/15-control-flow.tex:63 ("LDEL destinations are a, s, or p"); facts.md:173 (BRAN and LJMP are LDEA with destination p). `LDEL a, (#n, s)` makes no fetch with its EA, yet the rule gives it 0x1. `BRAN`/`LJMP` are LDEA, which gets 0x0, yet they are branch targets, which get 0x1. The rule is keyed on the mnemonic, but the real question is whether the destination is p.
- resolution: manual-wrong
- fix: "... \texttt{0x2} (data read) or \texttt{0x3} (data write) for \mnemonic{LOAD} and \mnemonic{STOR}; \texttt{0x1} (instruction fetch) when the effective address is written to \reg{p} (a branch or jump, including \mnemonic{LDEA}/\mnemonic{LDEL} with destination \reg{p}); and \texttt{0x0} (none) when it is written to another address register, which makes no access with it."
- confidence: high
- status: accepted

### F-exc-14
- file: reference/chapters/06-exceptions.tex
- lines: 835-838
- severity: major
- category: fact
- claim: "If it is active there, the CPU stops before the next instruction fetch; an instruction in progress, including its coprocessor dispatch slot, always completes first."
- evidence: none. reference/chapters/06-exceptions.tex:485-494 counts "the command it starts" (a DMA transfer running several slots) as part of the instruction for interrupts, but this sentence and hardware/chapters/03-signal-descriptions.tex:199-200 mention only the dispatch slot.
- resolution: unclear
- fix: Question for the author: does HALT wait for a running DMA (or maths) command to finish, or can it take effect at a phase 0 between DMA slots? State it here and in the hardware HALT description. Using the same wording as the interrupt-latency section ("its dispatch slot, and the command it starts") seems the natural choice.
- confidence: low
- status: accepted
- decision: HALT is sampled at every phase 0 (author's decision). If it is asserted there the CPU stops at that phase 0 with the bus idle; this can pause between an instruction and its coprocessor dispatch slot, between DMA slots, or before a fault, interrupt or reset dispatch, and the CPU resumes exactly where it stopped when HALT is released. Halt latency is at most 6 cycles. Interrupt lines are still sampled and latched while halted. After a reset, if HALT is still asserted the CPU halts at the first phase 0, before the reset-vector fetch. Replace 'an instruction in progress, including its coprocessor dispatch slot, always completes first' and every 'instruction boundary' wording for HALT in both manuals.

### F-exc-2
- file: reference/chapters/06-exceptions.tex
- lines: 313-319
- severity: major
- category: fact
- claim: "When the trace handler returns with \mnemonic{RETE} to exception level 0, the recorded trap is entered at that instruction boundary as if the \mnemonic{EXCP} had just executed"
- evidence: reference/chapters/06-exceptions.tex:25-35 (traps have priority 1, below every interrupt) and :472-473 (the highest-priority pending interrupt is selected at a boundary); :483-484 (interrupts wait while the CSR is non-empty). The manual does not say where the "recorded" trap is held (in the CSR or elsewhere), so it is undefined whether an enabled interrupt that is pending at that boundary goes before or after the trap.
- resolution: design
- fix: Options: (a) the recorded trap sits in the CSR, so it is entered first and pending interrupts wait (this matches the CSR-empty rule and keeps the "as if EXCP had just executed" wording true); (b) it is held as a separate pending trap at priority 1, so pending interrupts go first and LR0 is written only when the trap is finally taken. Recommend (a). Whichever is chosen, say where the trap is held and add it to the Reset State table (Table tab:reset-state, line 856-879), which currently lists no pending software exception.
- confidence: medium
- status: accepted
- decision: The recorded trap of a traced EXCP is held in the CSR, so when the trace handler RETEs to level 0 it is entered before any pending interrupt. Say so here, and add the pending software exception (cleared by reset) to the Reset State table.

### F-exc-21
- file: reference/chapters/06-exceptions.tex
- lines: 662-689
- severity: major
- category: fact
- claim: "\textbf{Bits 8--11:} Original fault type (4 bits) -- for a double fault, the fault that was already being handled, or whose vector could not be fetched ... Reserved and read as zero when the double fault flag is clear"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:530-541 (the vector-fetch double fault stores the fetch's bus fault as `original_fault` and has no Escalating field); facts.md:313 (Original is filled in even when D is clear). todo 14.
- resolution: code-wrong
- fix: No manual change. Implement todo 14.
- confidence: high
- status: simulator

### F-exc-22
- file: reference/chapters/06-exceptions.tex
- lines: 765-767
- severity: major
- category: fact
- claim: "the original exception does not enter and the remaining phases of its dispatch slot run as no-ops"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:514 (`self.phase = 0;` in `handle_vector_fetch_bus_fault`). todo 6.
- resolution: code-wrong
- fix: No manual change. Implement todo 6.
- confidence: high
- status: simulator

### F-exc-23
- file: reference/chapters/06-exceptions.tex
- lines: 771-780
- severity: major
- category: fact
- claim: "Link register 6 stores the state the exception would have saved ... The pending bit of an interrupt, including the Level 5 bit, is set again ... \texttt{PROT} is deasserted for it"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:561-571 (raises an ordinary fault with the current PC; no pending bit is restored); facts.md:446 (PROT always reflects SR.P). todo 22.
- resolution: code-wrong
- fix: No manual change. Implement todo 22.
- confidence: high
- status: simulator

### F-exc-24
- file: reference/chapters/06-exceptions.tex
- lines: 866-873, 921-924
- severity: major
- category: fact
- claim: "Status register & Cleared to \texttt{0x0000} when reset is requested." / "If a bus or protection fault occurs during the reset-vector fetch, the reset target is not loaded and no fault is raised"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:611-625 (`reset()` does not clear SR); eu-execution.rs:380-383 (SR is cleared only when the reset vector is taken); lib.rs:561-571 (a failed reset-vector fetch raises a Bus fault; facts.md:342-343). todo 23.
- resolution: code-wrong
- fix: No manual change. Implement todo 23.
- confidence: high
- status: simulator

### F-exc-25
- file: reference/chapters/06-exceptions.tex
- lines: 514-522
- severity: major
- category: fact
- claim: "the exception unit records the loss in an internal \emph{L5 overrun} flag ... it clears the flag and raises a Level Five Hardware Exception Conflict fault"
- evidence: facts.md:344-351 (no overrun flag; conflict only at level 6). todo 11.
- resolution: code-wrong
- fix: No manual change. Implement todo 11.
- confidence: high
- status: simulator

### F-exc-26
- file: reference/chapters/06-exceptions.tex
- lines: 504-505
- severity: major
- category: fact
- claim: "The edge is latched in every CPU state except reset, including bus wait states, a halt (Section~\ref{sec:halt-request}), and \mnemonic{WAIT}"
- evidence: facts.md:665-667 (Open question 7: the halt return comes before `raise_hardware_interrupt`; the stall path also returns early). The same gap affects 06-exceptions.tex:842-843. todo 17.
- resolution: code-wrong
- fix: No manual change. Implement todo 17.
- confidence: high
- status: simulator

### F-exc-27
- file: reference/chapters/06-exceptions.tex
- lines: 459-463
- severity: major
- category: fact
- claim: "A line is also not latched while the current exception level is at or above the level its own handler would run at"
- evidence: facts.md:346 (lines 1-4 re-latch every poll while held and enabled; there is no level check). todo 18.
- resolution: code-wrong
- fix: No manual change. Implement todo 18.
- confidence: high
- status: simulator

### F-exc-28
- file: reference/chapters/06-exceptions.tex
- lines: 483-490
- severity: major
- category: fact
- claim: "Hardware interrupts, software exceptions and instruction trace faults are recognized only at an instruction boundary, and only when the CSR is empty and no coprocessor is active"
- evidence: facts.md:668-669 (Open question 8: an interrupt dispatch clears a pending COP command); eu-execution.rs:476-477. The same requirement appears at 06-exceptions.tex:983-986. todo 12.
- resolution: code-wrong
- fix: No manual change. Implement todo 12.
- confidence: high
- status: simulator

### F-exc-29
- file: reference/chapters/06-exceptions.tex
- lines: 300-323
- severity: major
- category: fact
- claim: "no trace fault is raised at exception level 7, so the handler itself is not traced even while \texttt{TRCE} is asserted" / "A traced \mnemonic{EXCP} ... the exception unit records the trap as a pending software exception"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:389-393 (trace is sampled from SR.T or `force_trace_mode` with no level check); facts.md:668 (the trace dispatch drops the pending command). todo 19.
- resolution: code-wrong
- fix: No manual change. Implement todo 19.
- confidence: high
- status: simulator

### F-exc-30
- file: reference/chapters/06-exceptions.tex
- lines: 740, 286, 291-292
- severity: major
- category: fact
- claim: "Acceptance is decided before the exception-vector fetch" / "it is cancelled before it writes anything, and the link register stores its own address"
- evidence: facts.md:323 and eu-execution.rs:438-448 (a protected-mode EXCP below 0x60 faults in the EU slot with the already-advanced PC, after the vector fetch); lib.rs:366-377 (the EXCP is discarded inside a handler before any privilege check). todo 13, todo 20.
- resolution: code-wrong
- fix: No manual change. Implement todos 13 and 20.
- confidence: high
- status: simulator

### F-exc-31
- file: reference/chapters/06-exceptions.tex
- lines: 792-793
- severity: major
- category: fact
- claim: "\texttt{SR.EA} is set if the restored exception level is non-zero and cleared if it is 0, whatever the saved status register holds"
- evidence: facts.md:91 (RETE restores all 16 bits, then clears E only when returning to level 0); eu-execution.rs:372-378. todo 21.
- resolution: code-wrong
- fix: No manual change. Implement todo 21.
- confidence: high
- status: simulator

### F-exc-32
- file: reference/chapters/06-exceptions.tex
- lines: 246-256
- severity: major
- category: fact
- claim: "The AU checks this in phase 3, before any memory access ... The instruction is cancelled and the link register stores its own address"
- evidence: facts.md:357 (the PC-wrap fault is raised at the next fetch, `pu-execution.rs:99-107,138-148`). todo 15.
- resolution: code-wrong
- fix: No manual change. Implement todo 15.
- confidence: high
- status: simulator

### F-exc-33
- file: reference/chapters/06-exceptions.tex
- lines: 237-245
- severity: major
- category: fact
- claim: "The fault metadata register records the wrapped address and the bus access it was computed for ... An address-register auto-update that wraps is recorded the same way"
- evidence: facts.md:677-679 (Open question 11: the post-increment check tests the wrong value); execution_effective_address.rs:93-104. todos 16, 31 and 33.
- resolution: code-wrong
- fix: No manual change. Implement todos 16, 31 and 33.
- confidence: high
- status: simulator

### F-exc-34
- file: reference/chapters/06-exceptions.tex
- lines: 217-220
- severity: major
- category: fact
- claim: "The check is made at phase 0, before the fetch's bus cycle, so the odd address never reaches the bus. No instruction slot is used"
- evidence: facts.md:359-360 (alignment is checked on every processing-unit phase while `pl` is odd; `pu-execution.rs:86-93`). todo 28.
- resolution: code-wrong
- fix: No manual change. Implement todo 28.
- confidence: medium
- status: simulator

### F-exc-35
- file: reference/chapters/06-exceptions.tex
- lines: 950
- severity: major
- category: fact
- claim: "Exception coprocessor opcodes \texttt{0x2}--\texttt{0x8} are reserved; issuing one raises an invalid-opcode fault."
- evidence: facts.md:389 (reserved operations 2-8 are a no-op; eu-execution.rs:67-85,543-561). The same requirement appears at 06-exceptions.tex:266-267. todo 26.
- resolution: code-wrong
- fix: No manual change. Implement todo 26.
- confidence: high
- status: simulator

### F-enc-6
- file: reference/chapters/07-instruction-formats.tex
- lines: 98, 158, 228, 485, 513, 542
- severity: major
- category: fact
- claim: "11 = Reserved for future use"
- evidence: No chapter says what the CPU does when an ALU or COP opcode is executed with AF = 11 (reference/chapters/11-reading-instructions.tex:137 and appendix-c-undocumented.tex:113 only say "reserved"). Code leaves SR unchanged, i.e. behaves as [N] (sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:154; facts.md "Update source None (00), and the reserved value 11, leave SR unchanged").
- resolution: design
- fix: Options: (a) state that the effect on SR bits 7-0 is architecturally undefined and assemblers never emit it (parallels the reserved shift type, 09:497); (b) define it as identical to [N] (matches the simulator); (c) make it an invalid-opcode fault. Recommend (a), with one sentence in Ch. 7 under the AF description and a pointer from Appendix C, so an emulator author knows nothing is guaranteed.
- confidence: medium
- status: merged into F-alu-4

### F-enc-4
- file: reference/chapters/08-addressing-modes.tex
- lines: 431-433
- severity: major
- category: fact
- claim: "the auto-update forms must not name \reg{p} as the address-register pair ... Such an encoding is architecturally undefined and the assembler rejects it"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/load.rs:144,201 (rejects only a destination that aliases the pair; `LOAD r1, (#0, p)+` is accepted); only LJSR rejects p (ljsr.rs:161-162; control_flow_test.rs:302); general-todo.md item 27
- resolution: code-wrong
- fix: Keep the manual; tracked as general-todo 27 (also covers STOR -(#n, p), LDEA -(#n, p), LDEL (#n, p)+).
- confidence: high
- status: simulator

### F-enc-5
- file: reference/chapters/08-addressing-modes.tex
- lines: 91
- severity: major
- category: contradiction
- claim: "\texttt{OPR rD, rS1, rS2[, shift]}; register shorthand \texttt{OPR rD, rS2}."
- evidence: reference/chapters/09-shift-operations.tex:67 uses `ADDR r1, r2, LSL #1` (shorthand with a shift); sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:142-165 accepts `[reg, reg, shift]`. The table claims to list "exactly which operand forms are legal" (08:79-82).
- resolution: manual-wrong
- fix: "\texttt{OPR rD, rS1, rS2[, shift]}; register shorthand \texttt{OPR rD, rS2[, shift]} (encoded with R2 = rD, so the shift applies to rD)."
- confidence: high
- status: accepted

### F-enc-2
- file: reference/chapters/09-shift-operations.tex
- lines: 357, 334-336
- severity: major
- category: fact
- claim: "A shift count of 0 leaves the operand unchanged and clears the carry flag." / "a count of 16 behaves as a count of 0"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:485-515 (RTL sets C = result bit 0 and RTR sets C = result bit 15 for every count, so `RTL #0` or a register count of 16 on 0x0001 sets C=1; LSL/LSR/ASL/ASR do give C=0 at count 0). No test exercises RTL/RTR at all (grep of peripheral-cpu/tests for RotateLeft/RotateRight finds nothing). Not covered by general-todo.md (29 covers ASL/ASR only); related to facts.md Open question 14.
- resolution: code-wrong
- fix: Keep the manual. New simulator todo: for RTL/RTR, when the effective count (immediate, or register value mod 16) is 0, clear C instead of copying bit 0/bit 15; add tests for RTL/RTR with counts 0, 1, 15, and register counts 16 and 17.
- confidence: high
- status: simulator

### F-enc-3
- file: reference/chapters/09-shift-operations.tex
- lines: 374-376
- severity: major
- category: fact
- claim: "For \textbf{ASL}: Set if bit 15 changes at any point during the shift (signed overflow); that is, for a count $n$ from 1 to 15, set unless the original bits 15 down to $15 - n$ are all equal."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:417-443 (V from `set_alu_bits(..., Some((a, a, result)))`, which compares only the final sign with the original sign); general-todo.md item 29
- resolution: code-wrong
- fix: Keep the manual; tracked as general-todo 29. The same rule is restated at 09-shift-operations.tex:144 and :160 ("sets the overflow flag if the sign bit changes"); those are consistent with it.
- confidence: high
- status: simulator

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
- status: accepted

### F-sum-2
- file: reference/chapters/12-instruction-summary.tex
- lines: 185-186
- severity: major
- category: contradiction
- claim: "Coprocessor calls ... take a second 6-cycle slot for the coprocessor dispatch"
- evidence: reference/chapters/16-coprocessor-instructions.tex:211-214 (a command with coprocessor ID 0x0 is a no-op and "The CSR is 0x0000 again by the next instruction boundary, so the call never delays exception selection", which implies no dispatch slot); 16-coprocessor-instructions.tex:55 only states the extra slot for coprocessor 1; appendix-b-timing.tex:46 gives COPI/COPR = 12 unconditionally
- resolution: design
- fix: Decide whether a COPI/COPR naming coprocessor 0x0 takes 6 or 12 cycles. Options: (a) 6 cycles, no dispatch slot, consistent with "never delays exception selection"; (b) 12 cycles, uniform with every other call. Recommend (a), and state it in Ch. 12, Appendix B's COPI/COPR row and the coprocessor chapter's Timing row.
- confidence: medium
- status: merged into F-arch-7

### F-alu-1
- file: reference/chapters/13-alu-instructions.tex
- lines: 93-111, 1385-1386, 1404-1405, 1520-1521, 1652-1653, 1671-1672, 1792-1793, 1924-1925, 2060-2061, 2188-2189
- severity: major
- category: contradiction
- claim: "ignored (R1, bits 25--22), register form only: no destination register is written, so the CPU ignores this field. The assembler encodes a copy of \texttt{rS1}."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:120-140,166-189 accept a three-register form for every test mnemonic, and tc/tests/assembler/arithmetic_register_test.rs:34-36 assembles `CMPR ll, r7, r1`, `TSAR ah, r6, r2` and `TSXR al, r5, r3`. That form puts the first register in R1, which the CPU ignores, so `CMPR r1, r2, r3` silently compares r2 with r3. The chapter lists only `CMPR rS1, rS2[, shift]`, and assembly/chapters/03-operands.tex:69-80 does not cover the test mnemonics. Also reported as a toolchain gap in review/facts-enc.md:96.
- resolution: design
- fix: Option (a), recommended: the assembler rejects a three-register operand list (with or without a shift) for CMNR/CNCR/CMPR/CPCR/TSAR/TSOR/TSXR, and the assembly manual's Register Shorthand section says that test mnemonics take exactly two registers. The chapter text then stands as written. Option (b): keep the toolchain as it is, add `CMPR rX, rS1, rS2[, shift]` rows to the table, and change "The assembler encodes a copy of \texttt{rS1}" to "In the two-register form the assembler encodes a copy of \texttt{rS1}; in the three-register form it encodes the first register written, which has no effect."
- confidence: high
- status: accepted
- decision: The assembler will reject three-register test forms (toolchain todo). The Ch.13 text stands; assembly Ch.3 adds that CMNR/CNCR/CMPR/CPCR/TSAR/TSOR/TSXR have no destination and take exactly two registers, rS1, rS2[, shift].

### F-alu-3
- file: reference/chapters/13-alu-instructions.tex
- lines: 152-153
- severity: major
- category: fact
- claim: "An ALU instruction whose register field names \reg{sr} is permitted in protected mode, but only bits 7--0 of \reg{sr} are written; bits 15--8 are left unchanged"
- evidence: reference/chapters/05-status-register.tex (sec:sr-manual-updates) says that an ALU instruction targeting sr writes its result and then overwrites bits 7-0 with its own flags unless `[N]` is used. Code agrees (write_back.rs:186-189, `set_register_value` then `update_status_flags`). In protected mode, then, `ORRI sr, #3` with the default `[A]` changes nothing except the flags. The ALU chapter never states this ordering, and the sentence quoted above suggests the low byte can be written. Related: general-todo.md item 2; facts.md Open question 5.
- resolution: manual-wrong
- fix: After the quoted sentence, add: "With the \texttt{[A]} or \texttt{[S]} status source, the instruction's own flag update then overwrites bits 7--0, so the written value survives only in bits 15--8 (supervisor mode) or not at all (protected mode). Use \texttt{[N]} to write the low byte (Section~\ref{sec:sr-manual-updates})."
- confidence: high
- status: accepted

### F-alu-4
- file: reference/chapters/13-alu-instructions.tex
- lines: 247, 406-407 (and the identical SUS bullet in every instruction box)
- severity: major
- category: fact
- claim: "AF encoding \texttt{11} (binary) is reserved and has no assembly syntax."
- evidence: The chapter defines no effect for AF = 11 on an ALU opcode. reference/chapters/07-instruction-formats.tex:98,158,228 and 11-reading-instructions.tex:136 say only "Reserved for future use". appendix-c-undocumented.tex:113 lists it as a possible future flag source. The code leaves SR unchanged (write_back.rs:154, `_ => registers.sr`). By contrast, the chapter does give a rule (`U U U U`) for MOVI/MOVR with AF = 10 (lines 203-204, 1258-1260).
- resolution: design
- fix: Option (a), recommended: state in the Status Flag Updates section that "An ALU encoding with AF \texttt{11} leaves N, Z, C and V architecturally undefined (\texttt{U U U U}); the result write is unaffected." This keeps AF = 11 free for a future source and matches the MOVI/MOVR AF = 10 rule. Option (b): define AF = 11 as "no update", the same as `[N]`, which matches the simulator but uses up the encoding. In either case, also state that the register result is still written.
- confidence: medium
- status: accepted
- decision: AF = 11: the operation completes and writes its result; N, Z, C and V are architecturally undefined (U U U U); assemblers never emit it. State in Ch.7 under the AF description and in Ch.13 Status Flag Updates, with a pointer from Appendix C.

### F-alu-5
- file: reference/chapters/13-alu-instructions.tex
- lines: 208-211, 219-220
- severity: major
- category: fact
- claim: "in terms of the sign bit (bit 15) of the destination operand (\texttt{Dm}), the source operand (\texttt{Sm}), and the result (\texttt{Rm})"
- evidence: In the three-operand register form (`SUBR rD, rS1, rS2`) and in compare instructions, the destination register is not an input at all. The operands are the left operand, which is shift(rS1) or shift(Reg) per lines 134-136, and the right operand (rS2 or the immediate). The code computes V from (a, b, result), where a is the post-shift left operand (fetch_and_decode.rs:180-194; alu.rs:81-90,195-206). Read literally, "destination operand" points an implementer at the old value of rD and, for shifted forms, leaves open whether the pre-shift or post-shift value is meant.
- resolution: manual-wrong
- fix: Replace the sentence with: "in terms of the sign bit (bit 15) of the left operand after any shift (\texttt{Dm}), the right operand (\texttt{Sm}: \texttt{rS2}, imm16, or the zero-extended imm8), and the 16-bit result (\texttt{Rm}). For \mnemonic{ADC}/\mnemonic{SBC}, \texttt{Rm} includes the incoming carry." Alternatively, rename the symbols to \texttt{Am}/\texttt{Bm} so they do not suggest the destination register.
- confidence: high
- status: accepted

### F-alu-7
- file: reference/chapters/13-alu-instructions.tex
- lines: 149-150
- severity: major
- category: fact
- claim: "An ALU instruction in the last instruction slot of a segment still raises a segment-overflow fault when \texttt{SR.A} is set, because it advances the program counter, unless it writes \reg{pl}"
- evidence: The manual is consistent with 06-exceptions.tex (sec:segment-overflow-fault). The simulator raises the fault at the next fetch for every instruction whenever `pl + 2` carries, including one that writes pl, and records the new PC rather than the instruction's own address (pu-execution.rs:99-108,134-148; fetch_and_decode.rs:204). This is general-todo.md item 15, and is also noted in review/facts-arch.md:33.
- resolution: code-wrong
- fix: No manual change. Simulator todo 15 covers this; add a test case for an ALU instruction at `pl = 0xFFFE` that writes `pl` (no fault) and one that writes `r1` (fault, LR6 = 0xFFFE).
- confidence: high
- status: simulator

### F-mem-4
- file: reference/chapters/14-memory-instructions.tex
- lines: 104-108
- severity: major
- category: contradiction
- claim: "effective-address calculation can raise a segment-overflow fault when \texttt{SR.A} (Trap on Address Overflow) is set and the low-word address calculation wraps"
- evidence: docs/reference/chapters/04-data-representation.tex:152-154 and 06-exceptions.tex:243-245 say the post-increment/pre-decrement auto-update is checked separately and a wrap there also faults. Ch14 (here and at 256-258, 414-416) and Ch15 (98, 108-110, 479, 610) name only the effective-address calculation. Code currently checks the wrong quantities (general-todo.md item 16).
- resolution: manual-wrong
- fix: After "wraps" add: "For post-increment and pre-decrement forms the address-register auto-update is checked separately, and a wrap of either the effective address or the updated low word raises the fault (Section~\ref{sec:arithmetic-and-address-wraparound})." Add the same clause to the LOAD/STOR Exceptions paragraphs and to the Ch15 Exceptions row, the paragraph at 107-110 and the LDEA/LDEL Exceptions paragraphs.
- confidence: high
- status: accepted

### F-mem-5
- file: reference/chapters/14-memory-instructions.tex
- lines: 104-117
- severity: major
- category: fact
- claim: "Memory instructions can raise data bus and data bus-protection faults during the memory-access phase. ... effective-address calculation can raise a segment-overflow fault"
- evidence: docs/reference/chapters/06-exceptions.tex:246-256 and 04-data-representation.tex:157-160: with SR.A set, any instruction in the last slot of a segment that uses the advanced PC faults. A LOAD or STOR is such an instruction unless the LOAD writes pl. Ch15:110-113 states this for control flow, but Ch14 never does.
- resolution: manual-wrong
- fix: Add after the segment-overflow sentence: "With \texttt{SR.A} set, a memory instruction in the last instruction slot of a segment also raises a segment-overflow fault, in phase 3 and before its memory access, because it uses the wrapped next-instruction address; a \mnemonic{LOAD} whose destination is \reg{pl} is the exception (Section~\ref{sec:segment-overflow-fault})."
- confidence: high
- status: accepted

### F-mem-6
- file: reference/chapters/14-memory-instructions.tex
- lines: 94, 121-122
- severity: major
- category: contradiction
- claim: "Memory instructions do not update status flags. All four condition flags are preserved regardless of addressing form, shift result, or auto-update side effect."
- evidence: 14-memory-instructions.tex:111 and 269-270 say "LOAD into sr updates only bits 7--0" in protected mode, and bits 3-0 are the flags; write_back.rs:300-307 -> set_register_value (write_back.rs:42-59) writes sr directly.
- resolution: manual-wrong
- fix: Replace with "Memory instructions do not update status flags as a side effect. A \mnemonic{LOAD} whose destination is \reg{sr} writes the loaded value to the status register (bits 7--0 only in protected mode; every bit except SR.E in supervisor mode), which replaces the flags." Qualify table row 94 and Table tab:memory-status-flag-effects in the same way.
- confidence: high
- status: accepted

### F-mem-7
- file: reference/chapters/14-memory-instructions.tex
- lines: 227-228
- severity: major
- category: fact
- claim: "\textbf{Operands:} \texttt{rD} is the destination general-purpose register; a privileged high address register (\reg{ah}, \reg{lh}, \reg{ph}, or \reg{sh}) raises a privilege-violation fault in protected mode."
- evidence: The rD field is 4 bits and names any of the 16 registers (facts.md:28-35). 06-exceptions.tex:250,256 relies on "a LOAD instruction that writes pl" acting as a jump. write_back.rs:300-307 writes pl after decode has advanced it, so the loaded value wins. Ch14 never describes LOAD into pl or ph, or the odd-pl alignment consequence.
- resolution: manual-wrong
- fix: Replace "general-purpose register" with "any register, including \reg{sr}, the address-register halves and \reg{pl}". Add: "A \mnemonic{LOAD} into \reg{pl} is a jump within the current segment: the loaded value replaces the advanced program counter, and an odd value raises an alignment fault at the next fetch. A supervisor-mode \mnemonic{LOAD} into \reg{ph} changes the segment of the next fetch."
- confidence: medium
- status: accepted

### F-mem-1
- file: reference/chapters/15-control-flow.tex
- lines: 590-598
- severity: major
- category: fact
- claim: "if condition:\n    l = address_of_next_instruction\n    dest.high = src.high\n    dest.low  = src.low + displacement"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:226-245 (dest high = decoded.ad_h_, latched at decode; dest low = alu_output, computed in phase 3 before any write-back). docs/reference/chapters/07-instruction-formats.tex:608-610 says register reads use values latched before write-back. The non-post-increment form with src = l is legal (15-control-flow.tex:568-569, 770-771; not in the rejected list at :73-78).
- resolution: manual-wrong
- fix: Reorder the pseudocode so the target is computed from the original source before l is written: "target = src.high : (src.low + displacement); new_src_low = src.low + 1; l = address_of_next_instruction; dest = target; if post_increment: src.low = new_src_low". Add a sentence: "The source is read before the link is written, so LDEL p, (#0, l) (LJSR l) jumps to the old value of l." Apply the same change to the LJSR operation at 795-803 and the syntax comments at 578-583 and 783-788.
- confidence: high
- status: accepted

### F-mem-2
- file: reference/chapters/15-control-flow.tex
- lines: 72-78
- severity: major
- category: fact
- claim: "The assembler rejects exactly these forms: ... and any auto-update of \reg{p} (\texttt{(p)+} or \texttt{-(p)} in any instruction)"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/load.rs:141-149,195-206; store.rs:156-166,212-223,246-255; ldea.rs:133-140,187-196; ldel.rs:135-157 check only register/pair overlap, never p. Only ljsr.rs:160-166,220-226 rejects p. Known gap: general-todo.md item 27.
- resolution: code-wrong
- fix: Keep the manual. Same claim at 14-memory-instructions.tex:78-80 and the src-field notes at 15-control-flow.tex:438, 569. Toolchain todo 27.
- confidence: high
- status: simulator

### F-mem-3
- file: reference/chapters/15-control-flow.tex
- lines: 110-113
- severity: major
- category: fact
- claim: "With \texttt{SR.A} set, an instruction in the last slot of a segment also raises a segment-overflow fault if it uses the wrapped next-instruction address, either as the next program counter (condition false) or as the return address saved by \mnemonic{LDEL}, \mnemonic{BRSR}, or \mnemonic{LJSR}; a taken branch or jump that saves no return address does not"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:99-107,138-148 sets a flag at decode and raises the fault at the next fetch, so it is charged to whatever executes next and is not tied to the link use. Known gap: general-todo.md item 15.
- resolution: code-wrong
- fix: Keep the manual; simulator todo 15.
- confidence: high
- status: simulator

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
- status: accepted
- decision: General-purpose registers, the a/l/s pairs and the link registers are architecturally undefined after any reset. Change the Ch.16 box to 'are not cleared by reset; software must treat their contents as architecturally undefined', consistent with Ch.3 and Ch.6.

### F-cop-1
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 211-214
- severity: major
- category: contradiction
- claim: "The CSR is \texttt{0x0000} again by the next instruction boundary, so the call never delays exception selection."
- evidence: same chapter 16-coprocessor-instructions.tex:159 ("6 cycles for the processing-unit call, plus a further 6-cycle coprocessor dispatch slot (12 cycles in total)", stated for every COPI/COPR); reference/chapters/appendix-b-timing.tex:46 (COPI/COPR = 12); reference/chapters/02-cpu-architecture.tex:110-113 (the CU dispatches the CSR "in a further six-phase slot"; the CSR is 0x0000 only "while nothing is pending"). Code: an ID-0 command just runs the processing unit (cpu/src/pu/execution.rs:65; facts.md:377), i.e. no extra slot. A protected-mode `COPR sr` always builds an ID-0 command (16:131-132), so this case is reachable by user code.
- resolution: design
- fix: Decide whether an ID-0 command (any value 0x0000-0x0FFF, e.g. `COPI #0x0123`) costs a dispatch slot. Option A (recommended, matches the sim and the "never delays exception selection" sentence): ID-0 commands take 6 cycles; add "(6 for a command with coprocessor ID 0x0)" to 16:159, 16:55 and appendix-b:46. Option B: every non-zero CSR value takes a 6-cycle slot (12 cycles), and replace 16:213-214 with "The dispatch slot leaves the CSR at 0x0000; no operation is performed."
- confidence: high
- status: merged into F-arch-7

### F-cop-2
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 55
- severity: major
- category: contradiction
- claim: "Dispatch to coprocessor 1 then occupies one further complete 6-cycle slot beginning at the next phase 0, so a coprocessor-1 operation completes 12 cycles after the call's own fetch began."
- evidence: 16-coprocessor-instructions.tex:564-565 (RSET: 6 call + 6 RSTO hold + 6 vector fetch = 18, with no dispatch slot before the hold); 06-exceptions.tex:900-903 (RSTO is asserted when RSET's write-back commits); appendix-b-timing.tex:48. WAIT also does not complete in 12 (appendix-b-timing.tex:49).
- resolution: manual-wrong
- fix: Replace the end of the sentence with "...so most coprocessor-1 operations complete 12 cycles after the call's own fetch began. \mnemonic{RSET} instead goes straight from its write-back into the 6-cycle reset-output hold (18 cycles to the reset handler), and \mnemonic{WAIT} then idles until it is woken." Apply the same qualification to the COPI/COPR Timing line (16:159).
- confidence: high
- status: accepted

### F-cop-21
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 216-217
- severity: major
- category: fact
- claim: "Operation nibbles 0x2--0x8 are reserved and raise an invalid-opcode fault."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:67-73 maps reserved nibbles to `None` (no-op) with a comment calling them "architecturally undefined"; the sim also uses reserved 0x14FF as its exit hook (pu-execution.rs:170-177). general-todo.md items 26 and 25.
- resolution: code-wrong
- fix: Manual stays. Sim todo 26 (and 25 for the 0x14FF hook); also update the stale comment at execution.rs:67-71.
- confidence: high
- status: simulator

### F-cop-22
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 57
- severity: major
- category: fact
- claim: "A command waiting in the CSR is never discarded by an interrupt or an instruction trace fault"
- evidence: eu-execution.rs:140-219 selects a fault or eligible interrupt before `pending_coprocessor_command`, and EU write-back clears it (eu-execution.rs:476-477; facts.md Open question 8). general-todo.md item 12.
- resolution: code-wrong
- fix: Manual stays. Sim todo 12.
- confidence: high
- status: simulator

### F-cop-23
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 342-343
- severity: major
- category: fact
- claim: "The privilege check comes first, so a protected-mode \mnemonic{EXCP} with a vector below 0x60 inside a handler still raises a privilege-violation fault."
- evidence: cpu/src/lib.rs:371-377 discards a software exception at phase 0 when the level is non-zero, before the exception unit's privilege check (eu-execution.rs:438-448). general-todo.md item 20.
- resolution: code-wrong
- fix: Manual stays. Sim todo 20.
- confidence: high
- status: simulator

### F-cop-24
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 323-325
- severity: major
- category: fact
- claim: "a vector below 0x60 raises a privilege-violation fault in protected mode."
- evidence: 06-exceptions.tex:288-291 (privilege violation is retryable, link register stores the faulting instruction's own address). The sim raises it in the EU slot with the already-advanced PC and after fetching the vector (eu-execution.rs:438-448; facts.md:323). general-todo.md item 13.
- resolution: code-wrong
- fix: Manual stays. Sim todo 13.
- confidence: high
- status: simulator

### F-cop-25
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 403-409
- severity: major
- category: fact
- claim: "It leaves the wait state only when a hardware interrupt can be taken -- one that is enabled (Level 5 is always enabled) and whose priority is higher than the current exception level"
- evidence: cpu/src/lib.rs:315-322,594,603: any enabled request clears WAIT regardless of the exception level, and an already-pending request does not end the wait. general-todo.md item 24.
- resolution: code-wrong
- fix: Manual stays. Sim todo 24.
- confidence: high
- status: simulator

### F-cop-26
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 478-481
- severity: major
- category: fact
- claim: "\texttt{SR.EA} is not taken from the saved status register: it is set if the restored exception level is non-zero and cleared if it is zero"
- evidence: eu-execution.rs:372-378 restores all 16 bits of the saved SR and only clears E when returning to level 0, so a restored level >0 with EA=0 in an ETTR-edited saved SR leaves EA clear. general-todo.md item 21.
- resolution: code-wrong
- fix: Manual stays. Sim todo 21.
- confidence: high
- status: simulator

### F-cop-27
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 550-551
- severity: major
- category: fact
- claim: "The status register, the exception level, and the pending hardware exceptions are cleared when reset is requested."
- evidence: cpu/src/lib.rs:612-625 and eu-execution.rs:380-383: SR is set to 0 in the reset-vector dispatch (`ExceptionUnitOpCodes::Reset`), not when reset is requested. general-todo.md item 23.
- resolution: code-wrong
- fix: Manual stays. Sim todo 23.
- confidence: medium
- status: simulator

### F-cop-28
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 70-71
- severity: major
- category: fact
- claim: "A traced \mnemonic{EXCP} is the exception: its trace fault is taken first and the trap is entered when the trace handler returns"
- evidence: The sim raises the trace fault at EXCP write-back and the fault dispatch clears the pending command, so the trap is lost (facts.md Open question 8; eu-execution.rs:476-477). general-todo.md item 19.
- resolution: code-wrong
- fix: Manual stays. Sim todo 19.
- confidence: high
- status: simulator

### F-cop-29
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 65-67
- severity: major
- category: fact
- claim: "A coprocessor call in the last instruction slot of a segment still raises a segment-overflow fault when \texttt{SR.A} is set, because it advances the program counter; the call is cancelled before its command is dispatched"
- evidence: The sim sets a flag at decode and raises the fault at the next fetch (cpu/src/pu/execution.rs:99-107,138-148), after the call has written the CSR, so the command is dispatched first. general-todo.md item 15.
- resolution: code-wrong
- fix: Manual stays. Sim todo 15.
- confidence: medium
- status: simulator

### F-cop-4
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 1428-1431
- severity: major
- category: fact
- claim: "else if quotient does not fit in 16 bits:"
- evidence: 16-coprocessor-instructions.tex:1461-1463 (DIVS: two's-complement dividend and divisor, truncation toward zero) never states the DIVS quotient range; "fits in 16 bits" is ambiguous for DIVS (e.g. 40000 / 1 fits an unsigned 16-bit field but not a signed one). The only overflow example given is -2147483648 / -1.
- resolution: manual-wrong
- fix: "else if quotient is outside 0..0xFFFF (DIVU) or -32768..32767 (DIVS):", and add to the Signed division paragraph: "The quotient must lie in -32768..32767; any other quotient, including -2147483648 / -1, is quotient overflow. The remainder lies in -32768..32767 and is written to \reg{r1} as a 16-bit two's-complement value."
- confidence: medium
- status: accepted

### F-cop-6
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 768-772
- severity: major
- category: fact
- claim: "If it carries out of the low word with \texttt{SR.A} set, the fault is raised at that point, after the words have been transferred and with the register unchanged."
- evidence: For DMAT the advance updates two registers (16:1161-1162). The text does not say whether, when only one of `a`/`l` wraps, the other one has already advanced, nor which is checked first (which decides the recorded BAT/address, 16:777-778). 16:1182-1183 says both stay "at the last fully completed chunk boundary", but the faulting chunk was fully read and written, so either reading fits.
- resolution: design
- fix: Recommended: "For \mnemonic{DMAT}, both advances are checked before either is applied: if either would carry, neither \reg{a} nor \reg{l} is changed, and the fault records the \reg{a} advance (DMA read type) if it wraps, otherwise the \reg{l} advance (DMA write type)." Alternative: check and apply `a` first, then `l`, and say so.
- confidence: medium
- status: accepted
- decision: For DMAT, both chunk advances are checked before either is applied: if either would carry with SR.A set, neither a nor l changes; the fault records the a advance (DMA read type) if it wraps, otherwise the l advance (DMA write type).

### F-tim-1
- file: reference/chapters/appendix-b-timing.tex
- lines: 147-153
- severity: major
- category: fact
- claim: "ADDR|HI r3, r4, r5            ; 6 cycles (exec if r1 > r2)" / "; Total: 18 cycles (one conditional executes, one doesn't)"
- evidence: reference/chapters/13-alu-instructions.tex:173 (ADD default AF is `[A]`); facts.md:201-203 (assembler defaults ALU mnemonics to [A]). When r1 > r2 the first ADDR runs and replaces N/Z/C/V with the flags of r4 + r5, so `ADDR|LO` then tests those flags, not the CMPR result. It can execute as well (for example when r4 + r5 carries or is zero). "One conditional executes, one doesn't" therefore does not hold.
- resolution: manual-wrong
- fix: Suppress the flag update on the first conditional: `ADDR[N]|HI r3, r4, r5` (the assembler accepts `[N]` before `|cc`, instruction.rs:198-210). Optionally note "(unsigned)" in both comments, because HI/LO are unsigned conditions.
- confidence: high
- status: accepted

### F-tim-2
- file: reference/chapters/appendix-b-timing.tex
- lines: 8-9, 48
- severity: major
- category: contradiction
- claim: "the dispatched coprocessor occupies at least one further slot after the six-phase call" and "\mnemonic{RSET} & 18 (6 call + 6 \texttt{RSTO} hold + 6 reset-vector fetch)"
- evidence: The RSET row has no dispatch slot: reset is requested when the call's own write-back commits (06-exceptions.tex:901-903; 16-coprocessor-instructions.tex:564-565). The simulator agrees: it asserts RSTI at the phase-0 wrap after the PU write-back (sirc-vm/peripheral-cpu/src/lib.rs:476-486). But 01-introduction.tex:83-85 and 12-instruction-summary.tex:185-187 say every coprocessor call "and every meta-instruction that assembles to one" takes a second 6-cycle dispatch slot. Ch. 16:556 also says RSET "Inherits COPI command dispatch to the exception unit". An implementer who follows Ch. 1/12 would build 6 + 6 + 6 + 6 = 24.
- resolution: manual-wrong
- fix: Keep 18. In the Appendix B overview, Ch. 1:83-85 and Ch. 12:185-187, add "except \mnemonic{RSET}, whose reset request replaces the dispatch slot". In the RSET row add "(no dispatch slot)". Make Ch. 16:556 for RSET say the same.
- confidence: medium
- status: accepted

### F-tim-3
- file: reference/chapters/appendix-b-timing.tex
- lines: 178-182
- severity: major
- category: fact
- claim: "an exception or fault dispatch adds two exception-vector-fetch accesses; a coprocessor dispatch performs no bus access of its own"
- evidence: EXCP is a coprocessor call whose exception-unit dispatch does fetch a vector (16-coprocessor-instructions.tex:341-342 says a *discarded* EXCP fetches no vector, which implies an accepted one does). The bullet does not say whether an EXCP slot counts as a "coprocessor dispatch" (0 accesses) or an "exception dispatch" (2). Its wait-state totals list only ALU, load/store and DMA, so the EU meta-instructions get no figure.
- resolution: manual-wrong
- fix: Change to: "an exception-unit dispatch that enters a handler (a hardware interrupt, a fault, an accepted \mnemonic{EXCP}, or reset) adds two exception-vector-fetch accesses; any other coprocessor dispatch (\mnemonic{RETE}, \mnemonic{ETFR}, \mnemonic{ETTR}, \mnemonic{WAIT}, maths) performs no bus access of its own". Then add "$4w$ cycles to an accepted \mnemonic{EXCP} and $2w$ to the other exception-unit meta-instructions".
- confidence: medium
- status: accepted

### F-tim-4
- file: reference/chapters/appendix-b-timing.tex
- lines: 179
- severity: major
- category: fact
- claim: "a coprocessor dispatch performs no bus access of its own"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:323-341. Every exception-unit dispatch issues two BAT-100 reads at `vector*2` and `vector*2+1`, whatever the operation, so RETE (0x1A00, value 0) reads the reset vector and ETFR/ETTR/WAIT read the entry selected by their operand byte. This is not in general-todo.md.
- resolution: code-wrong
- fix: No manual change once F-tim-3 is applied. New simulator todo: the exception unit fetches a vector only for a dispatch that enters a handler (interrupt, fault, accepted EXCP, reset). Add a bus-trace test for RETE/ETFR that expects no BAT-100 access.
- confidence: medium
- status: simulator

### F-tim-5
- file: reference/chapters/appendix-b-timing.tex
- lines: 47
- severity: major
- category: fact
- claim: "Exception-unit meta-instructions (\mnemonic{EXCP}, \mnemonic{RETE}, \mnemonic{ETFR}, \mnemonic{ETTR}) & 12 (6 + 6 dispatch)"
- evidence: 16-coprocessor-instructions.tex:340-342: an EXCP issued at a non-zero exception level "is discarded -- the CSR is cleared, no vector is fetched, and no fault is raised". Neither chapter says whether a discarded EXCP still occupies its 6-cycle dispatch slot (12 cycles) or takes only 6.
- resolution: design
- fix: Options: (a) the slot always runs and its phases are no-ops, so it takes 12 cycles; (b) the CSR is cleared at the next phase 0 and fetch resumes at once, so it takes 6. Recommend (a): it keeps the timing data-independent and matches "every coprocessor call takes a dispatch slot". State the answer in the table and in Ch. 16's EXCP Timing line.
- confidence: low
- status: accepted
- decision: A discarded EXCP (inside a handler) takes 12 cycles: its dispatch slot runs with no-op phases. State it in the Appendix B table and Ch.16 EXCP Timing line.

### F-tim-6
- file: reference/chapters/appendix-b-timing.tex
- lines: 67-78
- severity: major
- category: fact
- claim: "The worst-case latency from the interrupt request to the handler's first instruction fetch is the time the current instruction still needs plus 6 cycles to dispatch"
- evidence: Table row 45 gives MULU/MULS/DIVU/DIVS as "At least 12; implementation-defined". 16-coprocessor-instructions.tex:1364-1366 says "a pending interrupt waits for them". The bulleted list stops at DMAT 528 and so reads as the global worst case, but on a part with a maths unit the latency has no bound fixed by the architecture.
- resolution: manual-wrong
- fix: Add a bullet: "A maths-unit command: 12 + its implementation-defined extra cycles + 6; the architecture sets no upper bound, so an implementation must document it." Then say that the DMAT figure is the worst case only for parts without a maths unit, or for ones whose maths operations finish in fewer cycles.
- confidence: medium
- status: accepted

### F-exc-3
- file: reference/chapters/appendix-c-undocumented.tex
- lines: 6-9
- severity: major
- category: fact
- claim: "Their behavior is architecturally undefined: an implementation decodes them without raising an invalid-opcode fault, but what they do is not part of the architectural contract"
- evidence: facts.md:56-63 (privilege checks on high-word and pair writes); reference/chapters/06-exceptions.tex:275-276 (the same "undefined, no fault" rule for all other encodings); general-todo.md items 1 and 8 (virtualisation requirement; LDEA/LDEL bits 25-24 reserved, panic). "Undefined" puts no limit on behaviour, so a conforming implementation could let protected-mode code use 0x27/0x2F, aliased writes or reserved field bits to write `ph`/`sh` or SR bits 15-8 without a privilege-violation fault.
- resolution: design
- fix: Options: (a) state that every undefined or undocumented encoding is still subject to the privilege rules: it cannot change the privileged SR byte, a high address-register word, or exception-unit state in protected mode, and it may write only registers named in its fields; (b) make them raise an invalid-opcode fault. Recommend (a). Add the sentence to Appendix C Overview and to 06-exceptions.tex:275-276, and refer to it from the aliased-write section.
- confidence: medium
- status: accepted
- decision: Undefined and undocumented encodings remain bound by the privilege rules: in protected mode they cannot change the privileged SR byte, a high address-register word or exception-unit state, and they may write only registers named in their fields. Add to Appendix C Overview and 06-exceptions:275-276, referenced from the aliased-write section.

### F-exc-38
- file: reference/chapters/appendix-c-undocumented.tex
- lines: 34-39
- severity: major
- category: fact
- claim: "LOAD r1, (#0, p)+ ... The assembler rejects the known hazardous forms"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/load.rs:140-149 (only a destination that overlaps the pair is rejected; a `p` auto-update is accepted); same in store.rs. todo 27.
- resolution: code-wrong
- fix: No manual change. Implement todo 27.
- confidence: high
- status: simulator

### F-sum-10
- file: reference/chapters/appendix-d-examples.tex
- lines: 40-43
- severity: major
- category: fact
- claim: "it reads exception metadata with \mnemonic{ETFR} to distinguish a program-counter overflow, which records no bus access, from a \mnemonic{LOAD} or \mnemonic{STOR} effective-address overflow, which records a data read or write"
- evidence: examples/faults/faults.sasm:206-214 compares only with $BAT_DATA_WRITE (0x3), but the EA overflow the program raises is `LOAD r7, (#2, a)` at :84, which records BAT 0x2 under reference/chapters/06-exceptions.tex:239; the handler then falls through to RETE and retries the LOAD forever. The comment at faults.sasm:233-235 ("This branch is not reached by anything in this example today") is also wrong under the manual. general-todo.md todo 32 (with todo 31).
- resolution: code-wrong
- fix: Manual stays; covered by todo 32. Also delete the "not reached" comment at faults.sasm:233-235 when fixing.
- confidence: high
- status: simulator

### F-sum-11
- file: reference/chapters/appendix-d-examples.tex
- lines: 31, 48, 73, 94, 109
- severity: major
- category: fact
- claim: "\lstinputlisting[caption={Example~\theexample: Byte sieve}]{../../examples/byte-sieve/byte-sieve.sasm}"
- evidence: examples/byte-sieve/byte-sieve.sasm:60, faults/faults.sasm:156 (`COPI #0b0001_0100_1111_1111` = 0x14FF), hardware-exception/hardware-exception.sasm:131, software-exception/software-exception.sasm:23, store-load/store-load.sasm:146 all end with `COPI #0x14FF` ("Halt CPU"), a simulator hook (`pu-execution.rs:170-177`). Architecturally this is an exception-unit command with operation 0x4, which 16-coprocessor-instructions.tex:217 says raises an invalid-opcode fault. general-todo.md todo 25.
- resolution: code-wrong
- fix: Manual stays; covered by todo 25. Until the examples change, the Program Conventions section could note that the final `COPI #0x14FF` is a simulator exit hook, not an architectural halt.
- confidence: high
- status: merged into F-con-1

### F-sum-4
- file: reference/chapters/notation-and-glossary.tex
- lines: 134-140
- severity: major
- category: fact
- claim: "In any other instruction operand (\mnemonic{MOVI}, \mnemonic{LOAD}, \mnemonic{STOR}, \mnemonic{LDEA}, \mnemonic{LDEL}, \mnemonic{LJMP}, \mnemonic{LJSR}) it resolves to the label's low word only."
- evidence: sirc-vm/toolchain/src/bin/linker.rs:170-177 patches only opcodes 0x07, 0x10, 0x14, 0x18, 0x1A, 0x1C, 0x1E; the parser accepts `@sym` in `LOAD rD, (@sym, A)+` (0x16) and `STOR -(@sym, A), rS` (0x12) (`load.rs:164-177`; `store.rs:181-194`), and the linker then panics "Can't patch address/offset for instruction" (`linker.rs:203-206`). docs/reference/review/facts.md Open question 3. Not in general-todo.md.
- resolution: code-wrong
- fix: Manual stays. New toolchain todo: add 0x12 (StoreRegisterToIndirectImmediatePreDecrement) and 0x16 (LoadRegisterFromIndirectImmediatePostIncrement) to the linker's patchable set and add link tests for both forms.
- confidence: high
- status: simulator

### F-asm-24
- file: assembly/chapters/01-introduction.tex
- lines: 35
- severity: minor
- category: fact
- claim: "The \textbf{simulator} loads the binary at address zero, maps whatever memory the program needs, and runs it."
- evidence: The simulator always maps fixed segments: PROGRAM 0x0 (0xFFFF words, read-only), TERMINAL 0x0A0000 and DEBUG 0x0B0000, plus VIDEO with -e. Any other memory must be mapped with --segment (sirc-vm/sirc-vm/src/main.rs:191-262). It does not work out what the program needs.
- resolution: manual-wrong
- fix: "The simulator loads the binary at address zero, maps its fixed devices and any memory segments given on the command line, and runs it."
- confidence: high
- status: accepted

### F-asm-15
- file: assembly/chapters/02-source-language.tex
- lines: 61-62
- severity: minor
- category: fact
- claim: "Two directive operands are written without the hash: the address of \texttt{.ORG} is a bare decimal or \texttt{0x}-prefixed hexadecimal number"
- evidence: Only one operand is named. The other unhashed directive operand is the .EQU name (`$NAME`; data.rs:77-83). .ORG also rejects binary (shared.rs:116-120: `alt((parse_hex, parse_dec))`; `.ORG 0b10` fails).
- resolution: manual-wrong
- fix: "One directive operand is written without the hash: the address of .ORG, which is a bare decimal or 0x-prefixed hexadecimal number (binary is not accepted) (Section 5.1)."
- confidence: medium
- status: accepted

### F-asm-22
- file: assembly/chapters/02-source-language.tex
- lines: 58-59
- severity: minor
- category: fact
- claim: "A negative decimal number is stored in two's complement. The value must fit the field it is encoded into"
- evidence: A negative literal in the 8-bit short-immediate field is always rejected. `ADDI r1, #-3, LSL #1` gives "Immediate values must fit into 8 bits when using a shift definition (65533 > 0xFF)", because the value is truncated to u16 before the 8-bit check (instruction.rs:114-116; arithmetic_immediate.rs:227-234). A negative shift count is rejected as 4294967295. A reader could take "fit the field" to mean that -128..-1 fit an 8-bit field.
- resolution: manual-wrong
- fix: Add: "The 8-bit short-immediate field and shift counts are unsigned. A negative number is rejected there."
- confidence: medium
- status: accepted

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
- status: accepted

### F-asm-21
- file: assembly/chapters/03-operands.tex
- lines: 69-70
- severity: minor
- category: fact
- claim: "A register-form ALU instruction with two register operands is shorthand for the three-operand form"
- evidence: MOVR is in the ALU family (04-instructions.tex:89), but `MOVR rD, rS` is its only form. It encodes r2 = 0, not a copy of rD, and a three-register MOVR is rejected (mov.rs:137-160,176-182).
- resolution: manual-wrong
- fix: "A register-form ALU instruction other than MOVR, written with two register operands, is shorthand for ..."
- confidence: high
- status: accepted

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
- status: accepted

### F-asm-16
- file: assembly/chapters/04-instructions.tex
- lines: 13-14
- severity: minor
- category: fact
- claim: "The operands follow after at least one space."
- evidence: parse_instruction_tag consumes `space0` after the mnemonic, suffix and condition (instruction.rs:401). `ADDIr1, #1` assembles.
- resolution: code-wrong
- fix: Keep the manual. Toolchain todo: require `space1` between the mnemonic group and the first operand when operands are present.
- confidence: high
- status: simulator

### F-asm-25
- file: assembly/chapters/04-instructions.tex
- lines: 74-75
- severity: minor
- category: fact
- claim: "The register-register form is selected by writing registers for the operands: \mnemonic{ADDR} rather than \mnemonic{ADDI}."
- evidence: The mnemonic selects the format, not the operand spelling. `ADDI r1, r2` is an error ("only supports immediate->register addressing mode", arithmetic_immediate.rs:282-289), and ADDR rejects an immediate.
- resolution: manual-wrong
- fix: "The register format is selected by the mnemonic: write ADDR with register operands, not ADDI."
- confidence: medium
- status: accepted

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
- status: accepted

### F-asm-14
- file: assembly/chapters/05-directives-and-symbols.tex
- lines: 8-9
- severity: minor
- category: fact
- claim: "It must be even, because instructions and data directives each occupy two words."
- evidence: Not enforced. `.ORG 0x0001` assembles without a diagnostic. object.rs:177-180 sets the byte offset to 2, and the next slot is written at `offset/4` (object.rs:133), which is the slot at word 0. Labels, however, record word 1. So the code overwrites the previous slot and its labels are off by one word. reference/chapters/notation-and-glossary.tex:122 states the same rule.
- resolution: code-wrong
- fix: Keep the manual. Toolchain todo: reject an odd .ORG address with a diagnostic.
- confidence: high
- status: simulator

### F-asm-20
- file: assembly/chapters/06-toolchain.tex
- lines: 65-67
- severity: minor
- category: fact
- claim: "\texttt{--segment-offset} (short form \texttt{-s}) is added to every label's address before it is patched in."
- evidence: N is parsed as a decimal u32 (clap `u32`, linker.rs:96-97), so `-s 0x10` is rejected. It is a word offset (linker.rs:136-140), and the option is required. The simulator's --segment offsets are hexadecimal (06-toolchain.tex:86), so readers will expect hex here too.
- resolution: manual-wrong
- fix: Add: "N is a decimal number of words, and the option is required." Optionally add the hex/decimal contrast with --segment.
- confidence: high
- status: accepted

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
- status: accepted

### F-asm-17
- file: assembly/chapters/07-diagnostics.tex
- lines: 6-22
- severity: minor
- category: fact
- claim: "The assembler stops at the first error. It prints the file, line and column ... error: parsing failed"
- evidence: The assembler reports the error by panicking (sirc-vm/toolchain/src/bin/assembler.rs:48-58). The real output starts with "thread 'main' panicked at toolchain/src/bin/assembler.rs:58:13:" / "Error parsing file:" and ends with the RUST_BACKTRACE note, and the process exits with status 101. Undefined constants and an oversized constant also surface as bare panics (object.rs:22-29,93-94), not in this format. The rest of the example reproduces exactly.
- resolution: code-wrong
- fix: Keep the manual's format as the intended output. Toolchain todo: print the formatted error to stderr and exit with a non-zero status without panicking, and route the placeholder errors through the same formatter. Optionally add "The assembler exits with a non-zero status on any error" to Section 7.1.
- confidence: high
- status: simulator

### F-asm-18
- file: assembly/chapters/07-diagnostics.tex
- lines: 45-46
- severity: minor
- category: fact
- claim: "When an instruction is given operands it does not accept, the message lists the operand forms the assembler tried in turn."
- evidence: For a well-formed but unsupported operand list the message lists the operands as parsed, in Rust Debug form: `LOAD r1, (#1, a), r2` gives "Invalid addressing mode for LOAD: ([DirectRegister(R1), IndirectImmediateDisplacement(Value(1), Address), DirectRegister(R2)])" (load.rs:242-248). ALU instructions give a fixed example instead (arithmetic_immediate.rs:283). Only a token that fails to parse produces an "expected ..." list of alternatives (`LOAD r1, (r2)`).
- resolution: manual-wrong
- fix: "When an instruction is given operands it does not accept, the message names the instruction and either shows the operands it parsed or gives an example of an accepted form. When an operand cannot be parsed at all, the message lists the tokens the assembler expected at that point."
- confidence: high
- status: accepted

### F-asm-19
- file: assembly/chapters/07-diagnostics.tex
- lines: 38
- severity: minor
- category: fact
- claim: "An instruction whose destination overlaps its address register & Aliased register writes are undefined for that instruction"
- evidence: The rejected set includes cases where no destination is involved: STOR pre-decrement whose source is a half of the pair, LDEL post-increment with source `l`, and LJSR post-increment from `l` or `p` (opcodes/mod.rs:29-49; facts.md "Forms rejected"). The assembly manual, which is authoritative for what the assembler accepts, never lists the set. Also known (review/facts-enc.md:52-54): the architecture manual says auto-update of `p` is rejected, but `LOAD r1, (#0, p)+` and `STOR -(#0, p), r1` assemble (confirmed here).
- resolution: manual-wrong
- fix: Change the row to "A pre-decrement or post-increment form whose other register is a half of the updated pair, or an LDEL/LJSR that would write `l` twice". Add a sentence pointing to the exact list in the Memory Access and Control Flow chapters of the architecture manual. The `p` gap stays as tracked in facts-enc.
- confidence: medium
- status: accepted

### F-asm-23
- file: assembly/chapters/08-limitations.tex
- lines: 27-28
- severity: minor
- category: fact
- claim: "The architecture requires that every meta-instruction assemble to exactly one instruction, so a source line is always one instruction"
- evidence: A line can hold a label, directive, comment or nothing. Tokens are separated by optional whitespace, not newlines, so `:a1 :b1 NOOP` and `NOOP NOOP` on one line both assemble (shared.rs:255-267).
- resolution: manual-wrong
- fix: "... so every mnemonic written in the source produces exactly one instruction (see ...)."
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted

### F-hw-26
- file: hardware/chapters/02-package-and-pinout.tex
- lines: 6
- severity: minor
- category: fact
- claim: "The SIRC CPU is most commonly provided as a DIP-64 package."
- evidence: hw Ch.1 line 23 ("the 64-pin package"); hw Ch.6 line 16 (package dimensions unspecified); no other package is described anywhere in the set
- resolution: manual-wrong
- fix: "The SIRC-1 is provided in a 64-pin dual in-line package (DIP-64)." If other packages are planned, say that their pinouts are not covered by this revision.
- confidence: medium
- status: accepted

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
- status: accepted

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
- status: accepted

### F-hw-22
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 149-151
- severity: minor
- category: contradiction
- claim: "A memory controller might trigger a bus error if BAT is 010 (data read), PROT is 1 (protected mode) and the address is in a protected memory region."
- evidence: hw Ch.3 lines 125-133 (BPER is the response for a disallowed access to a valid device) and lines 187-188 ("block access to protected regions of memory by raising bus protection errors")
- resolution: manual-wrong
- fix: Replace "trigger a bus error" with "assert BPER (bus protection error)".
- confidence: high
- status: accepted

### F-hw-23
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 111-113
- severity: minor
- category: fact
- claim: "This causes the CPU to abort the I/O operation and raise a bus fault exception."
- evidence: reference/chapters/06-exceptions.tex:921-924 (a BERR/BPER on the reset-vector fetch raises no fault and re-enters reset), :365-366 and :769 (on a fault-vector fetch it escalates to a double fault; on the double-fault vector fetch the CPU resets); same wording for BPER at hw Ch.3 line 128
- resolution: manual-wrong
- fix: Append to both BERR and BPER: "except during an exception-vector fetch, where the response is described in the Exception and Interrupt Handling chapter of the \archmanual{} (a failed fault-vector fetch raises a double fault, and a failed reset-vector fetch re-enters reset)."
- confidence: high
- status: accepted

### F-hw-25
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 257-264, 289-290
- severity: minor
- category: fact
- claim: "\item \textbf{Direction:} Input"
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:272-277 and :477-486 (the CPU asserts `reset_requested`, the RSTI field, in its own bus output for RSET and for a triple fault); facts.md Signals table ("in (and out for RSET or a triple fault)")
- resolution: code-wrong
- fix: Manual stays: RSTI is an input and a software reset is internal, visible only on RSTO. Simulator todo: carry the software/triple-fault reset request to the reset unit on an internal path, not on the shared RSTI field, so a bus monitor never sees RSTI driven by the CPU.
- confidence: medium
- status: simulator

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
- status: accepted

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
- status: merged into F-hw-18

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
- status: accepted
- decision: Keep IRQ asserted to the end of the interrupt timing figure (L L L L L); a real device holds the line until its handler acknowledges it.

### F-hw-20
- file: hardware/chapters/05-bus-timing.tex
- lines: 203
- severity: minor
- category: contradiction
- claim: "IRQ1--IRQ4  & L L H H H"
- evidence: hw Ch.3 lines 233-234 and Ch.4 line 96 ("The requesting device must hold the line asserted until its handler acknowledges it"); reference/chapters/06-exceptions.tex:462-463
- resolution: manual-wrong
- fix: Hold the IRQ row asserted for the whole figure (`L L L L L`), since the handler has not yet run; withdrawing it during the vector fetch shows a device the text says is non-conforming.
- confidence: high
- status: accepted

### F-hw-21
- file: hardware/chapters/05-bus-timing.tex
- lines: 191-195
- severity: minor
- category: fact
- claim: "The CPU samples the interrupt lines on every clock cycle, so the request is latched in the cycle it is first seen, but it is dispatched only at an instruction boundary."
- evidence: reference/chapters/06-exceptions.tex:489-491 and appendix-b-timing.tex:66-67 (dispatch needs a phase 0 at which the CSR is empty and no coprocessor is active); sirc-vm/peripheral-cpu/src/lib.rs:318,366-380 (lines sampled on the phase-0 poll itself before the dispatch decision)
- resolution: manual-wrong
- fix: "... but it is dispatched only at the next phase 0 at which no coprocessor call or command is pending or active. A request first sampled at the rising edge that starts phase 0 is dispatched in that phase 0." The second sentence fixes the edge relative to the dispatch decision, which the figure implies but the text does not state.
- confidence: medium
- status: accepted

### F-hw-24
- file: hardware/chapters/05-bus-timing.tex
- lines: 80, 182, 208
- severity: minor
- category: fact
- claim: "A0--A23     & D{vec+0} D{vec+1} D{} D{}"
- evidence: reference/chapters/06-exceptions.tex:1010 (vector 0x06 at 0x000C-0x000D); facts.md Exceptions (vector n at word addresses 2n and 2n+1)
- resolution: manual-wrong
- fix: Label the address band `2n` / `2n+1` (vector n) and add to the Ch.5 legend at lines 9-11 "EV ... for exception vector fetch of vector n, at word addresses 2n and 2n+1". "vec+0" reads as the vector number plus zero.
- confidence: medium
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: simulator

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
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: accepted

### F-data-19
- file: reference/chapters/04-data-representation.tex
- lines: 126-127
- severity: minor
- category: fact
- claim: "Those forms use 16-bit immediate displacements, register displacements, or assembler-resolved full-width branch offsets."
- evidence: facts.md Opcode map (BRAN = LDEA p,(#n,p) with a 16-bit imm16 displacement) and Linker (`.r` resolves to an i16 and panics if out of range; sirc-vm/toolchain/src/bin/linker.rs:40-86). "Full-width" suggests a 24-bit offset; branch offsets are 16-bit signed and reach only within the current segment.
- resolution: manual-wrong
- fix: "Those forms use 16-bit signed immediate displacements (including assembler-resolved PC-relative branch offsets) or register displacements."
- confidence: high
- status: accepted

### F-data-10
- file: reference/chapters/05-status-register.tex
- lines: 308-316
- severity: minor
- category: fact
- claim: "; Save status register (supervisor mode only -- see the manual updates section)"
- evidence: reference/chapters/05-status-register.tex:244-247,256-257 (in protected mode the read returns 0x00XX and the write updates only bits 7-0); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:40-48, write_back.rs:45-49. The sequence `MOVR r7, sr` / `MOVR[N] sr, r7` correctly saves and restores the flags in protected mode too; only the high byte is out of reach.
- resolution: manual-wrong
- fix: "; Save status register (in protected mode only bits 7-0 are saved and restored)"
- confidence: high
- status: accepted

### F-data-11
- file: reference/chapters/05-status-register.tex
- lines: 81-82
- severity: minor
- category: fact
- claim: "Overflow occurs when adding two positive numbers yields a negative result, or adding two negative numbers yields a positive result"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:195-267 (SUB/CMP/SBC/CPC compute V on (a, !b, result)); facts.md "Shift types" (ASL sets V on a sign change). The note gives only the addition rule, but V is also set by subtraction and ASL.
- resolution: manual-wrong
- fix: Append: "For subtraction (a - b), overflow occurs when a and b have different signs and the result's sign differs from a. Shift operations define V separately (Chapter~\ref{ch:shift-operations})."
- confidence: high
- status: accepted

### F-data-12
- file: reference/chapters/05-status-register.tex
- lines: 200-210
- severity: minor
- category: fact
- claim: "\textbf{Arithmetic Instructions} (\mnemonic{ADDI}, \mnemonic{SUBI}, \mnemonic{ADCI}, \mnemonic{SBCI})" ... "\textbf{Load Instructions} (\mnemonic{LOAD}, \mnemonic{LDEA}): Never update the condition flags"
- evidence: facts.md Opcode map (ADDR/SUBR/ADCR/SBCR and ANDR/ORRR/XORR update flags in the same way); write_back.rs:300-307 (memory/EA opcodes 0x10-0x1F never update SR, including STOR and LDEL). The compare and test bullets list both I and R forms, but the arithmetic and logical bullets list only I forms, and the no-flag bullet omits STOR, LDEL and the branch/jump aliases.
- resolution: manual-wrong
- fix: List the R forms in the arithmetic and logical bullets (ADDR, SUBR, ADCR, SBCR; ANDR, ORRR, XORR). Replace the load bullet with "\textbf{Memory and Address Instructions} (\mnemonic{LOAD}, \mnemonic{STOR}, \mnemonic{LDEA}, \mnemonic{LDEL} and their aliases): Never update the condition flags; AF selects the address register for these opcodes."
- confidence: high
- status: accepted

### F-data-8
- file: reference/chapters/05-status-register.tex
- lines: 248-252
- severity: minor
- category: contradiction
- claim: "This applies to every instruction that names \reg{sr} as its destination, including the flag-only comparison and test instructions (...), which do not write their destination and are therefore unaffected."
- evidence: none (internal: the sentence says the rule applies to these instructions, then says they are unaffected)
- resolution: manual-wrong
- fix: "This applies to every instruction that writes \reg{sr} as its destination. The flag-only comparison and test instructions (\mnemonic{CMNI} ... \mnemonic{TSXR}) do not write their destination register, so naming \reg{sr} there only affects the flags in bits 3--0."
- confidence: high
- status: accepted

### F-data-9
- file: reference/chapters/05-status-register.tex
- lines: 281
- severity: minor
- category: contradiction
- claim: "When returning from an exception with \mnemonic{RETE}, the saved status register is restored."
- evidence: reference/chapters/05-status-register.tex:159,166-169 (EA is set from the restored level, not from the saved SR)
- resolution: manual-wrong
- fix: "When returning from an exception with \mnemonic{RETE}, the saved status register is restored, except bit 13 (\texttt{EA}), which is set from the restored exception level."
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted

### F-exc-10
- file: reference/chapters/06-exceptions.tex
- lines: 765-767
- severity: minor
- category: contradiction
- claim: "If a bus or protection fault occurs during an exception-vector fetch, the original exception does not enter and the remaining phases of its dispatch slot run as no-ops (Section~\ref{sec:exception-and-fault-timing}). The CPU raises the corresponding fault"
- evidence: reference/chapters/06-exceptions.tex:921-924 (a failed reset-vector fetch raises no fault and re-enters reset); appendix-b-timing.tex:100-101.
- resolution: manual-wrong
- fix: Start the paragraph with "Except for the reset vector (Section~\ref{sec:reset-vector-fetch}), if a bus or protection fault occurs ..."
- confidence: high
- status: accepted

### F-exc-11
- file: reference/chapters/06-exceptions.tex
- lines: 573-575
- severity: minor
- category: contradiction
- claim: "If an \mnemonic{EXCP} instruction is executed while any exception or fault handler is already active, the software exception is ignored, is not queued, and does not fetch its vector."
- evidence: reference/chapters/16-coprocessor-instructions.tex:341-343 ("The privilege check comes first, so a protected-mode EXCP with a vector below 0x60 inside a handler still raises a privilege-violation fault"). The rule at 06-exceptions.tex:52-53 and :748 has no such qualifier either.
- resolution: manual-wrong
- fix: Append: "The privilege check is made first: a protected-mode \mnemonic{EXCP} with a vector below \texttt{0x60} raises a privilege-violation fault even inside a handler." Add the same qualifier at line 748.
- confidence: high
- status: accepted

### F-exc-12
- file: reference/chapters/06-exceptions.tex
- lines: 318-319
- severity: minor
- category: fact
- claim: "A debugger single-stepping through a system call therefore stops once before the trap handler runs, and again at the instruction after the \mnemonic{EXCP} once the handler has returned."
- evidence: reference/chapters/06-exceptions.tex:306-311. The trap handler's RETE runs with T=0 (it was cleared on entry), so RETE is not traced. The next trace fault fires after the instruction following the EXCP has executed, and LR6 then holds the address after that instruction. The first stop already reported the instruction after the EXCP as next.
- resolution: manual-wrong
- fix: "... stops once before the trap handler runs (with the instruction after the \mnemonic{EXCP} as the return address), and again after that instruction has executed once the handler has returned."
- confidence: medium
- status: accepted

### F-exc-13
- file: reference/chapters/06-exceptions.tex
- lines: 524-526
- severity: minor
- category: fact
- claim: "The L5 pending bit is cleared at the moment L5 is selected for dispatch, and set again only if its vector fetch fails. An edge that arrives after selection, even during the dispatch slot, is therefore a new request"
- evidence: reference/chapters/06-exceptions.tex:514-516 (overrun is recorded only when an edge arrives while the bit is already set) and :774. If a new edge sets the bit during the dispatch slot and the vector fetch then fails, "set again" sets a bit that is already set. One L5 request is lost with no overrun and no conflict fault.
- resolution: design
- fix: Options: (a) when the vector fetch fails and the L5 bit is already set, set the L5 overrun flag; (b) state that the two requests coalesce silently. Recommend (a), since the chapter's stated aim (line 342-344) is never to drop an L5 silently.
- confidence: medium
- status: accepted
- decision: If an L5 vector fetch fails while the L5 pending bit has already been set again by a new edge, the L5 overrun flag is set.

### F-exc-16
- file: reference/chapters/06-exceptions.tex
- lines: 743-745
- severity: minor
- category: fact
- claim: "A hardware interrupt is accepted only if its line is enabled in the status register and its priority is higher than the current exception level."
- evidence: reference/chapters/06-exceptions.tex:444-445 (Level 5 has no enable bit and cannot be disabled).
- resolution: manual-wrong
- fix: "A hardware interrupt is accepted only if its line is enabled in the status register (Level 5 is always enabled) and its priority is higher than the current exception level."
- confidence: high
- status: accepted

### F-exc-36
- file: reference/chapters/06-exceptions.tex
- lines: 100
- severity: minor
- category: fact
- claim: "\mnemonic{WAIT}] Stops instruction fetch until an interrupt that can be taken, or a reset, occurs."
- evidence: facts.md:352 (any enabled interrupt clears WAIT, with no level check); lib.rs:315-322. todo 24.
- resolution: code-wrong
- fix: No manual change. Implement todo 24.
- confidence: high
- status: simulator

### F-exc-37
- file: reference/chapters/06-exceptions.tex
- lines: 603-605
- severity: minor
- category: fact
- claim: "The 32-bit program-counter value to resume at ... with bits 31--24 carried unchanged from \reg{p}."
- evidence: facts.md:40-43 (pair addresses are masked to 24 bits, `registers.rs:289-295`). todo 5.
- resolution: code-wrong
- fix: No manual change. Implement todo 5.
- confidence: medium
- status: simulator

### F-exc-4
- file: reference/chapters/06-exceptions.tex
- lines: 709-718
- severity: minor
- category: contradiction
- claim: "For a fault that does not access the bus, this field is architecturally undefined, with three exceptions:"
- evidence: reference/chapters/06-exceptions.tex:242-244 ("An address-register auto-update that wraps is recorded the same way, with the wrapped register value"). That is a fourth defined case of a fault with no bus access, but this list leaves it out.
- resolution: manual-wrong
- fix: Change "three exceptions" to "four exceptions" and add: "An address-register auto-update segment overflow stores the wrapped register value (Section~\ref{sec:segment-overflow-fault})." Or change the third bullet to "An effective-address or auto-update segment overflow stores the wrapped value".
- confidence: high
- status: accepted

### F-exc-5
- file: reference/chapters/06-exceptions.tex
- lines: 250-256
- severity: minor
- category: fact
- claim: "and neither does an ALU or \mnemonic{LOAD} instruction that writes \reg{pl}"
- evidence: reference/chapters/appendix-b-timing.tex:110-111 (a skipped instruction in the last slot raises the fault in phase 3). An ALU or LOAD to `pl` whose condition is false writes nothing and continues at the advanced PC, so it does use the advanced address. As written, the sentence exempts it.
- resolution: manual-wrong
- fix: "... and neither does an executed (condition true) ALU or \mnemonic{LOAD} instruction that writes \reg{pl}". Apply the same qualifier to "an ALU or \mnemonic{LOAD} instruction that writes \reg{pl}" at line 256.
- confidence: high
- status: accepted

### F-exc-6
- file: reference/chapters/06-exceptions.tex
- lines: 169-170
- severity: minor
- category: contradiction
- claim: "The link register stores the program counter at the point the fault was detected within the exception handler."
- evidence: reference/chapters/06-exceptions.tex:333-339 and :514-521. The L5-overrun conflict fault is raised when L5 is next selected, which can happen at exception level 0, for example after a fault handler RETEs to user code. In that case no exception handler is running.
- resolution: manual-wrong
- fix: "The link register stores the program counter at the point the fault was detected, which may be inside an exception handler or, for an L5 overrun, in ordinary code."
- confidence: medium
- status: accepted

### F-exc-7
- file: reference/chapters/06-exceptions.tex
- lines: 1012
- severity: minor
- category: contradiction
- claim: "\texttt{0x08}                & \addr{0x0010}--\addr{0x0011} & 7                 & Double fault     & Not maskable         & Dispatch point"
- evidence: reference/chapters/06-exceptions.tex:172-174 (a double fault raised by a retryable fault in a handler stores that instruction's address) and :816-817 (a vector-fetch double fault returns to the original fault's return address). "Dispatch point" matches neither case. The same simplification appears at 630-631.
- resolution: manual-wrong
- fix: Change the Return Address cell to "See Section~\ref{sec:double-fault}". At 630-631 add "except as described in Section~\ref{sec:fault-categories}".
- confidence: high
- status: accepted

### F-exc-8
- file: reference/chapters/06-exceptions.tex
- lines: 531-532
- severity: minor
- category: contradiction
- claim: "it is not queued. It will raise a Level Five Hardware Exception Conflict fault instead, which consumes the pending request."
- evidence: reference/chapters/06-exceptions.tex:525-526 (a new edge "sets the pending bit again, and once the L5 handler is running it raises" the conflict). The request is queued in the pending bit, and the same sentence then speaks of "the pending request".
- resolution: manual-wrong
- fix: "it is not dispatched as a second L5. It sets the L5 pending bit, and at the next instruction boundary it raises a Level Five Hardware Exception Conflict fault, which consumes that pending bit."
- confidence: high
- status: accepted

### F-exc-9
- file: reference/chapters/06-exceptions.tex
- lines: 63-64
- severity: minor
- category: contradiction
- claim: "Each vector is a 24-bit target address stored as two consecutive 16-bit words"
- evidence: reference/chapters/04-data-representation.tex:87 ("a stored 32-bit pointer with contains a 24-bit address and eight bits of metadata"); 06-exceptions.tex:603-605 (bits 31-24 are carried in the return address).
- resolution: manual-wrong
- fix: "Each vector is a 32-bit pointer (a 24-bit target address and eight bits of metadata) stored as two consecutive 16-bit words"
- confidence: high
- status: accepted

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
- status: accepted

### F-enc-16
- file: reference/chapters/07-instruction-formats.tex
- lines: 359-367
- severity: minor
- category: fact
- claim: "\textbf{Opcodes (Imm / Short / Reg)}" ... "Add & ADDI / ADDR & CMNI / CMNR & \opcode{08} / \opcode{28} / \opcode{38}"
- evidence: The column header implies the opcodes of the row, but only the test-only opcodes are listed; ADDI/ADDR are 0x00/0x20/0x30 (appendix-a-opcode-map; facts.md opcode map).
- resolution: manual-wrong
- fix: Rename the column "Test-only opcodes (Imm / Short / Reg)", or add a "Save opcodes" column with 00/20/30 ... 06/26/36.
- confidence: high
- status: accepted

### F-enc-7
- file: reference/chapters/07-instruction-formats.tex
- lines: 62, 485
- severity: minor
- category: improvement
- claim: "\textbf{Note:} Due to the large 16-bit operand, shift is not supported in this format." / "AF (5--4) & Status update source: \texttt{00} none, \texttt{01} ALU result, \texttt{10} shifter result"
- evidence: reference/chapters/13-alu-instructions.tex:198-201 defines that [S] on an immediate-format instruction clears N, Z, C and V; Ch. 7 lists AF=10 "shifter result" for the immediate group with no hint that there is no shifted operand. Same gap in 09-shift-operations.tex:603-608 (Assembler Syntax Notes).
- resolution: n/a
- fix: Append to the note at 07:62: "AF = 10 is still encodable; with no shift fields the shifter result is treated as zero flags, so it clears N, Z, C and V (Chapter~\ref{ch:alu-instructions})." Add a matching bullet at 09:603-608.
- confidence: high
- status: accepted

### F-enc-15
- file: reference/chapters/08-addressing-modes.tex
- lines: 344
- severity: minor
- category: fact
- claim: "\textbf{Usage:} Loading constant values; arithmetic with known values; bit manipulation with masks."
- evidence: The short-immediate form cannot load a constant: there is no public short-immediate MOVI (08:93), and 0x27 is undocumented (07:351-353). Every short-immediate ALU op combines a shifted register with the immediate.
- resolution: manual-wrong
- fix: "\textbf{Usage:} Shift-and-add or shift-and-mask in one instruction, for example scaling a register and adding a small constant."
- confidence: high
- status: accepted

### F-enc-8
- file: reference/chapters/08-addressing-modes.tex
- lines: 92
- severity: minor
- category: fact
- claim: "\texttt{CMNR/CNCR/CMPR/CPCR/TSAR/TSOR/TSXR rS1, rS2[, shift]}."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:120-140,166-189 accept a three-register form for the test mnemonics too (`CMPR r1, r2, r3` encodes R1=r1 and compares r2 with r3). reference/chapters/13-alu-instructions.tex:99,105,1671-1672 and assembly/chapters/04-instructions.tex list only the two-register form.
- resolution: code-wrong
- fix: Keep the manual; toolchain todo: reject a three-register operand list for CMNR..TSXR (or, if the author prefers to keep it, document the form in the assembly manual and here).
- confidence: medium
- status: merged into F-alu-1

### F-enc-9
- file: reference/chapters/08-addressing-modes.tex
- lines: 221-225
- severity: minor
- category: fact
- claim: "\mnemonic{LJMP}/\mnemonic{LJSR} are meta-instructions that lower to this same encoding with \reg{p} as the named pair."
- evidence: The box says LJMP/LJSR use Address Register Direct for the *source* pair (08:212-213), but the field encoding only describes bits 23-22. The source pair goes in AF bits 5-4 (generated/register-format-encodings.tex `LJMP a, r2`: R1=0x3, AF=01; ljmp.rs:63-140).
- resolution: manual-wrong
- fix: "... lower to this same encoding with \reg{p} in bits 23--22; their source pair, also named directly, is encoded in AF, bits 5--4."
- confidence: high
- status: accepted

### F-enc-10
- file: reference/chapters/09-shift-operations.tex
- lines: 486-487
- severity: minor
- category: contradiction
- claim: "Using shifts for multiplication/division by powers of 2 is much faster than using a multiply/divide instruction (if one existed)"
- evidence: reference/chapters/16-coprocessor-instructions.tex:1257,1289 define MULU/MULS/DIVU/DIVS on the optional maths coprocessor.
- resolution: manual-wrong
- fix: "Using shifts for multiplication or division by powers of 2 avoids a call to the optional maths coprocessor (\mnemonic{MULU}, \mnemonic{DIVU}, Chapter~\ref{ch:coprocessor})."
- confidence: high
- status: accepted

### F-enc-11
- file: reference/chapters/09-shift-operations.tex
- lines: 198-200
- severity: minor
- category: fact
- claim: "ADDI r3, #0, ASR #1  ; r3 = (r3 >> 1) = (r4 + r5) / 2"
- evidence: First principles: the 16-bit ADDR drops the 17th bit, so r4 = r5 = 0x7000 gives r3 = 0xE000, then ASR gives 0xF000 (-4096), not 0x7000. The identity holds only if r4 + r5 does not overflow.
- resolution: manual-wrong
- fix: Change the comment to "; r3 = (r4 + r5) / 2, if r4 + r5 does not overflow" (or add a note that V from the ADDR must be clear).
- confidence: high
- status: accepted

### F-enc-12
- file: reference/chapters/09-shift-operations.tex
- lines: 207-208, 222, 241-242, 256
- severity: minor
- category: fact
- claim: "Bit 15 wraps into bit 0 and is copied into the carry flag" / "Bit 0 wraps into bit 15 and is copied into the carry flag"
- evidence: For a count n > 1, C holds the last bit to wrap: original bit 16 - n for RTL, bit n - 1 for RTR (alu.rs:485-515; 09:355-356). Read literally, the bullets say C = original bit 15 / bit 0 for any count.
- resolution: manual-wrong
- fix: RTL: "The last bit to wrap from bit 15 to bit 0 (original bit 16 - n for a count of n) is copied into the carry flag." RTR: "The last bit to wrap from bit 0 to bit 15 (original bit n - 1) is copied into the carry flag." Match the LSL/LSR bullets at 09:93 and 09:125.
- confidence: high
- status: accepted

### F-enc-13
- file: reference/chapters/09-shift-operations.tex
- lines: 569-573
- severity: minor
- category: fact
- claim: "; Check if r1 << 3 would be negative" / "ADDI[S] r1, #0, LSL #3        ; r1 = r1 << 3, flags from shift" / "BRAN|NS @overflow_detected"
- evidence: "would be" implies r1 is not changed, but ADDI writes r1. NS tests N, not V, so the label name misdescribes the condition.
- resolution: manual-wrong
- fix: Use "TSOI[S] r1, #0, LSL #3   ; flags from r1 << 3, r1 unchanged" and rename the label `@shifted_negative`.
- confidence: high
- status: accepted

### F-enc-14
- file: reference/chapters/09-shift-operations.tex
- lines: 606
- severity: minor
- category: fact
- claim: "The override applies to the entire instruction, including any conditional execution"
- evidence: A false condition changes nothing, including SR (facts.md "Condition false"; write_back.rs:104-106). The sentence can be read as saying flags update even when the condition fails.
- resolution: manual-wrong
- fix: "The override selects the flag source for the whole instruction. If the condition is false, the instruction has no effect and no flags change, whatever the override."
- confidence: medium
- status: accepted

### F-data-14
- file: reference/chapters/10-condition-codes.tex
- lines: 314-316
- severity: minor
- category: fact
- claim: "A predicated instruction costs the same six cycles whether or not it executes"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/memory_access.rs:25-27 (a condition-false LOAD/STOR makes no phase-4 bus access); reference/chapters/14-memory-instructions.tex:95,264,421 (wait states extend the memory-access phase). An executed LOAD/STOR with data wait states takes longer than a skipped one, and both are subject to fetch wait states.
- resolution: manual-wrong
- fix: "A predicated instruction takes six cycles, plus instruction-fetch wait states, whether or not it executes. A skipped \mnemonic{LOAD} or \mnemonic{STOR} performs no data access, so it does not incur the data wait states it would incur when executed."
- confidence: high
- status: accepted

### F-data-15
- file: reference/chapters/10-condition-codes.tex
- lines: 92-93
- severity: minor
- category: fact
- claim: "Prefer \texttt{HI}/\texttt{LO} over \texttt{CS}/\texttt{CC} for unsigned comparisons if that is easier to remember, since their names match their meaning directly."
- evidence: reference/chapters/10-condition-codes.tex:88-91 (HI is >, LO is <=, CS is <, CC is >=). They test different relations, so neither pair can replace the other. Unsigned < and >= have no condition other than CS and CC.
- resolution: manual-wrong
- fix: "\texttt{HI}/\texttt{LO} and \texttt{CS}/\texttt{CC} are not interchangeable: use \texttt{HI} for $>$, \texttt{LO} for $\leq$, \texttt{CS} for $<$ and \texttt{CC} for $\geq$."
- confidence: high
- status: accepted

### F-data-16
- file: reference/chapters/10-condition-codes.tex
- lines: 289-302
- severity: minor
- category: fact
- claim: "If equal                 & \texttt{CMPR a, b}            & \texttt{==}" / "If negative              & \texttt{CMPI a, \#0}          & \texttt{NS}"
- evidence: facts.md Addressing modes (`a` is an address-register name, not a register-direct operand; `b` is not a register); sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:99-199. Written in assembler syntax, these rows would be rejected by the assembler.
- resolution: manual-wrong
- fix: Use real registers, for example `CMPR r1, r2` and `CMPI r1, \#0`, and write the Intent column as "If r1 $>$ r2 (unsigned)" and so on.
- confidence: high
- status: accepted

### F-data-17
- file: reference/chapters/10-condition-codes.tex
- lines: 334-337
- severity: minor
- category: fact
- claim: "; CORRECT - flags preserved\nCMPR r1, r2\nBRAN|HI @target\nADDI r3, #100         ; After the branch"
- evidence: none (internal). The "correct" version is not equivalent to the "wrong" one: the add no longer runs when the branch is taken. The way to keep both the add and the flags is to suppress the flag update (reference/chapters/05-status-register.tex:239 uses `[N]` for this).
- resolution: manual-wrong
- fix: Replace the CORRECT listing with "CMPR r1, r2 / ADDI[N] r3, #100 ; [N]: flags untouched / BRAN|HI @target".
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted

### F-sum-3
- file: reference/chapters/12-instruction-summary.tex
- lines: 190-192
- severity: minor
- category: fact
- claim: "DMA meta-instructions dispatch through the normal 6-cycle \mnemonic{COPI} path; a transfer of more than a few words then runs on into further 6-cycle slots."
- evidence: reference/chapters/16-coprocessor-instructions.tex:738-739 ($6 + 6\lceil (k + 1)/6 \rceil$; 12 cycles for up to five accesses); appendix-b-timing.tex:43-44
- resolution: manual-wrong
- fix: "DMA meta-instructions dispatch through the normal \mnemonic{COPI} path; a transfer of more than five bus accesses (more than five words for \mnemonic{DMAR}/\mnemonic{DMAW}, more than two for \mnemonic{DMAT}) runs on into further 6-cycle slots."
- confidence: high
- status: accepted

### F-alu-10
- file: reference/chapters/13-alu-instructions.tex
- lines: 54-116, 266
- severity: minor
- category: improvement
- claim: "\textbf{Assembles to:} \texttt{ORRI[S] rD, \#0, shift}. See Table~\ref{tab:alu-legal-forms} for the full family of legal forms."
- evidence: Table tab:alu-legal-forms has no SHFT row. It also lists `MOVI rD, \#imm16` only, but mov.rs:98-140 accepts `@sym` (default `.l`) and `$placeholder`. The ALU immediates reject symbols (facts.md, Default symbol-reference types), and the table does not show that difference.
- resolution: n/a
- fix: Add a row `SHFT & \texttt{SHFT rD, shift} & \opcode{25} & Writes result; default \texttt{[S]}`, and change the MOVI row to `\texttt{MOVI rD, \#imm16 | @sym | \$name}`. Alternatively, point the SHFT box at Chapter 12 rather than at this table.
- confidence: medium
- status: accepted

### F-alu-2
- file: reference/chapters/13-alu-instructions.tex
- lines: 266, 276-277, 291, 317
- severity: minor
- category: contradiction
- claim: "\textbf{AF (bits 5--4):} fixed at \texttt{10} (Shift) by the meta-instruction." vs "\textbf{Flags:} Updated from the shifter result by default."
- evidence: tc/src/parsers/opcodes/arithmetic_immediate.rs:165-166 (`status_register_update_source.unwrap_or(Shift)`) means `SHFT[A]` and `SHFT[N]` are accepted and change AF. reference/chapters/11-reading-instructions.tex:143 says "SHFT defaults to [S] instead". The bytefield (line 277) hard-codes `10`.
- resolution: manual-wrong
- fix: Replace the AF bullet with "\textbf{AF (bits 5--4):} \texttt{10} (Shift) by default; an explicit \texttt{[A]} or \texttt{[N]} suffix encodes \texttt{01} or \texttt{00}." Relabel the bytefield AF box `SUS`. If the author instead wants SHFT to have a fixed AF, the assembler must reject a suffix on SHFT, and ch. 11:143 needs the same change.
- confidence: high
- status: accepted

### F-alu-6
- file: reference/chapters/13-alu-instructions.tex
- lines: 133
- severity: minor
- category: contradiction
- claim: "Condition false        & The instruction is skipped and has no side effects. Registers and status flags are preserved."
- evidence: reference/chapters/06-exceptions.tex (sec:segment-overflow-fault) says that an instruction in the last slot "and a branch whose condition is false" still advances the PC and raises segment overflow with SR.A set. A false-condition instruction is also still traced (06-exceptions.tex:121,321). Line 149-150 of this chapter itself says the PC-wrap fault applies to every ALU instruction that does not write pl, and a false-condition instruction does not write pl.
- resolution: manual-wrong
- fix: "The instruction writes no register and no status flag. The program counter still advances, so the segment-overflow check on the program counter and trace still apply."
- confidence: high
- status: accepted

### F-alu-8
- file: reference/chapters/13-alu-instructions.tex
- lines: 154-155
- severity: minor
- category: contradiction
- claim: "If trace mode was enabled when the instruction began, an instruction-trace fault is raised after the instruction commits."
- evidence: reference/chapters/06-exceptions.tex:121,307,321,763 say that no trace fault is raised at exception level 7. The simulator does not implement that exception yet (general-todo.md item 19).
- resolution: manual-wrong
- fix: "If trace mode was enabled when the instruction began, an instruction-trace fault is raised after the instruction commits, except at exception level 7 (Section~\ref{sec:instruction-trace-fault})."
- confidence: high
- status: accepted

### F-alu-9
- file: reference/chapters/13-alu-instructions.tex
- lines: 474
- severity: minor
- category: fact
- claim: "Adding zero (\texttt{ADDI r1, \#0}) can be used to move values or test flags"
- evidence: The immediate form uses rD as both the source and the destination (lines 134, 420), so `ADDI rD, #0` cannot move a value to another register. A register-to-register copy with flags needs `ADDR rD, rS, rZ` with a zero register, and SIRC has none (facts.md, Registers; ho:21-22).
- resolution: manual-wrong
- fix: "Adding zero (\texttt{ADDI r1, \#0}) sets N and Z from \texttt{r1} and clears C and V without changing it; use \mnemonic{MOVR} to copy a register."
- confidence: high
- status: accepted

### F-mem-10
- file: reference/chapters/14-memory-instructions.tex
- lines: 509
- severity: minor
- category: contradiction
- claim: "\item \textbf{ASL} -- Arithmetic Shift Left (same data result as LSL; sets V on signed overflow)"
- evidence: 14-memory-instructions.tex:94 and :564-566: a shift performed during a STOR cannot update the status register. The STOR AF field selects the address register (:364-365).
- resolution: manual-wrong
- fix: "\item \textbf{ASL} -- Arithmetic Shift Left (same stored value as LSL; no flags are updated)"
- confidence: high
- status: accepted

### F-mem-11
- file: reference/chapters/14-memory-instructions.tex
- lines: 6-7
- severity: minor
- category: fact
- claim: "Following the load/store architecture, these are the only instructions that can access memory"
- evidence: facts.md:407-412: DMAR/DMAW/DMAT coprocessor commands transfer words to and from memory (16-coprocessor-instructions.tex; appendix-b-timing.tex:176-181 counts DMA accesses).
- resolution: manual-wrong
- fix: "these are the only processing-unit instructions that access data memory; the optional DMA coprocessor commands (Chapter~\ref{ch:coprocessor-instructions}) also transfer memory"
- confidence: medium
- status: accepted

### F-mem-12
- file: reference/chapters/14-memory-instructions.tex
- lines: 95
- severity: minor
- category: fact
- claim: "Memory instructions complete in 6 cycles when the bus access is acknowledged without wait states. Bus wait states extend the memory-access phase."
- evidence: appendix-b-timing.tex:178-181: a load or store makes two instruction-fetch accesses plus one data access, so w wait states add 3w. Ch15:97 says "plus any instruction-fetch wait states", but Ch14 mentions only the data access. The same wording is at :264-265 and :421-422.
- resolution: manual-wrong
- fix: "6 cycles with no wait states. Each wait state on either instruction fetch or on the data access adds one cycle (3w for w wait states per access)."
- confidence: high
- status: accepted

### F-mem-8
- file: reference/chapters/14-memory-instructions.tex
- lines: 21-22, 90
- severity: minor
- category: contradiction
- claim: "Condition false     & The instruction is skipped and has no side effects. Registers, memory, address registers, and status flags are preserved."
- evidence: 15-control-flow.tex:110-112 and 06-exceptions.tex:246-249: an instruction whose condition is false still raises a segment-overflow fault in the last slot of a segment when SR.A is set. The same absolute wording appears at 15-control-flow.tex:21-22 and :91.
- resolution: manual-wrong
- fix: Append "(the program-counter advance is still checked for segment overflow; see Section~\ref{sec:segment-overflow-fault})" at all three locations.
- confidence: high
- status: accepted

### F-mem-13
- file: reference/chapters/15-control-flow.tex
- lines: 41-44
- severity: minor
- category: fact
- claim: "BRAN                 & \texttt{BRAN \#disp} ... BRAN                 & \texttt{BRAN @label}"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/branching.rs:85-117 also accepts `BRAN $placeholder` / `BRSR $placeholder` (an .EQU value).
- resolution: manual-wrong
- fix: Add rows "BRAN \$name" and "BRSR \$name": "displacement from an .EQU placeholder". Alternatively, state that \#disp may be written as a placeholder.
- confidence: high
- status: accepted

### F-mem-14
- file: reference/chapters/15-control-flow.tex
- lines: 285-286
- severity: minor
- category: improvement
- claim: "Protected-mode calls must preserve the high words written to \reg{l} and \reg{p}."
- evidence: write_back.rs:226-245 writes lh = ph of the next instruction. write_back.rs:69-86 faults if any written high word differs, so in protected mode BRSR/LDEL/LJSR fault unless lh already equals ph.
- resolution: n/a
- fix: Add: "Because the link write sets lh to the current ph, a protected-mode call faults unless lh already equals ph; protected-mode code must keep l in the program segment." Repeat for LDEL (618-619) and LJSR (824-825).
- confidence: high
- status: accepted

### F-mem-9
- file: reference/chapters/15-control-flow.tex
- lines: 394-404, 432-443
- severity: minor
- category: fact
- claim: "\bitbox{4}{0x0} & \bitbox{4}{rO} & \bitbox{1}{0} & \bitbox{3}{000} & \bitbox{4}{0x0}"
- evidence: 14-memory-instructions.tex:205-207 gives LOAD's register-form R2/SO/ST/SA a rule ("architecturally undefined", expected to be ignored). The LDEA/LDEL/LJMP/LJSR register forms (here, :521-560, :654-668, :750-764) and STOR's R1 field (14-memory-instructions.tex:352, "Unused, encoded as 0") draw fixed zeros but state no rule for a non-zero value. The assembler always writes 0 (ldea.rs:121-125; store.rs:119).
- resolution: manual-wrong
- fix: Add to each register-form field list: "R2 (bits 21--18) and the shift fields (bits 13--6) are reserved and written as zero; a non-zero value is architecturally undefined." For STOR R1, use "reserved, written as zero; a non-zero value is architecturally undefined."
- confidence: medium
- status: accepted

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
- status: accepted

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
- status: accepted

### F-cop-10
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 627-630
- severity: minor
- category: fact
- claim: "Values above \imm{7} are architecturally undefined; the assembler currently masks \texttt{\#n} to four bits without diagnosing the error."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/exception.rs:148,181,224,264,297,340 (`value & 0x000F`, with a TODO to throw an error); facts.md Open question 2 (`#8`-`#15` assemble and then panic the sim). Also `#16` silently becomes `#0`.
- resolution: code-wrong
- fix: Manual stays (once the assembler is fixed, drop "the assembler currently masks..." and say the assembler rejects values outside 0-7). Toolchain todo: reject ETFR/ETTR indices outside 0-7.
- confidence: high
- status: simulator

### F-cop-11
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 617-625
- severity: minor
- category: fact
- claim: "ETTR #n, a                    ; Transfer a to saved address only" / "ETTR #n, r7                   ; Transfer r7 to saved status only"
- evidence: The parser accepts the manual's order (exception.rs:281-345), but its error hints for ETTR suggest the reversed form, which it rejects: exception.rs:306 "Try ETTR r7, #{value} instead" and the matching `ETTR a, #{value}` hint (around exception.rs:330-335).
- resolution: code-wrong
- fix: Manual stays. Toolchain todo: change the ETTR hints to "Try ETTR #{value}, r7" / "Try ETTR #{value}, a".
- confidence: high
- status: simulator

### F-cop-12
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 1214-1218
- severity: minor
- category: fact
- claim: "ANDI[N] rS, #0x00FF           ; keep only the low byte (the count)"
- evidence: `rS` is not a register name; the assembler accepts only r1-r7, sr, lh…pl (facts.md:234; instruction.rs:443-501), so this listing does not assemble.
- resolution: manual-wrong
- fix: Use a real register, e.g. `ANDI[N] r1, #0x00FF` / `ORRI[N] r1, #0x2A00` / `COPR r1`, with a comment that any register may hold the command (COPR reads it at decode, before DMAT overwrites r1).
- confidence: high
- status: accepted

### F-cop-13
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 309-321
- severity: minor
- category: fact
- claim: "link[0] = { pc, sr, current_exception_level }\n        current_exception_level = 1\n        pc = vector_table[vector]"
- evidence: 06-exceptions.tex (exception entry) and facts.md:314 / eu-execution.rs:284-290: entry also clears SR.P and SR.T and sets SR.EA; `pc` here must be the address of the instruction after the EXCP (software_exceptions.rs:58-66), which the pseudocode does not say.
- resolution: manual-wrong
- fix: "link[0] = { address of next instruction, sr, current_exception_level }; sr.P = 0; sr.T = 0; sr.EA = 1; current_exception_level = 1; pc = vector_table[vector]".
- confidence: high
- status: accepted

### F-cop-14
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 136-140
- severity: minor
- category: terminology
- claim: "pending_coprocessor_command = command_operand"
- evidence: The manual's name for this register is the coprocessor select register (CSR) (02-cpu-architecture.tex:108; 06-exceptions.tex:960); `pending_coprocessor_command` is the simulator's field name (general-todo.md item 7). Also at 16:314 ("discard pending_coprocessor_command").
- resolution: manual-wrong
- fix: "CSR = command_operand" and, at 16:314, "CSR = 0x0000   ; discard the trap".
- confidence: high
- status: accepted

### F-cop-15
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 538-548
- severity: minor
- category: fact
- claim: "sr = 0x0000\n        assert reset output for 6 cycles\n        pc = vector_table[reset_vector]"
- evidence: The Description (16:550-551) and 06-exceptions.tex reset-state table also clear the exception level and the pending hardware exceptions when reset is requested; the pseudocode omits them.
- resolution: manual-wrong
- fix: Add "current_exception_level = 0; clear pending hardware exceptions (and the L5 overrun flag)" after `sr = 0x0000`.
- confidence: high
- status: accepted

### F-cop-16
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 419
- severity: minor
- category: fact
- claim: "\textbf{Timing:} 6 cycles for the call plus a 6-cycle exception-unit dispatch slot."
- evidence: appendix-b-timing.tex:49 ("12 + idle until an interrupt that can be taken, or reset"); the WAIT entry does not say the idle time is extra.
- resolution: manual-wrong
- fix: "6 cycles for the call plus a 6-cycle exception-unit dispatch slot, then idle until an interrupt that can be taken arrives or the CPU is reset."
- confidence: high
- status: accepted

### F-cop-17
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 425-428
- severity: minor
- category: fact
- claim: "BRAN @idle_loop            ; Resume waiting if woken spuriously"
- evidence: 16-coprocessor-instructions.tex:404-409: WAIT leaves the wait state only for an interrupt that can be taken, so there is no spurious wake; the BRAN runs after every interrupt handler RETEs (the interrupt returns to the next instruction, 06-exceptions.tex:1019).
- resolution: manual-wrong
- fix: "BRAN @idle_loop            ; After the interrupt handler returns, wait again"
- confidence: high
- status: accepted

### F-cop-18
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 1241-1243
- severity: minor
- category: improvement
- claim: "allowing supervisor software to emulate the operation and return as if the coprocessor call had completed."
- evidence: 06-exceptions.tex:270-273: invalid opcode is retryable, so link register 6 holds the faulting call's own address; a handler must redirect it with ETFR/ETTR or a plain RETE faults again.
- resolution: n/a
- fix: Append: "Because the fault is retryable, the handler must advance the saved return address in link register 6 by 2 with \mnemonic{ETFR}/\mnemonic{ETTR} before \mnemonic{RETE} (Section on Invalid Opcode Fault)."
- confidence: high
- status: accepted

### F-cop-19
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 226-229
- severity: minor
- category: contradiction
- claim: "Reserved operation nibbles within an implemented coprocessor raise invalid-opcode faults"
- evidence: 16-coprocessor-instructions.tex:211-212: ID 0x0 (the processing unit, implemented) "is a no-op for every operation nibble and operand"; also 16:216-217 and 06-exceptions.tex:955-957: EU nibble 0x0 issued by software is a no-op, while the ID table (16:201) lists 0x0 only as "internal".
- resolution: manual-wrong
- fix: "Reserved operation nibbles within an implemented coprocessor other than the processing unit (ID 0x0) raise invalid-opcode faults; exception-unit operation 0x0 is a defined no-op." In the ID table write "0x1, 0x9--0xD (software); 0x0 (no-op); 0xE--0xF (internal)".
- confidence: medium
- status: accepted

### F-cop-3
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 64-65
- severity: minor
- category: contradiction
- claim: "Coprocessor-call instructions do not perform data-memory access and do not raise data bus, data bus-protection, data alignment, or effective-address segment-overflow faults."
- evidence: 16-coprocessor-instructions.tex:746-749 (DMA reads and writes "can raise data bus and bus-protection faults ... segment-overflow faults"); 06-exceptions.tex:490 ("a failed DMA bus access" is a fault raised by the call itself).
- resolution: manual-wrong
- fix: "The processing-unit part of a coprocessor call performs no data-memory access and raises no data bus, data bus-protection, data alignment, or effective-address segment-overflow fault; a DMA command it starts can raise bus, bus-protection and segment-overflow faults (Section~\ref{sec:dma-unit})."
- confidence: high
- status: accepted

### F-cop-5
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 715-719
- severity: minor
- category: fact
- claim: "Negative counts are encoded as the backward direction bit plus the positive magnitude, not as a two's-complement count field. A zero count is encoded with the direction bit clear (forward)."
- evidence: 16-coprocessor-instructions.tex:985-988 ("every bit pattern (0--7) is a legal magnitude, so there is no invalid encoding"); the operand byte with bit 5 set and magnitude 0 (for example 0x2820, 0x2920) is neither defined nor listed as reserved. The assembler never emits it (coprocessor_meta.rs:157-162).
- resolution: design
- fix: Option A (recommended): add "An operand with the direction bit set and a magnitude of 0 is reserved and architecturally undefined." Option B: "A backward count of 0 behaves as a count of 0 (no-op)", which is what the pseudocode at 16:963-972 yields.
- confidence: medium
- status: accepted
- decision: A backward DMAR/DMAW count of 0 (direction bit set, magnitude 0) is a no-op, identical to a forward count of 0.

### F-cop-7
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 1169-1171
- severity: minor
- category: fact
- claim: "\reg{r1}--\reg{r7} are clobbered; after completion they contain the final transfer window."
- evidence: The Operation pseudocode (16:1159) writes only r1..r[chunk]. For `DMAT a, l, #3`, r4-r7 are not written; for `#16` the last chunk is 2 words, so r3-r7 hold words from the previous chunk. "The final transfer window" is not well defined for a final chunk shorter than 7.
- resolution: manual-wrong
- fix: "Each chunk writes \reg{r1}--\reg{r$c$}, where $c$ is the chunk size. After completion, \reg{r1}--\reg{r$c$} hold the last chunk and the higher registers hold the words of the previous chunk, or their original values if there was none. Software should treat \reg{r1}--\reg{r7} as clobbered."
- confidence: medium
- status: accepted

### F-cop-8
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 261-262
- severity: minor
- category: fact
- claim: "0x1C00-0x1C37   & ETFR" / "0x1D00-0x1D37   & ETTR"
- evidence: 16-coprocessor-instructions.tex:268-270 and 627-630: only link-register indices 0-7 are defined; bits 7-6 are not decoded. The range 0x1C00-0x1C37 includes 0x1C08-0x1C0F, 0x1C18-0x1C1F, etc. (indices 8-15, undefined; the sim panics, eu-execution.rs:232-235), and excludes the aliases 0x1C40-0x1CFF that the text says are not distinct operations.
- resolution: manual-wrong
- fix: Replace the range with "0x1C00 | sel<<4 | n (n = 0--7)" (and 0x1D…), or footnote: "Only values with bits 3 = 0 are defined; bits 7--6 are ignored."
- confidence: high
- status: accepted

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
- status: accepted
- decision: Treat 0x27 and 0x2F as undocumented opcodes: move them out of the MOV and COP rows of Table A.3 into an 'Undocumented' footnote row, rename the Ch.13 'Short-immediate move' paragraph to 'Opcode 0x27', and in Ch.16 say opcode 0x2F is undocumented (Appendix C), not a coprocessor call.

### F-tim-10
- file: reference/chapters/appendix-b-timing.tex
- lines: 76, 243
- severity: minor
- category: fact
- claim: "\textbf{At 24 MHz (maximum rated clock):} $\approx$ 4.0 million instructions/second"
- evidence: No other chapter of any of the three manuals states a maximum clock. hardware/chapters/06-electrical-characteristics.tex:12 lists "Clock frequency range" only as a topic. reference/manual-handover.md:101-102 records that the 24 MHz figure "ha[s] no stated basis" and leaves it to the author. Ch. 2 no longer contains it, so this appendix is now the only place the rating appears. The arithmetic is correct: 24/6 = 4.0, and 528 cycles / 24 MHz = 22 us.
- resolution: design
- fix: Options: (a) move the rating to the hardware manual's electrical characteristics and cite it here; (b) drop "(maximum rated clock)" and use 24 MHz only as an example frequency, in both places. Recommend (b) until an electrical basis exists.
- confidence: medium
- status: merged into F-hw-1

### F-tim-11
- file: reference/chapters/appendix-b-timing.tex
- lines: 86-88
- severity: minor
- category: contradiction
- claim: "Privilege Violation is detected at phase 2 for a write to a privileged register field and for a supervisor-only coprocessor operation, at phase 3 for an \mnemonic{EXCP} vector below \texttt{0x60}"
- evidence: docs/reference/chapters/general-todo.md item 13 says "Appendix B says Privilege Violation is detected at phase 2" for the EXCP check, which no longer matches this text. Ch. 6 (06-exceptions.tex:280-289) and Ch. 16 (:309-313) give no phase. The simulator checks it in the exception-unit dispatch slot (eu-execution.rs:438-448; todo 13).
- resolution: unclear
- fix: Confirm phase 3 (this is reasonable, because COPR's vector comes from a register that is read at phase 2) and update todo 13 to say phase 3. Otherwise change this sentence to phase 2. Either way, have Ch. 6 Privilege Violation state the phase.
- confidence: medium
- status: accepted
- decision: Keep phase 3 for the EXCP vector-below-0x60 privilege check; Ch.6 Privilege Violation states the phase.

### F-tim-12
- file: reference/chapters/appendix-b-timing.tex
- lines: 98-101
- severity: minor
- category: fact
- claim: "The dispatch slot in progress is cancelled and its remaining phases run as no-ops, and the dispatch for the new fault begins at the next phase 0."
- evidence: The simulator resets the phase immediately (`handle_vector_fetch_bus_fault`, sirc-vm/peripheral-cpu/src/lib.rs:530-571). Known: general-todo.md item 6.
- resolution: code-wrong
- fix: No manual change; tracked by todo 6.
- confidence: high
- status: simulator

### F-tim-13
- file: reference/chapters/appendix-b-timing.tex
- lines: 88, 110-111
- severity: minor
- category: fact
- claim: "Segment Overflow is detected at phase 3." / "a skipped instruction in the last instruction slot of a segment raises a segment-overflow fault in phase 3"
- evidence: The simulator raises the PC-wrap overflow at the next instruction's fetch, not in phase 3 of the wrapping instruction (pu-execution.rs:99-107,138-148). Known: general-todo.md item 15.
- resolution: code-wrong
- fix: No manual change; tracked by todo 15.
- confidence: high
- status: simulator

### F-tim-7
- file: reference/chapters/appendix-b-timing.tex
- lines: 41, 46-49, 59-60
- severity: minor
- category: fact
- claim: "Conditional (not executed) & 6" ... "A conditional \mnemonic{DMAR}/\mnemonic{DMAW}/\mnemonic{DMAT} whose condition is false does not dispatch to the DMA unit and takes only 6 cycles."
- evidence: The false-condition rule is stated only for DMA. The COPI/COPR, EXCP/RETE/ETFR/ETTR, RSET, WAIT and maths rows give only the true-condition figure. Ch. 16 says a false condition dispatches nothing (16-coprocessor-instructions.tex:345, 417, 562), which implies 6 cycles for each.
- resolution: manual-wrong
- fix: Replace the DMA-only sentence with: "Any coprocessor call or meta-instruction whose condition is false writes nothing to the CSR, dispatches nothing, and takes 6 cycles; the coprocessor rows give the condition-true figure."
- confidence: high
- status: accepted

### F-tim-8
- file: reference/chapters/appendix-b-timing.tex
- lines: 80
- severity: minor
- category: fact
- claim: "Reset holds \texttt{RSTO} for 6 cycles before the reset-vector fetch."
- evidence: 06-exceptions.tex:895-899. RSTO stays asserted for as long as RSTI is held, and then for five more cycles after release. Six cycles is only the one-cycle-RSTI case (hardware/chapters/05-bus-timing.tex:167). A failed reset-vector fetch adds another 6-cycle hold (06-exceptions.tex:921-924).
- resolution: manual-wrong
- fix: "Reset holds \texttt{RSTO} for the cycles \texttt{RSTI} is asserted plus five more (6 cycles for a one-cycle \texttt{RSTI} or for \mnemonic{RSET}) before the reset-vector fetch; a failed reset-vector fetch repeats the hold."
- confidence: high
- status: accepted

### F-tim-9
- file: reference/chapters/appendix-b-timing.tex
- lines: 49
- severity: minor
- category: improvement
- claim: "\mnemonic{WAIT} & 12 + idle until an interrupt that can be taken, or reset"
- evidence: 16-coprocessor-instructions.tex:403-407. A request that can be taken and is already pending ends the wait at once. The row does not say that the interrupt's own 6-cycle dispatch slot then follows before the handler's first fetch.
- resolution: n/a
- fix: "12 + idle (0 if an interrupt that can be taken is already pending) + 6 for the interrupt dispatch, or until reset"
- confidence: medium
- status: accepted

### F-exc-15
- file: reference/chapters/appendix-c-undocumented.tex
- lines: 15-22
- severity: minor
- category: contradiction
- claim: "If two of those write-back targets are the same address-register pair, the final value is architecturally undefined."
- evidence: reference/chapters/15-control-flow.tex:72-78 and 14-memory-instructions.tex:76-79. Those chapters also call these cases undefined: LOAD post-increment into a half of the updated pair, STOR pre-decrement from a half of the updated pair (not a double write at all), and any auto-update of `p` competing with the PC advance. This appendix's own example `LOAD r1, (#0, p)+` (line 35) is not covered by the rule it states, and 14-memory-instructions.tex:79 points readers to this appendix for those cases.
- resolution: manual-wrong
- fix: Replace the rule with the full list from 15-control-flow.tex:72-78, or reword as: "Any of the following is architecturally undefined: two write-back targets in the same address-register pair (including the link write and the program-counter advance); a LOAD post-increment whose destination is a half of the updated pair; a STOR pre-decrement whose source is a half of the updated pair."
- confidence: high
- status: accepted

### F-sum-12
- file: reference/chapters/appendix-d-examples.tex
- lines: 98-104
- severity: minor
- category: fact
- claim: "followed by an exhaustive exercise of every \mnemonic{LJMP}/\mnemonic{LJSR} addressing form"
- evidence: examples/store-load/store-load.sasm:40-136 uses `LJMP/LJSR A, rX`, `A, #n` and plain `A` only. The assembler also accepts `LJSR (#n, A)+` and `LJSR (rX, A)+` (docs/reference/review/facts.md operand-forms table, `tc/src/parsers/opcodes/ljsr.rs`), which the program never uses, and plain `LJMP s` is reached only as the final jump to `:pass`, never as a checked step.
- resolution: manual-wrong
- fix: Replace "an exhaustive exercise of every LJMP/LJSR addressing form" with "an exercise of the register-offset and immediate-offset forms of LJMP and LJSR".
- confidence: high
- status: accepted

### F-sum-8
- file: reference/chapters/appendix-d-examples.tex
- lines: 36-39
- severity: minor
- category: fact
- claim: "deliberately triggers, in sequence, a bus fault, an alignment fault, two segment-overflow faults (...), an instruction-trace fault, a bus-protection fault, an invalid-opcode fault (which itself triggers a nested double fault), a privilege violation, and a level-five interrupt."
- evidence: examples/faults/faults.sasm:54-151. The order in the program is bus, alignment, segment overflow (EA, :84), segment overflow (PC, :108), trace (:117), bus protection (:127), privilege violation (:135), L5 interrupt (:143), invalid opcode last (:151, with the comment at :145-149 explaining why it is last). The L5 handler (:306-316) also raises a Level Five Conflict fault, which the sentence omits.
- resolution: manual-wrong
- fix: "...an instruction-trace fault, a bus-protection fault, a privilege violation, a level-five interrupt whose handler raises a second level-five request and so a level-five conflict fault, and finally an invalid-opcode fault (which itself triggers a nested double fault)."
- confidence: high
- status: accepted

### F-sum-9
- file: reference/chapters/appendix-d-examples.tex
- lines: 39-40
- severity: minor
- category: fact
- claim: "Each handler, defined from line 158 onward, increments a distinct register so a reader can confirm which faults fired and in what order."
- evidence: examples/faults/faults.sasm:172 and :267 (bus-protection and privilege handlers both increment r5 by 1); :248 and :292 (invalid opcode and double fault both r4 by 1); :303, :307, :319 (L5 conflict, L5 and trace all use r6, with different increments); :218, :231 (both segment-overflow paths use r3, increments 0x0F00 and 0x000F). The register dump also cannot show order.
- resolution: manual-wrong
- fix: "Each handler, defined from line 158 onward, adds a handler-specific amount to one of \reg{r1}--\reg{r6}, so the final register dump shows which faults fired."
- confidence: high
- status: accepted

### F-sum-13
- file: reference/chapters/appendix-e-quick-reference.tex
- lines: 102, 129-130
- severity: minor
- category: fact
- claim: "\mnemonic{RSET}   & Meta          & COPI \#0x1B00                                    & *          & *          & *          & *\textsuperscript{\ddag} & System reset (SR cleared to 0)"
- evidence: reference/chapters/11-reading-instructions.tex:103 defines `0` as "Cleared to zero" and `*` as "Updated from the instruction result"; appendix-e-quick-reference.tex:17 defines the double dagger as "updated from a named source". RSET has no source; it clears all four flags (eu-execution.rs:381).
- resolution: manual-wrong
- fix: Change the RSET row's N, Z, C, V cells to `0`, `0`, `0`, `0` with no double dagger, and shorten the footnote to "\textsuperscript{\ddag} \mnemonic{RETE} restores all four flags from the link register."
- confidence: high
- status: accepted

### F-sum-5
- file: reference/chapters/notation-and-glossary.tex
- lines: 134-138
- severity: minor
- category: fact
- claim: "In any other instruction operand (\mnemonic{MOVI}, \mnemonic{LOAD}, ..."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_immediate.rs:208,272 ("does not support symbol refs at this time" for ALU immediates); COPI, EXCP, ETFR/ETTR and the DMA/maths meta-instructions also reject symbols (docs/reference/review/facts.md, Default symbol-reference types). The same reading appears in 11-reading-instructions.tex:81-83 ("a low-word address in other instruction operands").
- resolution: manual-wrong
- fix: Replace "In any other instruction operand (...)" with "In a \mnemonic{MOVI}, \mnemonic{LOAD}, \mnemonic{STOR}, \mnemonic{LDEA}, \mnemonic{LDEL}, \mnemonic{LJMP} or \mnemonic{LJSR} operand"; add "Other instructions (ALU immediates, \mnemonic{COPI} and the coprocessor meta-instructions) do not accept a label." Change 11-reading-instructions.tex:82 "a low-word address in other instruction operands" to "a low-word address in the memory, effective-address and \mnemonic{MOVI} operands that accept one".
- confidence: high
- status: accepted

### F-sum-6
- file: reference/chapters/notation-and-glossary.tex
- lines: 153-166
- severity: minor
- category: fact
- claim: "BRAN|== @start                 ; @label defaults to a PC-relative offset for BRAN/BRSR" followed by ":level2_handler / RETE" and the ".DW" lookup data
- evidence: reference/chapters/16-coprocessor-instructions.tex:481-482 ("Executing RETE at exception level 0 ... is architecturally undefined"). When the CMPI is not equal, execution falls through from `BRAN|==` into `RETE` at level 0, then into the `.DW` slots as instructions (`.DW #10` = 0x0000000A, an ADDI r1 with condition 0xA).
- resolution: manual-wrong
- fix: Insert a terminating self-loop after the `BRAN|==` line, for example `:done` / `    BRAN @done                     ; stop here`, so the listing never falls into the handler or the data.
- confidence: high
- status: accepted

### F-sum-7
- file: reference/chapters/notation-and-glossary.tex
- lines: 154-155
- severity: minor
- category: fact
- claim: "MOVI al, @lookup               ; @label defaults to the low word for MOVI" / "LOAD r1, (#0, a)               ; r1 = lookup[0]"
- evidence: reference/chapters/03-registers.tex:17 (only sr and p have defined values after reset), so ah is undefined when the LOAD runs and the comment "r1 = lookup[0]" holds only if ah happens to be 0.
- resolution: manual-wrong
- fix: Add `    MOVI ah, @lookup.u             ; @label.u selects the high word` before the `MOVI al, @lookup` line.
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted
- decision: Point toolchain and object-file topics to the SIRC-1 Assembly Programmer's Manual; keep only the ABI (calling conventions, register usage, stack frames) as forthcoming. Apply in the preface and 03-registers:94-95.

### F-asm-26
- file: assembly/chapters/06-toolchain.tex
- lines: 95-98
- severity: nit
- category: fact
- claim: "a program that needs 64\,K words of scratch memory away from the program binary is run with: ... --segment SCRATCH:00010000:FFFF"
- evidence: A segment covers [base, base+size) (sirc-vm/peripheral-bus/src/lib.rs:45-50), so length 0xFFFF maps 65535 words, one short of 64 K. Word 0x1FFFF stays unmapped.
- resolution: manual-wrong
- fix: Use `SCRATCH:00010000:10000`, or say "just under 64 K words".
- confidence: high
- status: accepted

### F-asm-27
- file: assembly/chapters/06-toolchain.tex
- lines: 77-93
- severity: nit
- category: improvement
- claim: "sirc_vm --program-file <FILE> [--segment <SEGMENT>]... [--register-dump-file <FILE>]"
- evidence: The VM also has `-e/--enable-video` (with the `video` feature), which maps VIDEO at 0x0C0000 (sirc-vm/sirc-vm/src/main.rs:114-116,205-222). The fixed segments (PROGRAM, TERMINAL, DEBUG) are not listed anywhere in the assembly manual, so a reader choosing a --segment offset cannot avoid collisions.
- resolution: n/a
- fix: Add `--enable-video, -e` to the option list and a short table of the segments the simulator always maps.
- confidence: high
- status: accepted

### F-hw-27
- file: hardware/chapters/02-package-and-pinout.tex
- lines: 134
- severity: nit
- category: fact
- claim: "51           & BPER          & Bus protection error           \\"
- evidence: hw Ch.2 lines 132-133 (BACK, BERR say "input"); hw Ch.3 line 120 (BPER is an input)
- resolution: manual-wrong
- fix: "Bus protection error input"
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: accepted

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
- status: accepted

### F-data-13
- file: reference/chapters/05-status-register.tex
- lines: 54
- severity: nit
- category: fact
- claim: "\item[Tested by:] Equal (\texttt{==}), Not Equal (\texttt{!=}) condition codes"
- evidence: reference/chapters/10-condition-codes.tex:33-38 (HI, LO, >> and <= also test Z). The N, C and V entries list their compound users; Z does not.
- resolution: manual-wrong
- fix: "\item[Tested by:] Equal (\texttt{==}), Not Equal (\texttt{!=}), and the compound conditions \texttt{HI}, \texttt{LO}, \texttt{>>}, \texttt{<=}"
- confidence: high
- status: accepted

### F-exc-17
- file: reference/chapters/06-exceptions.tex
- lines: 987-988
- severity: nit
- category: fact
- claim: "(for example, \texttt{0x00} is priority 7 and \texttt{0x40} is priority 3)"
- evidence: reference/chapters/06-exceptions.tex:1004 (vector 0x00 is reset, priority "n/a").
- resolution: manual-wrong
- fix: "(for example, fault vector \texttt{0x05} is priority 7 and \texttt{0x40} is priority 3)"
- confidence: high
- status: accepted

### F-exc-18
- file: reference/chapters/06-exceptions.tex
- lines: 1020
- severity: nit
- category: contradiction
- claim: "\texttt{0x11}--\texttt{0x5F} & \addr{0x0022}--\addr{0x00BF} & n/a               & Reserved"
- evidence: reference/chapters/06-exceptions.tex:88 (the list form says "excluding 0x20, 0x30, 0x40, 0x50"); rows 1016-1019 assign those vectors.
- resolution: manual-wrong
- fix: Change the Vector cell to "\texttt{0x11}--\texttt{0x5F} (excluding \texttt{0x20}, \texttt{0x30}, \texttt{0x40}, \texttt{0x50})", or add a table note.
- confidence: high
- status: accepted

### F-exc-19
- file: reference/chapters/06-exceptions.tex
- lines: 381-382
- severity: nit
- category: fact
- claim: "\mnemonic{ETFR} only transfers the return address and status register of link registers 6 and 7"
- evidence: reference/chapters/16-coprocessor-instructions.tex:263-265 (ETFR takes any link-register index 0-7). The sentence reads as though ETFR is limited to registers 6 and 7.
- resolution: manual-wrong
- fix: "For link registers 6 and 7, as for every link register, \mnemonic{ETFR} transfers only the return address and status register"
- confidence: medium
- status: accepted

### F-exc-20
- file: reference/chapters/06-exceptions.tex
- lines: 698-706
- severity: nit
- category: style
- claim: "\item \textbf{\texttt{0x01}:} Bus fault"
- evidence: reference/chapters/06-exceptions.tex:683-688 (fault-type fields are 4 bits wide). Also "always \texttt{0x08}" at line 683.
- resolution: n/a
- fix: Write the 4-bit fault codes as one hex digit (\texttt{0x1}--\texttt{0x9}), or note that they are the vector numbers.
- confidence: medium

## Code-wrong findings (manual is canonical)
- status: accepted

### F-enc-17
- file: reference/chapters/07-instruction-formats.tex
- lines: 172
- severity: nit
- category: terminology
- claim: "\texttt{0x5} = Negative (<0)"
- evidence: The condition table (07:282) names 0x5 "Negative Set (N = 1)"; "<0" reads like signed less-than, which is 0xC (`<<`, N != V) and differs when V = 1.
- resolution: n/a
- fix: "\texttt{0x5} = Negative Set (NS, N = 1)"
- confidence: high
- status: accepted

### F-enc-18
- file: reference/chapters/09-shift-operations.tex
- lines: 588
- severity: nit
- category: fact
- claim: "BRAN|>= @r1_was_greater"
- evidence: `>=` (code 0xB, N = V) is greater-or-equal, so the branch is also taken when r1 = r2.
- resolution: manual-wrong
- fix: "BRAN|>= @r1_was_greater_or_equal"
- confidence: high
- status: accepted

### F-data-18
- file: reference/chapters/10-condition-codes.tex
- lines: 55, 140, 340
- severity: nit
- category: terminology
- claim: "Typically used after \mnemonic{CMP} to test equality."
- evidence: facts.md Opcode map (the mnemonics are CMPI and CMPR; there is no CMP)
- resolution: n/a
- fix: Write "\mnemonic{CMPI}/\mnemonic{CMPR}" (or "a compare instruction") at lines 55, 59-64, 140 and 340.
- confidence: high
- status: accepted

### F-sum-14
- file: reference/chapters/12-instruction-summary.tex
- lines: 101
- severity: nit
- category: fact
- claim: "See Table~\ref{tab:flag-effect-symbols} for the flag-effect symbol definitions."
- evidence: reference/chapters/12-instruction-summary.tex:28-94. The Flags column uses "NZCV", "NZ (C,V=0)" and "--", none of which are symbols from Table tab:flag-effect-symbols (11-reading-instructions.tex:100-107); "--" is not "-".
- resolution: manual-wrong
- fix: Either drop the sentence, or replace it with "The Flags column lists the flags an instruction updates with [A]; see Appendix~\ref{appendix:quick-reference}, Table~\ref{tab:mnemonic-index}, for the per-flag effect using the symbols of Table~\ref{tab:flag-effect-symbols}."
- confidence: high
- status: accepted

### F-mem-15
- file: reference/chapters/15-control-flow.tex
- lines: 674-675
- severity: nit
- category: fact
- claim: "\reg{p} must not be used with an auto-update form."
- evidence: LJMP has no auto-update form (15-control-flow.tex:46-48; ljmp.rs:75-121).
- resolution: manual-wrong
- fix: Delete the sentence from the LJMP src field.
- confidence: high
- status: accepted

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
- status: accepted

### F-cop-20
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 963-972
- severity: nit
- category: fact
- claim: "for i in 1..count:"
- evidence: `1..count` is exclusive in Rust-style notation (it would transfer count-1 words); the same at 16:1065. The DMAT pseudocode (16:1157) uses `remaining` without initialising it.
- resolution: manual-wrong
- fix: "for i in 1 to count (inclusive):" and add "remaining = n" as the first DMAT line.
- confidence: high
- status: accepted

### F-cop-9
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 607-611
- severity: nit
- category: fact
- claim: "bits 7--6 are not decoded, bits 5--4 (\texttt{x}) select the register"
- evidence: In `0x1Cxy`, the hex digit x is bits 7-4, not bits 5-4, and y is bits 3-0. With bits 7-6 written as 0 the digit is 0-3, but the labelling is wrong as stated.
- resolution: manual-wrong
- fix: "the high hex digit \texttt{x} holds bits 7--4: bits 7--6 are not decoded and written as zero, and bits 5--4 select the register..."
- confidence: high
- status: accepted

### F-tim-14
- file: reference/chapters/appendix-b-timing.tex
- lines: 213-219
- severity: nit
- category: fact
- claim: "LOAD r1, (#0, a)+          ; Same result, fewer cycles"
- evidence: `ADDI al, #1` uses the default `[A]` and so rewrites N/Z/C/V (13-alu-instructions.tex:173). LOAD with post-increment changes no flags (facts.md:171). The register results match; the flags do not.
- resolution: manual-wrong
- fix: "; Same register result in one instruction (6 cycles saved); flags are not changed"
- confidence: high
- status: accepted

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
- status: accepted

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
- status: accepted

### F-sum-15
- file: reference/chapters/appendix-e-quick-reference.tex
- lines: 107
- severity: nit
- category: fact
- claim: "\mnemonic{SHFT}   & Meta          & ORRI[S] rD,\#0"
- evidence: reference/chapters/12-instruction-summary.tex:147 gives "ORRI[S] rD, \#0, shift"; the shift operand is what selects the short-immediate opcode 0x25 (docs/reference/review/facts.md, operand forms: SHFT -> ORRI short with imm 0).
- resolution: manual-wrong
- fix: "ORRI[S] rD,\#0,shift"
- confidence: high
- status: accepted

