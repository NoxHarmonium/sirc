# Facts review: assembly programmer's manual (G11, tag `asm`)

Scope: docs/assembly/chapters/preface.tex, 01-introduction.tex through 08-limitations.tex. Checked against
docs/reference/review/facts.md (Toolchain), sirc-vm/toolchain (parsers, bin/, data/object.rs, types/object.rs),
sirc-vm/sirc-vm/src/main.rs and the architecture manual. `make listings` in docs/assembly: all 11 tagged listings
assemble. The Ch.6 worked example was assembled, linked and run; the register dump shows r1 = 0xa as claimed, and the
Ch.7 error example reproduces line for line (apart from the panic wrapper, F-asm-17).

### F-asm-1
- file: assembly/chapters/03-operands.tex
- lines: 69-76
- severity: blocker
- category: contradiction
- claim: "These two lines assemble to the same instruction: ... ADDR r1, r2, r3 ; r1 = r2 + r3 / ADDR r1, r2 ; r1 = r1 + r2"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_register.rs:102-121 (two-register form sets r2 = destination). Assembled and linked: `ADDR r1, r2, r3` = 0xC0048010, `ADDR r1, r2` = 0xC0004010, `ADDR r1, r1, r2` = 0xC0004010. The listing's own comments give two different operations.
- resolution: manual-wrong
- fix: Change the first listing line to `    ADDR r1, r1, r2         ; r1 = r1 + r2` (keep the second line as is).
- confidence: high

### F-asm-2
- file: assembly/chapters/01-introduction.tex
- lines: 15
- severity: major
- category: contradiction
- claim: "SIRC Assembly Programmer's Manual  & Complete assembly syntax, directives, macros, pseudoinstructions (this manual)"
- evidence: assembly/chapters/08-limitations.tex:13,20 and 04-instructions.tex:104-105 (no macros, no multi-instruction pseudoinstructions). Known; already reported in review/consistency.md:52-55 and review/facts-arch.md:164-165.
- resolution: manual-wrong
- fix: Report once. Apply the consistency.md fix ("Complete assembly syntax, directives, symbols, and the toolchain") at assembly/chapters/01-introduction.tex:15, hardware/chapters/01-introduction.tex:15, reference/chapters/preface.tex:55 and both READMEs.
- confidence: high

### F-asm-3
- file: assembly/chapters/03-operands.tex
- lines: 69-79
- severity: major
- category: fact
- claim: "A register-form ALU instruction with two register operands is shorthand for the three-operand form with the destination repeated as the first source."
- evidence: Known from other reviewers: review/facts-alu.md F-alu-1 (line 20-22) and review/facts-enc.md:96-98. The assembler accepts `CMPR r1, r2, r3` / `TSAR ...` three-register forms (arithmetic_register.rs:122-141,167-190) that put an ignored register in R1. Neither manual documents them, and this section does not say how the test mnemonics (CMNR..TSXR), which have no destination, fit the rule.
- resolution: design
- fix: Follow the option chosen for F-alu-1. Recommended: the toolchain rejects three-register test forms, and this section adds "The test instructions (CMNR, CNCR, CMPR, CPCR, TSAR, TSOR, TSXR) have no destination and take exactly two registers, `rS1, rS2[, shift]`."
- confidence: high

### F-asm-4
- file: assembly/chapters/03-operands.tex
- lines: 4-6, 10-32
- severity: major
- category: fact
- claim: "This chapter lists every operand form the assembler accepts."
- evidence: The zero-displacement shorthand `(addr)`, `(addr)+` and `-(addr)` is accepted (sirc-vm/toolchain/src/parsers/instruction.rs:150-160; `LOAD r1, (a)+`, `STOR -(s), r1` and `LJSR (a)+` all assemble). The architecture manual defines it normatively (reference/chapters/08-addressing-modes.tex:105-132; 14-memory-instructions.tex:71-74; 15-control-flow.tex:64-70, including "LJMP (src) and LJSR (src) are not public syntax"). Table 3.1 omits all three forms.
- resolution: manual-wrong
- fix: Add a paragraph after Table 3.1: "An immediate displacement of zero may be omitted: `(addr)`, `(addr)+` and `-(addr)` are the same as `(#0, addr)`, `(#0, addr)+` and `-(#0, addr)`. The shorthand is accepted only where the full form is, and `LJMP`/`LJSR` take `addr` rather than `(addr)`. Register displacements have no shorthand."
- confidence: high

### F-asm-5
- file: assembly/chapters/03-operands.tex
- lines: 38-41
- severity: major
- category: fact
- claim: "An immediate operand may be a number, a label reference, or a named constant."
- evidence: ALU immediates (ADDI..TSXI), COPI, EXCP, ETFR/ETTR and the DMA/maths meta-instructions reject label references: arithmetic_immediate.rs:206-214,270-278 ("The [ADDI] opcode does not support symbol refs at this time"); facts.md "Default symbol-reference types". Ran `ADDI r1, @x`: rejected. The linker can patch only opcodes 0x07, 0x10, 0x14, 0x18, 0x1A, 0x1C, 0x1E (sirc-vm/toolchain/src/bin/linker.rs:170-177). Ch.5 Table 5.2 (05-directives-and-symbols.tex:76-78) lists only MOVI, LOAD, STOR, LDEA, LDEL, LJMP, LJSR, BRAN, BRSR, which hints at the restriction but never states it. reference/chapters/notation-and-glossary.tex:97 makes the same general claim.
- resolution: design
- fix: Option (a), recommended: document the limitation. Ch.3: "A label reference is accepted only by MOVI, LOAD, STOR, LDEA, LDEL, LJMP, LJSR, BRAN and BRSR; other instructions with an immediate operand accept a number or a named constant only." Add a row to Table 8.1 ("Label references in ALU and coprocessor immediates: load the address with MOVI first"). Option (b): extend the assembler and linker to patch every immediate-format opcode.
- confidence: high

### F-asm-6
- file: assembly/chapters/03-operands.tex
- lines: 34-36
- severity: major
- category: fact
- claim: "The offset in parentheses may be a number, a label reference or a named constant."
- evidence: The parser accepts `@sym` in `LOAD rD, (@sym, A)+` (opcode 0x16) and `STOR -(@sym, A), rS` (0x12) (load.rs:164-177; store.rs:181-194; both assemble here). The linker has no patch arm for them and panics with "Can't patch address/offset for instruction" (linker.rs:170-177,203-208). facts.md Open question 3. The same gap makes 07-diagnostics.tex:50 "The linker reports two problems" incomplete, because this panic can be reached from source the assembler accepts.
- resolution: code-wrong
- fix: Keep the manual. Toolchain todo: add LoadRegisterFromIndirectImmediatePostIncrement (0x16) and StoreRegisterToIndirectImmediatePreDecrement (0x12) to the linker's patchable list. Until that lands, either note the gap in Table 8.1 or reject `@sym` in those two forms at assemble time.
- confidence: high

### F-asm-7
- file: assembly/chapters/05-directives-and-symbols.tex
- lines: 7-10
- severity: major
- category: fact
- claim: "\texttt{.ORG addr} sets the word address at which the following instructions and data are assembled."
- evidence: .ORG sets an offset within the current object only (sirc-vm/toolchain/src/data/object.rs:177-180). The linker then shifts every object by the total size of the objects before it (sirc-vm/toolchain/src/types/object.rs:53-80). Ran two objects: m1 (.ORG 0, .DQ, .ORG 0x10, one instruction; 36 bytes) and m2 (`.ORG 0x0040` then `NOOP`). The NOOP lands at byte 0xA4 (word 0x52), not word 0x40. So .ORG is absolute only in the first object, and "put code or data at fixed addresses such as the reset and exception vector table" works only there. 06-toolchain.tex:56 and 08-limitations.tex:17 ("Place code and data with .ORG") depend on the same rule.
- resolution: design
- fix: Option (a), recommended for now: state the rule. Add to Section 5.1: "In a program linked from several objects, .ORG is relative to the start of its own object, and each object starts where the previous one ends. Put the vector table and any other fixed-address code in the first object." Option (b): make the linker place objects at absolute .ORG addresses and diagnose overlaps.
- confidence: high

### F-asm-8
- file: assembly/chapters/05-directives-and-symbols.tex
- lines: 46-48
- severity: major
- category: fact
- claim: "Each may also take a label reference or a named constant instead of a number."
- evidence: For a label reference the linker ignores the directive's size and writes the full 32-bit word address into all 4 bytes for .DB, .DW and .DQ alike (sirc-vm/toolchain/src/bin/linker.rs:64-84,150-158; object.rs:81-89 records no size). A named constant, by contrast, is truncated to the directive's size (object.rs:90-105). reference/chapters/notation-and-glossary.tex:130-131 states this: "A label reference always fills the whole slot". Table 5.1 implies that `.DB @label` puts the value in the low byte.
- resolution: manual-wrong
- fix: Replace the sentence with: "Each may also take a named constant, which is placed and truncated like a number, or a label reference. A label reference with no suffix always fills the whole slot with the label's address, whichever of the three directives is used; with .l, .u or .r the 16-bit value goes in the low word."
- confidence: high

### F-asm-9
- file: assembly/chapters/02-source-language.tex
- lines: 71-83
- severity: major
- category: fact
- claim: "A constant stands in for a numeric value in an immediate operand or in a data directive."
- evidence: Ran these cases. (1) `.EQU $N #-1` then `ADDI r1, $N` panics with "Immediate value (resolved from [N] placeholder) can only be up to 16 bits (4294967295 > 0xFFFF)" (object.rs:25-29). `ADDI r1, #-1` assembles, so a negative constant behaves differently from the same literal. (2) A constant is rejected as a shift count (`LSL $S`; instruction.rs:221-249 uses parse_number only). (3) EXCP and ETFR/ETTR reject constants ("does not support symbol refs or placeholders at this time"). Same claim at 05-directives-and-symbols.tex:52-53 and reference/chapters/notation-and-glossary.tex:132-133 ("usable anywhere a #immediate operand is accepted").
- resolution: code-wrong
- fix: Keep the manual's intent. Toolchain todo: resolve placeholders with the same u16 truncation as literals (or accept values in -32768..65535), and accept them in shift counts, EXCP and ETFR/ETTR. If the author prefers to keep the code, add to Table 8.1: "Named constants in shift counts, EXCP, ETFR and ETTR, and negative named constants in instructions".
- confidence: high

### F-asm-10
- file: assembly/chapters/02-source-language.tex
- lines: 71-80
- severity: major
- category: fact
- claim: "\texttt{.EQU} defines a named constant. The name starts with a dollar sign at the definition and at every use"
- evidence: Placeholders resolve in token order, so a use before its .EQU panics with "Could not find a value for placeholder name [X]" (object.rs:22-23,93-94,192-194; reproduced with `.DQ $X` ahead of its definition). A second .EQU of the same name silently replaces the value for later uses (HashMap insert, object.rs:193). Labels, by contrast, may be used before declaration (02-source-language.tex:35-36). The manual states neither rule.
- resolution: design
- fix: Recommended: document the current behaviour. "A constant must be defined before its first use in the file. Defining the same name again changes its value for the lines that follow." Alternative: make the assembler resolve constants in a second pass and reject redefinition.
- confidence: high

### F-asm-11
- file: assembly/chapters/02-source-language.tex
- lines: 35-36
- severity: major
- category: fact
- claim: "A label may be referred to before or after its declaration, and from any object file linked into the same program."
- evidence: Duplicate labels are not diagnosed. Symbols are collected into one list (types/object.rs:59-79), and the linker resolves a reference to the first match (linker.rs:120-130), whether the duplicate is in the same file or a different object. 08-limitations.tex:18 says "Give every label a unique name" but not what happens otherwise.
- resolution: design
- fix: Recommended: make a duplicate label an assembler error (within a file) and a linker error (across files), and add "A label name may be declared only once in a program" here. Minimum alternative: document "If a name is declared more than once, every reference resolves to the first declaration in link order."
- confidence: high

### F-asm-12
- file: assembly/chapters/06-toolchain.tex
- lines: 85-87
- severity: major
- category: fact
- claim: "With a file, the segment is initialised from it."
- evidence: The file is memory-mapped read-write (sirc-vm/device-ram/src/lib.rs:61-69, `OpenOptions::new().read(true).write(true)`, `map_mut`), and segments from --segment are writable by default (sirc-vm/sirc-vm/src/main.rs:62-83). Every STOR into the segment therefore changes the file on disk. The file must already exist and be writable, or the VM panics (`.unwrap()` at lib.rs:68-69).
- resolution: manual-wrong
- fix: "With a file, the segment is mapped onto it: the program reads the file's contents, and every store to the segment writes through to the file. The file must exist and be writable."
- confidence: high

### F-asm-13
- file: assembly/chapters/02-source-language.tex
- lines: 58-59
- severity: major
- category: fact
- claim: "for example an immediate operand of a 16-bit instruction is 16 bits wide."
- evidence: Every SIRC instruction is 32 bits (facts.md "Instruction formats"; reference/chapters/07-instruction-formats.tex). The 16-bit field is the immediate format's, and the short immediate format's is 8 bits (03-operands.tex:43-44).
- resolution: manual-wrong
- fix: "for example the immediate operand of an immediate-format instruction is 16 bits wide, and that of a short-immediate (shifted) instruction is 8 bits."
- confidence: high

### F-asm-14
- file: assembly/chapters/05-directives-and-symbols.tex
- lines: 8-9
- severity: minor
- category: fact
- claim: "It must be even, because instructions and data directives each occupy two words."
- evidence: Not enforced. `.ORG 0x0001` assembles without a diagnostic. object.rs:177-180 sets the byte offset to 2, and the next slot is written at `offset/4` (object.rs:133), which is the slot at word 0. Labels, however, record word 1. So the code overwrites the previous slot and its labels are off by one word. reference/chapters/notation-and-glossary.tex:122 states the same rule.
- resolution: code-wrong
- fix: Keep the manual. Toolchain todo: reject an odd .ORG address with a diagnostic.
- confidence: high

### F-asm-15
- file: assembly/chapters/02-source-language.tex
- lines: 61-62
- severity: minor
- category: fact
- claim: "Two directive operands are written without the hash: the address of \texttt{.ORG} is a bare decimal or \texttt{0x}-prefixed hexadecimal number"
- evidence: Only one operand is named. The other unhashed directive operand is the .EQU name (`$NAME`; data.rs:77-83). .ORG also rejects binary (shared.rs:116-120: `alt((parse_hex, parse_dec))`; `.ORG 0b10` fails).
- resolution: manual-wrong
- fix: "One directive operand is written without the hash: the address of .ORG, which is a bare decimal or 0x-prefixed hexadecimal number (binary is not accepted) (Section 5.1)."
- confidence: medium

### F-asm-16
- file: assembly/chapters/04-instructions.tex
- lines: 13-14
- severity: minor
- category: fact
- claim: "The operands follow after at least one space."
- evidence: parse_instruction_tag consumes `space0` after the mnemonic, suffix and condition (instruction.rs:401). `ADDIr1, #1` assembles.
- resolution: code-wrong
- fix: Keep the manual. Toolchain todo: require `space1` between the mnemonic group and the first operand when operands are present.
- confidence: high

### F-asm-17
- file: assembly/chapters/07-diagnostics.tex
- lines: 6-22
- severity: minor
- category: fact
- claim: "The assembler stops at the first error. It prints the file, line and column ... error: parsing failed"
- evidence: The assembler reports the error by panicking (sirc-vm/toolchain/src/bin/assembler.rs:48-58). The real output starts with "thread 'main' panicked at toolchain/src/bin/assembler.rs:58:13:" / "Error parsing file:" and ends with the RUST_BACKTRACE note, and the process exits with status 101. Undefined constants and an oversized constant also surface as bare panics (object.rs:22-29,93-94), not in this format. The rest of the example reproduces exactly.
- resolution: code-wrong
- fix: Keep the manual's format as the intended output. Toolchain todo: print the formatted error to stderr and exit with a non-zero status without panicking, and route the placeholder errors through the same formatter. Optionally add "The assembler exits with a non-zero status on any error" to Section 7.1.
- confidence: high

### F-asm-18
- file: assembly/chapters/07-diagnostics.tex
- lines: 45-46
- severity: minor
- category: fact
- claim: "When an instruction is given operands it does not accept, the message lists the operand forms the assembler tried in turn."
- evidence: For a well-formed but unsupported operand list the message lists the operands as parsed, in Rust Debug form: `LOAD r1, (#1, a), r2` gives "Invalid addressing mode for LOAD: ([DirectRegister(R1), IndirectImmediateDisplacement(Value(1), Address), DirectRegister(R2)])" (load.rs:242-248). ALU instructions give a fixed example instead (arithmetic_immediate.rs:283). Only a token that fails to parse produces an "expected ..." list of alternatives (`LOAD r1, (r2)`).
- resolution: manual-wrong
- fix: "When an instruction is given operands it does not accept, the message names the instruction and either shows the operands it parsed or gives an example of an accepted form. When an operand cannot be parsed at all, the message lists the tokens the assembler expected at that point."
- confidence: high

### F-asm-19
- file: assembly/chapters/07-diagnostics.tex
- lines: 38
- severity: minor
- category: fact
- claim: "An instruction whose destination overlaps its address register & Aliased register writes are undefined for that instruction"
- evidence: The rejected set includes cases where no destination is involved: STOR pre-decrement whose source is a half of the pair, LDEL post-increment with source `l`, and LJSR post-increment from `l` or `p` (opcodes/mod.rs:29-49; facts.md "Forms rejected"). The assembly manual, which is authoritative for what the assembler accepts, never lists the set. Also known (review/facts-enc.md:52-54): the architecture manual says auto-update of `p` is rejected, but `LOAD r1, (#0, p)+` and `STOR -(#0, p), r1` assemble (confirmed here).
- resolution: manual-wrong
- fix: Change the row to "A pre-decrement or post-increment form whose other register is a half of the updated pair, or an LDEL/LJSR that would write `l` twice". Add a sentence pointing to the exact list in the Memory Access and Control Flow chapters of the architecture manual. The `p` gap stays as tracked in facts-enc.
- confidence: medium

### F-asm-20
- file: assembly/chapters/06-toolchain.tex
- lines: 65-67
- severity: minor
- category: fact
- claim: "\texttt{--segment-offset} (short form \texttt{-s}) is added to every label's address before it is patched in."
- evidence: N is parsed as a decimal u32 (clap `u32`, linker.rs:96-97), so `-s 0x10` is rejected. It is a word offset (linker.rs:136-140), and the option is required. The simulator's --segment offsets are hexadecimal (06-toolchain.tex:86), so readers will expect hex here too.
- resolution: manual-wrong
- fix: Add: "N is a decimal number of words, and the option is required." Optionally add the hex/decimal contrast with --segment.
- confidence: high

### F-asm-21
- file: assembly/chapters/03-operands.tex
- lines: 69-70
- severity: minor
- category: fact
- claim: "A register-form ALU instruction with two register operands is shorthand for the three-operand form"
- evidence: MOVR is in the ALU family (04-instructions.tex:89), but `MOVR rD, rS` is its only form. It encodes r2 = 0, not a copy of rD, and a three-register MOVR is rejected (mov.rs:137-160,176-182).
- resolution: manual-wrong
- fix: "A register-form ALU instruction other than MOVR, written with two register operands, is shorthand for ..."
- confidence: high

### F-asm-22
- file: assembly/chapters/02-source-language.tex
- lines: 58-59
- severity: minor
- category: fact
- claim: "A negative decimal number is stored in two's complement. The value must fit the field it is encoded into"
- evidence: A negative literal in the 8-bit short-immediate field is always rejected. `ADDI r1, #-3, LSL #1` gives "Immediate values must fit into 8 bits when using a shift definition (65533 > 0xFF)", because the value is truncated to u16 before the 8-bit check (instruction.rs:114-116; arithmetic_immediate.rs:227-234). A negative shift count is rejected as 4294967295. A reader could take "fit the field" to mean that -128..-1 fit an 8-bit field.
- resolution: manual-wrong
- fix: Add: "The 8-bit short-immediate field and shift counts are unsigned. A negative number is rejected there."
- confidence: medium

### F-asm-23
- file: assembly/chapters/08-limitations.tex
- lines: 27-28
- severity: minor
- category: fact
- claim: "The architecture requires that every meta-instruction assemble to exactly one instruction, so a source line is always one instruction"
- evidence: A line can hold a label, directive, comment or nothing. Tokens are separated by optional whitespace, not newlines, so `:a1 :b1 NOOP` and `NOOP NOOP` on one line both assemble (shared.rs:255-267).
- resolution: manual-wrong
- fix: "... so every mnemonic written in the source produces exactly one instruction (see ...)."
- confidence: high

### F-asm-24
- file: assembly/chapters/01-introduction.tex
- lines: 35
- severity: minor
- category: fact
- claim: "The \textbf{simulator} loads the binary at address zero, maps whatever memory the program needs, and runs it."
- evidence: The simulator always maps fixed segments: PROGRAM 0x0 (0xFFFF words, read-only), TERMINAL 0x0A0000 and DEBUG 0x0B0000, plus VIDEO with -e. Any other memory must be mapped with --segment (sirc-vm/sirc-vm/src/main.rs:191-262). It does not work out what the program needs.
- resolution: manual-wrong
- fix: "The simulator loads the binary at address zero, maps its fixed devices and any memory segments given on the command line, and runs it."
- confidence: high

### F-asm-25
- file: assembly/chapters/04-instructions.tex
- lines: 74-75
- severity: minor
- category: fact
- claim: "The register-register form is selected by writing registers for the operands: \mnemonic{ADDR} rather than \mnemonic{ADDI}."
- evidence: The mnemonic selects the format, not the operand spelling. `ADDI r1, r2` is an error ("only supports immediate->register addressing mode", arithmetic_immediate.rs:282-289), and ADDR rejects an immediate.
- resolution: manual-wrong
- fix: "The register format is selected by the mnemonic: write ADDR with register operands, not ADDI."
- confidence: medium

### F-asm-26
- file: assembly/chapters/06-toolchain.tex
- lines: 95-98
- severity: nit
- category: fact
- claim: "a program that needs 64\,K words of scratch memory away from the program binary is run with: ... --segment SCRATCH:00010000:FFFF"
- evidence: A segment covers [base, base+size) (sirc-vm/peripheral-bus/src/lib.rs:45-50), so length 0xFFFF maps 65535 words, one short of 64 K. Word 0x1FFFF stays unmapped.
- resolution: manual-wrong
- fix: Use `SCRATCH:00010000:10000`, or say "just under 64 K words".
- confidence: high

### F-asm-27
- file: assembly/chapters/06-toolchain.tex
- lines: 77-93
- severity: nit
- category: improvement
- claim: "sirc_vm --program-file <FILE> [--segment <SEGMENT>]... [--register-dump-file <FILE>]"
- evidence: The VM also has `-e/--enable-video` (with the `video` feature), which maps VIDEO at 0x0C0000 (sirc-vm/sirc-vm/src/main.rs:114-116,205-222). The fixed segments (PROGRAM, TERMINAL, DEBUG) are not listed anywhere in the assembly manual, so a reader choosing a --segment offset cannot avoid collisions.
- resolution: n/a
- fix: Add `--enable-video, -e` to the option list and a short table of the segments the simulator always maps.
- confidence: high
