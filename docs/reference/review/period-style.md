# SIRC-1 Period Style Conventions

Conventions the corpus manuals share, set against the SIRC-1 form. Key and page-cite scheme are as in `period-gaps.md`.
`STYLE.md` already adopted the 2026-09-05 findings from the PRM and 6500. This file adds the hardware and assembly
manuals and the Z80, ARM, UM and MASM manuals. Section 9 is the section `STYLE.md` cites for the `$`-prefix rejection.

## 1. Instruction entry field order

| Manual | Field order | Cite |
|---|---|---|
| PRM | Operation, Assembler Syntax, Attributes, Description, Condition Codes, Instruction Format, Instruction Fields | Fig 3-3 (3-33); ADD 4-4, Bcc 4-25, MOVE 4-116 |
| Z80 | Operation, Op Code and Operands with encoding, Description, M Cycles / T States, Condition Bits Affected, Example | 145, 265 |
| ARM | Encoding figure, field notes, Syntax, Examples | 2-29, 2-30 |
| 6500 | Operation line with flag row, then a table of addressing form, opcode, bytes, cycles | B-3 |
| SIRC-1 | Applicability, Opcodes, diagrams, Instruction Fields, Syntax, Operands, Operation, Description, Flags, Write-back, Exceptions, Condition field, Timing, Privilege, Example, Notes | `11-reading-instructions.tex`; ADDI at `13-alu-instructions.tex:353` |

Three of the four lead with Operation. SIRC-1 puts the opcodes, diagrams and Instruction Fields before Syntax and
Operation. Adopting the PRM order (Operation and Syntax first, encoding and Instruction Fields last) is the largest
single change to reader feel. It is an author decision because `STYLE.md` section 4 fixed the current order.

The PRM also names each field once. SIRC-1 gives one field three names: "rD (Reg or R1, bits 25--22)" in ADDI, with
`rS1 (R2 ...)` and `imm16 (Immediate Value ...)` following the same pattern. Use the diagram label alone and keep the
aliases in one table in Chapter 7.

## 2. Register transfer notation in Operation

| Form | Example | Cite |
|---|---|---|
| PRM | `Source + Destination -> Destination`; `If Condition True / Then PC + dn -> PC` | 4-4, 4-25; symbols 3-2 |
| 6500 | `A + M + C -> A, C` | B-3 |
| Z80 | `A <- A + r` | 145 |
| SIRC-1 | `result = operand1 + operand2` / `destination = result` / `SR.N = result[15]` | ADDI Operation block |

SIRC-1's Operation is statement-level pseudocode, which is more precise than the period one-liner. Keep it, and add the
period one-line arrow form as the first line of each Operation block (`rS1 + rS2 -> rD`). Define the arrow in Table 11.1,
following PRM Table 3-1. BRAN's block opens with a `;` comment line, so apply one form throughout.

## 3. Condition code effect table

| Manual | Form | Cite |
|---|---|---|
| PRM | A drawn row `X N Z V C` with one symbol under each, then one sentence per flag, for example "N -- Set if the result is negative; cleared otherwise" | 4-4 |
| PRM | A single table of formulas for every instruction, with `?` meaning "see special definition" | Table 3-18 (3-18) |
| 6500 | Row `N Z C I D V` with `/` for changed and `_` for unchanged | B-3 |
| Z80 | Prose per flag: "S is set if result is negative; otherwise, it is reset." | 145 |
| SIRC-1 | `\textbf{Flags:} N Z C V`, legend in Table 11.2 | ADDI |

SIRC-1 names only the flags that change, so an unchanged flag is invisible and no sentence defines any flag. Adopt the
PRM pair: a drawn row in SR bit order using the existing symbols (`* 0 1 - S U`), then one sentence per flag. Table 13.4
already holds the formulas, which matches PRM Table 3-18.

## 4. Bit-field diagrams

| Point | Period practice | Cite | SIRC-1 |
|---|---|---|---|
| Bit order | MSB on the left, bit numbers 15 to 0 printed above every bit | PRM 4-4 | Matches (`endianness=big`, `\bitheader{0-31}`) |
| Fixed bits | Literal bits drawn in the box (`1 1 0 1`) | PRM 4-4 | Matches (`00`, `p`, `0x00`) |
| Numbering | Entry diagrams unnumbered; chapter-level diagrams numbered | PRM Fig 2-2 (2-2) | Matches (Figs 7.1 to 7.3) |
| Field list | One line per field, same name as the box | PRM 4-5 | Aliased names, see section 1 |

No change needed to the drawing style.

## 5. Normative vocabulary and reserved bits

| Practice | Cite | SIRC-1 position |
|---|---|---|
| Reserved bits "read as zeros and must be written as zeros for future compatibility" | PRM 1-27 | Rejected by `STYLE.md` (advisory only). Keep |
| "U -- Undefined, Reserved for Motorola Use" fuses two terms | PRM 3-4 | Keep the two terms separate |
| "(Undefined, Reserved)" in a code table | UM 3-9 | Use "Reserved" alone in code tables |
| "should" for hard requirements ("should never be simultaneously asserted") | UM 5-10 | Do not copy; keep "must" |
| "may produce unpredictable results" | UM 3-7 | Do not copy; `STYLE.md` bans "unpredictable" |

## 6. Numbering of figures and tables

| Manual | Scheme | Cite |
|---|---|---|
| PRM, UM | By section: `Figure 1-1`, `Table 3-18`, `Table B-1` | PRM LoF/LoT; UM LoT |
| Z80 | One sequence for the whole manual: Figures 1 to 41, Tables 1 to 23 | Z80 xii to xiv |
| 6500 | Examples numbered by chapter, `2.1`, `8.3`, and listed in front matter | 6500 vii to ix |
| SIRC-1 | By chapter with a dot: `7.1`, `A.1` | All three manuals |

SIRC-1 matches the majority. The front-matter "Table 0.1" and "Table 0.2" have no period counterpart; harmless.

## 7. Voice

| Manual | Practice | Cite |
|---|---|---|
| PRM | Third person, present tense, subject elided: "Adds the source operand to the destination operand ..." | 4-4 |
| UM | Third person, present: "The processor places valid function codes on FC2-FC0" | 5-8 |
| MASM | Third person, "the assembler", "the user"; imperative only for "Refer to" | 1-5, 3-2 |
| SIRC-1 architecture, hardware | Matches | Chapters 6, 13 to 17; hardware Ch.3 to 5 |
| SIRC-1 assembly | Second person: "This manual assumes you have read the Assembly Language Summary" (preface); a column headed "You wrote" (Table 7.1) | `assembly/chapters/preface.tex`, `07-diagnostics.tex` |

`STYLE.md` bans "you" but was written for the architecture manual. Extend the rule to the other two: use "Source" for
the column and "assumes the reader has read" in the preface.

## 8. Marking examples non-normative

| Practice | Cite |
|---|---|
| Worked programs live in a separate chapter or section, not in the entries | PRM 3.3.1 (3-20); 6500 Ch.11 |
| Every example is numbered by chapter and listed in front matter | 6500 vii to ix |
| The example shows the assembled output beside the source | ARM 2-30 ("Assembles to EAFFFFFE"); Z80 265 (object-code table) |
| SIRC-1 | `Example N-M` numbering and a List of Examples are already in place; Notes are marked informative |

SIRC-1 is ahead on marking. The missing piece is the encoded word beside each example line, which `qa.sh` could generate.

## 9. Hex and binary literals

| Manual | Hex | Binary | Cite |
|---|---|---|---|
| PRM | `$FF`, `Format $0` | Bare digit groups in field tables: `01 -- Byte operation` | 4-26, B-3; 4-116 |
| MASM | `$1000`; `%` binary, `@` octal | | 2-31 |
| ARM | Trailing `H`: `FFFFFFFFH` | | 2-30 |
| SIRC-1 | `0x2A`, uppercase digits; bare binary groups in tables; assembler `#0x2A`, `#0b0010_1010` | | `STYLE.md` section 3 |

Keep `0x`; do not copy `$` or a trailing `H`. One consequence for the assembly manual: a reader trained on the 68000
manuals reads `$ANSWER` as hex. Section 2.6 and Table 2.1 should say once that `$` introduces a named constant and never
a number.

## 10. The notation and conventions section

| Manual | Form | Cite |
|---|---|---|
| PRM | One table, grouped under sub-heads (operations, register specifications, data formats, qualifiers, register names, register codes, stack pointers, miscellaneous) | Table 3-1 (3-2 to 3-4) |
| Z80 | Operand placeholders with their value ranges: "n ... (0 to 255)", "e ... (-126 to +129)" | Table 4 (39) |
| MASM | Meta-symbols for syntax: `< >`, `|`, `[ ]`, `[ ] ...` | 1-4 |
| ARM | Stated inline at the first syntax: "Items in { } are optional. Items in < > must be present." | 2-29 |
| SIRC-1 | Flat 14-row Table 11.1; no value ranges; typographic table in the Notation chapter; no syntax meta-symbols | Ch.11; Table 0.2 |

Adopt a value-range column in Table 11.1 (`#imm8`, `#disp`, shift count 0 to 15), and group its rows. The assembly manual
needs the meta-symbol rule most. Its syntax `MNEMONIC[S]|cond operand` uses `[` and `|` as literal characters, so a reader
cannot tell which are meta. The ARM convention fits: `{ }` for optional, `< >` for required, and `[ ]` and `|` always
literal.

## 11. Hardware manual conventions

| Convention | Period practice | Cite | SIRC-1 |
|---|---|---|---|
| Signal entry | Prose, bold name with polarity overbar: "Bus Error (BERR)" | UM 3-7 | Bulleted Direction and Signal Type, then prose |
| Signal entry | Inline attributes: "A15-A0. Address Bus (output, active High, tristate)." | Z80 5 | As above |
| Signal entry | Table of name, pin, description, polarity last | ARM 2-5 | Pin table (Table 2.1) separate from descriptions |
| Three-state | Stated for every bus signal | UM 3-10, Z80 5 | Never stated: D0-D15 are "Bi-directional, Active High" |
| Active-low names | Overbar | UM 3-7 | Plain names, polarity in prose only |
| Bus-cycle narrative | "STATE 0 The read cycle starts in S0 ..." under each figure | UM 5-8 | Phase row inside diagrams only |
| AC table | Min/Typ/Max per speed grade, units, conditions, footnotes | ARM 2-21, UM 10-10 | Not yet written |
| Timing figures | Numbered parameters, notes under the figure | UM 10-13 | Opening paragraph only |

Adopt: the overbar on active-low names in headings and diagrams (cheap, strongly period); a three-state attribute; a state
narrative under each cycle figure; the AC table form when Chapter 6 is written; and a signal summary table.

## 12. Assembly manual conventions

| Convention | Period practice | Cite | SIRC-1 |
|---|---|---|---|
| Directive entry | Fixed fields "FORMAT:" then "DESCRIPTION:", NOTE blocks | MASM 3-2 to 3-4 | Prose per directive |
| Directive index | One table by function, `*` for "label required" | MASM Table 3-1 | Data directives only (Table 5.1) |
| Line structure | Label, operation, operand, comment fields defined | MASM 2-2 | "Whitespace is otherwise insignificant" |
| Error catalogue | `ERROR xxx` / `WARNING xxx`, grouped, exact text | MASM E-1 to E-3 | Paraphrases in Table 7.1 |
| Name collisions | Rule that a label must not match a mnemonic | 6500 168 | Not stated for `r1`, `ADDI` |
| Reading guide | "Chapters 1 through 4 ... basic; 5 through 8 ... advanced" | MASM 1-1 | None |

## 13. Conventions of the era not to copy

| Convention | Cite | Reason |
|---|---|---|
| Colour-free pages | all | A print constraint, not a virtue. The red and grey accents are fine in a PDF |
| Cross-references by page ("see page 262", "paragraph 3.5.2.10") | Z80 TOC; MASM 1-3 | Hyperlinks replace them; keep `Chapter~\ref` |
| Section-page numbering (`4-25`) | PRM, UM | Existed for loose-leaf replacement pages; continuous pagination is better on screen |
| Minus prefix for active-low (`-IRQ`) | ARM ix | The preface says it was a desktop-publishing limit; use an overbar |
| `$`, `%`, `@`, trailing `H` literals; `*` comment column | MASM 2-1, 2-31; ARM 2-30 | Assembler-specific; SIRC-1 uses `0x`, `;` |
| Half-clock state names (S0 to S19) | UM 5-8 | Describe a 68000 internal; copy the narrative method, not the names |
| Legal disclaimers, sales offices, marketing essays | Z80 ii; ARM 9-3, 1-3 | Not reference content |
| Per-entry processor lists ("MC68020, MC68030 only") | PRM 4-1 | SIRC-1 already has a one-line Applicability |
