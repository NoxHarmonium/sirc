# Random Rust Code Snippets

## Printing Hex

println!("Some u32 value: 0x{:08x}", 0x12345678);

## Interrupts

The SERC based CPU is based mainly on the 6502 (http://wilsonminesco.com/6502interrupts/) but with two IRQ lines,
one of which can take priority over the other.

## Reference FPGA

https://www.crowdsupply.com/radiona/ulx3s#products

## 6502 Simulator

http://visual6502.org

## Assembly Syntax

<instruction>[|<condition>] [<target_address>,] [<source_address>]

LOAD|EQ y1, (x1, a)

## Addressing Modes

Used with LOAD/STOR

Registers r = x1, x2, x3, y1, y2, y3, z1, z2, z3 (d), ah*, al (a), sh*, sl (s), ph*, pl (p), sr*

Address registers a = a, p, s

\* ah, sh, ph and sr are considered privileged, when system mode bit is not set, certain access patterns may trigger a privilege violation

Implied | - | HALT
Immediate | #n | LOAD x1, #123
Register direct | xN, yN, zN, aB, sB, pB, sr | LOAD x1, y2
Register range direct | rN->rM, | STMR (a), x1->z1
Address register direct | a | LDEA a, (x1, a)
Address register indirect | (a) | STOR (s), x1
Address register indirect with register displacement | (r, a) | STOR (y1, a), x1
Address register indirect with immediate displacement | (#n, a) | LOAD y1, (#-3, a)

\* Address register indirect doesn't have dedicated opcodes because it is equivalent to (#0, a). The assembler will alias this automatically.

#### Valid Source Operands

Source operands are on the right side of the arguments. Destination operands are always first.

|      | Implied | Immediate | Register Direct | Address Direct | Indirect Immediate | Indirect Register | Register Range |
| ---- | ------- | --------- | --------------- | -------------- | ------------------ | ----------------- | -------------- |
| HALT | 0x00    |           |                 |                |                    |
| NOOP | 0x01    |           |                 |
| WAIT | 0x02    |           |                 |
| RETI | 0x03    |           |                 |
| RETS | 0x04    |           |                 |
| ADDR |         |           | 0x05            |
| ADDC |         |           | 0x06            |
| SUBR |         |           | 0x07            |
| SUBC |         |           | 0x08            |
| MULR |         |           | 0x09            |
| DIVR |         |           | 0x0A            |
| ANDR |         |           | 0x0B            |
| ORRR |         |           | 0x0C            |
| XORR |         |           | 0x0D            |
| LSLR |         |           | 0x0E            |
| LSRR |         |           | 0x0F            |
| ASLR |         |           | 0x10            |
| ASRR |         |           | 0x11            |
| ROTL |         |           | 0x12            |
| ROTR |         |           | 0x13            |
| COMP |         |           | 0x14            |
| PUSH |         |           | 0x15            |
| POPR |         |           | 0x16            |
| EXCP |         | 0x17      |                 |
| SJMP |         | 0x18      |                 |                |
| SJSR |         | 0x19      |                 |                |
| BRSR |         | 0x1A      |                 |
| BRAN |         | 0x1B      |                 |
| LJMP |         |           |                 | 0x1C           |
| LJSR |         |           |                 | 0x1D           |
| LDEA |         |           |                 |                | 0x1F               | 0x20              |
| LDMR |         |           |                 |                | 0x21               |                   |                |
| LOAD |         | 0x21      | 0x23            |                | 0x24               | 0x25              |
| STOR |         |           |                 |                | 0x26               | 0x27              |
| STMR |         |           |                 |                |                    |                   | 0x28           |

42 Total Opcodes

# TODO: Make sure privileged/ non privileged SP bits are in separate bytes for easier checks

# TODO: How to propagate carry (new instruction or just 6502 it and only provide ADD with carry)

# TODO: address register can be target for lea only

# TODO: Load/store single byte?
