# crc-report: a manual-conformance exercise

## Purpose

This example is not primarily a demo -- it's a conformance check. It is being
written by an agent working **only from `docs/reference/` (the SIRC-1 CPU
Reference Manual)**, with instructions not to read anything under `sirc-vm/`
(the Rust implementation of the assembler, linker, or simulator). The goal is
to see whether a program written strictly from the manual's own description
of the architecture behaves, when actually run on the simulator, the way the
manual says it should. Any mismatch between "what the manual says should
happen" and "what the simulator actually does" is exactly the kind of finding
this exercise exists to surface.

**Rule for whoever (human or agent) is implementing this:** you may read
`docs/reference/**` freely. Do not read any file under `sirc-vm/` -- not to
understand an instruction's behavior, not to check a calling convention, not
to debug a failure. You *should* actually run the assembler/linker/simulator
via the `Makefile`/`cargo run` commands documented below (and in the sibling
`examples/*/Makefile` files, which you may look at purely as build-recipe
examples, not for their `.sasm` content) -- observing the simulator's
black-box behavior and comparing it against what you expected from the
manual is the entire point. If you get stuck on something the manual doesn't
answer (a missing detail, an apparent contradiction, or simulator behavior
that doesn't match your reading of the manual and you can't tell why), **stop
and write it up in `NOTES.md` in this directory rather than guessing or going
around the manual to find out.** A clearly-written "I got stuck here, and
here's why" is a more valuable outcome than a program that happens to work
for unclear reasons.

## This CPU model has no Maths Unit or DMA Unit

The manual documents the Integer Maths Unit (coprocessor 3, `MULU`/`MULS`/
`DIVU`/`DIVS`) and the DMA Unit (coprocessor 2, `DMAR`/`DMAW`/`DMAT`) as
*optional* -- present on some SIRC-1 models, absent on others. This
particular model does not implement either one. Per the manual, executing
one of those instructions on a model that lacks the coprocessor raises an
Invalid Opcode Fault (Chapter 6).

You have two legitimate ways to handle this, and either is fine:

- **Work around it:** simply don't use those instructions. The plan below
  already does this for the itoa step (ordinary shift/subtract division).
- **Emulate it:** the manual explicitly describes Invalid Opcode Fault as a
  forward-compatibility mechanism for exactly this situation -- "programs
  written for a future coprocessor... will trap on older hardware, and the
  handler can emulate the operation in software" (Chapter 6). If you'd
  rather exercise that path, you could issue e.g. `MULU`/`DIVU` anyway, let
  it fault, and have a fault handler emulate the operation before returning
  with `RETE` (see the retryable-fault and `ETFR`/`ETTR` material in
  Chapter 6, and the Invalid Opcode Fault entry specifically).

Whichever you pick, this program should only require coprocessor 0
(processing unit) and coprocessor 1 (exception unit), both of which the
manual says are present on every SIRC-1 model.

## What the program does

1. Compute a CRC-16/XMODEM checksum (polynomial `0x1021`, initial value
   `0x0000`, no input/output reflection, no final XOR) over a fixed 16-byte
   buffer containing the byte values `0x00` through `0x0F` in order.
2. Convert the resulting 16-bit CRC value to a 5-character, zero-padded
   decimal ASCII string (e.g. the value `1234` becomes `"01234"`; `65535`
   becomes `"65535"`). Do this with ordinary shift/subtract division, or by
   emulating `DIVU` yourself via a fault handler as described above -- see
   "This CPU model has no Maths Unit or DMA Unit" for why `MULU`/`MULS`/
   `DIVU`/`DIVS`/`DMAR`/`DMAW`/`DMAT` aren't available directly.
3. Write those 5 ASCII bytes into a separate memory segment (details below)
   by making one or more calls into a small supervisor-mode "monitor"
   routine reached via `EXCP`, not by touching that segment's address
   register directly. The main program should run in **protected mode**
   throughout (see the manual's Chapter 3 privilege model) -- it must not be
   able to write the destination segment's address-register high word
   itself. That's the actual point of routing this through a trap: a
   protected-mode program architecturally cannot reach another segment
   without a supervisor routine doing it on its behalf.

## Memory layout

- The main program is loaded at segment `0x00` (word address `0x000000`),
  same as every other example -- this is fixed by the VM's `--program-file`
  loading behavior, not something you need to set up yourself.
- A second segment, referred to here as the "report file," lives at segment
  `0xE0` (word address `0x00E00000`) and is backed by a real host file
  (`report_file.bin`) via the simulator's `--segment` flag -- already wired
  up in this directory's `Makefile`. Anything the program writes there will
  be visible in `report_file.bin` after the run. Write the 5 ASCII digit
  bytes starting at word offset `0` within that segment.

## Suggested build order (please work through these roughly in order, and
## get each one actually running before moving to the next -- don't try to
## write the whole thing in one pass)

1. **CRC-16 core.** Write just the checksum routine, operating on the fixed
   16-byte buffer (put it in the main program's own segment as ordinary
   data). Leave the 16-bit result in a register at the end. Run it and note
   what value comes out (`make debug` or add a way to dump the register --
   see `--register-dump-file` in the sibling examples' Makefiles for the
   pattern). Work out by hand (or however you like, just not by reading the
   Rust source) what the CRC-16/XMODEM of `{0x00..0x0F}` *should* be, and
   compare.
2. **itoa core.** Separately, write the shift-subtract decimal-conversion
   routine against a couple of known fixed 16-bit test inputs of your
   choosing (worth trying at least one small value, one requiring all 5
   digits, and `0` itself). Verify the ASCII bytes it produces land in
   memory where you expect.
3. **Compose 1 + 2.** Feed the CRC routine's output into the itoa routine.
   No segment/trap involvement yet -- just confirm the two pieces work
   together and nothing about combining them (register clobbering, flag
   side effects) breaks either one.
4. **Bare trap round-trip.** Before wiring up the real monitor, get the
   exception vector table set up and prove a minimal `EXCP` call to a
   user-vector handler (vector `>= 0x60`, per the manual's privilege rules
   for software exceptions) actually reaches your handler and returns
   correctly via `RETE`. Keep this step trivial -- e.g. the handler just
   increments a register -- so a mistake here is easy to isolate from
   mistakes in the real monitor logic later.
5. **Real monitor + cross-segment write.** Extend the trap handler into the
   actual monitor: define your own small register-based calling convention
   (the manual explicitly leaves calling conventions out of scope, so this
   part is your own design -- document whatever you choose in a comment
   above the handler) for "write this byte to this offset in the report
   segment," and have it perform the actual privileged write. Test it with
   a couple of manual calls with fixed values before hooking it up to the
   real data.
6. **Full integration.** Wire stages 1-3's output into stage 5's monitor
   calls, writing the 5 ASCII digits into the report segment. Run `make
   inspect` and confirm `report_file.bin` starts with the 5 digit bytes you
   expect.

## Verification

`make inspect` runs the program and hexdumps the start of `report_file.bin`.
There is **deliberately no committed `expected.hex`** for this example --
figuring out what the correct output *should* be from the manual (and your
own arithmetic) is part of the exercise, not something to diff against. Once
you're done, report what you expected at each stage above, what the
simulator actually produced, and flag anything that didn't match (or
anything the manual didn't give you enough information to be sure about) in
`NOTES.md`.
