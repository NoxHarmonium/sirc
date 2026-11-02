# Fact review: G7 (coprocessor and meta-instructions)

Reviewer tag: cop. Chapters: `reference/chapters/16-coprocessor-instructions.tex`, `reference/chapters/17-meta-instructions.tex`.

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
