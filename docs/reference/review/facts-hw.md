# Hardware manual technical review (G10, tag hw)

Scope: `docs/hardware/chapters/` preface, 01-06. Checked against `docs/reference/review/facts.md` (Hardware, Exceptions),
`docs/reference/chapters/06-exceptions.tex`, `16-coprocessor-instructions.tex`, `appendix-b-timing.tex`,
`sirc-vm/peripheral-cpu/src/lib.rs` and `sirc-vm/peripheral-bus/`. Simulator gaps cite the number in
`docs/reference/chapters/general-todo.md`.

Checked and consistent (no finding): the pin table and the DIP figure agree pin for pin (1-64); the signal-group
counts sum to 64 (24+16+10+7+2+1+4); the BAT table matches `06-exceptions.tex:671-680`, Appendix E and
`bus/src/device.rs:15-44`; the response priority BERR > BPER > BACK matches `cpu/src/lib.rs:496-504`; the
one-cycle-RSTI reset diagram (6 RSTO cycles, then RV high/low) matches `06-exceptions.tex:895-899` and
`bus/src/reset_unit.rs:19-30`; the 6-phase ALU/LOAD diagrams and the 2w/3w wait-state costs match Appendix B; the supply
(5 V +/- 5%) agrees between Ch.3 and Ch.6.

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
