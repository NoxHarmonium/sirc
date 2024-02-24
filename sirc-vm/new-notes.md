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
| LOAD | 0x07      |                 |                    |                   |                |               |
| CMPI | 0x0A      |                 |                    |                   |                |               |
| TSAI | 0x0C      |                 |                    |                   |                |               |
| TSXI | 0x0E      |                 |                    |                   |                |               |
| COPI | 0x0F      |                 |                    |                   |                |               |         |
| STOR |           |                 | 0x10               | 0x11              |                | 0x13          |         |
| LOAD |           |                 | 0x14               | 0x15              | 0x17           |               |         |
| LDEA |           |                 | 0x18               | 0x19              |                |               |         |
| BRAN |           |                 | 0x1A               | 0x1B              |                |               |
| LJSR |           |                 | 0x1C               | 0x1D              |                |               |         |
| BRSR |           |                 | 0x1E               | 0x1F              |                |               |
| ADDI |           |                 |                    |                   |                |               |         | 0x20                    |
| ADCI |           |                 |                    |                   |                |               |         | 0x21                    |
| SUBI |           |                 |                    |                   |                |               |         | 0x22                    |
| SBCI |           |                 |                    |                   |                |               |         | 0x23                    |
| ANDI |           |                 |                    |                   |                |               |         | 0x24                    |
| ORRI |           |                 |                    |                   |                |               |         | 0x25                    |
| XORI |           |                 |                    |                   |                |               |         | 0x26                    |
| LOAD |           |                 |                    |                   |                |               |         | 0x27                    |
| CMPI |           |                 |                    |                   |                |               |         | 0x2A                    |
| TSAI |           |                 |                    |                   |                |               |         | 0x2C                    |
| TSXI |           |                 |                    |                   |                |               |         | 0x2E                    |
| COPI |           |                 |                    |                   |                |               |         | 0x2F                    |
| ADDR |           | 0x30            |                    |                   |                |               |
| ADCR |           | 0x31            |                    |                   |                |               |
| SUBR |           | 0x32            |                    |                   |                |               |
| SBCR |           | 0x33            |                    |                   |                |               |
| ANDR |           | 0x34            |                    |                   |                |               |
| ORRR |           | 0x35            |                    |                   |                |               |
| XORR |           | 0x36            |                    |                   |                |               |
| LOAD |           | 0x37            |                    |                   |                |               |         |
| CMPR |           | 0x3A            |                    |                   |                |               |
| TSAR |           | 0x3C            |                    |                   |                |               |
| TSXR |           | 0x3E            |                    |                   |                |               |
| COPR |           | 0x3F            |                    |                   |                |               |         |                         |

NUL
LSL
LSR
ASL
ASR
RTL
RTR

SHFR r1, r2, r3, LSL #3

NOOP is pseudo instruction -> write register to itself
LJMP -> LDEA p, x
SHFI/SHFR -> ADD r #0, (with SR source set to shift)
RETS -> LDEA p, l

Exception handling is done by a coprocessor
The operand for the COPI/COPR instructions is made up of the following:
4 bits cooprocessor ID (0x1 is exception unit)
4 bits coprocessor opcode (for exception unit 0x9 is WAIT and 0xA is RETS)
8 bits argument (e..g. the trap number for exception handling)

EXCP -> COPI #0x11FF
WAIT -> COPI #0x1900
RETE -> COPI #0x1A00

# 0x1X Instructions

These are the memory access and branch instructions which are a bit special since they aren't just an operation on a register.

The instructions at 0x10-0x1F follow a pattern to (hopefully) simplify the decoder.

| 7   | 6   | 5   | 4        | 3/2                                                                | 1                                        | 0                          |
| --- | --- | --- | -------- | ------------------------------------------------------------------ | ---------------------------------------- | -------------------------- |
| ?   | ?   | ?   | Always 1 | 00 = Store 01 = Load 10 = Load Address 11 = Load Address With Link | 0 = Both Registers 1 = Only Low Register | 0 = Immediate 1 = Register |

E.g.

LDEA (LONG JUMP) Immediate would be:
Load Address: 10
Both Registers: 0
Immediate: 0

= 0x18

Where as LJSR Immediate would be:
Load Address with Link: 11
Both Registers: 0
Immediate: 0

= 0xC

The second bit is used to distinguish between operations that write to both registers in an address register pair, vs ones that only write to the lower register.
Why do we need instructions that only write to the lower register? Because when the system mode/privileged bit is not set, updating the upper register
in an address register pair is illegal to prevent escaping the bank/segment and provide a crude memory protection.
