# Cold read: SIRC-1 reference manual (Phase 6)

Findings from a straight-through, first-time read of `docs/reference/chapters` in chapter order.
Ranked most damaging first. Each item names the chapter and a concrete fix.

### 1. RETE/ETFR/ETTR/EXCP/COPI are used constantly in Chapters 5–6 but not defined until Chapter 16
`ch:status-register` first uses `RETE` describing the EA bit ("Automatically Cleared: ... via RETE"),
and `ch:exceptions` uses `RETE`, `EXCP`, `ETFR`, `ETTR`, and `COPI` dozens of times, including a full
worked code example (`fault_handler`, lines ~246–275 of `06-exceptions.tex`) that calls `ETFR #6` /
`ETTR #7` roughly 430 lines *before* the same chapter's own "Accessing Link Registers" section
explains what those mnemonics do (line ~681). The real instruction-box definitions (opcode, syntax,
operation) don't appear until `ch:coprocessor` (Chapter 16), ten chapters later. A first-time reader
spends the entire exception model not knowing what these words assemble to.
Fix: add a one-line forward pointer ("see Chapter 16 for full syntax") the first time each mnemonic
is used in Chapters 5 and 6, and move the fault-handler example after the ETFR/ETTR explanation
within Chapter 6, or add a two-line inline gloss before the example.

### 2. `SR.X` dot-notation and register/operand notation (rD, rS, addr/src, `SR.X`) aren't glossed until Chapter 11
`04-data-representation.tex` uses `SR.A` ("Trap on Address Overflow") before Chapter 5 even names the
status-register bits, and before Chapter 11's notation table (`tab:instruction-description-notation`)
defines the `SR.X` convention at all. The convention is guessable, but a reader hits it cold three
chapters early with no gloss and no forward reference.
Fix: move the notation table (or a trimmed version of it) into Chapter 1 or Chapter 3, and leave
Chapter 11's table as the canonical, expanded version referenced back to.

### 3. `\reg{pl}`/`\reg{ph}` used in Chapter 2 before Chapter 3 explains the high/low split
`02-cpu-architecture.tex`'s Phase 1–2 descriptions ("Advance `pl` by 2... `ph` is unchanged") use the
low/high register-half naming convention before `03-registers.tex` introduces that every address-register
pair splits into an `Xh`/`Xl` component. On first read, "pl" and "ph" look like typos or unexplained new
registers.
Fix: add one sentence in Chapter 2's phase-0 description ("each address-register pair splits into a
high and low half, see Chapter 3") or swap the chapter order so registers precede CPU architecture.

### 4. Appendix D (Example Programs) is a wall of code with no orienting prose
`appendix-d-examples.tex` is six `\lstinputlisting` includes under one-line section headers ("Byte
Sieve", "Faults", ...) with zero commentary on what each program demonstrates or which lines matter.
`faults.sasm` is 267 lines and `hardware-exception.sasm` is 214 lines; a reader is dropped into raw
assembly with no roadmap, which is exactly the "wall with no orienting prose" failure mode.
Fix: add 2–3 sentences per example naming the specific mechanism being demonstrated (e.g., "this
program deliberately triggers a segment-overflow fault at line N and shows the retry idiom") and
point at the specific instructions worth reading closely.

### 5. Chapter 1 uses `COPI`/`COPR` and the letters `l, a, s, p` with no expansion or forward pointer
`01-introduction.tex`'s Coprocessors section says devices are "activated using the COPI or COPR
coprocessor-call instructions" with no forward reference to where these are defined (Chapter 16). The
Key Features table also lists "Address Register Pairs: 4 (l, a, s, p)" with no expansion of what the
letters stand for; that only appears in Chapter 3.
Fix: add "(see Chapter 16)" after COPI/COPR, and expand or footnote the l/a/s/p abbreviations the
first time they appear in a table.

### 6. Chapter 6's L5-vs-fault interaction paragraph requires two reads
The paragraph beginning "These two rules compose in a non-obvious way..." (`06-exceptions.tex`,
~line 432) walks through a fault-inside-an-L5-handler scenario relying on priority numbers (6 vs 7)
introduced pages earlier, and the text itself flags the interaction as "non-obvious." Prose alone
doesn't make the sequence stick.
Fix: replace or supplement the paragraph with a 4-step timeline diagram/table (time → active level →
event → outcome), which the surrounding pages otherwise favor for less complex material.

### 7. Chapter 8's two big tables ("Addressing Mode Matrix" and "Legal Modes by Instruction Family") overlap without a bridge
Both tables encode which addressing modes are legal for which instructions, from different angles
(mode-first vs. instruction-first), immediately back to back with no paragraph telling the reader
when to consult one versus the other or that they must agree.
Fix: add one sentence: "Use the mode-first table to check a syntax form's semantics; use the
instruction-first table to check whether a given instruction accepts a form."

### 8. Chapter 10's unsigned-comparison truth table breaks its own column layout on one row
In the "Unsigned Comparison (CMPR rA, rB)" table (`10-condition-codes.tex`), the "rA ≤ rB" row uses
`\multicolumn{2}{c}{C = 0 OR Z = 1}` spanning the Z and C columns, while every other row has separate
Z and C values. The row has to be read twice because it briefly looks misaligned with the rest of the
table.
Fix: keep Z and C in separate cells (e.g. "0 or 1" / "0 or 1" with a footnote) so every row has the
same shape.

### 9. Chapter 12's meta-instruction cross-reference (the clearest map of EXCP/RETE/WAIT/etc. to real opcodes) arrives after Chapter 6 has already used them heavily
`12-instruction-summary.tex`'s "Meta-Instructions" table is the first place a reader can see, in one
glance, that `RETE` = `COPI #0x1A00` etc. Chapter 6 (Exceptions) uses these mnemonics extensively
without this map being available yet, compounding finding #1.
Fix: pull a trimmed copy of the exception-related rows of this table into Chapter 6's overview, or add
a forward pointer to Table `tab:meta-instructions` the first time `RETE`/`EXCP`/`ETFR` appear.

### 10. Chapter 2's pin/timing material and Chapter 6's exception material overlap and are read out of order
Reset and interrupt-sampling behavior is described in detail in Chapter 2 (External Interface,
RSTI/RSTO/IRQ pins, timing diagrams) using vocabulary ("reset-output hold", "pending
hardware-exceptions register") that Chapter 6 later defines properly. A reader hits electrical-level
reset sequencing before the architectural concept of "exception level" or "pending interrupt" exists.
Fix: either move Chapter 2's External Interface section after Chapter 6, or trim it in Chapter 2 to
signal names/timing only and push all behavioral vocabulary to a forward reference.

---

## Top three

1. **Finding 1** — the exception-handling chapters (5–6) are unreadable on a first pass because they
   lean entirely on mnemonics (`RETE`, `ETFR`, `ETTR`, `EXCP`) that aren't defined until Chapter 16;
   the in-chapter worked example even uses `ETFR`/`ETTR` before the same chapter explains them.
2. **Finding 3** — `pl`/`ph` notation appears in Chapter 2 before Chapter 3 explains that address
   registers split into high/low halves, so the six-phase execution model — the most load-bearing
   diagram in the book — is confusing on the very first read-through.
3. **Finding 4** — Appendix D's example programs are dropped in with no walkthrough prose, wasting
   the one part of the manual that could otherwise cement everything taught in Parts I–III.

Overall: this reads as a rigorously fact-checked reference that has not yet been read start-to-finish
by its own authors — the forward-reference gaps cluster exactly where a heavy multi-agent edit would
leave seams (Chapters 5–6 and Chapter 11's notation table), suggesting a draft nearing completion
rather than a finished one.
