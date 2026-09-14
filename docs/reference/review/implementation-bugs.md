# Implementation bugs and gaps found by the manual editorial pass

Out of scope for the manual pass. Each item is a place where the author ruled that the manual is
right and the Rust is wrong or incomplete. Each entry is self-contained (claim, evidence with exact
file/line citations, and fix); the finding IDs (e.g. `F-data-1`) are carried over from the 2026-09
editorial pass's triage for cross-referencing this document's revision history, not as pointers
into a still-existing file. Cleaned up 2026-09-13: the previous version of this file had ~10
entries duplicated 4-5 times each by a merge script, several near-duplicate findings describing the
same root cause at different manual locations, and a stale "still open" copy of an issue that a
later entry already recorded as fixed. No technical content was changed in this pass, only
deduplication, merging of same-cause findings, and reorganization into Open / Already fixed /
Superseded.

## Open — needs a Rust (or generator) fix

### F-enc-18 (code-wrong, blocker) — ASR does not sign-fill for shift counts above 1

- category: fact
- claim: manual states (correctly, and unchanged by this pass) "Bit 15 (sign bit) is replicated
  into all bits vacated by the shift" and gives worked examples of `ASR #2`, `ASR #3`, `ASR #8` as
  signed division.
- evidence, verified 2026-09-08 by running the actual code (`perform_arithmetic_right_shift`,
  `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:446-476`) with a standalone
  reproduction: `wide_result = extended_a.rotate_right(clamped_b) | sign_bit`. For a shift count `n`
  in `1..=16`, this is exactly a plain zero-filled logical right shift with bit 15 alone forced to
  the *original* sign -- bits 14 down to `16-n` are **zero**, not sign-extended, for any `n > 1`. At
  `n = 1` there is only one vacated bit (bit 15), so the result happens to be correct -- this is why
  a single-bit `ASR #1` (the only form used in the chapter 9 and appendix D examples) looks right on
  inspection. Confirmed against the existing (passing) test
  `arithmetic_register_test.rs::test_arithmetic_shift_right_register`, which asserts the buggy value
  as expected output: input `0xCCCD` (signed -13107), `ASR #6` gives `0x8333`. A true arithmetic
  right shift (signed divide by 64, rounding toward negative infinity: `floor(-13107/64) = -205`) is
  `0xFF33`. The test locks in the bug rather than catching it.
- resolution: code-wrong.
- fix: rewrite `perform_arithmetic_right_shift` to compute a genuine arithmetic shift, for example
  by sign-extending to a signed 32-bit value before shifting: `((i32::from(a as i16)) >>
  clamped_b.min(16)) as u16` (using Rust's native arithmetic right shift on a signed type, which
  correctly saturates to all-1s or all-0s once the shift reaches the value's width). This also fixes
  the register-count-\ge{}16 case (see `F-alu-8`/`F-enc-21` below) for free: shifting a
  sign-extended value by 16 or more naturally gives `0xFFFF` (negative) or `0x0000` (positive),
  matching true arithmetic-shift semantics, without needing a separate special case.
- confidence: high
- status: code-wrong
- ruling: Gate 2 F, unchanged: ASR must sign-fill every vacated bit. Manual text already states this
  correctly and needs no further change (its worked examples were already right).

### F-data-1 (code-wrong, blocker) — HI/LO condition predicates use the wrong carry polarity

- category: fact
- claim (as originally documented): "HI (Unsigned Higher) Executes when C = 1 AND Z = 0. Tests if
  first operand $>$ second operand (unsigned)."
- evidence: `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:195-206`
  (`perform_subtract` sets C from `a.overflowing_sub(b)`, i.e. C = borrow = `a < b` unsigned,
  confirmed by `tests/instructions/arithmetic_immediate_test.rs:260-272`);
  `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:128-136`
  (`UnsignedHigher = Carry AND NOT Zero`, `UnsignedLowerOrSame = NOT Carry OR Zero` -- the file
  comment at line 114 says this predicate set "copied from ARM," which defines C after subtraction
  as *no borrow*, the opposite polarity).
- resolution: **code-wrong** -- the manual's original ARM-style glosses for `HI`/`LO`/`CS`/`CC` were
  correct in *intent*; only two lines of `definitions.rs` need to change to make the hardware
  actually deliver that intent, given this CPU's real (and correct, and intentional) borrow-on-subtract
  convention.
- fix (author-confirmed, 2026-09-08): change only the `Carry` term in the two compound predicates;
  leave `Zero`, and leave `CarrySet`/`CarryClear`, untouched:

  ```rust
  // definitions.rs, ConditionFlags::should_execute
  Self::UnsignedHigher => {
      !sr_bit_is_set(StatusRegisterFields::Carry, registers)   // was: sr_bit_is_set(...)
          && !sr_bit_is_set(StatusRegisterFields::Zero, registers)
  }
  Self::UnsignedLowerOrSame => {
      sr_bit_is_set(StatusRegisterFields::Carry, registers)     // was: !sr_bit_is_set(...)
          || sr_bit_is_set(StatusRegisterFields::Zero, registers)
  }
  ```

  Do **not** touch `CarrySet`/`CarryClear` (lines 122-123) or the signed predicates
  `GreaterOrEqual`/`LessThan`/`GreaterThan`/`LessThanOrEqual` (they key off Negative/Overflow only,
  never Carry, and are unaffected by this bug).

  Resulting truth table, all four relations now distinct:

  | Mnemonic | Test | Meaning |
  |---|---|---|
  | `CS` | C=1 | unsigned lower (`a < b`) -- matches x86's borrow-flag convention, not ARM's |
  | `CC` | C=0 | unsigned higher or same (`a >= b`) |
  | `HI` | C=0 AND Z=0 | unsigned higher, strict (`a > b`) |
  | `LO` | C=1 OR Z=1 | unsigned lower or same (`a <= b`) |

  This restores `HI` and `LO` to meaning exactly what their names say (matching the manual's
  original glosses without changing manual wording for those two). `CS` and `CC` keep their
  existing single-flag test unchanged, but their *English gloss* is the opposite of ARM's `CS`/`CC`
  (ARM: `CS` = higher-or-same; here: `CS` = lower) -- the manual has been corrected to state this
  chip-specific meaning (see `chapters/10-condition-codes.tex`, `13-alu-instructions.tex`,
  `appendix-b-timing.tex`, and `05-status-register.tex`).

  After the code change, find and update every Rust test that currently asserts the old `HI`/`LO`
  truth values (search the test suite for `UnsignedHigher`, `UnsignedLowerOrSame`, and any
  `should_execute`/condition-flag test using `HI`/`LO` in its name or assembly fixture), and
  re-check any example program using `BRAN|HI`, `BRAN|LO`, or an `|HI`/`|LO` conditional suffix
  against the corrected behavior.
- confidence: high
- status: code-wrong
- ruling: Gate 2 B, revised 2026-09-08 by the author: the manual's ARM-style `HI`/`LO` glosses stand
  unchanged; `CS`/`CC` glosses are corrected to their real (non-ARM) meaning;
  `definitions.rs`'s `UnsignedHigher`/`UnsignedLowerOrSame` predicates are to be fixed as shown
  above. The author will apply the code and test changes separately; this pass updated only the
  manual.
- affected manual locations, corrected 2026-09-08: `chapters/10-condition-codes.tex` (main encoding
  table, CS/CC/HI/LO glosses, the CS/CC-equivalence note, the unsigned truth table, and the
  selection guide -- the CS and CC rows swap which relationship they describe; HI and LO keep their
  English meaning and only their flag-value column changes) and `chapters/07-instruction-formats.tex`
  (a duplicate copy of the same encoding table, same fix). Checked and found already correct as
  written, no change needed: `chapters/13-alu-instructions.tex` (its one worked example already uses
  `HI` with the correct "greater than" meaning), `chapters/05-status-register.tex` (its Carry-flag
  description never commits to a direction), `chapters/09-shift-operations.tex` (its `CS` usage
  tests a shift-result carry, not a subtraction borrow, so the polarity question does not apply),
  and `chapters/appendix-b-timing.tex` (see `F-tim-1`, superseded below -- both its `HI`/`LO` timing
  example and its `BRAN|HI` loop are correct once the code fix lands).

### F-exc-3 (code-wrong, blocker) — REVISED 2026-09-12, retryable fault set now unified

- category: fact
- claim: chapter 6 previously said (per the superseded Gate 2 A ruling) that only Alignment and
  PC-wrap Segment Overflow are retryable. A 2026-09-08 revision expanded that to Alignment, Bus, Bus
  Protection, Privilege Violation, and PC-wrap Segment Overflow, while keeping Invalid Opcode Fault
  and non-PC-wrap Segment Overflow non-retryable. The author has now (2026-09-12) simplified the
  design further: every fault in this category is retryable, with no exceptions. The "Aborted,
  non-retryable faults" category has been removed from chapter 6 entirely and its two members folded
  into "Retryable faults."
- evidence: `exception_unit/definitions.rs:121-131`, the CPU's own design comment: "Abort Exception
  means that the instruction does not have any effect... The program address stored in the link
  register is the address of the faulting instruction so it can be retried... This is important for
  things like privilege violation because you don't want the illegal instruction to do anything,"
  explicitly listing Bus Fault, Alignment Fault, Privilege Violation, and Invalid Opcode Fault (plus
  Reset) in the intended abort/retryable set -- this comment was accurate all along for Invalid
  Opcode Fault; only the code hadn't caught up to it.
- root cause, traced 2026-09-08: `processing_unit/execution.rs:140-151`. The privilege check runs
  and calls `raise_fault` (marking a fault pending), but the very next line,
  `registers.pl = self.decoded_instruction.npc_l_;`, runs unconditionally in the same step,
  advancing the program counter past the faulting instruction regardless of whether a fault was just
  raised. `raise_fault` itself only marks the fault pending; it does not capture a return address. By
  the time a later pipeline stage reads `registers.pl` to populate the fault link register, `pl` has
  already moved on, so the saved address is the *next* instruction's, not the faulting one. Bus and
  Bus Protection faults are detected even later (at the bus-response stage, after decode has already
  run), so they inherit the same problem. Only Alignment (detected before decode, at fetch) and
  PC-wrap Segment Overflow avoid it, which is why those two were the only ones observed to work
  correctly. Invalid Opcode Fault has a different root cause: it is raised a full poll cycle after
  the invalid `pending_coprocessor_command` was written (`lib.rs:375-391`), by which point `pl` has
  already advanced past that instruction during its own decode step, so the fix here needs its own
  capture point rather than reusing the decode-time-fault fix below. The non-PC-wrap Segment Overflow
  case (`execution_effective_address.rs`) is detected in the same effective-address stage as the
  decode-time faults and can likely reuse that fix.
- resolution: code-wrong.
- fix: when a decode-time fault (Privilege Violation) is raised, do not advance `registers.pl` to
  `npc_l_` in that same step -- leave it at the faulting instruction's address so the later
  fault-dispatch stage captures the correct return address. The same principle applies to Bus Fault,
  Bus Protection Fault, and non-PC-wrap Segment Overflow at whichever stage each is detected: capture
  `pl` as it stood before the faulting instruction, not after. Invalid Opcode Fault needs a separate
  fix, since its fault is raised on the poll cycle *after* the triggering instruction already
  completed its own decode step and advanced `pl`: capture and carry forward the faulting
  instruction's address (e.g. from `pending_coprocessor_command`'s originating instruction) rather
  than reading `pl` fresh at dispatch time.
- confidence: high
- status: manual updated (`chapters/06-exceptions.tex`, `chapters/14-memory-instructions.tex`,
  `chapters/appendix-b-timing.tex`) to the author's final, unified retryable set: Alignment, Bus, Bus
  Protection, Invalid Opcode Fault, Privilege Violation, and Segment Overflow (both the
  program-counter-wraparound and address-computation cases) are all retryable; no fault in this
  chapter is documented as non-retryable any more. Code fix (above) left for the author. Once it
  lands, `examples/faults/faults.sasm`'s `segment_overflow_fault_handler` and
  `invalid_opcode_fault_handler` will need an explicit `ETFR`/`ETTR` return-address redirect in the
  branches that currently rely on the old non-retryable auto-advance (today they either do nothing
  after the fault, or only redirect in the PC-wrap branch) -- without that redirect they will
  re-fault on the same instruction indefinitely once retry becomes the default everywhere.

### F-exc-7 / F-sum-7 / F-cop-9 (code-wrong, major) — EXCP below vector 0x60 panics instead of raising a fault

- category: contradiction
- claim: three manual locations describe EXCP with a reserved vector (below 0x60) as a controlled
  fault condition:
  - `chapters/06-exceptions.tex:93-99`, item 5, lists "Triggering a software exception with a vector
    below 0x60" as a Privilege Violation Fault cause.
  - `chapters/16-coprocessor-instructions.tex:201-202` hedges: "Triggers a software exception at the
    specified user vector, normally 0x60--0xFF."
  - `chapters/appendix-d-examples.tex:39-42`'s included listing
    (`examples/software-exception/software-exception.sasm:18`) itself executes `EXCP #0x40`, a
    reserved vector.
- evidence: `sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:330-333` `assert!`s
  (panics) when a SoftwareException opcode carries a vector below `USER_EXCEPTION_VECTOR_START`
  (0x60) -- it does not raise a fault at all. `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:46-48`
  decides privilege purely from the COP opcode nibble, so `EXCP` (opcode 0x1) can never be treated
  as privileged. `sirc-vm/toolchain/src/parsers/opcodes/exception.rs:73` masks the vector to 8 bits
  with no range check, so the assembler happily emits `EXCP #0x40` (as the appendix D example does).
  Against all of this, `exception_unit/definitions.rs:51-55` documents it as a privilege violation.
  No test covers any of this. facts.md Open question 4.
- resolution: code-wrong
- fix: add a real range check in the exception unit -- comparing the vector *number*, not the
  doubled vector *address* -- that raises Privilege Violation Fault (matching the `definitions.rs`
  doc comment) instead of panicking when a software exception's vector is below 0x60. Once that
  check exists, move `appendix-d-examples.tex`'s example off vector 0x40 to vector 0x60 (and its
  vector-table `.ORG` from 0x0080 to 0x00C0), and replace chapter 16's hedged "normally 0x60--0xFF"
  with an unhedged statement of the ruled behavior.
- confidence: high
- status: code-wrong
- ruling: Gate 2 J: EXCP with a vector below 0x60 must raise a fault (privilege violation), not
  panic; the guard must compare vector numbers, not doubled addresses. The appendix D example must
  move to vector 0x60 when the check lands. Manual stands (all three locations). Originally filed as
  three separate findings, `F-exc-7`, `F-sum-7`, and `F-cop-9`; merged here since they share one
  root cause and one ruling.

### F-arch-6 / F-cop-14 (not-implemented, major) — DMA unit documented "Required" but not implemented

- category: contradiction
- claim: two manual locations both mark the DMA unit (coprocessor `0x2`) "Required":
  - `chapters/01-introduction.tex:95-100, 110`: "All implementations of the SIRC-1 will have the
    \"processing unit\" as coprocessor 0x0, the \"exception unit\" as coprocessor 0x1, and the DMA
    unit as coprocessor 0x2." / the coprocessor table's DMA row is marked required (not optional).
  - `chapters/16-coprocessor-instructions.tex:141`: "0x2 & DMA Unit & Yes & 0x8--0xA & Required
    standard DMA transfer unit".
- evidence: `sirc-vm/peripheral-cpu/src/lib.rs:375-394` dispatches only coprocessor IDs 0 and 1 and
  raises `Faults::InvalidOpCode` for every other ID, including `0x2` -- so the reference
  implementation is a conforming SIRC-1 (per the manual's own definition) that nonetheless omits a
  coprocessor the manual calls required. The facts digest's "Coprocessors" table records DMA as
  "not implemented in peripheral-cpu."
- resolution: code-wrong
- fix: settle whether the reference implementation is a deliberately incomplete model (say so once,
  e.g. "the reference simulator omits the DMA unit and raises an invalid-opcode fault for
  coprocessor 0x2") or whether DMA should be optional like the math unit. As written, an emulator
  author can't tell from the manual whether omitting coprocessor 0x2 is conformant, and the
  probe-by-invalid-opcode-fault advice in Chapter 16 is meaningless for a "required" coprocessor
  that the reference CPU faults on. The DMA timing figures in Chapter 16 depend on the same
  decision.
- confidence: medium
- status: not-implemented
- ruling: Gate 2 M: the DMA unit is Required and planned; the reference emulator has not implemented
  it yet. Manual stands (both locations). Originally filed as two separate findings, `F-arch-6` and
  `F-cop-14`; merged here since they're the same gap.

### Make Level 5 (NMI) hardware exception edge-triggered in the simulator (code-wrong, major) — NEW 2026-09-12

- category: fact
- claim: chapter 6 Section 6.3.4 ("Level 5 NMI-like Behavior") and Appendix F ("Non-Maskable
  Interrupt Inputs (NMI)" and the external input sampling rules table) now document Level 5 as
  edge-triggered, unlike interrupt levels 1-4, which are level-sensitive. This was the original
  design intent (author-confirmed 2026-09-12): edge-triggering avoids spurious Level Five Hardware
  Exception Conflict faults from a line that simply stays asserted, matching the period convention
  of level-sensitive maskable IRQs alongside an edge-triggered NMI.
- evidence: `peripheral-cpu/src/lib.rs:499-518` (`raise_hardware_interrupt()`) ORs the asserted
  interrupt bits, including Level 5's, into `eu_registers.pending_hardware_exceptions` on every poll
  cycle the pin is high -- pure level sampling, with no rising/falling-edge detection anywhere in the
  codebase. The NMI-like re-entry special case in `get_cause_register_value()` /
  `handle_exception()` (`exception_unit/execution.rs:176-184, 242-249`) only governs what happens
  while the CPU is *already at* exception level 6 (raising a Level Five Hardware Exception Conflict
  fault instead of coalescing); it does not implement edge-triggering, and a Level 5 pin held high
  while the CPU is below level 6 is still just OR-latched exactly like levels 1-4.
- resolution: code-wrong (manual now documents the intended design; simulator has not implemented
  it).
- fix: add real rising-edge detection for the Level 5 / NMI line specifically -- latch a pending
  Level 5 exception only on a 0→1 transition of the pin, and require the pin to be released and
  reasserted before it can latch again -- separate from the OR-latch level-sampling logic used for
  interrupt levels 1-4.
- confidence: high
- status: not started; author's call for a future simulator branch. Manual updated to the target
  (edge-triggered) design in this pass.

### F-con2-20 (code-wrong, major) — stray `SHFT` row in the register-format generator output

- category: contradiction
- claim: `generated/register-format-encodings.tex:44-45` lists `SHFT` as a register-format encoding
  with a note that it is "a meta-instruction using ORRI opcode 0x25 with AF=10".
- evidence: `SHFT` lowers to the short-immediate format, not the register format, so it does not
  belong in this table.
- resolution: code-wrong (generator bug -- the chapters only include this file's generated output,
  so the fix must go in the generator itself)
- fix: remove the `SHFT` row from the register-format-encodings generator and delete the orphaned
  note.
- confidence: high
- status: not started

### F-alu-8 / F-enc-21 (minor) — REVISED 2026-09-08, register shift counts above 15

- category: fact
- claim: `chapters/09-shift-operations.tex` and every Instruction Fields list in chapter 13
  previously said "a register-supplied shift count greater than 15 is architecturally undefined."
- evidence, verified 2026-09-08 directly against `perform_logical_left_shift`,
  `perform_logical_right_shift`, `perform_arithmetic_left_shift`, `perform_arithmetic_right_shift`
  (`clamped_b = b.clamp(0, u16::BITS as u16)`, i.e. clamp to 16) and `perform_rotate_left`/
  `perform_rotate_right` (Rust's native `rotate_left`/`rotate_right`, which reduce the count modulo
  the type's bit width). This is real, deterministic, well-defined behavior, not undefined -- it was
  only untested (no existing test exercises a register-sourced count above 15; the tests with counts
  of 16/255 in `arithmetic_register_test.rs` use `ShiftOperand::Immediate`, whose 4-bit encoding
  truncates the count to 0-15 before it ever reaches this code, so they do not exercise this path at
  all).
- resolution: **the manual is now correct as documented** (updated 2026-09-08 to state the real
  clamp/modulo behavior instead of "undefined"), contingent on the `F-enc-18` fix above landing
  first -- until then, `ASR` with a register count of 16 or more gives the same wrong value as any
  other `ASR` count above 1 (`0x8000` or `0x0000` rather than `0xFFFF`/`0x0000`), which the
  `F-enc-18` fix corrects as a side effect.
- confidence: high
- status: manual updated; the `ASR` half depends on `F-enc-18`'s code fix landing; `LSL`/`LSR`/`ASL`/`RTL`/`RTR`
  need no code change, only the documentation fix already applied.

### Register renumbering (code-wrong, minor) — NEW 2026-09-08, resolves NOOP (`F-cop-3`) cleanly

- category: fact
- claim: the manual previously said `NOOP` assembles to `ADDI[N] sr, #0` and is unprivileged; the
  code faulted on it in protected mode because register field `0x0` names `sr`, a privileged
  register, and the general privilege check (`PRIVILEGED_REGISTERS.contains(&instruction.des)`,
  `execution.rs:21-43`) has no reason to exempt this one specific all-zero encoding.
- resolution (author decision, 2026-09-08, superseding the earlier "exempt this encoding" framing):
  renumber the `RegisterName` enum in `peripheral-cpu/src/registers.rs:46-62` so `r1` occupies index
  `0x0` and `sr` moves to `0x7`, with `r2`..`r7` shifting down to fill `0x1`..`0x6`. Verified
  2026-09-08 that nothing else depends on the old numeric order: every reference to `sr`'s index
  goes through the `RegisterName::Sr` enum symbol (`execution.rs:22`, `fetch_and_decode.rs:42`,
  `write_back.rs:47,57`, `protected_mode_test.rs:98,114`), not a hardcoded literal, so it follows the
  reordering automatically. The address-register-pair mechanism (`l`/`a`/`s`/`p`, indices
  `0x8`-`0xF`) is a separate 2-bit field and is unaffected. The toolchain's mnemonic-to-index table
  (`toolchain/src/parsers/instruction.rs:482-497`) maps by name, not position, so register *names*
  (`r1`-`r7`, `sr`) do not change, only their numeric encoding. One string-keyed lookup at
  `peripheral-cpu/src/registers.rs:538` (`"r7" => 7`) also needs updating to match.
- fix: reorder the enum, update the one hardcoded string-index table noted above, and reassemble
  every existing `.sasm`/binary that encodes a register field directly (assembly source using
  register *names* is unaffected and does not need editing, only reassembling).
- consequence: `NOOP` now assembles to `ADDI[N] r1, #0` (still the all-zero word `0x00000000`, since
  `r1` is now index `0x0`), and it is genuinely unprivileged, since `r1` is an ordinary
  general-purpose register -- no privilege-check special case is needed. This also means erased or
  uninitialized memory (which reads back as all-zero words) continues to execute safely as a no-op,
  which the earlier "just pick a non-zero-encoding register" option would have lost. Manual updated:
  `chapters/03-registers.tex` (Register Encoding table and its explanation),
  `chapters/17-meta-instructions.tex` (NOOP entry and cross-reference table),
  `chapters/12-instruction-summary.tex`, `chapters/appendix-e-quick-reference.tex`.
- confidence: high
- status: manual updated to the target (post-renumbering) state; the Rust enum reorder, the one
  string table, and reassembly of existing binaries are left for the author.

### F-con-66 / F-con2-30 (minor) — `0b` prefix in the generated encoding tables

- category: cosmetic
- claim: `generated/immediate-format-encodings.tex`, `generated/short-immediate-format-encodings.tex`,
  and `generated/register-format-encodings.tex` print two-bit fields with a `0b` prefix inside table
  cells where the column header already says the base -- 22 occurrences across the three files.
- evidence: these files are machine output, so the fix belongs in the generator, not the `.tex`. The
  same literals in `chapters/07-instruction-formats.tex` were already corrected by the chapter
  editor (that one file is hand-maintained, not generated).
- resolution: cosmetic, generator bug
- fix: drop the `0b` prefix in the generator so the AF column renders as `01`, `10`, `11` instead of
  `0b01`, `0b10`, `0b11`.
- confidence: high
- status: not started. Originally filed as two findings, `F-con-66` and `F-con2-30` (the second
  re-confirmed the first with an exact occurrence count); merged here.

### Remove `system_ram_offset` (cleanup, minor) — NEW 2026-09-09

- category: dead code
- claim: none -- this is not a manual/code mismatch, just cleanup surfaced while confirming chapter
  4's exception-vector addressing.
- evidence: `peripheral-cpu/src/registers.rs:166` (`pub system_ram_offset: u32` on `Registers`),
  `peripheral-cpu/src/lib.rs:197-200` (`new_cpu_peripheral(system_ram_offset: u32)` stores it
  verbatim), and `peripheral-cpu/src/coprocessors/exception_unit/execution.rs:304-305`
  (`vector_address = registers.system_ram_offset | (vector * INSTRUCTION_SIZE_WORDS)`). Every call
  site in the codebase passes `0x0` (`sirc-vm/src/main.rs:186`; no test overrides it), so the field
  has no observable effect anywhere it is actually used.
- author's context (2026-09-09): added very early in development to represent a fixed,
  manufacturing-time base-address register (not something the chip user could configure), then never
  actually used. The author is leaning toward removing it.
- resolution: n/a (cleanup, not a bug). The manual does not need to change either way: chapter 4
  already documents a hardwired `0x000000` vector-table base with no offset register, which is what
  the code does in every real configuration today, and which the author confirmed (2026-09-09) is
  period-plausible -- the base 68000 (pre-68010) has no Vector Base Register at all and is
  permanently fixed at address 0 in hardware, and the 6502's vectors are fixed at the top of the
  address space on every variant. If `system_ram_offset` is removed, chapter 4's wording needs no
  update; if it is instead wired up to a genuine fixed-at-manufacture value (i.e. still not
  runtime-configurable), chapter 4's wording also needs no update, since it already describes the
  address as fixed rather than describing *why* it is fixed.
- fix: remove the `system_ram_offset` field from `Registers`, the `new_cpu_peripheral` parameter,
  and its use in `exception_unit/execution.rs`; update the one call site in `sirc-vm/src/main.rs`
  and any test helper that constructs a `CpuPeripheral` to match the new signature.
- confidence: high
- status: not started; author's call on timing, not required for manual accuracy.

## Already fixed (kept for the record)

### F-mem-5 / F-data-10 (code-wrong, blocker) — FIXED 2026-09-14 — pre-decrement effective-address overflow check was inverted

- category: contradiction
- claim: two manual locations state the same rule -- a segment-overflow fault is raised when
  `SR.A` is set and the low-word address calculation overflows or underflows:
  `chapters/14-memory-instructions.tex:94-95` ("Effective-address calculation can raise a
  segment-overflow fault when \texttt{SR.A}... is set and the 16-bit low-word address calculation
  wraps.") and `chapters/04-data-representation.tex:154-157` ("If the low word overflows or
  underflows and \texttt{SR.A}... is set, the instruction raises a segment-overflow fault.").
- evidence: `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:76-115`.
  For pre-decrement forms, `addr_inc` is `-1`, added as `0xFFFF`: `displaced.overflowing_add(0xFFFF)`
  reported overflow for every `displaced != 0` and *no* overflow for `displaced == 0`. With `SR.A`
  set, this meant (a) a pre-decrement whose displaced address is nonzero always faulted, even though
  no underflow actually occurred, and (b) a genuine underflow (displaced address 0, decrementing to
  `0xFFFF`) produced no carry and did not fault -- the check was inverted. No test covered a
  pre-decrement or underflow with `SR.A` set prior to this fix.
- resolution: code-wrong (fixed)
- fix (applied): the borrow out of the `-1` decrement was being fed into the same overflow term as
  the carry out of the displacement add, inverting the test for every pre-decrement form.
  `execution_effective_address.rs` now computes the pre-decrement's borrow separately, via
  `displaced.overflowing_sub(1)`, instead of routing `-1` through `overflowing_add(0xFFFF)`; the `0`
  and `+1` (regular/post-increment) branches are unchanged, since `overflowing_add` was already
  correct for those. The same sentence also appears at `chapters/15-control-flow.tex:103-105`
  (covering `LDEA` pre-decrement, opcodes 0x1A/0x1B, which shares this code path) and needed no
  further change.
- test coverage added: `sirc-vm/peripheral-cpu/tests/exceptions/faults.rs` gained
  `test_segment_overflow_fault_with_predecrement_nonzero_does_not_fault` and
  `test_segment_overflow_fault_with_predecrement_underflow_faults`, both using `LDEA`
  pre-decrement (opcode `0x1A`), covering exactly the two previously-untested cases (a nonzero
  pre-decrement, which must not fault, and a genuine `0x0000 -> 0xFFFF` underflow, which must).
- confidence: high
- status: fixed
- ruling: Gate 2 D: the address-overflow trap must detect signed wrap of the low word, not unsigned
  carry. Manual rule stands (all three locations); the code now matches it. Originally filed as two
  separate findings, `F-mem-5` and `F-data-10`; merged here since they're the same code bug.

### F-sum-30 (minor) — FIXED 2026-09-09 — vector count comment in every example source

Six `examples/*/*.sasm` headers (`boring`, `byte-sieve`, `faults`, `hardware-exception`,
`store-load`, `software-exception`) said "128x32 bit exception vectors" while reserving room for 256
(`.ORG 0x0200` = 512 words = 256 vectors x 2 words). Changed the comment to "256x32 bit exception
vectors" in all six. Re-assembled all six with `cargo run -p toolchain --bin assembler` to confirm
no breakage.

### F-sum-32 (minor) — FIXED 2026-09-09 — `examples/faults/faults.sasm` taught a field that does not exist

The example defined `$CPU_PHASE_INSTRUCTION_FETCH` (`#0x0000`) and `$CPU_PHASE_EFFECTIVE_ADDRESS`
(`#0x0003`) and commented them as "the phase section of the status register". There is no such
field; `r7` here holds fault metadata (via `ETFR r7, #7`), and bits 0-2 are the bus access type
(BAT), per `06-exceptions.tex` and the BAT table in `appendix-f-external-interface.tex` (`0b000` =
No I/O, `0b011` = Data Write -- exactly the two values this example tests). Renamed to `$BAT_NONE` /
`$BAT_DATA_WRITE` and corrected both "Mask off the phase section of the status register" comments to
"Mask off the bus access type (bits 0-2) from the fault metadata register". Re-assembled to confirm
no breakage.

### F-sum-31 (minor) — FIXED 2026-09-09 — `examples/hardware-exception/hardware-exception.sasm`

Comment said "set bits 9-13 of SR" for a mask (`0b0001_1110_0000_0000`) covering bits 9-12.
Corrected to "Enable all four maskable hardware interrupt lines (bits 9-12 of SR)".

### F-sum-34 (nit) — FIXED 2026-09-09 — `examples/byte-sieve/byte-sieve.sasm`

The stack-pointer setup (`LOAD sh, #0x00F0` / `LOAD sl, #0xFFFF`) was commented "Array Start =
0x00F0_0000", copy-pasted from the actual array-start comment three lines above. Corrected to "Stack
top = 0x00F0_FFFF".

### F-con-4 / G (minor) — REVISED 2026-09-08 — LOAD's shift is dead code

- category: fact
- claim: the manual previously said (matching the original design intent) that a register-offset
  `LOAD`'s optional shift suffix applies to the offset register `rO` before the effective-address
  calculation.
- evidence, verified 2026-09-08 by tracing the full path from assembler to bus write:
  - `toolchain/src/parsers/opcodes/load.rs:214-231` (the `LOAD rD, (rO, addr), shift` parser):
    encodes the destination into `r1`, the offset register into `r3`, and hardcodes `r2: 0x0 //
    Unused`. The shift fields are encoded into the instruction word regardless.
  - `peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:160-193`: for a
    register-format instruction, `sr_a` = register field `r2`, `sr_b` = register field `r3`.
    `do_shift` is called unconditionally on `sr_a` (giving `sr_a_`); `sr_b_` is always the raw,
    unshifted register value.
  - `peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:76-116`:
    the effective address for every `0x10`-`0x1F` opcode is `ad_l_.overflowing_add(sr_b_)` -- always
    the *unshifted* value.
  - `peripheral-cpu/src/coprocessors/processing_unit/stages/memory_access.rs:58`: the `LOAD`'s read
    address uses `intermediate_registers.alu_output` (derived from `sr_b_`), never `sr_a_`.
  - Net effect: for a register-offset `LOAD`, `r2` is always `0x0` (which register that names
    depends on the register-renumbering fix above), `do_shift` runs on *that* register's value at
    runtime, and the result (`sr_a_`) is written nowhere -- it has zero effect on the address, the
    loaded value, or any flag. The offset register `rO` (`sr_b`/`r3`) is always used unshifted.
  - Contrast with `STOR`: `toolchain/src/parsers/opcodes/store.rs:134-150` encodes the *source*
    register (the value to store) into `r2` (`sr_a`, shifted) and the offset register into `r3`
    (`sr_b`, unshifted, used for the address). `memory_access.rs:68`: the bus write's `data` field is
    `decoded.sr_a_` -- the shifted value. `STOR`'s shift is fully functional and matches its
    documentation exactly; this bug is `LOAD`-only.
- resolution: **author chose to fix this in the manual, not the code** (2026-09-08): the shift
  suffix is removed from `LOAD`'s legal syntax entirely (`chapters/08-addressing-modes.tex`,
  `chapters/14-memory-instructions.tex`).
- optional follow-up: if the assembler is changed to match, reject a `ShiftDefinition` operand on
  the `LOAD rD, (rO, addr)` and `LOAD rD, (rO, addr)+` forms in
  `toolchain/src/parsers/opcodes/load.rs` (currently accepted at lines 214 and 296) with an error
  message, matching the existing rejection of a shift on the direct-register-to-register form
  (`load.rs:135-142`, "Cannot use a shift with direct register -> direct register loads. Use SHFT
  instruction instead."). No change needed to `STOR`.
- confidence: high
- status: manual updated (done); the optional assembler-side rejection above is not required for
  correctness, since the encoding was already inert.

## Superseded / no longer applicable

### F-tim-1 — superseded by `F-data-1`

`appendix-b-timing.tex`'s `ADDR|HI`/`ADDR|LO` timing comments and its `SUBI`/`BRAN|HI` unrolled-loop
example (line 199) were checked against the corrected `HI`/`LO` predicates from `F-data-1`
(`!Carry && !Zero` / `Carry || Zero`) and found already correct as written -- no manual change
needed once `F-data-1`'s code fix lands. No separate fix required.

### F-cop-3 — superseded by "Register renumbering" above

Original claim: `chapters/17-meta-instructions.tex:67` said `NOOP`'s "Privilege: Available in
protected mode and supervisor mode," but the code's privilege check treated `NOOP`'s destination
register field (`0x0`, which named `sr`) as a write to a privileged register, so it would
incorrectly fault in protected mode. Resolved by the register renumbering above: moving `sr` to
index `0x7` and `r1` to index `0x0` makes `NOOP`'s all-zero encoding genuinely unprivileged with no
special-case needed. Ruling: Gate 2 I, NOOP must not fault in protected mode -- satisfied by the
renumbering rather than a NOOP-specific exemption.

### Invalid Opcode Fault doc-comment entry — removed 2026-09-12

An earlier version of this file briefly logged a finding claiming
`exception_unit/definitions.rs:121-131`'s comment was wrong to list Invalid Opcode Fault as
retryable. The author has since ruled the opposite way (see `F-exc-3` above): Invalid Opcode Fault
*is* retryable, so that comment was correct all along, and the finding was removed as moot.
