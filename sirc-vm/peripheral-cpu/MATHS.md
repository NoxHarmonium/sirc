# Reference integer maths datapath

This is the concrete timing model for the simulator's optional coprocessor 3.
The architecture permits implementation-defined maths latency. This reference
unit uses a small iterative design suitable for a retro implementation, with
one word adder/subtractor, carry/borrow, shifting latches and a sequencer.
These are SIRC timings, not an emulation of a particular historical chip.

No native Rust multiplication, division or remainder operation computes a
hardware result. Native arithmetic appears only in independent test oracles.

## Word operations

Multiply latches two 16-bit operands. For each of 16 bits it conditionally adds
the multiplicand to a 17-bit accumulator, then right-shifts the concatenated
accumulator/multiplier. The accumulator's carry bit shifts into its word and
its low bit shifts into the multiplier. Each bit consumes two CU cycles.

Divide latches a 32-bit dividend and 16-bit divisor. After sign normalization,
the high word starts the remainder and the low word becomes the quotient.
Each of 16 iterations first left-shifts remainder:quotient, then trial-subtracts
the divisor. Borrow selects the original shifted remainder; otherwise the
subtraction result is retained and the quotient's low bit is set. The wide
latch contains at most 17 significant bits. Each bit consumes two CU cycles.

Signed operations normalize magnitudes with word-wise two's complement.
Multiply applies the XOR of operand signs to the final 32-bit product. Divide
applies that sign to the quotient and the dividend's sign to the remainder,
truncating toward zero. The signed quotient limit is 32767 or 32768 according
to its sign; remainder and quotient are committed together only on success.

Division checks zero before overflow. A normalized dividend high word greater
than or equal to the divisor already implies a quotient wider than 16 bits.
These errors finish early. Signed quotients that fit 16 unsigned bits but exceed
the signed limit are detected after iteration. Errors write r3=E|DZ (3) or
E|OV (5), preserving the original r1:r2. Success writes r3=0. CPU flags, r4..r7
and address pairs are preserved for every operation.

## Exact cycle schedule

Count cycle 0 as the issuing COP instruction's first fetch. With no fetch waits:

| Cycles | Action |
| --- | --- |
| 0–5 | Ordinary COPI/COPR instruction |
| 6 | Setup: latch command, inputs and signs; no result write |
| 7 | First normalization word (or idle if unnecessary) |
| 8 | Second normalization word (or idle) |
| 9 | Normalize divisor / latch multiplicand |
| 10 | Operand checks (multiply has no error check) |
| 11–42 | Sixteen two-cycle add/shift or shift/subtract iterations |
| 43 | Signed quotient range check (otherwise idle) |
| 44 | Result low-word sign correction (or idle) |
| 45 | Product high-word / remainder sign correction (or idle) |
| 46 | Atomic architectural r1/r2/r3 write-back |
| 47 | Idle slot padding |
| 48 | Resume PU fetch or dispatch pending trace/interrupt |

Normal multiply/divide and late signed overflow take **48 cycles** from first
COP fetch to resumed fetch. Early divide-by-zero or high-word overflow instead
writes status at cycle 11 and resumes at cycle 12: **12 cycles** total.
Normalization and sign stages remain allocated for unsigned/positive operands,
so ordinary latency is independent of operand bit patterns. This simple fixed
schedule is intentional; future implementations may choose a different latency.
For comparison, the [Motorola 68000 manual](https://www.nxp.com/docs/en/reference-manual/MC68000UM.pdf)
section 8.3 describes operand-dependent multiplication timings; SIRC does not
claim to reproduce that implementation's clocks.

The CU supplies every advancing cycle and owns its six-phase clock. HALT pauses
before phase 0 and resumes the exact pending arithmetic stage; it does not
finish the calculation in the background. Reset discards internal work without
publishing an unfinished result. Hardware interrupts and the issuing instruction's
sampled trace wait for command retirement. A maths status error is successful
command completion, not a CPU fault. Reserved opcodes do fault and cancel trace.

Maths issues no bus requests and keeps SYNC low throughout its dispatch slots.
Instruction-fetch waits extend the issuing instruction normally; there are no
maths data accesses to wait on. The unit never polls the bus or keeps a second
phase counter. Tests cover intermediate datapath state, atomic write-back,
exact cycle traces, HALT, reset and interrupt/trace ownership.
