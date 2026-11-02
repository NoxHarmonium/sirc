# Continuity Re-Check (Phase 4) -- con2

Scope: whole SIRC-1 set after the 41 chapter editors applied `triaged.md`. Checked the author
decisions in `triaged.md` against every chapter that restates them, the cross-manual topics,
`\ref`/`\label`, includes, listings and STYLE.md terminology. No claim was checked against code.
Findings from `consistency.md` that `triaged.md` resolved are not repeated. The one exception is
F-con2-13: its fix was accepted but never applied.

Mechanical checks (all three manuals): every `\ref` target has a `\label`, no `\label` is
duplicated, every `\input` in each `main.tex` exists and every chapter file is included,
`lstlisting` and `instructionbox` environments are balanced, and no listing contains a LaTeX
command. The three `main.log` files report no undefined or multiply defined references. No
`latex` findings.

---

## Architecture manual: internal contradictions

### F-con2-1
- file: reference/chapters/06-exceptions.tex
- lines: 320-326, 893
- severity: blocker
- category: contradiction
- claim: "The recorded trap is held in the CSR, so it is entered before any pending hardware interrupt."
- evidence: reference/chapters/06-exceptions.tex:492-493 ("Hardware interrupts, software exceptions and instruction trace faults are recognized only at an instruction boundary, and only when the CSR is empty"); 06-exceptions.tex:1004-1006; 02-cpu-architecture.tex:118-123 ("The exception coprocessor never overwrites a pending command. It keeps its own record of the exception it wants to service"); 16-coprocessor-instructions.tex:58 (Fault side effects row: "A command waiting in the CSR is never discarded by an interrupt or an instruction trace fault")
- resolution: design
- fix: The decision for F-exc (traced EXCP, triaged.md:714) put the recorded trap in the CSR. That conflicts with three rules stated elsewhere. (1) The trace fault is taken while the CSR is not empty, but trace faults are selected only when the CSR is empty. (2) The trace handler's own ETFR/ETTR/RETE write the CSR, and that would overwrite the held trap. (3) The exception unit is said to keep its own record and never load a trap over a pending command. Options: (a) recommended: hold the trap in the exception unit's own pending-software-exception record, not the CSR. Have that record outrank pending hardware interrupts when the level returns to 0, and change "held in the CSR" at 06-exceptions.tex:325 and :893 to "held by the exception unit". (b) Keep the CSR wording and add an explicit exception to 06-exceptions.tex:492, 1004 and 02-cpu-architecture.tex:118, saying how the trace handler's own coprocessor calls avoid overwriting it.
- confidence: high

### F-con2-2
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: 795-796
- severity: blocker
- category: contradiction
- claim: "The fault metadata is captured as for any other segment-overflow fault, with the bus access type of the DMA access whose address overflowed (or, for the advance, of the last access performed)."
- evidence: reference/chapters/16-coprocessor-instructions.tex:787-790 (decision triaged.md:1294: "the fault records the \reg{a} advance (DMA read type) if it wraps, otherwise the \reg{l} advance (DMA write type)"). For DMAT the last access performed in a chunk is always a write, so "of the last access performed" gives DMA write (110) where the decision gives DMA read (101) for an a-advance wrap.
- resolution: manual-wrong
- fix: "...with the bus access type of the DMA access whose address overflowed. For a forward \mnemonic{DMAR} or \mnemonic{DMAW} advance it is the type of that command's accesses; for a \mnemonic{DMAT} advance it is the type given in the list above."
- confidence: high

### F-con2-3
- file: reference/chapters/06-exceptions.tex
- lines: 294-296
- severity: minor
- category: contradiction
- claim: "The check is made at phase 2 for a write to a privileged register field and for a supervisor-only coprocessor operation, and at phase 3 for an \mnemonic{EXCP} vector below \texttt{0x60}."
- evidence: reference/chapters/appendix-b-timing.tex:92-94 ("...and at phase 5 for an address-pair write that would change a high word"). The list at 06-exceptions.tex:287-292 includes case 2, the address-register-pair write-back, but the phase sentence gives no phase for it.
- resolution: manual-wrong
- fix: Append ", and at phase 5 for an address-register-pair write-back that would change a high word" before the full stop.
- confidence: high

### F-con2-4
- file: reference/chapters/12-instruction-summary.tex
- lines: 195
- severity: minor
- category: contradiction
- claim: "\item \mnemonic{WAIT}, DMA transfers and maths-unit operations take longer than 12 cycles."
- evidence: reference/chapters/16-coprocessor-instructions.tex:753-754 ("12 cycles for up to five accesses ... A zero count needs only the setup cycle and takes 12 cycles"); 16-coprocessor-instructions.tex:1026, 1126, 1217; appendix-b-timing.tex:46 (maths "At least 12")
- resolution: manual-wrong
- fix: "\item \mnemonic{WAIT}, DMA transfers and maths-unit operations can take longer than 12 cycles: a \mnemonic{WAIT} idles until woken, a DMA transfer of more than five accesses runs into further slots, and a maths operation may add implementation-defined cycles."
- confidence: high

### F-con2-5
- file: reference/chapters/appendix-b-timing.tex
- lines: 184-190
- severity: minor
- category: contradiction
- claim: "$4w$ cycles to an accepted \mnemonic{EXCP}, and $2w$ cycles to the other exception-unit meta-instructions."
- evidence: reference/chapters/appendix-b-timing.tex:49 (RSET = 6 call + 6 RSTO hold + 6 reset-vector fetch); 06-exceptions.tex:929-936 (the reset-vector fetch reads two words); the same paragraph (line 185) lists reset among dispatches that "add two exception-vector-fetch accesses". RSET is an exception-unit meta-instruction with 4 accesses, not 2.
- resolution: manual-wrong
- fix: "$4w$ cycles to an accepted \mnemonic{EXCP} or to \mnemonic{RSET}, and $2w$ cycles to the other exception-unit meta-instructions."
- confidence: high

### F-con2-6
- file: reference/chapters/appendix-b-timing.tex
- lines: 104-107
- severity: minor
- category: contradiction
- claim: "A bus or protection fault during an exception-vector fetch is handled the same way as one during an instruction fetch."
- evidence: reference/chapters/06-exceptions.tex:782-787 (a failed fault-vector fetch escalates to a double fault; a failed double-fault vector fetch resets) and 06-exceptions.tex:942-945 (a failed reset-vector fetch raises no fault and repeats the RSTO hold). The restatement drops all three exceptions.
- resolution: manual-wrong
- fix: Insert after the sentence: "The exceptions are a failed fault-vector fetch, which raises a double fault; a failed double-fault vector fetch, which resets the CPU; and a failed reset-vector fetch, which raises no fault and repeats the reset-output hold (Chapter~\ref{ch:exceptions})."
- confidence: high

### F-con2-7
- file: reference/chapters/02-cpu-architecture.tex
- lines: 241-243
- severity: minor
- category: contradiction
- claim: "When the reset request (\texttt{RSTI}) is raised, the CPU abandons the current instruction or bus wait and restarts the phase sequence at phase 0 with the reset-vector fetch"
- evidence: reference/chapters/06-exceptions.tex:915-919 (RSTO is asserted for the RSTI cycles plus five before the reset-vector fetch); appendix-b-timing.tex:84-86; 06-exceptions.tex:862-863 (a still-asserted HALT stops the CPU before the reset-vector fetch)
- resolution: manual-wrong
- fix: "...abandons the current instruction or bus wait, holds the reset output (Section~\ref{sec:reset-output-hold}), and then restarts the phase sequence at phase 0 with the reset-vector fetch (Section~\ref{sec:reset-vector-fetch})."
- confidence: high

### F-con2-8
- file: reference/chapters/12-instruction-summary.tex
- lines: 125
- severity: nit
- category: contradiction
- claim: "\item \textbf{+\opcode{0F}}: Coprocessor call (neither result nor flags are written)"
- evidence: reference/chapters/12-instruction-summary.tex:123 (the +07 line carries "(\opcode{27} is undocumented)"); appendix-a-opcode-map.tex:128-130; 16-coprocessor-instructions.tex:39-40. Decision triaged.md:2752 (0x2F is not a coprocessor call) is applied to the +07 column but not the +0F column.
- resolution: manual-wrong
- fix: "\item \textbf{+\opcode{0F}}: Coprocessor call, \opcode{0F} and \opcode{3F} only (neither result nor flags are written; \opcode{2F} is undocumented)"
- confidence: high

### F-con2-9
- file: reference/chapters/notation-and-glossary.tex
- lines: 256-257
- severity: minor
- category: contradiction
- claim: "\item[Retryable fault] A fault whose triggering operation does not take effect, so that returning with \mnemonic{RETE} safely retries it."
- evidence: reference/chapters/06-exceptions.tex:148-156 and 828-830; 16-coprocessor-instructions.tex:801-831 (for DMAR/DMAW with a negative count and for DMAT, a bare RETE is not safe). The glossary restates the rule without its DMA exception.
- resolution: manual-wrong
- fix: "...safely retries it, except after a DMA burst (Section~\ref{sec:dma-fault-retry}). See Chapter~\ref{ch:exceptions} for the full retryable set."
- confidence: high

---

## Architecture manual against assembly manual

### F-con2-10
- file: reference/chapters/notation-and-glossary.tex
- lines: 121-125
- severity: major
- category: contradiction
- claim: "\item[\texttt{.ORG addr}] Sets the word address at which subsequent instructions and data are assembled."
- evidence: assembly/chapters/05-directives-and-symbols.tex:12-14 (decision triaged.md:225: "In a program linked from several objects, \texttt{.ORG} is relative to the start of its own object ... The vector table and any other fixed-address code must therefore go in the first object"). The glossary entry presents .ORG as an absolute address and gives the vector table as its use case, without the per-object rule.
- resolution: manual-wrong
- fix: Add to the glossary entry: "In a program linked from several objects the address is relative to the start of its own object, so the vector table must be in the first object (see the Directives and Symbols chapter of the \asmmanual{})." Better, per STYLE.md Scope ("ASM owns what the tools accept"), shorten the whole entry to one sentence and a pointer to the \asmmanual{}.
- confidence: high

### F-con2-11
- file: reference/chapters/notation-and-glossary.tex
- lines: 133-134
- severity: minor
- category: contradiction
- claim: "\item[\texttt{.EQU \$NAME \#value}] Defines a named constant, usable anywhere a \texttt{\#immediate} operand is accepted by writing \texttt{\$NAME} in its place."
- evidence: assembly/chapters/02-source-language.tex:85-87 (decision triaged.md:126: "A constant must be defined before its first use in the file. Defining the same name again changes its value for the lines that follow ... It cannot be used as the address of \texttt{.ORG}"). The entry directly below it (@label, line 135) says labels may be used before declaration, so a reader will assume the same for constants.
- resolution: manual-wrong
- fix: "...by writing \texttt{\$NAME} in its place. Unlike a label, a constant must be defined before its first use in the file; it cannot be the address of \texttt{.ORG}."
- confidence: high

### F-con2-12
- file: assembly/chapters/07-diagnostics.tex
- lines: 47-51
- severity: minor
- category: contradiction
- claim: "It is reported for a pre-decrement or post-increment form whose other register is a half of the updated address register, and for an \mnemonic{LDEL} or \mnemonic{LJSR} that would write \reg{l} twice. The Memory Access Instructions and Control Flow Instructions chapters of the \archmanual{} give the exact list."
- evidence: reference/chapters/15-control-flow.tex:75-81 ("The assembler rejects exactly these forms": also LDEA pre-decrement with destination equal to source, LDEL post-increment with destination equal to source, and any (p)+ or -(p) auto-update); reference/chapters/14-memory-instructions.tex:81; 08-addressing-modes.tex:433; appendix-c-undocumented.tex:49-50; 16-coprocessor-instructions.tex:977, 1075 (DMAR/DMAW "the assembler rejects a literal outside that range", which the assembly manual does not mention). The assembly manual's summary is narrower than the architecture manual's list, and the toolchain rule is owned by the wrong manual (STYLE.md Scope: ASM owns what the tools accept).
- resolution: design
- fix: Recommended: move the exact list of rejected aliased forms, and the DMA count range check, into assembly Ch.7 (the form, then the rule, then the diagnostic, per STYLE.md §9). In the architecture chapters keep only "architecturally undefined" and add a pointer to the Diagnostics chapter of the \asmmanual{}. Alternative: keep the list in ARCH Ch.15, make assembly Ch.7 cite it without a partial paraphrase, and add the (p) and LDEA forms to its summary.
- confidence: medium

### F-con2-13
- file: assembly/README.md
- lines: 8
- severity: minor
- category: contradiction
- claim: "| SIRC Assembly Programmer's Manual | Complete assembly syntax, directives, macros, pseudoinstructions | this directory |"
- evidence: accepted fix F-con-4 (triaged.md:103-115) listed hardware/README.md:8 and assembly/README.md:8. Both still read "macros, pseudoinstructions", against assembly/chapters/08-limitations.tex:13, 20 and the manual-set tables (assembly/chapters/01-introduction.tex:15, hardware/chapters/01-introduction.tex:15, reference/chapters/preface.tex:55), which were corrected.
- resolution: manual-wrong
- fix: In assembly/README.md:8 and hardware/README.md:8 replace "Complete assembly syntax, directives, macros, pseudoinstructions" with "Complete assembly syntax, directives, symbols, meta-instructions, and toolchain usage". The README file is not a chapter, so the orchestrator may need to assign it an editor.
- confidence: high

---

## Hardware manual against architecture manual

### F-con2-14
- file: hardware/chapters/04-bus-interface.tex
- lines: 149
- severity: blocker
- category: contradiction
- claim: "\activelow{HALT} & Sampled at instruction boundaries. If asserted, the CPU halts before the next instruction fetch and resumes when deasserted; interrupt lines are still sampled while halted."
- evidence: hardware/chapters/03-signal-descriptions.tex:204-209 and reference/chapters/06-exceptions.tex:853-863 (decision triaged.md:701 and :359: "HALT is sampled at every phase 0 ... Replace ... every 'instruction boundary' wording for HALT in both manuals"). The Ch.3 signal description and the architecture manual were updated, but this Ch.4 sampling table still gives the old rule. It also contradicts HW Ch.3 inside the same manual.
- resolution: manual-wrong
- fix: "\activelow{HALT} & Sampled at every phase 0. If asserted, the CPU stops at that phase 0 with the bus idle, which can fall between an instruction and its coprocessor dispatch slot, between DMA slots, or before a fault, interrupt, or reset dispatch. Halt latency is at most six cycles. The CPU resumes where it stopped when the input is deasserted; interrupt lines are still sampled while halted."
- confidence: high

### F-con2-15
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 114-116, 130-133
- severity: blocker
- category: contradiction
- claim: "This causes the CPU to abort the I/O operation and raise a bus fault exception, except during an exception-vector fetch. The response to a failed vector fetch is described in the Exception and Interrupt Handling chapter of the \archmanual{}: a failed fault-vector fetch raises a double fault, and a failed reset-vector fetch re-enters reset."
- evidence: reference/chapters/06-exceptions.tex:782-794 (a failed vector fetch for a hardware interrupt, Level 5 request or software exception does raise the bus or bus-protection fault, with bus access type 0x4; only fault, double-fault and reset vectors behave differently). The hardware text says that no exception-vector fetch raises a bus fault. The same wording appears for BPER at 130-133.
- resolution: manual-wrong
- fix: "This causes the CPU to abort the I/O operation and raise a bus fault exception. A failed fault-vector fetch raises a double fault instead, a failed double-fault vector fetch resets the CPU, and a failed reset-vector fetch re-enters reset (see the Exception and Interrupt Handling chapter of the \archmanual{})." Make the same change for BPER, with "bus protection fault".
- confidence: high

### F-con2-16
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 284-287
- severity: minor
- category: contradiction
- claim: "The reset-output hold is six CLKI cycles. The cycle in which \activelow{RSTI} is first sampled asserted counts as the first, and \activelow{RSTO} stays asserted for five further cycles after \activelow{RSTI} is sampled deasserted, so an \activelow{RSTI} assertion of $n$ cycles gives $n + 5$ cycles of \activelow{RSTO}."
- evidence: the next clause in the same sentence gives n + 5; hardware/chapters/05-bus-timing.tex:167-170 qualifies the same statement ("six CLKI cycles in this case", for a one-cycle RSTI); reference/chapters/06-exceptions.tex:917-919; appendix-b-timing.tex:84-85
- resolution: manual-wrong
- fix: "The reset-output hold lasts for as long as \activelow{RSTI} is asserted plus five cycles: the cycle in which ... so an \activelow{RSTI} assertion of $n$ cycles gives $n + 5$ cycles of \activelow{RSTO} (six for a one-cycle assertion)."
- confidence: high

### F-con2-17
- file: hardware/chapters/03-signal-descriptions.tex
- lines: 247
- severity: nit
- category: style
- claim: "\section{Non-Maskable Interrupt Inputs (NMI)}"
- evidence: STYLE.md §5 [HW] heading form "Name Direction (SIGNAL)"; the signal is one pin (hardware/chapters/02-package-and-pinout.tex:140, "Non-maskable interrupt input")
- resolution: n/a
- fix: "\section{Non-Maskable Interrupt Input (NMI)}"
- confidence: high

---

## Terminology (STYLE.md)

### F-con2-18
- file: reference/chapters/08-addressing-modes.tex
- lines: n/a
- severity: minor
- category: terminology
- claim: "Memory address = address register + immediate offset"
- evidence: STYLE.md §1 ("displacement" for addressing-mode and branch values; "offset" only for memory or structure offsets); reference/chapters/notation-and-glossary.tex:236-238 defines the same split. The `#offset` placeholder of Table 11.1 is sanctioned and is not part of this finding. Prose and table labels using "offset" for an addressing-mode displacement appear in: reference/chapters/08-addressing-modes.tex, reference/chapters/14-memory-instructions.tex, reference/chapters/15-control-flow.tex (legal-forms Notes column), reference/chapters/notation-and-glossary.tex (Table "Operand Forms" rows and the listing comment "PC-relative offset"), reference/chapters/appendix-d-examples.tex, assembly/chapters/03-operands.tex (table names and "The offset in parentheses").
- resolution: n/a
- fix: Replace in prose and labels, not in placeholders: `grep -nE "(immediate|register|PC-relative) offset|offset is taken|The offset in parentheses" docs/reference/chapters/*.tex docs/assembly/chapters/*.tex`. Write "immediate displacement" and "register displacement". Leave assembly/chapters/05-directives-and-symbols.tex:113 ("offset (low word)") alone; it is a segment offset.
- confidence: medium

### F-con2-19
- file: reference/chapters/appendix-c-undocumented.tex
- lines: n/a
- severity: minor
- category: terminology
- claim: "public assembly syntax and are reserved for expansion in future CPU revisions."
- evidence: STYLE.md §1 ("Do not use 'undocumented' and 'reserved' interchangeably"). Undocumented opcodes 0x27 and 0x2F are called "reserved" in reference/chapters/appendix-c-undocumented.tex (Overview and "Opcode Space for Expansion") and reference/chapters/appendix-a-opcode-map.tex ("Undocumented Instructions" section).
- resolution: n/a
- fix: `grep -nE "reserved for (expansion|future)" docs/reference/chapters/appendix-[ac]-*.tex`. Write "may be assigned in a future revision" instead of "reserved for ...".
- confidence: high

### F-con2-20
- file: reference/chapters/appendix-c-undocumented.tex
- lines: 108
- severity: minor
- category: terminology
- claim: "\item Document any implementation-specific behavior."
- evidence: STYLE.md §1 and §2 (undocumented opcodes are "architecturally undefined", not implementation-defined); the same appendix, lines 7-9 and 71-73
- resolution: n/a
- fix: "\item Do not promise any behavior for these opcodes; it is architecturally undefined."
- confidence: medium

### F-con2-21
- file: reference/chapters/16-coprocessor-instructions.tex
- lines: n/a
- severity: nit
- category: terminology
- claim: "the high hex digit \texttt{x} holds bits 7--4"
- evidence: STYLE.md §1 ("hexadecimal ... | hex (prose)"). Found in reference/chapters/16-coprocessor-instructions.tex (ETFR/ETTR Instruction Fields) and assembly/chapters/02-source-language.tex (Number Forms table, "hex digits").
- resolution: n/a
- fix: `grep -nwE "hex" docs/reference/chapters/*.tex docs/assembly/chapters/*.tex`. Replace with "hexadecimal".
- confidence: high

### F-con2-22
- file: reference/chapters/appendix-c-undocumented.tex
- lines: n/a
- severity: nit
- category: terminology
- claim: "An implementation -- whether an emulator or a hardware implementation -- decodes undocumented instructions normally"
- evidence: STYLE.md §1 ("assembler, linker, simulator | VM, emulator"). Only file: reference/chapters/appendix-c-undocumented.tex.
- resolution: n/a
- fix: `grep -nw "emulator" docs/*/chapters/*.tex`. Write "simulator".
- confidence: high

### F-con2-23
- file: reference/chapters/13-alu-instructions.tex
- lines: n/a
- severity: nit
- category: terminology
- claim: "sets N and Z from \texttt{r1} and clears C and V without changing it"
- evidence: STYLE.md §1 and §3 (`\reg{rN}` for a concrete register in prose; `\texttt` only inside a whole code example). Found in reference/chapters/13-alu-instructions.tex (ADDI notes and CMNI description).
- resolution: n/a
- fix: `grep -nE "(from|when|sets [A-Z ]+from) \\\\texttt\{r[1-7]\}" docs/reference/chapters/*.tex`. Replace the bare `\texttt{rN}` with `\reg{rN}`. Leave full instructions such as `\texttt{ADDI r1, \#0}` as they are.
- confidence: medium

### F-con2-24
- file: reference/chapters/03-registers.tex
- lines: 95-96
- severity: nit
- category: terminology
- claim: "Toolchain and object-file topics are covered in the \emph{SIRC-1 Assembly Programmer's Manual}."
- evidence: STYLE.md Scope (use the book-title macros, not hand-typed titles); reference/preamble.tex:131 defines `\asmmanual{}`, which reference/chapters/preface.tex:44 uses for the same sentence.
- resolution: n/a
- fix: "Toolchain and object-file topics are covered in the \asmmanual{}."
- confidence: high

---

## Summary

| Manual | contradiction | terminology | style | latex | structure | Total |
|---|---|---|---|---|---|---|
| Architecture (reference) | 11 (F-1 to F-11) | 6 (F-18 to F-20, F-22 to F-24; F-18 and F-21 also cover assembly) | 0 | 0 | 0 | 17 |
| Hardware | 3 (F-14 to F-16) | 0 | 1 (F-17) | 0 | 0 | 4 |
| Assembly | 2 (F-12, F-13; F-13 also covers hardware/README.md) | 1 (F-21) | 0 | 0 | 0 | 3 |
| **Total** | **16** | **7** | **1** | **0** | **0** | **24** |

Most damaging contradictions:

1. F-con2-14: the hardware manual's input-sampling table still says HALT is sampled at instruction boundaries, against the phase-0 decision applied in HW Ch.3 and ARCH Ch.6.
2. F-con2-1: the traced-EXCP decision holds the trap "in the CSR". That conflicts with the CSR-empty rule for selecting trace faults, and with the trace handler's own coprocessor calls overwriting the CSR.
3. F-con2-15: the hardware BERR/BPER descriptions say no exception-vector fetch raises a bus fault. The architecture manual raises one for interrupt and trap vector fetches.
