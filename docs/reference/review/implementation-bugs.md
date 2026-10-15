# Implementation bugs

Found during the 2026-09-16 scoped editorial pass. Per `manual-handover.md`'s "Source of Truth"
section, the manual is canonical — these are real Rust-side bugs, not manual errors, discovered
while fact-checking the manual against the implementation.

## Open

### IB-1: Segment-overflow trap uses unsigned add for the general displacement, still wrong for negative displacements

- **Where:** `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:79`
- **What:** `let (displaced, displacement_overflowed) = decoded.ad_l_.overflowing_add(decoded.sr_b_);`
  computes the base-address-plus-displacement using an *unsigned* `u16::overflowing_add`. For a
  negative displacement (e.g. `#-4`, stored as `0xFFFC`), the carry-out this produces is the
  opposite of a genuine signed wrap.
- **Effect:** `STOR (#-4, s), r2` with `al = 0x1000` spuriously raises `SegmentOverflow` when
  `SR.A` is set, even though `0x1000 - 4` does not wrap the low word. Conversely `al = 0x0000`
  with `#-1` — a genuine underflow — does not trap at all.
- **Why it's still open:** commit `d6b8eb4` ("fix: pre-decrement addressing not faulting on
  underflow", 2026-09-14) fixed exactly this bug class, but only for the *auto-decrement* step
  used by pre-decrement addressing (the `decoded.addr_inc == -1` special case, a few lines below
  this one). The general immediate/register displacement calculation at line 79 was never
  converted to the same signed-wrap logic.
- **Manual side:** no change needed. `chapters/04-data-representation.tex:162-163`,
  `chapters/08-addressing-modes.tex:236-238`, and `chapters/14-memory-instructions.tex:104` all
  state the same (correct) rule consistently: the trap should fire only on a genuine signed wrap,
  regardless of the displacement's sign.
- **Suggested fix:** apply the same `overflowing_sub` treatment used for the pre-decrement case to
  the general displacement add — e.g. sign-extend `sr_b_` and compute a signed wrap check, or
  branch on `sr_b_`'s sign and use `overflowing_sub` for negative displacements. Add a regression
  test for a negative displacement with `SR.A` set, alongside the existing
  `test_segment_overflow_fault_with_predecrement_nonzero_does_not_fault`-style tests in
  `sirc-vm/peripheral-cpu/tests/exceptions/faults.rs`.
- **Status:** logged only, not fixed this pass.

### IB-2: Exception-unit reserved opcodes panic instead of behaving as undefined

- **Where:** `sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:69-77`,
  `deconstruct_cause_value`
- **What:** an unimplemented (reserved, `0x2`-`0x8`) exception-unit opcode currently panics:
  ```rust
  // TODO: Define what happens when exception CoP gets an unimplemented opcode
  // category=Hardware
  // Real CPU can't panic. Work out what should actually happen here (probably nothing)
  let op_code = num::FromPrimitive::from_u8(instruction.op_code).unwrap_or_else(|| {
      panic!("Unimplemented op code [{:X}] for exception co-processor", instruction.op_code)
  });
  ```
  The comment already acknowledges this is wrong.
- **Manual side:** per Chapter 16's "Model and Revision Policy" (§16.2): reserved operation codes
  within an *implemented* coprocessor are architecturally undefined, not a fault and certainly not
  a crash. `chapters/06-exceptions.tex:139-140` and `chapters/16-coprocessor-instructions.tex:
  702-703` are being corrected this pass to stop implying these should fault (see the manual diff
  this same commit).
- **Suggested fix:** replace the panic with genuinely undefined-but-non-crashing behavior —
  e.g. treat the reserved opcode as a no-op, or fall through with an arbitrary but harmless
  default, matching how the processing unit's own undocumented ALU opcodes are decoded and
  executed rather than rejected.
- **Status:** logged only, not fixed this pass.

### IB-4: `.EQU` constants can't be used as a data directive's value, despite the manual's claim

- **Where:** `sirc-vm/toolchain/src/parsers/data.rs:36-47`, `parse_data_type`;
  `sirc-vm/toolchain/src/data/object.rs:92-96`
- **What:** `parse_data_type` is `alt((parse_number, parse_symbol_reference))`, with no branch for
  a `$NAME` placeholder, so `.DW $ANSWER` fails to assemble (`expected '#' ... or expected '@'`).
  `object.rs` already has a `DataType::PlaceHolder` match arm that this parser can never produce --
  it looks like a feature that was half-built and never finished, not a deliberate restriction.
- **Manual side:** no change. Author's call (2026-09-16): keep the manual's existing claim that
  `.EQU` constants are usable "anywhere a `#immediate` operand is accepted," including data
  directives, and treat this as the toolchain owing the manual a feature rather than the manual
  overclaiming.
- **Suggested fix:** add a `parse_placeholder` branch to `parse_data_type` so `.DB`/`.DW`/`.DQ` can
  take a `$NAME` operand, reaching the `DataType::PlaceHolder` handling that already exists in
  `object.rs`.
- **Status:** logged only, not fixed this pass.

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
