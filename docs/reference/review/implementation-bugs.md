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

### F-exc-3 (code-wrong, blocker) — FIXED 2026-09-14 — retryable faults captured the wrong return address

- category: fact
- claim: chapter 6 documents a unified retryable fault set with no exceptions: Alignment, Bus, Bus
  Protection, Invalid Opcode Fault, Privilege Violation, and Segment Overflow (both the
  program-counter-wraparound and address-computation cases) all retry the faulting instruction
  itself, not the one after it.
- root cause: `raise_fault` (`lib.rs`) only marked a fault pending and wrote bus-fault *metadata*; it
  never captured a return address. The actual retry address was read later, fresh, in
  `handle_exception` (`exception_unit/execution.rs`), from whatever `registers.pl` happened to be at
  dispatch time -- and by then `pl` had already been advanced past the faulting instruction for every
  fault type except Alignment and PC-wrap Segment Overflow (which are detected before decode runs).
  A second, undocumented instance of the same bug was found during implementation:
  `write_back.rs`'s `set_address_register_values` (Privilege Violation on an illegal address-register
  high-word write) has the identical problem, several pipeline phases later than the decode-time
  check this finding originally cited.
- resolution: code-wrong (fixed)
- fix (applied): `ExceptionUnitRegisters.pending_fault` changed from `Option<Faults>` to
  `Option<PendingFault>`, a new struct bundling the fault with the address it should return to.
  `raise_fault` gained a `fault_pc: u32` parameter and became the sole writer of `pending_fault`
  (no call site can raise a fault without stating its retry address). `DecodedInstruction` gained
  `pc_l_`/`pc_h_` (the instruction's own fetch address, captured before decode advances `pl`), letting
  later-phase and later-cycle call sites (non-PC-wrap Segment Overflow, both Privilege Violation
  sites, Bus/Bus Protection, Invalid Opcode) recover the correct address. `handle_exception` now
  reads the address off `pending_fault` instead of `registers.get_full_pc_address()`. The one
  deliberate exception: `handle_vector_fetch_bus_fault`'s double-fault-during-vector-fetch escalation
  explicitly carries the *original* fault's address forward rather than computing a new one.
- test coverage added: corrected two `sirc-vm/peripheral-cpu/tests/exceptions/faults.rs` assertions
  that were pinning the old bug (`test_double_fault`), added a return-address assertion to every
  existing fault test that lacked one, and added `test_privilege_violation_fault_on_address_register_high_word_write`
  and `test_bus_fault_during_instruction_fetch` for the two previously-uncovered call sites.
- example fixes: `examples/faults/faults.sasm`'s fault handlers (`bus_fault_handler`,
  `bus_protection_fault_handler`, `invalid_opcode_fault_handler`, `double_fault_handler`,
  `privilege_violation_fault_handler`) gained the `ETFR`/`ETTR` return-address redirects this finding
  predicted would be needed once retry became universal; the deliberate nested-double-fault test was
  also moved to run last, since it leaves `current_exception_level` unable to fully unwind (a
  separate, pre-existing limitation -- see follow-up below) and would otherwise corrupt the tests
  after it. `examples/math-coprocessor-emulation/math-coprocessor-emulation.sasm` needed one line
  (`ADDI al, #2`) to skip past the trapped `MULU` before saving its return-state snapshot. Both
  examples' golden register-dump fixtures were regenerated.
- follow-up filed separately (not part of this fix): making faults genuinely retryable exposed that
  `ExceptionLinkRegister.saved_exception_level` has no way to survive a second fault reusing the same
  link-register slot, so `current_exception_level` gets stuck at the fault level after any nested
  fault instead of unwinding back to 0. This was pre-existing and previously masked by the very bug
  fixed here.
- confidence: high
- status: fixed
- ruling: Gate 2 unchanged -- the unified retryable set stands; the simulator now matches it.

### F-data-1 (code-wrong, blocker) — FIXED 2026-09-14 — HI/LO condition predicates used the wrong carry polarity

- category: fact
- claim (as originally documented): "HI (Unsigned Higher) Executes when C = 1 AND Z = 0. Tests if
  first operand $>$ second operand (unsigned)."
- evidence: `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:195-206`
  (`perform_subtract` sets C from `a.overflowing_sub(b)`, i.e. C = borrow = `a < b` unsigned);
  `definitions.rs`'s `UnsignedHigher`/`UnsignedLowerOrSame` arms were written assuming ARM's
  *no-borrow* Carry convention (per the file's own "copied from ARM" comment) -- the opposite
  polarity from what this CPU's subtraction actually produces.
- resolution: code-wrong (fixed) -- the manual's original ARM-style glosses for `HI`/`LO`/`CS`/`CC`
  were correct in *intent*; only the `Carry` term in two compound predicates needed to change.
- fix (applied): `definitions.rs`, `ConditionFlags::should_execute` --
  `UnsignedHigher = !Carry && !Zero` (was `Carry && !Zero`), `UnsignedLowerOrSame = Carry || Zero`
  (was `!Carry || Zero`). `CarrySet`/`CarryClear` and the signed predicates
  (`GreaterOrEqual`/`LessThan`/`GreaterThan`/`LessThanOrEqual`) were left untouched, as they don't
  depend on this polarity. The stale "copied from ARM" comment at line 114 was also updated to
  note `HI`/`LO`/`CS`/`CC` are the one exception, matching the ARM-divergence note already in
  `chapters/10-condition-codes.tex`.
- test coverage added: zero existing tests exercised `HI`/`LO` at all prior to this fix -- a real
  coverage gap, which is presumably why the bug shipped unnoticed. New file
  `sirc-vm/peripheral-cpu/tests/instructions/condition_flags_test.rs` adds an exhaustive
  `should_execute` truth-table test across all four `(Carry, Zero)` combinations, plus six
  end-to-end integration tests (`CMPR`-equivalent flag setup followed by a conditional `ADDI`)
  proving `HI`/`LO` gate correctly through the real assemble/decode/execute path.
- example fix: `examples/comprehensive-test/comprehensive-test.sasm`'s TEST 35/36 were found to
  currently "pass" only by accident -- their `CMPR` operand order was backwards relative to their
  own stated intent (`CMPR r2, r1` when comparing "is r1 > r2"), which happened to cancel out
  against the polarity bug. Fixed the operand order to `CMPR r1, r2` in both, matching the
  manual's established convention; both tests now pass correctly rather than accidentally, and
  the example's golden register-dump fixture needed no changes (verified via `make check`). No
  other `.sasm` file uses `|HI`/`|LO`/`|CS`/`|CC` in a way that executes.
- confidence: high
- status: fixed
- ruling: Gate 2 B stands: the manual's ARM-style `HI`/`LO` glosses are unchanged; `CS`/`CC`
  glosses state their real (non-ARM) meaning; the code now matches both.

### F-enc-18 (code-wrong, blocker) — FIXED 2026-09-15 — ASR did not sign-fill for shift counts above 1

- category: fact
- claim: manual states "Bit 15 (sign bit) is replicated into all bits vacated by the shift" and
  gives worked examples of `ASR #2`, `ASR #3`, `ASR #8` as signed division.
- evidence: `perform_arithmetic_right_shift` (`sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs`)
  computed `extended_a.rotate_right(clamped_b) | sign_bit` -- this only forced bit 15 to the
  original sign; for shift count `n > 1`, bits 14 down to `16-n` stayed zero instead of
  sign-extended. At `n = 1` there's only one vacated bit, so the result happened to look correct,
  which is why this went unnoticed -- the manual's own worked examples and the appendix D listings
  only ever use `ASR #1`.
- resolution: code-wrong (fixed)
- fix (applied): rewrote `perform_arithmetic_right_shift` to sign-extend to `i32` before shifting
  (`(i32::from(a as i16) >> clamped_b) as u16`), using Rust's native arithmetic right shift on a
  signed type, which correctly saturates to all-1s/all-0s once the shift count reaches 16
  (`clamped_b` is already bounded to `0..=16` by the existing clamp, nowhere near `i32`'s actual
  32-bit shift-panic threshold). The carry computation (a rotate-into-bit-31 trick, independent of
  the sign-fill bug) was kept unchanged; only the now-unnecessary `sign_bit` variable and its OR
  were removed. No other shift function needed changes.
- test coverage: the doc's own evidence section cited one buggy assertion, but two independent
  research passes found **four** across two files, all corrected: `arithmetic_register_test.rs::test_arithmetic_shift_right_register`
  (shift counts 6, 15, and `u8::MAX`-truncated-to-15, all with input `0xCCCD`) and
  `arithmetic_short_immediate_test.rs::test_arithmetic_shift_right_immediate` (shift count 15,
  input `0x80CD` -- this one was already flagged suspect with a `// Failing` comment, now removed).
  Also added `test_arithmetic_shift_right_register_operand_count_above_15`, since no test anywhere
  in the workspace previously exercised `ShiftOperand::Register` at all -- it proves the
  `F-alu-8`/`F-enc-21` register-count-\ge{}16 case is genuinely fixed as a side effect, not just
  claimed to be.
- example fix: `examples/comprehensive-test/comprehensive-test.sasm` TESTs 24, 53, and 57 (not
  mentioned in the doc) asserted the buggy value (`0x8F00`/`0xA000`/`0x8F00`); corrected to the
  true arithmetic-shift results (`0xFF00`/`0xE000`/`0xFF00`). The golden register-dump fixture was
  regenerated (`r6` changes from `0x8f00` to `0xff00`; the pass count stayed at 72/`0x48` and the
  success marker stayed `0x0fab`, confirming the fix didn't just trade one failure for another).
- confidence: high
- status: fixed
- ruling: Gate 2 F stands: ASR must sign-fill every vacated bit. Manual text needed no change (its
  worked examples were already right).

### F-alu-8 / F-enc-21 (minor) — FIXED 2026-09-15 — register shift counts above 15

- category: fact
- claim: `chapters/09-shift-operations.tex` and every Instruction Fields list in chapter 13
  previously said "a register-supplied shift count greater than 15 is architecturally undefined."
- evidence: verified directly against `perform_logical_left_shift`, `perform_logical_right_shift`,
  `perform_arithmetic_left_shift`, `perform_arithmetic_right_shift` (`clamped_b = b.clamp(0,
  u16::BITS as u16)`, i.e. clamp to 16) and `perform_rotate_left`/`perform_rotate_right` (Rust's
  native `rotate_left`/`rotate_right`, which reduce the count modulo the type's bit width). This is
  real, deterministic, well-defined behavior, not undefined -- it was only untested.
- resolution: manual is correct as documented (clamp/modulo behavior, not "undefined"); the `ASR`
  half was contingent on `F-enc-18`'s code fix, which has now landed (see above) -- confirmed by
  the new `test_arithmetic_shift_right_register_operand_count_above_15` test.
- confidence: high
- status: fixed; `LSL`/`LSR`/`ASL`/`RTL`/`RTR` never needed a code change (only the documentation
  fix, already applied); `ASR` is now fixed too.

### Make Level 5 (NMI) hardware exception edge-triggered in the simulator (code-wrong, major) — FIXED 2026-09-15

- category: fact
- claim: chapter 6 ("Level 5 NMI-like Behavior") and Appendix F ("Non-Maskable Interrupt Inputs
  (NMI)" and the external input sampling rules table) document Level 5 as edge-triggered, unlike
  interrupt levels 1-4, which are level-sensitive: edge-triggering avoids spurious Level Five
  Hardware Exception Conflict faults from a line that simply stays asserted.
- evidence: `raise_hardware_interrupt()` (`sirc-vm/peripheral-cpu/src/lib.rs`) ORed every asserted
  interrupt bit, including Level 5's, into `eu_registers.pending_hardware_exceptions` on every poll
  cycle the pin was high -- pure level sampling, with no rising/falling-edge detection anywhere in
  the codebase. The existing NMI re-entry special case in `get_cause_register_value()`/
  `handle_exception()` only governs what happens once a *second* L5 is already latched while the
  CPU is at exception level 6 (raising `LevelFiveInterruptConflict`) -- it's downstream of, and
  orthogonal to, the latching mechanism and needed no changes.
- resolution: code-wrong (fixed)
- fix (applied): added a private `l5_interrupt_pin_previously_asserted: bool` field to
  `CpuPeripheral` (following the exact precedent of the existing `is_halted` pin-edge-tracking
  field -- reset in `reset()`, not an architectural register so it doesn't live in
  `ExceptionUnitRegisters`). `raise_hardware_interrupt` now splits levels 1-4 (unchanged
  level-sensitive OR-latch) from level 5 (only latches on a 0→1 transition of its bit versus the
  previous call, and won't re-latch until released and reasserted). A critical wiring fix was
  needed alongside this: `raise_hardware_interrupt` was previously only called when
  `bus_assertions.interrupt_assertion > 0`, so it had no way to ever observe "the pin went low" --
  the call site in `poll_internal` now calls it unconditionally every poll cycle (a no-op for
  levels 1-4 when nothing is asserted, exactly as before).
- test coverage added: `sirc-vm/peripheral-cpu/tests/exceptions/hardware_exceptions.rs` gained
  `test_level_five_interrupt_is_edge_triggered` (proves a held-high pin latches once, not
  re-latching until released and reasserted) and
  `test_level_one_to_four_interrupts_remain_level_sensitive` (regression-proves levels 1-4 are
  unaffected by the restructuring). No existing test needed correcting -- every prior L5 touchpoint
  (unit tests and `examples/faults/faults.sasm`, whose L5 assertions go through a one-shot,
  self-clearing debug-device register write) already asserted L5 as a single discrete pulse, so
  this was a pure coverage gap, not a masked regression.
- confidence: high
- status: fixed
- ruling: manual stands; the simulator now matches the documented edge-triggered design.

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

### F-arch-6 / F-cop-14 (contradiction) — REVISED 2026-09-15, supersedes the Gate 2 M ruling — DMA unit is now Optional

- category: contradiction
- original claim (2026-09-05): two manual locations both marked the DMA unit (coprocessor `0x2`)
  "Required" (`chapters/01-introduction.tex`'s coprocessor table and prose; the "Required" column
  of `chapters/16-coprocessor-instructions.tex`'s Standard Coprocessor IDs table), while
  `sirc-vm/peripheral-cpu/src/lib.rs:375-394` dispatches only coprocessor IDs 0 and 1 and raises
  `Faults::InvalidOpCode` for every other ID, including `0x2` -- so the reference implementation
  omitted a coprocessor the manual called required, with no stated basis for that requirement.
- original ruling (Gate 2 M, 2026-09-05): the DMA unit is Required and planned; the reference
  emulator just hadn't implemented it yet. Manual stood as written.
- revised ruling (2026-09-15, author's call): reconsidered and reversed. There was no architectural
  reason DMA needed to be required -- unlike the processing unit and exception unit, it doesn't
  participate in reset or exception handling, and software without it can always fall back to
  ordinary `LOAD`/`STOR` loops, the same story as the already-optional Integer Maths Unit. The DMA
  section's own hedge ("software must not assume Coprocessor 2 is present") was already in tension
  with calling it required. The manual now marks DMA optional in both locations (matching the Maths
  Unit's "No" / "Optional" table entries and phrasing), and every `DMAR`/`DMAW`/`DMAT`
  `Applicability` line and `Exceptions` field was updated to match ("an optional coprocessor absent
  on models that do not implement it" / "Missing DMA hardware ... raise invalid-opcode faults").
- resolution: **author chose to fix this in the manual, not the code** -- the reference
  implementation's current non-support for coprocessor `0x2` is now fully conformant, since an
  optional coprocessor is allowed to be absent. No Rust change is required by this finding.
- confidence: high
- status: manual updated (done); no code fix needed or expected.

## Superseded / no longer applicable

### F-tim-1 — superseded by `F-data-1`

`appendix-b-timing.tex`'s `ADDR|HI`/`ADDR|LO` timing comments and its `SUBI`/`BRAN|HI` unrolled-loop
example (line 199) were checked against the corrected `HI`/`LO` predicates from `F-data-1`
(`!Carry && !Zero` / `Carry || Zero`, now fixed in code too -- see "Already fixed" above) and found
already correct as written. No separate fix required.

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
retryable. The author has since ruled the opposite way (see `F-exc-3`, now fixed, above under
"Already fixed"): Invalid Opcode Fault *is* retryable, so that comment was correct all along, and
the finding was removed as moot.

### `DMAT` register-count form / `DMATR` mnemonic — reverted 2026-09-15, same day it was proposed

Briefly added and then reverted within the same session. The idea: a register-operand form of
`DMAT` (first drafted as a separate four-letter-breaking mnemonic `DMATR`, then folded into `DMAT`
as a second syntax form) so a transfer's word count could be supplied at runtime instead of a fixed
assembly-time immediate. This entry's earlier revision had an "Add assembler support" finding open
against it. The author then pointed out a project-wide rule this violated that hadn't been written
down yet: SIRC-1 meta-instructions are always aliases for exactly one real instruction, so that a
program's instruction count (and therefore its timing) is always predictable from its source
without needing to track which mnemonics secretly expand to more than one. The register-count form
assembles to three instructions (`ANDI[N]`, `ORRI[N]`, `COPR`), which breaks that rule -- there was
no way to make it a single instruction, since `COPR` relays a register's full 16-bit value as the
command verbatim, with no field-extraction mechanism to combine a fixed header with a runtime count
in one instruction.
- resolution: reverted. `chapters/16-coprocessor-instructions.tex`'s `DMAT` entry is back to its
  single, immediate-only form. A "Notes" item on that entry now documents the `ANDI[N]`/`ORRI[N]`/`COPR`
  sequence as an explicit technique a programmer can use directly (ordinary use of the
  already-documented `COPR` instruction), not as a `DMAT` form or a new mnemonic. The general rule
  itself is now stated explicitly in `chapters/17-meta-instructions.tex`'s overview, so this doesn't
  need to be rediscovered per-instruction again.
- no assembler work needed: nothing to implement, since the feature was withdrawn before being
  built.
