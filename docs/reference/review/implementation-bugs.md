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

### SHFR (register-count shift) — undocumented, unimplemented feature gap — NEW 2026-09-15

- category: enhancement
- claim: none -- not a manual/code mismatch (the manual documents no `SHFR` or register-count-shift
  meta-instruction anywhere); noticed while investigating a misplaced `SHFT` example row in the
  generated register-format-encodings table (see "SHFT generator row..." under Already fixed).
- evidence: `StatusRegisterUpdateSource::Shift` (`peripheral-cpu/src/coprocessors/processing_unit/definitions.rs`)
  is a fully decoder-supported value with an explicit comment: "At the moment hardcoded to Alu for
  all ALU instructions except for SHFI/SHFR ... It is supported by the decode[r], I just need to
  work out a syntax for the assembler." A test-file comment
  (`peripheral-cpu/tests/instructions/arithmetic_register_test.rs`) independently names the same
  planned feature: "These tests would be the equivalent of the SHFR meta instruction -- reuses the
  same opcode as ADDR but the StatusRegisterUpdateSource is Shift." Register-format ALU instructions
  already support `ShiftOperand::Register` (a register-supplied shift count, e.g. `SUBR|>> r1, r2,
  r3, ASR r4`), so the hardware primitive SHFR would need already exists, and is exercised by
  existing tests under a raw `Instruction::AddRegister` encoding, not through an assembler mnemonic.
  `SHFR`/`SHFI` appear nowhere in the manual, any `.sasm` example, or the toolchain parser (`grep`
  across all three: zero matches) -- there is currently no way to write this in assembly.
- resolution: n/a (feature gap, not a bug -- nothing documents SHFR as existing today, so there is no
  contradiction to resolve)
- fix (if implemented): add assembler syntax for a register-count shift meta-instruction (working
  name `SHFR`, matching the existing `SHFT`/short-immediate naming and the code comments), lowering
  to the ALU opcode already used by the corresponding register-format instruction (e.g. `ADDR`) with
  `StatusRegisterUpdateSource::Shift` set and a redundant zero-valued operand, mirroring how `SHFT`
  lowers to `OrShortImmediate` with `AF=10` today. Would need manual coverage (a new entry in the
  meta-instructions or ALU-instructions chapter) and a `reference-encodings.sasm` example once
  implemented.
- confidence: high (the decoder-side support and the two independent code comments are unambiguous)
- status: not started; author's call on timing and whether this is wanted at all.

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

### F-exc-7 / F-sum-7 / F-cop-9 (code-wrong, major) — FIXED 2026-09-15 — EXCP below vector 0x60 panicked instead of raising a fault

- category: contradiction
- claim: three manual locations describe `EXCP` with a reserved vector (below `0x60`) as a
  controlled fault condition: `chapters/06-exceptions.tex`'s Privilege Violation Fault cause list,
  `chapters/16-coprocessor-instructions.tex`'s `EXCP` entry (hedged: "normally 0x60--0xFF"), and
  `chapters/appendix-d-examples.tex`'s included listing
  (`examples/software-exception/software-exception.sasm`) itself executed `EXCP #0x40`, a reserved
  vector.
- evidence: `sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs`'s
  `ExceptionUnitExecutor::step` `assert!`ed (panicked) when a `SoftwareException` opcode carried a
  vector below `USER_EXCEPTION_VECTOR_START` (0x60) instead of raising a fault.
  `processing_unit/execution.rs`'s `check_privilege` decides privilege purely from the static COP
  sub-opcode nibble, so it structurally cannot see `EXCP`'s runtime vector value -- the fix had to
  live in the exception unit itself.
- **found during implementation, beyond the original write-up**: the assert wasn't just wrong to
  panic, it compared the wrong representation and was silently broken even as a panic. It compared
  `self.vector_address as u8` (a *doubled, offset-OR'd byte address*) against
  `USER_EXCEPTION_VECTOR_START` (a raw *vector number*). Vectors `0x30`-`0x5F` double to
  `0x60`-`0xBE`, passing the `>= 0x60` check -- so the assert never actually fired for the example
  that supposedly demonstrated the bug (`0x40 * 2 = 0x80 >= 0x60`), which is why that example passed
  CI despite using a reserved vector. Only vectors below `0x30` would have panicked. The fix compares
  the raw pre-doubling `vector` value instead.
- **also found during implementation**: the manual's own pseudocode for `EXCP`
  (`16-coprocessor-instructions.tex`) gates the fault on protected mode -- `if protected_mode and
  vector < 0x60 then raise privilege_violation_fault` -- which the original finding's fix
  description didn't call out. Supervisor-mode code triggering a low-vector software exception must
  still dispatch normally (matching `check_privilege`'s general "no restrictions outside protected
  mode" convention elsewhere in the codebase); an early draft of the fix missed this and would have
  faulted unconditionally.
- resolution: code-wrong (fixed)
- fix (applied): removed the assert. In `ExceptionUnitExecutor::step`'s `ExecutionEffectiveAddressExecutor`
  phase, split the `SoftwareException` case out of the shared `Fault | HardwareException |
  SoftwareException => handle_exception(...)` dispatch arm with a match guard --
  `SoftwareException if vector < USER_EXCEPTION_VECTOR_START && sr_bit_is_set(ProtectedMode,
  registers)` -- that calls `raise_fault(eu_registers, Faults::PrivilegeViolation, ...)` instead of
  dispatching. `raise_fault` only sets `pending_fault`; the actual jump happens the *next* poll
  cycle when `get_cause_register_value` sees it and re-enters with a `Fault` op-code, which is why
  `Fault` can never be misrouted by this guard -- it's only ever the op-code when `pending_fault` was
  already `Some` going in. `registers.get_full_pc_address()` is used as the fault's return address,
  matching this same function's existing `LevelFiveInterruptConflict` precedent: the exception unit's
  own multi-cycle dispatch has no pre-PC-advance snapshot threaded into it (unlike the main
  pipeline's write-back stage), so this fault resumes at the instruction after `EXCP`, not `EXCP`
  itself -- a pre-existing structural limit of this dispatch path, not something this fix changes.
- test coverage added: `sirc-vm/peripheral-cpu/tests/exceptions/faults.rs` gained
  `test_privilege_violation_fault_on_reserved_software_exception_vector` (proves the fault, not a
  panic) and `test_reserved_software_exception_vector_allowed_in_supervisor_mode` (regression-proves
  the protected-mode gate found during implementation). No existing test needed correcting --
  nothing previously exercised this path (`grep -rn "should_panic"` across the crate: zero matches).
- consequence: `examples/software-exception/software-exception.sasm` moved off the reserved vectors
  `0x40`/`0x41` to `0x60`/`0x61` (vector table `.ORG 0x0080` -> `0x00C0`); its
  `.register-dump-expected` is unaffected, since its addresses are all in the unmoved `:start`/handler
  code region. `examples/math-coprocessor-emulation/math-coprocessor-emulation.sasm`'s `EXCP #0x70`
  was already valid and unaffected; no other example used a reserved `EXCP` vector. Manual:
  `chapters/16-coprocessor-instructions.tex`'s hedged "normally 0x60--0xFF" tightened to state the
  fault condition directly; `chapters/appendix-d-examples.tex`'s prose vector numbers updated to
  match the moved example; `chapters/06-exceptions.tex` was already correct and needed no change.
- confidence: high
- status: fixed
- ruling: Gate 2 J: EXCP with a vector below 0x60 must raise a fault (privilege violation) in
  protected mode, not panic; satisfied. Manual stands (all three locations). Originally filed as
  three separate findings, `F-exc-7`, `F-sum-7`, and `F-cop-9`; merged here since they share one
  root cause and one ruling.

### F-con2-20 (code-wrong, major) — FIXED 2026-09-15 — stray `SHFT` row in the register-format generator output

- category: contradiction
- claim: `generated/register-format-encodings.tex` listed `SHFT` as a register-format encoding.
- evidence: `SHFT` always lowers to the short-immediate instruction format -- confirmed
  independently against `toolchain/src/parsers/opcodes/arithmetic_immediate.rs`: its only accepted
  operand shape constructs `InstructionData::ShortImmediate`, never `Register`. It does not belong
  in the register-format table.
- **found during implementation, correcting the original claim**: the generated `.tex` file itself
  carried no explanatory note (the "meta-instruction using ORRI opcode 0x25 with AF=10" text the
  original finding quoted lives in the hand-maintained `chapters/07-instruction-formats.tex`'s
  Notes list, which `\input`s the generated table -- conflated by the original write-up).
- **user pushback, investigated rather than dismissed**: could the misplaced line be a stub for a
  genuine "SHFT has a register-count form" design, not a mistake? Verified it wasn't a working
  stub -- the line read `SHFT r1, ASL #3`, an *immediate* count, byte-for-byte identical to the
  correctly-placed short-immediate example; a register-count form would need syntax like `SHFR r1,
  ASL r2`, which appears nowhere. The question did surface a real, separate finding: see "SHFR
  (register-count shift)" under Open.
- resolution: code-wrong (fixed)
- fix (applied): removed the stray `@manual-example SHFT r1, ASL #3` annotation and instruction from
  `examples/reference-encodings/reference-encodings.sasm` (its correctly-placed short-immediate
  `SHFT r3, RTR #5` example was untouched); dropped the now-orphaned "(see the SHFT row)"
  cross-reference and standalone SHFT note from `chapters/07-instruction-formats.tex`'s
  register-format Notes list (the short-immediate section's identical note, where SHFT genuinely
  belongs, was left alone). Regenerated `generated/register-format-encodings.tex` via `make
  manual-tex`.
- **unplanned bug found during regeneration**: removing the line dropped the fixture's total
  instruction count to 33 (odd), which exposed a latent off-by-one in the assembler's
  `ensure_program_size` (`toolchain/src/data/object.rs`) -- it called `resize(min_size + 1, ...)`
  instead of `resize(min_size, ...)`, silently appending one trailing zero-padding instruction word
  to any assembled program whose token count left that pre-allocated slot unfilled. Invisible for
  ordinary programs (nothing executes past a halt), but corrupted this fixture's own word-for-word
  count check, and shifted addresses inside two other examples that happened to trip the same
  parity (see "off-by-one in `ensure_program_size`" below). Fixed alongside this finding at the
  user's direction.
- confidence: high
- status: fixed

### Off-by-one in `ensure_program_size` silently padded assembled programs — FIXED 2026-09-15

- category: fact (found during implementation of F-con2-20, not part of the original review)
- claim: none from the manual pass -- discovered while regenerating `reference-encodings` after
  removing the stray `SHFT` row above.
- evidence: `toolchain/src/data/object.rs`'s `ensure_program_size(program, min_size)` called
  `program.resize(min_size + 1, [0x0, 0x0, 0x0, 0x0])` instead of `resize(min_size, ...)`. Every
  token write goes through exactly one `[u8; 4]` slot (`inject_data_value` confirmed to write a
  single slot even for `DQ_VALUE`, a 32-bit value), so the `+1` had no legitimate purpose -- it just
  over-allocated one extra word ahead of where the token stream had actually reached. That extra
  slot is silently overwritten by the *next* token's own write in the common case, so the bug was
  invisible for any program that ends in an unreachable halt/loop; it only leaves a lasting trailing
  (or, across an `.ORG` jump, mid-file) zero-padding word when the token-count parity leaves nothing
  to overwrite it. `reference-encodings`'s own word-for-word count check
  (`examples == words`) caught it directly; `comprehensive-test`'s and
  `protected-mode-writeback`'s `.register-dump-expected` link-register addresses (baked-in
  downstream of the stray padding) were shifted by it too.
- resolution: code-wrong (fixed)
- fix (applied): `program.resize(min_size, [0x0, 0x0, 0x0, 0x0])`.
- test coverage added: `toolchain/tests/types/object_test.rs` gained
  `test_build_object_does_not_pad_a_trailing_word_for_odd_instruction_counts`, asserting a
  3-instruction program assembles to exactly 12 bytes. No existing test needed correcting for
  behavior -- the two toolchain assembler snapshot tests (`arithmetic_immediate_test`,
  `arithmetic_register_test`) both happen to assemble to even instruction counts (36 and 26 words)
  and were confirmed byte-for-byte unaffected. `comprehensive-test.register-dump-expected` was
  regenerated (one link-register return address shifted by exactly one instruction, `0x7ca` ->
  `0x7c8`); verified against a pre-session commit (`f39aa96`, before any of this session's work)
  that `protected-mode-writeback`'s own separate, pre-existing double-fault crash (unrelated to this
  fix) predates this and every other change made this session -- flagged separately, not fixed here
  since it's out of scope and not part of the CI example matrix.
- confidence: high
- status: fixed

### F-con-66 / F-con2-30 (minor) — FIXED 2026-09-15 — `0b` prefix in the generated encoding tables

- category: cosmetic
- claim: `generated/immediate-format-encodings.tex`, `generated/short-immediate-format-encodings.tex`,
  and `generated/register-format-encodings.tex` printed two-bit fields with a `0b` prefix inside
  table cells where the column header already states the base.
- **correction to the original claim**: cited as "22 occurrences" -- actually 34 at the time of
  this fix (15 register, 12 immediate, 7 short-immediate), dropping to 33 once the SHFT row (above)
  was removed.
- evidence: these are machine-generated files, so the fix belonged in the generator
  (`examples/reference-encodings/manual-tex.awk`), not the `.tex`.
  `chapters/07-instruction-formats.tex` (hand-maintained) was confirmed already clean (`grep -n
  "0b"`: zero matches).
- resolution: cosmetic, generator bug (fixed)
- fix (applied): dropped the literal `"0b"` immediately before `bin_plain(af, 2)` in all three
  row-emitting functions (`emit_immediate_row`, `emit_short_immediate_row`, `emit_register_row`) in
  `manual-tex.awk`. Regenerated all three `generated/*.tex` files via `make manual-tex`; diff
  confirmed to change only the 33 `0b`-prefixed cells, nothing else.
- confidence: high
- status: fixed. Originally filed as two findings, `F-con-66` and `F-con2-30`; merged here.

### Remove `system_ram_offset` (cleanup, minor) — FIXED 2026-09-15

- category: dead code
- claim: none -- not a manual/code mismatch, cleanup surfaced while confirming chapter 4's
  exception-vector addressing.
- evidence: `pub system_ram_offset: u32` on `Registers`, threaded through
  `new_cpu_peripheral(system_ram_offset: u32)`, OR'd into the exception unit's vector-address
  calculation. Exhaustively verified every one of the 33 call sites across the workspace (2
  production: `sirc-vm/src/main.rs`, `sirc-vm/benches/byte_sieve.rs`; 31 test call sites) passes
  exactly `0x0` -- no exceptions -- confirming the field has zero observable effect anywhere.
- author's context (2026-09-09): added very early in development to represent a fixed,
  manufacturing-time base-address register, then never actually used.
- resolution: n/a (cleanup, not a bug). Manual needed no change either way, as originally assessed --
  chapter 4 already documents a hardwired `0x000000` vector-table base with no offset register.
- fix (applied): removed the field from `Registers`, `new_cpu_peripheral`'s (now zero-argument)
  signature, and its use in `exception_unit/execution.rs` (simplified to `vector as u32 *
  INSTRUCTION_SIZE_WORDS` directly, no more OR). Updated all 33 call sites from
  `new_cpu_peripheral(0x0)` to `new_cpu_peripheral()`.
- **found during implementation, beyond the original scope**: three more stale references the
  original finding didn't mention, all updated: a comment in
  `peripheral-cpu/tests/exceptions/reset.rs` ("the reset vector table lives at
  `system_ram_offset|0 = 0`"), a not-yet-exposed-registers TODO entry in
  `sirc-vm/sirc-vm/src/debug_adapter/server.rs`, and a naming-convention example in
  `docs/reference/STYLE.md`'s displacement-vs-offset row (replaced with `program_offset`, a
  still-existing identifier). Also required regenerating the field's line out of 9 examples'
  `.register-dump-expected` files (the `{:#?}`-formatted register dump): `boring`, `faults`,
  `hardware-exception`, `software-exception`, `store-load`, `comprehensive-test`,
  `math-coprocessor-emulation`, `protected-mode-writeback`, `basic-video` (`byte-sieve` has no
  register-dump file, so it was unaffected).
- confidence: high
- status: fixed

### Register renumbering (code-wrong, minor) — FIXED 2026-09-15 — resolves NOOP (`F-cop-3`) cleanly

- category: fact
- claim: the manual previously said `NOOP` assembles to `ADDI[N] sr, #0` and is unprivileged; the
  code faulted on it in protected mode because register field `0x0` named `sr`, a privileged
  register, and the general privilege check (`PRIVILEGED_REGISTERS.contains(&instruction.des)`,
  `execution.rs:21-43`) had no reason to exempt this one specific all-zero encoding.
- resolution (author decision, 2026-09-08): renumber the `RegisterName` enum in
  `peripheral-cpu/src/registers.rs` so `r1` occupies index `0x0` and `sr` moves to `0x7`, with
  `r2`..`r7` shifting down to fill `0x1`..`0x6`.
- fix (applied): reordered `RegisterName` to `R1, R2, ..., R7, Sr, Lh, ...` (implicit discriminants,
  so `R1=0 .. R7=6, Sr=7`), which every symbolic reference (`RegisterName::Sr as u8` in
  `execution.rs`, `fetch_and_decode.rs`, `write_back.rs`; `RegisterName::Sr.to_register_index()` in
  `protected_mode_test.rs`) followed automatically. Two things the original write-up missed and
  this fix also had to update: `Registers`' `Index<u8>`/`IndexMut<u8>` impls
  (`peripheral-cpu/src/registers.rs`) are a second, hand-written `match` on raw index -> field, not
  derived from the enum, and needed the same reorder; and the string-keyed
  `register_name_to_index()` table (confirmed to have zero callers today, but kept in sync to avoid
  a landmine). The toolchain's mnemonic parser (`toolchain/src/parsers/instruction.rs`) maps names
  straight to `RegisterName` variants and needed no change.
- undocumented test-breakage found during implementation: `peripheral-cpu/tests/instructions/common.rs`'s
  `get_register_index_range()`/`get_non_address_register_index_range()` hardcoded the old layout
  (excluding `sr`=0, `ph`/`pl`=14/15) and are consumed by ~80+ call sites as a generic writable ALU
  register via `.set_at_index()`; left alone they would have silently dropped `r1` from coverage and
  started writing arbitrary ALU results into `sr` (now inside the old bound). Fixed by deriving both
  ranges from `RegisterName` symbols instead of literals. Three more independent hardcoded-`0x1`
  literals meant as "r1" (`peripheral-cpu/tests/exceptions/common.rs`, `faults.rs` x2), a
  `load_test.rs`/`store_test.rs` quickcheck `Arbitrary` impl duplicating the same hardcoded-range
  bug with an additional off-by-one `+ 1`, one hardcoded assertion in
  `toolchain/tests/assembler/coprocessor_test.rs` (`COPR r4` parses to raw index `4`), a
  `fetch_and_decode.rs` doctest that set `registers.r4` to exercise raw field `0x4`, and three
  toolchain parser doctests (`arithmetic_immediate.rs`, `arithmetic_register.rs`, `ljmp.rs`)
  asserting raw parsed register indices — all silently wrong post-renumbering, all found only by
  running the full test suite and root-causing each failure, not by static review.
- test coverage added: `all_zero_instruction_word_does_not_fault_in_protected_mode` in
  `peripheral-cpu/tests/instructions/protected_mode_test.rs`, proving the actual fix end-to-end
  (nothing previously did -- `test_empty_memory_is_noop` only tested the decoder in isolation, and
  `direct_status_register_writes_fault_in_protected_mode` tests a different, symbolically-addressed
  instruction that correctly keeps faulting).
- consequence: `NOOP` now assembles to `ADDI[N] r1, #0` (still the all-zero word `0x00000000`, since
  `r1` is now index `0x0`), and it is genuinely unprivileged, since `r1` is an ordinary
  general-purpose register -- no privilege-check special case is needed. This also means erased or
  uninitialized memory (which reads back as all-zero words) continues to execute safely as a no-op.
  Manual chapters (`03-registers.tex`, `17-meta-instructions.tex`, `12-instruction-summary.tex`,
  `appendix-e-quick-reference.tex`) were already at the target (post-renumbering) state and needed
  no changes; the machine-generated `docs/reference/generated/*-format-encodings.tex` fragments were
  regenerated via `examples/reference-encodings`' `make manual-tex` to reflect the new raw hex, and
  `toolchain/tests/assembler/snapshots/*arithmetic_immediate*`/`*arithmetic_register*` insta
  snapshots were re-recorded for the same reason. No example `.bin` needed reassembling -- none are
  committed (all gitignored, rebuilt fresh by `make check`).
- confidence: high
- status: fixed
- ruling: manual stands; `NOOP` no longer faults in protected mode, satisfying Gate 2 I.

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

### `examples/protected-mode-writeback` double-fault crash — FIXED 2026-09-16

- category: fact (not a manual/code mismatch -- an example program left behind by an earlier
  architecture change; found incidentally while sweeping the example matrix for the bundled cleanup
  above)
- claim: none from the manual pass -- `make clean check` in this example crashed with a repeating
  "Double fault! [PrivilegeViolation] raised when a fault was already being serviced" cascade,
  ending in a u32 overflow panic.
- evidence: `git bisect` across the 122 commits since this example was added (`92789a7`, June 2026;
  no `Cargo.lock`/`Cargo.toml` changes in that range, so a clean automated bisect with no
  dependency-refetch risk) isolated the regression to `a5d82c3` ("fix: match fault behaviour with the
  spec in the manual", 2026-09-14) -- well before this session, confirmed by reverting to the
  pre-session commit and reproducing the identical crash there. `a5d82c3` made `PrivilegeViolation`
  retryable (the fault's stored return address is the faulting instruction's own, not the following
  one's), matching the manual's "Retryable" designation and the pattern already used correctly
  elsewhere (`write_back.rs`'s `set_address_register_values`, `check_privilege`'s call site) -- this
  CPU change is correct and intentional, not a bug. Traced (via a tightly-bounded `-vvvv` capture --
  the unbounded `-vv` run was killed after producing 78MB+ with no end in sight) to the example's own
  `privilege_violation_handler`: it clears `ProtectedMode` in the saved SR before `RETE` (correct, to
  get back to supervisor mode) but never redirects the *return address* past the blocked instruction.
  Since the return address is the faulting instruction's own, `RETE` retries it -- now in supervisor
  mode, so the previously-blocked write actually succeeds, corrupting `ph` (confirmed via trace: the
  very next fault's `pc` was `0x34023C`, exactly `(ph=0x1234, pl=~0x23E)` combined -- `0x1234` being
  the literal value the deliberately-blocked `LOAD ph, #0x1234` test case was trying to write).
  `examples/faults/faults.sasm`'s `privilege_violation_fault_handler` already has the exact fix
  (found and applied during this session's earlier `F-exc-3` work) with a comment describing the
  identical mechanism; `protected-mode-writeback.sasm` predates that fix and isn't part of the CI
  example matrix, so nothing caught the drift when `a5d82c3` landed.
- resolution: code-wrong in the example only (fixed); no Rust change
- fix (applied): added the same redirect-past-the-instruction pattern to
  `protected-mode-writeback.sasm`'s shared handler, adapted to a uniform "+2 to the stored return
  address's low word" (skip exactly one instruction) rather than per-call-site labels, since one
  handler serves six single-instruction negative test cases here (`ETFR a, #6` / `ADDI al, #2` /
  `ETTR #6, a`, mirroring `faults.sasm`'s `ETFR a, #6` / `LOAD al, @label` / `ETTR #6, a`).
- consequence: no `.register-dump-expected` regeneration needed -- the fix restores exactly the
  original (pre-`a5d82c3`) end-to-end behavior, so the file committed alongside the example in
  `92789a7` (June 2026) already matched byte-for-byte once this session's earlier, unrelated
  `system_ram_offset` line removal was applied. Confirmed `r1` (success marker) = `0x600D`, `r2`
  (fault count) = `6` (all six negative test cases), `r6` (handler guard) = `0`, matching the file's
  own header comment exactly.
- confidence: high
- status: fixed

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
