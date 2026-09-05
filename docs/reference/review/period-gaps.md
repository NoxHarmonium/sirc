# Period coverage gaps: SIRC-1 reference manual vs 1992-era CPU manuals

Corpus read: `M68000PM_AD_Rev_1_Programmers_Reference_Manual_1992.pdf` (cited as
`M68000PRM:<printed page>`; section-page numbering, e.g. `4-4` is PDF page 108) and
`mcs6500_family_programming_manual.pdf` (MOS Technology, 1976; cited as `MCS6500:p.<n>` or
appendix page, e.g. `B-3`). **Absent from corpus:** MC68000 User's Manual (bus operation,
signal descriptions, electrical/AC, instruction timing). The PRM itself defers those topics
to it (`M68000PRM:B-1`, "refer to the appropriate user's manual"), so the bus/timing/signal
rows below are graded against what the corpus can show, with the UM noted as the missing
comparator. Pages read: M68000PRM 56, MCS6500 50.

Manuscript grades: **present** / **partial** / **absent**, with chapter (`ch`) or appendix (`app`).

## Coverage matrix

| Section or feature | M68000PRM (1992) | MCS6500 (1976) | SIRC-1 manuscript |
|---|---|---|---|
| Preface / "about this manual", audience | Absent as such; Section 1 opens with device list and a scoping NOTE (`1-1`) | Present: 1.0 Manual Introduction, two reading levels, audience, companion Hardware Manual (`p.1-2`) | **Partial**: ch1 "Document Organization" and "Scope" only; no audience statement |
| Related documents | Refers to device user's manuals (`1-8`, `B-1`) | Hardware Manual named (`p.1`) | **Partial**: "SIRC-1 ABI and Toolchain documentation" named once in ch1 Scope; no list |
| Manual-wide notation and conventions | Present: Table 3-1 Notational Conventions, 3 pages (`3-2`..`3-4`); size suffix rules (`3-32`) | Present: notation key at head of Appendix B (`B-2`) | **Partial**: ch11 notation table covers instruction-entry symbols only (`rD`, `#imm16`, `SR.X`); no manual-wide key |
| Table of contents, list of figures, list of tables | Present (`iii`..`xii`) | TOC + List of Examples (`iii`..`viii`) | **Present**: `main.tex` emits TOC, LOT, LOF |
| Programming model diagram (user) | Present: Figure 1-1 register stack with bit widths (`1-2`) | Appendix F programming model (`F-1`, image) | **Partial**: ch3 has "Address Register Pair Formation" figure and register encoding table; no single all-registers figure |
| Supervisor / privileged model | Present: 1.3, Table 1-1 supervisor registers, Figure 1-8 SR with system/user byte (`1-8`..`1-11`); Section 6 privileged instructions | Not applicable (no privilege) | **Partial**: privilege rules spread over ch1 §Privilege Levels, ch3 §Privilege Restrictions, ch5 control flags, ch6; no supervisor-model figure or register/bit table |
| Data organisation in registers and memory | Present: 1.7, Figures 1-18..1-21 (`1-25`..`1-29`) | Ch1 byte/word discussion (`p.2`); page concept (`p.51`) | **Present**: ch4 word/byte ordering, address values, instruction and vector storage |
| Addressing-mode chapter with EA calculation per mode | Present: 2.2, each mode has GENERATION / ASSEMBLER SYNTAX / EA MODE FIELD / EA REGISTER FIELD / NUMBER OF EXTENSION WORDS box plus diagram (`2-6`, `2-7`); summary Table 2-4 | Ch5-6 tutorial; Appendix E cycle-by-cycle bus status per mode (`E-1`, `E-2`) | **Present** (ch8: Description / Syntax / Effective Address / Usage per mode). Per-mode block lacks encoding fields and word count; see gap 11 |
| Per-instruction description template, fixed fields | Present: Figure 3-3 template (`3-33`); Operation, Assembler Syntax, Attributes, Description, Condition Codes, Instruction Format, Instruction Fields (`4-4`, `4-25`, `4-108`, `4-116`) | Prose entries (`p.4`, `p.7`, `p.36`); tabular summary per instruction in Appendix B (`B-3`) | **Present** (`instructionbox`, ch11 field list) but ch11 list and actual entries disagree in order and names; see gaps 1-3, 9, 13 |
| Instruction Format bit diagram in each entry | Present on every entry (`4-4`, `4-25`, `4-108`, `4-116`) | OP CODE column per addressing mode (`B-3`) | **Absent**: zero `bytefield` diagrams in ch13-16; entries give "Opcodes: 0x00 (Imm), 0x20 (Short Imm), 0x30 (Reg)" only |
| Instruction Fields (field-by-field encoding) in each entry | Present (`4-5`, `4-26`, `4-116`) | n/a | **Absent** from entries; field semantics only in ch7 |
| Instruction format summary / opcode map | Present: Section 8 field definitions, Table 8-2 opcode map by bits 15-12 (`8-4`), then every instruction's binary word in opcode order (`8-5`..) | Appendix D hexadecimal opcode listing (`D-1`, `D-2`); Appendix C modes vs times (`C-1`) | **Partial**: ch7 formats (3 bytefields), ch12 opcode list, app A opcode map; no per-instruction full-word binary layouts in opcode order |
| Condition-code computation appendix | Present: Table 3-18 boolean formulas for V, C, Z per instruction (`3-18`); Table 3-19 conditional tests with formulas (`3-19`) | 3.8 Flag Summary, per-flag "instructions which affect" lists (`p.29`, `p.30`) | **Partial**: ch10 condition encodings and truth tables; ch13 flag-effect table (`*`/`0`/`-`); no V/C formulas |
| Exception vector table | Present: Table B-1 vector number, hex offset, assignment (`B-2`) | 9.0 vectors at FFFA-FFFF (`p.124`) | **Present** as an itemised list (ch6 §Vector Table) plus "Exception Quick Reference" table; not a numbered table |
| Exception priorities | Deferred to user's manual (`B-1`); SR interrupt mask shown (`1-11`) | 9.5, 9.10 NMI (TOC `p.129`, `p.142`) | **Present**: ch6 §Exception Priority |
| Stack frames or saved state | Present: Figures B-1..B-23 per frame format (`B-3`..`B-13`) | RTI/BRK stack discussion (TOC `p.132`, `p.144`) | **Present**: ch6 §Link Registers with bytefield of saved address/SR/level |
| Reset | Vectors 0 and 1 only (`B-2`) | Present: 9.1-9.4, reset line discipline and start sequence (`p.125`..`p.127`) | **Present**: ch6 §Reset and Startup (state, output hold, vector fetch) |
| Privilege violations list | Section 6 privileged instructions; SR S-bit (`1-11`) | n/a | **Present**: ch6 enumerates five privileged operations |
| Bus operation and timing | Absent (in MC68000UM, not in corpus) | 5.1 pipelining, "every clock cycle is a memory cycle", Figure 5.2 (`p.52`); Appendix E bus status per cycle | **Present**: ch2 §Timing Notes, tikz-timing Figures 2.3-2.10, "CPU bus output timing rules" table |
| Signal descriptions | Absent (UM) | Reset line behaviour only (`p.125`) | **Present**: ch2 §Chip Layout, §Pin Descriptions, 64-pin DIP figure and table |
| Electrical / AC characteristics | Absent (UM) | Absent (Hardware Manual) | **Absent**; out of scope for an ISA manual, as it is for the PRM. State this in the preface |
| Instruction timing tables | Absent (UM) | Present: Appendix B cycles and bytes per addressing mode, "* Add 1 if page boundary is crossed" (`B-3`); Appendix C | **Partial**: app B "Instruction Timing" by instruction type; per-entry "Timing:" line. Adequate for a fixed 6-cycle machine |
| Coprocessor interface | 8.1.1 Coprocessor ID field (`8-1`); Section 5 FP instructions; B.3 FP stack frames | n/a (peripheral chips ch11) | **Present**: ch16, ch1 §Coprocessors, compatibility policy |
| Instruction examples | Separate 3.3 Instruction Examples (`3-20`); NOTE blocks in entries (`4-6`) | Numbered examples throughout, listed in front matter (`vii`) | **Present**: inline "Example:" in each entry, usage sections, app D programs |
| Ordering of instruction entries | Alphabetical by mnemonic within section (`4-1`); Appendix A per-processor mnemonic lists (`A-12`) | Grouped by concept; Appendix A/B alphabetical (`A-1`, `B-1`) | **Partial**: grouped by family (ALU, memory, control, coprocessor); app E mnemonic index gives alphabetical lookup |
| Subject index | Absent (TOC only) | Absent from TOC, though `p.2` refers to one | **Absent**; `main.tex` comment "Index would go here" |
| Undocumented / reserved opcode policy | "(Unassigned, Reserved)" rows (`8-4`, `B-2`); Line 1010/1111 emulator vectors | "Future Expansion" rows in opcode map (`D-1`) | **Present**: app C, app A §Undocumented Instructions |

## Ranked gap list (backlog for the author)

Ranked by how much a 1992 reader would miss the item. Tag: **[TEMPLATE FIELD]** = add a
field to `instructionbox` (or the addressing-mode block); **[SECTION]** = new content.

1. **[TEMPLATE FIELD] Instruction Format diagram in every entry.** A bit-numbered
   16-bit word (both words for immediate format) showing opcode, condition, register,
   AF, shift and immediate fields, as `M68000PRM:4-4` and `4-25` do on every page; the
   6500 gives at least the OP CODE per form (`B-3`). Today a reader must reconstruct the
   encoding from ch7 plus a hex opcode.
2. **[TEMPLATE FIELD] Instruction Fields list in every entry.** One line per field
   naming its legal values for this instruction ("Opmode field: 000/001/010 ...",
   `M68000PRM:4-5`; "Size field: 01 byte, 11 word, 10 long", `4-116`), including which
   AF and shift encodings are legal here.
3. **[TEMPLATE FIELD] Condition Codes block.** Replace `Flags: N Z C V` with the period
   form: an `N Z C V` header, a symbol row (`*`, `-`, `0`, `1`, `U`), then one sentence per
   flag, "Set if ...; cleared otherwise." (`M68000PRM:4-4`, `4-116`, `4-2`). Also rename:
   the manuscript's "Condition codes:" line describes predicated execution, which a 1992
   reader will read as the CCR-effect line.
4. **[SECTION] Condition-code computation table.** A ch10 or appendix table giving the
   boolean formulas for V, C, Z per instruction family in terms of Sm, Dm, Rm
   (`M68000PRM:3-18`), plus the condition-test formulas already implicit in ch10
   (`3-19`). Needed for SUB/SBC borrow polarity and shifter-sourced `[S]` flags.
5. **[SECTION] Preface / About This Manual.** Audience, what the reader is assumed to
   know, companion documents (ABI, toolchain, simulator), what is out of scope
   (electrical/AC), how to read the instruction entries, revision status
   (`MCS6500:p.1-2`; `M68000PRM:1-1` NOTE).
6. **[SECTION] Manual-wide notational conventions table.** Operators, register names,
   `#imm`, `( )` indirection, `SR.X`, `->`/`=` assignment, literal prefixes, symbol legend
   for flag tables, in one place near the front (`M68000PRM:3-2`..`3-4`; `MCS6500:B-2`).
   Ch11's table is the seed; generalise and move forward.
7. **[SECTION] Instruction format summary in opcode order.** An appendix listing every
   opcode 0x00-0x3F with its full 32-bit word layout drawn as a bit diagram, following a
   field-definition preamble and the opcode map (`M68000PRM:8-1`..`8-5`; `MCS6500:D-1`).
   App A gives the map but not the layouts.
8. **[SECTION] Programming model figures.** One figure of all user-visible registers
   with bit widths (`M68000PRM:1-2`) and one for the supervisor additions (SR system
   byte, link registers, cause register) with a table of which registers/bits are
   privileged (`1-9`, `1-11`). Ch3 has the parts but no picture.
9. **[TEMPLATE FIELD] Legal forms table in each entry.** Per entry, a table of accepted
   operand/addressing forms with their encodings and dashes for disallowed ones
   (`M68000PRM:4-5`, `4-108`), or a cross-reference line to the chapter "Legal Forms"
   table. A 1992 reader expects the entry to be self-contained.
10. **[SECTION] Exception vector table as a table, with saved state per class.** Convert
    the ch6 itemised list to columns: vector number, table address, assignment,
    priority, retryable/post-instruction (`M68000PRM:B-2`), and give one saved-state
    figure per exception class as B-3..B-13 do per frame format.
11. **[SECTION] Per-mode encoding box in ch8.** Add to each addressing-mode block the
    fixed lines GENERATION (EA formula), ASSEMBLER SYNTAX, format/field encoding and
    number of instruction words (`M68000PRM:2-6`, `2-7`), so each mode is a template, not
    prose. (This is "add fields to the addressing-mode template", not the instruction
    template.)
12. **[SECTION] Alphabetical access and index.** Either order entries alphabetically
    within Part III (`M68000PRM:4-1`) or make app E a true mnemonic-to-page index with
    every alias (BRAN, RETS, LJMP...) and add a short subject index; neither corpus
    manual has a subject index, so this ranks low.
13. **[TEMPLATE FIELD] Applicability line under the title.** "(M68000 Family)" or
    "(MC68020, MC68030, MC68040)" under each entry title (`M68000PRM:4-1`, `4-4`); for
    SIRC-1, the coprocessor model/revision or "SIRC-1 all revisions", tying into ch16's
    revision policy.

Not gaps: electrical/AC characteristics (out of scope for both the PRM and this manual;
say so in the preface); cycle-by-cycle bus tables per addressing mode (`MCS6500:E-2`)
are superseded by the ch2 timing diagrams; an Attributes/size line (`M68000PRM:4-4`) is
not applicable to a fixed 16-bit-word ISA.
