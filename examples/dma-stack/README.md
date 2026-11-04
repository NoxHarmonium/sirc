# Saving and restoring a register window with DMA

Run `make clean check`. One binary runs with DMA enabled, explicitly disabled,
and omitted (the default). The latter two use a missing-chip software fallback.

A routine that needs all seven general registers can save them together:

```asm
DMAW s, #-7    ; reserve seven words below s, then store r1..r7
; work which may overwrite r1..r7
DMAR s, #7     ; restore r1..r7, then release the seven words
```

This is a descending stack: the initial pointer is `0001:0200`, the push moves
it to `0001:01F9`, and the pop returns it to `0001:0200`. The frame is:

| Word address | Saved register |
| --- | --- |
| `0001:01F9` | r1 |
| `0001:01FA` | r2 |
| `0001:01FB` | r3 |
| `0001:01FC` | r4 |
| `0001:01FD` | r5 |
| `0001:01FE` | r6 |
| `0001:01FF` | r7 |

The negative DMA count decrements the pointer once, before the burst, and
transfers ascending addresses. Individual pre-decrement stores in r1..r7 order
would produce the opposite layout. Equivalent scalar code reserves seven words
first and stores ascending offsets, or pre-decrements while storing r7..r1.

A seven-word burst takes 18 cycles without waits, including the issuing COP
instruction. Seven scalar STORs (or LOADs) take 42 cycles. Device waits add to
either approach; the DMA unit is optional and these instructions are privileged.
DMA does not save SR or the address pairs, and pending interrupts wait until the
whole command completes. This is a register-save idiom, not a complete calling
convention or interrupt-handler prologue.

The example checks every saved stack slot, overwrites all seven registers,
restores them and checks every value and the final stack pointer. Only then does
it repurpose r5 as the execution-path report (0 hardware, 1 software), r6 as the
result (`CAFE` success, `DEAD` failure), and r7 as the debug exit request.

The fixed-command fallback handles only `DMAW s,#-7` and `DMAR s,#7`. It reserves
l for scratch outside the stack, preserves a there, and never borrows r1..r7 as
handler scratch. Control flow stays inline so l is available for that purpose;
a real calling convention must arrange its return address and handler scratch
separately. No interrupts are enabled and this illustrative handler is not
reentrant. It assumes valid mapped addresses without overflow. It handles a
missing chip, not recovery after a partially completed DMA bus fault.

The expected dumps differ in r5 and the fault links: software leaves the last
rejected command (`2887`) in LR7 and its adjusted retry PC/SR in LR6. Hardware
never enters the fault handler. Both must finish with r6=`CAFE` and s=`0001:0200`.
