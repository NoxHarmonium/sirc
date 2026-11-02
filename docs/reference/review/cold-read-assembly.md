# Cold read: assembly programmer's manual

Chapter 6 worked example tried for real (tools built with `cargo build --bins` in `sirc-vm`; the commands in 6.1 ran as written once the binaries were on the path, and the dump showed r1 = 0xa). Notes are in F-cra-1 and F-cra-2.

### F-cra-1
- file: assembly/chapters/06-toolchain.tex
- lines: 1-8
- severity: minor
- category: structure
- claim: "Putting cargo run --bin tool -- in front of the arguments builds and runs a tool in one step."
- evidence: none (tried: the bare commands fail until the binaries are on PATH; `cargo run` must be issued from sirc-vm/ while count.sasm is elsewhere, so relative paths break)
- resolution: n/a
- fix: Add a short "Before you start" block: `cd sirc-vm && cargo build --bins`, where the binaries land (`target/debug`), and that they must be on PATH or called by path. Show the three worked-example commands once in the `cargo run --bin assembler --` form, or give the working-directory rule.
- confidence: high

### F-cra-2
- file: assembly/chapters/06-toolchain.tex
- lines: 6.1
- severity: minor
- category: structure
- claim: "The register dump written at the end shows r1 holding 0xA."
- evidence: none (the dump is about 70 lines, with nested exception-unit state; sr is 0x1 and pl 0x20a with no explanation)
- resolution: n/a
- fix: Show the first lines of the dump (the `r1: 0xa,` line) and say which lines to look at. Mention that the linker prints a line `object: count.sasm prev_offset: 0`, so a reader does not think it is an error.
- confidence: high

### F-cra-3
- file: assembly/chapters/02-source-language.tex
- lines: 2.3-2.6
- severity: minor
- category: structure
- claim: "Two directive operands are written without the hash."
- evidence: none
- resolution: n/a
- fix: Chapter 2 introduces `.EQU`, `.ORG` and `.DQ` and uses `@start` and `$NAME` before Chapter 5 defines them. Add forward references ("Section 5.1") at first use, or put a one-page "Prefix sigils" table (`#` number, `@` label reference, `$` constant, `:` label declaration, `.` directive) at the start of Chapter 2. A reader has to collect the four sigils from five sections.
- confidence: high

### F-cra-4
- file: assembly/chapters/02-source-language.tex
- lines: 2.1
- severity: minor
- category: structure
- claim: "A source file is a sequence of instructions, directives and labels separated by whitespace."
- evidence: none
- resolution: n/a
- fix: Read twice. "Separated by whitespace" with "otherwise insignificant" sounds as if two instructions could share a line; then "by convention" one per line. State whether a newline is required, and show a complete small file (the Chapter 6 listing) in Chapter 1 or 2 so the reader sees the whole shape first.
- confidence: medium

### F-cra-5
- file: assembly/chapters/03-operands.tex
- lines: 3.1-3.5
- severity: minor
- category: structure
- claim: "an immediate operand ... is 16 bits wide, and that of a short-immediate (shifted) instruction is 8 bits."
- evidence: none
- resolution: n/a
- fix: Chapter 2 and Chapter 3 use "immediate format" and "short immediate format" before Section 4.4 explains how the assembler chooses between them. Add a forward reference, or move 4.4 ahead of the width statements. The width rule is also split across 2.4, 3.2 and 4.4; gather it into one place.
- confidence: medium

### F-cra-6
- file: assembly/chapters/03-operands.tex
- lines: 3.1
- severity: minor
- category: structure
- claim: "(rO, addr)       Indirect, register offset"
- evidence: none
- resolution: n/a
- fix: The notation `rO`, `rC`, `rS1, rS2{, shift}`, `{...}`, `<SUS>` and `<COND>` is used without a key. Add a short notation key under Table 3.1 (and in 1.3 Conventions) that says braces mean optional and names the operand placeholders.
- confidence: medium

### F-cra-7
- file: assembly/chapters/04-instructions.tex
- lines: 4.1
- severity: minor
- category: structure
- claim: "<MNEMONIC>{[<SUS>]}{|<COND>} <OPERAND>, ..."
- evidence: none
- resolution: n/a
- fix: The syntax line uses `{}` for optional and literal `[` `]` and `|` in the same line, which took two readings. Say that the brackets and bar are literal and the braces are not; give the same in a one-line legend.
- confidence: medium

### F-cra-8
- file: assembly/chapters/04-instructions.tex
- lines: 4.3
- severity: minor
- category: structure
- claim: ">=, <<, >>, <=     Signed comparisons"
- evidence: none
- resolution: n/a
- fix: Table 4.1 lumps four signed conditions into one row and does not say which of `<<`/`>>` is less or greater, nor give the flag test for each. Expand to one row per condition with its meaning (for example "signed less than"), with a pointer to the Condition Codes chapter for the flag expressions. Also HI/LO is described as "lower or same" for LO, which reads as inconsistent with the name.
- confidence: medium

### F-cra-9
- file: assembly/chapters/04-instructions.tex
- lines: 4.2
- severity: minor
- category: structure
- claim: "[N], [A] or [S] after the mnemonic selects what updates the status flags: nothing, the ALU result, or the shifter result."
- evidence: none
- resolution: n/a
- fix: The reader is told the default is [A] "for most" instructions but not which are exceptions. Either list them or say where to find them. Add one example of the effect, such as `ADDI[N]` not setting flags before a following conditional instruction.
- confidence: medium

### F-cra-10
- file: assembly/chapters/05-directives-and-symbols.tex
- lines: 5.2
- severity: minor
- category: structure
- claim: "Despite their names they differ only in where the value is placed within the slot"
- evidence: none
- resolution: n/a
- fix: A reader meets `.DQ @start` in Chapter 2/5 and the .DB/.DW/.DQ table afterwards. The surprise (a 4-byte slot for each) is explained after the first use; mention it at the first `.DQ` or reorder. "Slot" is not a defined term. Also say plainly how to emit a table of bytes or a string, since the directives do not pack; no example of a data table with a label follows.
- confidence: medium

### F-cra-11
- file: assembly/chapters/05-directives-and-symbols.tex
- lines: 5.2-5.4
- severity: minor
- category: structure
- claim: "With .l, .u or .r, the 16-bit value goes in the low word."
- evidence: none
- resolution: n/a
- fix: The suffixes are introduced here in prose before Table 5.3 defines them, and Table 5.2 and 5.3 both describe resolution. Merge into one table (context by default, with each suffix as a column). Also "segment" for the high word is used here with a different meaning from the simulator's `--segment` in Chapter 6 and "memory segment" in the linker text; add a sentence connecting them.
- confidence: medium

### F-cra-12
- file: assembly/chapters/05-directives-and-symbols.tex
- lines: 5.1
- severity: minor
- category: structure
- claim: "It must be even, because instructions and data directives each occupy two words."
- evidence: none
- resolution: n/a
- fix: A first-time reader asks "what is the address space and the word size?" and "where does the program start?". The manual points to the architecture manual for vectors but does not say which `.ORG` value is the reset vector slot (0x0000 only appears in the example) or why 0x0200 is a good code start. Add one sentence and a pointer to the memory map chapter.
- confidence: medium

### F-cra-13
- file: assembly/chapters/06-toolchain.tex
- lines: 6.4
- severity: minor
- category: structure
- claim: "The simulator always maps the segments in Table 6.1."
- evidence: none
- resolution: n/a
- fix: A programmer who wants any output will ask how to print. The TERMINAL segment (0x0A0000) is listed but the manual never says what its registers are or shows a program that writes to it. See the improvement below.
- confidence: high

### F-cra-14
- file: assembly/chapters/06-toolchain.tex
- lines: 6.3
- severity: minor
- category: structure
- claim: "--segment-offset (short form -s) is added to every label's address before it is patched in."
- evidence: none (tried: `--segment-offset 0x10` is rejected as stated, with a clap error)
- resolution: n/a
- fix: When the reader would choose a nonzero offset is not explained. Say what the offset is for with an example, or say that a reader new to the tools should always pass 0.
- confidence: medium

### F-cra-15
- file: assembly/chapters/06-toolchain.tex
- lines: 6.4
- severity: minor
- category: structure
- claim: "sirc_vm --program-file prog.bin --segment SCRATCH:00010000:10000"
- evidence: none (ran this and it worked)
- resolution: n/a
- fix: The text before it says the format is `label:offset:length` with hexadecimal numbers but the example does not show the program using the scratch memory (which address does it see? 0x010000). Add two lines of assembly that store to it, and say the label is only a name.
- confidence: low

### F-cra-16
- file: assembly/chapters/07-diagnostics.tex
- lines: 7.1
- severity: minor
- category: structure
- claim: "The ADDR diagnostic is wrapped in the text Parsing Failure: Base { location: ..., kind: External(\"...\") }"
- evidence: none
- resolution: n/a
- fix: Table 7.1 mixes six error messages with a sidebar about wrapping, and the aliased-write list that follows refers to "a different phrase for each form" with none shown. Show the full output for one second error (the ADDR one) and one aliased-write message verbatim, or drop the claim about phrases.
- confidence: medium

### F-cra-17
- file: assembly/chapters/07-diagnostics.tex
- lines: 7.1
- severity: minor
- category: structure
- claim: "The assembler is to reject any auto-update of p ... It does not yet do so"
- evidence: none
- resolution: n/a
- fix: A normative-sounding "is to" sits next to a statement about current behaviour in a chapter titled Diagnostics, and the same limitation is not in Chapter 8's list. Move to Chapter 8 (limitations) and cross-reference.
- confidence: medium

### F-cra-18
- file: assembly/chapters/08-limitations.tex
- lines: 1-5
- severity: minor
- category: structure
- claim: "Four behaviors are worth knowing about:"
- evidence: none
- resolution: n/a
- fix: This repeats the table above it and 7.1; the COPI hook is the third place it is stated (6.1, 8). The chapter's first paragraph states purpose, but a reader hitting "COPI #0x14FF" in 6.1 is not told it is the only way to stop a program and what a real program should do instead (loop, WAIT?). Say what a program should do at the end.
- confidence: medium

### F-cra-19
- file: assembly/chapters/01-introduction.tex
- lines: 1.2
- severity: minor
- category: structure
- claim: "Chapter 6 describes how to run each tool."
- evidence: none
- resolution: n/a
- fix: The Preface says the manual covers "the assembler, the linker, and the simulator"; the intro never says the simulator is the only target and that no real hardware exists to run on, and does not give a "start here" route (read Chapter 6.1 first, then 2 to 5). Add a three-line reading guide, and a pointer to the Assembly Language Summary the Preface says must be read first, named with its chapter.
- confidence: medium

### F-cra-20
- file: assembly/chapters/06-toolchain.tex
- lines: 6.1
- severity: minor
- category: structure
- claim: "On a conforming CPU, the instruction raises an invalid-opcode fault, because exception coprocessor opcode 0x4 is reserved"
- evidence: none
- resolution: n/a
- fix: A first program is built around a hook that cannot be explained at this point (exception coprocessor, opcode 0x4, `COPI` encoding of 0x14FF). Give the explanation one line of decode (what the 0x14FF fields mean), or say "treat as a magic constant" up front, before the paragraph about conforming CPUs.
- confidence: low

## Improvements

### F-cra-21
- file: assembly/chapters/06-toolchain.tex
- lines: 6.1
- severity: minor
- category: improvement
- claim: "Save this program as count.sasm."
- evidence: none
- resolution: n/a
- fix: Add a second worked example that produces visible output: a "hello" program writing characters to the TERMINAL segment, with the terminal register offsets given (or pointed to), a loop over a data table with `.DQ` entries, and a call and return with BRSR/RETS. The one example uses no data, no memory, no subroutine and no output; those are what a first program needs next.
- confidence: high

### F-cra-22
- file: assembly/chapters/04-instructions.tex
- lines: 4.5
- severity: minor
- category: improvement
- claim: "Table 4.2: Mnemonics Recognized by the Assembler"
- evidence: none
- resolution: n/a
- fix: Add a one-page syntax card as an appendix: every mnemonic with its accepted operand forms in one line each (the manual says to look at the architecture manual per instruction, which means flipping between two PDFs). A reader would keep this beside them.
- confidence: high

### F-cra-23
- file: assembly/chapters/03-operands.tex
- lines: 3.1
- severity: minor
- category: improvement
- claim: "Table 3.1: Operand Forms"
- evidence: none
- resolution: n/a
- fix: Add a table of which instruction accepts which operand forms (label reference allowed, shift allowed, status update source allowed). The facts are scattered across 3.2, 3.4, 4.2 and Table 7.1.
- confidence: medium

### F-cra-24
- file: assembly/chapters/05-directives-and-symbols.tex
- lines: 5.4
- severity: minor
- category: improvement
- claim: "A full 24-bit address is loaded into an address register pair in two halves, as follows:"
- evidence: none
- resolution: n/a
- fix: Add a figure of a 24-bit address split into .u and .l, and of the 4-byte slot layouts for .DB/.DW/.DQ (the byte table is hard to picture). A linker diagram (two objects joined end to end, with `.ORG` relative to each object) would make 5.1 and 6.3 easier.
- confidence: medium

### F-cra-25
- file: assembly/chapters/07-diagnostics.tex
- lines: 7.1
- severity: minor
- category: improvement
- claim: "The diagnostics for the most common mistakes are:"
- evidence: none
- resolution: n/a
- fix: Add the common mistakes a newcomer makes by this manual's own rules: colon after the label name, missing `#`, lower-case mnemonic, `@label` in ADDI, `.ORG #0x200`, `.EQU` used before definition. Show the actual message for each.
- confidence: medium

### F-cra-26
- file: assembly/chapters/06-toolchain.tex
- lines: 6.5
- severity: minor
- category: improvement
- claim: "A Makefile keeps the three steps in sync."
- evidence: none
- resolution: n/a
- fix: The make fragment could be cut (it is the obvious pattern) or replaced by a pointer to a real example in `examples/`, naming the directory. Move the simulator's flag list to a reference table at the end of the chapter, since that is lookup material, not walkthrough.
- confidence: low

### F-cra-27
- file: assembly/chapters/08-limitations.tex
- lines: Table 8.1
- severity: minor
- category: improvement
- claim: "This chapter will shrink as the toolchain grows."
- evidence: none
- resolution: n/a
- fix: A chapter whose purpose is what is missing fits better as the last section of Chapter 1 or as an appendix; as Chapter 8 it comes after diagnostics when a reader first wants to know whether a macro or include exists. Mention the main gaps (no macros, no includes) in the Chapter 1 scope.
- confidence: low

### F-cra-28
- file: assembly/chapters/01-introduction.tex
- lines: Table 1.2
- severity: minor
- category: improvement
- claim: "Table 1.2: The SIRC Toolchain"
- evidence: none
- resolution: n/a
- fix: Add a pipeline figure (source -> assembler -> .o -> linker -> .bin + .dbg -> sirc_vm) with the file names from the worked example, and a numbered list of the options of each tool in one table (the options are given only in prose in 6.2 to 6.4).
- confidence: low

## Summary

Top three: (1) the single worked example in Chapter 6 shows no data, memory, subroutine or output, and needs setup text about building and running the binaries (F-cra-21, F-cra-1); (2) sigils and notation (`#`, `@`, `$`, `:`, `{}`, `rO`) are used in Chapters 2 to 4 before they are defined or keyed (F-cra-3, F-cra-6, F-cra-7); (3) there is no per-instruction operand and syntax card, so the reader flips to the architecture manual for every instruction (F-cra-22).

The manual reads as a finished reference for the lookup it covers, but the first-time path (setup, a second example, a syntax card) is still draft.
