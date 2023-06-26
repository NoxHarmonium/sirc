// Instruction (32 bit)
//
// Instruction formats:
//
// Implied: (e.g. NOOP)
// 6 bit instruction identifier (max 64 instructions)
// 22 bit reserved
// 4 bit condition flags
//
// Immediate: (e.g. BRAN #-3)
// 6 bit instruction identifier (max 64 instructions)
// 4 bit register identifier
// 16 bit value
// 2 bit address register a, p or s (if any)
// 4 bit conditions flags
//
//
// Immediate with shift: (e.g. ADDI r1, #2, ASL #1)
// 6 bit instruction identifier (max 64 instructions)
// 4 bit register identifier
// 8 bit value
// 8 bit shift (1 bit operand, 3 bit shift type, 4 bit shift count)
// 2 bit address register a, p or s (if any)
// 4 bit conditions flags
//
// Register: (e.g. ADD r1, r2)
// 6 bit instruction identifier (max 64 instructions)
// 4 bit register identifier
// 4 bit register identifier
// 4 bit register identifier (if any)
// 8 bit shift (1 bit operand, 3 bit shift type, 4 bit shift count)
// 2 bit address register a, p or s (if any)
// 4 bit condition flags
//
// Segment 0x00 is reserved by the CPU for parameters.
// The other segments are flexible because they are defined in this hardcoded segment.
//
// 0x00 0000 : DW Initial PC
// 0x00 0002 : DW System SP
// 0x00 0004 : DW Base System RAM (for storing in interrupt vectors etc.)
// ...

|      | Immediate | Register Direct | Indirect Immediate | Indirect Register | Post Increment | Pre Decrement | Implied | Immediate (Short+Shift) |
| ---- | --------- | --------------- | ------------------ | ----------------- | -------------- | ------------- | ------- | ----------------------- |
| ADDI | 0x00      |                 |                    |                   |                |               |
| ADCI | 0x01      |                 |                    |                   |                |               |
| SUBI | 0x02      |                 |                    |                   |                |               |
| SBCI | 0x03      |                 |                    |                   |                |               |
| ANDI | 0x04      |                 |                    |                   |                |               |
| ORRI | 0x05      |                 |                    |                   |                |               |
| XORI | 0x06      |                 |                    |                   |                |               |
| CMPI | 0x07      |                 |                    |                   |                |               |
| TSAI | 0x08      |                 |                    |                   |                |               |
| TSXI | 0x09      |                 |                    |                   |                |               |
| SHFI | 0x0A      |                 |                    |                   |                |               |
| BRAN | 0x0B      |                 |                    |                   |                |               |
| BRSR | 0x0C      |                 |                    |                   |                |               |
| SJMP | 0x0D      |                 |                    |                   |                |               |
| SJSR | 0x0E      |                 |                    |                   |                |               |
| EXCP | 0x0F      |                 |                    |                   |                |               |         |
| LDEA |           |                 | 0x10               | 0x11              |                |               |         |
| LJMP |           |                 | 0x12               | 0x13              |                |               |         |
| LJSR |           |                 | 0x14               | 0x15              |                |               |         |
| LOAD | 0x16      | 0x17            | 0x18               | 0x19              | 0x1D           |               |         |
| STOR |           |                 | 0x1A               | 0x1B              |                | 0x1F          |         |
| ADDI |           |                 |                    |                   |                |               |         | 0x20                    |
| ADCI |           |                 |                    |                   |                |               |         | 0x21                    |
| SUBI |           |                 |                    |                   |                |               |         | 0x22                    |
| SBCI |           |                 |                    |                   |                |               |         | 0x23                    |
| ANDI |           |                 |                    |                   |                |               |         | 0x24                    |
| ORRI |           |                 |                    |                   |                |               |         | 0x25                    |
| XORI |           |                 |                    |                   |                |               |         | 0x26                    |
| CMPI |           |                 |                    |                   |                |               |         | 0x27                    |
| TSAI |           |                 |                    |                   |                |               |         | 0x28                    |
| TSXI |           |                 |                    |                   |                |               |         | 0x29                    |
| SHFI |           |                 |                    |                   |                |               |         | 0x2A                    |
| BRAN |           |                 |                    |                   |                |               |         | 0x2B                    |
| BRSR |           |                 |                    |                   |                |               |         | 0x2C                    |
| SJMP |           |                 |                    |                   |                |               |         | 0x2D                    |
| SJSR |           |                 |                    |                   |                |               |         | 0x2E                    |
| EXCP |           |                 |                    |                   |                |               |         | 0x2F                    |
| ADDR |           | 0x30            |                    |                   |                |               |
| ADCR |           | 0x31            |                    |                   |                |               |
| SUBR |           | 0x32            |                    |                   |                |               |
| SBCR |           | 0x33            |                    |                   |                |               |
| ANDR |           | 0x34            |                    |                   |                |               |
| ORRR |           | 0x35            |                    |                   |                |               |
| XORR |           | 0x36            |                    |                   |                |               |
| CMPR |           | 0x37            |                    |                   |                |               |
| TSAR |           | 0x38            |                    |                   |                |               |
| TSXR |           | 0x39            |                    |                   |                |               |
| SHFR |           | 0x3A            |                    |                   |                |               |
| RETS |           |                 |                    |                   |                |               | 0x3B    |
| NOOP |           |                 |                    |                   |                |               | 0x3C    |
| WAIT |           |                 |                    |                   |                |               | 0x3D    |
| RETE |           |                 |                    |                   |                |               | 0x3E    |

NUL
LSL
LSR
ASL
ASR
RTL
RTR

SHFR r1, r2, r3, LSL #3
