# SIRC-1 Manual Set — House Style Sheet

For copy editors working across the three SIRC-1 manuals: Architecture Reference (`docs/reference`,
22 chapters and appendices), Hardware Reference (`docs/hardware`) and Assembly Programmer's Manual
(`docs/assembly`). Rules were set by majority usage found in the current chapters, except where
the handover ("Resolved:" items), Chapter 11, an approved decision, or the period benchmark
(`review/period-style.md`) already dictate a form. Apply these rules; do not introduce new
house-style debates during copyediting. Every inconsistency found in the first derivation was
ruled on by the author (**Approved decisions**). Later additions for the hardware and assembly
manuals stand until the author rules on **Decisions needing human approval**.

## Scope

- Every rule applies to all three manuals unless tagged **[ARCH]**, **[HW]** or **[ASM]**.
- The manuals share `preamble.tex`; its macros mean the same everywhere.
- ARCH owns what the CPU does. HW owns what is driven or sampled on each pin. ASM owns what
  the tools accept and print, and does not restate instruction semantics. State a fact only in
  its owning manual; the others refer to it.
- Cross-manual references are plain text by chapter name, never number: "the Exception and
  Interrupt Handling chapter of the `\archmanual{}`". Always use `\archmanual{}` or `\hwmanual{}`
  for the book title (16 vs 7), not "the architecture manual". Inside one manual, use `\ref` (§5).

## 1. Terminology

| Preferred | Avoid | Notes |
|---|---|---|
| protected mode | user mode | Resolved in handover Workstream 1; "user mode" is now unused (0 hits). |
| supervisor mode / supervisor | kernel mode / kernel | Resolved; "kernel" unused (0 hits). |
| fault | abort exception | "Fault" dominant (207 vs 4); "abort exception" survives only as a stray label, already flagged fixed in handover. |
| hardware exception | — | Formal architectural term. "Interrupt" remains acceptable for pin-level/informal use (interrupt pins, interrupt priority, interrupt enable, NMI) — Ch.6 explicitly defines the two as synonyms at different registers of formality. Do not merge into one term. |
| meta-instruction | meta instruction, alias, convenience instruction | Resolved in handover; hyphenated form dominant (73 vs 4). |
| word address | byte address | SIRC-1 is word-addressed; "byte address" only appears where genuinely discussing external byte order (Ch.4) — not a competing form. |
| high word / high byte / low word / low byte | upper word / upper byte | 43 "high" vs 12 "upper"; standardize on "high"/"low". |
| coprocessor | co-processor | 155 vs 7; drop the hyphen everywhere, including headings. |
| general-purpose register | general purpose register | Compound adjective before a noun takes a hyphen; usage is split (11/8) but this is a grammar fix, not a terminology choice. |
| architecturally undefined | undefined (bare), implementation-defined (when describing undocumented-opcode behavior) | Handover resolution: undocumented opcodes are valid encodings with *architecturally undefined* behavior, not vendor-style "implementation-defined" behavior. `appendix-a-opcode-map.tex:140` still says "implementation-defined" for undocumented opcodes — flag for correction to match the resolved definition; true implementation-defined language stays reserved for genuine per-model timing variance (e.g., extra maths-coprocessor cycles). |
| reserved | — | See §2. Do not use "undocumented" and "reserved" interchangeably: "reserved" = architecturally set aside; "undocumented" (App. C) = a specific, currently-unassigned opcode with observed behavior worth describing as a caution, not a promise. |
| displacement | offset (for addressing-mode/branch values) | Keep "displacement" for addressing-mode and branch/call values; keep "offset" for memory/structure offsets, e.g. `program_offset`. Do not swap the two. |
| R1/R2/R3 (upper case, no macro) | — | Reserved for abstract *field-position* labels in instruction-format diagrams and field-order prose (Ch.7). Never use for a concrete register operand. |
| `\reg{r7}` | bare "R7" for a concrete register | `06-exceptions.tex:648–649` uses bare "R7" for the real register `r7`; correct to `\reg{r7}` to match the 89-instance convention. |
| behavior, signaling, labeled, canceled, recognized, initialized, gray | behaviour, signalling, labelled, cancelled, recognised, initialised, grey | American spelling set-wide; see §8. |
| pin | line, leg | The physical contact. Use "signal" for the function it carries. |
| asserted / deasserted | activated, de-asserted, pulled low | Logical state. "High"/"low" give only voltage. "Active" appears only inside active-low/active-high. |
| active-low, active-high (adjective) | active low (adjective), negative-true | The `Signal Type:` value stays `Active Low`. |
| three-state | tri-state, tristate | Not yet stated for any pin; see approvals. |
| bidirectional | bi-directional | `Direction:` values: `Input`, `Output`, `Bidirectional`. |
| wait state (noun), wait-state (modifier) | waitstate | 13 vs 3 is the noun/modifier split. |
| bus cycle | bus transaction | 6 vs 3. "Bus Access Strobe/Type" are signal names and stay. |
| diagnostic (printed text); error (the condition) | error message | [ASM] 7 vs 1. |
| symbol (any label or named constant), label, named constant | identifier, variable, equate | [ASM]. |
| meta-instruction | pseudoinstruction | A pseudoinstruction expands to several instructions; the assembler has none (ASM Ch. 8). Not a synonym. |
| assembler, linker, simulator | VM, emulator | `\texttt{sirc\_vm}` only for the executable or a command line. |
| hexadecimal, runtime, setup (noun), set up (verb) | hex (prose), run-time, run time | runtime 3, run-time 2, run time 1. |

## 2. Normative vocabulary

- **must / must not** — a mandatory architectural requirement; non-conformant hardware or software is broken.
- **may** — a permitted option; the CPU, the assembler, or the programmer is free to choose.
- **should / should not** — a recommendation. Not required for conformance. Do not use for anything that is actually mandatory.
- **shall** — do not use. The manual currently has zero instances; keep it that way and use "must" instead.
- **reserved** — a field, opcode, or coprocessor ID not currently assigned. Software must not assume any particular read value and must not rely on any behavior it may have. Do *not* require writing zero (see "Reserved fields stay advisory" below).
- **undefined / architecturally undefined** — the architecture does not specify the result. It may vary by implementation, by revision, or between executions; software must not depend on any particular outcome, and doing so is not a portability guarantee. Prefer "architecturally undefined" in instruction-reference prose; bare "undefined" is acceptable in flag tables and informal descriptions.
- **implementation-defined** — behavior that is fixed and consistent within one CPU implementation/model but may differ between models (e.g., extra maths-coprocessor cycles). Not a synonym for "architecturally undefined."
- **unpredictable** — not used in this manual. Do not introduce it as a synonym for "undefined"; the manual already distinguishes "architecturally undefined" from "implementation-defined" and a third term would blur that line.
- **can** — capability, not permission; permission is "may". Avoid "will", "could", "might" in normative statements; use the present tense. "i.e." is not used; write "that is".
- **[HW]** "must" binds the external system; "may" is an option for it; "should" is advice. The CPU's own behavior is stated in the present tense without a modal. A value not yet fixed is "not yet specified", never "undefined", and gets no invented number.
- **[ASM]** "must" is a rule the tools enforce with a diagnostic; "may" is an accepted option; "should" is advice the tools do not check. A missing feature is "not supported", never "must not". "Undefined" is the architecture's term: write "architecturally undefined" and refer to the `\archmanual{}`.

## 3. Notation

- **Registers**: use `\reg{rN}` (lowercase) for concrete general-purpose registers (`r1`–`r7`), and `\reg{a}`, `\reg{l}`, `\reg{s}`, `\reg{p}`, `\reg{sr}` for the named registers, including their `ah`/`al` (high/low) forms. Never write a bare `rN`/`R1` in running prose; `\texttt{}` alone is acceptable only inside a `lstlisting` block or a syntax cell where the whole cell is already a code example.
- **Opcodes**: use `\opcode{XX}` (two hex digits, no `0x`, macro adds it) only for a literal instruction-opcode byte value (e.g. `\opcode{04}`). This macro is nearly unused today (3 hits) versus 700+ bare `0x` literals; the manual-wide pass was approved at Gate 1 (see Approved decisions).
- **Immediates**: use `\imm{N}` for a concrete immediate value written as an operand (renders `#N`). Use bare `\texttt{\#imm16}` / `\texttt{\#imm8}` only for the *placeholder* names defined in Chapter 11's notation table, never for literal values.
- **Addresses**: use `\addr{...}` for a concrete memory address or vector target.
- **Other hex/binary constants** (status-word values, encoded field values, bit patterns, coprocessor command words) that don't fit `\opcode`/`\imm`/`\addr`: wrap in `\texttt{0x...}`. Never leave a hex literal in the surrounding body font.
- **Hex literals**: `0x` prefix, uppercase hex digits (e.g. `0x1A`, not `0x1a`). Never use the `$` prefix (period artifact of Motorola assemblers, explicitly rejected in `period-style.md` §9).
- **Binary literals**: bare digit groups in tables (`00`, `10`, `111`), no `0b` prefix. The two existing `0b10` instances in `07-instruction-formats.tex` are the outliers; convert to the bare form used everywhere else.
- **Bit numbers**: write "bit 15", "bits 4–7" as plain text (dominant existing form; 70+ instances). MSB is the highest-numbered bit and is drawn on the left in every `bytefield` diagram — do not reverse this. Use `\bitheader{0-N}` syntax as-is (its hyphen is required package syntax, not a style choice).
- **Numeric ranges in prose and tables**: use an en dash `--` (`0x00--0x0F`, `bits 5--4`), never a plain hyphen. A hyphen is reserved for compound words and for LaTeX macro arguments/asm comments/source excerpts that are not proofread prose (e.g. `\bitheader{0-31}`, inline `; bits 9-12` comments).
- **Instruction mnemonics**: always uppercase inside `\mnemonic{}` (universal; 0 exceptions found). Never write a mnemonic as plain `\texttt{}` outside of an assembled syntax example.
- **Word size**: state "16-bit word" once per chapter/major section on first use, then use plain "word" afterward. Do not repeat "16-bit" on every mention.
- **Status flags**: use the Chapter 11 symbol set exactly: `*` (updated), `0`, `1`, `-` (preserved), `S` (from shifter), `U` (undefined). Do not invent new symbols without updating Chapter 11 first.
- **Status register fields**: `\texttt{SR.X}`, as in Chapter 11 (58 uses). `\statusbit` is unused; do not introduce it.
- **Bit numbering**: bit 0 is least significant everywhere, including signal groups (`A0`). Not `b15`, not `bit~15`.
- **Signal and pin names [HW]**: plain upper case in prose, headings, tables and diagrams (117 plain vs 14 `\texttt`). `\texttt` is for logic-level letters (`L`, `H`) and cycle labels (`IF`, `DR`, `DW`, `EV`, `EA`, `RV`). Pin numbers: "pin 64".
- **Active-low names [ALL]**: write an active-low signal with an overbar (all three manuals, including the hardware pinout figure; author decision 2026-11-02), using `\activelow{BERR}`, in prose, tables and timing-diagram labels (period convention, UM 3-7). Do not use a minus, `/`, `#` or `n` prefix. Section headings that name a signal keep the plain name. Polarity also stays in the `Signal Type:` bullet; prose says asserted/deasserted. A merged waveform row may read `\activelow{BERR}/\activelow{BPER}`; prose says "\activelow{BERR} or \activelow{BPER}".
- **Signal ranges [HW]**: repeat the prefix, en dash: `A0--A23`, `BAT0--BAT2`, `IRQ1--IRQ4`, `VCC1--VCC2`. Not `VCC1--2`.
- **Units [HW]**: `5~V`, `24~MHz`, `5~V $\pm$ 5\%`. Not `5V`, not `+/-`.
- **Source text [ASM]**: a full instruction line, directive or operand is `\texttt{...}`; a lone mnemonic is `\mnemonic{}` (79 vs 18); a concrete register is `\reg{r1}` (correct the 6 `\texttt{r1}`). Directives are upper case with a leading dot; registers lower case; mnemonics, conditions and shift types upper case. Source immediates keep the hash: `\texttt{\#0x2A}`.
- **Number forms [ASM]**: hex `0x2A`, binary `0b0010_1010`, decimal bare. Say once, in the Numbers section, that `$` introduces a named constant and never a number (period 68000 readers assume hex; period-style §9).
- **Syntax meta-symbols [ASM]**: `<NAME>` is a placeholder (upper case), `{ }` marks an optional part, `...` a repeatable part. `[ ]`, `|`, `#`, `@`, `:` and `( )` are always literal. Example: `<MNEMONIC>{[<SUS>]}{|<COND>} <OPERAND>, ...`. State the legend once in Chapter 1, Conventions. [ARCH] uses Table 11.1's named operands and no meta-symbols.

## 4. Instruction entry template

Chapter 11 and the actual entries in Chapters 13–17 disagree on field order and one field
name; the entries (dozens of instances across five chapters) are the de facto standard and
win over Chapter 11 (a single description). Canonical order and names, to be used for
every entry and matched by Chapter 11:

`Opcodes` (or `Assembles to` for meta-instructions) → `Syntax` → `Operands` → `Operation` →
`Description` → `Flags` → `Write-back` → `Exceptions` → `Condition codes` → `Timing` →
`Privilege` → `Example`/`Examples` → `Notes`.

Field labels are bold, sentence case, followed by a colon (`\textbf{Condition codes:}`),
matching the unanimous form already in Chapters 13–17. Do not use "Status Flags" or
"Condition Codes" (title case) as field labels — use "Flags" and "Condition codes."

## 5. Prose

- **Voice/tense**: third person, present tense, subject often elided ("Adds two values and
  stores the result..."). No first person ("we"), no second person ("you"). This already
  matches the manual; treat any surviving "you" as a defect to flag.
- **Sentence length**: prefer one architectural fact per sentence in Operation/Flags/
  Exceptions fields; narrative Overview sections may run longer.
- **Headings**: Title Case for `\chapter`, `\section`, `\subsection`, and table/figure
  column headers (unanimous in the sampled chapters).
- **Lists**: no terminal period on short phrase items; full sentences in a list get a
  period. Use the Oxford comma (18 vs 3 in the corpus; also period-appropriate).
- **"for example", never "e.g."**: ruled at Gate 1; see Approved decisions.
- **must/may/should**: use per §2; do not use "should" where a requirement is meant.
- **Cross-references**: `Chapter~\ref{...}`, `Section~\ref{...}`, `Table~\ref{...}`,
  `Figure~\ref{...}`, `Appendix~\ref{...}` — capitalized word, non-breaking space, then
  `\ref`. This is already 100% consistent; do not switch to lowercase or `\pageref`.
- **Examples vs normative text**: Operation, Flags, Exceptions, and the Instruction Format
  are normative. "Example:"/"Examples:" and "Notes:" blocks are informative and must not
  introduce new requirements — move any requirement found inside an example or note into a
  normative field instead.
- **Second person**: "you", "your" and "we" are banned in all three manuals, back covers included ("Inside you will find" becomes "This manual contains"). Write "assumes the reader has read" in the ASM preface and "Source" for the "You wrote" column. [ASM] An imperative is allowed in tutorial steps ("Save this program as ...") and workaround hints ("Use \mnemonic{LJMP}"), nowhere else.
- **Headings**: Title Case in all three manuals. Lower case for articles, conjunctions and prepositions of four letters or fewer, except first and last word. Signal names and mnemonics keep their case. Serial comma applies in headings ("Lines, Whitespace, and Comments").
- **[HW] signal sections**: heading `Name Direction (SIGNAL)` ("Bus Error Input (BERR)"); then bullets `\textbf{Direction:}`, `\textbf{Signal Type:}`, any other bold-label bullet (`Max Rate:`); then prose.
- **Introducing examples**: one full sentence saying what the example shows, ending in a colon when the listing or table follows directly. [ARCH] entries use the numbered `Example N-M:` block.
- **References**: `Chapter~\ref`, `Section~\ref`, `Table~\ref`, `Figure~\ref`. Never cite pages.

## 6. Tables and figures

- Caption **below** the table body (`\end{tabular}` then `\caption{}`), unanimous (54/54
  sampled). Figures: caption after the content, following the same pattern.
- Use `booktabs` (`\toprule`/`\midrule`/`\bottomrule`) for every table; no vertical rules,
  no hand-drawn `\hline`.
- Numeric/hex columns: left-align unless the table is purely numeric comparison data, in
  which case center or right-align consistently within that table (follow the existing
  table's own alignment; do not mix within one table).
- Every table that is cross-referenced from prose needs a `\label{tab:...}`; tables that
  exist purely for local, non-referenced illustration do not require one.
- Bit-field diagrams: full bit header on every bit (not just edges) for instruction-word
  diagrams that appear in an instruction entry; edge-only headers remain fine for
  wide-register diagrams that are purely illustrative.
- **Captions are Title Case, no final period**, all manuals ([HW] and [ASM] are sentence case today; see approvals). Floats use `[H]`, `\centering`, caption below.
- **Timing diagrams [HW]**: `tikztimingtable` with `sirc timing diagram`. Rows: `CLKI`, `Phase`, then signals in the existing figures' order. Waveform letters are physical levels, so `L` is asserted for an active-low signal. Value bands use only the abbreviations defined in Chapter 5's opening paragraph; define new ones there. Every figure has a `fig:` label.

## 7. Code listings

- `\lstlisting` language is always the preamble's default `SIRC` — never set a per-listing
  `language=` override.
- Comments use `;`, matching the `SIRC` language definition; never `//` or `#`.
- Listings inside instruction entries are illustrative, not normative — they show one
  legal encoding of the syntax already defined in `Syntax`/`Operands`; they must not be the
  sole source of a legality or flag claim.
- Architecture/reset/bus/exception/memory-model chapters should prefer pseudocode,
  diagrams, and tables over assembly listings (per handover Workstream 12); confine
  assembly listings mainly to the instruction-reference chapters and the examples appendix.
- Label declarations use a **leading** colon (`:loop`), never a trailing one (`loop:`) — the
  assembler only accepts the leading form, confirmed 2026-09-16 (`examples/crc-report`) after
  several existing manual listings turned out to use the trailing form and fail to assemble.
  Label *references* (branch/call targets, and any operand naming a symbol's address) use
  `@label`, with no `#` prefix even where the operand position is otherwise an immediate value
  (`BRAN|cond @loop`, `MOVI al, @buffer` — not `MOVI al, #buffer`). When adding or editing a
  listing that uses labels, prefer actually assembling it (`cargo run --bin assembler`, from
  `docs/reference/` add `../../sirc-vm` to the path) over eyeballing the syntax.
- **Marker**: a bare `lstlisting` of real assembly takes `% qa:assemble` on the line immediately before it (blank lines allowed). Listings inside an `example` float are extracted automatically. Placeholder-syntax and pseudocode blocks are not marked. [ASM] marks all 11 real listings.
- **Shell and diagnostics [ASM]**: `[language={}]`, never marked, no prompt character. Show tool output verbatim from a real run. A column headed "What the assembler says" holds the exact message; a paraphrase goes under "Meaning".
- **Comments**: `;`, start lower case, no final period. [HW] has no listings; use a figure or table.

## 8. Spelling

One dialect for the set: **American**. ARCH is already American (48 `behavior`, 0 `behaviour` in the .tex files); HW and ASM use about 8 British forms. Convert: behaviour to behavior, signalling to signaling, recognised to recognized, initialised to initialized, optimise to optimize, labelled to labeled, cancelled to canceled, grey to gray, colour to color, centre to center. Macro and color names (`sircgray`, `\color`) are code, not prose.

## 9. Hardware and assembly notes

- [HW] Describe pins and signals only; for what the CPU does with them, refer to the `\archmanual{}`. Never invent an electrical value; unspecified values appear only in Chapter 6.
- [ASM] Document only what the tools do. State a source rule as the form, then the rule, then the diagnostic. A missing feature goes in Chapter 8.

## Approved decisions (Gate 1, 2026-09-05)

The author ruled on every inconsistency flagged during the style derivation. These are
now house rules; editors apply them without further consultation.

### Terminology / vocabulary
- **"for example", never "e.g."** Spell it out in prose and tables alike. Replace every
  `e.g.` manual-wide.
- **Reserved fields stay advisory.** Keep the Chapter 5 wording ("reserved for future use;
  software must not rely on them"). Do *not* adopt the M68000 PRM "must be written as
  zeros" form; the implementation does not promise it.

### Notation
- **Hex-literal macro pass applies to all chapters.** Every hex literal in prose or a table
  cell takes `\opcode{XX}` (opcode bytes), `\imm{N}` (immediate operands), `\addr{...}`
  (addresses), or `\texttt{0x...}` (everything else). No hex literal remains in body font.
  Inside `lstlisting` blocks the literal stays as written.
- **Flag tables keep the symbol form and gain `U`.** The symbol set is `*` (updated), `0`,
  `1`, `-` (preserved), `S` (from shifter), and `U` (undefined: the implementation leaves the
  flag in an unspecified state). Chapter 11's notation table must define `U`. Do not rewrite
  flag tables as per-flag prose.

### Prose
- **Examples are numbered.** Each `Example:` block in an instruction entry becomes
  `Example N-M:` where N is the chapter number (the two-digit file prefix without the
  leading zero, or the appendix letter) and M counts from 1 within the chapter. Multiple
  examples in one entry take consecutive numbers. A List of Examples in the front matter is
  new content and is tracked in the handover as Workstream 14.

### Instruction entry template (accepted period-gap fields)
These three additions are applied to every instruction entry in Chapters 13--17 during
the editing phase:
- **Rename the predicated-execution line.** The line currently labelled
  `Condition codes:` describes which condition field values the instruction accepts
  (predicated execution), not the CCR effect. Relabel it `Condition field:` so it cannot be
  confused with the `Flags:` block.
- **Applicability line under the entry title.** Immediately after the title, one line:
  `SIRC-1, all revisions` for CPU instructions; for coprocessor instructions, the
  coprocessor and revision they require, following Chapter 16's revision policy.
- **Instruction Fields list.** After the opcode list, one line per encoding field that this
  instruction uses, naming its legal values here: register fields, condition field, AF
  (with which AF encodings are legal for this instruction), shift type and count, immediate
  width. Take the values from the facts digest and Chapter 7; do not invent them.

### Not adopted
- Instruction Format bit diagram per entry and legal-forms table per entry: tracked in the
  handover as Workstream 14, not applied in this pass.

## Approved decisions (Gate 1, 2026-11-02)

Ruled on by the author for the three-manual pass. All approved as written except item 3, which the author overrode in favour of the period overbar. Citations are to `review/period-style.md`.

1. **American spelling set-wide.** ARCH is American; HW and ASM are British (about 8 hits).
2. **Single-l `canceled`, `labeled`.** The set has only double-l forms (11 `cancelled`, 2 `labelled`, all ARCH). Chosen for dialect consistency, against the majority.
3. **Active-low names take a period overbar** (`\activelow{}`), overriding the proposed plain form. Period manuals use an overbar (UM 3-7); `-IRQ` is rejected (ARM ix).
4. **Captions in Title Case.** ARCH 144 of 157 are Title Case; HW and ASM 27 of 30 are sentence case, so about 30 captions change.
5. **Syntax meta-symbols** `{ }`, `< >` (ARM 2-29, MASM 1-4). Rewrites one `[--segment <SEGMENT>]...` synopsis and the Chapter 4 syntax line.
6. **"you" banned everywhere; imperatives allowed in ASM steps and hints.** Period assemblers use third person (MASM 1-5).
7. **ASM diagnostics quoted verbatim** (MASM E-1 to E-3). Table 7.1 paraphrases today.
8. **Hardware additions not authorized by this sheet:** three-state statement per bus signal (UM 3-10, Z80 5), state narrative under each cycle figure (UM 5-8), AC table (ARM 2-21, UM 10-10).
9. **Instruction entry order stays Opcodes first**, though period manuals lead with Operation (PRM 4-4, Z80 145).
10. **Not applied:** arrow one-liner in Operation, per-flag sentences, value-range column in Table 11.1 (PRM 3-2, 4-4; Z80 39).
11. **`bus cycle` over `bus transaction` (6 vs 3).** The Chapter 4 heading "Bus Transaction Completion" changes.
