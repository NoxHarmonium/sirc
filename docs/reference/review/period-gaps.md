# SIRC-1 Period Coverage Gaps

Benchmark of the three SIRC-1 manuals against the six-manual corpus in `docs/reference/review/corpus/`.
Page cites are printed page numbers (`manual:page`); the Z80 and ARM cites were checked against the PDF page offset.

| Key | Manual | Notes |
|---|---|---|
| PRM | M68000 Family Programmer's Reference Manual, 1992 | No bus or electrical content (out of scope for it) |
| UM | MC68000 User's Manual, 9th ed. (c) 1993 | Bus, signals, timing, electrical |
| 6500 | MCS6500 Family Programming Manual, 1976 | Tutorial style; the oldest manual here |
| Z80 | Zilog Z80 CPU User Manual UM008011, 2016 reissue | Reissue of an early-1980s text, so not strictly a 1992 manual. It has no electrical chapter |
| ARM | VLSI Acorn RISC Machine Data Manual, 1990 | ARM2 (VL86C010) is Section 2 |
| MASM | M68000 Resident Structured Assembler Manual, 1984 | The only assembler manual in the corpus; 68k-specific |

Cell legend: a page cite means present, `-` means absent, `~` means partial. In the last column, A is the architecture
manual, H the hardware manual and S the assembly manual.

## Coverage matrix

| Section or feature | Belongs in | PRM | UM | 6500 | Z80 | ARM | MASM | SIRC-1 |
|---|---|---|---|---|---|---|---|---|
| Preface, audience, reading guide | all three | ~1-1 | ~1-1 | 1 | iii, 1 | ix | 1-1 | A present (Preface); H present; S present, no beginner/advanced reading guide (MASM 1-1 splits Ch.1-4 from 5-8) |
| Notation and conventions section | all three | 3-2 to 3-4 | - | B-1 | 39 | 2-29, ix | 1-4 | A present (Notation chapter, Table 11.1); H absent (only 3.1 Signal Conventions); S partial (Ch.1 Conventions is two sentences, no syntax metasymbols) |
| Related documents | all three | - | - | - | - | - | 1-5 | A, H, S present as Table 1.1 / Table 0.1 |
| Programming model diagrams | A | 1-2, 1-4, 1-8 | 2-2, 2-3 | F-1 | 2 | 2-25 | - | A present (Ch.3, Figs 3.1, 3.2) |
| Data organisation, registers and memory | A | 1-25 to 1-30 (Figs 1-18 to 1-21) | 2-5 to 2-7 | - | - | 2-25 | - | A partial (Ch.4, Tables 4.1 to 4.3; no layout figure) |
| Addressing modes, EA calculation per mode | A | 2-4 to 2-18 (2-6) | 2-4 (list) | Ch.5-6, E-1 | 33-38 | 2-37 | 2-4 to 2-21 | A present (Ch.8.7 `addrmodebox`) |
| Per-instruction template | A | 3-33, 4-4 | - | B-3 | 145 | 2-29 | 3-2 (directives) | A present (Ch.11, 13-17); fields missing, see backlog |
| Instruction format summary, opcode map | A | 2-2, 8-1 to 8-4 | - | D-1 | 40 to 65 | 2-49 | A-1 | A present (Ch.7, App. A) |
| Condition code computation | A | 3-18, 3-19 | - | 24 to 30 | 65 | 2-29 | - | A present, distributed (Ch.10; Tables 13.3, 13.4, 14.3, 15.3) |
| Exceptions: vector table | A | B-2 | 6-7 | 124 | 17 | 2-26 | - | A present (6.1) |
| Exceptions: priorities | A | - | 6-8, 6-9 | 142 | 6 | 2-26 | - | A present (6.1) |
| Exceptions: stack frames or saved state | A | B-3 to B-13 | 6-9, 6-10 | 129 to 133 | - | 2-26 | - | A present (Fig 6.2, link registers) |
| Reset | A + H | B-2 (vectors) | 5-29, 6-11 | 125 to 129 | 5 (pin) | 2-5 | - | A present (6.9, Table 6.3); H present (RSTI/RSTO, Fig 5.9) |
| Privilege model | A | 1-8 | 6-1 to 6-3 | - | - | 2-5 (M1/M0) | - | A present (3.6, Fig 3.3); H present (PROT) |
| Bus operation | H | - | 4-1 to 5-38 | - | 8 to 17 | 2-6 to 2-11 | - | H partial (Ch.4: rule tables, no state-by-state narrative, no flowcharts) |
| Bus arbitration, interrupt-acknowledge protocol | H | - | 5-9 to 5-23 | ~129 | 11, 12 | ~2-5 | - | H absent (no statement that there is no external bus master or IACK cycle) |
| Signal descriptions | H | - | 3-1 to 3-10 | - | 5 to 8 | 2-5, 2-6 | - | H present (Ch.3); no signal summary table (UM 3-10) |
| Timing diagrams | H | - | 5-3 to 5-37, 10-13 | - | 8 to 17 | 2-22, 2-23 | - | H present (Ch.5, ten figures, cycle-level only) |
| Electrical and AC characteristics | H | - | 10-1 to 10-29 | - | - | 2-21 to 2-24 | - | H absent (Ch.6 is a 112-word stub) |
| Package, mechanical, ordering | H | - | 11-1 to 11-16 | - | 317 (support) | 8-3, 9-3 | - | H partial (Ch.2 pin table only) |
| Instruction timing tables | A | - | 7-1 to 9-13 | B-1, C-1 | per entry, 145 | 2-13 to 2-20 | - | A present (App. B, Table B.1, per-entry Timing) |
| Coprocessor interface | A + H | 8.1.1 | - | - | - | 2-45 to 2-48 | - | A present (Ch.16); H partial (no statement that coprocessors are on-chip) |
| Minimum-system and design example | H | - | ~B-2 (M6800 circuit) | - | 21 to 25 | - | - | H absent (TODO on decoupling in Ch.3) |
| Assembler syntax and directives | S | per entry | - | 168 | 65 | per entry | 2-1 to 3-14 | S partial (Ch.2-5; no grammar, no directive table, no reserved-word rule) |
| Expressions and operators | S | - | - | - | - | - | 2-28 to 2-33 | S absent by design (Ch.8) |
| Assembler diagnostics | S | - | - | - | - | - | E-1 to E-3, 4-4 | S partial (Ch.7: six examples, two linker errors) |
| Listing, object and binary formats | S | C-1 (S-record) | - | - | - | 7-5 | 4-5, C-1 | S absent (object file stated to be opaque; binary and `.dbg` undocumented) |
| Macros, conditional assembly | S | - | - | - | - | 7-5 | 5-1 to 6-10 | S absent by design (Ch.8) |
| Index | all three | - | - | - | - | - | - | Absent from the six corpus PDFs and from SIRC-1; not a gap against this corpus |

## Ranked backlog

Ranked by how much a 1992 reader would miss the section. "Section" means add a section; "Field" means add a field to the
instruction template.

| # | Gap | Manual | Kind | What the addition contains |
|---|---|---|---|---|
| 1 | Electrical and AC characteristics | H | Section | Absolute maximum ratings, DC levels, clock limits, and setup, hold and propagation times per speed grade, as a Min/Typ/Max table (UM 10-10, ARM 2-21). |
| 2 | Parameter-annotated timing diagrams | H | Section | Each Ch.5 figure gains numbered or symbolic parameters keyed to the AC table (UM 10-13, ARM 2-22), with notes under each figure. |
| 3 | Directive reference and formal syntax | S | Section | A directive table by function, one entry per directive with Format and Description (MASM 3-1 to 3-4), the line grammar, reserved words, and the character set. |
| 4 | Complete diagnostic catalogue | S | Section | Every assembler and linker message, verbatim, grouped by cause, with errors distinguished from warnings (MASM E-1). |
| 5 | Package, mechanical and ordering data | H | Section | Package outline and dimensions, thermal data, and an ordering and part-marking table (UM Section 11, ARM Section 8). |
| 6 | Per-flag definition lines | Template | Field | One sentence per flag after the flag row, for example "N: set if the result is negative; cleared otherwise" (PRM 4-4). |
| 7 | Legal-forms table with opcode and cycles per entry | Template | Field | Rows of syntax form, opcode, words and cycles (6500 B-3); already tracked as handover Workstream 14. |
| 8 | Encoded form beside each example | Template | Field | The assembled word next to each example line, as in "Assembles to EAFFFFFE" (ARM 2-30) and Z80 265. |
| 9 | Signal summary table | H | Section | One table of signal, pin, direction, active level and three-state behaviour (UM 3-10). |
| 10 | Bus-mastership and interrupt-acknowledge statement | H | Section | States that the CPU always owns the bus and that no IACK cycle exists, plus a BACK/BERR/BPER/HALT result table (UM 5-31). |
| 11 | Bus-cycle narrative | H | Section | A numbered state-by-state description under each cycle figure (UM 5-8). |
| 12 | Data organisation figure | A | Section | Word and byte layout in registers and in memory (PRM 1-26 to 1-29). |
| 13 | Running mnemonic on continuation pages | Template | Field | Mnemonic repeated in the page header of multi-page entries (PRM 4-4, 4-5). |
| 14 | Listing, object and map formats | S | Section | A sample listing, and the layout of the `.o`, `.bin` and `.dbg` files (MASM 4-5, C-1). |
| 15 | Minimum-system example | H | Section | Memory, wait-state and reset circuit with decoupling guidance (Z80 21 to 25). |

## Claims that conflict across the set

| Item | Where | Conflict |
|---|---|---|
| Clock limit | `hardware/chapters/03` CLKI vs `06` | Ch.3 gives "Max Rate: 24 MHz"; Ch.6 says the clock range is not yet specified. |
| Assembler scope | `hardware/chapters/01` and `assembly/chapters/01`, Table 1.1 | Both say the assembly manual covers "macros, pseudoinstructions"; Ch.8 says there are none. |
| Diagnostic wording | `assembly/chapters/07`, Table 7.1 vs the sample above it | The table paraphrases the message; the sample shows the longer text actually emitted. |
