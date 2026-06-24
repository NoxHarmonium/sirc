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

- Fix the `LJSR` assembler lowering. Resolved: `sirc-vm/toolchain/src/parsers/opcodes/ljsr.rs` now encodes `LJSR`
  normal and post-increment forms with destination address-register `p`. Verified by
  `sirc-vm/toolchain/tests/assembler/control_flow_test.rs`, including `LJSR a`, `LJSR a, #4`, `LJSR a, r3`,
  `LJSR (#0, a)+`, and `LJSR (r3, a)+`. Targeted verification command passed:
  `cargo test -p toolchain --test mod assembler::control_flow_test -- --nocapture`.

- Verify all encoding examples.
  - Some examples in `chapters/07-instruction-formats.tex` appear to have inconsistent hex widths for 32-bit instructions.
  - Generate examples from the assembler or a small shared encoder so the manual cannot drift from implementation.

### P1: Make the ISA contract complete

These are the biggest gaps compared to the M68k manual.

- Apply the existing instruction-description template consistently across all instruction entries.
- Add legal operand/addressing-mode tables per instruction.
- Add exact flag-effect tables per instruction.
- Add exception/fault behavior per instruction.
- Add reserved, undefined, and implementation-defined behavior rules.
- Add reset-state and startup behavior tables.
- Add vector/link-register saved-state diagrams.

### P2: Make the manual useful to implementers

These are important for emulator, hardware, OS, debugger, and compiler work.

- Add bus timing diagrams and signal sequencing.
- Data layout rules. Mostly resolved: Chapter 4 now defines vector word order, stored address layout, immediate
  sign/zero behavior, address wraparound, and memory ordering. Remaining work is to audit older examples for byte-style
  offsets and decide whether to add non-normative software conventions for multi-word integers and stack layout.
  Short immediates are documented as zero-extended ALU operands only; memory, effective-address, branch, jump, and
  subroutine-call displacements use 16-bit or register displacement forms.
- Add ABI/calling convention guidance or explicitly state that it is outside the ISA.
- Add assembler/object/binary format appendix if SIRC has a canonical ROM or object representation.
- Add revision/model compatibility tables for optional coprocessors.

### P3: Polish and maintainability

- Fix PDF layout issues in the instruction summary and instruction reference pages.
  - Section 12.2, "Complete Instruction List", does not fit cleanly on one page. Convert it to a multi-page table
    (`longtable`, `ltablex`, or a generated split table), or split it into separate opcode-family tables if that reads
    better.
  - Some `instructionbox` sections are too tall for the remaining page space, for example "XORI / XORR - XOR
    Immediate / XOR Register", "LOAD - Load/Move", and "CMPI / CMPR - Compare Immediate / Compare Register". This is
    not exhaustive.
  - Review `tcolorbox` layout options such as `breakable` plus a consistent page-start/need-space policy so instruction
    boxes do not appear unpredictably centered or stranded at the bottom of a page.

- Modernize the manual's visual design.
  - Move toward a cleaner modern serif technical-manual style, closer to the original reference manual inspiration and
    other CPU manuals.
  - Revisit the current font package, heading style, color palette, table styling, box styling, and listing styling as a
    coherent design pass rather than isolated tweaks.

- List of figures and list of tables. Resolved: `main.tex` includes both.
- Add notation/glossary section beyond the instruction-notation material already in Chapter 11.
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
  - "meta instruction" vs "alias" vs "convenience"; prefer "meta instruction" consistently for assembler-level
    instructions such as `BRAN`, `BRSR`, `LJMP`, `LJSR`, `SHFT`, `RETS`, `NOOP`, `WAIT`, `RETE`, and `EXCP`
    - Progress: Chapter 15 and the instruction summary now use "meta-instruction" for assembler-level instruction
      forms. Remaining "aliased register writes" wording refers to architectural register write hazards, not
      assembler-level instruction naming.

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
- Undocumented opcodes are documented as valid encodings that execute with architecturally undefined behavior; software
  must not depend on their effects, and future CPU revisions may repurpose them.
- Invalid-opcode faults are scoped to coprocessor dispatch: missing coprocessors and unimplemented/reserved
  coprocessor operations. Normal processing-unit encodings outside the documented spec are reserved or architecturally
  undefined and are not required to trap.

## Workstream 2: Machine-Checked Encodings

Goal: ensure every binary and hex encoding in the manual is correct.

Progress:

- `examples/reference-encodings` assembles the annotated manual encoding examples and generates the instruction-format
  LaTeX tables under `docs/reference/generated/`.
- Decision: do not add Makefile/CI automation for this yet. A prototype stale-generated-table check was intentionally
  removed because it was more machinery than wanted right now. Do not re-add it unless explicitly requested.

Tasks:

- Expand machine checking beyond the generated instruction-format tables by auditing all remaining literal instruction
  encodings in the manual and moving them into generated or verified sources.

- Later, if desired, add or extend checks for:
  - 32-bit instruction width
  - opcode values
  - register field values
  - immediate sign/zero representation
  - condition-code field values
  - additional flags
  - shift operand/type/amount fields

- Later, if desired, add CI coverage for a reference-manual validation command.

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

- "How to Read Instruction Descriptions" chapter. Resolved: Chapter 11 defines the instruction entry fields, notation,
  flag-effect symbols, conditional execution behavior, and status update overrides. Remaining work is to apply the
  template consistently to every instruction entry.
  - Progress: Chapter 13 ALU instruction entries now include per-entry operands, condition-code behavior, timing, and
    privilege fields, alongside existing syntax, operation, flags, write-back, exceptions, examples, and notes. The SBC
    entry also now lists its short-immediate form.
  - Progress: Chapter 14 memory instruction entries now include per-entry operands, operation pseudocode,
    condition-code behavior, and privilege fields for `LOAD` and `STOR`, alongside existing write-back, flags,
    exceptions, timing, and examples.
  - Progress: Chapter 15 control-flow and effective-address entries now include per-entry operands, operation
    pseudocode, write-back, flags, exceptions, condition-code behavior, timing, and privilege fields for `BRAN`, `BRSR`,
    `LDEA`, `LDEL`, `LJMP`, and `LJSR`.
  - Progress: Chapter 16 coprocessor entry now includes per-entry operands, condition-code behavior, and privilege
    fields for `COPI` and `COPR`, alongside existing operation, write-back, flags, exceptions, timing, and examples.
  - Progress: meta-instruction entries now include per-entry operands, write-back, flags, exceptions, condition-code
    behavior, timing, and privilege fields in their relevant instruction-family chapters. The `NOOP` entry remains in
    Chapter 17 and documents the current lowering as `ADDI[N] r1, #0`.

- Create per-instruction legality tables:
  - ALU instructions: immediate, short immediate, register forms.
  - Memory instructions: indirect immediate, indirect register, post-increment, pre-decrement.
  - Control flow: branch, long jump, subroutine call, return, effective address load.
  - Coprocessor/meta-instructions: immediate/register forms, privilege rules.
    - Progress: Chapter 16 now has coprocessor legal forms and common semantics. The spec hides inherited register and
      shift fields from programmers: `COPI` takes an immediate command operand, `COPR` takes a source register operand,
      and hidden encoding fields are reserved/canonicalized by the assembler.
    - Parser follow-up resolved: the toolchain now accepts source-only `COPI #value` and `COPR rS` syntax and rejects
      the old destination-register forms. Opcode 0x2F has no public assembler syntax and should be labelled
      undocumented rather than specified as a programmer-visible form.

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

- Clarify Chapter 17 meta-instruction descriptions.
  - `SHFT` resolved: documented as a meta instruction that lowers to `ORRI[S] rD, #0, shift`, preserving the shifted
    value while taking status flags from the shifter result. It is no longer described as a separate CPU operation.
  - Common meta-instruction semantics are now expressed by the entries in their relevant instruction-family chapters:
    meta-instructions inherit condition-code, timing, flag, fault, and privilege behavior from the emitted instruction
    unless explicitly stated otherwise.
  - Resolved: meta-instructions now live with their relevant instruction families. `SHFT` is documented in the
    ALU chapter; `BRAN`, `BRSR`, `LJMP`, `LJSR`, and `RETS` are documented with control flow; `EXCP`, `WAIT`, `RETE`,
    `RSET`, `ETFR`, and `ETTR` are documented with coprocessor/exception-unit instructions. Chapter 17 is now a short
    meta-instruction cross-reference plus the standalone `NOOP` entry.

- Add exact flag-effect tables.
  - Use symbols such as `0`, `1`, `-`, `*`, or `U`, but define them.
  - Example columns: `Z`, `N`, `C`, `V`, reserved lower bits, privileged bits.
  - Progress: Chapter 13 now has an implementation-backed ALU flag-effect table and explicit status override wording.
    Public `LOAD` forms are documented as preserving flags and not accepting status update override syntax. Clear
    syntax mismatches found during the audit were fixed (`CMPR` for register comparisons, `RTL`/`RTR` shift names, and
    `SHFT` for variable-count pure shifts).
  - Resolved: `RTL`/`RTR` follow the simulator and are documented as normal 16-bit circular rotates. The incoming carry
    flag is not consumed; `C` is an output copied from the bit that wrapped around.

- Resolve per-instruction exception behavior for Chapters 13--17.
  - Resolved: Chapter 13 ALU instructions distinguish instruction-specific faults from global instruction-fetch and
    trace behavior. ALU execution itself has no data-memory, privilege, segment-overflow, or invalid-opcode fault path.
  - Resolved: Chapter 14 memory instructions distinguish data bus, data bus-protection, and `SR.A`-gated
    segment-overflow faults from global fetch/trace behavior. Destination writes and address-register auto-updates occur
    only in write-back after a successful memory-access phase.
  - Resolved: Chapter 15 control-flow/effective-address instructions document `SR.A`-gated segment-overflow faults and
    protected-mode address-register write-back privilege faults, while excluding data bus, data bus-protection, and
    invalid-opcode faults from documented forms.
  - Resolved: Chapter 16 coprocessor calls document privilege faults before dispatch and invalid-opcode faults during
    coprocessor dispatch, while excluding data-memory, bus-protection, alignment, and segment-overflow faults from
    documented `COPI`/`COPR` forms.
  - Resolved: Chapter 17 `NOOP` inherits `ADDI[N]` exception behavior: no data-memory access or instruction-specific
    faults, while global instruction-fetch and trace faults still apply.
  - Remaining exception work has moved to Workstream 6: exception-entry side effects, reset/vector fetch behavior, link
    register diagrams, pending interrupt behavior, and reset-state tables.

Acceptance criteria:

- A reader can implement each instruction without guessing legal operands, flag effects, exceptions, or side effects.
- The manual distinguishes meta-instructions from real CPU instructions.

## Workstream 4: Addressing Modes and Operand Legality

Goal: match the M68k manual's useful distinction between effective address forms and where they are legal.

Progress:

- Added an addressing mode matrix with syntax, effective value/address, memory access behavior, auto-update side effects,
  and legal instruction families.
- Added operand category definitions and common address calculation rules, including word-sized auto-update behavior and
  the PC-relative rule that `p`-relative displacements are relative to the next instruction address after fetch.
- Corrected addressing examples that implied byte offsets in word-addressed memory.
- Renumbered chapter source files after adding the new data-representation chapter so source filenames again match manual
  order: data representation is Chapter 4, status register is Chapter 5, exceptions is Chapter 6, and later chapters are
  shifted accordingly.

Tasks:

- Add "legal mode by instruction family" tables.
  - This is one of the most useful patterns from real CPU manuals.

- Document omitted zero-displacement shorthand for memory indirect forms.
  - Any instruction that accepts memory indirect access with immediate displacement should allow the displacement to be
    omitted when it is zero.
  - For example, `LOAD rD, (a)` must be specified as equivalent to `LOAD rD, (#0, a)`.
  - Audit all affected instructions and parser tests so this shorthand is accepted consistently, not only for the forms
    that currently happen to support it.
  - Cover normal, post-increment, and pre-decrement immediate-displacement forms where applicable, and state clearly
    whether omitted displacement is allowed for each syntax family.

Acceptance criteria:

- The addressing chapter can be used as a compact legality reference.
- PC-relative, stack, and auto-update modes are unambiguous.

## Workstream 5: Data Representation and Memory Model

Goal: define how values are represented in registers and memory.

Progress:

- Added `chapters/04-data-representation.tex` and included it in the main manual. It now defines external word byte
  order, multi-word storage order, address-register-pair interpretation, stored 24-bit address/vector order,
  instruction storage, immediate interpretation, address wraparound, and program-order memory visibility.
- Cross-referenced the new chapter from the introduction, register model, and exception vector table.

Tasks:

- Audit examples in other chapters for byte-oriented offset assumptions, especially stack, structure, vector, and
  multi-word address examples. Prefer word offsets everywhere unless a section is explicitly discussing external byte
  representation.
- Decide whether to add more non-normative software conventions for multi-word integers beyond the architectural
  high-word-first convention already documented.

Acceptance criteria:

- Compiler, assembler, emulator, and OS code can agree on the layout of values in memory.
- No one has to infer vector or address word order from examples.

## Workstream 6: Exceptions, Interrupts, and Reset

Goal: make exception handling as implementation-ready as the M68k exception appendix.

Progress:

- Chapter 6 now documents the privilege-violation cases consistently with the simulator and status-register chapter:
  direct high address-register writes fault, protected address-register-pair write-back faults if it would change a
  high word, supervisor-only coprocessor operations fault in protected mode, and software exception vectors below
  0x60 are privileged. Protected-mode writes to privileged status-register bits are masked rather than faulting.
- Chapter 6 now documents the fault metadata low bits as the captured bus access type (`BAT0`--`BAT2`), not the internal
  CPU execution phase.
- Chapter 6 now clarifies exception vector-table lookup as a high-word then low-word fetch from
  `system_ram_offset + vector * 2`, followed by loading the fetched target address into PC.
- The fault-handler link-register preservation example in Chapter 6 now uses current public memory syntax and restores
  the level 7 metadata and level 6 fault link registers in the correct order.
- Chapter 6 now includes a reset-state table, documents hardware `RSTI` and software `RSET` reset-output hold behavior,
  and defines reset-vector fetch as an `ExceptionVectorFetch` of vector 0x00 high word then low word. Chapter 2 reset
  pin wording now defers to Chapter 6 for the detailed reset sequence.
- Chapter 6 now makes reset a clean exception boundary: pending hardware exceptions and pending faults are cleared by
  reset. External interrupt pins that remain asserted after reset may be sampled again through the normal interrupt
  rules.

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

- Add reset-state table. Progress: Chapter 6 now has a first implementation-backed reset-state table.
  - all general-purpose registers
  - address register pairs
  - status register
  - exception unit registers
  - pending interrupt state
  - bus pins during and after reset
  - PC load sequence
  - RSTO behavior

- Clarify reset vector fetch.
  - vector word order. Progress: Chapter 6 now says reset vector 0x00 is fetched high word first from
    `system_ram_offset + 0x0000`, then low word from `system_ram_offset + 0x0001`.
  - bus access type. Progress: Chapter 6 now says reset-vector reads use `ExceptionVectorFetch`.
  - behavior if reset vector fetch faults. Progress: Chapter 6 now marks this implementation-defined; a future hardware
    decision should make it fully normative.
  - behavior if RSTI is asserted during another exception

- Align simulator reset internals with the reset-state table. Resolved: `CpuPeripheral::reset()` clears
  `pending_fault`, `pending_hardware_exceptions`, and `current_exception_level`, and reset tests cover stale exception
  state being cleared before reset-vector fetch.

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

- Decide final protected-mode status-register write behavior.
  - Current simulator behavior: protected-mode reads of `sr` mask the privileged byte, and protected-mode writes to
    `sr` update only the lower byte while preserving the privileged byte.
  - Open design question: align `sr` with the other privileged registers by raising a privilege violation when a
    protected-mode instruction attempts to write privileged `sr` bits, or make all protected-mode writes to `sr`
    invalid/faulting for simplicity.
  - Consider whether user-mode software has a legitimate need to write the lower status byte directly. ALU and shift
    instructions already update condition flags through normal execution, and allowing direct lower-byte writes gives
    user code explicit control over condition flags and reserved low-byte bits.

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

- Add visual PDF QA for known problem pages.
  - Render and inspect the page containing Section 12.2, "Complete Instruction List".
  - Render instruction reference pages containing tall boxes such as "XORI / XORR", "LOAD - Load/Move", and
    "CMPI / CMPR".
  - Treat overfull tables, clipped boxes, and awkward bottom-of-page instructionbox placement as documentation build
    defects.

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

1. Finish the instruction-description template rollout and close remaining per-instruction legality/flag/exception gaps.
2. Upgrade ALU instructions first, since they drive flag/condition-code correctness.
3. Upgrade memory/control-flow instructions next, since they touch addressing, PC, link register, and bus behavior.
4. Upgrade exceptions/reset, including diagrams and quick-reference tables.
5. Add bus timing diagrams and signal tables.
6. Add ABI/software convention guidance or explicitly move it out of the ISA manual.
7. Add compatibility, binary/object format, and quick-reference appendices.
8. Add QA automation and generated tables when the desired workflow is clear.
9. Return to machine-checked encoding verification only when the lightweight workflow is agreed.

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
