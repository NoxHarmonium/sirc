# SIRC-1 Reference Manual — House Style Sheet

For copy editors working across the 22 chapters/appendices of the SIRC-1 CPU Reference
Manual. Rules were set by majority usage found in the current chapters, except where the
handover ("Resolved:" items), Chapter 11, or a 2026-09-05 benchmark against period CPU
manuals (M68000 and MCS6500 family reference manuals) already dictate a form. Apply these
rules; do not introduce new house-style debates during copyediting. Every inconsistency found
during the initial style derivation was ruled on by the author and is recorded in
**Approved decisions** at the end.

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
| displacement | offset (for addressing-mode/branch values) | Keep "displacement" for addressing-mode and branch/call values; keep "offset" for memory/structure offsets and the `system_ram_offset` register name. Do not swap the two. |
| R1/R2/R3 (upper case, no macro) | — | Reserved for abstract *field-position* labels in instruction-format diagrams and field-order prose (Ch.7). Never use for a concrete register operand. |
| `\reg{r7}` | bare "R7" for a concrete register | `06-exceptions.tex:648–649` uses bare "R7" for the real register `r7`; correct to `\reg{r7}` to match the 89-instance convention. |

## 2. Normative vocabulary

- **must / must not** — a mandatory architectural requirement; non-conformant hardware or software is broken.
- **may** — a permitted option; the CPU, the assembler, or the programmer is free to choose.
- **should / should not** — a recommendation. Not required for conformance. Do not use for anything that is actually mandatory.
- **shall** — do not use. The manual currently has zero instances; keep it that way and use "must" instead.
- **reserved** — a field, opcode, or coprocessor ID not currently assigned. Software must not assume a read value and must write zero unless a chapter states an explicit encoding. (See Decisions needing approval — current wording is softer than this in places.)
- **undefined / architecturally undefined** — the architecture does not specify the result. It may vary by implementation, by revision, or between executions; software must not depend on any particular outcome, and doing so is not a portability guarantee. Prefer "architecturally undefined" in instruction-reference prose; bare "undefined" is acceptable in flag tables and informal descriptions.
- **implementation-defined** — behavior that is fixed and consistent within one CPU implementation/model but may differ between models (e.g., extra maths-coprocessor cycles). Not a synonym for "architecturally undefined."
- **unpredictable** — not used in this manual. Do not introduce it as a synonym for "undefined"; the manual already distinguishes "architecturally undefined" from "implementation-defined" and a third term would blur that line.

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
