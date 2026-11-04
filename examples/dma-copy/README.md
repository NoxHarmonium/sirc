# Optional DMA hardware and software fallback

Run `make clean check`. It assembles one binary and executes it three times:

- `--optional-coprocessors dma` executes `DMAT a, l, #8` in hardware.
- `--optional-coprocessors none` takes the missing-coprocessor invalid-opcode fault and copies the words in its handler.
- Omitting the option verifies that the default machine also has no DMA chip.

Both paths check the advanced source/destination pointers and all eight copied
words. R6 = `0xCAFE` signals success (`0xDEAD` signals failure). R5 = 0 for
hardware or 1 for fallback proves which path ran. The eight-word transfer also
exercises a full seven-register chunk followed by a one-word chunk.

The two expected dumps intentionally differ: hardware leaves r4 = `0x4444`
from its first chunk, while the handler uses r4 = `0x0808` to save the advanced
source pointer. The fallback also leaves the invalid-opcode command in LR7 and
the adjusted return PC/SR in LR6. These are architectural side effects of that
path, rather than a requirement that emulation preserve DMA's scratch window.

The fallback is deliberately limited to this example's fixed `DMAT #8` call.
It is not a general DMA emulator or a fault-recovery routine: it assumes these
mapped addresses do not fault or wrap. Missing hardware faults before any
transfer, so it can safely copy once, advance the saved retry PC by two words,
and return with `RETE`. A partly completed hardware DMA fault would need a
separate recovery policy and cannot safely use this handler.

DMAR/DMAW burst details are covered by the CPU regressions in
`sirc-vm/peripheral-cpu/tests/exceptions/dma/`: all three address pairs, both
directions, zero/short/full bursts, COPI/COPR timing, held requests, partial
bus faults and segment overflow. This example concentrates on running the same
DMAT program with different hardware configurations.
