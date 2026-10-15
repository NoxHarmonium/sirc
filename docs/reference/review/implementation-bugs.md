# Implementation bugs

Found during the 2026-09-16 scoped editorial pass. Per `manual-handover.md`'s "Source of Truth"
section, the manual is canonical — these are real Rust-side bugs, not manual errors, discovered
while fact-checking the manual against the implementation.

## Open

### IB-5: Linker only supports `FullAddress` label references on data directives, so explicit `.r`/`.l`/`.u` suffixes panic there

- **Where:** `sirc-vm/toolchain/src/bin/linker.rs:125-134`
- **What:** `.DW @label.l` (or `.r`/`.u`) assembles without error, but the linker panics at link
  time: "Only FullAddress ref type currently supported for data directives." Only the implicit,
  no-suffix form (which resolves to `FullAddress` for a data directive) actually works.
- **Manual side:** no change. Author's call (2026-09-16): log this as a toolchain gap rather than
  scoping Appendix D.1's description of the `.r`/`.l`/`.u` suffixes to instruction operands only.
- **Suggested fix:** implement `LowerWord`/`UpperWord`/`Offset` handling for data directives in the
  linker, matching what already exists for instruction operands.
- **Status:** logged only, not fixed this pass.

## Fixed this pass

### IB-4: `.EQU` constants can't be used as a data directive's value, despite the manual's claim

- **Where:** `sirc-vm/toolchain/src/parsers/data.rs`, `parse_data_type`;
  `sirc-vm/toolchain/src/data/object.rs`, `inject_data_value`.
- **What:** `parse_data_type` was `alt((parse_number, parse_symbol_reference))`, with no branch for
  a `$NAME` placeholder, so `.DW $ANSWER` failed to assemble (`expected '#' ... or expected '@'`).
  `object.rs` already had a `DataType::PlaceHolder` match arm that the parser could never produce
  -- confirmed a half-built, never-finished feature, not a deliberate restriction. The doc's own
  citations for both files were accurate (unlike IB-1/IB-2's, which were stale).
- **Confirmed wider than the original report**: the dead `DataType::PlaceHolder` resolution arm in
  `inject_data_value` also ignored `data.size_bytes` entirely, always writing the full 4-byte
  representation of the resolved `u32` regardless of whether the directive was `.DB`/`.DW`/`.DQ` --
  unlike the sibling `DataType::Value` arm right above it, which truncates per size. This would
  have produced silently wrong bytes for `.DB $NAME`/`.DW $NAME` the moment the parser fix made
  this code path reachable (e.g. `.DB $NAME` with a placeholder value above `0xFF` would write
  garbage into what should be the zero-padded high bytes of that program slot), so it had to be
  fixed alongside the parser gap rather than left for later.
- **Fix:** added a `parse_placeholder` branch to `parse_data_type`, mirroring the existing
  three-branch shape `parse_value` already uses for instruction immediates
  (`sirc-vm/toolchain/src/parsers/instruction.rs`). Made `inject_data_value`'s
  `DataType::PlaceHolder` arm truncate per `data.size_bytes` (`DB_VALUE`/`DW_VALUE`/`DQ_VALUE`),
  matching the `DataType::Value` arm's existing truncation logic exactly. No other call site needed
  changes -- the printer's `DataType::PlaceHolder` arm already round-trips correctly and was
  simply unreachable dead code until now.
- **Test coverage added:** a new `test_build_object_resolves_equ_placeholders_on_data_directives_truncated_per_size`
  in `sirc-vm/toolchain/tests/types/object_test.rs` -- defines three `.EQU` constants whose values
  deliberately exceed their directive's width (except `.DQ`, already the max width) and asserts the
  assembled bytes are correctly truncated per directive, not just passed through. This is also the
  first test coverage in the suite for `$NAME` placeholder resolution at all (none existed for
  instruction operands either).
- **Manual side:** no change needed -- `chapters/appendix-d-examples.tex` already states `.EQU`
  constants are usable "anywhere a `#immediate` operand is accepted," a general claim that already
  covers data directives; confirmed independently, not just trusted from the doc's own note.

### IB-2: Exception-unit reserved opcodes panic instead of behaving as undefined

- **Where:** `sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs`,
  `deconstruct_cause_value`
- **What:** an unimplemented (reserved, `0x2`-`0x8`) exception-unit opcode panicked:
  ```rust
  let op_code = num::FromPrimitive::from_u8(instruction.op_code).unwrap_or_else(|| {
      panic!("Unimplemented op code [{:X}] for exception co-processor", instruction.op_code)
  });
  ```
  Reachable from software: a program can issue `COPI`/`COPR` targeting coprocessor 1 (the
  exception unit) with any op nibble, including one of the 7 reserved values.
- **Confirmed this pass**: the doc's own citations (`06-exceptions.tex:139-140`,
  `16-coprocessor-instructions.tex:702-703`) point at unrelated content (Invalid Opcode Fault, DMA
  faults respectively) and were wrong even before this pass — the manual did **not** need
  correcting. The actually-correct wording already existed, unchanged, in two other places:
  `chapters/06-exceptions.tex:816` ("opcodes 0x2–0x8 are reserved; issuing one is architecturally
  undefined") and `chapters/16-coprocessor-instructions.tex:203-204` plus §16.2 "Model and
  Revision Policy" (reserved operation nibbles within an implemented coprocessor are
  architecturally undefined, not a fault). This was a Rust-only bug.
- **Fix:** replaced the panic with a fallback to `ExceptionUnitOpCodes::None`, the existing
  no-op variant — simpler than mirroring the processing unit's raw-bit-derived ALU behavior, since
  reserved exception-unit nibbles have no comparable sub-structure to preserve. `step`'s
  `ExecutionEffectiveAddressExecutor` phase already does nothing for `None`, and `level` (computed
  alongside `op_code`) is never read on that path, so no other call site needed changes.
- **Test coverage added:** a new unit test,
  `reserved_exception_unit_opcode_behaves_as_a_no_op_instead_of_panicking`, in the existing
  `#[cfg(test)] mod tests` block in the same file — constructs a raw cause value for each of the 7
  reserved nibbles (`0x2`-`0x8`) directly (bypassing `construct_cause_value`, which only accepts
  real `ExceptionUnitOpCodes` variants by design) and asserts `deconstruct_cause_value` resolves
  each to `None` instead of panicking.
- **Manual side:** no change needed (see above) — the manual was already correct.

### IB-3: Interrupt dispatch gives up on the whole cycle if the single highest-priority pending line was disabled after being latched

- **Where:** `sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs`,
  `get_cause_register_value`
- **What:** dispatch found the single highest-priority pending interrupt line, re-tested only
  *that* line's enable bit, and if it had been disabled since being latched, returned without
  dispatching anything — never considering any other pending line, even a lower-priority one that
  was still enabled.
- **Effect:** a hardware interrupt line that was latched while enabled and then masked before its
  turn to dispatch would silently block every other pending interrupt indefinitely, until its own
  pending bit was separately cleared.
- **Confirmed as a bug, not a design choice**, by the author (2026-09-16): no intentional
  "keep it simple" rationale, no comment near the code acknowledging the tradeoff, unlike the
  carefully-documented NMI re-entry special case two lines above it.
- **Fix:** iterate pending interrupt lines from highest to lowest priority, dispatching the first
  one whose enable bit is still set, instead of only ever considering the single highest pending
  bit. See the commit that added this entry for the actual code change.
- **Manual side:** added one clarifying sentence to
  `chapters/06-exceptions.tex` making explicit that a masked pending interrupt does not block
  lower-priority ones (this was always the intended reading, now also true in the implementation).

### IB-1: Segment-overflow trap uses unsigned add for the general displacement, still wrong for negative displacements

- **Where:** `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs`,
  the `MemoryRefDisplacement` arm.
- **What:** `let (displaced, displacement_overflowed) = decoded.ad_l_.overflowing_add(decoded.sr_b_);`
  computed the base-address-plus-displacement using an *unsigned* `u16::overflowing_add`. For a
  negative displacement (e.g. `#-4`, stored as `0xFFFC`), the carry-out this produces is the
  opposite of a genuine signed wrap.
- **Effect:** `STOR (#-4, s), r2` with `al = 0x1000` spuriously raised `SegmentOverflow` when
  `SR.A` was set, even though `0x1000 - 4` does not wrap the low word. Conversely `al = 0x0000`
  with `#-1` — a genuine underflow — did not trap at all.
- **Confirmed wider than the original report**: this shared `displaced`/`displacement_overflowed`
  pair feeds *every* `addr_inc` branch (regular, pre-decrement, post-increment) and every opcode
  in the `0x10..=0x1F` range (`LOAD`, `STOR`, `LDEA`, `LDEL`, immediate- and register-displacement
  forms alike) — not just the general/regular case the original report's example cited. The prior
  fix (`d6b8eb4`, "fix: pre-decrement addressing not faulting on underflow", 2026-09-14) only
  converted the pre-decrement special case's own extra `-1` step to signed-aware logic; the shared
  base+displacement step every branch starts from was untouched, so pre-decrement was only ever
  half-fixed.
- **Fix:** branch on `sr_b_`'s sign (via `cast_signed()`) and use `overflowing_sub` with the
  magnitude (`sr_b_.wrapping_neg()`) for negative displacements instead of routing the raw
  two's-complement bits through `overflowing_add`, mirroring the exact idiom the pre-decrement fix
  already established. Since every `addr_inc` branch starts from this same value, fixing it here
  fixes all three uniformly.
- **Test coverage added:** a new `test_segment_overflow_fault_with_negative_displacement_does_not_fault`
  (general/regular path, `al=0x1000`, displacement `#-4` — must not fault, the exact reported false
  positive) in `sirc-vm/peripheral-cpu/tests/exceptions/faults.rs`. Also found during
  implementation: the existing `test_segment_overflow_fault_with_load` was unknowingly asserting
  the *buggy* behavior — its displacement (`0xCAFE`) is itself negative as a signed value
  (`-13570`), and its old `al=0xFFFE` produces `0xFFFE + (-13570) = 0xCAFC`, which does not
  genuinely wrap. Fixed by changing that test's `al` to `0x0`, a genuine underflow under the
  corrected signed-wrap rule, preserving the test's original intent ("a real wrap faults") rather
  than flipping its assertion.
- **Manual side:** no change needed — `chapters/04-data-representation.tex:162-163`,
  `chapters/08-addressing-modes.tex:236-238`, and `chapters/14-memory-instructions.tex:103-104` all
  already state the correct rule consistently and were left as-is.
