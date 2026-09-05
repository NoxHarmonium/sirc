# Period style conventions: what the SIRC-1 manuscript should adopt

Corpus: `M68000PRM` (Motorola M68000 Family Programmer's Reference Manual, 1992; cites are
printed section-page numbers) and `MCS6500` (MOS Technology MCS6500 Family Programming
Manual, 1976; cites are printed pages or appendix pages). MC68000UM absent. SIRC-1 quotes
are from `preamble.tex`, `chapters/11-reading-instructions.tex`, the ADD entry in ch13,
and the BRAN/LDEA entries in ch15.

## 1. Fixed field order of an instruction entry

| Position | M68000PRM (`3-33` template; `4-4`, `4-25`, `4-108`, `4-116`) | MCS6500 App B (`B-3`) | SIRC-1 `instructionbox` (ADD, ch13) | SIRC-1 ch11 says |
|---|---|---|---|---|
| Title | `ADD  Add  ADD` (mnemonic both margins, full name centre), then `(M68000 Family)` | `ADC  Add memory to accumulator with carry  ADC` | `ADDI / ADDR -- Add Immediate / Add Register` as box title | Mnemonic |
| 1 | Operation: | Operation: | Opcodes: | Opcodes |
| 2 | Assembler Syntax: | (Ref: 2.2.1) | Syntax: | Syntax |
| 3 | Attributes: (Size = ...) | flag row `N Z C I D V` | Operands: | Operands |
| 4 | Description: | table: Addressing Mode, Assembly Language Form, OP CODE, No. Bytes, No. Cycles | Operation: | Operation |
| 5 | Condition Codes: | footnote `* Add 1 if page boundary is crossed.` | Description: | Write-back |
| 6 | Instruction Format: | | Flags: | Status Flags |
| 7 | Instruction Fields: | | Write-back: | Condition Codes |
| 8 | NOTE (optional) | | Exceptions: | Exceptions |
| 9 | | | Condition codes: | Timing |
| 10 | | | Timing: | Privilege |
| 11 | | | Privilege: | |
| 12 | | | Example: / Notes: | |

Adopt: the PRM order for the shared fields, so the entry reads Operation, Assembler
Syntax, Description, Condition Codes (flag effects), Instruction Format, Instruction
Fields, then the SIRC-specific Exceptions, Timing, Privilege, Notes. Put Opcodes into the
Instruction Format diagram rather than a hex list. Make ch11 and the entries agree: today
ch11 lists Write-back before Status Flags and the ADD entry has Flags before Write-back;
ch11 calls it "Status Flags", the entry says "Flags:". Every entry must carry every
field, even when the content is "Not affected." (`4-25`, `4-108`).

## 2. Register-transfer notation on the Operation line

| Convention | Corpus | SIRC-1 form |
|---|---|---|
| Arrow for assignment, destination on right | `Source + Destination → Destination` (`4-4`); `Destination Address → PC` (`4-108`); `A + M + C → A` (`MCS6500:p.7`, `B-3`); `M → A` (`p.4`) | `destination = result` in a `verbatim` block; `p = p + displacement` |
| Conditional as prose RTN | `If Condition True Then PC + dn → PC` (`4-25`); Table 3-1 defines `If <condition> then <operations> else <operations>` (`3-2`) | `if condition:` / indented Python-style block |
| Complex sequence expanded once in notation table | TRAP expanded as `SSP - 2 → SSP; PC → (SSP); ...` (`3-2`) | Per-entry pseudocode, e.g. `dest.high = src.high` / `dest.low = src.low + displacement` |
| Flags not restated in Operation | Operation line is data flow only; flags live in Condition Codes | ADD Operation block includes `SR.N = result[15]` ... `SR.V = signed_overflow` |
| Subscripts for sizes | `Source10 + Destination10` for BCD (`4-2`); `dn` displacement width (`4-25`) | `#imm16`, `#imm8` (fine) |

Adopt: one-line RTN with `→` and `;` separators, `If cond Then ... Else ...` for
predication, flags removed from the Operation block. Keep the longer pseudocode, if
needed, as a second block titled "Detailed operation".

## 3. Condition-code effect table

Corpus form (`4-4`, `4-116`, `4-2`):

```
Condition Codes:
   X N Z V C
   * * * * *            (or  — * * 0 0 ;  * U * U * )
   X — Set the same as the carry bit.
   N — Set if the result is negative; cleared otherwise.
   Z — Set if the result is zero; cleared otherwise.
   V — Set if an overflow is generated; cleared otherwise.
   C — Set if a carry is generated; cleared otherwise.
```

Symbol legend (`3-4`, `3-18`): `*` affected, `—` not affected, `0`/`1` cleared/set,
`U` undefined, `?` see special definition. 6500 uses a `/` mark under each affected flag
(`B-3`). "Not affected." replaces the whole block when nothing changes (`4-25`).

SIRC-1 form: `\textbf{Flags:} N Z C V` (ADD) and `\textbf{Flags:} Preserved.` (LDEA);
legend in ch11 uses `*`, `0`, `1`, `-`, `S`. Adopt the corpus block: header row in the
manual's flag order (N Z C V), symbol row, per-flag sentence with the semicolon-"cleared
otherwise" idiom; keep `S` for shifter-sourced flags but define it in the manual-wide
legend. Add `U` for undefined results if any exist.

## 4. Bit-field diagrams

| Convention | Corpus | SIRC-1 |
|---|---|---|
| Every bit numbered along the top, MSB left | `15 14 13 ... 1 0` on every format (`2-2`, `4-4`, `8-5`) | `bytefield` with `\bitheader{0,15,16,31}` (only edge bits) in ch3/ch6; ch7 formats |
| Field names in caps inside the box; fixed bits written as literal 0/1 | `1101 REGISTER OPMODE EFFECTIVE ADDRESS / MODE REGISTER` (`4-4`); `0110 CONDITION 8-BIT DISPLACEMENT` (`4-25`) | Similar in ch7, absent from entries |
| Following words stacked beneath, with a rule per word | `16-BIT DISPLACEMENT IF 8-BIT DISPLACEMENT = $00` (`4-25`); `16-BIT WORD DATA` under ORI (`8-5`) | n/a |
| Register-width diagrams show `31 ... 15 ... 0` markers and MSB/LSB labels | Figure 1-18 (`1-26`), Figure 1-8 SR with bit legend below (`1-11`) | ch5 SR bytefield |
| Memory maps show address at left, word per row, `+$02` offsets | Stack frames (`B-3`); Figure 1-20 (`1-27`) | ch6 link register diagram |

Adopt: full bit headers (every bit) on instruction-word diagrams, two stacked 16-bit
words for immediate format, literal opcode bits inside the box, and a bit legend under
the SR figure as in Figure 1-8.

## 5. Normative vocabulary and reserved bits

| Term | Corpus usage | SIRC-1 |
|---|---|---|
| Reserved bits | "undefined bits reserved for future definition by Motorola. Those particular bits read as zeros and must be written as zeros for future compatibility." (`1-27`) | "reserved for future use. They should not be relied upon by software." (ch5 §Reserved Flags); "11 = Reserved for future use" (ch7) |
| Undefined | "All other bits ... are undefined and must not be used." (`1-11`); `U` in flag tables; "N — Undefined." (`4-2`) | "undefined" 16 uses, "architecturally undefined" (ch6) |
| Unassigned encodings | "(Unassigned, Reserved)" rows (`8-4`, `B-2`); "Future Expansion" (`MCS6500:D-1`) | "Undocumented" rows (ch12), app C |
| must / must not | "must be set to one" (`8-1`); "must be a control addressing mode" (`4-108`) | must 32, must not 5, should 22, shall 0 |
| Notes | Set-off `NOTE` block, centred heading, no number (`1-1`, `4-6`, `4-26`) | `\item` lists under "Notes:" |

Adopt: define "reserved" once (read value, write requirement, future use) and use the
PRM's "read as zero, must be written as zero" pattern where that is the hardware truth;
reserve "should" for advice, "must" for requirements; rename the ch12 "Undocumented" rows
to "(Unassigned, Reserved)" unless the app C behaviour is being promised; use a
`NOTE` block environment for the asides currently in "Notes:" bullets.

## 6. Numbering

| Item | Corpus | SIRC-1 |
|---|---|---|
| Figures and tables | `Figure 1-1`, `Table 3-18`, numbered per section; front-matter lists (`ix`, `xi`) | `Figure 2.1`, `Table 1.1`, per chapter; LOF/LOT emitted. Equivalent |
| Pages | Section-page `4-4`, `B-2`; running head "MOTOROLA / M68000 FAMILY PROGRAMMER'S REFERENCE MANUAL / 4-4" | Continuous page numbers with fancyhdr chapter/section marks |
| Headings | `3.1.1`, `8.1.7.1` decimal, caps titles | `\chapter`/`\section`/`\subsection` decimal. Equivalent |
| Examples | `Example 4.3`, numbered per chapter and listed in front matter (`MCS6500:vii`, `p.36`) | Unnumbered `lstlisting` blocks |
| Instruction pages | Mnemonic in running head both margins (`4-4`) | Chapter name in head |

Adopt: numbered examples with a List of Examples (6500), and the mnemonic in the running
head on instruction pages (PRM). Section-page numbering is optional; keep continuous.

## 7. Voice

Corpus: third person, present tense, subject elided in descriptions: "Adds the source
operand to the destination operand ... and stores the result" (`4-4`); "Program
execution continues at the effective address" (`4-108`); imperative only in notes and
procedures ("refer to Table 3-19", `4-25`). The 6500 is tutorial ("we add the value",
`p.38`; "one loads", `p.8`) and addresses "the user": do not copy.

SIRC-1 already matches the PRM: "Adds two values and stores the result in the destination
register." No "we"; nine uses of "you" should go. Avoid evaluative asides in entries
("This is useful for pointer arithmetic", LDEA): move usage remarks to Notes.

## 8. Marking examples as non-normative

PRM keeps examples out of entries (3.3 Instruction Examples, `3-20`) and labels advice
"Most assemblers automatically make this distinction" inside a NOTE (`4-6`). 6500 numbers
every example (`Example 2.1`, `p.7`) with a stated purpose. SIRC-1 has one
"non-normative" statement (ch4) and unlabelled "Example:" blocks in every entry. Adopt: a
one-line rule in the conventions section ("Examples and Notes are informative; the
Operation, Condition Codes and Instruction Format fields are normative"), numbered
examples, and no requirements introduced inside an example comment.

## 9. Hex and binary literals

| Corpus | SIRC-1 |
|---|---|
| Hex with `$` prefix, upper case: `$FF`, `$00000000`, `+$02` (`4-25`, `1-27`, `B-3`); 6500 bare `0100`, `FFFA` (`p.36`, `p.124`) | `0x` prefix, upper-case digits, 735 uses; `\opcode{}` macro |
| Binary bare in encoding tables: `000`, `111`, `0110` (`4-5`, `8-4`); grouped `0000 1101` in worked examples (`MCS6500:p.7`) | Bare in tables (`00`, `11`); `0b` twice |
| Vector offsets as three hex digits without prefix in tables (`B-2`) | `0x60-0xFF` |

Adopt: keep `0x` (the C convention was already common in 1992 and `$` is a Motorola
assembler artefact), but state it once in the conventions section, remove the two `0b`
literals in favour of the bare-binary-in-tables rule, and never mix prefixed and
unprefixed forms in one table.

## 10. Notation and conventions section as a template

`M68000PRM:3-2`..`3-4` (Table 3-1) is the model. Group headings: Single- and Double-Operand
Operations (`+`, `-`, `~`, `Λ`, `V`, `⊕`, `→`, `←→`, `<op>`, `sign-extended`); Other
Operations (`If <condition> then ... else`); Register Specifications (`An`, `Dn`, `Rn`,
`Xn`); Data Format and Type (`B, W, L`, `<fmt>`); Subfields and Qualifiers (`#<data>`,
`( )`, `[ ]`, `dn`, `LSB`, `MSB`); Register Names; Register Codes (`*`, `U`, `—`); Stack
Pointers; Miscellaneous (`<ea>`, `<label>`, `m-n` bits). The 6500's `B-2` key is the
compact version. SIRC-1 equivalent: ch11 Notation (8 rows) plus Status Flag Effect
Symbols; extend to the groups above and place it in the front matter or ch1.

## 11. Smaller conventions worth copying

- Per-instruction legal addressing-mode table with mode/register encodings and `—` for
  disallowed forms (`4-5`, `4-108`, `4-117`); mode categories named ("control", "data
  alterable", "memory alterable").
- Cross-references by number, not page: "refer to Table 3-19" (`4-25`), "Refer to Section
  1 Introduction" (`3-17`). SIRC-1 uses `\ref` (102) and no `\pageref`: correct.
- Entry title carries the full English name: "Move Data from Source to Destination"
  (`4-116`); SIRC-1 "ADDI / ADDR -- Add Immediate / Add Register" is close.
- Scope NOTE at the front stating which device names imply which variants (`1-1`).

## 12. Period conventions not to copy, with reasons

| Convention | Where | Why not |
|---|---|---|
| Monochrome only, colour absent | Both manuals | Print constraint, not a virtue; keep colour for syntax and links, but ensure meaning survives in greyscale. Fix `sircblue` = RGB(255,0,0), a red named blue |
| Page-number cross-references and section-page numbering | `4-4`, `B-2` | Obsolete with hyperlinks; `\ref` plus hyperref is strictly better |
| `$` hex prefix | `4-25` | Motorola assembler idiom; `0x` is the toolchain's syntax |
| Tutorial second person, "we"/"one"/"the user", motivational prose | `MCS6500:p.1`, `p.8`, `p.38` | 1976 introductory register; the PRM's terse third person is the 1992 standard |
| Rotated, typewriter-set summary tables | `MCS6500:A-2`, `C-1` | Unreadable in text extraction and on screen; set tables upright |
| One physical page per instruction with hard page breaks | PRM Section 4 | Wastes space for a 6-cycle 40-opcode ISA; use a box per entry but allow flow |
| Family-wide processor applicability tables (Appendix A) | `A-12` | Only one implementation exists; a revision line per entry is enough |
| Deferring bus/timing/signals to a separate user's manual | `B-1` | SIRC-1 has no second volume; keeping ch2 in this manual is right |
| Unnumbered NOTE blocks that carry requirements | `4-6` | Keep NOTE blocks, but requirements must appear in normative fields, not notes |
