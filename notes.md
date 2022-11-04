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
Register Direct | xN, yN, zN, aB, sB, pB, sr | LOAD x1, y2
Address register indirect with register displacement | (r, a) | STOR (y1, a), x1
Address register indirect with immediate displacement | (#n, a) | LOAD y1, (#-3, a)

|      | Implied | Immediate | Register Direct | Indirect Immediate | Indirect Register |
| ---- | ------- | --------- | --------------- | ------------------ | ----------------- |
| HALT | 0x00    |           |                 |                    |                   |
| ADDR |         |           | 0x01            |
| SUBR |         |           | 0x02            |
| MULR |         |           | 0x03            |
| DIVR |         |           | 0x04            |
| ANDR |         |           | 0x05            |
| ORRR |         |           | 0x06            |
| XORR |         |           | 0x07            |
| COMP |         |           | 0x08            |
| SJMP | 0x09    |           |                 |
| LJMP | 0x0A    |           |                 |
| BRAN |         | 0x0B      |                 |
| LOAD |         | 0x0C      | 0x0D            | 0x0E               | 0x0F              |
| STOR |         |           |                 | 0x12               | 0x13              |
| WAIT | 0x16    |           |                 |
| RETI | 0x17    |           |                 |
| EXCP |         | 0x18      |                 |
| INOF | 0x19    |           |                 |
| INON | 0x1A    |           |                 |
| BRSR |         | 0x1B      |                 |
| SJSR | 0x1C    |           |                 |
| LJSR | 0x1D    |           |                 |
| RETS | 0x1E    |           |                 |
| LSLR |         |           | 0x1F            |
| LSRR |         |           | 0x20            |
| ASLR |         |           | 0x21            |
| ASRR |         |           | 0x22            |
| ROTL |         |           | 0x23            |
| ROTR |         |           | 0x24            |
| NOOP | 0x25    |           |                 |
| CLSR | 0x26    |           |                 |
| SPLT |         |           | 0x27            |
| JOIN |         |           | 0x28            |

Gaps between LOAD/STOR/WAIT are space to allow more addressing modes in future versions

// TODO: DAMMIT NEED POP/PUSH
