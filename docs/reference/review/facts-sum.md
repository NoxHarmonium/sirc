# Fact-check findings: G8 (summaries and appendices)

Reviewer tag: sum. Chapters: 11-reading-instructions, 12-instruction-summary, appendix-a-opcode-map,
appendix-d-examples, appendix-e-quick-reference, notation-and-glossary.

Checked and found consistent (no finding): the 64 hex/binary pairs in Table tab:opcode-map; the opcode-bit
decoding rules for 0x10-0x1F; the 62/2 documented/undocumented count; meta-instruction lowerings
(NOOP = ADDI[N] r1,#0 per `tc/src/parsers/opcodes/meta.rs:72-84`, EXCP/WAIT/RETE/RSET/ETFR/ETTR/DMA/maths command
words); the BAT table; the SR bit table against `05-status-register.tex`; the condition-code spellings; the
`.DB/.DW/.DQ` byte layouts (`tc/src/data/object.rs:71-79`); vector 0x40 = word 0x0080 = level 2 (terminal device
asserts interrupt bit 0x2, `sirc-vm/device-terminal/src/lib.rs:149`); every cross-referenced table label in
Table tab:table-locations exists; Appendix D line references for byte-sieve, faults handlers, hardware-exception,
software-exception and store-load.

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
