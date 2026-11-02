# Findings: G3 (exceptions and undocumented behaviour), reviewer tag `exc`

Chapters: `reference/chapters/06-exceptions.tex`, `reference/chapters/appendix-c-undocumented.tex`.
Code-wrong findings cite the matching `docs/reference/chapters/general-todo.md` item as "todo N".

## Manual findings

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
