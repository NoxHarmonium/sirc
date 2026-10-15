# Notes on this exercise

This file records places where the manual didn't answer a question I needed
answered, or where the simulator's observed behavior didn't match what I
derived from `docs/reference/`. Per PLAN.md, everything here was found by
running the assembler/linker/simulator as a black box (`cargo run --bin
assembler|linker|sirc_vm`) and reading their output -- never by reading
`sirc-vm/` source.

## Tooling discoveries (not manual conformance issues, recorded for completeness)

These aren't CPU-behavior questions -- the reference manual documents the
CPU architecture, not the assembler's pseudo-op syntax, so there was nothing
in `docs/reference/` to check them against in the first place. I discovered
them by triggering assembler parse errors deliberately (the parser's error
messages enumerate the exact grammar it expected) and by inspecting linked
`.bin` output byte-for-byte. Recorded here only so the design choices below
make sense, not as findings about the manual.

- Labels are declared `:name` (colon prefix), not `name:` (colon suffix) --
  despite several of the manual's own code examples in
  `docs/reference/chapters/` using the suffix form (e.g. the "While Loop"
  example in Chapter 10, `while_start:`). Both forms appear in the manual's
  examples inconsistently; only the prefix form actually assembles.
- A symbol reference used as an operand is written `@label` with **no**
  leading `#`, even in an immediate-value position (e.g. `LOAD al,
  @crc_buffer`). This is consistent with the notation table in Chapter 11
  ("`@label` ... an absolute low-word address" for non-branch contexts) but
  isn't spelled out with a worked example anywhere.
- Data directives `.DW`, `.DB`, and `.DQ` exist (revealed by the parser's
  error grammar) but none are documented in `docs/reference/`. Empirically,
  each one -- regardless of which of the three is used -- reserves a full
  32-bit (2-word) slot in the linked binary, with the given value
  zero-extended into the *low* word and the high word always zero, unless
  the value is a label reference, in which case the high/low words hold the
  label's full stored-address form (matching the Chapter 4 "24-bit address
  stored as a 32-bit quantity, high word first" layout exactly). This makes
  a single `.DW @label` exactly the right shape for one exception-vector
  table entry, but means `.DW` is not a compact way to lay out an array of
  individual 16-bit words -- each entry costs 2 words, not 1. Given this
  is undocumented assembler behaviour, not architecture, this program
  mostly avoids `.DW` for bulk data and builds working buffers at runtime
  with ordinary `STOR` instead (see below).
- A comment consisting of just `;` with nothing after it fails to parse
  (`error in IsNot`); every comment needs at least one character after the
  `;`.
- `.ORG` takes a bare hex address with no `#` (`.ORG 0x0100`), unlike `.DW`
  which needs one (`.DW #0x0100`).

## Simulator behavior findings

### Leading NOOP requirement (unexplained -- flagged, not resolved)

**What I expected:** Per Chapter 2 ("Six-Stage Execution Phases") and
Chapter 10 ("SIRC-1 has no overlapped pipeline to flush"), execution is a
strict, non-overlapped sequence of whole instructions. Loading a program via
`--program-file` is described by PLAN.md as simply starting execution at
word address `0x000000`. I expected the very first instruction in the file
to execute exactly as written, with no special-casing.

**What actually happened:** If the *first* instruction in the loaded image
is anything other than `NOOP` (all-zero encoding), the very first run
reliably raises a cascading fault storm -- a Bus Fault (`No device was
mapped for address [...]` at a garbage segment like `0x800000` or
`0xC00030`) immediately followed by a Double Fault, which then loops
forever (matching the manual's own statement that double-fault dispatch has
"no limit" -- Chapter 6). This reproduced with several different genuinely
unprivileged first instructions (`LOAD al, #0x0020`; `LOAD ah, #0x0000`),
ruling out a privilege-violation explanation. Prefixing the exact same
program with one extra leading `NOOP` before the real first instruction
made the fault disappear every time I tried it (confirmed independently on
two different minimal test programs). This looks like some kind of
off-by-one in how the very first fetch/decode cycle after a fresh
`--program-file` load is handled, but I could not pin down the mechanism
from black-box testing alone, and the manual gives no reason to expect a
program's first instruction to be treated specially.

**Where I looked:** Chapter 2 (six-phase execution model), Chapter 4
("Memory Ordering" -- explicitly rules out speculative access), Chapter 6
(reset and exception dispatch), Chapter 10 ("no overlapped pipeline").
None of these predict or explain the behavior.

**What I did about it:** Every program in this example now starts with an
unconditional `NOOP` as its literal first instruction, as a workaround.
This is flagged here rather than silently worked around because it's
exactly the kind of manual/simulator mismatch this exercise exists to
surface -- I do not know whether this is a simulator bug or an
undocumented architectural requirement.

### Segment 0 is only backed by the loaded program's own length

**What I expected:** Chapter 8/14 describe addressing and memory access
uniformly; nothing suggested that an address within the current 64K segment
would behave differently depending on whether the loaded program file
happened to be long enough to cover it.

**What actually happened:** A `STOR` to an address well beyond the end of
the `--program-file` image (e.g. word address `0x0300` when the loaded
file was only ~25 instructions / 50 words long) raised a genuine Bus Fault
(`No device was mapped for address [...]`), which -- because no fault
vector table was installed yet at that point in testing -- cascaded into a
double fault. Once the assembled image was padded (via a trailing `.ORG
0x0400` / `.DW #0x0000`, see "Tooling discoveries" above) so that the file
itself covered the target address, the same `STOR` succeeded with no fault.

This is plausibly just the VM's `--program-file` loader mapping exactly as
many words as the file contains for segment 0 (as opposed to a full 64K
segment backed by zero-fill past the image), which is a reasonable,
documented-by-implication choice (Chapter 1's `--segment` flag examples in
the sibling Makefiles all specify an explicit size for *additional*
segments) but is not something `docs/reference/` states one way or the
other for the implicit segment-0 program image. Recorded here as a
clarification I had to discover empirically, not a contradiction.

## CRC-16/XMODEM expected value

Worked out independently (Python, implementing the bit-by-bit algorithm
directly from the manual's stated parameters -- poly `0x1021`, init
`0x0000`, no input/output reflection, no final XOR -- not by looking at any
existing CRC implementation): CRC-16/XMODEM of the byte sequence
`0x00, 0x01, ..., 0x0F` is **`0x513D`** (20797 decimal), i.e. the report
file should end up holding the ASCII string `"20797"`.

## Build continued: stages 1-6 all completed and verified

Picking this exercise back up, stage 1 (CRC-16 core) needed writing from
scratch -- despite the fixed CRC expected-value note above already being in
this file, `crc-report.sasm` at that point only contained the itoa5 test
harness, with no CRC code at all. Below are the new findings from finishing
stages 1-6; everything already recorded above still stands.

### Stage 1 confirmed: CRC-16 core matches the independently-derived value exactly

Implemented the standard bit-by-bit CRC-16/XMODEM algorithm (shift the
accumulator left one bit at a time, conditionally XOR-ing in the polynomial
`0x1021` when the bit shifted out of bit 15 was a 1, after first XOR-ing
each input byte into the top byte of the accumulator) directly from the
manual's ALU/shift chapters -- no multiply/divide needed, so the "no Maths
Unit" restriction never came up. One instruction-set detail worth recording
because it made the inner loop simpler than expected: `CMPI r1, #0` (a
`SUB`-family instruction) sets `N` to bit 15 of `r1` with `r1` itself
unmodified (Chapter 13's flag formula: `N = Rm`, and subtracting 0 leaves
`Rm == r1`), so testing the CRC accumulator's top bit before each
conditional shift-XOR needed no extra register or explicit masking --
`CMPI r1, #0` / `BRAN|NC` was sufficient. Ran this standalone (buffer built at
runtime with `STOR` in a loop, matching the existing `.DW`-avoidance
convention) and read the result back via `--register-dump-file`: **r1 came
back as `0x513d`**, exactly matching the independently-derived expected
value above. No manual/simulator mismatch here -- this stage is a clean
conformance confirmation, not just a workaround.

### Tooling discovery: getting the simulator to exit at all (needed for every `--register-dump-file` dump)

**What I hit:** every program in this exercise ends with an infinite
`BRAN @halt`-style loop (matching the style of the itoa5 test harness from
the earlier session). Running such a program under `sirc_vm` with
`--register-dump-file` simply never returns -- confirmed it hangs
indefinitely (tried `WAIT` too, on the theory that a supervisor-mode wait
with no interrupt source wired up might be treated as terminal by the
simulator; it is not -- `WAIT` just hangs the same way). Killing the process
after a timeout, in both cases, left no dump file on disk at all: the dump
is evidently only written on a clean process exit, never speculatively.

**What resolved it:** running the sibling `examples/boring` example (via its
own `Makefile`, as explicitly permitted -- I did not read `boring.sasm`
itself, only its build output) showed that it exits cleanly and produces
this log line: `Processor asserted simulation aborted (e.g. COP 0x14FF).
This type of error exits with code zero for testing purposes.` This is a
raw `COPI #0x14FF` call -- coprocessor 1 (exception unit), operation nibble
`0x4`. Chapter 16 explicitly states "Exception-unit operation nibble ...
operation nibbles `0x2`--`0x8` are architecturally undefined," so the
manual makes no promise about what this does; the simulator has evidently
implemented a specific, deliberate behavior for operand `0xFF` on that
undefined nibble anyway, purely as a testing hook. Using `COPI #0x14FF` as
the final instruction in every program in this exercise (instead of an
infinite loop) made every run exit cleanly and produce a populated register
dump. I'm recording this as a tooling discovery rather than a manual
mismatch, the same way the `.DW`/label-syntax findings above are recorded:
the manual explicitly disclaims this opcode range as undefined, so the
simulator choosing to do something specific with one value in that range
isn't a violation of anything the manual asserts -- but it's still
undocumented behavior this exercise ended up depending on for basic tooling
(without it, `--register-dump-file` is unusable for any program that
doesn't otherwise terminate, and nothing in `docs/reference/` describes any
other way to end a simulation run cleanly from within a program).

### Stage 3 (composition) and stage 4 (bare trap round-trip): both confirmed working exactly as expected

Feeding the CRC-16 result straight into the existing, already-tested itoa5
routine and reading the 5 ASCII digits back out of memory into registers
produced `r3..r7` = `0x32, 0x30, 0x37, 0x39, 0x37` = `"20797"`, matching
expectations with no register-clobbering surprises between the two
routines.

For the bare trap round-trip, the vector table was populated using exactly
the mechanism the earlier session's `.DW`-with-label-operand discovery
predicted would work: a `BRAN` immediately after the leading `NOOP` skips
over the low word addresses, then `.ORG 0x0100` (word address `0x80 * 2`,
matching Chapter 6's vector-table layout) followed by a single
`.DW @monitor_entry` writes the complete 2-word stored-address vector entry
in one directive. A trivial handler (`LOAD r6, #0x2222` / `RETE`) reached
via `EXCP #0x80` from the main program left `r6 == 0x2222` on return exactly
as expected, with every other register (the CRC/itoa results already in
r3, r4, r5, r7) undisturbed. This is a second clean conformance
confirmation: `.ORG` correctly padded the large gap between the early
`BRAN` and word address `0x100` (the assembled file grew to cover it
without any fault or corruption), and `EXCP`/vector-fetch/`RETE` all
behaved exactly as Chapter 6 and Chapter 16 describe.

### Stages 5-6 (real monitor, full integration): confirmed working exactly as expected, including the protected-mode privilege boundary

The finished program (`crc-report.sasm` / `monitor.sasm`) enters protected
mode once, early, via `ORRI sr, #0x0100` (executed while still in
supervisor mode, per the manual's own worked example in Chapter 5), and
never issues a direct write to `ah` again anywhere in the main program from
that point on -- only `al` (unprivileged) and the monitor's own trap-routed
writes touch address-register state afterward. The monitor
(`monitor.sasm`) implements a two-call convention documented in a comment
above it: `EXCP #0x80` ("write_word": `r1` = word offset into the
`report_file` segment, `r2` = value to write) and `EXCP #0x81` ("halt":
routes the `COPI #0x14FF` testing hook above through a trap too, purely so
the main program never has to leave protected mode even to end the
simulation).

Running the full integration (`make inspect`) produced
`00 32 00 30 00 37 00 39 00 37 ...` at the start of `report_file.bin` --
i.e. the ASCII bytes `"20797"`, one per word (high byte `0x00`, low byte
the digit), exactly matching the independently-derived expected CRC value
from the top of this file. Reran `make inspect` a second time from a clean
`report_file.bin` and got byte-for-byte identical output.

Every privilege-model behavior exercised here matched the manual exactly,
with no surprises: `ORRI sr, #0x0100` from supervisor mode entered
protected mode as described; `EXCP #0x80`/`#0x81` (both `>= 0x60`) were
accepted from protected mode with no privilege fault; exception entry
automatically cleared `SR.P` (confirmed via the final register dump, which
shows `sr = 0x2000` -- `EA` set, `P` clear -- because the dump was taken
mid-handler, inside `monitor_halt`, at the exact instant the simulation-abort
hook fired); the monitor's direct `LOAD ah, #0x00E0` succeeded because the
handler runs in supervisor mode; and `RETE` correctly restored the
caller's saved (protected-mode) status register between each of the five
`EXCP #0x80` calls, since the digit values loaded into `r3`-`r7` before the
write loop survived all five round trips undisturbed. **No manual/simulator
mismatch was found anywhere in the trap/privilege portion of this
exercise** -- worth recording explicitly, since a clean conformance result
is as much a legitimate outcome of this exercise as a flagged mismatch.

The "leading NOOP" workaround (see above) continued to be necessary and
sufficient throughout -- no new instance of the fault-storm behavior showed
up anywhere in the much larger, control-flow-heavier final program (large
forward `.ORG` jump, dynamic `EXCP`/`RETE` trap dispatch, mode switches),
which is consistent with that anomaly being specific to the very first
fetch cycle after a `--program-file` load rather than a general hazard with
jumps or traps elsewhere in a program.
