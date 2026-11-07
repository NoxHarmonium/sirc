# SIRC VM hardware configuration

Optional coprocessors are selected when starting the VM. The default machine
has only the mandatory processing and exception units. Missing optional units
raise invalid opcode so programs can provide software emulation.

```sh
cargo run --no-default-features --bin sirc_vm -- \
  --program-file program.bin --optional-coprocessors dma
```

`--optional-coprocessors` accepts a comma-separated list: `dma`, `math`, or
`dma,math`. `none` explicitly selects the default machine and must be used alone.
Duplicate or unknown units are errors. Model-number aliases are not defined yet.

The maths chip implements user-callable MULU/MULS/DIVU/DIVS with iterative
word operations, not native host multiplication/division. Ordinary calls take
48 cycles; early division errors take 12, excluding instruction-fetch waits.
See the [reference datapath and exact cycle schedule](peripheral-cpu/MATHS.md).
The [square-root example](../examples/maths-square-root/README.md) runs the same
protected-mode program using hardware or a missing-chip software emulator.

The DMA chip provides supervisor-only DMAR, DMAW and DMAT. It shares the CU's
six-phase clock, bus acknowledgment/error handling and interrupt/trace ownership.
HALT pauses at slot boundaries; reset aborts active work. Faults preserve already
committed words and the manual's pointer commit rules, without rolling back or
automatically retrying an entire command.

See [dma-copy](../examples/dma-copy/README.md) for a single binary checked with
both the DMA chip and a missing-chip software fallback.
The [dma-stack](../examples/dma-stack/README.md) example demonstrates DMAR/DMAW
as a batch register push/pop, also checked with and without the chip.

## Embedding and adding optional hardware

Rust callers can use `new_cpu_peripheral_with_config(CpuConfig { ... })` with a
list of boxed `OptionalCoprocessor` implementations, such as
`peripheral_cpu::coprocessors::dma::DmaCoprocessor::default()` or
`peripheral_cpu::coprocessors::maths::MathsCoprocessor::default()`. The existing
`new_cpu_peripheral()` constructor retains the empty default configuration.
IDs 0 and 1 are reserved for the mandatory units; duplicate IDs and IDs outside
2–15 are rejected.

These plugins are compiled-in hardware implementations, installed once at CPU
construction. Each implements `start`, `step`, `complete_access`,
`is_finished` and `reset`. The CU calls completion only on successful ACK,
handles bus faults itself, and resets the unit on abort. Implementations share
the CU phase clock and held-request mechanism rather than owning another clock
or polling the bus. There is no dynamic library loader.
