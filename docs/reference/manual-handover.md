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
- Data layout rules. Resolved: Chapter 4 defines vector word order, stored address layout, immediate sign/zero
  behavior, address wraparound, memory ordering, and the boundary between architectural storage order and
  non-normative software conventions. Short immediates are documented as zero-extended ALU operands only; memory,
  effective-address, branch, jump, and subroutine-call displacements use 16-bit or register displacement forms.
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

Status: Complete for the current reference-manual scope.

Progress:

- `examples/reference-encodings` assembles the annotated manual encoding examples and generates the instruction-format
  LaTeX tables under `docs/reference/generated/`.
- Current audit found no hand-written raw 32-bit instruction-word examples outside `docs/reference/generated/`. The only
  eight-digit hex values in chapter prose are target-address comments in the control-flow chapter, not instruction
  encodings.
- `examples/reference-encodings` was verified with `make check`, and `make manual-tex` regenerates the same encoded
  values. Its unformatted output differs from the checked-in fragments only by LaTeX whitespace/column formatting before
  `latexindent` is applied.
- Opcode, condition-code, shift-type, undocumented-opcode, and instruction-family field literals were audited against
  `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs`. The values match the simulator source of
  truth as of this pass.
- Decision: do not add Makefile/CI automation for this yet. A prototype stale-generated-table check was intentionally
  removed because it was more machinery than wanted right now. Do not re-add it unless explicitly requested.

Completed scope:

- The full raw instruction-word examples in Chapter 7 are generated from an assembler fixture rather than maintained by
  hand.
- Remaining manual numeric literals were audited and classified as opcode IDs, field values, vector addresses,
  coprocessor command operands, data constants, or address comments rather than unverified raw instruction-word
  examples.
- Invalid 9-hex-digit instruction-word examples are covered by the audit pattern and were not found.

Future optional QA/tooling, if desired:

- Add or extend generated checks for:
  - 32-bit instruction width
  - opcode values
  - register field values
  - immediate sign/zero representation
  - condition-code field values
  - additional flags
  - shift operand/type/amount fields
- Add CI coverage for a reference-manual validation command once the validation command exists and is worth the
  maintenance cost.

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

Status: Complete.

Progress:

- Added an addressing mode matrix with syntax, effective value/address, memory access behavior, auto-update side effects,
  and legal instruction families.
- Added operand category definitions and common address calculation rules, including word-sized auto-update behavior and
  the PC-relative rule that `p`-relative displacements are relative to the next instruction address after fetch.
- Corrected addressing examples that implied byte offsets in word-addressed memory.
- Renumbered chapter source files after adding the new data-representation chapter so source filenames again match manual
  order: data representation is Chapter 4, status register is Chapter 5, exceptions is Chapter 6, and later chapters are
  shifted accordingly.
- Added a "Legal Modes by Instruction Family" table to Chapter 8 so the addressing chapter can be used as a compact
  legality reference.
- Documented zero-displacement shorthand for immediate-displacement indirect forms:
  - `LOAD rD, (addr)` and `LOAD rD, (addr)+`
  - `STOR (addr), rS` and `STOR -(addr), rS`
  - `LDEA dest, (src)` and `LDEA dest, -(src)`
  - `LDEL dest, (src)` and `LDEL dest, (src)+`
  - `LJSR (src)+`
- Clarified that shorthand does not create otherwise illegal addressing forms, and that non-auto-update `LJMP`/`LJSR`
  use `LJMP src`/`LJSR src` rather than `LJMP (src)`/`LJSR (src)`.
- Added assembler tests in `sirc-vm/toolchain/tests/assembler/addressing_mode_test.rs` for accepted shorthand forms and
  rejected shorthand forms that would otherwise imply illegal addressing families. Verified with
  `cargo test -p toolchain`.

Acceptance criteria:

- The addressing chapter can be used as a compact legality reference.
- PC-relative, stack, and auto-update modes are unambiguous.

## Workstream 5: Data Representation and Memory Model

Goal: define how values are represented in registers and memory.

Status: Complete.

Progress:

- Added `chapters/04-data-representation.tex` and included it in the main manual. It now defines external word byte
  order, multi-word storage order, address-register-pair interpretation, stored 24-bit address/vector order,
  instruction storage, immediate interpretation, address wraparound, and program-order memory visibility.
- Cross-referenced the new chapter from the introduction, register model, and exception vector table.
- Audited older examples for byte-oriented offset assumptions. Stack, structure, vector, saved-address, control-flow,
  and memory examples now either use word offsets or explicitly discuss external byte representation.
- Clarified the introduction's memory map so vector IDs are not mistaken for word addresses. The table now gives each
  vector range and its corresponding word-address range.
- Added a short software-layout-conventions section. The ISA defines high-word-first architectural storage and word
  offsets, while full ABI, stack-frame, argument-passing, and software integer conventions remain non-normative unless a
  separate ABI or toolchain document defines them.

Tasks:

- Audit examples in other chapters for byte-oriented offset assumptions. Resolved.
- Decide whether to add more non-normative software conventions for multi-word integers. Resolved for the ISA manual:
  full ABI and software-integer conventions are deferred to Workstream 9 or a separate ABI/toolchain document.

Acceptance criteria:

- Compiler, assembler, emulator, and OS code can agree on the layout of values in memory.
- No one has to infer vector or address word order from examples.

## Workstream 6: Exceptions, Interrupts, and Reset

Goal: make exception handling as implementation-ready as the M68k exception appendix.

Progress:

- Chapter 6 now documents the privilege-violation cases consistently with the simulator and status-register chapter:
  direct high address-register writes fault, protected address-register-pair write-back faults if it would change a
  high word, supervisor-only coprocessor operations fault in protected mode, and software exception vectors below
  0x60 are privileged. Protected-mode direct writes to the status register fault.
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

- Add an exception quick-reference appendix/table. Resolved: Chapter 6 now includes an exception quick-reference table
  covering vector number, vector-table address, priority, source, maskability, saved return-address meaning, metadata,
  and protected-mode availability.

- Add link-register/saved-state diagrams. Resolved: Chapter 6 now includes a saved-state bitfield for exception link
  registers, a link-register assignment table for software/hardware/fault levels, and a fault metadata bitfield.

- Add reset-state table. Resolved: Chapter 6 now has an implementation-backed reset-state table covering general
  registers, address register pairs, status register, exception unit state, pending interrupt/fault state, reset-output
  hold behavior, and PC load sequence.

- Clarify reset vector fetch. Resolved: Chapter 6 now says reset vector 0x00 is fetched high word first from
  `system_ram_offset + 0x0000`, then low word from `system_ram_offset + 0x0001`, using bus access type
  `ExceptionVectorFetch`. If reset-vector fetch gets a bus/protection fault, the CPU raises the corresponding fault
  using the normal exception-vector fetch fault path rather than entering a separate reset-failed state.

- Align simulator reset internals with the reset-state table. Resolved: `CpuPeripheral::reset()` clears
  `pending_fault`, `pending_hardware_exceptions`, and `current_exception_level`, and reset tests cover stale exception
  state being cleared before reset-vector fetch.

- Clarify pending interrupt behavior. Resolved: Chapter 6 now documents enabled-line latching, disabled-line ignoring,
  repeated level-sensitive pin coalescing, lower-priority pending interrupts, level 5 conflict behavior, and trace/fault
  priority over hardware interrupts.

- Define exact exception-entry side effects. Resolved: Chapter 6 now states that entry commits after vector-target fetch
  and acceptance, writes the selected link register first, clears `SR.P` and `SR.T`, sets `SR.EA`, preserves condition
  flags, interrupt-enable bits, and `SR.A`, updates current exception level, and loads PC from the fetched vector
  target. Chapter 6 now defines vector-fetch bus/protection fault behavior, and simulator tests cover normal
  exception-vector, reset-vector, fault-vector, and double-fault-vector fetch failures.

- Decide whether lower/equal-priority software exceptions should be screened before vector fetch. Resolved: software
  exceptions are accepted only from normal execution. If an `EXCP` executes while an exception/fault handler is active,
  it is ignored, not queued, and no vector fetch occurs.

- Decide final protected-mode status-register write behavior. Resolved: protected-mode reads of `sr` still mask the
  privileged byte, but any direct protected-mode write to `sr` raises a privilege violation fault. ALU and shift
  instructions may still update condition flags as normal instruction side effects.

- Define exception-vector fetch fault behavior.
  - Resolved: if exception-vector fetch fails before entry commits, the original exception does not enter; raise a
    bus/protection fault with `BAT = ExceptionVectorFetch`.
  - Resolved: if a fault-vector fetch fails, escalate to double fault.
  - Resolved: if the double-fault vector fetch fails, request reset.
  - Resolved: reset-vector fetch failures use the same bus/protection fault path; there is no special reset-failed
    architectural state.
  - Simulator coverage: `peripheral_cpu` exception tests cover software-exception vector fetch bus faults,
    reset-vector fetch bus faults, fault-vector fetch escalation to double fault, and double-fault-vector fetch reset
    requests.

Acceptance criteria:

- A system programmer can write robust exception handlers from the manual alone.
- Reset behavior is deterministic and complete.
- Exception diagrams act as quick references.

## Workstream 7: External Bus and Pin Timing

Goal: make the hardware interface precise enough for board/device designers.

Status: Complete.

Progress:

- Chapter 2 now defines cycle-level logical bus timing rules for A, D, BRW, BAT, PROT, BAS, SYNC, BACK, BERR, BPER,
  IRQ, NMI, HALT, TRCE, and RSTI. The timing tables specify stable bus outputs during wait states, read/write data-bus
  ownership, bus response priority, and boundary-sampled versus asynchronous inputs.
- Chapter 2 now aligns bus-facing pin polarity with an MC68000-style convention: address/data and value pins use
  positive logic, BAS and bus response pins are active low, BRW is read-high/write-low, and reset/interrupt/halt/trace
  controls are active low.
- Signal audit complete: Chapter 2, Chapter 6, Appendix B, and simulator-facing bus comments now use asserted/deasserted
  wording consistently for active-low pins, and SYNC is described as an instruction-start sync signal rather than an
  instruction-complete strobe.
- Chapter 2 now includes `tikz-timing` waveform diagrams for the two-word instruction fetch sequence, data read, data
  write, exception vector fetch, wait-state insertion, BERR/BPER abort timing, a complete register-only ALU instruction,
  a complete LOAD instruction, hardware reset through reset-vector fetch, and an enabled IRQ sampled at an instruction
  boundary. The normal `make all` docs build now acts as the package-availability smoke test for `tikz-timing`.
- The timing appendix now points back to Chapter 2 for bus signal timing and clarifies that instruction fetch, data
  memory access, and exception vector fetch can all add wait states.

Completed scope:

- Bus timing diagrams are resolved in Chapter 2 with `tikz-timing` figures for:
  - two-word instruction fetch
  - data read
  - data write
  - exception vector fetch
  - wait-state insertion via BACK
  - BERR and BPER abort timing
  - complete register-only ALU instruction
  - complete LOAD instruction
  - hardware reset through reset-vector fetch
  - enabled IRQ sampled at an instruction boundary

- Timing-diagram authoring path is resolved: use `tikz-timing` directly in LaTeX. The package is documented in
  `docs/reference/README.md`, and a missing package fails clearly during the normal PDF build.

- Signal timing relative to `CLKI` is resolved at the architectural level:
  - when A, D, BRW, BAT, PROT, and BAS become valid
  - when data is sampled on reads
  - when data is driven/released on writes
  - how BACK, BERR, BPER, IRQ, NMI, HALT, TRCE, and RSTI are sampled
  - Exact electrical setup/hold and propagation numbers remain out of scope for the current reference manual and should
    be handled in a future datasheet/hardware implementation note.

- Bus ownership is resolved:
  - when data bus is input, output, or high impedance
  - when address/control outputs are valid
  - behavior during reset and halt
  - Chapter 2 states that external devices must qualify address/data/control pins with BAS, defines data-bus ownership
    for read, write, wait-state, and no-bus cycles, and diagrams the common bus-cycle waveforms.

- Interrupt sampling is resolved:
  - level-triggered vs edge-triggered
  - sampled at instruction boundary or clock edge
  - minimum assertion duration
  - behavior when pin remains asserted
  - Chapter 2 matches Chapter 6 by documenting IRQ1--IRQ4 and NMI as level-sensitive, instruction-boundary sampled
    inputs, with an enabled IRQ timing example.

- Electrical assumptions are intentionally deferred:
  - voltage levels
  - fanout/current
  - decoupling recommendations
  - maximum clock rate basis
  - Chapter 2 explicitly defers setup/hold, propagation delay, voltage, fanout, and board-loading requirements to a
    future datasheet or hardware implementation note.

Acceptance criteria:

- Met: External memory and peripherals can be designed against the manual without guessing signal order at the
  architectural timing level.
- Met: Timing notes are diagrams and tables, not just prose.

## Workstream 8: Coprocessor and Model Compatibility

Goal: clarify optional coprocessors and future CPU variants.

Status: In progress. Core manual/spec updates and assembler lowering are implemented; reference maths emulation examples
remain.

Progress:

- Chapter 16 now has a standard coprocessor ID compatibility table and model/revision policy.
- Chapter 16 now specifies the optional DMA unit (coprocessor 0x2), including `DMAR`, `DMAW`, and `DMAT` syntax,
  encodings, implicit registers, clobbers, count rules, fault behavior, overlap behavior, and privilege rules.
- Chapter 16 now specifies the optional integer maths unit (coprocessor 0x3), including `MULU`, `MULS`, `DIVU`, and
  `DIVS` syntax, encodings, implicit registers, result/status behavior, and the software-emulation compatibility story.
- The assembler now parses the DMA and maths meta-instructions and lowers them to canonical `COPI #command` forms.
  Toolchain coverage lives in `sirc-vm/toolchain/tests/assembler/coprocessor_test.rs`.
- The instruction summary, addressing-mode legality table, meta-instruction cross-reference, introduction coprocessor
  table, and undocumented-opcode expansion notes now point to the standard coprocessor forms.

Design decisions to carry forward:

- Keep the standard coprocessor command layout: bits 15--12 select the coprocessor ID, bits 11--8 select the operation,
  and bits 7--0 are the operation operand. The public assembler surface should use meta-instructions for standard
  coprocessor services where possible, while `COPI`/`COPR` remain the raw command forms.
- Coprocessor 0x2 is the optional standard DMA unit. It should be useful but simple: no autonomous descriptor engine,
  no hidden DMA registers, and no architectural chunk/prefetch semantics beyond ordered word transfers. Smart memory
  controllers may optimize the existing DMA read/write burst bus access types, but software observes the simple word
  transfer contract.
- Define three DMA meta-instructions:
  - `DMAR #n` reads sequential words from memory at address register `a` into `r1` through `rn`. It is limited to at
    most 7 words because the general-purpose register window is `r1`--`r7`.
  - `DMAW #n` writes sequential words from `r1` through `rn` to memory at address register `a`. It is limited to at
    most 7 words.
  - `DMAT #n` copies sequential words from memory at address register `a` to memory at address register `l`, using
    `r1`--`r7` as a transfer window internally. It transfers up to one 8-bit count operand per command; larger copies
    are done by looping in software.
- DMA command encodings:
  - `DMAR #n` -> `COPI #0x2800 | n`
  - `DMAW #n` -> `COPI #0x2900 | n`
  - `DMAT #n` -> `COPI #0x2A00 | n`
  These operation nibbles are supervisor-only by the existing coprocessor privilege rule.
- DMA register convention:
  - `a` is the memory pointer for `DMAR` and `DMAW`.
  - `a` is the source pointer for `DMAT`.
  - `l` is the destination pointer for `DMAT`.
  - `r1`--`r7` are the transfer window; `DMAT` clobbers them.
  - Successful operations advance the relevant address-register low words by the number of words transferred.
- DMA fault policy: data bus and bus-protection failures raise normal faults with DMA read/write `BAT` values. If an
  address-register low word overflows or underflows and `SR.A` is set, DMA raises a segment-overflow fault; otherwise
  the low word wraps. Partial side effects are possible for a fault that occurs mid-transfer, and DMA operations are not
  guaranteed to be restartable by simply re-executing the same instruction.
- DMA overlap policy: overlapping `DMAT` source/destination ranges are architecturally undefined unless source and
  destination are identical.
- DMA count encoding decision: operand value `0x00` means no-op. `DMAR` and `DMAW` accept counts 0--7, where counts
  above 7 are invalid DMA operations. `DMAT` accepts counts 0--255, with larger transfers done by software loops.
- Coprocessor 0x3 is the optional standard integer maths unit. It should provide a stable software-visible convention
  so missing hardware can be emulated through invalid-opcode faults and later CPU models can accelerate the same
  binaries.
- Define maths meta-instructions initially for integer multiply/divide only:
  - `MULU`: unsigned `r1 * r2 -> r1:r2`, high word in `r1`, low word in `r2`.
  - `MULS`: signed `r1 * r2 -> r1:r2`, high word in `r1`, low word in `r2`.
  - `DIVU`: unsigned `r1:r2 / r3 -> r1` remainder, `r2` quotient, `r3` status.
  - `DIVS`: signed `r1:r2 / r3 -> r1` remainder, `r2` quotient, `r3` status.
- Maths command encodings:
  - `MULU` -> `COPI #0x3000`
  - `MULS` -> `COPI #0x3100`
  - `DIVU` -> `COPI #0x3200`
  - `DIVS` -> `COPI #0x3300`
  These operation nibbles are user-callable by the existing coprocessor privilege rule.
- Maths status convention: `r3` is written as a status word. Bit 0 means any maths error, bit 1 means divide by zero,
  bit 2 means quotient overflow, and bits 3--15 are reserved/written as zero. On successful divide, `r1` receives the
  remainder, `r2` receives the quotient, and `r3` is zero. On divide-by-zero or quotient overflow, set the relevant
  status bits and leave `r1:r2` unchanged. Signed division uses two's-complement arithmetic, quotient truncated toward
  zero, and remainder with the sign of the dividend; `-2147483648 / -1` is quotient overflow.
- Reference maths emulation routines should live in an examples directory and be testable there. The rendered manual
  should document the ABI and may include compact pseudocode or short excerpts, but should not carry long hand-maintained
  multiplication/division listings.

Tasks:

- Add a coprocessor compatibility table. Resolved in Chapter 16.
  - ID
  - name
  - required/optional
  - stable across models
  - exception behavior if absent
  - privilege restrictions
  - instruction/opcode space

- Add model/revision policy. Resolved in Chapter 16.
  - what must remain backward compatible
  - what reserved fields must do
  - what optional features software can probe
  - how future revisions advertise capabilities

- Expand the coprocessor instruction chapter. Resolved for the current standard coprocessor scope.
  - exact COP operand encoding
  - coprocessor ID field
  - opcode field
  - immediate/register forms
  - invalid coprocessor/opcode behavior
  - supervisor-only operation behavior

- Add the DMA unit specification. Resolved in Chapter 16 and assembler lowering.
  - ID 0x2, optional standard coprocessor
  - `DMAR`, `DMAW`, and `DMAT` meta-instructions and encodings
  - implicit register conventions
  - legal counts and `0x00` no-op behavior
  - address-register update rules
  - `r1`--`r7` clobber behavior
  - bus access type usage and fault side effects
  - source/destination overlap behavior

- Add the integer maths unit specification. Resolved in Chapter 16 and assembler lowering.
  - ID 0x3, optional standard coprocessor
  - `MULU`, `MULS`, `DIVU`, and `DIVS` meta-instructions and encodings
  - implicit register conventions
  - result and status behavior
  - divide-by-zero and quotient-overflow handling
  - software-emulation compatibility story for missing hardware

- Add reference maths emulation examples. Pending.
  - Put full assembly routines in `examples/` rather than only in LaTeX.
  - Make the examples assemble and, if practical, add tests around the expected register/status results.
  - Document in the manual that these are reference software routines, not additional ISA requirements.

Acceptance criteria:

- Software can detect and handle absent optional coprocessors.
- Future revisions have a documented compatibility contract.
- DMA programmers can use block register/memory transfers without guessing implicit registers, clobbers, counts, fault
  behavior, or bus-visible transfer type.
- Maths code can call standard multiply/divide operations and rely on the same ABI whether the operation is handled by
  hardware or emulated after an invalid-opcode fault.

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
