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
https://www.masswerk.at/6502/

## Reference Manuals

http://wpage.unina.it/rcanonic/didattica/ce1/docs/68000.pdf

## Assembly Syntax

<instruction>[|<condition>] [<target_address>,] [<source_address>]

LOAD|EQ r4, (r1, a)

## Addressing Modes

Used with LOAD/STOR

Registers r = r1, r2, r3, r4, r5, r6, r7, lh*, ll (l), ah*, al (a), sh*, sl (s), ph*, pl (p), sr\*

Address registers a = a, p, s, l

\* ah, sh, ph, lh, and sr are considered privileged, when system mode bit is not set, certain access patterns may trigger a privilege violation

Immediate | #n | LOAD r1, #123
Register direct | rN, aB, sB, pB, lB, sr | LOAD r1, r5
Address register direct | a | LDEA a, (r1, a)
Address register indirect | (a) | STOR (s), r1
Address register indirect with post increment | (a)+ | LOAD (s)+, r1
Address register indirect with pre decrement | -(a) | STOR r1, -(s)
Address register indirect with register displacement | (r, a) | STOR (r4, a), r1
Address register indirect with immediate displacement | (#n, a) | LOAD r4, (#-3, a)

MAYBE?
Address register indirect with pre/post increment STOR (a-), r1
THAT WOULD GET RID OF PUSH/POP
++++ ALSO maybe the subroutine instructions have an address register argument to store the return address
\_\_\_\_ IF WE CAN SAY THAT THE STORE TO THE ADDRESS REGISTER CAN HAPPEN IN A SINGLE CYCLE (why not?)
THEN RETURN FROM SUBROUTINE COULD JUST BE a transfer from address register to PC
Now that I know about barrel shifters... https://www.cs.umd.edu/users/meesh/cmsc411/website/handouts/Simple_Operation_of_MIPS.htm looks pretty good (except extra fetch for 2xword instructions)
If we can read/write address registers in one cycle, why not 32 bit CPU?? dunno simolicity? artifical constraint
// Maybe the double register/32 bit read/write can be done by the address unit and ALU doesn't need to know
ALU is just condition/shift/arith/logic on 16 bit words
Addressing unit can access the address registers as double words (probs doesn't have to do much with them)

\* Address register indirect doesn't have dedicated opcodes because it is equivalent to (#0, a). The assembler will alias this automatically.

#### Valid Source Operands

Source operands are on the right side of the arguments. Destination operands are always first.

|      | Implied | Immediate | Register Direct | Address Direct | Indirect Immediate | Indirect Register | Register Range | Post Increment | Pre Decrement |
| ---- | ------- | --------- | --------------- | -------------- | ------------------ | ----------------- | -------------- | -------------- | ------------- |
| HALT | 0x00    |           |                 |                |                    |
| NOOP | 0x01    |           |                 |
| WAIT | 0x02    |           |                 |
| RETE | 0x03    |           |                 |
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
| SJMP |         | 0x18      |                 |                |                    |
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

# TODO LOAD/STORE/LDMY/STMN PRE POST INCRE

42 Total Opcodes

#### NEW Valid Source Operands

6 bits to encode op
3 bits for addressing mode
3 bits left

Implied | | TATS
Immediate | #123 | ADDI r1, #0x01
Register direct | rN, aB, sB, pB, lB, sr | LOAD r1, r5
Address register indirect with immediate displacement | (#n, a) | LOAD r4, (#-3, a)
Address register indirect with register displacement | (r, a) | STOR (r4, a), r1
Address register indirect with post increment | (a)+ | LOAD (s)+, r1
Address register indirect with pre decrement | -(a) | STOR r1, -(s)

##### Removed for now to reduce complexity

Register range direct | rN->rM, | LDMR r1->r7, (a)

0x1\_ = Arithmatic
0x2\_ = Control Flow
0x3\_ = Load/Store

Source operands are on the right side of the arguments. Destination operands are always first.

|      | Immediate | Register Direct | Indirect Immediate | Indirect Register | Post Increment | Pre Decrement | Implied |
| ---- | --------- | --------------- | ------------------ | ----------------- | -------------- | ------------- | ------- |
| ADDI | 0x00      |                 |                    |                   |                |               |
| ADCI | 0x01      |                 |                    |                   |                |               |
| SUBI | 0x02      |                 |                    |                   |                |               |
| SBCI | 0x03      |                 |                    |                   |                |               |
| ANDI | 0x04      |                 |                    |                   |                |               |
| ORRI | 0x05      |                 |                    |                   |                |               |
| XORI | 0x06      |                 |                    |                   |                |               |
| LSLI | 0x07      |                 |                    |                   |                |               |
| LSRI | 0x08      |                 |                    |                   |                |               |
| ASLI | 0x09      |                 |                    |                   |                |               |
| ASRI | 0x0A      |                 |                    |                   |                |               |
| RTLI | 0x0B      |                 |                    |                   |                |               |
| RTRI | 0x0C      |                 |                    |                   |                |               |
| CMPI | 0x0D      |                 |                    |                   |                |               |
| ADDR |           | 0x10            |                    |                   |                |               |
| ADCR |           | 0x11            |                    |                   |                |               |
| SUBR |           | 0x12            |                    |                   |                |               |
| SBCR |           | 0x13            |                    |                   |                |               |
| ANDR |           | 0x14            |                    |                   |                |               |
| ORRR |           | 0x15            |                    |                   |                |               |
| XORR |           | 0x16            |                    |                   |                |               |
| LSLR |           | 0x17            |                    |                   |                |               |
| LSRR |           | 0x18            |                    |                   |                |               |
| ASLR |           | 0x19            |                    |                   |                |               |
| ASRR |           | 0x1A            |                    |                   |                |               |
| RTLR |           | 0x1B            |                    |                   |                |               |
| RTRR |           | 0x1C            |                    |                   |                |               |
| CMPR |           | 0x1D            |                    |                   |                |               |
| BRAN | 0x20      |                 |                    |                   |                |               |
| BRSR | 0x21      |                 |                    |                   |                |               |
| LDEA |           |                 | 0x22               | 0x23              |                |               |
| RETS |           |                 |                    |                   |                |               | 0x24    |
| LJMP |           |                 | 0x25               | 0x26              |                |               |         |
| LJSR |           |                 | 0x28               | 0x29              |                |               |         |
| SJMP | 0x2A      |                 |                    |                   |                |               |
| SJSR | 0x2B      |                 |                    |                   |                |               |
| LOAD | 0x30      | 0x31            | 0x32               | 0x33              | 0x34           |               |
| STOR |           |                 | 0x35               | 0x36              |                | 0x37          |
| NOOP |           |                 |                    |                   |                |               | 0x3C    |
| WAIT |           |                 |                    |                   |                |               | 0x3D    |
| EXCP | 0x3E      |                 |                    |                   |                |               |         |
| RETE |           |                 |                    |                   |                |               | 0x3F    |

Meta instructions:
HALT -> ORR sr, 0b0010_0000
WAIT -> ORR sr, 0b0001_0000
RETS -> LDEA p, (a)
NOOP -> ADDI r1, #0 (or any instruction with status code 0xF)

42 Total Opcodes

Max 0x3F instruction ID

0x30 - 0x30

|      | RI ALU | RR ALU | MR imm | MR RD | Branch | Stack | Double Register | NOOP |
| ---- | ------ | ------ | ------ | ----- | ------ | ----- | --------------- | ---- |
| ADDI | 0x00   |        |        |       |        |
| ADCI | 0x01   |        |        |       |        |
| SUBI | 0x02   |        |        |       |        |
| SBCI | 0x03   |        |        |       |        |
| ANDI | 0x04   |        |        |       |        |
| ORRI | 0x05   |        |        |       |        |
| XORI | 0x06   |        |        |       |        |
| LSLI | 0x07   |        |        |       |        |
| LSRI | 0x08   |        |        |       |        |
| ASLI | 0x09   |        |        |       |        |
| ASRI | 0x0A   |        |        |       |        |
| RTLI | 0x0B   |        |        |       |        |
| RTRI | 0x0C   |        |        |       |        |
| CMPI | 0x0D   |        |        |       |        |
| ADDR |        | 0x10   |        |       |        |
| ADCR |        | 0x11   |        |       |        |
| SUBR |        | 0x12   |        |       |        |
| SBCR |        | 0x13   |        |       |        |
| ANDR |        | 0x14   |        |       |        |
| ORRR |        | 0x15   |        |       |        |
| XORR |        | 0x16   |        |       |        |
| LSLR |        | 0x17   |        |       |        |
| LSRR |        | 0x18   |        |       |        |
| ASLR |        | 0x19   |        |       |        |
| ASRR |        | 0x1A   |        |       |        |
| RTLR |        | 0x1B   |        |       |        |
| RTRR |        | 0x1C   |        |       |        |
| CMPR |        | 0x1D   |        |       |        |
| BRAN | 0x20   |        |        |       |        |
| BRSR | 0x21   |        |        |       |        |
| SJMP | 0x22   |        |        |       |        |
| SJSR | 0x23   |        |        |       |        |
| LJMP |        |        |        |       |        |       | 0x24            |
| LJSR |        |        |        |       |        |       | 0x25            |
| RETS |        |        |        |       |        |       | 0x26            |
| TSTA |        |        |        |       |        |       | 0x27            |
| TATS |        |        |        |       |        |       | 0x28            |
| LOAD | 0x30   | 0x31   | 0x32   | 0x33  |        | 0x34  |
| STOR |        |        | 0x35   | 0x36  |        | 0x37  |
| NOOP |        |        |        |       |        |       |                 | 0x3C |
| WAIT |        |        |        |       |        |       |                 | 0x3D |
| EXCP |        |        |        |       |        |       |                 | 0x3E |
| RETE |        |        |        |       |        |       |                 | 0x3F |

# TODO: Make sure privileged/ non privileged SP bits are in separate bytes for easier checks

# TODO: How to propagate carry (new instruction or just 6502 it and only provide ADD with carry)

# TODO: address register can be target for lea only

# TODO: Load/store single byte?

## Execution Steps

Based on https://www.cs.umd.edu/users/meesh/cmsc411/website/handouts/Simple_Operation_of_MIPS.htm

```

1. Instruction Fetch (IF) (1/2)

IR[0] <- Mem[PC]          ; Instruction Register (high word), Program Counter

2. Instruction Fetch (IF) (2/2)

IR[1] <- Mem[PC+1]        ; Instruction Register (low word), Program Counter
NPC <- PC + 2		      ; Next Program Counter

3. Decode/Register Fetch (ID)

Ins = Instruction
Des = Destination
SrA = Source Operand A
SrB = Source Operand B
Con = Condition Flags
imm = Immediate value (from instruction)
Adr = Source Address Register
; TODO: What to do in commands on whole address register (e.g. long jump which is effectively copy from address register to PC)
AdrL = The index of the lower source address register (in the VM register indexes)
AdrH = The index of the higher source address register (in the VM register indexes)
Sr = Value of status register
addr_inc = Value added onto address register in later stage (if applicable)

; Prime versions (e.g. SrA' are the actual dereferenced value, rather than the index)

; Get the raw values/indexes
; Some might have garbage in them (be invalid) depending on the instruction type
Ins <- IR(0..5)
Des <- IR(6..9)
SrA <- IR(10..13)
SrB <- IR(14..17)
Con <- IR(28..31)
imm <- IR(10..26)
Adr <- IR(24..27)

; Times two because one address register index refers to a pair of address registers (32 bit)
AdL <- (Adr * 2) + 8
AdH <- (Adr * 2) + 7

; Times two because one address register index refers to a pair of address registers (32 bit)
DesAdL <- (Des * 2) + 8
DesAdH <- (Des * 2) + 7

Des' = Regs[Des]
SrA' = Regs[SrA]
SrB' = Regs[SrB]
AdL' <- Regs[AdrIdxL]
AdH' <- Regs[AdrIdxH]
Con' <- condition_decode(Con, Regs[sr])
Sr  <- Regs[sr]

if (I == LOAD (a)+) addr_inc <- 1      ; Post increment
if (I == STOR -(a)) addr_inc <- -1     ; Pre decrement
else                addr_inc <- 0


4. Execution (EX)

one of the following:

a. Cond == 0 || No Operation

NOP

b. Memory Reference (reg displacement)

if (addr_inc == -1)   ; Pre decrement
    ALUoutput <- SrA' + AdL' + addr_inc
else
    ALUoutput <- SrA' + AdL'

c. Memory Reference (imm displacement)

if (addr_inc == -1)   ; Pre decrement
    ALUoutput <- imm + AdL' + addr_inc
else
    ALUoutput <- imm + AdL'

d. Register-Register ALU:

ALUoutput <- SrA' op SrB'
Regs[sr] <- status(SrA' op SrB', Sr)

e. Register-Immediate ALU operation:

ALUoutput <- Des' op imm
Regs[sr] <- status(Des' op imm, Sr)

f. Branch:

ALUoutput <- PC + imm

5. Memory access/branch completion (MEM):

one of the following:

a. Memory load

LMD <- Mem[AdrH | ALUOutput]

b. Memory store

Mem[AdrH | ALUOutput] <- A

c. Branch/Jump

if (Cond') {
    L <- PC
    PC <- [AdrH | ALUOutput]
}
else {
    PC <- NPC
}

6. Write-back cycle (WB):

one of the following:

a. Memory load

Regs[Des] <- LMD

b. Register-Register ALU or Register-Immediate ALU:

Regs[Des] <- ALUoutput

c. Load Effective Address

Regs[DesAdL] <- ALUOutput
Regs[DesAdH] <- AdrH

d. Else:

NOP

// TODO: TODO: Post increment and PC increment


```

TODO: Short/long jumps, LEA etc.
Removing LDEA because the displacements are easy to calculate (it could be done with a meta instruction)
Adding address register direct to load to allow atomic transfer between address registers (required to set up stack without it getting corrupted by an interrupt half way through)

Need a special instruction for system mode to set up the user stack pointer? Otherwise maybe should dump the system mode stuff
If we use the 4th address value for user stack (u) then we can set it in system mode!
