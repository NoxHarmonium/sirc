# Integer square root with optional maths hardware

Run `make clean check`. The same assembled program runs four times: with `math`,
with `none`, with the empty default, and with `dma,math`. It runs in protected
mode in every configuration. The latter combined configuration also verifies
that installing another optional unit does not alter the maths result.

The classic Babylonian/Newton iteration estimates the integer square root:

```
x_next = floor((x + floor(N / x)) / 2)
```

For N = 123456789, start at the upper bound x = 32768. Repeated `DIVU` operations
produce decreasing estimates until the candidate stops decreasing. The program
then uses `MULU` to prove `root² <= N < (root+1)²`, rather than accepting mere
convergence. For this input the floor root is 11111, reached in five divisions.
The chosen input/initial bound keep the intermediate sum and quotients within
16 bits. This is a worked example, not a general square-root routine for every
32-bit input without further bounds/overflow handling.

| Final register | Meaning | Expected value |
| --- | --- | --- |
| r4 | Integer square root | 11111 (`2B67`) |
| r5 | Newton division count | 5 |
| r6 | Verification result | `CAFE` (`DEAD` on failure) |
| r7 | Calls executed by the software shim | 0 with hardware, 11 without |

Before Newton iteration, the program checks a maximal unsigned product, a
carry-heavy division, divide-by-zero and quotient overflow. These four calls,
five Newton divisions and two square-bound multiplies account for the eleven
emulated calls. The final r1:r2 = `075C:1A40` is (11111+1)² = 123476544; r3=0.

## Missing-chip emulation

Missing maths hardware raises invalid opcode before changing the operands.
The supervisor handler reads the rejected command from EU metadata link 7 and
implements MULU with software shifts/adds and DIVU with shifts/subtractions.
It preserves r4..r7 and a in a fixed RAM frame addressed by l. Only r1/r2/r3
return the architectural maths results. Division errors preserve the original
dividend words and return status in r3, without raising another CPU fault.

The handler advances the saved retry PC in EU fault link 6 by two words, then
`RETE` restores the application's protected-mode SR and PC atomically. All
flags altered by the software arithmetic are discarded on return. No optional
maths or DMA instruction is used by the emulator itself, so it works on the
baseline machine. Each `.EQU` and the arithmetic loops are explained in the
source listing.

This illustrative shim supports only MULU/DIVU, uses reserved l/s registers,
and is not reentrant: no interrupts are enabled and RAM is assumed accessible.
A general operating-system emulator would need per-context storage, command
validation, signed operations and memory-fault handling. The existing
[math-coprocessor-emulation](../math-coprocessor-emulation/README.md) example
shows an alternative trampoline and return-gate structure.

Hardware and software fixtures intentionally differ in r7 and exception links.
The emulation path records its last rejected command and adjusted retry PC/SR
in LR7/LR6. Both must agree on arithmetic outputs, root, iteration count and
success marker. The hardware uses the [reference iterative datapath](../../sirc-vm/peripheral-cpu/MATHS.md),
with exact clock tests; software emulation takes the cycles of its actual
instruction stream, rather than pretending to have hardware latency.
