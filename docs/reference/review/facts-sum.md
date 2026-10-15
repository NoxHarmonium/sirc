# Review findings: chapter group G8 (summaries and appendices)

Reviewer tag: `sum`. Files reviewed: `chapters/11-reading-instructions.tex`,
`chapters/12-instruction-summary.tex`, `chapters/appendix-a-opcode-map.tex`,
`chapters/appendix-d-examples.tex`, `chapters/appendix-e-quick-reference.tex`.

No facts digest existed for this pass, so every claim below was checked directly against the
`sirc-vm` sources, against the assembler/linker binaries in `sirc-vm/target/debug/`, or against
another chapter of the manual. Checks that passed and are not reported: the 64-row opcode map in
Appendix A (every hex value, binary column, and mnemonic matches
`peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:210-283` and the assembler's
mnemonic tags); the format/save-vs-test/memory-decoding bullet lists in Appendix A (including the
"low word only, always by one word" auto-update rule, `stages/fetch_and_decode.rs:147-157`); the
64-row instruction table and the 20-row meta-instruction table in Chapter 12 (all COPI command
words recomputed from `parsers/opcodes/exception.rs` and `parsers/opcodes/coprocessor_meta.rs`);
the 6-phase timing model (`coprocessors/shared.rs:6-14`); the status-update encoding table and its
defaults in Chapter 11 (`definitions.rs:96-102`, `parsers/opcodes/arithmetic_immediate.rs:98,154`);
Appendix E's 46-row mnemonic index (exactly the 46 tags the assembler accepts), its SR bit table
(`registers.rs:14-40`), and its BAT table (`peripheral-bus/src/device.rs:19-29`); and the byte
layouts, slot size, and label resolution of `.DB`/`.DW`/`.DQ` in Appendix D.1, which I reproduced
by assembling and linking test inputs.

### F-sum-1
- file: chapters/appendix-d-examples.tex
- lines: 42-45
- severity: blocker
- category: fact
- claim: "A label reference, on any of the three directives, always assembles to the full 4-byte slot holding the label's address, since a stored address must be able to address anywhere in the 32-bit address space."
- evidence: chapters/01-introduction.tex:144 ("With a 24-bit address space"); chapters/04-data-representation.tex:38 and :64-77 ("Architectural addresses are 24-bit word addresses", "Stored 24-bit Address Format"); chapters/06-exceptions.tex:599-601; sirc-vm/toolchain/src/types/object.rs:15-16 ("Full 24 bit program address"); linker writes a 24-bit word address zero-extended into 4 bytes, sirc-vm/toolchain/src/bin/linker.rs:130,137-140 (verified: `.DB @lbl` linked to `00 00 00 02`)
- resolution: manual-wrong
- fix: Replace "the 32-bit address space" with "the 24-bit address space", i.e. "...since a stored address must be able to address anywhere in the 24-bit address space." The same 32-bit-address error appears twice more in this section and must be fixed in all three places: line 27 ("it resolves to the label's full 32-bit address") and lines 32-33 ("with a label's full 32-bit address") should both read "full 24-bit address" (the four-byte slot width is already stated separately and is correct).
- confidence: high

### F-sum-2
- file: chapters/appendix-d-examples.tex
- lines: 15-16
- severity: major
- category: contradiction
- claim: "\item[\texttt{.ORG \#addr}] Sets the word address at which subsequent instructions and data are assembled. The address is a bare hexadecimal (\texttt{0x}-prefixed) or decimal number, without the \texttt{\#} used elsewhere for immediate operands."
- evidence: sirc-vm/toolchain/src/parsers/shared.rs:116-120 (`parse_origin_` = `tag(".ORG")` then `alt((parse_hex, parse_dec))`, no `#`); assembling `.ORG #0x0010` fails with `expected "0x" ... in section "origin"`; every listing in this appendix writes `.ORG 0x0000`
- resolution: manual-wrong
- fix: Change the description-list label to `\item[\texttt{.ORG addr}]` so the headword matches the body sentence and the assembler grammar.
- confidence: high

### F-sum-3
- file: chapters/appendix-d-examples.tex
- lines: 21-22
- severity: major
- category: fact
- claim: "\item[\texttt{@label}] References a previously declared label."
- evidence: Example D-1 in this same section (line 62) writes `.DQ @start` before `:start` is declared on line 67, and I assembled and linked that listing successfully (reset slot links to `00 00 00 82`); symbol refs are recorded unresolved by the assembler (sirc-vm/toolchain/src/data/object.rs:83-91) and resolved by the linker over the merged symbol table (sirc-vm/toolchain/src/bin/linker.rs:69-81), so forward references are legal and are the normal case for the reset vector
- resolution: manual-wrong
- fix: "References a label declared anywhere in the program, before or after the reference; references are resolved at link time."
- confidence: high

### F-sum-4
- file: chapters/appendix-d-examples.tex
- lines: 46-49
- severity: major
- category: fact
- claim: "Defines a named constant equal to \texttt{\#value}, usable anywhere a \texttt{\#immediate} operand is accepted by writing \texttt{\$NAME} in its place."
- evidence: data directives accept a `#value` operand but reject `$NAME`: sirc-vm/toolchain/src/parsers/data.rs:36-47 (`parse_data_type` is `alt((parse_number, parse_symbol_reference))` with no placeholder branch); assembling `.DW $ANSWER` fails with `expected '#' ... or expected '@'`. Note that sirc-vm/toolchain/src/data/object.rs:92-96 already contains a `DataType::PlaceHolder` arm that the parser can never produce, which looks like an unfinished toolchain feature rather than a deliberate restriction
- resolution: unclear
- fix: Either narrow the manual to what the toolchain implements -- "usable anywhere an instruction takes a \texttt{\#immediate} operand" (a data directive's value must be a literal or a label reference) -- or keep the broader claim and log the parser gap as an implementation bug (`parse_data_type` needs a `parse_placeholder` branch to reach the existing `DataType::PlaceHolder` handling). Manual-is-canonical suggests the latter; flagging for the editor to decide.
- confidence: high

### F-sum-5
- file: chapters/appendix-d-examples.tex
- lines: 15-18
- severity: major
- category: fact
- claim: "Sets the word address at which subsequent instructions and data are assembled."
- evidence: `.ORG` is stored as a byte offset of `word address * 2` (sirc-vm/toolchain/src/data/object.rs:165-168) and every emitted item is placed at `offset / 4` (object.rs:124), so an odd word address is silently rounded down to the containing 2-word slot. Verified: `.ORG 0x0000 / .DW #0x1111 / .ORG 0x0003 / .DW #0x2222` links to `0000 1111 0000 2222`, i.e. the second value lands at word address 2, not 3
- resolution: unclear
- fix: State the alignment rule, e.g. "The address must be even (2-word aligned), since instructions and data directives each occupy a 2-word slot." If an odd `.ORG` is meant to be legal, the manual should say what it does; the current toolchain silently truncates rather than erroring, which is worth logging as an implementation gap either way.
- confidence: high

### F-sum-6
- file: chapters/appendix-d-examples.tex
- lines: 29-30
- severity: minor
- category: fact
- claim: "An explicit suffix overrides this default: \texttt{@label.r} forces the PC-relative displacement form, \texttt{@label.l} forces the low word, and \texttt{@label.u} forces the high word."
- evidence: the three suffixes map correctly (sirc-vm/toolchain/src/types/shared.rs:5-7, parsers/shared.rs:122-136), but on a data directive any ref type other than `FullAddress` aborts the link: sirc-vm/toolchain/src/bin/linker.rs:125-134 ("Only FullAddress ref type currently supported for data directives"). Verified: `.DW @lbl.l` assembles but panics the linker
- resolution: unclear
- fix: Add a sentence scoping the override to instruction operands, e.g. "The suffixes apply to label references in instruction operands; a label reference in a \texttt{.DB}/\texttt{.DW}/\texttt{.DQ} directive is always a full address." Alternatively record the linker's missing `LowerWord`/`UpperWord`/`Offset` support for data directives as an implementation gap.
- confidence: high

### F-sum-7
- file: chapters/appendix-d-examples.tex
- lines: 24-28
- severity: minor
- category: fact
- claim: "\item In an immediate-form \mnemonic{LOAD}, it resolves to the label's low word only."
- evidence: the same implied low-word default applies to every other instruction that takes a symbol operand: sirc-vm/toolchain/src/parsers/opcodes/store.rs:89-91, ldea.rs:90-92, ldel.rs:83-85, ljmp.rs:102-104, ljsr.rs:113-115, load.rs:94-96; only branching.rs:101-103 uses `RefType::Offset`
- resolution: manual-wrong
- fix: "In any other instruction that takes a symbol operand (\mnemonic{LOAD}, \mnemonic{STOR}, \mnemonic{LDEA}, \mnemonic{LDEL}, \mnemonic{LJMP}, \mnemonic{LJSR}), it resolves to the label's low word only."
- confidence: high

### F-sum-8
- file: chapters/appendix-d-examples.tex
- lines: 52-55
- severity: minor
- category: fact
- claim: "Every program in this appendix begins with a \texttt{.ORG 0x0000} / \texttt{.DQ @label} pair for exactly this reason."
- evidence: examples/faults/faults-high.sasm:13-17 starts `.ORG 0x0000` followed by instructions, not a `.DQ` (it is mapped as a separate segment, per its own comment at lines 10-11); examples/hardware-exception/data.sasm has no `.ORG` at all -- both are printed in this appendix as Examples D-4 and D-7
- resolution: manual-wrong
- fix: "Every program in this appendix that is loaded at segment zero begins with a \texttt{.ORG 0x0000} / \texttt{.DQ @label} pair for exactly this reason; the separately mapped high segment and the standalone data segment shown later do not need one."
- confidence: high

### F-sum-9
- file: chapters/appendix-d-examples.tex
- lines: 64-65, 73
- severity: minor
- category: fact
- claim: "    .ORG 0x0080\n    .DQ @overflow_handler          ; vector 0x40: exception vector table entry"
- evidence: chapters/06-exceptions.tex:620 assigns vector 0x40 (table address 0x0080-0x0081) to the Level 2 hardware exception; the segment-overflow fault is vector 0x03 at address 0x0006 (06-exceptions.tex:609). The address arithmetic in the comment is right, but the handler name implies a fault vector that lives elsewhere
- resolution: manual-wrong
- fix: Rename the label in Example D-1 to `:level2_handler` (both the `.DQ @...` on line 65 and the `:...` on line 73) and keep the comment as "vector 0x40: level 2 hardware exception vector".
- confidence: high

### F-sum-10
- file: chapters/appendix-d-examples.tex
- lines: 108
- severity: minor
- category: fact
- claim: "Each handler, defined from line 144 onward, increments a distinct register so a reader can confirm which faults fired and in what order."
- evidence: examples/faults/faults.sasm line 144 is inside the level-five interrupt test; the handler section starts at line 158 (`.ORG 0x0300`) with `:bus_fault_handler` at line 160. The file gained 66 lines in commit a5d82c3 (2026-09-14), after this prose was written, so the reference has drifted
- resolution: manual-wrong
- fix: "Each handler, defined from line 158 onward, increments a distinct register..."
- confidence: high

### F-sum-11
- file: chapters/appendix-d-examples.tex
- lines: 109-111
- severity: minor
- category: fact
- claim: "The segment-overflow handler (lines 167--204) is worth particular attention"
- evidence: examples/faults/faults.sasm has `:segment_overflow_fault_handler` at line 197 and `:segment_overflow_fault_handler_end` / `RETE` at lines 240-242; lines 167-204 currently span the tail of `:bus_fault_handler`, the bus-protection and alignment handlers, and only the first few lines of the segment-overflow handler
- resolution: manual-wrong
- fix: "The segment-overflow handler (lines 197--242) is worth particular attention"
- confidence: high

### F-sum-12
- file: chapters/appendix-d-examples.tex
- lines: 87-89
- severity: minor
- category: fact
- claim: "storing one byte per candidate number in a memory-mapped array rather than in a register or the stack"
- evidence: memory is word-addressed and the smallest addressable unit is a 16-bit word (chapters/01-introduction.tex:144, chapters/04-data-representation.tex:38); the sieve marks composites with `STOR (r2, a), r3` (examples/byte-sieve/byte-sieve.sasm:47), a full 16-bit word store, and indexes the array one word per candidate
- resolution: manual-wrong
- fix: "storing one word per candidate number in a memory-mapped array rather than in a register or the stack" (the program's name comes from the classic BYTE-magazine sieve, not from a byte-sized element).
- confidence: medium

### F-sum-13
- file: chapters/appendix-d-examples.tex
- lines: 132
- severity: minor
- category: terminology
- claim: "The exception handler itself is installed at vector 0x0080 and defined from line 137 onward"
- evidence: chapters/06-exceptions.tex:599-623 defines vectors as IDs 0x00-0xFF whose table entry sits at `vector * 2`; 0x0080 is the table address of vector 0x40. examples/hardware-exception/hardware-exception.sasm:35-36 uses `.ORG 0x0080`
- resolution: manual-wrong
- fix: "The exception handler itself is installed at vector 0x40 (table address 0x0080) and defined from line 136 onward". The same word-address-as-vector-number slip occurs on line 155 ("the handler at vector 0x0500 (line 37)"), where 0x0500 is the handler's own `.ORG` address, not a vector: reword as "the handler placed at address 0x0500 (line 37)".
- confidence: high

### F-sum-14
- file: chapters/11-reading-instructions.tex
- lines: 132-134
- severity: minor
- category: contradiction
- claim: "\mnemonic{LOAD} always encodes \texttt{[N]} and never updates the status flags, regardless of any override syntax."
- evidence: chapters/13-alu-instructions.tex:178-180 states the precise rule -- "Public \mnemonic{LOAD} forms do not support status update override syntax and always preserve the status flags; an encoding of the \mnemonic{LOAD} opcode with a non-\texttt{None} AF field, which is not reachable from public assembly syntax, leaves all four status flags architecturally undefined"; the assembler rejects any override on LOAD outright (sirc-vm/toolchain/src/parsers/opcodes/load.rs:29-37)
- resolution: manual-wrong
- fix: "\mnemonic{LOAD} does not accept status update override syntax; it always encodes \texttt{[N]} and preserves the status flags (see Chapter~\ref{ch:alu-instructions} for the behaviour of a hand-built \mnemonic{LOAD} encoding with a non-\texttt{[N]} AF field)." The matching footnote at chapters/12-instruction-summary.tex:101-104 has the same problem -- it says "an explicit status update override on \mnemonic{LOAD} leaves all four flags architecturally undefined", which implies the override is writable in assembly; it should say "an encoding of the \mnemonic{LOAD} opcode with a non-\texttt{[N]} AF field, not reachable from assembly syntax, leaves all four flags architecturally undefined".
- confidence: high

### F-sum-15
- file: chapters/11-reading-instructions.tex
- lines: 19-22
- severity: minor
- category: structure
- claim: "Values come from Chapter~\ref{ch:instruction-formats} and the facts digest; an entry must not invent a value here."
- evidence: "the facts digest" is an internal review artifact (`docs/reference/review/`), not a part of the published manual; there is no reader-visible document of that name, and the sentence is an instruction to authors rather than a statement about the architecture
- resolution: manual-wrong
- fix: "Values come from Chapter~\ref{ch:instruction-formats}; an entry does not introduce a value that is not defined there."
- confidence: high

### F-sum-16
- file: chapters/11-reading-instructions.tex
- lines: 73-75
- severity: nit
- category: fact
- claim: "\texttt{@label} & Symbol reference resolved by the linker; PC-relative for \mnemonic{BRAN} and \mnemonic{BRSR}, a low-word address otherwise"
- evidence: the cross-reference itself is correct -- `sec:assembler-directives` resolves to Appendix D.1 (main.aux:656, `{D.1}{187}`) -- but "otherwise" is not true for data directives, which resolve to a full address (sirc-vm/toolchain/src/parsers/data.rs:91-98); the parenthetical "including data directives" partly covers this
- resolution: manual-wrong
- fix: "...PC-relative for \mnemonic{BRAN} and \mnemonic{BRSR}, a low-word address in other instruction operands (see Section~\ref{sec:assembler-directives} for the full resolution rules, including data directives)".
- confidence: medium
