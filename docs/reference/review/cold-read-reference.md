# Cold read: Architecture Reference Manual (tag: CR)

Read from the PDF text (PDF newer than chapters). Line numbers are approximate chapter-source lines.

### F-CR-1
- file: reference/chapters/notation-and-glossary.tex
- lines: 55-160
- severity: minor
- category: structure
- claim: "Instruction entries in this manual use the canonical SIRC assembly notation. This summary is enough to read and write the listings in the manual."
- evidence: none
- resolution: n/a
- fix: The Assembly Language Summary sits in the back-matter Notation chapter (page xix) but is the first thing a first-time reader needs, and the Preface never points to it. Add a sentence to the Preface "How to Read" saying "read the Notation and Glossary before Chapter 3; listings use its syntax". Its example listing also uses vector 0x40, RETE and a .ORG 0x0080 before vectors (Ch 6) are defined; add one line explaining that .ORG 0x0080 is word address 2 x 0x40.
- confidence: high

### F-CR-2
- file: reference/chapters/notation-and-glossary.tex
- lines: 120-130
- severity: minor
- category: structure
- claim: "Despite their names, all three directives occupy exactly one 4-byte (2-word) program slot"
- evidence: none
- resolution: n/a
- fix: A reader meets byte/word/slot arithmetic (.DB assembling to "00 00 00 AB") before the big-endian byte order of Chapter 4 and while the manual insists the CPU is word-addressed. Add a pointer to Section 4.2, and a one-line table of the three directives (name, apparent width, real width, value placement).
- confidence: medium

### F-CR-3
- file: reference/chapters/02-cpu-architecture.tex
- lines: 40-60
- severity: minor
- category: structure
- claim: "Evaluate condition codes"
- evidence: 10-condition-codes.tex
- resolution: n/a
- fix: The six-phase list (Phase 2 "Evaluate condition codes", Phase 5 "post-increment/pre-decrement", "Advance pl") uses condition codes, auto-update addressing, and p/pl before any of them is introduced. Add one forward-reference sentence ("conditions: Chapter 10; addressing modes: Chapter 8; pl/ph: Section 3.4") at the top of Section 2.2.
- confidence: medium

### F-CR-4
- file: reference/chapters/02-cpu-architecture.tex
- lines: 105-125
- severity: minor
- category: structure
- claim: "The exception coprocessor never overwrites a pending command. It keeps its own record of the exception it wants to service"
- evidence: none
- resolution: n/a
- fix: The CSR paragraph in 2.3.2 is the densest passage in Part I (CSR, dispatch slot, trace fault, traced EXCP in four sentences) and I read it twice. It is also needed by Chapters 1, 12, 16 as the explanation of "12 cycles". Split into "what the CSR is" (keep here) and "when an interrupt can be taken relative to a coprocessor call" (move to 6.3.3 with a pointer). Section 2.3.2 should only say what a reader needs to follow Figure 2.1.
- confidence: medium

### F-CR-5
- file: reference/chapters/06-exceptions.tex
- lines: 25-60
- severity: minor
- category: structure
- claim: "Exceptions are prioritized as follows (highest to lowest):"
- evidence: 05-status-register.tex (E1-E4, "Hardware interrupt line n is dispatched at exception level n+1"); 06 vector list
- resolution: n/a
- fix: Five numbering schemes are in play (IRQ pin/line 1-5, "Level 1-5 hardware exception", priority 2-6, exception level 2-6, vector 0x50..0x10 running the opposite way, link register 1-5, enable bit E1-E4) and are explained in prose across Ch 1, 5 and 6. Line n -> vector 0x60-0x10n is never stated in one place. Add a single table at 6.1.1 (rows: IRQ pin, Level name, priority/exception level, enable bit, vector, link register) and refer to it from Ch 5 and the Hardware manual.
- confidence: high

### F-CR-6
- file: reference/chapters/06-exceptions.tex
- lines: 12-25
- severity: minor
- category: structure
- claim: "so a trap can never overwrite link register 0 (Section 6.4)"
- evidence: 06-exceptions.tex 6.5.1
- resolution: n/a
- fix: "Link register" is used from 6.1.1 and Chapter 2 but defined in 6.5.1 (banked registers, one per level, return address + SR + level). Add a two-sentence definition and forward pointer at first use in 6.1, and a pointer from Section 2.3.7.
- confidence: high

### F-CR-7
- file: reference/chapters/06-exceptions.tex
- lines: 160-190
- severity: minor
- category: structure
- claim: "Exception dispatch faults ... Raised during the exception handling machinery itself"
- evidence: none
- resolution: n/a
- fix: The retryable / post-instruction / exception-dispatch categorisation, plus the DMA and alignment carve-outs, is a three-paragraph wall of prose. Replace with a table: fault, vector, category, return address stored, RETE behaviour, handler must ETFR/ETTR? (yes/no). A reader writing a fault handler wants exactly that and currently has to assemble it from 6.2.2-6.2.10.
- confidence: high

### F-CR-8
- file: reference/chapters/06-exceptions.tex
- lines: 340-400
- severity: minor
- category: structure
- claim: "(Section 6.2.10)"
- evidence: 06-exceptions.tex: Section 6.2.10 (Double Fault) says "when a fault's own vector cannot be fetched (Section 6.2.10)" - a section pointing at itself; the vector-fetch case is the sub-heading "Fault Vector Fetch Failure"
- resolution: n/a
- fix: Point to the subsection by name. The two double-fault cases (raised inside a handler: unrecoverable; vector fetch failure: recoverable) have opposite recovery advice and are separated by 25 lines of RETE-stuck-at-level-7 reasoning. Put them in a two-row comparison table so the reader can find "can I RETE?" at once.
- confidence: medium

### F-CR-9
- file: reference/chapters/05-status-register.tex
- lines: 205-225
- severity: minor
- category: structure
- claim: "an encoding of the opcode with the status-update field (AF, Chapter 7) set to 10, not reachable from public assembly syntax, leaves all four flags architecturally undefined (U U U U)."
- evidence: 07-instruction-formats.tex, 11-reading-instructions.tex
- resolution: n/a
- fix: Section 5.5.1 uses AF, [N]/[A]/[S] "status source" and "shift operand" before they are defined (Chapters 7, 9, 11). The brackets are first seen in the Notation chapter as "[N], [A] or [S]" with a pointer to Chapter 11. Add one sentence at 5.1 or 5.5: "[N]/[A]/[S] select where flags come from; defined in Table 11.3". Also this bullet buries an unreachable edge case in the main rule; move undefined AF encodings to Chapter 7.
- confidence: medium

### F-CR-10
- file: reference/chapters/05-status-register.tex
- lines: 288-320
- severity: minor
- category: structure
- claim: "The SIRC-1 provides syntax to manually control how status flags are updated during ALU instructions ... See Section 9.12"
- evidence: 05-status-register.tex Examples 5-1, 5-2
- resolution: n/a
- fix: Section 5.8 promises worked examples of flag-update control and condition-driven branching, then gives (a) entering protected mode and (b) saving/restoring sr, and defers the promised content to Chapters 9, 10, 13. Either retitle ("Usage Examples: privilege and save/restore") and add a pointer sentence, or add one real example (CMPI then conditional ADDI[N]).
- confidence: medium

### F-CR-11
- file: reference/chapters/07-instruction-formats.tex
- lines: 140-200
- severity: minor
- category: structure
- claim: "BRAN #4 | 0x18 ... LDEA a, (#4, s) | 0x18"
- evidence: 07-instruction-formats.tex Table 7.2
- resolution: n/a
- fix: Table 7.2 shows BRAN and LDEA sharing opcode 0x18 (differing only in Reg/AF) and uses LDEL, BRSR, conditions (|==, |>=) not yet introduced. Say once, directly under the table, "BRAN is LDEA p, ...; see Section 12.4 / Chapter 15", otherwise a reader sees two instructions with one opcode and wonders if the table is wrong. Likewise the AF field means pair selector or status source depending on opcode, but the opcode-to-meaning map is only in Appendix A; point to it from 7.3.
- confidence: medium

### F-CR-12
- file: reference/chapters/07-instruction-formats.tex
- lines: 230-300
- severity: minor
- category: structure
- claim: "Table 7.5: Condition Code Encoding" / "Table 7.6: Shift Type Encoding"
- evidence: 10-condition-codes.tex Table 10.1; 09-shift-operations.tex Table 9.1
- resolution: n/a
- fix: The condition-code and shift-type encodings are each tabulated twice (7.5 vs 10.1, 7.6 vs 9.1), and the Assembly Language Summary cites the Chapter 7 copies while Chapter 10/9 are where the explanation lives. A reader cannot tell which is authoritative. Keep one table per concept in its own chapter and have Chapter 7 refer to it; update the Notation chapter's "(Table 7.5)", "(Table 7.6)" pointers.
- confidence: high

### F-CR-13
- file: reference/chapters/09-shift-operations.tex
- lines: 40-60
- severity: minor
- category: terminology
- claim: "In register-format instructions the shifted operand is R2 (first source)"
- evidence: 11-reading-instructions.tex Table 11.1 (rS1/rS2); 07 register format
- resolution: n/a
- fix: "R2" is the encoding field name, but the shifted operand is rS1 in notation and "first source" in prose. Three names for one thing. Add a field-name/notation mapping row (R1 = rD, R2 = rS1, R3 = rS2?) in Chapter 7 or Table 11.1 and use it consistently.
- confidence: medium

### F-CR-14
- file: reference/chapters/12-instruction-summary.tex
- lines: 150-170
- severity: minor
- category: structure
- claim: "COPI #0x11vv ... COPI #0x1Cxy ... COPI #0x28xx ... COPI #0x2Ann"
- evidence: 16-coprocessor-instructions.tex
- resolution: n/a
- fix: Table 12.2 uses placeholders vv, xy, xx, nn with no key. Add a one-line key under the table, or point to Section 6.10.2 (CSR layout: ID, op, operand) and Chapter 16. Also note this table is repeated in Chapter 17; say which is canonical.
- confidence: medium

### F-CR-15
- file: reference/chapters/12-instruction-summary.tex
- lines: 170-200
- severity: minor
- category: structure
- claim: "A coprocessor call whose condition is true (COPI, COPR and every meta-instruction that assembles to one) takes a second 6-cycle slot"
- evidence: 01-introduction.tex (1.3.2), 02-cpu-architecture.tex (2.4.2), appendix-b-timing.tex
- resolution: n/a
- fix: The 12-cycle rule and its three exceptions are restated at least four times (1.3.2, 2.4.2, 12.5, App B) with slightly different wording and different levels of detail, so the reader cannot tell which is complete. State it once in Appendix B (or 12.5), and shorten the others to a pointer.
- confidence: medium

### F-CR-16
- file: reference/chapters/13-alu-instructions.tex
- lines: 126-160
- severity: minor
- category: structure
- claim: "ALU instruction execution does not perform data-memory access and does not raise data bus, data bus-protection, data alignment, effective-address segment-overflow, or invalid-opcode faults."
- evidence: none
- resolution: n/a
- fix: This paragraph under "Common ALU Semantics" runs about 20 lines and mixes faults, sr write rules, [A]/[S] interaction, trace, and invalid opcodes; I read it three times. Turn it into a bulleted list or a table with rows for Faults possible, Privilege, Writing sr, Trace, and put the sr-write rule only in Chapter 5 with a pointer. The status-update override rules are also spread over 5.5, 9.12, 11.5 and 13.5; have one canonical place.
- confidence: medium

### F-CR-17
- file: reference/chapters/03-registers.tex
- lines: 125-145
- severity: minor
- category: structure
- claim: "Loading a value directly into the p pair performs a jump. This includes an ALU or LOAD instruction that names pl as its destination"
- evidence: 06-exceptions.tex 6.2.5
- resolution: n/a
- fix: A basic fact (writing p is a jump, pl advances by 2 in decode) is buried in a paragraph that quickly goes into segment-overflow exemptions (SR.A, "program-counter wrap check") of Section 6.2.5, which the reader has not yet seen. Put the plain rule in 3.4.5 and move the exemption to Chapter 6 with a pointer back. Also 3.4.1 states "p advances by 2 during decode" and 3.4.5 repeats it.
- confidence: medium

### F-CR-18
- file: reference/chapters/03-registers.tex
- lines: 20-45
- severity: minor
- category: structure
- claim: "The high word of each address register pair, marked * , is read-only in protected mode"
- evidence: none
- resolution: n/a
- fix: Section 3.2 describes the figure in terms of privilege (*, dagger, italic low byte) before the reader knows what protected mode is (Ch 1.3.4 and 5.4.1 are the only earlier mentions). Add a one-sentence definition of protected vs supervisor mode at the top of Chapter 3 with a pointer to Section 5.4.1. Also a table (register x mode x read/write) would replace the three footnote paragraphs and Figure 3.3 for most readers.
- confidence: medium

### F-CR-19
- file: reference/chapters/appendix-d-examples.tex
- lines: 10-40
- severity: minor
- category: structure
- claim: "Byte Sieve ... storing one word per candidate number in a memory-mapped array"
- evidence: appendix-d-examples.tex listing D.1
- resolution: n/a
- fix: The title says "Byte" but the text says one word per candidate; the listing keeps a "TODO: Log this somehow" comment and uses numeric labels (@1, @4) while the Summary says names use letters, digits and underscores (no mention that numeric labels are legal and that the text's "labels :2 and :3 (lines 35-50)" is the target of line numbers that the code listing must match). It also relies on an unexplained memory-mapped file segment at 0x00F0 which belongs to the simulator, not the architecture. Say so, and give a one-line "what to look for" for each example.
- confidence: medium

### F-CR-20
- file: reference/chapters/11-reading-instructions.tex
- lines: 1-60
- severity: minor
- category: structure
- claim: "This part of the manual describes each SIRCIS instruction in a consistent format. The goal is to make each instruction entry usable as an implementation contract"
- evidence: preface.tex
- resolution: n/a
- fix: The purpose of Chapter 11 is "how to read the entries", but the first paragraph talks about the manual's goal rather than saying that. Open with "This chapter explains the fields, notation and symbols used in every entry of Chapters 13-17." The Preface is right to say it should be read first; yet it sits at page 99, after nine chapters of examples that already use these symbols. Table 11.1 (rD, rS1, #imm16, SUS) could move to the front matter with the other notation.
- confidence: medium

## Improvements

### F-CR-21
- file: reference/chapters/12-instruction-summary.tex
- lines: 1-200
- severity: minor
- category: improvement
- claim: "Complete Instruction List"
- evidence: none
- resolution: n/a
- fix: Add a one-page "ISA at a glance" card: three format bit diagrams side by side (Figures 7.1-7.3), the register ID table (Table 3.1), the condition-code table, and a mnemonic-by-function list. Appendix E partly does this, but the bit diagrams and condition codes are not on one page. A programmer or emulator writer keeps this beside them.
- confidence: medium

### F-CR-22
- file: reference/chapters/06-exceptions.tex
- lines: 1-120
- severity: minor
- category: improvement
- claim: "Figure 6.1: SIRC-1 Exception Handling Overview"
- evidence: none
- resolution: n/a
- fix: Add a timeline diagram of one exception entry and return (instruction boundary, vector fetch, link register save, handler, RETE) with the phase numbers, and a second showing a nested interrupt (L2 then L4) with the link registers and exception level at each step. Figure 6.1 shows structure but not sequence; the entry/return sequence is only in prose (6.6.1, 6.7).
- confidence: high

### F-CR-23
- file: reference/chapters/appendix-d-examples.tex
- lines: 1-60
- severity: minor
- category: improvement
- claim: "Example Programs"
- evidence: none
- resolution: n/a
- fix: There is no short tutorial listing for a first-time reader. Add a 15-line "hello" program (reset vector, set up s, call a subroutine with BRSR/RETS, loop with CMPI and BRAN|!=) that is annotated line by line with the encoded hex of each instruction, before the sieve. Add a short software-interrupt example (EXCP, handler, RETE) with the register/link state shown after each step.
- confidence: medium

### F-CR-24
- file: reference/chapters/17-meta-instructions.tex
- lines: 1-40
- severity: minor
- category: improvement
- claim: "Every meta-instruction is an alias for exactly one real instruction encoding"
- evidence: 12-instruction-summary.tex Table 12.2
- resolution: n/a
- fix: Chapter 17 and Section 12.4 are two copies of the same table. Merge: keep the "Lowers To" and "Primary Documentation" columns in one place (Chapter 17), and in 12.4 give a pointer. Fewer places to keep in sync, and the reader has one place to look.
- confidence: medium

### F-CR-25
- file: reference/chapters/05-status-register.tex
- lines: 1-60
- severity: minor
- category: improvement
- claim: "Figure 5.1: Status Register Bit Layout"
- evidence: none
- resolution: n/a
- fix: Add a table of "which instructions write which flags" near the flags (arithmetic, logic, shift, compare, memory) with the C/V behaviour for subtraction as borrow-set (rather than prose in 5.3.3, 10.3.1 and 13.5), and a worked 16-bit example of C and V after ADDI and CMPI. Chapter 10 explains borrow in prose in three places.
- confidence: medium

### F-CR-26
- file: reference/chapters/04-data-representation.tex
- lines: 1-120
- severity: minor
- category: improvement
- claim: "Table 4.2: Address Register Pair Interpretation"
- evidence: none
- resolution: n/a
- fix: Add a memory-layout figure showing a 24-bit address, the segment (high word), and an instruction pair at p / p+1 with the byte image it would have in a ROM file (big-endian, 4 bytes), plus one vector table entry (vector 0x06 at word 0x0C). This is the question an emulator writer's first ROM loader asks.
- confidence: medium

### F-CR-27
- file: reference/chapters/01-introduction.tex
- lines: 1-120
- severity: minor
- category: improvement
- claim: "Document Organization"
- evidence: none
- resolution: n/a
- fix: Add a "reader paths" table to Section 1.5 and the Preface: emulator writer (3, 4, 5, 6, 7, App A), assembler author (7, 8, 11, App E, Assembly manual), kernel writer (3, 5, 6, 16), hardware designer (2, 6.8-6.9, Hardware manual). The document-organisation text now only lists the parts.
- confidence: medium

### F-CR-28
- file: reference/chapters/06-exceptions.tex
- lines: 280-330
- severity: minor
- category: improvement
- claim: "Fault Metadata Register Format"
- evidence: none
- resolution: n/a
- fix: Add a worked example of a fault handler that reads the fault metadata register after a bus fault on a STOR, with the register contents and the code, and how it differs for a double fault. Appendix D has the code; the explanation is not tied to the field layout.
- confidence: low

### F-CR-29
- file: reference/chapters/notation-and-glossary.tex
- lines: 1-60
- severity: minor
- category: improvement
- claim: "Glossary of Architectural Terms"
- evidence: none
- resolution: n/a
- fix: The glossary lacks entries for terms the reader meets early: link register (as the exception link banks), dispatch slot, CSR is present, but "exception link register", "retry", "write-back", "post-instruction fault", "slot" (program slot) and "L5" are not. Add them with section pointers, and make the glossary appear once with page-number links.
- confidence: low

### F-CR-30
- file: reference/chapters/12-instruction-summary.tex
- lines: 1-40
- severity: minor
- category: improvement
- claim: "Four cases differ"
- evidence: none
- resolution: n/a
- fix: Replace the timing prose with a timing table (instruction class, cycles, notes: ALU 6, LOAD/STOR 6 (+waits), taken/untaken branch 6, coprocessor call true 12/false 6/cop0 6, RSET 18, WAIT variable, DMA, maths). Appendix B has timing but the first-time reader first meets it in prose in Chapter 1 and has to rebuild the table.
- confidence: medium

## Summary

Top three:
1. F-CR-5 / F-CR-7: exception levels, priorities, lines, vectors and link registers use five numbering schemes with no single table, and the fault categories and recovery rules are prose walls.
2. F-CR-12 / F-CR-15 / F-CR-24: the same content (condition codes, shift types, meta-instructions, 12-cycle rule) is stated in two to four places, so the reader cannot tell which copy is canonical.
3. F-CR-1 / F-CR-9 / F-CR-18: the assembly summary, [N]/[A]/[S], and protected-mode notions are used in Chapters 2-5 before they are defined, and the Preface does not direct the reader to the back-matter Notation chapter.

Verdict: the manual reads as a nearly finished, thorough reference whose Part III is solid, but Parts I-II still read as a draft in their ordering, duplicated tables and dense prose paragraphs.
