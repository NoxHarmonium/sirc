# Fact-check findings: Appendix B (timing), reviewer tag `tim`

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
