# Full-manual consistency review, Phase 3 (reviewer tag: con2)

Scope: every chapter and appendix in reading order, plus `docs/reference/generated/`, checked
against `docs/reference/STYLE.md` ("Approved decisions") and the Gate 2 rulings at the top of
`docs/reference/review/triaged.md`. No claim was checked against the implementation.

Mechanical LaTeX checks all passed and produced no findings: every `\ref` target has a matching
`\label` (0 dangling), every `\label` is unique (0 duplicates), every `\input` in `main.tex`
resolves and every file in `chapters/` is included, and every `lstlisting`, `instructionbox`,
`table`, `tabular`, `tabularx`, `figure`, `bytefield`, `verbatim`, `itemize`, `enumerate` and
`description` environment is balanced. Example numbering has no gaps and no duplicates in any
chapter that numbers examples.

## Findings

### F-con2-1
- file: chapters/03-registers.tex
- lines: 167-169
- severity: blocker
- category: contradiction
- claim: "The program counter register pair points to the next instruction to be executed. It is automatically incremented by 1 after each 16-bit word fetch. Since instructions are 32 bits (two words), the PC is incremented twice during a complete instruction fetch, advancing by 2 words total per instruction."
- evidence: chapters/02-cpu-architecture.tex:20-21,26,35 ("\reg{p} is unchanged", "Fetch bits 15--0 from memory address in \reg{p} + 1", "Advance \reg{pl} by 2 ... during Decode"); chapters/04-data-representation.tex:90-92; chapters/07-instruction-formats.tex:373-380; chapters/08-addressing-modes.tex:154-157 (PC-relative displacements are relative to the branching instruction's own address); chapters/15-control-flow.tex:176 ("\texttt{BRAN \#0} branches to itself")
- resolution: manual-wrong
- fix: Replace with: "The program counter register pair points to the instruction currently being fetched. It is not modified during either fetch phase; the low word \reg{pl} advances by 2 during the Decode and Register Fetch phase, and \reg{ph} is never incremented, so a wrap stays inside the segment." The two-increment model is the only surviving copy of the pre-Phase-2 text (F-arch-1) and it breaks the branch/link base address that Chapters 8 and 15 depend on.
- confidence: high

### F-con2-2
- file: chapters/12-instruction-summary.tex
- lines: 14
- severity: blocker
- category: contradiction
- claim: "\item \textbf{Coprocessor (\opcode{0F}, \opcode{2F}, \opcode{3F})}: Coprocessor interface"
- evidence: chapters/12-instruction-summary.tex:75 (its own table gives \opcode{2F} as "-- / Undocumented"); chapters/07-instruction-formats.tex:355-358; chapters/16-coprocessor-instructions.tex:36-38, 76; chapters/appendix-c-undocumented.tex:6-13, 59; triaged.md Gate 2 ruling H
- resolution: manual-wrong
- fix: Change the bullet to "\textbf{Coprocessor (\opcode{0F}, \opcode{3F})}: Coprocessor interface" and add "\opcode{2F} is undocumented (see Appendix~\ref{appendix:undocumented})". Note that chapters/12-instruction-summary.tex:11 correctly already excludes \opcode{2F} from the ALU ranges.
- confidence: high

### F-con2-3
- file: chapters/appendix-a-opcode-map.tex
- lines: 111-116
- severity: blocker
- category: contradiction
- claim: "\item Bit 3 = 0: Save result (\opcode{00}--\opcode{07}, \opcode{20}--\opcode{27}, \opcode{30}--\opcode{37}) \item Bit 3 = 1: Test only, discard result (\opcode{08}--\opcode{0F}, \opcode{28}--\opcode{2F}, \opcode{38}--\opcode{3F})"
- evidence: chapters/07-instruction-formats.tex:349-358 ("\texttt{0x\_0}--\texttt{0x\_6}: Save result ... \texttt{0x\_8}--\texttt{0x\_E}: Test only ... \texttt{0x\_F}: Coprocessor call ... not a test-only form" and "Opcodes \texttt{0x27} and \texttt{0x2F} are exceptions to this pattern"); chapters/12-instruction-summary.tex:122-125
- resolution: manual-wrong
- fix: Rewrite as "Bit 3 = 0 and low nibble \texttt{0}--\texttt{6}: save result (\opcode{00}--\opcode{06}, \opcode{20}--\opcode{26}, \opcode{30}--\opcode{36}). Bit 3 = 1 and low nibble \texttt{8}--\texttt{E}: test only, discard result (\opcode{08}--\opcode{0E}, \opcode{28}--\opcode{2E}, \opcode{38}--\opcode{3E}). Low nibble \texttt{F} is the coprocessor call (\opcode{0F}, \opcode{3F}), not a test form. Low nibble \texttt{7} is \mnemonic{LOAD} (\opcode{07}, \opcode{37}). \opcode{27} and \opcode{2F} are undocumented." This is the last place in the manual that still asserts the pre-Gate-2 "add 0x8" pattern across the whole low nibble.
- confidence: high

### F-con2-4
- file: chapters/appendix-b-timing.tex
- lines: 43-44
- severity: blocker
- category: contradiction
- claim: "\mnemonic{DMAR}/\mnemonic{DMAW} & $6 + \max(|n|, 1)$ \\ \mnemonic{DMAT}            & $6 + \max(2n, 1)$  \\"
- evidence: chapters/16-coprocessor-instructions.tex:511-513 ("\mnemonic{DMAR} and \mnemonic{DMAW} take $12 + \max(|n| - 1, 0)$ cycles, and \mnemonic{DMAT} takes $12 + \max(2n - 1, 0)$ cycles"), :577-580, :643-646, :702-706; chapters/appendix-b-timing.tex:45 (its own table gives \mnemonic{COPI}/\mnemonic{COPR} as 12 = 6 + 6); triaged.md Gate 2 ruling M ("DMA timing is restated on the 6-cycle-slot model")
- resolution: manual-wrong
- fix: Change the two rows to $12 + \max(|n| - 1, 0)$ and $12 + \max(2n - 1, 0)$. Appendix B was not updated to the 6-cycle-slot model; as written a zero-count \mnemonic{DMAR} takes 7 cycles here and 12 cycles in Chapter 16, and the appendix's own \mnemonic{COPI} row contradicts its DMA rows.
- confidence: high

### F-con2-5
- file: chapters/appendix-b-timing.tex
- lines: 46
- severity: blocker
- category: contradiction
- claim: "Exception-unit meta-instructions (\mnemonic{EXCP}, \mnemonic{RETE}, \mnemonic{RSET}, \mnemonic{ETFR}, \mnemonic{ETTR}) & 12 (6 + 6 dispatch) \\"
- evidence: chapters/16-coprocessor-instructions.tex:392-393 ("6 cycles for the call, plus a 6-cycle \texttt{RSTO} hold, plus 6 cycles for the exception unit to fetch the reset vector -- 18 cycles from the \mnemonic{RSET} fetch to the first instruction of the reset handler"); chapters/06-exceptions.tex:643-645 ("\texttt{RSTO} is asserted for six cycles before the reset-vector fetch begins")
- resolution: manual-wrong
- fix: Remove \mnemonic{RSET} from this row and give it its own row: "\mnemonic{RSET} & 18 (6 call + 6 \texttt{RSTO} hold + 6 reset-vector fetch)". The remaining row keeps \mnemonic{EXCP}, \mnemonic{RETE}, \mnemonic{ETFR}, \mnemonic{ETTR} at 12.
- confidence: high

### F-con2-6
- file: chapters/16-coprocessor-instructions.tex
- lines: 128
- severity: blocker
- category: contradiction
- claim: "\textbf{Timing:} 6 cycles, plus any instruction-fetch wait states."
- evidence: chapters/16-coprocessor-instructions.tex:54 (its own Common Semantics table: "Dispatch to coprocessor 1 then occupies one further complete 6-cycle slot ... so a coprocessor-1 operation completes 12 cycles after the call's own fetch began"); chapters/appendix-b-timing.tex:45 ("\mnemonic{COPI}/\mnemonic{COPR} & 12 (6 + 6 coprocessor dispatch)"); chapters/12-instruction-summary.tex:184-186 ("for 12 cycles in total before the next instruction is fetched")
- resolution: manual-wrong
- fix: Replace with: "\textbf{Timing:} 6 cycles for the processing-unit call, plus a further 6-cycle coprocessor dispatch slot (12 cycles in total), plus any bus wait states." Every other \mnemonic{COPI}-derived entry in Chapter 16 already states "6 cycles for the call plus a 6-cycle ... dispatch slot"; only the base \mnemonic{COPI}/\mnemonic{COPR} entry omits the dispatch slot.
- confidence: high

### F-con2-7
- file: chapters/appendix-e-quick-reference.tex
- lines: 12-14
- severity: blocker
- category: contradiction
- claim: "\emph{Flags}: uses the symbols defined in Table~\ref{tab:flag-effect-symbols} (Chapter~\ref{ch:reading-instructions}); \texttt{--} means all four flags preserved and \texttt{COP} means the effect is defined by the selected coprocessor."
- evidence: chapters/11-reading-instructions.tex:82-98 defines the complete symbol set as `*`, `0`, `1`, `-`, `S`, `U`; the Appendix E Flags column actually uses `NZCV` (lines 26-29, 38-39, 79-80, 83-84), `NZ (C,V=0)` (lines 30-31, 70-71, 87-90, 96-97), `COP` (40-41, 44-45, 63-64), `all` (74, 76) and `--`; none of `NZCV`, `NZ (C,V=0)`, `COP`, `all` or `--` is in the Chapter 11 set. STYLE.md §3 "Status flags" and Gate 1 "Flag tables keep the symbol form and gain `U`"
- resolution: manual-wrong
- fix: Convert the Appendix E Flags column to the Chapter 11 symbols: `NZCV` becomes `* * * *`; `NZ (C,V=0)` becomes `* * 0 0`; `--` becomes `- - - -`; `S` stays; `COP` (COPI, COPR, DIVS, DIVU, MULS, MULU) becomes `- - - -` for the call itself with a footnote that the selected coprocessor may define later effects; `all` (RETE, RSET) becomes `* * * *` with a footnote naming the restore/clear source. Split the column into four (N Z C V) as Chapters 13--15 do. The identical defect exists in chapters/12-instruction-summary.tex:24-99 (Flags column, and line 99 which likewise cites Table~\ref{tab:flag-effect-symbols} while using `NZCV`), and in the per-entry `\textbf{Flags:}` lines of Chapter 13 (lines 323, 409, 489, 572, 652, 737, 825, 1003, 1100, 1194) which read "N Z C V" / "N Z (C and V cleared)" rather than symbols.
- confidence: high

### F-con2-8
- file: chapters/06-exceptions.tex
- lines: 541-566
- severity: major
- category: contradiction
- claim: "\item \textbf{\texttt{0x0A}--\texttt{0x0F}:} Reserved for future privileged exceptions \item \textbf{\texttt{0x10}:} Level 5 hardware exception vector ... \item \textbf{\texttt{0x60}--\texttt{0xFF}:} User exception vectors (160 user-accessible trap vectors)"
- evidence: triaged.md Gate 2 ruling P ("Vector table has 256 entries: 0x00--0x5F reserved (96), 0x60--0xFF user (160)"). The string "256" appears nowhere in the manual in a vector context, and vectors \texttt{0x11}--\texttt{0x1F}, \texttt{0x21}--\texttt{0x2F}, \texttt{0x31}--\texttt{0x3F}, \texttt{0x41}--\texttt{0x4F} and \texttt{0x51}--\texttt{0x5F} are unclassified in this list, in chapters/01-introduction.tex:161-176 (which jumps straight from \texttt{0x50} to \texttt{0x60} and from word address \addr{0x0000A1} to \addr{0x0000C0}), and in chapters/06-exceptions.tex:570-598 (Table~\ref{tab:exception-quick-reference})
- resolution: manual-wrong
- fix: State the total once in Chapter 6 ("The vector table has 256 entries. Vectors \texttt{0x00}--\texttt{0x5F} (96 entries) are reserved for the CPU; vectors \texttt{0x60}--\texttt{0xFF} (160 entries) are user trap vectors.") and add a catch-all item "\texttt{0x11}--\texttt{0x5F} (excluding \texttt{0x20}, \texttt{0x30}, \texttt{0x40}, \texttt{0x50}): Reserved" to the list and a matching row to Table~\ref{tab:exception-quick-reference}. Add the same reserved band to Table~\ref{tab:intro-vector-table-regions} in Chapter 1 so the word-address column has no hole between \addr{0x0000A1} and \addr{0x0000C0}.
- confidence: high

### F-con2-9
- file: chapters/appendix-c-undocumented.tex
- lines: 6-13
- severity: major
- category: contradiction
- claim: "Two of the fourteen, \opcode{27} and \opcode{2F}, are the exception -- each has fully defined, tested behaviour (\opcode{27}: \texttt{rD} is loaded with the zero-extended 8-bit immediate; \opcode{2F}: the coprocessor command is the 8-bit immediate and the coprocessor ID is always 0, so the instruction is never privileged)"
- evidence: chapters/appendix-a-opcode-map.tex:143-147 lists \opcode{27} and \opcode{2F} in the same bullet list and then says "These opcodes execute in hardware, but their behavior is architecturally undefined and may change between CPU revisions"; chapters/13-alu-instructions.tex:130-131 ("Encodings outside the documented ALU forms execute with architecturally undefined results"); chapters/16-coprocessor-instructions.tex:36-38 (\opcode{2F} is "in the same class as" \opcode{08}/\opcode{09}/\opcode{0B}/\opcode{0D}, which are undefined)
- resolution: unclear
- fix: One side must give. Either Appendix A's blanket sentence is narrowed to "the twelve test-variant opcodes" and Chapter 13/Chapter 16 add the same carve-out for \opcode{27}/\opcode{2F}, or Appendix C drops the "fully defined, tested behaviour" claim and says only that these two are undocumented because they have no public assembly syntax. Gate 2 ruling H settles the classification (both are Undocumented, count stays 14) but not whether their observed behaviour is architectural.
- confidence: high

### F-con2-10
- file: chapters/appendix-a-opcode-map.tex
- lines: 62, 70
- severity: major
- category: contradiction
- claim: "\opcode{27}           & 100111          & LOAD              & Undocumented ... \opcode{2F}           & 101111          & COPI              & Undocumented"
- evidence: chapters/12-instruction-summary.tex:67, 75 give both as mnemonic "--"; chapters/13-alu-instructions.tex:871, 905-906 ("Opcode \texttt{0x27} has no public assembly syntax and is undocumented"); chapters/16-coprocessor-instructions.tex:76 ("Opcode 0x2F is Undocumented"); chapters/appendix-c-undocumented.tex:54, 59 give the Pattern column as "LOAD-like" and "COP-like", not the mnemonic
- resolution: manual-wrong
- fix: Set the Mnemonic column of both rows to `--` and move "LOAD-like" / "COP-like" into the Description column, matching Table~\ref{tab:complete-instruction-set}. Naming a mnemonic implies public assembly syntax that Chapters 13 and 16 explicitly deny.
- confidence: high

### F-con2-11
- file: chapters/14-memory-instructions.tex
- lines: 146, 150, 252, 256
- severity: major
- category: contradiction
- claim: "\textit{SIRC-1, all revisions}" and "\textbf{Instruction fields:}"
- evidence: STYLE.md "Approved decisions" (Gate 1 T-template) requires an applicability line under the title and names the field "Instruction Fields"; chapters/11-reading-instructions.tex:9-11, 19; every other entry in Chapters 13, 15, 16 and 17 uses "\textbf{Applicability:} SIRC-1, all revisions" and "\textbf{Instruction Fields:}" (for example chapters/13-alu-instructions.tex:275, 279; chapters/15-control-flow.tex:139, 143; chapters/16-coprocessor-instructions.tex:74, 78; chapters/17-meta-instructions.tex:43, 47)
- resolution: manual-wrong
- fix: In both Chapter 14 entries replace `\textit{SIRC-1, all revisions}` with `\textbf{Applicability:} SIRC-1, all revisions` and `\textbf{Instruction fields:}` with `\textbf{Instruction Fields:}`. Chapter 14 is the only chapter of the five that did not adopt the Gate 1 template labels.
- confidence: high

### F-con2-12
- file: chapters/16-coprocessor-instructions.tex
- lines: 235-281, 283-319, 321-357, 359-396, 398-457
- severity: major
- category: structure
- claim: "\textbf{Description:} Triggers a software exception at the specified user vector, normally 0x60--0xFF."
- evidence: chapters/11-reading-instructions.tex:25-27 lists Operation as a mandatory field ("Pseudocode for the architectural effect of the instruction") ahead of Description; STYLE.md §4 canonical order; every entry in Chapters 13, 15 and 17 has an Operation block, as do \mnemonic{DMAR}, \mnemonic{DMAW}, \mnemonic{DMAT}, \mnemonic{MULU}/\mnemonic{MULS} and \mnemonic{DIVU}/\mnemonic{DIVS} in this same chapter
- resolution: manual-wrong
- fix: Add an `\textbf{Operation:}` pseudocode block to the \mnemonic{EXCP}, \mnemonic{WAIT}, \mnemonic{RETE}, \mnemonic{RSET} and \mnemonic{ETFR}/\mnemonic{ETTR} entries, placed before Description. The normative effects already exist in prose in these entries and in Chapter 6 (exception entry side effects, RETE restore steps, reset-vector sequence, link-register transfer) and only need restating as pseudocode.
- confidence: high

### F-con2-13
- file: chapters/16-coprocessor-instructions.tex
- lines: 106-116
- severity: minor
- category: structure
- claim: "\textbf{Description:} ... \textbf{Operation:}"
- evidence: STYLE.md §4 canonical field order is Operation then Description; chapters/11-reading-instructions.tex:25-27; chapters/13-alu-instructions.tex:306-321, chapters/14-memory-instructions.tex:190-206 and chapters/15-control-flow.tex:165-177 all place Operation before Description
- resolution: manual-wrong
- fix: Swap the two blocks so Operation precedes Description. The same inversion occurs in this chapter at \mnemonic{DMAR} (lines 546-561), \mnemonic{DMAW} (610-627), \mnemonic{DMAT} (672-684), \mnemonic{MULU}/\mnemonic{MULS} (789-806) and \mnemonic{DIVU}/\mnemonic{DIVS} (850-863).
- confidence: high

### F-con2-14
- file: chapters/14-memory-instructions.tex
- lines: 208-211
- severity: minor
- category: structure
- claim: "\textbf{Write-back:} If the condition is true, writes the loaded value to \texttt{rD}. ... \textbf{Flags:} Preserved."
- evidence: STYLE.md §4 canonical order is `Flags` then `Write-back`; chapters/11-reading-instructions.tex:28-29; chapters/13-alu-instructions.tex:323-325 and 908-910 put Flags first
- resolution: manual-wrong
- fix: Swap the Flags and Write-back blocks so Flags comes first. Same inversion at chapters/14-memory-instructions.tex:314-317 (\mnemonic{STOR}); chapters/15-control-flow.tex:179-181, 245-247, 307-309, 379-382, 457-460, 531-533, 598-600; chapters/16-coprocessor-instructions.tex:118-120, 259-261, 308-310, 345-347, 384-386, 436-439, 563-568, 629-634, 686-690, 808-810, 865-868. Chapter 13 and Chapter 17 are the only ones that follow the approved order, so this is a 3-chapter fix, not a change to the rule.
- confidence: high

### F-con2-15
- file: chapters/14-memory-instructions.tex
- lines: 218-222
- severity: minor
- category: structure
- claim: "\textbf{Timing:} 6 cycles when the memory read is acknowledged without wait states ... \textbf{Condition field:} If the condition is false, no memory read occurs"
- evidence: STYLE.md §4 canonical order is `Condition codes` (now `Condition field`) then `Timing`; chapters/11-reading-instructions.tex:32-35; chapters/13-alu-instructions.tex:329-331, chapters/15-control-flow.tex:185-187 and chapters/16-coprocessor-instructions.tex:125-128 all place Condition field before Timing
- resolution: manual-wrong
- fix: Swap the Timing and Condition field blocks in both Chapter 14 entries (lines 218-222 and 324-328). Chapter 14 is the only chapter with this ordering.
- confidence: medium

### F-con2-16
- file: chapters/13-alu-instructions.tex
- lines: 210-212, 266-267
- severity: major
- category: contradiction
- claim: "\item \textbf{Shift type and count:} \texttt{NUL}, \texttt{LSL}, \texttt{LSR}, \texttt{ASL}, \texttt{ASR}, \texttt{RTL}, \texttt{RTR}" ... "All six shift types are supported, plus \mnemonic{NUL}: NUL, LSL, LSR, ASL, ASR, RTL, and RTR. \texttt{SHFT rD, NUL \#0} sets the flags from the unshifted value of \texttt{rD}."
- evidence: chapters/07-instruction-formats.tex:308-326 (Table~\ref{tab:shift-types}) gives shift code 000 the mnemonic "--"; chapters/09-shift-operations.tex:21-39 (Table~\ref{tab:shift-encoding}) also gives "--"; chapters/14-memory-instructions.tex:158-160 and 263-265 name code \texttt{000} "none". `NUL` appears 12 times in Chapter 13 and nowhere else in the manual
- resolution: manual-wrong
- fix: Add the assembler mnemonic to both shift-type tables: change the Mnemonic cell of shift code 000 from `--` to `NUL` in Table~\ref{tab:shift-types} (07-instruction-formats.tex:314) and Table~\ref{tab:shift-encoding} (09-shift-operations.tex:27), and add one sentence to Chapter 9 defining \mnemonic{NUL} as the no-shift form that still lets the shifter drive the flags. Otherwise the reader meets `NUL` in Chapter 13 with no definition and no forward reference.
- confidence: high

### F-con2-17
- file: chapters/11-reading-instructions.tex
- lines: 46-75
- severity: major
- category: structure
- claim: "\texttt{rS}      & Source general-purpose register"
- evidence: The notation table defines `rD`, `rS`, `rS1`, `rS2`, `addr / src`, `dest`, `#offset / #disp` but not `rO`; `rO` is used as the offset/index register throughout chapters/08-addressing-modes.tex:42, 44, 46, 68-70, 89-92, 126-127 and chapters/14-memory-instructions.tex:20, 25-27, 42-43, 59-65, 156, 174-188, 261-292; chapters/15-control-flow.tex:34, 36, 344-345, 422-423, 502-503 uses `rS` for the same displacement-register role in \mnemonic{LDEA}/\mnemonic{LDEL}/\mnemonic{LJMP}/\mnemonic{LJSR}, and chapters/08-addressing-modes.tex:91-93 uses `rO` for those very instructions
- resolution: manual-wrong
- fix: Add a row to Table~\ref{tab:instruction-description-notation}: "\texttt{rO} & Offset or index general-purpose register supplying a register displacement in a memory or effective-address form", then make Chapter 15 use `rO` for the \mnemonic{LDEA}/\mnemonic{LDEL}/\mnemonic{LJMP}/\mnemonic{LJSR} displacement register so it matches Chapter 8's legality table and Chapter 14. Grep pattern for the Chapter 15 half: `(rS, src)` -> `(rO, src)`, `LJMP src, rS` -> `LJMP src, rO`, `LJSR src, rS` -> `LJSR src, rO`, `\texttt{rS}, a general-purpose register supplying the displacement` -> `\texttt{rO}, a general-purpose register supplying the displacement`.
- confidence: high

### F-con2-18
- file: chapters/14-memory-instructions.tex
- lines: 479
- severity: major
- category: contradiction
- claim: "For flag updates based on a shift result, use an ALU instruction or the \mnemonic{SHFT} meta-instruction (see Chapter~\ref{ch:meta-instructions})."
- evidence: The \mnemonic{SHFT} instruction entry is in chapters/13-alu-instructions.tex:200-269; chapters/09-shift-operations.tex:550 ("See Chapter~\ref{ch:alu-instructions} for the full \mnemonic{SHFT} instruction entry"); chapters/17-meta-instructions.tex:16 (the Chapter 17 cross-reference table itself points \mnemonic{SHFT} at Chapter~\ref{ch:alu-instructions})
- resolution: manual-wrong
- fix: Change `Chapter~\ref{ch:meta-instructions}` to `Chapter~\ref{ch:alu-instructions}` on this line. Chapter 17 contains only \mnemonic{NOOP} plus a cross-reference table, so it is never the primary documentation for \mnemonic{SHFT}.
- confidence: high

### F-con2-19
- file: chapters/08-addressing-modes.tex
- lines: 69-70
- severity: major
- category: contradiction
- claim: "Post-Increment     & \texttt{(\#offset, addr)+}, \texttt{(addr)+}, \texttt{(rO, addr)+} & ... & LOAD, LDEL"
- evidence: chapters/08-addressing-modes.tex:93 (the same chapter's legality table lists \texttt{LJSR (\#offset, src)+} and \texttt{LJSR (rO, src)+} as legal control-flow forms); chapters/15-control-flow.tex:52-53, 69-70, 571-573 (\mnemonic{LJSR} post-increment forms); chapters/16 n/a
- resolution: manual-wrong
- fix: Change the Post-Increment "Legal families" cell to "LOAD, LDEL, LJSR (which assembles to LDEL)". For symmetry, change the Pre-Decrement cell to "STOR, LDEA (and the LDEA-derived meta-instructions)". As written, Table~\ref{tab:addressing-mode-matrix} and Table~\ref{tab:legal-modes-by-family} in the same chapter disagree about whether \mnemonic{LJSR} may take a post-increment operand.
- confidence: high

### F-con2-20
- file: generated/register-format-encodings.tex
- lines: 44-45
- severity: major
- category: contradiction
- claim: "\texttt{SHFT r1, ASL \#3}           & 0x25        & 0x1         & 0x0         & 0x0         & 0           & 011         & 0x3         & 0b10        & 0x0           & \texttt{0x94400CE0}"
- evidence: This row appears in the table titled "Register Format Encoding Examples" under chapters/07-instruction-formats.tex:254 (\section{Register Format}), but opcode \opcode{25} is a Short Immediate opcode: chapters/appendix-a-opcode-map.tex:104-108 ("Opcode bits 5--4 = 10: Short Immediate with Shift (\opcode{20}--\opcode{2F})"), chapters/07-instruction-formats.tex:337-341, chapters/12-instruction-summary.tex:111-115. The R2 and R3 columns (0x0, 0x0) misdescribe bits 21--14, which in this encoding are the 8-bit immediate; the same instruction is already shown correctly in generated/short-immediate-format-encodings.tex:25-26
- resolution: manual-wrong
- fix: Remove the \mnemonic{SHFT} row from the register-format generator output, and delete the now-orphaned note "SHFT is a meta-instruction using ORRI opcode \opcode{25} with AF=10" at chapters/07-instruction-formats.tex:268 and the reference to "(see the \texttt{SHFT} row)" at :265. The \mnemonic{SHFT} example belongs only in the short-immediate table.
- confidence: high

### F-con2-21
- file: chapters/16-coprocessor-instructions.tex
- lines: 166
- severity: major
- category: contradiction
- claim: "0x1         & Exception Unit       & Yes               & 0x0--0x1, 0x9--0xD (software); 0xE--0xF (internal) & Handles reset, faults, interrupts, traps, and links"
- evidence: chapters/06-exceptions.tex:718-726 ("The following opcodes are used internally by the exception unit and are not directly accessible via user instructions: ... \item \textbf{None (\texttt{0x0}):} No operation, used when no exception is pending"); chapters/06-exceptions.tex:696-712 (Table~\ref{tab:exception-coprocessor-summary} lists only \texttt{0x1}, \texttt{0x9}--\texttt{0xD} as software-issuable)
- resolution: manual-wrong
- fix: Change the Operations cell to "\texttt{0x1}, \texttt{0x9}--\texttt{0xD} (software); \texttt{0x0}, \texttt{0xE}--\texttt{0xF} (internal)" and extend the note at line 176 to "Exception-unit operation nibble \texttt{0x0} is the internal no-operation encoding and operation nibbles \texttt{0x2}--\texttt{0x8} are architecturally undefined."
- confidence: high

### F-con2-22
- file: chapters/01-introduction.tex
- lines: 69
- severity: major
- category: structure
- claim: "Addressing Modes            & 7                      \\"
- evidence: triaged.md F-arch-12 (status: phase4, "Add a cross-reference from this row to the addressing-mode chapter's numbered list"); Gate 2 ruling N; chapters/08-addressing-modes.tex:6-29 now settles the list at seven (Immediate, Register Direct, Address Register Direct, Indirect Immediate, Indirect Register, Post-Increment, Pre-Decrement) in Table~\ref{tab:addr-modes}
- resolution: manual-wrong
- fix: The count itself is now correct and needs no change. Add the cross-reference F-arch-12 asked for: "Addressing Modes & 7 (see Table~\ref{tab:addr-modes})". This is the only outstanding half of F-arch-12.
- confidence: high

### F-con2-23
- file: chapters/11-reading-instructions.tex
- lines: 24
- severity: major
- category: structure
- claim: "\item[Operands] The permitted operand kinds and addressing modes for the instruction."
- evidence: triaged.md F-sum-11 (status: phase4, "Chapter 11 should name the closed set (or point at the Chapter 8 table by label) so that entries in Chapters 13-17 cannot invent mode names"); Gate 2 ruling N; chapters/08-addressing-modes.tex:12-29 (Table~\ref{tab:addr-modes}) is now the settled seven-mode list
- resolution: manual-wrong
- fix: Replace with: "\item[Operands] The permitted operand kinds and addressing modes for the instruction. Addressing-mode names come from the closed set in Table~\ref{tab:addr-modes} (Chapter~\ref{ch:addressing-modes}): Immediate, Register Direct, Address Register Direct, Indirect Immediate, Indirect Register, Post-Increment, and Pre-Decrement. An entry must not introduce a mode name outside that set; the 8-bit short-immediate encoding is a width variant of Immediate, not a mode." This is the outstanding half of F-sum-11.
- confidence: high

### F-con2-24
- file: chapters/04-data-representation.tex
- lines: 98-109
- severity: major
- category: contradiction
- claim: "Vector entry \texttt{V} starts at word address \texttt{system\_ram\_offset + V * 2}, where \texttt{system\_ram\_offset} is the internal vector-table base and is not program-visible"
- evidence: chapters/06-exceptions.tex:543-546 ("The exception vectors are stored starting at address \addr{0x0} ... The high word is stored first at \texttt{vector * 2}, followed by the low word at \texttt{vector * 2 + 1}") and :469-470 ("The vector-table address is computed by multiplying the vector ID by 2"); chapters/01-introduction.tex:158 ("The CPU expects the exception vector table to begin at word address \addr{0x000000}"); chapters/06-exceptions.tex:577-593 (Table~\ref{tab:exception-quick-reference} gives absolute addresses \addr{0x0000}--\addr{0x01FF} with no base term)
- resolution: unclear
- fix: Pick one model and state it in all three places. If the base is architecturally fixed at zero, drop `system_ram_offset` from Chapter 4 and say "Vector entry V starts at word address V * 2". If the base is a real internal register, add the same "+ \texttt{system\_ram\_offset}" qualifier to chapters/06-exceptions.tex:543-546 and :469-470 and add a note to Table~\ref{tab:exception-quick-reference} and Table~\ref{tab:intro-vector-table-regions} that their addresses assume a base of zero. As it stands Chapter 4 introduces a base term that no other chapter acknowledges.
- confidence: medium

### F-con2-25
- file: chapters/16-coprocessor-instructions.tex
- lines: 305
- severity: minor
- category: structure
- claim: "Interrupts masked by the \texttt{HIE} bits do not wake the CPU, and faults and software exceptions cannot occur while waiting."
- evidence: `HIE` occurs exactly once in the manual and is never expanded; chapters/05-status-register.tex:110-119 names them "Hardware Interrupt Enable Bits (E1-E4)" and the rest of the manual uses \texttt{SR.E1}--\texttt{SR.E4} (chapters/06-exceptions.tex:589-592, chapters/appendix-e-quick-reference.tex:123-126)
- resolution: manual-wrong
- fix: Replace `\texttt{HIE}` with "the hardware interrupt enable bits (\texttt{SR.E1}--\texttt{SR.E4})".
- confidence: high

### F-con2-26
- file: chapters/02-cpu-architecture.tex
- lines: 17-57, 165, 623
- severity: minor
- category: contradiction
- claim: "The ALU and AU operate in parallel during Stage 4" ... "SYNC & Asserted while phase 0 (Instruction Fetch, high word) of an instruction or exception-unit dispatch is active."
- evidence: The chapter's own `enumerate` at lines 17-57 numbers the phases 1--6 (Instruction Fetch High Word = 1, Execute and Address Calculation = 4), while lines 373, 569-570, 614 and 623 use 0-based numbering ("phase 0" = Instruction Fetch High Word, "resets to phase 0"); chapters/appendix-b-timing.tex:61-72 also uses 0-based ("the vector's high word is fetched at phase 0 ... Privilege Violation is detected at phase 2; Segment Overflow is detected at phase 2 or phase 3") while chapters/appendix-b-timing.tex:17-24 repeats the 1--6 enumerate; chapters/12-instruction-summary.tex:175-182 repeats the 1--6 enumerate
- resolution: manual-wrong
- fix: Pick 0-based numbering (it is what the fault-detection prose and the SYNC definition need) and renumber the three copies of the phase list to "Phase 0 ... Phase 5" using a `description` list rather than `enumerate`, in chapters/02-cpu-architecture.tex:17-57, chapters/appendix-b-timing.tex:17-24 and chapters/12-instruction-summary.tex:175-182. Then change "during Stage 4" at 02-cpu-architecture.tex:165 to "during phase 3 (Execute and Address Calculation)". A reader currently cannot tell whether "phase 2" means Instruction Fetch (Low Word) or Decode.
- confidence: high

### F-con2-27
- file: chapters/10-condition-codes.tex
- lines: 181-186, 200-210
- severity: minor
- category: contradiction
- claim: "rA = rB                 & 1          & *          & ==                 & 0001          & Equal            \\ ... rA $\leq$ rB (unsigned) & *          & *          & LO                 & 1010          & Lower or Same    \\"
- evidence: chapters/11-reading-instructions.tex:88 defines `*` as "Updated from the instruction result"; STYLE.md §3 "Status flags" fixes the symbol set at `*`, `0`, `1`, `-`, `S`, `U`. Chapter 10 reuses `*` with the opposite meaning ("don't care") in two truth tables, and the LO row degenerates to `* *`, which states no predicate at all
- resolution: manual-wrong
- fix: Replace `*` with `X` in both truth tables and add a one-line legend under each: "\texttt{X} = the flag value does not affect this relationship." Give the LO row a real predicate: "rA $\leq$ rB (unsigned) & 1 or -- & 0 or -- & LO & 1010 & Lower or Same (C = 0 OR Z = 1)" or drop the Z/C columns for that row and cite the expression. Do not extend Chapter 11's symbol table with a "don't care" symbol unless the author wants it manual-wide.
- confidence: medium

### F-con2-28
- file: chapters/05-status-register.tex
- lines: 11
- severity: minor
- category: terminology
- claim: "\item \textbf{Upper Byte (bits 15--8):} Control and mode flags -- accessible only in supervisor mode"
- evidence: STYLE.md §1 ("high word / high byte / low word / low byte" preferred; "upper word / upper byte" avoided; 43 "high" vs 12 "upper"); chapters/03-registers.tex:184-186 uses "Low Byte"/"High Byte" for the same two halves; chapters/appendix-e-quick-reference.tex:109 uses "The high byte (bits 15--8)"
- resolution: manual-wrong
- fix: `s/Upper Byte (bits 15--8)/High Byte (bits 15--8)/`. For consistency with Chapter 3 also change "Lower Byte (bits 7--0)" at 05-status-register.tex:10 to "Low Byte (bits 7--0)". This is the only surviving "upper byte/word" in the manual.
- confidence: high

### F-con2-29
- file: chapters/13-alu-instructions.tex
- lines: 277
- severity: minor
- category: style
- claim: "\textbf{Opcodes:} 0x00 (Imm), 0x20 (Short Imm), 0x30 (Reg)"
- evidence: STYLE.md "Approved decisions -- Notation" ("Every hex literal in prose or a table cell takes \opcode{XX} (opcode bytes) ... No hex literal remains in body font"); chapters/14-memory-instructions.tex:148, 254 and chapters/12-instruction-summary.tex, chapters/appendix-a-opcode-map.tex, chapters/appendix-e-quick-reference.tex all use `\opcode{}`
- resolution: manual-wrong
- fix: Apply the `\opcode{}` macro to the bare opcode literals on the Opcodes lines and in the Encoding columns of the legal-forms tables. Grep pattern for the affected chapters: in chapters/13-alu-instructions.tex lines 57-89 and 277, 365, 446, 529, 608, 693, 781, 871, 958, 1056, 1150; chapters/15-control-flow.tex lines 33-40 and 335, 412; chapters/16-coprocessor-instructions.tex lines 25-26, 33, 36, 76. Replace `0xNN` with `\opcode{NN}` in those cells and lines only (leave `lstlisting` contents alone).
- confidence: high

### F-con2-30
- file: generated/immediate-format-encodings.tex
- lines: 8-47
- severity: minor
- category: style
- claim: "\texttt{ADDI r1, \#100}     & 0x00            & 0x1          & 0x0064                & 0b01        & 0x0           & \texttt{0x00401910}"
- evidence: STYLE.md §3 "Binary literals: bare digit groups in tables (`00`, `10`, `111`), no `0b` prefix. The two existing `0b10` instances in `07-instruction-formats.tex` are the outliers"; chapters/07-instruction-formats.tex:88-99, 111-112, 220-231, 264-265 now use bare `00`/`01`/`10`/`11`; chapters/08-addressing-modes.tex:404-408 (Table~\ref{tab:addr-reg-encoding}) uses bare digits; chapters/11-reading-instructions.tex:116-119 uses bare digits
- resolution: manual-wrong
- fix: In the generator that produces `generated/*.tex`, drop the `0b` prefix from the AF column so the rendered value is `01`, `10`, `11`. 22 occurrences across generated/immediate-format-encodings.tex, generated/short-immediate-format-encodings.tex and generated/register-format-encodings.tex. Grep pattern: `& 0b\([01][01]\) *&` -> `& \1        &`.
- confidence: high

### F-con2-31
- file: chapters/17-meta-instructions.tex
- lines: 41-88
- severity: minor
- category: structure
- claim: "\section{NOOP -- No Operation}\n\n\textbf{Applicability:} SIRC-1, all revisions"
- evidence: Every other instruction entry in Chapters 13--16 is wrapped in `\begin{instructionbox}{...}` (32 entries); chapters/11-reading-instructions.tex:9-12 describes an entry as beginning "with a title naming the mnemonic or mnemonic family, followed immediately by one applicability line"
- resolution: manual-wrong
- fix: Wrap the \mnemonic{NOOP} entry in `\begin{instructionbox}{NOOP -- No Operation}` ... `\end{instructionbox}` so it renders like the other 32 entries, keeping the surrounding `\section{NOOP -- No Operation}` or dropping it as the other chapters do.
- confidence: high

### F-con2-32
- file: chapters/15-control-flow.tex
- lines: 489-543
- severity: minor
- category: structure
- claim: "\textbf{Privilege:} Same as \mnemonic{LDEA}. Protected-mode jumps must preserve the high word of \reg{p}.\n\n\end{instructionbox}"
- evidence: chapters/11-reading-instructions.tex:37-39 lists Example/Examples as an entry field; 24 of the 33 instruction entries have one. Missing entirely from \mnemonic{LJMP} (15-control-flow.tex:489-543), \mnemonic{WAIT} (16:283-319), \mnemonic{RETE} (16:321-357), \mnemonic{RSET} (16:359-396), \mnemonic{DMAR} (16:521-583), \mnemonic{DMAW} (16:585-649), \mnemonic{DMAT} (16:651-712), \mnemonic{MULU}/\mnemonic{MULS} (16:764-822), \mnemonic{DIVU}/\mnemonic{DIVS} (16:824-885)
- resolution: manual-wrong
- fix: Add one numbered `Example N-M:` block to each of the nine entries, continuing the existing sequences (Chapter 15 resumes at 15-7, Chapter 16 at 16-4). \mnemonic{LJMP} in particular has legal-form syntax in Table~\ref{tab:control-flow-legal-forms} that no example illustrates.
- confidence: medium

### F-con2-33
- file: chapters/09-shift-operations.tex
- lines: 343-404, 406-435, 482-512
- severity: minor
- category: structure
- claim: "\subsection{Examples}\n\n\textbf{Default Behavior (ALU flags):}\n\begin{lstlisting}"
- evidence: STYLE.md "Approved decisions -- Prose" ("Each `Example:` block ... becomes `Example N-M:`"); the same chapter numbers Examples 9-1 through 9-6 at lines 97, 128, 147, 175, 208, 239, so a section literally titled "Examples" containing three unnumbered listings is internally inconsistent
- resolution: manual-wrong
- fix: Number the listings in \S Common Use Cases, \S Multi-Word Shifts and \S Status Flag Update Control as Examples 9-7 onward. The same omission applies to chapters/appendix-b-timing.tex:88-122 (\S Program Timing Examples, three unnumbered listings that should be Examples B-1 to B-3) and chapters/appendix-b-timing.tex:153-200 (four optimization listings). Chapters 2, 4, 6, 7, 12 and Appendices A, C and E have no numbered examples at all; that is acceptable only if the rule is scoped to chapters that already number some.
- confidence: medium

### F-con2-34
- file: chapters/appendix-b-timing.tex
- lines: 146-148
- severity: minor
- category: contradiction
- claim: "Every instruction performs two instruction-fetch bus accesses; a load or store adds one data access; a coprocessor dispatch adds two vector-fetch accesses."
- evidence: chapters/16-coprocessor-instructions.tex:54, 63-64 ("The coprocessor-call instruction itself uses the normal 6 execution phases and performs no data-memory bus access" and coprocessor calls "do not perform data-memory access"); vector fetches belong to exception dispatch only -- chapters/06-exceptions.tex:472, 494-496 and chapters/appendix-b-timing.tex:61-64 in this same appendix ("A hardware-exception or fault dispatch occupies one 6-cycle slot: the vector's high word is fetched at phase 0, the low word at phase 1")
- resolution: manual-wrong
- fix: Replace "a coprocessor dispatch adds two vector-fetch accesses" with "an exception or fault dispatch adds two exception-vector-fetch accesses; a coprocessor dispatch performs no bus access of its own".
- confidence: high

### F-con2-35
- file: chapters/15-control-flow.tex
- lines: 344, 422, 502, 560
- severity: minor
- category: terminology
- claim: "\item \textbf{r3 (bits 17--14, register forms only):} \texttt{rS}, a general-purpose register supplying the displacement."
- evidence: STYLE.md §1 ("R1/R2/R3 (upper case, no macro) ... Reserved for abstract *field-position* labels in instruction-format diagrams and field-order prose (Ch.7). Never use for a concrete register operand."); chapters/07-instruction-formats.tex:210 ("Register 3 (bits 17--18)" upper case); chapters/14-memory-instructions.tex:156, 261 ("\textbf{R3 (bits 17--14, Register format ...)}") and chapters/16-coprocessor-instructions.tex:84 ("\textbf{Register 3 (Register format, bits 17--14)}") both use the upper-case form
- resolution: manual-wrong
- fix: `s/\\textbf{r3 (bits 17--14/\\textbf{R3 (bits 17--14/` in chapters/15-control-flow.tex (4 occurrences). Lower-case `r3` reads as the concrete general-purpose register \reg{r3}, which is not what is meant.
- confidence: high

### F-con2-36
- file: chapters/appendix-e-quick-reference.tex
- lines: 186, 190, 194
- severity: nit
- category: latex
- claim: "Complete opcode map (all 64 opcodes) & Appendix~\ref{appendix:opcode-map}    & ---                                \\"
- evidence: The Label column is `---` for three entries that do have labels: `tab:opcode-map` (chapters/appendix-a-opcode-map.tex:93), `tab:addr-modes` (chapters/08-addressing-modes.tex:28) and `tab:meta-instruction-cross-reference` (chapters/17-meta-instructions.tex:38). Other rows in the same table do cite labels
- resolution: manual-wrong
- fix: Fill in `\ref{tab:opcode-map}`, `\ref{tab:addr-modes}` and `\ref{tab:meta-instruction-cross-reference}` in the Label column of those three rows. The remaining `---` entries (exception vector table, instruction timing) genuinely have no labelled table.
- confidence: high

### F-con2-37
- file: chapters/appendix-a-opcode-map.tex
- lines: 163
- severity: nit
- category: terminology
- claim: "OR                 & \opcode{05}         & \opcode{25}                    & \opcode{35}        \\"
- evidence: chapters/13-alu-instructions.tex:29 and 155 name the family \mnemonic{ORR}, matching \mnemonic{ORRI}/\mnemonic{ORRR}; every other row of this table uses the three-letter family name (ADD, ADC, SUB, SBC, AND, XOR, CMP, TSA, TSX, COP)
- resolution: manual-wrong
- fix: Change `OR` to `ORR` in the Operation column.
- confidence: high

### F-con2-38
- file: chapters/14-memory-instructions.tex
- lines: 424
- severity: nit
- category: contradiction
- claim: "\item \textbf{ASL} -- Arithmetic Shift Left (same as LSL)"
- evidence: chapters/09-shift-operations.tex:30 ("Same data result as LSL; sets V on signed overflow") and :137-145, :339 ("For \textbf{ASL}: Set if the sign bit changes during the shift (signed overflow)")
- resolution: manual-wrong
- fix: Change to "\item \textbf{ASL} -- Arithmetic Shift Left (same data result as LSL; sets V on signed overflow)". "Same as LSL" drops the one behavioural difference between the two.
- confidence: high

### F-con2-39
- file: chapters/03-registers.tex
- lines: 184
- severity: nit
- category: contradiction
- claim: "\item \textbf{Low Byte (bits 7--0):} Condition flags, accessible in both supervisor and protected modes"
- evidence: chapters/05-status-register.tex:10 (the Gate-2-corrected wording: "readable in both modes, and updated as a side effect of ALU and shift instructions in both modes; a direct write to \reg{sr} is privileged and raises a privilege violation fault in protected mode"); chapters/03-registers.tex:189 immediately afterwards says the same thing correctly; chapters/appendix-e-quick-reference.tex:110-111
- resolution: manual-wrong
- fix: Replace "accessible in both supervisor and protected modes" with "readable in both modes and updated as a side effect of ALU and shift instructions in both modes; direct writes to \reg{sr} are privileged". Bare "accessible" reads as "writable" and contradicts the sentence five lines below it.
- confidence: high

### F-con2-40
- file: chapters/07-instruction-formats.tex
- lines: 51
- severity: nit
- category: contradiction
- claim: "\item[Additional Flags (2 bits)] Used in memory operations to specify address register pairs (a, p, s, l) and in ALU operations to specify how the status register is updated."
- evidence: The encoded order is l, a, s, p -- chapters/07-instruction-formats.tex:87-92 and 218-224 (00 = l, 01 = a, 10 = s, 11 = p); chapters/08-addressing-modes.tex:399-413 (Table~\ref{tab:addr-reg-encoding}); chapters/14-memory-instructions.tex:164-165; chapters/15-control-flow.tex:341-342
- resolution: manual-wrong
- fix: Change "(a, p, s, l)" to "(l, a, s, p)" so the parenthetical matches the encoding order used everywhere else.
- confidence: high

## Counts by category

- contradiction: 24 (F-con2-1 to F-con2-10, 16, 18, 19, 20, 21, 24, 26, 27, 34, 38, 39, 40, plus the two ordering findings F-con2-13/14 counted under structure)
- structure: 11 (F-con2-12, 13, 14, 15, 17, 22, 23, 25, 31, 32, 33)
- terminology: 3 (F-con2-28, 35, 37)
- style: 2 (F-con2-29, 30)
- latex: 1 (F-con2-36)
- fact: 0
- grammar: 0

## Counts by severity

- blocker: 7
- major: 16
- minor: 12
- nit: 5
- total: 40

## The three most damaging contradictions

1. **F-con2-1** -- Chapter 3 still describes the program counter as incrementing once per word fetch. Chapters 2, 4, 7, 8 and 15 all now depend on \reg{p} holding the *current* instruction's address during decode; the surviving Chapter 3 text moves every PC-relative branch target and every \mnemonic{BRSR}/\mnemonic{LDEL} link value by two words.
2. **F-con2-3 with F-con2-2** -- Appendix A's "bit 3 = 1 means test only, \opcode{08}--\opcode{0F}" and Chapter 12's "Coprocessor (\opcode{0F}, \opcode{2F}, \opcode{3F})" both re-assert the pre-Gate-2 opcode pattern that Chapter 7, Chapter 16 and Appendix C were rewritten to overturn. A reader following Appendix A would decode \opcode{0F}/\opcode{3F} as flag-only ALU test instructions instead of coprocessor calls.
3. **F-con2-4 with F-con2-5 and F-con2-6** -- The three cycle-count sources disagree with each other. Appendix B gives \mnemonic{DMAR} 7 cycles where Chapter 16 gives 12, gives \mnemonic{RSET} 12 where Chapter 16 gives 18, and Chapter 16's own \mnemonic{COPI} entry says 6 where its Common Semantics table, Chapter 12 and Appendix B all say 12. Nothing in the manual lets a reader decide which number to build to.
