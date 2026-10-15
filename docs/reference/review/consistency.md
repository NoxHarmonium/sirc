# Consistency review (reviewer tag: cons)

Whole-manual cross-file pass. No claim below was checked against the implementation; every
`evidence` entry cites another part of the manual.

Mechanical LaTeX checks all pass and are reported here for the record, not as findings:
140 `\label`s, zero duplicates; every `\ref`/`\autoref`/`\cref`/`\pageref` target resolves;
every `\input` in `main.tex` exists and every file in `chapters/` is included; all
`lstlisting`, `instructionbox`, and `tabular`/`tabularx` environments balance.

---

### F-cons-1
- file: chapters/06-exceptions.tex
- lines: 636, 640, 644
- severity: blocker
- category: contradiction
- claim: "\texttt{0x01} & \addr{0x0002}--\addr{0x0003} & 7 & Bus fault & Not maskable & Next instruction & Fault metadata & Yes"
- evidence: chapters/06-exceptions.tex:68-69 (Bus Fault, Bus Protection Fault and Privilege Violation Fault are all listed as *Retryable*); chapters/06-exceptions.tex:91-93 ("the link register stores the address of the \textit{faulting} instruction"); chapters/06-exceptions.tex:111-112; chapters/06-exceptions.tex:162-163; chapters/06-exceptions.tex:273-275; chapters/06-exceptions.tex:763-765; chapters/14-memory-instructions.tex:95; chapters/appendix-b-timing.tex:71-72 ("Alignment, Bus, Bus Protection, Invalid Opcode, Privilege Violation, and Segment Overflow all save the address of the faulting instruction and are retryable")
- resolution: manual-wrong
- fix: In Table~\ref{tab:exception-quick-reference}, change the **Return Address** cell from "Next instruction" to "Faulting instruction" in three rows: \texttt{0x01} (Bus fault, line 636), \texttt{0x05} (Privilege fault, line 640), and \texttt{0x09} (Bus protection, line 644). Rows \texttt{0x02}, \texttt{0x03} and \texttt{0x04} already read "Faulting instruction"; \texttt{0x06} (Trace fault) correctly stays "Next instruction". Every other location in the manual agrees that the six retryable faults save the faulting instruction's address, so the table is the sole outlier.
- confidence: high

### F-cons-2
- file: chapters/09-shift-operations.tex
- lines: 492
- severity: blocker
- category: contradiction
- claim: "\item A register-supplied shift count greater than 15 is architecturally undefined"
- evidence: chapters/09-shift-operations.tex:322-337 (same chapter: "When the shift count is taken from a register (SO = 1), the full 16-bit register value is available as a count, and a count greater than 15 is well defined:", then three bullets giving the exact result for LSL/LSR/ASL, ASR, and RTL/RTR); chapters/13-alu-instructions.tex:375, 501, 620, 741, 858, 981, 1107, 1347, 1483, 1615 (ten instruction boxes: "a register count is used in full (16 or more clamps to a full shift for LSL/LSR/ASL/ASR, and is taken modulo 16 for RTL/RTR)"); chapters/13-alu-instructions.tex:261-262 and 316-317 (SHFT box: "a register count above 15 is architecturally undefined"); chapters/14-memory-instructions.tex:355-356 (STOR: "a register count above 15 is architecturally undefined")
- resolution: manual-wrong
- fix: Adopt the defined behaviour (the majority form: Chapter 9 §Shift Count plus ten of the twelve instruction boxes) and delete/replace the three "architecturally undefined" statements. Delete the bullet at `09-shift-operations.tex:492`; replace the SHFT box text at `13-alu-instructions.tex:261-262` and the Note at `13-alu-instructions.tex:316-317` with the wording already used by the other ten boxes; replace `14-memory-instructions.tex:355-356` likewise. If instead the undefined form is the intended architecture, the change is far larger: Chapter 9 §Shift Count (lines 322-337) and all ten instruction boxes must be rewritten. Flagging both sides because the manual is the source of truth and cannot adjudicate itself here.
- confidence: high

### F-cons-3
- file: chapters/06-exceptions.tex
- lines: 729-732
- severity: blocker
- category: contradiction
- claim: "Steps 1 and 2 each occupy their own fetch cycle, an instruction-decode cycle follows between step 2 and step 3 (not itemized above), and steps 3 and 4 then happen together in a single execute cycle. The reset-vector fetch therefore takes four clock cycles in total"
- evidence: chapters/appendix-b-timing.tex:47 ("\mnemonic{RSET} & 18 (6 call + 6 \texttt{RSTO} hold + 6 reset-vector fetch)"); chapters/16-coprocessor-instructions.tex:534-535 ("6 cycles for the call, plus a 6-cycle \texttt{RSTO} hold, plus 6 cycles for the exception unit to fetch the reset vector -- 18 cycles from the \mnemonic{RSET} fetch to the first instruction of the reset handler")
- resolution: unclear
- fix: The reset-vector fetch is 4 cycles in Chapter 6 and 6 cycles in both Appendix B and Chapter 16, which also makes the \mnemonic{RSET} total 16 vs 18. Pick one. If Chapter 6's four-cycle breakdown is right, change `appendix-b-timing.tex:47` to "16 (6 call + 6 \texttt{RSTO} hold + 4 reset-vector fetch)" and `16-coprocessor-instructions.tex:534-535` to "plus 4 cycles for the exception unit to fetch the reset vector -- 16 cycles". If 6 is right, replace Chapter 6's phase-by-phase paragraph at 729-732. Note that the reset-output hold itself (6 cycles) is consistent everywhere (`06-exceptions.tex:708-714`, `appendix-b-timing.tex:65`, `appendix-f-external-interface.tex:689` which shows one assert + five hold columns).
- confidence: high

### F-cons-4
- file: chapters/07-instruction-formats.tex
- lines: 253-254
- severity: blocker
- category: contradiction
- claim: "When only two register operands are given, R2 is encoded as a copy of R1 and the second operand is encoded in R3, for example \texttt{ADDR r4, r5} uses R1=\texttt{0x4}, R2=\texttt{0x4}, R3=\texttt{0x5}"
- evidence: chapters/03-registers.tex:283-298, Table~\ref{tab:register-encoding} (\reg{r4} = \texttt{0x3}, \reg{r5} = \texttt{0x4}; \texttt{0x5} is \reg{r6}); generated/register-format-encodings.tex:38 (the generated table encodes \texttt{CMPR r4, r5} as R1=\texttt{0x3}, R2=\texttt{0x3}, R3=\texttt{0x4})
- resolution: manual-wrong
- fix: Replace the field values with the correct register IDs: "for example \texttt{ADDR r4, r5} uses R1=\texttt{0x3}, R2=\texttt{0x3}, R3=\texttt{0x4}". The register-index table and the generated encoding table agree with each other, so Chapter 7's note is the outlier; as written it reads register names as if they were field values and is off by one for every register.
- confidence: high

### F-cons-5
- file: chapters/11-reading-instructions.tex
- lines: 132-134
- severity: blocker
- category: contradiction
- claim: "\mnemonic{LOAD} always encodes \texttt{[N]} and never updates the status flags, regardless of any override syntax."
- evidence: chapters/05-status-register.tex:190-193 ("An explicit status update override written on \mnemonic{LOAD} leaves all four flags architecturally undefined (\texttt{U U U U})"); chapters/12-instruction-summary.tex:101-104 ("an explicit status update override on \mnemonic{LOAD} leaves all four flags architecturally undefined"); chapters/13-alu-instructions.tex:178-180 ("Public \mnemonic{LOAD} forms do not support status update override syntax ... an encoding of the \mnemonic{LOAD} opcode with a non-\texttt{None} AF field, which is not reachable from public assembly syntax, leaves all four status flags architecturally undefined"); chapters/13-alu-instructions.tex:1219-1221 (same, in the LOAD box)
- resolution: manual-wrong
- fix: Three mutually exclusive stories are in print: (a) the override may be written and is ignored (Ch.11), (b) the override may be written and yields \texttt{U U U U} (Ch.5, Ch.12), (c) the override cannot be written at all and only a non-public *encoding* yields \texttt{U U U U} (Ch.13, twice). Chapter 13's (c) is the most specific and is stated twice, and Chapter 8's legality table (`08-addressing-modes.tex:93`) also gives LOAD no override form, so adopt (c) everywhere. Rewrite `11-reading-instructions.tex:132-134` to "\mnemonic{LOAD} always encodes \texttt{[N]} and never updates the status flags; its public syntax does not accept a status update override." Rewrite `05-status-register.tex:190-193` and `12-instruction-summary.tex:101-104` to say "an encoding of the \mnemonic{LOAD} opcode with a non-\texttt{None} AF field, not reachable from public assembly syntax, leaves all four flags architecturally undefined".
- confidence: high

### F-cons-6
- file: chapters/notation-and-glossary.tex
- lines: 89-91
- severity: major
- category: contradiction
- claim: "One of the specialized execution units a \mnemonic{COPI}/\mnemonic{COPR} call can dispatch to -- the processing unit itself (coprocessor \texttt{0x0}), the exception unit (\texttt{0x1}), the DMA unit (\texttt{0x2}), and the optional integer maths unit (\texttt{0x3})."
- evidence: chapters/01-introduction.tex:99 ("every other coprocessor, including the DMA unit at \texttt{0x2}, is optional and varies by SIRC-1 model"); chapters/01-introduction.tex:113 (Table~\ref{tab:coprocessor-list}: DMA Unit, Optional = Yes); chapters/16-coprocessor-instructions.tex:194 (Table~\ref{tab:standard-coprocessor-ids}: DMA Unit, Required = No); chapters/16-coprocessor-instructions.tex:652 ("The optional DMA unit"); chapters/16-coprocessor-instructions.tex:837, 929, 1023 (DMAR/DMAW/DMAT applicability lines: "requires Coprocessor 2 (DMA Unit), an optional coprocessor absent on models that do not implement it")
- resolution: manual-wrong
- fix: The glossary is the last place that still implies DMA is always present: it marks only the maths unit "optional", so by contrast the DMA unit reads as required. Change to "... the exception unit (\texttt{0x1}), the optional DMA unit (\texttt{0x2}), and the optional integer maths unit (\texttt{0x3})". The rest of the manual is clean on this point: Chapter 12's meta-instruction table, Appendix B's timing table, and Appendix E's mnemonic index list the DMA mnemonics without asserting the unit is present, and no file says "required"/"always present" of coprocessor 2. (Checked separately: no trace of a reverted DMAT register-count form survives anywhere — no "DMATR", no `DMAT rS`, no `0x2B00`, and every one of the six files that names DMAT gives the immediate-only `COPI #0x2Ann` form.)
- confidence: high

### F-cons-7
- file: chapters/appendix-e-quick-reference.tex
- lines: 189-190
- severity: major
- category: contradiction
- claim: "See Chapter~\ref{ch:cpu-architecture}, Section \"Pin Descriptions\", for usage examples including the PROT pin interaction."
- evidence: chapters/02-cpu-architecture.tex has no "Pin Descriptions" section (its sections are Overview, Six-Stage Execution Phases, Architectural Components, Design Characteristics, External Interface); chapters/02-cpu-architecture.tex:175-176 ("the chip package, the pin-by-pin descriptions, and the bus timing diagrams are given in Appendix~\ref{appendix:external-interface}"); chapters/appendix-f-external-interface.tex:152 (`\subsection{Pin Descriptions}`); chapters/appendix-f-external-interface.tex:293-295 (the PROT/BAT usage example the sentence promises)
- resolution: manual-wrong
- fix: Replace with "See Appendix~\ref{appendix:external-interface}, Section \"Pin Descriptions\", for usage examples including the PROT pin interaction." The `\ref` resolves, so this passes the mechanical check, but it sends the reader to the wrong document division.
- confidence: high

### F-cons-8
- file: chapters/notation-and-glossary.tex
- lines: 65-67
- severity: major
- category: contradiction
- claim: "[reserved] A field, opcode, or coprocessor ID that is not currently assigned a meaning. Software must not assume any particular value when reading a reserved field, and must write zero unless a chapter states an explicit encoding."
- evidence: chapters/05-status-register.tex:87-89 ("Bits 4 through 7 of the lower byte are reserved for future use; software must not rely on them. These bits can be written by a supervisor-mode direct write to \reg{sr}"); chapters/appendix-e-quick-reference.tex:146 ("Reserved for future use; software must not rely on the value read"); docs/reference/STYLE.md §"Approved decisions" ("Reserved fields stay advisory. Keep the Chapter 5 wording ... Do *not* adopt the M68000 PRM \"must be written as zeros\" form; the implementation does not promise it.")
- resolution: manual-wrong
- fix: The glossary defines "reserved" with a mandatory write-zero requirement; Chapter 5 and Appendix E define it as advisory-read-only, and Chapter 5 explicitly permits a supervisor write to the reserved SR bits. STYLE's approved decision settles it in favour of the advisory form. Change the glossary entry to "Software must not assume any particular value when reading a reserved field, and must not rely on any behaviour it may have. A future CPU revision may assign a reserved field a meaning." and drop "must write zero unless a chapter states an explicit encoding." Note the same wording appears in `STYLE.md` §2 and should be corrected there too if the style sheet is being maintained.
- confidence: high

### F-cons-9
- file: chapters/12-instruction-summary.tex
- lines: 124-127
- severity: minor
- category: contradiction
- claim: "\item \textbf{+\opcode{00} to +\opcode{07}}: Save result to destination register \item \textbf{+\opcode{08} to +\opcode{0E}}: Test only (update flags, discard result)"
- evidence: chapters/07-instruction-formats.tex:340-348 ("\texttt{0x\_0}--\texttt{0x\_6}: Save result to destination ... \texttt{0x\_8}--\texttt{0x\_E}: Test only ... Opcodes \texttt{0x27} and \texttt{0x2F} are exceptions to this pattern"); chapters/appendix-a-opcode-map.tex:124-131 ("Bit 3 = 0 and low nibble \texttt{0}--\texttt{6}: save result ... Low nibble \texttt{7} is \mnemonic{LOAD} ... \opcode{27} and \opcode{2F} are undocumented")
- resolution: manual-wrong
- fix: Chapter 12 widens both ranges by one and so claims \opcode{27} is a save-result form (it is undocumented, per `12-instruction-summary.tex:17` in the same chapter) and that \opcode{08}/\opcode{09}/\opcode{0B}/\opcode{0D} are documented test-only forms (they are undocumented, per the chapter's own Table~\ref{tab:complete-instruction-set}). Match Chapter 7 and Appendix A: "+\opcode{00} to +\opcode{06}: Save result to destination register; +\opcode{07}: \mnemonic{LOAD} (\opcode{27} is undocumented); +\opcode{08} to +\opcode{0E}: test-only where a documented mnemonic exists; +\opcode{0F}: coprocessor call."
- confidence: high

### F-cons-10
- file: chapters/appendix-d-examples.tex
- lines: 64-65
- severity: minor
- category: contradiction
- claim: "    .ORG 0x0080\n    .DQ @overflow_handler          ; vector 0x40: exception vector table entry"
- evidence: chapters/06-exceptions.tex:620 ("\textbf{\texttt{0x40}:} Level 2 hardware exception vector"); chapters/06-exceptions.tex:649 (quick-reference row: \texttt{0x40}, \addr{0x0080}--\addr{0x0081}, priority 3, Level 2 hardware); chapters/06-exceptions.tex:609 (segment overflow is vector \texttt{0x03}, at \addr{0x0006})
- resolution: manual-wrong
- fix: The address and the "vector 0x40" annotation are both correct, but naming the handler `overflow_handler` invites the reader to think vector 0x40 is the overflow vector when it is the Level 2 hardware exception. Rename the label to `@level2_handler` (and the `:overflow_handler` declaration at line 73) or change the comment to "vector 0x40: level 2 hardware exception vector".
- confidence: medium

### F-cons-11
- file: chapters/appendix-d-examples.tex
- lines: 132, 155
- severity: minor
- category: contradiction
- claim: "The exception handler itself is installed at vector 0x0080 and defined from line 137 onward" ... "because the handler at vector 0x0500 (line 37) sets \reg{r1} to 0xBB only if it runs"
- evidence: chapters/06-exceptions.tex:597-603 ("The vector table has 256 entries", vectors \texttt{0x00}--\texttt{0xFF}, occupying word addresses \addr{0x000000}--\addr{0x0001FF}); chapters/04-data-representation.tex:99-105 (vector entry \texttt{V} starts at word address \texttt{V * 2})
- resolution: manual-wrong
- fix: Both sentences use "vector" for a word address. \texttt{0x0080} is the *address* of vector \texttt{0x40}, and \texttt{0x0500} is not a vector address at all — it is past the end of the 256-entry table, so it must be the handler's code address. Rewrite as "installed at vector \texttt{0x40} (word address \addr{0x0080})" and "because the handler at address \addr{0x0500} ...". Same conflation at line 65's comment (see F-cons-10).
- confidence: high

### F-cons-12
- file: chapters/notation-and-glossary.tex
- lines: 99-100
- severity: minor
- category: contradiction
- claim: "[Fault] A priority-7 exception raised by the CPU itself in response to an error condition (for example, a bus error or a privilege violation) or a debug event (instruction trace). Faults cannot be masked or deferred."
- evidence: chapters/06-exceptions.tex:41-42 ("Two faults are conditional on whether they are raised at all: Segment Overflow requires \texttt{SR.A}, and Instruction Trace requires \texttt{SR.T} or the \texttt{TRCE} input."); chapters/06-exceptions.tex:638 and 641 (quick-reference **Maskability** column: "SR.A gated" for \texttt{0x03}, "SR.T or TRCE gated" for \texttt{0x06})
- resolution: manual-wrong
- fix: Add Chapter 6's caveat to the glossary entry: "Faults cannot be masked or deferred once raised, though two of them are conditional on whether they are raised at all: Segment Overflow requires \texttt{SR.A}, and Instruction Trace requires \texttt{SR.T} or the \texttt{TRCE} input." As written the glossary flatly denies the gating the exceptions chapter documents twice.
- confidence: high

### F-cons-13
- file: chapters/appendix-f-external-interface.tex
- lines: 370-374
- severity: minor
- category: contradiction
- claim: "Interrupt inputs are level-sensitive and are sampled at instruction boundaries. ... A line that is asserted while its enable bit is set is latched at the moment of assertion, even if it is released before the next instruction boundary; only an assertion that begins and ends while the enable bit is 0 is lost."
- evidence: chapters/06-exceptions.tex:429-436 ("Hardware interrupt lines 1--4 are level-sensitive inputs ... When an enabled level-sensitive interrupt line is observed asserted, the corresponding bit is set in the pending hardware-exceptions register."); chapters/06-exceptions.tex:438 ("At an instruction boundary, the exception unit selects the highest-priority pending hardware interrupt"); chapters/05-status-register.tex:130-131 ("Hardware interrupt lines 1-4 are level-sensitive, not edge-triggered. If an interrupt pin remains asserted, it will be serviced again as soon as the current handler is finished.")
- resolution: unclear
- fix: Appendix F's paragraph contradicts its own first sentence (sampled at instruction boundaries vs latched at the moment of assertion) and makes a guarantee about sub-instruction pulses that neither Chapter 5 nor Chapter 6 makes. Decide whether IRQ1--IRQ4 are latched continuously or only sampled at boundaries, then make `appendix-f-external-interface.tex:370-374`, its summary row at `appendix-f-external-interface.tex:739`, and `06-exceptions.tex:429-436` say the same thing.
- confidence: medium

### F-cons-14
- file: chapters/13-alu-instructions.tex
- lines: 220, 1348-1349
- severity: minor
- category: contradiction
- claim: "AF encoding \texttt{11} (binary) is reserved and has no assembly syntax; the status register is left unchanged." / "\textbf{AF (bits 5--4):} \texttt{00} = None, \texttt{01} = Alu (default), \texttt{10} = Shift, \texttt{11} = Reserved (status register unchanged)."
- evidence: chapters/07-instruction-formats.tex:97, 156, 225 ("11 = Reserved for future use"); chapters/11-reading-instructions.tex:125 ("Reserved & \texttt{11} & Reserved for future use"); chapters/notation-and-glossary.tex:65-67 (reserved fields have no assumable behaviour); chapters/appendix-c-undocumented.tex:199 ("The reserved status-update source encoding (AF = 11) could select a new flag-update source")
- resolution: manual-wrong
- fix: Chapter 13 promises a specific behaviour ("status register unchanged") for an encoding that Chapter 7, Chapter 11, the glossary, and Appendix C all describe as reserved and therefore unspecified — and that Appendix C expects a future revision to repurpose. Drop the parenthetical: change line 220 to "AF encoding \texttt{11} (binary) is reserved and has no assembly syntax." and the six instruction-box lines matching `11 = Reserved (status register unchanged)` (13-alu-instructions.tex:1348-1349 and 1484-1485, plus any other boxes carrying the same phrase) to "\texttt{11} = Reserved". Grep pattern: `Reserved (status register unchanged)`.
- confidence: medium

### F-cons-15
- file: chapters/appendix-d-examples.tex
- lines: 6-7
- severity: minor
- category: structure
- claim: "\section{Assembler Directives}\n\label{sec:assembler-directives}"
- evidence: chapters/11-reading-instructions.tex:73-75 (the only inbound reference: "see Section~\ref{sec:assembler-directives} for the full resolution rules, including data directives"); chapters/appendix-d-examples.tex:18, 52, 119, 143 (Appendix D's only `\ref`s all point at Chapter 6 or at its own examples)
- resolution: n/a
- fix: The Chapter 11 -> Appendix D link is accurate — Appendix D §Assembler Directives does define `@label` resolution for BRAN/BRSR, immediate-form LOAD, and the `.DB`/`.DW`/`.DQ` data directives, exactly as Chapter 11 promises. The return leg is missing: add a sentence to the `@label` entry (appendix-d-examples.tex:21) such as "The placeholder names used in instruction entries are defined in Chapter~\ref{ch:reading-instructions}, Table~\ref{tab:instruction-description-notation}." Separately, `\label{ch:example-programs}` (appendix-d-examples.tex:2) and `\label{ch:quick-reference}` (appendix-e-quick-reference.tex:2) are never referenced from anywhere, so Appendices D and E are unreachable by cross-reference even though the preface (preface.tex:34-36) advertises both.
- confidence: high

### F-cons-16
- file: chapters/appendix-d-examples.tex
- lines: 2
- severity: nit
- category: latex
- claim: "\label{ch:example-programs}"
- evidence: chapters/appendix-a-opcode-map.tex:2 (`\label{appendix:opcode-map}`); chapters/appendix-b-timing.tex:2 (`appendix:timing`); chapters/appendix-c-undocumented.tex:2 (`appendix:undocumented`); chapters/appendix-f-external-interface.tex:2 (`appendix:external-interface`); chapters/appendix-e-quick-reference.tex:2 (`\label{ch:quick-reference}`)
- resolution: n/a
- fix: Four of the six appendices use the `appendix:` label prefix; Appendices D and E use `ch:`. Rename to `\label{appendix:example-programs}` and `\label{appendix:quick-reference}`. Both are currently unreferenced (F-cons-15), so no `\ref` sites need updating — but fix it before anything starts pointing at them.
- confidence: high

### F-cons-17
- file: chapters/15-control-flow.tex
- lines: 33-53, 370, 497
- severity: minor
- category: style
- claim: "LDEA & \texttt{LDEA dest, (\#offset, src)} & 0x18 & Immediate-offset effective address"
- evidence: docs/reference/STYLE.md §3 Notation ("**Opcodes**: use `\opcode{XX}` ... only for a literal instruction-opcode byte value"); chapters/13-alu-instructions.tex:57-95 and chapters/14-memory-instructions.tex:56-63 (the equivalent legal-forms tables in the neighbouring chapters use `\opcode{}` throughout)
- resolution: n/a
- fix: Wrap the bare opcode literals in `\opcode{}` in Table~\ref{tab:control-flow-legal-forms} (16 cells, lines 33-53) and in the LDEA/LDEL **Opcodes:** lines (370, 497). The same slip appears in Chapter 16: `16-coprocessor-instructions.tex:25-26` (`0x0F`, `0x3F` in Table~\ref{tab:coprocessor-legal-forms}), `16-coprocessor-instructions.tex:33-36`, and `16-coprocessor-instructions.tex:76`. Grep pattern: `& 0x[0-9A-F][0-9A-F] *&` and `Opcodes:} 0x`.
- confidence: high

### F-cons-18
- file: chapters/16-coprocessor-instructions.tex
- lines: 131-135, 293-305, 368-377, 438-449, 509-519, 604-616, 872-882, 964-974, 1054-1062, 1230-1242, 1320-1329
- severity: minor
- category: style
- claim: "\textbf{Operation:}\n\begin{lstlisting}\nif condition_true then\n    pending_coprocessor_command = command_operand\nendif\n\end{lstlisting}"
- evidence: chapters/13-alu-instructions.tex (12 `\begin{verbatim}` Operation blocks); chapters/14-memory-instructions.tex (2); chapters/15-control-flow.tex (7); chapters/16-coprocessor-instructions.tex (0 `verbatim`, 11 `lstlisting` Operation blocks)
- resolution: n/a
- fix: Chapter 16 is the only instruction-reference chapter that puts **Operation** pseudocode inside `lstlisting` instead of `verbatim`. Because `lstlisting` defaults to the `SIRC` language (STYLE §7), the pseudocode is syntax-highlighted as assembly and contains constructs that are not assembler syntax — trailing-colon block labels (`MULU:` at line 1231, `MULS:` at 1237, `else:` at 1325) and Python-style `for i in 1..count:` (lines 877, 969). These are the only trailing-colon-looking lines left in any listing in the manual, and they read as violations of the leading-colon label rule even though they are not labels. Convert Chapter 16's Operation blocks to `verbatim` to match Chapters 13-15. (Audited separately: all 27 real assembler label declarations manual-wide already use the leading-colon form, and every branch/call/symbol operand already uses `@label` — no survivors from the pre-fix trailing-colon era.)
- confidence: high

### F-cons-19
- file: chapters/05-status-register.tex
- lines: 266
- severity: minor
- category: terminology
- claim: "the idioms themselves belong to Chapter~\ref{ch:condition-codes} and the individual instruction entries (e.g. \mnemonic{TSAI}/\mnemonic{TSAR} in Chapter~\ref{ch:alu-instructions})"
- evidence: docs/reference/STYLE.md §5 Prose and §"Approved decisions" ("**\"for example\", never \"e.g.\"** Spell it out in prose and tables alike. Replace every `e.g.` manual-wide.")
- resolution: n/a
- fix: Replace with "for example". Grep pattern: `e\.g\.` — three hits total, at `05-status-register.tex:266` (running prose), `16-coprocessor-instructions.tex:812` (a listing comment). The related Latin abbreviation `i.e.` also survives once, at `06-exceptions.tex:149`; replace with "that is". (The Chapter 5 cross-references themselves check out: `sec:status-override` lands on Chapter 9's Status Flag Update Control section with three worked examples, Chapter 10 does cover condition-code-driven branching, and the TSAI/TSAR box in Chapter 13 does carry the bit-testing example Chapter 5 defers to.)
- confidence: high

### F-cons-20
- file: chapters/appendix-c-undocumented.tex
- lines: 7, 8, 11, 13
- severity: minor
- category: terminology
- claim: "Most of these opcodes are valid encodings with architecturally undefined behaviour: software must not depend on their results, side effects, timing, flag behaviour, or fault behaviour."
- evidence: 17 files use the American spelling "behavior" (01, 02, 05, 06, 09, 11, 13, 14, 15, 17, appendix-a, appendix-c, appendix-d, notation-and-glossary, preface, and more); only `appendix-c-undocumented.tex` (5 instances) and `appendix-e-quick-reference.tex` (1 instance, line 17) use "behaviour"
- resolution: n/a
- fix: Standardize on "behavior". Grep pattern: `behaviour` — six hits across two files. Note `appendix-c-undocumented.tex` itself mixes both spellings (4 "behavior" against 5 "behaviour"), sometimes in adjacent sentences.
- confidence: high

### F-cons-21
- file: chapters/16-coprocessor-instructions.tex
- lines: 1104-1106
- severity: nit
- category: terminology
- claim: "which would make its instruction count unpredictable like every other meta-instruction's is not"
- evidence: docs/reference/STYLE.md §2 ("**unpredictable** — not used in this manual. Do not introduce it as a synonym for \"undefined\"; the manual already distinguishes \"architecturally undefined\" from \"implementation-defined\" and a third term would blur that line.")
- resolution: n/a
- fix: This is the manual's only instance of "unpredictable", and it is used in the ordinary-English sense (an instruction count a reader cannot predict), not as a behaviour-class term — but STYLE asks for zero instances so the word never starts drifting toward the normative sense. Reword, for example "which would make its instruction count vary with its operand, unlike every other meta-instruction". Grep pattern: `unpredictable`. The sentence is also hard to parse as written ("unpredictable like every other meta-instruction's is not"); the rewrite fixes both.
- confidence: medium

---

## Counts by category

| Category | blocker | major | minor | nit | total |
|---|---|---|---|---|---|
| contradiction | 5 | 3 | 6 | 0 | 14 |
| structure | 0 | 0 | 1 | 0 | 1 |
| latex | 0 | 0 | 0 | 1 | 1 |
| style | 0 | 0 | 2 | 0 | 2 |
| terminology | 0 | 0 | 2 | 1 | 3 |
| fact | 0 | 0 | 0 | 0 | 0 |
| grammar | 0 | 0 | 0 | 0 | 0 |
| **total** | **5** | **3** | **11** | **2** | **21** |

## The three most damaging contradictions

1. **F-cons-1 — Chapter 6's exception quick-reference table gets the return address wrong for
   three of the six retryable faults.** Bus Fault, Privilege Violation and Bus Protection are
   listed as returning to the *next* instruction, while six other passages in the same chapter,
   plus Chapter 14 and Appendix B, say they return to the *faulting* instruction. The table is
   exactly the artifact an implementer builds the exception unit from, and it is the one place
   that is wrong. Getting this backwards silently skips an instruction on every recoverable
   bus fault — a virtual-memory handler built from this table would corrupt every program it
   paged in for.

2. **F-cons-2 — register-supplied shift counts above 15 are simultaneously fully specified and
   architecturally undefined.** Chapter 9 spends fifteen lines defining exactly what a count of
   16-or-more does for each shift type, then three sections later declares the same case
   undefined; Chapter 13 splits the same way *internally*, with ten instruction boxes giving the
   defined rule and the SHFT box giving the undefined one, and Chapter 14 sides with undefined.
   Two conformant implementations could read this manual and disagree on a result that either
   is or is not part of the architectural contract.

3. **F-cons-4 — Chapter 7's worked encoding example is off by one on every register field.**
   "`ADDR r4, r5` uses R1=0x4, R2=0x4, R3=0x5" reads register *names* as if they were field
   values; Table~\ref{tab:register-encoding} and the generated register-format table both encode
   that instruction as R1=0x3, R2=0x3, R3=0x4. It sits in the chapter a toolchain author goes to
   for the encoding, immediately below the correct field-layout diagram, and it is wrong in the
   most plausible-looking way possible — an assembler written from it would emit the wrong
   register for every two-operand register-form ALU instruction.
