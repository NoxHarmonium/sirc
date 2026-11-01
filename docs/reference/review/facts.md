# SIRC-1 implementation facts digest

What the `sirc-vm` code and tests do as of the `human-proofread` branch. The manuals are canonical
(`docs/reference/manual-handover.md:14-31`). This file records code behaviour so reviewers can spot where code
and manual disagree. It does not settle which one is right. Where code looks buggy, the entry says so.

**Citation prefixes.** `cpu/` = `sirc-vm/peripheral-cpu/`, `bus/` = `sirc-vm/peripheral-bus/`,
`tc/` = `sirc-vm/toolchain/`, `vm/` = `sirc-vm/sirc-vm/`, `ram/` = `sirc-vm/device-ram/`,
`gen/` = `docs/reference/generated/`, `ho` = `docs/reference/manual-handover.md`.
`cpu/src/pu/` = `cpu/src/coprocessors/processing_unit/`, `cpu/src/eu/` = `cpu/src/coprocessors/exception_unit/`.
Bare `pu-execution.rs` = `cpu/src/pu/execution.rs`; bare `eu-execution.rs` = `cpu/src/eu/execution.rs`; bare `definitions.rs` = `cpu/src/pu/definitions.rs`; `eu/definitions.rs` = `cpu/src/eu/definitions.rs`; `write_back.rs`, `alu.rs`, `fetch_and_decode.rs`, `memory_access.rs`, `execution_effective_address.rs` live in `cpu/src/pu/stages/`; `encoding.rs` = `cpu/src/pu/encoding.rs`; bare `lib.rs`/`registers.rs` = `cpu/src/…`; bare test files are under `cpu/tests/{instructions,exceptions}/`; bare toolchain parser files are under `tc/src/parsers/` or `tc/src/parsers/opcodes/`.

**Deliberate decisions in the handover:**
- The manual is canonical (ho:14-31).
- There is no zero register, which is why `SHFR` was rejected (ho:21-22).
- The DMA coprocessor is optional (ho:22-23).
- In protected mode, a write to `sr` changes only the low byte, silently and with no fault (ho:218-227).
- The decode-time privilege check fires only for instructions that actually *write* `ah/lh/ph/sh`.
  `CMPI/CMPR/TSAI/TSXI/STOR` that name those registers do not fault (ho:228-233).
- Still open: the reset-vector-fetch cycle count. The manual says 4 in Ch. 6 and 6 elsewhere (ho:216-217).

---

# Architecture

## Registers

| Idx | Name | Width | Notes |
|---|---|---|---|
| 0-6 | `r1`-`r7` | 16 | General purpose (`cpu/src/registers.rs:47-55`, `:174-181`). Index 0 is `r1`, so there is no zero register. An all-zero instruction word is `ADDI r1,#0` with no SR update (`cpu/src/pu/encoding.rs:742-754`; `cpu/tests/instructions/protected_mode_test.rs:245`). |
| 7 | `sr` | 16 | Status register (`registers.rs:55,182`). |
| 8,9 | `lh`,`ll` | 16 each | Link pair `l` (`registers.rs:56-57`). |
| 10,11 | `ah`,`al` | 16 each | Address pair `a`. |
| 12,13 | `sh`,`sl` | 16 each | Stack pair `s`. |
| 14,15 | `ph`,`pl` | 16 each | Program counter pair `p`. |

- **Address-register index** (the AF field and the destination field of LDEA/LDEL): `l`=0, `a`=1, `s`=2, `p`=3
  (`registers.rs:76-85`). Decode maps index n to registers `0x8|n<<1` (high) and `0x9|n<<1` (low)
  (`cpu/src/pu/stages/fetch_and_decode.rs:199-202`).
- **Address width.** A pair forms `(high<<16|low) & 0x00FF_FFFF`, so only the low 8 bits of the high register reach
  the bus (`registers.rs:8`, `:289-295`, `:349-356`). The low word wraps within its segment. The PC increment
  touches only `pl` (`fetch_and_decode.rs:204-206`). An effective-address result keeps the source's high word
  (`write_back.rs:196-201`).
- **Addressing unit.** Addresses count 16-bit words. An instruction is 2 words, and the PC advances by 2
  (`cpu/src/pu/definitions.rs:40-41`; `fetch_and_decode.rs:204`).
- **Internal registers, not visible to programs:**
  - `pending_coprocessor_command`, the cause register (`registers.rs:162-167`).
  - Exception-unit state: 8 link registers, pending interrupts, the pending fault, the WAIT flag, the halted flag
    and the current exception level (`registers.rs:588-601`).
- **Reset value.** `Registers::default()` is all zero, but only at construction (`registers.rs:138`;
  `cpu/src/lib.rs:219-233`). The reset sequence changes only SR (to 0) and the PC, which it loads from the vector
  (`eu-execution.rs:380-383`; `lib.rs:612-625`). `r1`-`r7`, `l`, `a` and `s` keep their values.
- **Privilege, read side.** In protected mode a read of `sr` returns only the low byte; the privileged byte reads
  as 0 (`fetch_and_decode.rs:40-48`; `registers.rs:9-11`).
- **Privilege, write side.**
  - A direct write (ALU result or LOAD) to `ah`, `lh`, `ph` or `sh` faults with PrivilegeViolation at decode, but
    only when the condition passes (`pu-execution.rs:23-54`).
  - Tests: `protected_mode_test.rs:62` (fault); `:196` (CMP/TSA/STOR do not fault); `:262` (no fault when the
    condition fails).
  - `pl` and the low halves are not privileged.
  - An address-pair write (LDEA, LDEL, branches, auto-increment) in protected mode faults only if a high word
    would *change*; otherwise only the low halves are written (`write_back.rs:62-95`; tests
    `protected_mode_test.rs:280,305,354,376,422,473,528,548,568`).

## Status register

| Bit | Name | Meaning |
|---|---|---|
| 0 | Z | Result zero (`registers.rs:16`) |
| 1 | N | Bit 15 of the result (`registers.rs:17`; `alu.rs:573-577`) |
| 2 | C | Carry out; set when a subtract borrows (`registers.rs:18`; `definitions.rs:115-117`) |
| 3 | V | Signed overflow (`registers.rs:19`) |
| 4-7 | - | Unused (no field defined) |
| 8 | P | Protected mode when set. Clearing it (P=0) is supervisor mode (`registers.rs:22-24`). |
| 9-12 | I1-I4 | Enable for interrupt lines 1-4. Bit 9 is line 1, the lowest priority (`registers.rs:25-32`, `:475-514`). |
| 13 | E | Exception active. Set on exception entry and cleared by RETE to level 0 (`registers.rs:33-35`; `eu-execution.rs:287,376-378`). |
| 14 | TrapOnAddressOverflow | Enables the SegmentOverflow trap (`registers.rs:36-38`) |
| 15 | T | Trace. Raises InstructionTrace after each instruction (`registers.rs:39-40`). |

How each bit can change:
- **Supervisor write** to `sr` (P=0): every bit except E is written. E is preserved (`write_back.rs:50-56`).
- **Protected write** (P=1): only bits 7-0 are written; bits 15-8 are silently kept, with no fault
  (`write_back.rs:45-49`; tests `protected_mode_test.rs:105,159,179`).
- **ALU flag update:**
  - With update source ALU or Shift, the instruction replaces bits 7-0 and never touches bits 15-8
    (`write_back.rs:140-156`).
  - Update source None (00), and the reserved value 11, leave SR unchanged (`write_back.rs:154`).
  - For an ALU write whose destination is `sr`, the register write happens first and the flag update then
    overwrites the low byte (`write_back.rs:186-189`). See Open question 5.
- **Exception entry** clears P and T and sets E (`eu-execution.rs:284-287`).
- **RETE** restores all 16 bits of the saved SR, then clears E if it returns to level 0 (`eu-execution.rs:372-378`).
- **Reset** sets SR to 0 (`eu-execution.rs:381`).
- **TRCE pin:** `force_trace_mode` makes the CPU behave as if T were set (`lib.rs:389-393`).

## Instruction formats

There are 3 formats. Every instruction is 32 bits, stored big-endian as two words, high word first
(`encoding.rs:47-51`, `:115-260`; `pu-execution.rs:118-130`).

| Bits | Immediate | Short immediate | Register |
|---|---|---|---|
| 31-26 | opcode | opcode | opcode |
| 25-22 | reg | reg | r1 |
| 21-18 | imm16[15:12] | imm8[7:4] | r2 |
| 17-14 | imm16[11:8] | imm8[3:0] | r3 |
| 13 | imm16[7] … | shift operand (0 = immediate count, 1 = register) | same |
| 12-10 | imm16[6:4] … | shift type | same |
| 9-6 | imm16[3:0] | shift count, or the index of the register holding it | same |
| 5-4 | AF | AF | AF |
| 3-0 | cond | cond | cond |

The immediate field occupies bits 21-6.

- **Format by opcode** (`encoding.rs:272-290`; `fetch_and_decode.rs:23-38`):
  - 0x00-0x0F: immediate.
  - 0x10-0x1F: even opcodes immediate, odd opcodes register.
  - 0x20-0x2F: short immediate.
  - 0x30-0x3F: register.
- **The 2-bit AF field** has two uses:
  - For ALU and COP opcodes it is the SR update source: 00 None, 01 ALU, 10 Shift, 11 Reserved
    (`definitions.rs:95-102`).
  - For opcodes 0x10-0x1F it is the source address register.
  - Decode feeds the same 2 bits to both uses (`fetch_and_decode.rs:199-200,224`).
- **Stale doc comments.** `encoding.rs:96,362,415` call bits 5-4 "2 bit reserved". `definitions.rs:9` calls them
  "address register a, p or s".
- **Generated tables.** These worked examples match the layout above: `gen/immediate-format-encodings.tex:8-42`,
  `gen/short-immediate-format-encodings.tex`, `gen/register-format-encodings.tex`. For example, `ADDI r1, #100`
  encodes as `0x00001910`.

## Opcode map

The ALU operation is `opcode & 7`. Bit 3 set (except for 0xF) means "simulate": flags only, no result written
(`cpu/src/pu/stages/execution_effective_address.rs:63-69`; `alu.rs:28-39,542-548`). Assembler mnemonics come from
`tc/src/parsers/opcodes/*`.

| Op (imm / short / reg) | Mnemonic (I / R) | Semantics |
|---|---|---|
| 00 / 20 / 30 | ADDI / ADDR | d = a + b |
| 01 / 21 / 31 | ADCI / ADCR | d = a + b + C |
| 02 / 22 / 32 | SUBI / SUBR | d = a − b |
| 03 / 23 / 33 | SBCI / SBCR | d = a − b − C |
| 04 / 24 / 34 | ANDI / ANDR | d = a & b |
| 05 / 25 / 35 | ORRI / ORRR (SHFT = 0x25) | d = a \| b |
| 06 / 26 / 36 | XORI / XORR | d = a ^ b |
| 07 / 27 / 37 | MOVI / MOVR | d = b (opcode 0x27 is never emitted by the assembler) |
| 08 / 28 / 38 | CMNI / CMNR | flags of a + b |
| 09 / 29 / 39 | CNCI / CNCR | flags of a + b + C |
| 0A / 2A / 3A | CMPI / CMPR | flags of a − b |
| 0B / 2B / 3B | CPCI / CPCR | flags of a − b − C |
| 0C / 2C / 3C | TSAI / TSAR | flags of a & b |
| 0D / 2D / 3D | TSOI / TSOR | flags of a \| b |
| 0E / 2E / 3E | TSXI / TSXR | flags of a ^ b |
| 0F / 2F / 3F | COPI / COPR | cause register = b, with no flag update (`write_back.rs:111,291-293`). The assembler never emits 0x2F (`coprocessor.rs:103-106`). |

How the operands a and b are formed (`fetch_and_decode.rs:180-194`):
- **Immediate format:** a = `reg`, b = imm16.
- **Short immediate:** a = shift(`reg`), b = imm8 zero-extended. The shift applies to the *register*, not to the
  immediate (`fetch_and_decode.rs:190-193`; test `cpu/tests/instructions/arithmetic_short_immediate_test.rs:508`
  shifts the register with imm 0).
- **Register format:** a = shift(r2), b = r3, destination r1.
- **The opcode test-variant table** is confirmed by `tc/tests/assembler/test_variants_test.rs:97-116`.

Memory and effective-address opcodes. EA = `(src.high, src.low + disp)` with 16-bit wrap. The displacement is
imm16 (even opcodes) or the r3 value (odd opcodes) (`execution_effective_address.rs:78-128`;
`memory_access.rs:30-74`; `write_back.rs:108-119`).

| Op | Assembler form | Effect |
|---|---|---|
| 10/11 | `STOR (#n,A), rS` / `STOR (rX,A), rS[,shift]` | mem[EA] = data. The imm form stores the `reg` field register. The register form stores shift(r2); r1 is unused (`store.rs:111-155`; `cpu/tests/instructions/store_test.rs:142-177`). |
| 12/13 | `STOR -(#n,A), rS` / `-(rX,A), rS` | mem[EA−1] = data; A.low −= 1 |
| 14/15 | `LOAD rD, (#n,A)` / `(rX,A)` | rD = mem[EA]. No flag update and no shift (`write_back.rs:300-307`; `load_test.rs:168`). |
| 16/17 | `LOAD rD, (#n,A)+` / `(rX,A)+` | rD = mem[EA]; A.low += 1. The base increments by 1; the displacement is not added. |
| 18/19 | `LDEA X, (#n,A)` / `(rX,A)` | X = EA. Aliases: `LJMP` (X=p), `BRAN` (A=p, X=p), `RETS` = `LDEA p,(#0,l)` (`branching.rs:21-27`; `meta.rs:56-69`; `ljmp.rs:63-140`). |
| 1A/1B | `LDEA X, -(#n,A)` / `-(rX,A)` | X = EA−1; A.low −= 1 (`ldea_test.rs:142-183`). |
| 1C/1D | `LDEL X, (#n,A)` / `(rX,A)` | l = address of the next instruction; X = EA. Aliases: `LJSR`, `BRSR`. |
| 1E/1F | `LDEL X, (#n,A)+` / `(rX,A)+` | l = next; A.low += 1; X = EA (`ldel_test.rs:16-58`). Write order is l, then source, then destination (`write_back.rs:248-275`). |

- **PC-relative base.** When A = p, the base is the address of the current instruction, not of the next one. Decode
  reads `pl` before advancing it (`fetch_and_decode.rs:233-234` vs `pu-execution.rs:168`). Test `ljmp_test.rs:225`:
  `pl` 0xFAC0 with `#0xE` lands at 0xFACE; the link holds 0xFAC2 (`:292`).
- **Undecodable fields.** A register field ≥ 4 in LDEA/LDEL produces a register index above 15, which panics
  (`fetch_and_decode.rs:201-202`; `registers.rs:191`). See Open question 12.
- **Simulator exit.** `COPI` with value 0x14FF asserts `exit_simulation` at decode (`pu-execution.rs:170-177`).

## Flag effects

These come from `set_alu_bits` (`alu.rs:551-600`): Z = (result==0), N = bit 15, C as passed in, and
V = `sign(i1)==sign(i2) && sign(result)!=sign(i1)`, or cleared when no inputs are passed.

| Operation | N Z | C | V |
|---|---|---|---|
| ADD, CMN | result | unsigned carry out (`alu.rs:81-90`) | signed overflow |
| ADC, CNC | result | carry from either addition (`alu.rs:132-153`) | computed on (a, b, result) |
| SUB, CMP | result | **set on borrow** (a <u b) (`alu.rs:195-206`; `definitions.rs:115-117`) | computed on (a, !b, result) |
| SBC, CPC | result | borrow from either step; computes a − b − C (`alu.rs:245-267`) | computed on (a, !b, result) |
| AND/OR/XOR, TSA/TSO/TSX | result | 0 | 0 (`alu.rs:271-305`) |
| MOVI/MOVR (Load) | value | 0 | 0 (`alu.rs:307-317`; test `arithmetic_immediate_test.rs:547-580`) |
| Shift flags (update source S) | shifted value | see the shift table | see the shift table |
| [S] on an immediate-format opcode | 0 | 0 | 0. The shift result is constant 0, which clears every flag including Z (`fetch_and_decode.rs:189`; `arithmetic_immediate_test.rs:34-38`). |

- **Updating flags.** Flags change only when AF = 01 (ALU) or 10 (Shift). The assembler defaults to [A] for ALU
  mnemonics and to [S] for SHFT (`arithmetic_immediate.rs:109-110,165-166`; `arithmetic_register.rs:96-97`;
  `mov.rs:80-82`).
- **Condition false.** The instruction does nothing except advance the PC (`write_back.rs:104-106`;
  `condition_flags_test.rs:101-114`).

## Shift types

The shift-type field is 3 bits (`definitions.rs:69-81`). Assembler names are in `tc/src/parsers/instruction.rs:359-381`.
The count is 4 bits, 0-15 (`definitions.rs:43`). With the register operand form, the 4-bit field is a register
index and the full 16-bit value of that register is the count (`fetch_and_decode.rs:64-67`;
`arithmetic_register_test.rs:960-996`).

| Code | Name (asm) | Result | C | V |
|---|---|---|---|---|
| 0 | None (`NUL`) | operand | 0 | 0 (N and Z from the operand) (`alu.rs:330-338`) |
| 1 | LSL | `a << min(n,16)` | last bit shifted out (bit 16 of the wide result) (`alu.rs:370-390`) | 0 |
| 2 | LSR | `a >> min(n,16)` | last bit shifted out | 0 (`alu.rs:393-414`) |
| 3 | ASL | same as LSL | same as LSL | set if the result's sign differs from the original sign (`alu.rs:417-443`) |
| 4 | ASR | sign-extended `a >> min(n,16)`; a count ≥ 16 fills with the sign bit | last bit shifted out | always 0, because the sign never changes (`alu.rs:450-483`) |
| 5 | RTL (rotate left) | `a.rotate_left(n mod 16)`, unclamped | result bit 0 | 0 (`alu.rs:485-499`) |
| 6 | RTR (rotate right) | `a.rotate_right(n mod 16)` | result bit 15 | 0 (`alu.rs:501-515`) |
| 7 | Reserved | operand unchanged | 0 | 0. Z is 0 even for a zero operand, and the assembler cannot name this type (`alu.rs:358-360`). |

The rotate carry rule carries a TODO, "Check the purpose of bit shifted of ROL out going to carry" (`alu.rs:487-490`).

## Addressing modes

The assembler's `AddressingMode` enum and its syntax are in `instruction.rs:84-106`; parse order is at `:253-297`.

| Mode | Syntax | Encoding |
|---|---|---|
| Immediate | `#n`, `@sym[.u/.l/.r]`, `$ph` | imm16, or imm8 when a shift is present |
| Register direct | `r1-r7, sr, lh, ll, ah, al, sh, sl, ph, pl` | 4-bit index |
| Address register direct | `a, s, p, l` | 2-bit index |
| Indirect, immediate displacement | `(#n, A)`; `(A)` is shorthand for `(#0, A)` (`instruction.rs:150-160`) | even opcode 0x10-0x1E, AF = A |
| Indirect, register displacement | `(rX, A)` | odd opcode, r3 = rX |
| Post-increment | `(#n, A)+`, `(rX, A)+` | 0x16/17, 0x1E/1F |
| Pre-decrement | `-(#n, A)`, `-(rX, A)` | 0x12/13, 0x1A/1B |
| Shift definition | `LSL #n` or `LSL rX` | shift fields |

Which instruction families accept which modes:
- LOAD: no pre-decrement.
- STOR: no post-increment.
- LDEA: no post-increment.
- LDEL: no pre-decrement.
- LJMP and LJSR: no `(A)`.
- Tests: `tc/tests/assembler/addressing_mode_test.rs:42-171`; `mov_test.rs:138-147`.

## Condition codes

The condition field is 4 bits. Evaluation is in `definitions.rs:47-67,118-159`; assembler spellings in
`instruction.rs:317-357`.

| Code | Asm | Name | Condition |
|---|---|---|---|
| 0 | AL | Always | true |
| 1 | `==` | Equal | Z |
| 2 | `!=` | NotEqual | !Z |
| 3 | CS | CarrySet | C |
| 4 | CC | CarryClear | !C |
| 5 | NS | NegativeSet | N |
| 6 | NC | NegativeClear | !N |
| 7 | OS | OverflowSet | V |
| 8 | OC | OverflowClear | !V |
| 9 | HI | UnsignedHigher | !C && !Z |
| A | LO | UnsignedLowerOrSame | C \|\| Z |
| B | `>=` | GreaterOrEqual | N == V |
| C | `<<` | LessThan | N != V |
| D | `>>` | GreaterThan | !Z && N == V |
| E | `<=` | LessThanOrEqual | Z \|\| N != V |
| F | NV | Never | false |

HI and LO invert C relative to ARM because SIRC sets C on a borrow (`definitions.rs:115-117`). Tests:
`cpu/tests/instructions/condition_flags_test.rs:19-58,118-149`.

## Exceptions

**Vectors.** Vector n is a 32-bit entry: the high word is at word address `n*2` and the low word at `n*2+1`
(`eu-execution.rs:324-344`; tests `faults.rs:199-207`).

| Vector | Exception | Level |
|---|---|---|
| 0x00 | Reset. The cause value `0x1B00` is seeded by `reset()` (`lib.rs:622-624`). | - |
| 0x01 | Bus fault (BERR) | 7 |
| 0x02 | Alignment (odd `pl`) | 7 |
| 0x03 | Segment overflow | 7 |
| 0x04 | Invalid opcode (absent coprocessor) | 7 |
| 0x05 | Privilege violation | 7 |
| 0x06 | Instruction trace | 7 |
| 0x07 | Level-5 interrupt conflict | 7 |
| 0x08 | Double fault | 7 |
| 0x09 | Bus protection (BPER) | 7 |
| 0x0A-0x0F | Reserved | - |
| 0x10 / 0x20 / 0x30 / 0x40 / 0x50 | Hardware interrupt L5 (NMI) / L4 / L3 / L2 / L1 | 6 / 5 / 4 / 3 / 2 |
| 0x60-0xFF | User `EXCP` | 1 |

Sources for the table: `cpu/src/eu/definitions.rs:5-113,132-176`. The level of a non-software dispatch is
`7 − (vector>>4)` (`eu-execution.rs:77-82`). Software exceptions are always level 1.

- **Link registers.** A level-L dispatch stores {return address, full SR, previous level} in `LR[L−1]`
  (`eu-execution.rs:278-282`):
  - LR[0]: software exceptions.
  - LR[1..5]: hardware L1-L5.
  - LR[6]: faults.
  - LR[7]: fault metadata, written by every `raise_fault` (`lib.rs:64-65,193-205`).
- **Fault metadata in LR[7]** (`lib.rs:84-92,130-155`; layout test `faults.rs:1372-1450`):
  - Return-address field: the bus address at the time of the fault. For an InvalidOpCode fault it holds the
    offending command value instead (`faults.rs:1080-1083`).
  - SR field bits 2-0: BAT.
  - Bit 3: double fault.
  - Bits 7-4: the fault code.
  - Bits 11-8: the original fault, which is filled in even when there is no double fault (`faults.rs:1207-1218`).
- **On entry** the CPU clears P and T, sets E, loads PC from the vector and sets the level
  (`eu-execution.rs:284-290`). The interrupt-enable bits and TrapOnAddressOverflow are untouched. Tests:
  `hardware_exceptions.rs:13`; `software_exceptions.rs:29`; `faults.rs:1767-1777` (the saved SR keeps T).
- **Return addresses.**
  - These faults return to the faulting instruction's own address, so they are retryable: Bus, BusProtection,
    Alignment, SegmentOverflow, InvalidOpCode, PrivilegeViolation (both the decode-time and write-back checks).
    Tests: `faults.rs:306-311,363-366,462-467,753-757,817-822,933-938,1084-1089,1499-1504,1567-1573`.
  - InstructionTrace returns to the next instruction (`faults.rs:1761-1765`).
  - Software exceptions return to the next instruction (`software_exceptions.rs:58-66`).
  - An `EXCP` below 0x60 in protected mode faults with the already-advanced PC (`eu-execution.rs:438-448`).
  - LevelFiveConflict stores the current PC (`eu-execution.rs:250-258`).
- **A faulting instruction is aborted.** Every later processing-unit stage returns early once a fault is pending
  (`pu-execution.rs:76-79`). A decode-time PrivilegeViolation stops LOAD before it reads the bus (ho:231-232).
- **Priority** (`eu-execution.rs:140-219`; `lib.rs:366-381`):
  - Selection order: a pending fault first. Then the highest pending, enabled hardware line whose level is above
    the current one (L5 may re-enter at level 6). Then the pending coprocessor command.
  - A lower-priority exception is ignored unless it is a fault (`eu-execution.rs:262-266`).
  - A software exception issued while the level is not 0 is silently discarded (`lib.rs:371-377`; tests
    `software_exceptions.rs:75,115`).
  - A higher-level interrupt preempts a running handler (`hardware_exceptions.rs:155-207`).
  - Several pending lines dispatch highest first (`hardware_exceptions.rs:50-105`).
- **Double and triple faults.**
  - A fault raised while the level is ≥ 7 becomes DoubleFault (vector 8) (`lib.rs:183-190`; test
    `faults.rs:1165-1281`).
  - A bus error while fetching a fault vector escalates to DoubleFault and keeps the original PC
    (`lib.rs:530-559`; `faults.rs:578`).
  - A bus error while fetching the double-fault vector is a triple fault. The CPU drops the pending fault and holds
    RSTI asserted until it is reset (`lib.rs:520-528,272-277`; `faults.rs:632,666`).
  - A bus error while fetching a hardware, software or reset vector raises an ordinary Bus fault, and the original
    dispatch never enters (`lib.rs:561-571`; `faults.rs:471,532`).
- **Interrupt latching and masking.**
  - Interrupt bits: bit 0 is L1 and bit 4 is L5/NMI (`bus/src/device.rs:9-13`).
  - Lines 1-4 latch only if they are enabled *when asserted*, and they are level-sensitive (re-latch while held).
  - L5 is always enabled and latches on the rising edge (`lib.rs:576-606`; tests `hardware_exceptions.rs:217,245`).
  - A latched line that is later masked stays pending and is not dispatched; lower enabled lines still dispatch
    (`eu-execution.rs:166-212,500-540`).
  - An L5 that arrives while the level is 6 raises LevelFiveInterruptConflict (`eu-execution.rs:250-258`;
    `faults.rs:1681`).
  - Interrupts are sampled every poll, including during WAIT, and an enabled one clears WAIT (`lib.rs:315-322,594,603`).
- **SegmentOverflow** fires only when SR bit 14 is set:
  - For a memory or EA displacement that genuinely wraps the low word, using signed arithmetic
    (`execution_effective_address.rs:79-90,130-144`; `faults.rs:761,826`).
  - For a pre-decrement that wraps (`:94-104`; `faults.rs:942,974`).
  - For a PC increment past 0xFFFF, on the next fetch (`pu-execution.rs:99-107,138-148`; `faults.rs:855`).
  - See Open question 11.
- **Alignment** is checked on every processing-unit phase while `pl` is odd (`pu-execution.rs:86-93`;
  `faults.rs:722`).
- **Reset entry state:**
  - The reset-vector entry is at word addresses 0x000000 (high) and 0x000001 (low).
  - SR = 0x0000: supervisor mode, L1-L4 masked, T=0, E=0.
  - The level is 0; pending fault, interrupts and WAIT are cleared; the phase is 0 (`lib.rs:612-625`;
    `eu-execution.rs:380-383`; `reset.rs:152-192,253-281`).
  - The link registers are not cleared.
- **Header comment versus code.** `definitions.rs:29-35` describes the 0x000000 block as "Initial PC / System SP /
  Base System RAM". No code reads a system SP. See Open question 13.

## Coprocessors

The cause register is `[15:12]` coprocessor ID, `[11:8]` operation, `[7:0]` value (`lib.rs:74-77`;
`eu/encoding.rs:1-15`).

| ID | Unit | In the VM |
|---|---|---|
| 0 | Processing unit (`pu-execution.rs:65`) | Yes. A command with ID 0 (for example from a short COPI) just runs the processing unit. |
| 1 | Exception unit (`eu-execution.rs:300`) | Yes |
| 2 | DMA (assembler only: `coprocessor_meta.rs:14`) | **Absent.** A call raises InvalidOpCode at the next fetch-low (`lib.rs:415-446`; test `faults.rs:1038-1162`). |
| 3 | Maths (assembler only: `coprocessor_meta.rs:15`) | Absent; same as ID 2 |
| 4-F | - | InvalidOpCode |

**Exception-unit operations** (`eu/definitions.rs:146-161`):

| Op | Meaning | Mnemonic |
|---|---|---|
| 0 | None | - |
| 1 | Software exception; value = vector | `EXCP` |
| 2-8 | Reserved. Treated as a no-op rather than a fault (`eu-execution.rs:67-85,543-561`). | - |
| 9 | Wait | `WAIT` |
| A | Return from exception | `RETE` |
| B | Reset | `RSET` |
| C | Transfer from register | `ETFR` |
| D | Transfer to register | `ETTR` |
| E | Fault | internal |
| F | Hardware exception | internal |

- **ETFR/ETTR value field:** bits 3-0 index LR[0..7]; an index above 7 panics. Bits 5-4 select what moves:
  1 = return address ↔ `a`, 2 = saved SR ↔ `r7`, 3 = both (`eu-execution.rs:221-237,384-428`).
- **Privileged commands.**
  - In protected mode, `(b & 0x0F00) > 0x0700` faults for any coprocessor ID; that is, operation nibbles 8-F are
    privileged (`pu-execution.rs:49-51`).
  - A short COPI carries only 8 bits of value, so it cannot reach a privileged command (`protected_mode_test.rs:657-674`).
  - Faulting is gated by the condition code (`:611-655`).
  - `EXCP` with a vector below 0x60 is also privileged in protected mode (`eu-execution.rs:438-448`;
    `faults.rs:1609,1655`).
- **DMA and maths command encodings** (assembler only; `coprocessor_meta.rs:14-247`; tests
  `coprocessor_test.rs:136-179`):
  - `DMAR` = 0x28xx, `DMAW` = 0x29xx. Operand bits 7-6 select the register (a=00, l=01, s=10). Bit 5 is set for a
    negative count; the magnitude is 0-7.
  - `DMAT a, l, #0-255` = 0x2Ann.
  - `MULU` / `MULS` / `DIVU` / `DIVS` = 0x3000 / 0x3100 / 0x3200 / 0x3300.

---

# Hardware

## Bus and timing

- **Six phases per instruction or dispatch, always** (`CYCLES_PER_INSTRUCTION = 6`, `lib.rs:71`;
  `coprocessors/shared.rs:5-14`). An exception-unit dispatch occupies one full 6-phase slot in place of an
  instruction (`exceptions/common.rs:125-167`).
- **Phases and the bus:**

| Ph | Processing unit (`pu-execution.rs:95-205`) | Exception unit (`eu-execution.rs:322-479`) |
|---|---|---|
| 0 | Read the word at PC (BAT 001). SYNC asserted. Alignment and PC-overflow faults are raised here. | Read the vector high word at `vec*2` (BAT 100) |
| 1 | Latch the high word; read PC+1 (BAT 001) | Latch; read `vec*2+1` (BAT 100) |
| 2 | Latch the low word, decode, read registers, privilege check, `pl += 2`. No bus. | Latch the vector. No bus. |
| 3 | ALU or effective-address calculation. No bus. | Execute the operation (jump, RETE, …) |
| 4 | LOAD: read EA (BAT 010). STOR: write EA (BAT 011). Otherwise idle. | Idle |
| 5 | Write back; LOAD data is latched. No bus. | Clear the pending command and the dispatched fault |

- **Cycles, wait states and acknowledge.**
  - A phase's request completes on the next poll if BACK, BERR or BPER is asserted. Until then the CPU re-asserts
    the same request every poll and the phase does not advance (`lib.rs:114-117,279-296,465-467`). BERR wins over
    BPER (`lib.rs:496-504`).
  - The test harness acknowledges every cycle (`lib.rs:631-648`). The VM's program RAM uses a 2-clock latency
    (`vm/src/main.rs:191`; `ram/src/lib.rs:80-115`).
  - Absolute timing in clocks or ns is not derivable from code.
- **Unmapped access.** A strobe that no device claims is turned into BERR by the bus (`bus/src/lib.rs:248-254`).
- **Bus access type** (BAT2-0) (`bus/src/device.rs:15-44`):
  - 000 None, 001 Instruction fetch, 010 Data read, 011 Data write, 100 Exception vector fetch.
  - 101 DMA read burst and 110 DMA write burst are defined, but the CPU never emits them.
  - 111 Reserved.
- **PROT** reflects SR.P on every poll output (`lib.rs:236-244`).
- **HALT** is sampled only at phase 0:
  - Once latched, the CPU emits nothing until HALT drops, then continues that same poll (`lib.rs:301-313`).
  - Interrupts are not sampled while halted. See Open question 7.
- **WAIT** returns idle bus assertions every poll until an enabled interrupt arrives (`lib.rs:318-322`;
  `reset.rs:229-248`).
- **Reset.**
  - RSTI asserted makes the reset unit call `cpu.reset()` (which aborts any stall) and assert RSTO on that poll and
    5 more, 6 polls in total, without polling the CPU (`bus/src/reset_unit.rs:19-30`; `bus/src/lib.rs:228-239`).
  - Holding RSTI asserted extends RSTO; the 5-poll countdown starts when it is released (`reset.rs:196-225`).
  - After the hold, the exception unit fetches the reset vector from address 0 (`reset.rs:152-192,287-325`).
  - Software `RSET`: the CPU asserts RSTI in its own output on the poll where write-back wraps to phase 0. RSTO then
    runs for 6 polls (`lib.rs:477-486`; `reset.rs:87-147`).
  - A triple fault re-asserts RSTI every poll until reset (`lib.rs:272-277`).
  - At VM start the CPU is reset with `cpu.reset()` directly, with no RSTO hold (`vm/src/main.rs:186-188`).
- **Electrical modelling.** The simulator models every pin as active high; electrical polarity is not modelled
  (`bus/src/device.rs:114-118`).

## Signals

Pins are named in the `BusAssertions` doc comments (`bus/src/device.rs:119-196`).

| Pin | Field | Direction (CPU) | When |
|---|---|---|---|
| A0-A23 | `address` | out | With each strobe; 24-bit masked |
| D0-D15 | `data` | in/out | Out during a STOR write in phase 4; read in the phase after a read request |
| BRW | `op` | out | Read or write |
| BAS | `bus_access_strobe` | out | Phases 0 and 1, and phase 4 for LOAD/STOR; held until completion |
| BACK | `bus_acknowledge` | in | Completes a request |
| BERR | `bus_error` | in | Completes the request and raises a Bus fault (vector 1) |
| BPER | `bus_protection_error` | in | Completes the request and raises BusProtection (vector 9) |
| BAT0-2 | `bus_access_type` | out | With each strobe |
| IRQ1-4, NMI | `interrupt_assertion` bits 0-4 | in | Sampled every poll except while halted or under reset |
| RSTI | `reset_requested` | in (and out for RSET or a triple fault) | See Reset |
| RSTO | `reset_devices_on_bus` | from the reset unit | 6 polls |
| HALT | `halt_requested` | in | Phase 0 |
| TRCE | `force_trace_mode` | in | Sampled at phase 0 (`lib.rs:389-393`) |
| SYNC | `instruction_sync` | out | The poll that runs phase 0 (`lib.rs:471-475`). Also the stall path at `:287`; see Open question 6. |
| PROT | `protected_mode_active` | out | Every poll |

`exit_simulation` and `device_was_activated` are simulator-only (`device.rs:157-165`).

**VM memory map** (`vm/src/main.rs:195-241`):

| Segment | Base | Size (words) | Writable |
|---|---|---|---|
| PROGRAM | 0x0 | 0xFFFF | no |
| TERMINAL | 0x0A0000 | 0xF | yes |
| DEBUG | 0x0B0000 | 0xF | yes |
| VIDEO (with `-e`) | 0x0C0000 | 0xFFFF | yes |

A segment covers `[base, base+size)` (`bus/src/lib.rs:45-50`).

---

# Toolchain

## Assembler syntax

All references are under `tc/src/parsers/`.
- **Tokens** are comment, instruction, label, `.ORG`, data directive and `.EQU`, separated by optional whitespace
  (`shared.rs:255-272`).
  - Comment: `;` to the end of the line. An empty comment is allowed. An instruction with no operands may take a
    trailing comment (`shared.rs:241-248`; `tc/tests/assembler/comment_test.rs:24-77`).
  - Label definition: `:name`, where the name is alphanumeric or `_` (`shared.rs:37-42,112-114`).
  - Symbol reference: `@name` with an optional suffix `.r` (offset), `.l` (lower word) or `.u` (upper word)
    (`shared.rs:122-150`; `tc/src/types/shared.rs:5-7`).
  - Placeholder: `$name` (`shared.rs:103-105`).
- **Numbers** take a `#` prefix (`shared.rs:99-101`):
  - `0x` hex (the prefix is lowercase only; digits may be either case; `_` allowed) (`:61-75`).
  - `0b` binary with `_` (`:44-59`).
  - Decimal with an optional `+`/`-` and `_` (`:77-97`).
  - An operand value is truncated to u16 without a diagnostic (`instruction.rs:109-127`).
- **Case.** Mnemonics, directives and condition codes are upper case. Registers are lower case only
  (`instruction.rs:443-501`).
- **Instruction suffixes** go after the mnemonic in the order `[A|S|N]` then `|CC`. The status-update letter is
  case-insensitive (`instruction.rs:198-210,383-412`).

**Directives** (`tc/src/types/data.rs:5-13`; `data.rs:36-128`; `tc/src/data/object.rs:55-195`):

| Directive | Effect |
|---|---|
| `.ORG n` | n is hex or decimal with **no `#`**. It is a word address; the byte offset becomes n×2 (`shared.rs:116-120`; `object.rs:177-180`). |
| `.DB` / `.DW` / `.DQ` value | Each occupies one **4-byte slot** with the value right-aligned big-endian (`object.rs:71-79,190`). The value may be `#n`, `@sym` (defaults to the full address) or `$ph`. |
| `.EQU $name #n` | Defines a placeholder. Placeholders resolve in token order, so they must be defined before use (`object.rs:151-153,192-194`). |

**Operand forms the assembler accepts:**

| Mnemonics | Forms |
|---|---|
| ADDI…TSXI | `rD, #imm16`; `rD, #imm8, SHIFT` (`arithmetic_immediate.rs:150-290`) |
| SHFT | `rD, SHIFT` → `ORRI` short with imm 0 and default [S] |
| ADDR…TSXR | `rD, rB` (r2 = rD); `rD, rA, rB`; either form with a trailing `SHIFT` (`arithmetic_register.rs:99-199`) |
| MOVI / MOVR | `rD, #imm\|@sym\|$ph` / `rD, rS`. `[S]` is rejected. MOVR with a shift is rejected (`mov.rs:58-180`; `mov_test.rs:122-135`). |
| LOAD | `rD, (#n,A)`, `(rX,A)`, `(#n,A)+`, `(rX,A)+` |
| STOR | `(#n,A), rS`, `(rX,A), rS[,SHIFT]`, `-(#n,A), rS`, `-(rX,A), rS[,SHIFT]` |
| LDEA | `X, (#n,A)`, `(rX,A)`, `-(#n,A)`, `-(rX,A)` |
| LDEL | `X, (#n,A)`, `(rX,A)`, `(#n,A)+`, `(rX,A)+` |
| LJMP | `A`, `A, #n`, `A, rX` |
| LJSR | `A`, `A, #n`, `A, rX`, `(#n,A)+`, `(rX,A)+` |
| BRAN / BRSR | `#n`, `@label`, `$ph` only (`branching.rs:85-127`) |
| RETS, NOOP | no operands (`meta.rs:51-96`) |
| COPI / COPR | `#imm` / `rS` (`coprocessor.rs:62-139`) |
| EXCP | `#n` → `0x1100\|(n&0xFF)` |
| WAIT, RETE, RSET | no operands → 0x1900, 0x1A00, 0x1B00 |
| ETFR | `#n`, `r7, #n`, `a, #n` |
| ETTR | `#n`, `#n, r7`, `#n, a` (`exception.rs:36-376`) |
| DMAR/DMAW, DMAT, MULU/MULS/DIVU/DIVS | see Coprocessors |

- **SHIFT** is one of `NUL LSL LSR ASL ASR RTL RTR` followed by `#0-15` or a register (`instruction.rs:213-251`).
- **Default symbol-reference types** (`*.rs` calls to `override_ref_token_type_if_implied`):
  - BRAN/BRSR: `.r`.
  - MOVI, LOAD, STOR, LDEA, LDEL, LJMP, LJSR: `.l`.
  - Data directives: full address.
  - ALU immediates, COPI, EXCP, ETFR/ETTR and the DMA/maths meta-instructions reject symbols.
- **Forms rejected**, each with a dedicated error:
  - Any `[x]` suffix on LOAD, STOR, LDEA, LDEL, LJMP, LJSR, BRAN, BRSR, RETS, NOOP, COP or exception-unit
    instructions.
  - `LOAD rD, #n` and `LOAD rD, rS`, and LOAD with a shift (`load.rs:131-139,227-241`).
  - Aliased auto-update writes (`opcodes/mod.rs:29-49`; tests `control_flow_test.rs:295-309`):
    - LOAD post-increment whose destination is the pair's high or low half.
    - STOR pre-decrement whose source is the pair's high or low half.
    - LDEA pre-decrement with destination = source.
    - LDEL with destination `l`; LDEL post-increment with source `l` or destination = source.
    - LJSR post-increment with source `l` or `p`.
  - BRAN or BRSR with any non-immediate operand (`control_flow_test.rs:289-292`).
  - An immediate wider than 0xFF together with a shift.
  - A shift count above 15.

## Linker and VM command lines

**Assembler** (`tc/src/bin/assembler.rs:31-73`):
- Flags: `-i/--input-file FILE` and `-o/--output-file FILE`, both required.
- It appends a newline and writes a postcard-serialised object.

**Linker** (`tc/src/bin/linker.rs:88-223`):
- Flags: positional `INPUT FILES…`, `-o/--output-file FILE` and `-s/--segment-offset N`, all required. N is a word
  offset.
- Objects are concatenated in order (`tc/src/types/object.rs:46-103`).
- References resolve against word addresses (byte offset / 2 + segment offset):
  - `.l` = low 16 bits.
  - `.u` = bits 31-16.
  - `.r` = target − reference location as an i16. It panics if out of range.
  - Full address is valid only in data directives (`linker.rs:40-86`).
- The linker can patch symbol references only in opcodes 0x07, 0x10, 0x14, 0x18, 0x1A, 0x1C and 0x1E
  (`linker.rs:170-177`). See Open question 3.
- It writes the raw binary and a debug map (`linker.rs:218-220`).

**VM** (`vm/src/main.rs:102-123`):
- `-p/--program-file FILE`: required.
- `-s/--segment label:hexoffset:hexlength[:file]`: repeatable. Default writable.
- `-r/--register-dump-file`, `-d/--debug` (DAP server), `-e/--enable-video` (with the `video` feature), and
  clap-verbosity `-v/-q`.
- It exits with code 0 when the simulation is stopped by `COPI #0x14FF` (`main.rs:175-176`).

## Diagnostics

**Assembler parse errors** are formatted as `error: parsing failed\n  --> file:line:col` with a source excerpt,
then the nom context chain, then `external error: <msg>` (`tc/src/utils/error_formatter.rs:59-106`; snapshots
`tc/tests/utils/error_formatter_test.rs:151-240`). Messages, abbreviated:
- `Invalid addressing mode for {TAG}: ({modes:?})`. This covers LOAD, STOR, LDEA, LDEL, LJMP, LJSR, MOV and the
  exception-unit/COP instructions.
- `The [{tag}] opcode only supports immediate->register addressing mode (e.g. ADDI y1, #1)`.
- `… register->register or register->register->register addressing mode (e.g. ADDR y1, z2, a2 or SUBR y1, a2)`.
- Context hints that name nonexistent registers: `register (e.g. x1)` and `(e.g. (x1, a))` (`instruction.rs:260,285`).
- `The [{tag}] opcode does not support an explicit status register update source. Only ALU instructions …`. The
  COP variant reads `… Coprocessor calls preserve the status register.`
- `The [{tag}] opcode does not support symbol refs at this time`.
- `… does not support symbol refs or placeholders at this time`.
- `The [{tag}] alias only supports PC-relative immediate or label operands (e.g. BRAN #-3 or BRAN @label)`.
- `The [{tag}] does not support any addressing modes (e.g. NOOP or RETE)`.
- `Aliased register writes are undefined for {INS}: {details}`.
- `Shift definitions can only be in the range of 0-15, got {n}`.
- `Immediate values must fit into 8 bits when using a shift definition ({v} > 0xFF)`.
- SHFT misuse: `The [{tag}] meta instruction only supports a single register operand with a shift definition …`
  and `… does not support a single register operand with a shift definition. Only the SHFT instruction supports that.`
- MOV: `… does not support the [S] status register update source because it has no shift operand. Use [A] or [N].`
  and `Invalid addressing mode for MOVR:. Cannot use a shift … Use SHFT instruction instead.`
- LOAD: `… LOAD only reads memory. Use MOVI …` / `… Use MOVR …` and
  `… LOAD does not support a shift. Use SHFT on the offset register …`.
- COP: `… does not support a destination register field. Use COPI #value/COPR rS instead.` and
  `… does not support shift syntax.`
- ETFR/ETTR: `The return_status_register can only be transferred to r7…` and
  `The return_address can only be transferred to a…`.
- DMA: `… only supports counts in the range -7..7`, `… only supports address registers a, l, or s`,
  `… only supports transfers from a to l`, `… requires an explicit address register operand (a, l, or s)` and
  `… requires explicit source and destination address registers: DMAT a, l, #n`.

**Object-building panics** (`tc/src/data/object.rs:22-41,93-94`):
- `Could not find a value for placeholder name [..]. Make sure it is defined with the .EQU directive.`
- `Immediate value (resolved from [..] placeholder) can only be up to 16 bits` (or `8 bits when using a shift
  definition`).

**Linker panics** (`linker.rs:47-61,130-135,203-208`):
- `Cannot find symbol [..] in symbol definitions. Check you have a label with that name defined in your program.`
- `Offset … does not fit into a 16 bit signed integer (-32768-32767)`.
- `RefType should not be FullAddress when resolving a 16-bit value (try the DQ directive)`.
- `Can't patch address/offset for instruction: …`.

**VM:** a bad segment argument reports `Incorrect format for segment args … <label>:<offset>:<length>:<optional_mapped_file>:<writable>.`
(`vm/src/main.rs:41-47`). The message mentions a `writable` field that the parser does not accept.

---

## Open questions

1. **RETE at level 0.** `link_registers[(current_exception_level - 1)]` underflows when the level is 0. The code
   carries a TODO (`eu-execution.rs:363-369`). Behaviour is a panic, or a wrap in release builds.
2. **ETFR/ETTR index above 7** panics (`eu-execution.rs:232-235`). The assembler masks the index to 4 bits
   (`tc/src/parsers/opcodes/exception.rs:148,181,224,264,297,340`), so `#8`-`#15` assemble and then crash.
3. **The linker cannot patch symbols in LOAD post-increment (0x16) or STOR pre-decrement (0x12).** The parser
   accepts `@sym` there (`load.rs:164-177`; `store.rs:181-194`), but the linker panics (`linker.rs:170-177,203-206`).
4. **Vector-count comments contradict the code.** The comments say "128 32-bit vector addresses", "first 48
   privileged / remaining 80" and "128 user exception vectors" (`eu/definitions.rs:6-9,109-112`). The code has an
   8-bit vector and starts user vectors at 0x60, which gives 160 user vectors.
5. **`MOVI sr, #v` with the default `[A]`.** The value is written, then the ALU flags of v overwrite bits 7-0
   (`write_back.rs:186-189`; `mov.rs:80-82`). The CPU tests use AF=0 (`protected_mode_test.rs:105-155`), so the
   assembler default is untested.
6. **SYNC during a stalled phase-0 fetch** compares the already-advanced phase, so SYNC is low for the
   wait-stated cycles (`cpu/src/lib.rs:286-288` vs `:469-475`).
7. **HALT stops interrupt sampling.** The halt return happens before `raise_hardware_interrupt`
   (`lib.rs:303-318`), so level-sensitive and NMI edges are not observed while halted.
8. **A pending software or COP command can be lost.** When a hardware interrupt dispatches first, exception-unit
   write-back clears `pending_coprocessor_command`. The code has a TODO about this (`eu-execution.rs:476-477`).
9. **The reset test's comments miscount polls.** They say "Polls 6-10: 5 countdown cycles" and "Poll 11: hold
   expired", but the code asserts RSTO for 6 polls after the request poll (`cpu/tests/exceptions/reset.rs:111-142`).
   This is also relevant to the open reset-timing item (ho:216-217): the exception-unit vector fetch itself is a
   6-phase slot with bus reads in phases 0-1 (`eu-execution.rs:323-347`).
10. **The segment `writable` flag is enforced only by `BusPeripheral::write_address`** (`bus/src/lib.rs:212-217`).
    It is not enforced on CPU bus writes (`bus/src/memory_mapped_device.rs:53-60`), so a STOR into the read-only
    PROGRAM segment succeeds.
11. **The post-increment overflow check tests the wrong value.** It checks for a wrap of `displaced + 1`
    (`execution_effective_address.rs:99-104,131-133`). The value actually written back is `ad_l + 1` (`:93`), so a
    real wrap of the source register is not trapped. This matches the handover's "IB-1 still-incomplete" note (ho:213-214).
12. **LDEA/LDEL with a destination field ≥ 4** index register 0x8|(reg<<1). That can exceed 15 and panic
    (`fetch_and_decode.rs:201-202`; `registers.rs:191`). No spec behaviour is defined for these encodings.
13. **Stale header comment.** `definitions.rs:29-35` says segment 0x00 holds "Initial PC, System SP, Base System
    RAM". The code uses only the vector table, with the reset vector at 0x0-0x1.
14. **The ROL/ROR carry rule is marked unverified** in the code (`alu.rs:487-490`). Rotates also use `n mod 16`
    while the shifts clamp at 16 (`alu.rs:372,486,502`). The manual should state which behaviour is intended.
15. **The decimal parser accepts a lone `_`.** It matches `is_a("0123456789_")` (`shared.rs:80`), so `#_` would
    attempt to parse an empty string. The parse fails, but with a generic map_res error.
