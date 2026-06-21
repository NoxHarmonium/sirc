# SIRC-1 Reference Manual Handover

Date: 2026-06-20

This handover captures the work needed to bring the SIRC-1 CPU Reference Manual closer to the standard of a production CPU programmer's reference manual, using the Motorola M68000 Family Programmer's Reference Manual as the comparison point.

The SIRC-1 manual already has a strong base: architecture overview, register model, status register, exception handling, instruction formats, addressing modes, condition codes, instruction summaries, opcode map, timing appendix, and undocumented instruction notes. The next step is to make it more normative and implementation-ready: every encoding, side effect, exception condition, reset state, and reserved behavior should be unambiguous enough that an emulator, assembler, compiler backend, or hardware implementation can be built from the manual alone.

## Priority Overview

### P0: Correctness blockers

These should be fixed before expanding the manual, because they create contradictory architectural contracts.

- Fix the instruction format count. Resolved: the CPU has three instruction formats: Immediate, Short Immediate with Shift, and Register.

- Fix the addressing mode count. Resolved: the CPU has seven addressing modes; operandless meta-instructions assemble to real CPU instructions and are not an addressing mode.

- Resolve carry flag semantics for subtraction. Resolved: subtraction sets C when a borrow occurs; SBC subtracts the incoming C bit as the borrow input.

- Resolve logical instruction flag semantics. Resolved: logical instructions update N and Z and clear C and V.

- Resolve protected-mode privilege semantics for control-flow address-register write-back. Resolved: direct writes to high address registers fault, while address-register-pair write-back preserves the high word and updates only the low word in protected mode.

- Fix the `LJSR` assembler lowering. The CPU write-back path appears to use the instruction destination address-register field for subroutine control-flow targets, and CPU tests construct `LongJumpToSubroutine*` with destination `p`. The toolchain `ljsr` parser currently marks the immediate-form destination field as `0x0`/unused, which likely assembles `LJSR a` to update the wrong address-register pair. Add parser tests that assemble `LJSR a`, `LJSR a, #offset`, and `LJSR a, rN`, then verify they encode destination `p`, source `a`, and preserve the expected link-register behavior.

- Verify all encoding examples.
  - Some examples in `chapters/06-instruction-formats.tex` appear to have inconsistent hex widths for 32-bit instructions.
  - Generate examples from the assembler or a small shared encoder so the manual cannot drift from implementation.

### P1: Make the ISA contract complete

These are the biggest gaps compared to the M68k manual.

- Add a formal instruction-description template and apply it consistently.
- Add legal operand/addressing-mode tables per instruction.
- Add exact flag-effect tables per instruction.
- Add exception/fault behavior per instruction.
- Add reserved, undefined, and implementation-defined behavior rules.
- Add reset-state and startup behavior tables.
- Add vector/link-register saved-state diagrams.

### P2: Make the manual useful to implementers

These are important for emulator, hardware, OS, debugger, and compiler work.

- Add bus timing diagrams and signal sequencing.
- Add data layout rules: vector word order, 32-bit address storage, sign extension, stack layout, and multi-word conventions.
- Add ABI/calling convention guidance or explicitly state that it is outside the ISA.
- Add assembler/object/binary format appendix if SIRC has a canonical ROM or object representation.
- Add revision/model compatibility tables for optional coprocessors.

### P3: Polish and maintainability

- Add list of figures and list of tables.
- Add notation/glossary section.
- Add quick-reference appendices.
- Add manual lint/check scripts.
- Add generated tables where possible.

## Workstream 1: Internal Consistency Pass

Goal: remove contradictions and make the basic architecture summary agree across chapters.

Tasks:

- Audit all top-level summary tables against the detailed chapters:
  - instruction formats
  - addressing modes
  - register count
  - condition-code count
  - coprocessor count
  - vector ranges
  - timing claims

- Normalize terminology:
  - "protected mode" vs "user mode"
  - "supervisor mode" vs "kernel mode"
  - "hardware exception" vs "interrupt"
  - "fault" vs "abort exception"
  - "word address" vs "byte address"
  - "high byte", "high word", and "upper byte" when describing registers

- Make reserved behavior consistent:
  - reserved bits
  - reserved opcodes
  - reserved coprocessor IDs
  - reserved shift type `111`
  - reserved additional flag encodings

Acceptance criteria:

- No chapter gives a different count or meaning for instruction formats, addressing modes, flags, vectors, or privilege modes.
- The intro/spec table matches the detailed chapters.
- Reserved/undefined behavior uses a consistent vocabulary.

## Workstream 2: Machine-Checked Encodings

Goal: ensure every binary and hex encoding in the manual is correct.

Tasks:

- Create a small encoder/verifier for the manual examples.
  - Best option: call into the existing assembler/toolchain tests if feasible.
  - Alternative: create a focused script that encodes examples from a structured source file.

- Replace manually typed encoding examples with generated values where possible.

- Add checks for:
  - 32-bit instruction width
  - opcode values
  - register field values
  - immediate sign/zero representation
  - condition-code field values
  - additional flags
  - shift operand/type/amount fields

- Add a CI or `make check` target for reference-manual validation.

Acceptance criteria:

- Every hex encoding in the manual is either generated or covered by a verification test.
- Invalid example widths, such as 9-hex-digit instruction words, are impossible to reintroduce silently.

## Workstream 3: Instruction Reference Upgrade

Goal: make each instruction description as precise as the M68k instruction reference.

Recommended instruction template:

- Mnemonic and full name
- Availability and privilege level
- Instruction format(s)
- Opcode(s)
- Syntax
- Operand forms
- Legal registers/addressing modes
- Operation pseudocode
- Result write-back behavior
- Flag effects
- Exception/fault conditions
- Conditional execution behavior
- Bus cycles/signals used
- Timing
- Reserved field behavior
- Examples
- Notes and common pitfalls

Tasks:

- Add a "How to Read Instruction Descriptions" section before the instruction reference, similar to the M68k manual's instruction-description format explanation.

- Create per-instruction legality tables:
  - ALU instructions: immediate, short immediate, register forms.
  - Memory instructions: indirect immediate, indirect register, post-increment, pre-decrement.
  - Control flow: branch, long jump, subroutine call, return, effective address load.
  - Coprocessor/meta-instructions: immediate/register forms, privilege rules.

- Complete the control-flow opcode rework.
  - See `control-flow-opcode-rework-handover.md` for the dedicated design handover covering `LDEL`, `LDEA`, `BRAN`, `BRSR`, `LJMP`, `LJSR`, opcode reuse, implementation staging, and remaining implementation choices.

- For each instruction, explicitly document:
  - what happens when the condition code is false
  - whether flags are updated
  - whether memory is read or written
  - whether address registers are modified
  - whether PC/link registers are modified
  - whether the instruction can fault
  - whether partial side effects are possible on faults

- Add exact flag-effect tables.
  - Use symbols such as `0`, `1`, `-`, `*`, or `U`, but define them.
  - Example columns: `Z`, `N`, `C`, `V`, reserved lower bits, privileged bits.

- Add exact exception behavior per instruction.
  - bus fault
  - bus protection fault
  - alignment fault
  - segment overflow fault
  - invalid opcode fault
  - privilege violation fault
  - trace fault

Acceptance criteria:

- A reader can implement each instruction without guessing legal operands, flag effects, exceptions, or side effects.
- The manual distinguishes assembler conveniences from real CPU instructions.

## Workstream 4: Addressing Modes and Operand Legality

Goal: match the M68k manual's useful distinction between effective address forms and where they are legal.

Tasks:

- Add an addressing mode summary matrix.
  - Columns should include syntax, encoding source, effective value/address, whether it reads memory, whether it writes memory, whether it alters an address register, and legal instruction families.

- Add operand category definitions.
  - Suggested categories:
    - register value
    - immediate value
    - memory source
    - memory destination
    - branch target
    - coprocessor operand
    - privileged register
    - alterable destination

- Add "legal mode by instruction family" tables.
  - This is one of the most useful patterns from real CPU manuals.

- Clarify pre-decrement/post-increment units.
  - State whether increments/decrements are in words, bytes, or instruction words.
  - State whether the offset participates before or after the address register update.
  - State whether the high address byte can carry/borrow and when segment-overflow faults occur.

- Clarify PC-relative addressing.
  - Define the PC value used for effective address calculations: instruction address, next word, next instruction, or post-fetch PC.

Acceptance criteria:

- The addressing chapter can be used as a compact legality reference.
- PC-relative, stack, and auto-update modes are unambiguous.

## Workstream 5: Data Representation and Memory Model

Goal: define how values are represented in registers and memory.

Tasks:

- Add a data formats chapter or subsection.
  - 16-bit word
  - signed two's complement word
  - 8-bit high/low byte within a word
  - 24-bit address
  - 32-bit stored address/vector
  - multi-word integer conventions

- Define memory word order for 32-bit quantities.
  - Exception vectors are 32-bit addresses stored as two 16-bit words; specify which word comes first.
  - Define how `ph:pl`, `ah:al`, etc. map to memory if stored manually.

- Define immediate interpretation.
  - Which immediates are sign-extended?
  - Which are zero-extended?
  - Are ALU immediates interpreted differently by arithmetic vs logical instructions?
  - How does 8-bit short immediate behave?

- Define overflow and wraparound.
  - register arithmetic
  - address arithmetic
  - PC increment
  - pre-decrement/post-increment
  - segment crossing

- Define memory ordering.
  - If there are no caches or reordering, say so.
  - If external devices can observe every bus access in program order, say so.

Acceptance criteria:

- Compiler, assembler, emulator, and OS code can agree on the layout of values in memory.
- No one has to infer vector or address word order from examples.

## Workstream 6: Exceptions, Interrupts, and Reset

Goal: make exception handling as implementation-ready as the M68k exception appendix.

Tasks:

- Add an exception quick-reference appendix.
  - vector number
  - vector address
  - priority
  - source
  - maskability
  - retryable/post-instruction/dispatch category
  - saved return address meaning
  - metadata available
  - whether it can occur in protected mode

- Add link-register/saved-state diagrams.
  - Show return address high/low words.
  - Show saved status register.
  - Show fault metadata register fields.
  - Show which exception level uses which banked link register.

- Add reset-state table.
  - all general-purpose registers
  - address register pairs
  - status register
  - exception unit registers
  - pending interrupt state
  - bus pins during and after reset
  - PC load sequence
  - RSTO behavior

- Clarify reset vector fetch.
  - vector word order
  - bus access type
  - behavior if reset vector fetch faults
  - behavior if RSTI is asserted during another exception

- Clarify pending interrupt behavior.
  - disabled interrupts are ignored, not queued
  - lower-priority interrupts while higher-priority handlers run
  - repeated level-triggered interrupt pins
  - NMI/level 5 conflict behavior
  - trace mode interaction

- Define exact exception-entry side effects.
  - when P, T, EA, and interrupt-enable bits change
  - whether condition flags are preserved
  - when link registers are written
  - when PC changes
  - what happens if vector fetch faults

Acceptance criteria:

- A system programmer can write robust exception handlers from the manual alone.
- Reset behavior is deterministic and complete.
- Exception diagrams act as quick references.

## Workstream 7: External Bus and Pin Timing

Goal: make the hardware interface precise enough for board/device designers.

Tasks:

- Add bus timing diagrams.
  - instruction fetch read
  - data read
  - data write
  - exception vector fetch
  - wait-state insertion via BACK
  - BERR and BPER abort timing

- Choose a timing-diagram authoring path.
  - `tikz-timing` looks well suited for waveform diagrams, but it adds a LaTeX package dependency that must be available wherever the PDF is built.
  - Prefer dependencies included in a normal TeX Live/MacTeX install, or document the exact package requirement in the docs build prerequisites.
  - If avoiding extra LaTeX package requirements is more important, generate waveform PDFs/SVGs from a checked-in script and include the rendered assets from LaTeX.
  - Whichever path is chosen, add a tiny build smoke test so missing timing-diagram tooling fails clearly.

- Define signal timing relative to `CLKI`.
  - when A, D, BRW, BAT, PROT, and BAS become valid
  - when data is sampled on reads
  - when data is driven/released on writes
  - setup/hold requirements for BACK, BERR, BPER, IRQ, NMI, HALT, TRCE, and RSTI

- Define bus ownership.
  - when data bus is input, output, or high impedance
  - whether address bus is always driven
  - behavior during reset and halt

- Define interrupt sampling.
  - level-triggered vs edge-triggered
  - sampled at instruction boundary or clock edge
  - minimum assertion duration
  - behavior when pin remains asserted

- Define electrical assumptions or intentionally defer them.
  - voltage levels
  - fanout/current
  - decoupling recommendations
  - maximum clock rate basis
  - if this is out of scope for the ISA manual, split into a datasheet appendix.

Acceptance criteria:

- External memory and peripherals can be designed against the manual without guessing signal order.
- Timing notes are diagrams and tables, not just prose.

## Workstream 8: Coprocessor and Model Compatibility

Goal: clarify optional coprocessors and future CPU variants.

Tasks:

- Add a coprocessor compatibility table.
  - ID
  - name
  - required/optional
  - stable across models
  - exception behavior if absent
  - privilege restrictions
  - instruction/opcode space

- Add model/revision policy.
  - what must remain backward compatible
  - what reserved fields must do
  - what optional features software can probe
  - how future revisions advertise capabilities

- Expand the coprocessor instruction chapter.
  - exact COP operand encoding
  - coprocessor ID field
  - opcode field
  - immediate/register forms
  - invalid coprocessor/opcode behavior
  - supervisor-only operation behavior

Acceptance criteria:

- Software can detect and handle absent optional coprocessors.
- Future revisions have a documented compatibility contract.

## Workstream 9: ABI, Calling Convention, and Software Conventions

Goal: separate hardware facts from software conventions and document the conventions that examples rely on.

Tasks:

- Decide whether the reference manual should include an ABI appendix.
  - If yes, document it.
  - If no, state that register usage conventions are non-normative and belong in a separate ABI/toolchain document.

- If included, define:
  - caller-saved registers
  - callee-saved registers
  - argument passing
  - return values
  - stack growth and alignment
  - stack frame layout
  - interrupt handler save/restore expectations
  - use of link register for nested calls

- Update examples to follow the documented convention.

Acceptance criteria:

- Examples do not imply an undocumented ABI.
- Compiler and assembly programmers have a clear convention to follow, or a clear pointer to a separate document.

## Workstream 10: Binary/Object/Loader Format Appendix

Goal: provide an equivalent to the M68k manual's practical output-format appendix if SIRC has a canonical format.

Tasks:

- Decide what belongs in the CPU reference manual:
  - raw ROM image format
  - assembler object format
  - linker output format
  - debug map format
  - relocation records
  - symbol tables

- If there is a canonical format, document:
  - file layout
  - record types
  - endianness/word order
  - load addresses
  - checksums
  - relocation model
  - examples

- If not canonical, add a short "Program Image Format" appendix that defines the minimum boot ROM/vector layout.

Acceptance criteria:

- A user can understand how assembled code gets into memory and how reset finds the first instruction.

## Workstream 11: Quick Reference Appendices

Goal: make the manual faster to use once the reader already understands the architecture.

Tasks:

- Add instruction summary by mnemonic.
- Add instruction summary by opcode.
- Add instruction summary by functional group.
- Add condition code quick reference.
- Add register encoding quick reference.
- Add addressing mode quick reference.
- Add exception vector quick reference.
- Add status register bit quick reference.
- Add bus access type quick reference.

Acceptance criteria:

- Common lookup tasks are answerable from appendices without reading narrative chapters.

## Workstream 12: Documentation QA and Build Hygiene

Goal: prevent the manual from drifting away from the implementation.

Tasks:

- Add a reference-manual QA target.
  - build PDF
  - verify generated examples
  - check references
  - check glossary/terminology
  - optionally render selected pages for visual inspection

- Generate tables from source where possible.
  - opcode map
  - register map
  - condition codes
  - instruction encodings
  - coprocessor IDs

- Add a "manual source of truth" note.
  - Decide whether implementation code, LaTeX tables, or a shared machine-readable spec is canonical.

- Add TODO tracking.
  - Use explicit `TODO(manual)` comments in LaTeX or track via issues.
  - Avoid prose TODOs in the rendered manual unless intentionally visible.

- Audit assembly examples by chapter.
  - Architecture, reset, bus, exception, and memory-model chapters should prefer architectural pseudocode, diagrams, timing tables, and state-transition descriptions.
  - Assembly listings should be confined mainly to instruction reference, assembler-facing sections, software convention sections, and the examples appendix.
  - Where architecture chapters currently use assembly to illustrate behavior, decide whether to replace it with pseudocode, move it to the relevant instruction chapter, or explicitly mark it as non-normative example code.

Acceptance criteria:

- Manual examples and tables are tested against implementation data.
- Manual build failures catch broken references and stale generated content.
- The manual distinguishes architectural specification from assembler usage examples.

## Suggested Execution Order

1. Fix P0 contradictions and flag semantics.
2. Add machine-checked encoding verification.
3. Define the instruction-description template.
4. Upgrade ALU instructions first, since they drive flag/condition-code correctness.
5. Upgrade memory/control-flow instructions next, since they touch addressing, PC, link register, and bus behavior.
6. Upgrade exceptions/reset, including diagrams and quick-reference tables.
7. Add bus timing diagrams and signal tables.
8. Add data representation and ABI/software convention appendices.
9. Add compatibility, binary/object format, and quick-reference appendices.
10. Add QA automation and generated tables.

## Definition of Done

The manual is "up to scratch" when:

- There are no internal contradictions in architectural facts.
- Every instruction has explicit legal operands, encoding, side effects, flags, exceptions, timing, and privilege behavior.
- Every manually visible encoding example is generated or verified.
- Reset, exception entry, exception return, interrupt priority, and vector layout are fully specified.
- Data representation and memory layout are unambiguous.
- Bus timing is precise enough for external device design.
- Optional coprocessor behavior and compatibility rules are documented.
- Quick-reference appendices cover the common lookup tasks.
- The manual can be built and checked with a repeatable command.
