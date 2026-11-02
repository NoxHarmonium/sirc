# Cold read: hardware reference manual

Reviewer tag: CRH. Reader: a board designer wiring the CPU to memory and devices. Source: PDF text plus figures at 80 dpi.

### F-CRH-1
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 5.5 (CLKI) / 3.5
- severity: minor
- category: structure
- claim: "Each rising edge of CLKI advances the CPU by one execution phase"
- evidence: none
- resolution: n/a
- fix: "Phase", "execution phase", "phase 0", "slot", "dispatch slot", "coprocessor slot" and "pipeline phases" are used from Chapter 3 onward but defined nowhere in this manual. Add a short "Execution phases" section to Chapter 4 (or 1) with a table of the six phases (IF high, IF low, Decode, Execute/EA, Memory, Writeback), saying which have bus access, and point to the Architecture manual chapter that defines them. Link from 3.5.
- confidence: high

### F-CRH-2
- file: hardware/chapters/05-bus-timing.tex
- lines: 5.1 intro
- severity: minor
- category: structure
- claim: "IF high and IF low are the high-word and low-word instruction-fetch cycles, and Decode, Execute, Memory, and Writeback are the pipeline phases of the same names"
- evidence: none
- resolution: n/a
- fix: The figure-label legend is one dense paragraph of about twenty definitions, placed before the first figure and read again with every figure. Turn it into a two-column table (label, meaning) and say that the manual defines "pipeline phases" only by this list. A reader should not need the Architecture manual to decode a hardware figure.
- confidence: high

### F-CRH-3
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 3.2-3.19
- severity: minor
- category: structure
- claim: "Direction: Output / Signal Type: Active High"
- evidence: none
- resolution: n/a
- fix: 18 signals are described as per-signal prose lists with no summary. Add one table at the start of Chapter 3 or end of Chapter 2: signal, direction, active level, three-state/driven when idle, sampled when. A designer wants this beside the pinout. Table 2.1 already has name and description but omits direction and polarity, and there is no overbar convention stated (the figures use overbars, Table 2.1 and the prose do not).
- confidence: high

### F-CRH-4
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 3.16-3.17 (IRQ, NMI)
- severity: minor
- category: structure
- claim: "NMI ... the CPU samples the Level 5 hardware exception line"
- evidence: none
- resolution: n/a
- fix: "Level 5", "pending hardware-exceptions register", "exception level 7", "Level Five Hardware Exception Conflict", "E1-E4", "T flag" and "P bit" are used with no definition or chapter pointer (only a general "see Exception and Interrupt Handling chapter"). IRQ priority between IRQ1-IRQ4 is never stated. Add a small table mapping each pin to its level/priority and vector number, or say explicitly that the table is in the Architecture manual and name it.
- confidence: high

### F-CRH-5
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 3.17
- severity: minor
- category: structure
- claim: "An NMI held asserted across a reset does not fire afterward, because reset does not clear the edge detector's last sampled level"
- evidence: none
- resolution: n/a
- fix: Needed two reads. The NMI section has five overlapping rules (edge semantics, reset interaction, conflict while running, lost edge while pending, release for one CLKI cycle) in two paragraphs and the second paragraph restates the first. Reorganise as a short list of rules, or a state table, and drop the repeated "must be released and reasserted". Also the closing "Level 5 handlers must therefore execute quickly" does not follow from the preceding sentence.
- confidence: medium

### F-CRH-6
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 3.18-3.19
- severity: minor
- category: structure
- claim: "an RSTI assertion of n cycles gives n + 5 cycles of RSTO (six for a one-cycle assertion)"
- evidence: none
- resolution: n/a
- fix: The reset rules are repeated near verbatim in 3.18, 3.19, Section 5.2 and (partly) 4.5. Reset-hold arithmetic, "software reset", "failed double-fault vector fetch" and "reset-vector fetch retry" are described in four places with slightly different wording. Keep one authoritative statement (3.18 or a new "Reset" section) and have the others point to it. The reader also cannot tell at 3.18 how RSTO relates to RSTI at the first edge (same cycle? next?) without reading Figure 5.9.
- confidence: medium

### F-CRH-7
- file: hardware/chapters/04-bus-interface.tex
- lines: 4.1
- severity: minor
- category: structure
- claim: "A no-wait-state bus access assumes that the selected external device can assert BACK and, for reads, drive valid data during the same CLKI interval in which BAS is asserted"
- evidence: none
- resolution: n/a
- fix: This is the most important thing a memory designer needs and it is buried in one sentence, with no statement of which clock edge the CPU changes outputs on, which edge it samples BACK, and how BAS is asserted relative to the edge (3.7 says BAS asserts after address is valid but does not say how long after the edge). The text says electrical timing is unspecified, but the logical edge relationships (outputs change after rising edge N, inputs sampled at rising edge N+1) can and should be stated explicitly in one numbered "bus cycle sequence" list. Compare "the CPU accepts the transfer on the next rising edge" with the figures, where BACK is already low across the whole cycle: say so.
- confidence: high

### F-CRH-8
- file: hardware/chapters/05-bus-timing.tex
- lines: Figures 5.1-5.10
- severity: minor
- category: structure
- claim: "Each labeled interval represents one CLKI cycle as observed between rising edges."
- evidence: figure pages 15-19
- resolution: n/a
- fix: At 80 dpi the figures are legible but small; the waveform lines for CLKI and the phase bands are not aligned with the label boundaries by any marked reference edge. No figure marks where the CPU samples BACK or data (a vertical line or arrow on the rising edge). Add sample-point markers and, for active-low signals, either state once that a low line means asserted or mark it on the line. Fig 5.6 uses a row named "Case" with no explanation in the text; Fig 5.6 also has BERR and BPER cases but never shows the cycle after (fault entry). Fig 5.4 and 5.9 use a 2n/2n+1 label with no statement of the vector-to-address mapping beyond the legend ("vector n at word addresses 2n and 2n+1"); a table of vector numbers would help.
- confidence: medium

### F-CRH-9
- file: hardware/chapters/05-bus-timing.tex
- lines: Figure 5.5
- severity: minor
- category: structure
- claim: "Wait-State Insertion when No Bus Response Is Asserted"
- evidence: figure page 17
- resolution: n/a
- fix: The figure labels the first cycle "req" and shows one wait cycle, but the DR band and A0-A23 band continue unchanged and no figure shows a wait state on a write (D0-D15 held by CPU), on an instruction fetch, or two wait states. The text promises wait states for "slower synchronous memory" but a designer wiring a slow ROM or a 2-wait-state SRAM sees one example only. Add a write-with-wait figure and a typical memory-select-to-BACK wiring example.
- confidence: medium

### F-CRH-10
- file: hardware/chapters/05-bus-timing.tex
- lines: 5.2, Figure 5.10
- severity: minor
- category: structure
- claim: "The figure holds IRQ1-IRQ4 asserted throughout because the requesting device must hold the line asserted until its handler acknowledges it."
- evidence: figure page 19
- resolution: n/a
- fix: The figure is described as showing "latched before an instruction boundary" but shows the IRQ line already asserted from the left edge of the figure, so the latch cycle and the dispatch gap cannot be seen; the "Prev WB" cycle precedes EV fetch directly. Also "acknowledges" is never defined: how does a handler acknowledge a device (a bus write to the device)? Say so, since the hardware designer must provide that mechanism. Show the IRQ assertion edge in the diagram, and the deassertion after acknowledge.
- confidence: medium

### F-CRH-11
- file: hardware/chapters/04-bus-interface.tex
- lines: 4.4
- severity: minor
- category: structure
- claim: "Phase 0 of the DMA slot is a setup cycle with no bus access"
- evidence: none
- resolution: n/a
- fix: "DMA slot", "DMA continuation slot", "coprocessor dispatch slot", "chunk" and "maths slot" are used (also in 3.6 and Table 4.1) before any definition; the reader learns the DMA coprocessor is "optional" only here, and Chapter 2 does not say whether a pin changes for it. Add a one-paragraph definition of coprocessor slots in Chapter 4 with a pointer to the Architecture chapter, and state in Chapter 1 that the DMA coprocessor is optional. Fig 4.1 also shows the Setup cycle with BAT "None" and then two trailing "None" cycles with no label.
- confidence: medium

### F-CRH-12
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 3.12
- severity: minor
- category: structure
- claim: "if a memory controller knows that the DMA coprocessor will be reading more than one sequential address, it can prefetch the next address."
- evidence: none
- resolution: n/a
- fix: The BAT table lists "DMA Coprocessor Read Burst" but each DMA access is an ordinary single-word cycle (4.4), so "burst" misleads: the reader expects a multi-word cycle with a single address phase. Say in the table note that burst means "one of a series of consecutive single-word cycles". Also say what "No I/O" (000) means while BAS is asserted (can it occur?) and what a device should do on Reserved (111).
- confidence: medium

### F-CRH-13
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 3.14, 3.15
- severity: minor
- category: structure
- claim: "Halt latency is at most six cycles."
- evidence: none
- resolution: n/a
- fix: There is no HALT acknowledge output, so a board designer cannot tell from the pins that the CPU has stopped, apart from BAS going idle and SYNC low. State that explicitly and say how to detect halted state. The HALT paragraph is a single long block of seven rules; a bullet list would help. TRCE: "the instruction being started is traced" is unclear to a hardware reader; say what appears on the pins (a trace fault vector fetch, BAT 100) and whether TRCE must be held for the whole instruction or only sampled once.
- confidence: medium

### F-CRH-14
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 3.3 and Chapter 6
- severity: minor
- category: structure
- claim: "These pins provide the 5 V power that the CPU requires to function."
- evidence: none
- resolution: n/a
- fix: The power pins appear in the middle of Chapter 3 between address and data bus, and the only electrical value in the manual is in 3.3 and again in Chapter 6. Move or cross-reference: a designer looks for VCC tolerance in the electrical chapter, and for decoupling and which VCC serves which pin group (VCC1 is pin 9 beside the address pins, VCC2 beside the data pins; GND1/GND2 likewise). Say whether both must be connected, and whether the two supplies may differ.
- confidence: low

### F-CRH-15
- file: hardware/chapters/06-electrical-characteristics.tex
- lines: whole chapter
- severity: minor
- category: structure
- claim: "Until then, designs must not assume any value for these characteristics."
- evidence: none
- resolution: n/a
- fix: A board designer cannot design to this, and the Preface says to see Chapter 6 for what is missing, yet the chapter body is a list of absent items. Say in the first paragraph that the chapter is a placeholder, say whether CMOS/TTL-compatible levels may be assumed, whether inputs have pull-ups (what an unconnected IRQ or HALT pin does), and what states the pins take during power-up before RSTI. Unconnected-input behaviour is a question every designer asks and a no-values caveat does not cover it (it concerns what to do with unused pins, not a numeric value).
- confidence: medium

### F-CRH-16
- file: hardware/chapters/01-introduction.tex
- lines: 1.1-1.3
- severity: minor
- category: structure
- claim: "This manual covers the concrete silicon: the 64-pin package, what each signal does, the bus protocol"
- evidence: none
- resolution: n/a
- fix: Chapter 1 never tells the reader how to use the manual (read order, where the typical wiring question is answered) and gives no block-level picture of the CPU and its bus. The Table 1.2 pin counts (10 bus control, 7 interrupts) should be reconciled with the group names in Chapter 3 order, and Table 1.1 duplicates the README and Preface text. Merge Preface "About this manual" and 1.1/1.2, which repeat the same scope statement three times.
- confidence: medium

### F-CRH-17
- file: hardware/chapters/02-package-and-pinout.tex
- lines: 2.1
- severity: minor
- category: structure
- claim: "The SIRC-1 is provided in a 64-pin dual in-line package (DIP-64)."
- evidence: none
- resolution: n/a
- fix: Chapter 2 opens with one sentence and a figure; it does not state pin 1 orientation (notch location), pin pitch or the section purpose. The figure and Table 2.1 carry the same information; the table adds only descriptions already in Chapter 3. Note the pins are not grouped logically (VCC1 sits among address pins, TRCE/HALT/PROT between BAT and IRQ), so a one-line note on placement for layout would help.
- confidence: low

### F-CRH-18
- file: hardware/chapters/04-bus-interface.tex
- lines: 4.2, 4.3
- severity: minor
- category: structure
- claim: "No response | Inserts a wait state."
- evidence: none
- resolution: n/a
- fix: Tables 4.2 and 4.3 and Section 3.7-3.11 overlap heavily (BACK/BERR/BPER semantics are given three times, plus Table 4.1 repeats 3.x per signal). Table 4.3 ("External Bus Cycle Ownership") adds little beyond Table 4.1. Merge 4.2 and 4.3 into one table of cycle types and responses, and keep per-signal prose in Chapter 3 shorter.
- confidence: medium

### F-CRH-19
- file: hardware/chapters/04-bus-interface.tex
- lines: 4.1
- severity: minor
- category: structure
- claim: "The value of A0-A23 while idle is architecturally undefined."
- evidence: none
- resolution: n/a
- fix: A designer with a bus-wired device asks: is the bus width always 16 bits, is there byte addressing (A0 is a word or byte address, since vector n is at "2n and 2n+1" word addresses)? The manual never says whether addresses are word addresses and that there are no byte strobes; this is implied only by the vector and DMA descriptions. State it in 3.2 (and that 24 address bits give 16M words).
- confidence: high

### F-CRH-20
- file: hardware/chapters/04-bus-interface.tex
- lines: 4.5
- severity: minor
- category: structure
- claim: "BACK, BERR, BPER | Sampled while BAS is asserted."
- evidence: none
- resolution: n/a
- fix: Table 4.4 restates the sampling rules already written out in 3.14-3.18, in a condensed form with slight differences (HALT, IRQ, NMI). The table is the right device but the prose in Chapter 3 should then shrink to a pointer, otherwise a reader has to compare the two to find which one is the authoritative statement.
- confidence: medium

## Improvements

### F-CRH-21
- file: hardware/chapters/05-bus-timing.tex
- lines: new
- severity: minor
- category: improvement
- claim: "none"
- evidence: none
- resolution: n/a
- fix: Add a timing figure for a write with one wait state and a figure for a BERR on an instruction fetch followed by the bus fault vector fetch (BAT 100, PROT low), plus a figure for a failed vector fetch (double fault). A designer's chief worry is the abort path, and Fig 5.6 shows only the abort cycle.
- confidence: medium

### F-CRH-22
- file: hardware/chapters/04-bus-interface.tex
- lines: new
- severity: minor
- category: improvement
- claim: "none"
- evidence: none
- resolution: n/a
- fix: Add a worked example: a minimal system (ROM, RAM, one device) with address decode, BACK generation (the "tie BACK low" case from 3.9, a one-wait-state counter), and BERR on unmapped addresses. This answers the first question this reader has and none of the text does.
- confidence: high

### F-CRH-23
- file: hardware/chapters/03-signal-descriptions.tex
- lines: new
- severity: minor
- category: improvement
- claim: "none"
- evidence: none
- resolution: n/a
- fix: Add a one-page signal summary card (name, pin, direction, level, when valid, when sampled) and a table of instruction-to-bus-cycle counts (how many bus cycles for a fetch, load, store, vector fetch, DMA), so memory designers can compute bandwidth without reading the Architecture manual.
- confidence: medium

### F-CRH-24
- file: hardware/chapters/05-bus-timing.tex
- lines: new
- severity: minor
- category: improvement
- claim: "none"
- evidence: none
- resolution: n/a
- fix: Add a protection-wiring example: a decode table of BAT x PROT x address-region giving BERR/BPER/BACK, expanding the single sentence in 3.12 (BAT 010, PROT 1, protected region). Add a glossary or index (only a contents and list of tables/figures exist); an index of signals would help a reader landing on "BAS" in a figure.
- confidence: medium

### F-CRH-25
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 3.14-3.19
- severity: minor
- category: improvement
- claim: "none"
- evidence: none
- resolution: n/a
- fix: Move CPU-internal behaviour (edge detector, pending register, level selection, exception-level rules, T flag, E1-E4) out of this manual into the Architecture manual, leaving only what is on the pins plus a pointer. Add a power-up and reset sequence timing figure that includes a software reset (RSET) and a failed reset-vector fetch retry, which are described but not drawn.
- confidence: medium

## Summary

Top three items: F-CRH-7 (the bus-cycle edge relationships, the sample and drive points, are never set out in one place, and no figure marks the sample edge); F-CRH-1 with F-CRH-2 (phase/slot vocabulary used throughout the figures and signal text but defined nowhere in this manual); F-CRH-22 (no worked wiring example, so the designer's main question goes unanswered).

Verdict: the manual reads as a competent but draft-state reference, with its pins and protocol described accurately but unevenly organised, repetitive across Chapters 3, 4 and 5, and with an electrical chapter that is only a placeholder.
