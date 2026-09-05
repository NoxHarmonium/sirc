# SIRC-1 architectural facts digest (source of truth for the manual proofreading pass)

Every row cites the Rust code or tests it was read from. Nothing here was taken from `docs/reference/chapters/`.
Authority order: tests > implementation > generated tables > handover "Resolved:" decisions.

## Citation key

| Key | Path |
|---|---|
| DEF | sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs |
| ENC | sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/encoding.rs |
| PU | sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs |
| FD | sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs |
| ALU | sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs |
| EX | sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs |
| MEM | sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/memory_access.rs |
| WB | sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs |
| SH | sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/shared.rs |
| CSH | sirc-vm/peripheral-cpu/src/coprocessors/shared.rs |
| EDEF | sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs |
| EEX | sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs |
| EENC | sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/encoding.rs |
| REG | sirc-vm/peripheral-cpu/src/registers.rs |
| LIB | sirc-vm/peripheral-cpu/src/lib.rs |
| UTIL | sirc-vm/peripheral-cpu/src/util.rs |
| BUS | sirc-vm/peripheral-bus/src/device.rs |
| RU | sirc-vm/peripheral-bus/src/reset_unit.rs |
| T-FAULT | sirc-vm/peripheral-cpu/tests/exceptions/faults.rs |
| T-RESET | sirc-vm/peripheral-cpu/tests/exceptions/reset.rs |
| T-HW | sirc-vm/peripheral-cpu/tests/exceptions/hardware_exceptions.rs |
| T-SW | sirc-vm/peripheral-cpu/tests/exceptions/software_exceptions.rs |
| T-ECOM | sirc-vm/peripheral-cpu/tests/exceptions/common.rs |
| T-ICOM | sirc-vm/peripheral-cpu/tests/instructions/common.rs |
| T-PROT | sirc-vm/peripheral-cpu/tests/instructions/protected_mode_test.rs |
| T-AI | sirc-vm/peripheral-cpu/tests/instructions/arithmetic_immediate_test.rs |
| T-AR | sirc-vm/peripheral-cpu/tests/instructions/arithmetic_register_test.rs |
| T-ASI | sirc-vm/peripheral-cpu/tests/instructions/arithmetic_short_immediate_test.rs |
| T-LDEA / T-LDEL / T-LJMP / T-LOAD / T-STOR | sirc-vm/peripheral-cpu/tests/instructions/{ldea,ldel,ljmp,load,store}_test.rs |
| A-INS | sirc-vm/toolchain/src/parsers/instruction.rs |
| A-SHR | sirc-vm/toolchain/src/parsers/shared.rs |
| A-MOD / A-AI / A-AR / A-BR / A-COP / A-CM / A-EXC / A-LDEA / A-LDEL / A-LJMP / A-LJSR / A-LOAD / A-META / A-STOR | sirc-vm/toolchain/src/parsers/opcodes/{mod,arithmetic_immediate,arithmetic_register,branching,coprocessor,coprocessor_meta,exception,ldea,ldel,ljmp,ljsr,load,meta,store}.rs |
| TA-ADDR / TA-CF / TA-COP | sirc-vm/toolchain/tests/assembler/{addressing_mode,control_flow,coprocessor}_test.rs |
| GEN-I / GEN-R / GEN-S | docs/reference/generated/{immediate,register,short-immediate}-format-encodings.tex |
| HO | docs/reference/manual-handover.md |

## Registers

16 registers of 16 bits, index 0-15; `is_valid_register_range` uses `register_count = 16` (REG:573-579). All fields are `u16` (REG:139-160). Register file default (power-on struct) is all zeros (REG:138; LIB:197-213).

| Idx | Name | Assembler name | Role | Privilege | Reset |
|---|---|---|---|---|---|
| 0 | sr | `sr` (A-INS:88; REG:531) | Status register (REG:141) | Read in protected mode returns only low byte (`SR_REDACTION_MASK` 0x00FF, REG:11; FD:40-48; WB:45-53). Any direct write as destination in protected mode raises PrivilegeViolation before write-back (PU:21-27, 29-51; T-PROT:95-132). Supervisor direct write cannot change bit 13 EA (WB:55-72). | 0 on RSET/reset-vector dispatch (EEX:367-369) |
| 1-7 | r1-r7 | `r1`..`r7` | General purpose | none | not cleared by `CpuPeripheral::reset()` (LIB:524-536) |
| 8,9 | lh, ll | `lh`,`ll`; pair `l` | Link register pair (REG:149-151, 77-78) | lh privileged (PU:21-27) | not cleared (LIB:524-536) |
| 10,11 | ah, al | `ah`,`al`; pair `a` | Address register pair (REG:152-154, 79-80) | ah privileged | not cleared |
| 12,13 | sh, sl | `sh`,`sl`; pair `s` | Stack pointer pair (REG:155-157, 81-82) | sh privileged | not cleared |
| 14,15 | ph, pl | `ph`,`pl`; pair `p` | Program counter pair (REG:158-160, 83-84) | ph privileged | loaded from reset vector (EEX:369; T-RESET:189-191) |

- Address-register pair encoding (2-bit field): 0 = l, 1 = a, 2 = s, 3 = p (REG:76-85). Pair index `k` maps to low register `0x9 | k<<1` and high `0x8 | k<<1` (FD:198-201).
- Only 24 address bits leave the CPU: `ADDRESS_MASK = 0x00FF_FFFF`; the top 8 bits of any `?h` register are ignored when forming an address (REG:8, 291-297, 350-358).
- Privileged destination set is exactly {sr, ah, lh, ph, sh} (PU:21-27). The check fires only when the condition code passes (PU:35-37; T-PROT:135-150) and never for COP opcodes (PU:39-43).
- Address-register-pair write-back in protected mode: if any pending write would change a high word, PrivilegeViolation is raised and no register is written; otherwise only the low words are written (WB:74-98; T-PROT:153-200, 226-268, 345-371, 401-438, 441-480). In supervisor mode both halves are written (WB:92-97; T-PROT:203-224, 374-398). Decision recorded HO:23.
- Internal (not program-visible) registers: `system_ram_offset` (vector-table base) and `pending_coprocessor_command` (REG:162-169).
- Exception-unit registers: `link_registers[8]` (index 7 = fault metadata), `pending_hardware_exceptions`, `pending_fault`, `waiting_for_exception`, `cpu_halted`, `current_exception_level` (REG:590-603; LIB:64). Each link register = `{return_address: u32, return_status_register: u16, saved_exception_level: u8}` (REG:581-588).

## Status register

Bit values from REG:13-41. Low byte is unprivileged; high byte (`SR_PRIVILEGED_MASK` 0xFF00, REG:9) is privileged.

| Bit | Name | Meaning | Set / cleared by |
|---|---|---|---|
| 0 | Z Zero | result == 0 (ALU:561-565) | ALU/shift result write-back only (WB:154-180) |
| 1 | N Negative | result bit 15 set (ALU:566-570) | same |
| 2 | C Carry | carry / borrow / bit shifted out (ALU:571-575) | same |
| 3 | V Overflow | signed overflow (ALU:579-590) | same |
| 4-7 | (none) | no field defined (REG:14-41) | ALU write-back replaces the whole low byte from the ALU/shift status (WB:170-177) |
| 8 | P ProtectedMode | privilege checks active when set (REG:22-24) | cleared on every exception entry (EEX:264; T-FAULT:261-264; T-HW:32-35); restored by RTE (EEX:359; T-HW:43-46); supervisor `LOAD sr` (WB:67) |
| 9-12 | HIE1-HIE4 HardwareInterruptEnable1-4 | enable IRQ lines 1-4 (REG:25-32) | `set_hardware_interrupt_enable` writes bits 9-12 from mask bits 0-3 (REG:507-516); NMI (line 5) is always enabled (REG:477-485) |
| 13 | EA ExceptionActive | CPU inside a handler (REG:33-35) | set on entry (EEX:267); cleared by RTE only when returning to level 0 (EEX:362-365; T-FAULT:864-869); cannot be changed by a direct SR write (WB:59-67) |
| 14 | A TrapOnAddressOverflow | enable SegmentOverflow faults (REG:36-38) | gates EA overflow trap (EX:110-115) and PC-wrap trap (PU:127-137) |
| 15 | T TraceMode | InstructionTrace fault after each instruction (REG:39-40) | cleared on exception entry (EEX:265; T-FAULT:1304-1308), preserved in saved SR (T-FAULT:1310-1314); TRCE pin forces it (LIB:349-353; BUS:184-187) |

- Flag write source is the 2-bit AF field of ALU-class instructions: 0 None (SR unchanged), 1 Alu, 2 Shift, 3 Reserved (unchanged) (DEF:96-102; WB:164-179). The privileged byte is never touched by flag updates (WB:172-177).
- Assembler syntax: `MNEM[a|s|n]|cc` where the bracket letter (case-insensitive) selects AF (A-INS:198-210, 383-396). Default is Alu for ALU mnemonics (A-AI:97-98; A-AR:88-89), Shift for `SHFT` (A-AI:153-154), and non-ALU mnemonics reject the bracket (A-LOAD:29-37; A-STOR:31-39; A-COP:56-58; A-EXC:50-58; TA-COP:123-133).

## Instruction formats

Three formats (DEF:1-27; HO:15). Every instruction is 32 bits = 2 words, stored big-endian (DEF:40-41; ENC:49, 402). Bits 31:26 are the opcode; bits 3:0 are the condition code in every format (ENC:47-59).

| Format | 31:26 | 25:22 | 21:14 | 13 | 12:10 | 9:6 | 5:4 | 3:0 | Source |
|---|---|---|---|---|---|---|---|---|---|
| Immediate | opcode | register | value[15:8] (value occupies 21:6) | — | — | — | AF | cond | ENC:115-137, 381-403 |
| Short immediate | opcode | register | value 8-bit | shift operand | shift type | shift count | AF | cond | ENC:168-203, 437-466 |
| Register | opcode | r1 | r2 (21:18), r3 (17:14) | shift operand | shift type | shift count | AF | cond | ENC:232-260, 498-528 |

- Opcode-to-format map: 0x00-0x0F Immediate; 0x10-0x1F even = Immediate, odd = Register; 0x20-0x2F Short immediate; 0x30-0x3F Register (ENC:272-290; FD:23-38). All 64 opcodes decode (ENC:726-739); no invalid-opcode detection for CPU instructions (EDEF:45-48).
- AF (bits 5:4) is dual-purpose: decoded simultaneously as `sr_src` (status update source) and as the address-register pair index (FD:198-199, 217). ALU opcodes use it as `sr_src`; memory / LDEA / LDEL opcodes use it as the source address register (A-LOAD:58; A-STOR:48; A-LDEA:48).
- In Immediate format the shift fields do not exist; decode forces shift = None, count 0 (FD:172-176), and `sr_shift` = 0 (FD:188).
- Worked encodings: GEN-I:8-42 (12 immediate examples), GEN-R:8-51 (15 register examples incl. `SHFT r1, ASL #3` = 0x94400CE0), GEN-S:8-27 (7 short-immediate examples). Doctests: ENC:103-111, 156-164, 219-229, 370-376, 423-432, 483-493.
- All-zero memory decodes as `ADDI` register 0, value 0, AF 0, cond Always (ENC:741-755) — see Open question 2.

## Opcode map

Mnemonics from the assembler parsers; semantics from EX/MEM/WB. `D` = register field / r1, `S1` = r2, `S2` = r3, `imm` = value field, `A` = address register selected by AF, `B` = address register selected by the register field (low 2 bits).

| Op | Enum (DEF:207-280) | Mnemonic / syntax | Fmt | Semantics |
|---|---|---|---|---|
| 0x00 | AddImmediate | `ADDI rD, #imm16` (A-AI:17) | I | D = D + imm; NZCV |
| 0x01 | AddImmediateWithCarry | `ADCI rD, #imm16` | I | D = D + imm + C |
| 0x02 | SubtractImmediate | `SUBI rD, #imm16` | I | D = D - imm |
| 0x03 | SubtractImmediateWithCarry | `SBCI rD, #imm16` | I | D = D - imm - C |
| 0x04 | AndImmediate | `ANDI` | I | D = D & imm |
| 0x05 | OrImmediate | `ORRI` | I | D = D \| imm |
| 0x06 | XorImmediate | `XORI` | I | D = D ^ imm |
| 0x07 | LoadRegisterFromImmediate | `LOAD rD, #imm16` (A-LOAD:41-49, AF=0) | I | D = imm |
| 0x08 | _Undocumented0x08 | none | I | ADD flags only, D unchanged (T-AI:579-645) |
| 0x09 | _Undocumented0x09 | none | I | ADC flags only (T-AI:651-750) |
| 0x0A | CompareImmediate | `CMPI rD, #imm16` | I | SUB flags only (T-AI:756-824) |
| 0x0B | _Undocumented0x0B | none | I | SBC flags only (T-AI:830-930) |
| 0x0C | TestAndImmediate | `TSAI` | I | AND flags only (T-AI:936-964) |
| 0x0D | _Undocumented0x0D | none | I | OR flags only (T-AI:970-998) |
| 0x0E | TestXorImmediate | `TSXI` | I | XOR flags only (T-AI:1004-1032) |
| 0x0F | CoprocessorCallImmediate | `COPI #imm16` (A-COP:63-77) | I | pending COP command = imm (WB:299-301) |
| 0x10 | StoreRegisterToIndirectImmediate | `STOR (#n, A), rD` (A-STOR:41-50) | I | mem[A + n] = D (MEM:65-74) |
| 0x11 | StoreRegisterToIndirectRegister | `STOR (rS2, A), rS1[, shift]` (A-STOR:112-156) | R | mem[A + S2] = shift(S1) |
| 0x12 | StoreRegisterToIndirectImmediatePreDecrement | `STOR -(#n, A), rD` | I | mem[A + n - 1] = D; A.low -= 1 (EX:88-93; T-STOR:250-293) |
| 0x13 | StoreRegisterToIndirectRegisterPreDecrement | `STOR -(rS2, A), rS1[, shift]` | R | mem[A + S2 - 1] = shift(S1); A.low -= 1 (T-STOR:184-246) |
| 0x14 | LoadRegisterFromIndirectImmediate | `LOAD rD, (#n, A)` | I | D = mem[A + n] (T-LOAD:63-108) |
| 0x15 | LoadRegisterFromIndirectRegister | `LOAD rD, (rS2, A)[, shift]` | R | D = shift(mem[A + S2]) (WB:313; T-LOAD:112-175) |
| 0x16 | LoadRegisterFromIndirectImmediatePostIncrement | `LOAD rD, (#n, A)+` | I | D = mem[A + n]; A.low += 1 (EX:101-105; T-LOAD:250-294) |
| 0x17 | LoadRegisterFromIndirectRegisterPostIncrement | `LOAD rD, (rS2, A)+[, shift]` | R | D = shift(mem[A + S2]); A.low += 1 (T-LOAD:179-246) |
| 0x18 | LoadEffectiveAddressFromIndirectImmediate | `LDEA B, (#n, A)` | I | B = (A.high, A.low + n) (WB:203-215; T-LDEA:21-74) |
| 0x19 | LoadEffectiveAddressFromIndirectRegister | `LDEA B, (rS2, A)` | R | B = (A.high, A.low + S2) (T-LDEA:78-138) |
| 0x1A | LoadEffectiveAddressFromIndirectImmediatePreDecrement | `LDEA B, -(#n, A)` | I | B = (A.high, A.low + n - 1); A.low -= 1 (WB:216-236; T-LDEA:142-180) |
| 0x1B | LoadEffectiveAddressFromIndirectRegisterPreDecrement | `LDEA B, -(rS2, A)` | R | as 0x1A with S2 (T-LDEA:184-228) |
| 0x1C | LoadEffectiveAddressAndLinkFromIndirectImmediate | `LDEL B, (#n, A)` | I | l = PC+2; B = (A.high, A.low + n) (WB:237-257; T-LJMP:292-344) |
| 0x1D | LoadEffectiveAddressAndLinkFromIndirectRegister | `LDEL B, (rS2, A)` | R | l = PC+2; B = (A.high, A.low + S2) (T-LJMP:436-488) |
| 0x1E | LoadEffectiveAddressAndLinkFromIndirectImmediatePostIncrement | `LDEL B, (#n, A)+` | I | l = PC+2; B = (A.high, A.low + n); A.low += 1 (WB:258-284; T-LDEL:16-56) |
| 0x1F | LoadEffectiveAddressAndLinkFromIndirectRegisterPostIncrement | `LDEL B, (rS2, A)+` | R | as 0x1E with S2 (T-LDEL:60-106) |
| 0x20-0x26 | Add/Adc/Sub/Sbc/And/Or/Xor ShortImmediate | `ADDI rD, #imm8, shift` etc. (A-AI:31-47, 206-269) | S | D = shift(D) op imm8 (FD:189-192) |
| 0x25 | OrShortImmediate | also `SHFT rD, shift` (imm 0, AF=Shift) (A-AI:38-40, 139-159) | S | D = shift(D); flags from shift |
| 0x27 | LoadRegisterFromShortImmediate | no assembler form (A-AI:31-47; A-LOAD:73) | S | D = imm8 (T-ASI:464-500) |
| 0x28,0x29,0x2B,0x2D | _Undocumented | none | S | flags-only variants |
| 0x2A/0x2C/0x2E | Compare/TestAnd/TestXor ShortImmediate | `CMPI/TSAI/TSXI rD, #imm8, shift` | S | flags only |
| 0x2F | CoprocessorCallShortImmediate | no assembler form (A-COP:49-140) | S | COP command = imm8 only (T-PROT:530-547) |
| 0x30-0x36 | Add..Xor Register | `ADDR rD, rS2` (r1=r2=D) or `ADDR rD, rS1, rS2` [, shift] (A-AR:14-27, 91-182) | R | D = shift(S1) op S2 (FD:180-186) |
| 0x37 | LoadRegisterFromRegister | `LOAD rD, rS` (r2=0, AF=0; shift rejected) (A-LOAD:112-144) | R | D = S2; shift on S1 ignored (ALU:307-317; T-AR:497-519) |
| 0x38,0x39,0x3B,0x3D | _Undocumented | none | R | flags-only variants |
| 0x3A/0x3C/0x3E | Compare/TestAnd/TestXor Register | `CMPR/TSAR/TSXR rD, rS[, shift]` | R | flags only |
| 0x3F | CoprocessorCallRegister | `COPR rS` (r1=r2=0, r3=S) (A-COP:110-127) | R | COP command = S2 value (ALU Load of b) |

- ALU decode rule: `alu_code = opcode & 0xF`; operation = `alu_code & 0x7` (0 Add, 1 Adc, 2 Sub, 3 Sbc, 4 And, 5 Or, 6 Xor, 7 Load); bit 3 set = "simulate" (result discarded, flags kept) except 0xF (EX:61-67; ALU:28-39, 535-541). Write-back class: 0x0-0x7 AluToRegister, 0x8-0xE AluStatusOnly, 0xF CoprocessorCall (WB:132-151).
- Operand sourcing: Immediate format `a = D, b = imm16` (FD:188); Short immediate `a = shift(D), b = imm8 zero-extended` (FD:189-192); Register `a = shift(S1), b = S2` (FD:180-186). The shift never applies to the immediate or to S2.
- Memory ops: effective address low = `A.low + b` (b = imm16 or S2) (EX:77-78); high = `A.high` (MEM:58, 67). Store data = `a` (MEM:68). Bus data returns one cycle later and is written at WB (PU:108, 119; WB:313).
- Pre-decrement: address = A.low + b - 1, A.low = A.low - 1 (EX:88-93). Post-increment: address = A.low + b, A.low = A.low + 1 (EX:101-105). The increment/decrement is always exactly 1 word regardless of displacement (EX:81).
- LDEL link value = address of the next instruction (`npc` = ph, pl+2) (FD:203-205; WB:243-248).
- Branch aliases (A-BR:21-27, 75-83; A-META:66-79; A-LJMP:63-71; A-LJSR:63-82): `BRAN #off` = `LDEA p, (#off, p)` (0x18, AF=3); `BRSR #off` = `LDEL p, (#off, p)` (0x1C); `LJMP A[, #n|rN]` = `LDEA p, (#n|rN, A)` (0x18/0x19); `LJSR A[, #n|rN]` = `LDEL p, (...)` (0x1C/0x1D); `LJSR (#n, A)+` / `(rN, A)+` = 0x1E/0x1F; `RETS` = `LDEA p, (#0, l)` (0x18, AF=0); `NOOP` = `ADDI` with register 0, value 0, AF 0 (A-META:80-95). Verified TA-CF:157-286; GEN-I:32-36; GEN-R:32-36.
- Branch base is the branching instruction's own PC (decoded before pl advances): pl=0xFAC0, offset 0xE lands at 0xFACE (T-LJMP:225-232; PU:124, 151); BRSR link = pl+2 = 0xFAC2 (T-LJMP:292-300). Offsets wrap within the segment (T-LJMP:274-289).
- Exception-unit meta instructions all lower to `COPI` (0x0F) with these values (A-EXC:64-367): `EXCP #v` = 0x1100 \| (v & 0xFF); `WAIT` = 0x1900; `RETE` = 0x1A00; `RSET` = 0x1B00; `ETFR #n` = 0x1C30 \| n; `ETFR r7, #n` = 0x1C20 \| n; `ETFR a, #n` = 0x1C10 \| n; `ETTR #n` = 0x1D30 \| n; `ETTR #n, r7` = 0x1D20 \| n; `ETTR #n, a` = 0x1D10 \| n.
- COP command word: bits 15:12 coprocessor ID, 11:8 opcode, 7:0 value (LIB:70-73; EEX:38-65; EENC:1-15).
- Simulator-only: `COPI #0x14FF` ends the simulation (PU:153-160).

## Flag effects

Common rule `set_alu_bits` (ALU:545-593): Z = (result == 0); N = result bit 15; C = the operation's carry input; V = set iff a signed-overflow triple is supplied and `sign(i1) == sign(i2) && sign(result) != sign(i1)`, otherwise cleared. Every ALU/shift update writes all four bits (no bit is ever "unchanged" once an update happens); unchanged only when AF = None/Reserved or the instruction class does not update flags (WB:164-179, 195-202).

| Operation | N | Z | C | V | Source |
|---|---|---|---|---|---|
| ADD / ADDI (0x0,0x20,0x30) | result | result | unsigned carry out | signed overflow of (a, b) | ALU:81-90; T-AI:62-128 |
| ADC (0x1) | result | result | carry from either add step | overflow of (a, b, final) | ALU:132-153; T-AI:134-233; T-ASI:202-218 |
| SUB / CMP (0x2, 0xA) | result | result | **set when a borrow occurs (a < b unsigned)** | overflow of (a, !b, result) | ALU:195-206; T-AI:260-273 (0x5FFF-0xFFFF sets C); HO:19 |
| SBC (0x3, 0xB) | result | result | borrow from either step; incoming C is subtracted as a borrow | overflow of (a, !b, result) | ALU:245-267; T-AI:334-365; HO:19 |
| AND / TSA (0x4, 0xC) | result | result | cleared | cleared | ALU:271-281; T-AI:419-447; HO:21 |
| OR (0x5, 0xD) | result | result | cleared | cleared | ALU:283-293; T-AI:453-481 |
| XOR / TSX (0x6, 0xE) | result | result | cleared | cleared | ALU:295-305; T-AI:487-515 |
| LOAD op (0x7 family) with AF=Alu | b | b | cleared | cleared (triple (a,b,b) can never satisfy the rule) | ALU:307-317; T-AR:497-534 (0xFACE sets N) |
| Flags-only ops (0x8-0xE) | as the underlying op | | | | EX:63-67; T-AI:579-1032 |

- Compare semantics: CMP computes D - operand and discards the result; the result register is unchanged (T-AI:756-824).
- The assembler emits AF = None for every `LOAD` form (A-LOAD:47, 128), so assembled `LOAD` never updates flags; immediate `LOAD` with AF = Shift clears N Z C V because the immediate-format shift status is constant 0 (FD:188; T-AI:29-33, 521-573). See Open question 9.
- Memory loads never update flags regardless of AF (WB:312-315). Stores, LDEA, LDEL, COP never update flags (WB:195, 203-301).
- ALU result is also gated by the condition code: false condition = no register, flag, memory, or address-register effect (EX:33-35; MEM:26-28; WB:128-130; T-LJMP:256-270).

## Shift types

Field values from DEF:71-81; mnemonics from A-INS:359-381; behaviour from ALU:322-508. Shift is applied during decode to operand `a` (FD:179-193), or to the loaded word for memory loads (WB:313). Shift status bits become the SR low byte only when AF = Shift (WB:175-177).

| Enc | Name | Mnemonic | Result | C | V | Notes |
|---|---|---|---|---|---|---|
| 0 | None | `NUL` | operand | 0 | 0 | N, Z from operand (ALU:330-339) |
| 1 | LogicalLeftShift | `LSL` | a << n | last bit shifted out (bit 16 of 32-bit product) | 0 | ALU:370-390; T-AR:542-642 |
| 2 | LogicalRightShift | `LSR` | a >> n, zero fill | last bit shifted out | 0 | ALU:393-414; T-AR:645-745 |
| 3 | ArithmeticLeftShift | `ASL` | a << n | as LSL | set iff sign of result differs from sign of a | ALU:417-443; T-AR:769-781 (0xB333 ASL 1 sets C and V) |
| 4 | ArithmeticRightShift | `ASR` | a >> n with bit 15 held | last bit shifted out | never set (sign preserved) | ALU:446-476; T-AR:851-951; T-ASI:852-864 |
| 5 | RotateLeft | `RTL` | a rotl n | bit 0 of result | 0 | ALU:478-492 |
| 6 | RotateRight | `RTR` | a rotr n | bit 15 of result | 0 | ALU:494-508 |
| 7 | Reserved | none | operand unchanged | 0 | 0 | all shift-status bits 0 (ALU:358-360) |

- Shift operand bit (bit 13): 0 = immediate count in bits 9:6 (0-15, `MAX_SHIFT_COUNT = 15`, DEF:43), 1 = bits 9:6 name a register whose full 16-bit value is the count (DEF:85-89; FD:60-69; A-SHR:208-224). Syntax `LSL #n` or `LSL rN` (A-INS:213-224).
- Immediate counts are masked to 4 bits at encode time: 16 -> 0, 17 -> 1, 255 -> 15 (ENC:339-342; T-ASI:659-700, 865-906).
- Shift functions clamp the count to 16 for LSL/LSR/ASL/ASR (ALU:372, 395, 425, 457); rotates use Rust `rotate_left/right` (count taken mod 16) (ALU:479, 495). Register-operand counts above 15 are untested (T-AR:953-959).
- Shift on `LOAD rD, rS` (0x37) is meaningless because LOAD takes operand b; the assembler rejects it and directs users to `SHFT` (A-LOAD:134-144; T-AR:504-509).

## Addressing modes

Decision: seven addressing modes; operandless meta-instructions are not a mode (HO:17). The parser distinguishes ten syntactic operand forms (A-INS:85-106); the grouping into seven below is this digest's inference (immediate-vs-register displacement collapsed per auto-update form), so proofreaders should check the manual's grouping against the syntax list, not this count.

| # | Mode | Syntax (A-INS:85-106) | Encoding | Accepted by |
|---|---|---|---|---|
| 1 | Immediate | `#n`, `@label`, placeholder | value field | ADDI family (16- or 8-bit + shift), LOAD 0x07, COPI, BRAN/BRSR, EXCP/ETFR/ETTR/DMA/maths meta |
| 2 | Register direct | `rN`, `lh`..`pl`, `sr` | 4-bit register field | ALU register/immediate forms, LOAD 0x37, COPR, STOR source, LOAD destination, ETFR/ETTR `r7` |
| 3 | Address register direct | `l`, `a`, `s`, `p` | 2-bit pair index in register field or AF | LDEA/LDEL destination; LJMP/LJSR source; DMAR/DMAW/DMAT; ETFR/ETTR `a` |
| 4 | Indirect, immediate displacement | `(#n, A)`; shorthand `(A)` = `(#0, A)` (TA-ADDR:41-84) | AF = A, value = n | LOAD 0x14, STOR 0x10, LDEA 0x18, LDEL 0x1C, LJMP/LJSR |
| 5 | Indirect, register displacement | `(rN, A)` | AF = A, r3 = N | LOAD 0x15, STOR 0x11, LDEA 0x19, LDEL 0x1D, LJMP/LJSR |
| 6 | Indirect post-increment | `(#n, A)+`, `(rN, A)+`; shorthand `(A)+` | opcodes 0x16/0x17, 0x1E/0x1F | LOAD, LDEL, LJSR only (TA-ADDR:165-172) |
| 7 | Indirect pre-decrement | `-(#n, A)`, `-(rN, A)`; shorthand `-(A)` | opcodes 0x12/0x13, 0x1A/0x1B | STOR, LDEA only (TA-ADDR:165-172) |
| mod | Shift definition (operand modifier, not a mode) | `LSL #n`, `LSL rN` | bits 13:6 | short-immediate ALU, register ALU, SHFT, LOAD/STOR register-displacement forms |

- Illegal families rejected by the assembler: `LOAD r1, -(a)`, `STOR (a)+, r1`, `LDEA p, (a)+`, `LDEL p, -(a)`, `LJMP (a)`, `LJSR (a)` (TA-ADDR:164-172).
- Aliased-write rejections (A-MOD:19-48; TA-CF:294-310): LDEA pre-decrement with destination == source (A-LDEA:135-141, 191-197); LDEL destination `l` (A-LDEL:51-57), LDEL post-increment with source `l` or destination == source (A-LDEL:144-157); LJSR post-increment from `l` or `p` (A-LJSR:154-167); LOAD post-increment into a half of the auto-updated pair (A-LOAD:240-246); STOR pre-decrement from a half of the auto-updated pair (A-STOR:160-166). Hardware behaviour in these cases is unspecified; write-back applies the source update first and the destination write last (WB:216-236).
- Symbol references: BRAN/BRSR resolve as PC-relative offsets (`RefType::Offset`, A-BR:96-107; TA-CF:261-285); all other immediates resolve to the symbol's lower word (`RefType::LowerWord`, A-LDEA:81-96; A-LOAD:89-100; A-STOR:80-95; A-LJMP:97-108). ALU immediates and EXCP/ETFR/ETTR/DMA reject symbol refs (A-AI:194-202; A-EXC:80-89; A-CM:33-41).
- Short-immediate values must fit in 8 bits when a shift is given (A-AI:214-222); short immediates are zero-extended (FD:191; HO:54).
- Two-operand register ALU form `ADDR rD, rS` sets r1 = r2 = D (A-AR:91-113; GEN-R:38-39).

## Condition codes

Encoding DEF:49-67; predicate DEF:115-157; assembler suffix `|cc` after the mnemonic (A-INS:317-357, 391-396). Evaluated at decode against the SR as it stood before the instruction (FD:228).

| Enc | Name | Mnemonic | True when |
|---|---|---|---|
| 0 | Always | `AL` (or no suffix) | always |
| 1 | Equal | `==` | Z |
| 2 | NotEqual | `!=` | !Z |
| 3 | CarrySet | `CS` | C |
| 4 | CarryClear | `CC` | !C |
| 5 | NegativeSet | `NS` | N |
| 6 | NegativeClear | `NC` | !N |
| 7 | OverflowSet | `OS` | V |
| 8 | OverflowClear | `OC` | !V |
| 9 | UnsignedHigher | `HI` | C && !Z |
| A | UnsignedLowerOrSame | `LO` | !C \|\| Z |
| B | GreaterOrEqual | `>=` | N == V |
| C | LessThan | `<<` | N != V |
| D | GreaterThan | `>>` | !Z && N == V |
| E | LessThanOrEqual | `<=` | Z \|\| N != V |
| F | Never | `NV` | never |

Note: with the borrow convention above, `HI`/`LO` follow the ARM definitions copied at DEF:114 only if the manual defines C after subtraction as "borrow", i.e. `HI` = no borrow and not equal.

## Exceptions

### Vectors and priorities

Vector numbers are 8-bit; vector-table address = `system_ram_offset | (vector * 2)`; high word at that address, low word at +1 (EEX:304-325; T-ECOM:133-150; HO:349-350). Priority level = `7 - (vector >> 4)` for hardware and faults; software exceptions are level 1 (EEX:79-84; EDEF:135-145). Link register index = level - 1 (EEX:258); index 7 holds fault metadata (LIB:64).

| Vector | Name (EDEF) | Level / link reg | Raised by |
|---|---|---|---|
| 0x00 | reset vector | — | `Reset` EU opcode (LIB:425, 535; EEX:367-370) |
| 0x01 | BUS_FAULT (EDEF:21) | 7 / LR6 | BERR pin (LIB:441-449); T-FAULT:245-267 |
| 0x02 | ALIGNMENT_FAULT (EDEF:28) | 7 | odd pl at any phase (PU:83-86); T-FAULT:606-637 |
| 0x03 | SEGMENT_OVERFLOW_FAULT (EDEF:37) | 7 | EA add wraps with SR.A set (EX:110-115), or pl wraps with SR.A set, raised at next fetch (PU:92-97, 127-137); T-FAULT:640-774 |
| 0x04 | INVALID_OPCODE_FAULT (EDEF:49) | 7 | COP ID not 0 or 1 (LIB:375-394); T-FAULT:777-823 |
| 0x05 | PRIVILEGE_VIOLATION_FAULT (EDEF:56) | 7 | decode privilege check (PU:141-145) or protected pair write-back (WB:86-90); T-FAULT:1179-1215 |
| 0x06 | INSTRUCTION_TRACE_FAULT (EDEF:60) | 7 | T bit sampled at phase 0, raised at phase 5 (LIB:349-353, 397-406); T-FAULT:1265-1317 |
| 0x07 | LEVEL_FIVE_HARDWARE_EXCEPTION_CONFLICT (EDEF:72) | 7 | NMI while already at level 6 (EEX:180-184, 242-249); T-FAULT:1218-1262 |
| 0x08 | DOUBLE_FAULT_VECTOR (EDEF:87) | 7 | fault while `current_exception_level >= 7` (LIB:164-171) or fault-vector fetch fails (LIB:475-492); T-FAULT:472-519, 885-998 |
| 0x09 | BUS_PROTECTION_FAULT (EDEF:94) | 7 | BPER pin; BERR wins over BPER (LIB:441-449); T-FAULT:340-362 |
| 0x0A-0x0F | reserved (EDEF:96) | | |
| 0x10 | LEVEL_FIVE_HARDWARE_EXCEPTION (NMI) | 6 / LR5 | interrupt bit 4 (EEX:103, 122-140) |
| 0x20 | LEVEL_FOUR | 5 / LR4 | bit 3 |
| 0x30 | LEVEL_THREE | 4 / LR3 | bit 2 |
| 0x40 | LEVEL_TWO | 3 / LR2 | bit 1 |
| 0x50 | LEVEL_ONE | 2 / LR1 | bit 0 (EEX:114-119 doctest) |
| 0x60-0xFF | user / software (EDEF:112-113) | 1 / LR0 | `EXCP #v` (A-EXC:64-79); T-SW:29-72 |

- Fault enum codes (EDEF:166-177): Bus 1, Alignment 2, SegmentOverflow 3, InvalidOpCode 4, PrivilegeViolation 5, InstructionTrace 6, LevelFiveInterruptConflict 7, DoubleFault 8, BusProtection 9 (same numbers as the vectors).
- Interrupt pins: `interrupt_assertion` bit 0 = IRQ1 ... bit 3 = IRQ4, bit 4 = NMI (BUS:129-133; REG:458-462). Level = `ilog2(highest set bit) + 2` (EEX:96-105; UTIL:18-33). Vector = `(6 - (level - 1)) << 4` (EEX:122-140).
- Interrupts are latched only if enabled at assertion time (LIB:499-518), re-checked against the enable mask at dispatch (EEX:186-188), and dispatched only at phase 0 (LIB:326-341). A pending interrupt is serviced only if its level exceeds `current_exception_level` (EEX:182-183); lower-priority ones stay pending and dispatch in priority order after each RTE (T-HW:50-105); a higher-priority interrupt pre-empts a running handler (T-HW:155-207); the same line re-triggers after RTE if still asserted (T-HW:108-152). Pending faults win over interrupts, which win over a pending COP command (EEX:146-211).
- NMI is re-entrant only in the sense that a second NMI at level 6 raises LevelFiveInterruptConflict (EEX:180-184, 242-249); at level 7 it is ignored (EEX:252-256).

### Entry, saved state, return

- Entry (EEX:231-271), committed at EU phase 3 after the vector target has been fetched: `link_registers[level-1] = {return_address = current PC, return_status_register = SR, saved_exception_level}`; clear P; clear T; set EA; PC = fetched vector value; `current_exception_level = level`. Condition flags, HIE bits and A bit are untouched. Lower-or-equal-priority non-fault exceptions are ignored (EEX:252-256); level 7 always enters. Decision recorded HO:386-390.
- Return address: for regular exceptions it is the instruction after the one that completed (hardware/software: T-SW:60-66 returns to 0x2 after EXCP at 0x0; invalid opcode: T-FAULT:939-942, 989-992; trace: T-FAULT:1299-1302). See Open question 1 for aborting faults.
- RTE (`ReturnFromException`, EEX:348-366): PC and SR restored from `link_registers[current_level-1]`; `current_exception_level = saved_exception_level`; EA cleared only if that is 0. Verified T-FAULT:852-869; T-HW:37-46.
- EU dispatch of Fault/HardwareException/SoftwareException, RTE, WAIT, RSET, ETFR, ETTR each occupies one 6-cycle slot: phase 0 vector fetch (address vector*2, BAT = ExceptionVectorFetch), phase 1 second word (+1), phase 2 decode, phase 3 execute, phase 5 cleanup (EEX:302-446; T-ECOM:125-167).
- ETFR/ETTR value byte: bits 5:4 register select (0 none, 1 `return_address` <-> `a`, 2 `return_status_register` <-> `r7`, 3 both), bits 3:0 link register index 0-7 (EDEF:1-3; EEX:213-229, 371-415). ETFR copies link -> registers, ETTR copies registers -> link.
- Fault metadata (link register 7, LIB:64): `return_address` = bus address at the fault (LIB:176), or the offending COP command word for InvalidOpCode (LIB:386; T-FAULT:819-822, 943-947), or the vector address for vector-fetch faults (T-FAULT:411-414); `return_status_register` bit layout: 2:0 bus access type, 3 double-fault flag, 7:4 fault, 11:8 original fault (LIB:81-88, 120-145; T-FAULT:1087-1166). `original_fault == fault` when not a double fault (T-FAULT:927-938). The register is clobbered on every fault (LIB:172-186).
- Bus access type codes (BUS:19-29): 0 None, 1 InstructionFetch, 2 DataRead, 3 DataWrite, 4 ExceptionVectorFetch, 5 DmaReadBurst, 6 DmaWriteBurst, 7 Reserved.
- Double / triple fault chain (LIB:147-188, 451-497): fault while at level 7 -> DoubleFault (vector 0x08) with metadata; BERR/BPER while fetching any fault vector -> DoubleFault; BERR/BPER while fetching the double-fault vector -> `reset_requested` asserted every cycle until `reset()` (T-FAULT:521-603). BERR/BPER while fetching a normal exception or reset vector -> the original exception is abandoned and a Bus/BusProtection fault is raised with BAT = ExceptionVectorFetch (LIB:494-496; T-FAULT:365-469; HO:400-409).
- Software exceptions are accepted only at level 0; `EXCP` inside any handler is discarded (command cleared, no vector fetch) (LIB:326-337; T-SW:74-143; HO:392-394). The assembler masks the vector to 8 bits (A-EXC:73).
- `WAIT` sets `waiting_for_exception`; the CPU idles (no bus activity) until an enabled interrupt (LIB:300-302, 514-515) or reset (T-RESET:227-248).

### Protected-mode rules

- Privileged operations (EDEF:51-56; PU:29-51): writing sr/ah/lh/ph/sh as an instruction destination; COP commands whose opcode nibble is > 7 (PU:46-48; T-PROT:484-527); address-pair write-back that would change a high word (WB:80-90).
- COP privilege is decided on operand b (imm16 for COPI, r3 for COPR); short-immediate COPI can never be privileged and never reaches the EU (T-PROT:530-547).
- P is cleared on entry to every exception including faults, so handlers run in supervisor mode (T-FAULT:261-264, 631-634; T-SW:53-56).
- Bus `PROT` pin mirrors SR.P every cycle (LIB:216-225; BUS:194-197).

### Reset

- `CpuPeripheral::reset()` (LIB:524-536): clears the pending bus request, halt latch, `reset_pending`, `waiting_for_exception`, `pending_fault`, `pending_hardware_exceptions`, `current_exception_level`; sets phase 0; seeds `pending_coprocessor_command` with the Reset cause (COP 1, opcode 0xB, vector 0). Verified T-RESET:252-278. General registers, address pairs, and link registers are not touched.
- The Reset EU opcode then executes as a normal 6-cycle dispatch: vector 0x00 high word from `system_ram_offset + 0`, low word from `+1`, BAT = ExceptionVectorFetch (EEX:303-323; T-RESET:176-191); at phase 3 SR = 0 and PC = fetched value (EEX:367-370).
- Software `RSET`: after write-back of the COPI, the CPU asserts `reset_requested` (LIB:422-431); the reset unit then holds RSTO for 6 cycles (assertion cycle + 5 countdown, RU:22-26; T-RESET:84-147) before the EU fetches the vector. Hardware RSTI: same 6-cycle hold, CPU idle, stall aborted (T-RESET:149-225, 280-323).

## Coprocessors

| ID | Unit | Presence | Notes |
|---|---|---|---|
| 0 | Processing unit | always | `COPROCESSOR_ID = 0` (PU:62); a command with ID 0 simply runs the PU (REG:167-169) |
| 1 | Exception unit | always | `COPROCESSOR_ID = 1` (EEX:280); opcodes 0x0 None, 0x1 SoftwareException, 0x9 WaitForException, 0xA ReturnFromException, 0xB Reset, 0xC TransferFromRegister, 0xD TransferToRegister, 0xE Fault (internal), 0xF HardwareException (internal) (EDEF:149-162) |
| 2 | DMA | not implemented in peripheral-cpu | assembler emits: `DMAR A, #n` = opcode 0x8, `DMAW A, #n` = 0x9, operand bits 7:6 pair (a=00, l=01, s=10; p rejected), bit 5 = negative count, bits 2:0 = magnitude 0-7; `DMAT a, l, #n` = 0xA with count 0-255 (A-CM:14-19, 82-91, 140-222; TA-COP:135-144, 154-184). Opcodes >= 8 are privileged (PU:48) |
| 3 | Maths | not implemented in peripheral-cpu | `MULU` 0x3000, `MULS` 0x3100, `DIVU` 0x3200, `DIVS` 0x3300 (A-CM:233-248; TA-COP:146-152); unprivileged |
| 2-15 | any absent unit | — | at the next phase 0 the CPU raises InvalidOpCode, records the command word in fault metadata `return_address`, and clears the command (LIB:375-394; T-FAULT:776-823); the handler may RTE to the instruction after the COP (T-FAULT:825-882). Intended use is software emulation / forward compatibility (EDEF:38-48) |

- Every COP instruction costs two 6-cycle slots: the PU instruction, then the coprocessor dispatch at the following phase 0 (T-FAULT:781-791; T-SW:41-51). The command register is cleared at the EU's phase 5 (EEX:443-444).
- Coprocessor calls preserve the SR (A-COP:14-23) and take a condition code (`COPI|== #0x1900`, TA-COP:62-74); a false condition leaves no pending command (T-PROT:497-509).

## Bus and timing

- Fixed pipeline: `CYCLES_PER_INSTRUCTION = 6` (LIB:66-67); phases 0 InstructionFetchLow, 1 InstructionFetchHigh, 2 InstructionDecode, 3 ExecutionEffectiveAddress, 4 MemoryAccess, 5 WriteBack (CSH:6-14; LIB:520-522). Every PU instruction, of every class, occupies exactly these six cycles plus any wait states; there is no per-class cycle table in the code.
- Phase 0: address = PC, read, BAS asserted, BAT = InstructionFetch; phase 1: address = PC + 1, same (PU:89-117). PC advances by 2 at phase 2 (PU:151). Phase 4 issues the data read/write (BAT DataRead/DataWrite, address (A.high, ea)) (MEM:56-74); the read data is consumed at phase 5 (WB:313).
- Wait states: when BAS is asserted the request is re-driven every cycle and the phase does not advance until BACK, BERR, or BPER is seen (LIB:106-110, 259-276; T-RESET:283-299).
- SYNC is asserted during phase 0 of PU instructions and EU dispatches (LIB:266-268, 416-420; BUS:188-191). HALT is sampled at phase 0 and stalls the CPU while asserted (LIB:283-293; BUS:180-183). TRCE forces trace sampling at phase 0 (LIB:349-353). Interrupt pins are latched every cycle but only dispatched at phase 0 (LIB:295-298, 326-341); the handover records them as level-sensitive, instruction-boundary sampled (HO:473-479).
- Memory is word-addressed (16-bit words; T-STOR:105-107) and words are big-endian on the byte bus (T-STOR:176; ENC:49).
- Exception dispatch slot: vector high word at phase 0, low word at phase 1 (BAT ExceptionVectorFetch), entry at phase 3 (EEX:303-334; T-ECOM:133-153). A software exception therefore starts its handler 12 cycles after the EXCP fetch began; a data bus fault detected at phase 4/5 starts its handler 6 cycles later (T-FAULT:124-184).
- Reset output hold: 6 cycles of RSTO (T-RESET:84-147, 166-182); reset-vector fetch begins on the cycle after the hold (T-RESET:133-146, 176-187).
- Cycle counts for DMA bursts, wait-state limits, electrical timing, and any per-instruction figures beyond "6 cycles + wait states" are **not derivable from code**; the timing chapter must be checked by a human (see HO:425-439, 458-464 for what the manual claims to cover).

## Open questions

1. **Return address for "abort" faults.** EDEF:121-131 says abort exceptions (bus, alignment, privilege violation, invalid opcode) save the faulting instruction's address so it can be retried, and EDEF:19-20 says pl is not incremented for a bus fault. In the implementation pl is advanced unconditionally at decode (PU:151) even when the decode-time privilege check has just raised a fault (PU:141-145), data bus faults arrive after decode (LIB:312-324), and invalid-opcode faults save the next address (T-FAULT:939-942). Only alignment (PU:83-86, before decode) and PC-wrap segment overflow (PU:92-97) leave the faulting address. No test asserts the link register for PrivilegeViolation, Bus, BusProtection, or Alignment.
2. **NOOP / zeroed memory faults in protected mode.** `NOOP` and all-zero memory encode as `ADDI` with register field 0 = `sr` (A-META:82-95; ENC:741-755). `sr` is a privileged destination (PU:21-27), so in protected mode NOOP raises PrivilegeViolation (PU:42-51). No test covers NOOP in protected mode.
3. **Vector count comments.** EDEF:6-9 says 128 vectors, 48 privileged, 80 user; EDEF:110 says 128 user vectors. The vector field is 8 bits and user vectors span 0x60-0xFF = 160 entries with 96 below (EDEF:112-113; EENC:3).
4. **Software exception below 0x60.** EDEF:55 lists it as a PrivilegeViolation; the implementation asserts (panics) at EU decode (EEX:330-333) and the privilege check only looks at the opcode nibble (PU:46-48), so EXCP is never privileged. No test.
5. **RTE at level 0** underflows the link-register index (EEX:350-356, TODO). Hardware behaviour undefined.
6. **Unimplemented EU opcodes 0x2-0x8** panic in `deconstruct_cause_value` (EEX:69-77, TODO). Hardware behaviour undefined.
7. **Fault raised while one is pending** panics (LIB:152-157, TODO).
8. **Negative displacements with SR.A set.** The overflow test is an unsigned 16-bit add of the raw displacement (EX:77-83), so `LOAD r1, (#-2, a)` with `al >= 2` wraps and raises SegmentOverflow whenever A is set. Tests only use positive displacements (T-FAULT:640-693). The manual must state whether negative displacements are usable with the trap enabled.
9. **LOAD flag semantics.** Hardware LOAD op with AF=Alu sets N/Z and clears C/V (ALU:307-317; T-AR:497-534); the assembler always emits AF=None (A-LOAD:47, 128); the immediate LOAD test uses AF=Shift and expects all flags cleared (T-AI:29-33, 521-573). Three observable behaviours; the manual must say which is architectural.
10. **Opcodes without assembler syntax.** 0x27 LoadRegisterFromShortImmediate (A-AI:31-47; A-LOAD:73) and 0x2F CoprocessorCallShortImmediate (A-COP:49-140), which can only deliver an 8-bit command with COP ID 0 (T-PROT:530-547). Whether these are "undocumented" like 0x08/0x09/0x0B/0x0D or merely unassembled is a manual decision.
11. **Addressing-mode count.** HO:17 fixes seven modes; the parser has ten operand forms (A-INS:85-106). The seven-way grouping in this digest is inferred, not cited.
12. **Reset state of general registers.** `reset()` leaves r1-r7, a/l/s pairs, and all link registers unchanged (LIB:524-536); only SR = 0 and PC are set by the Reset opcode (EEX:367-370). HO:369-371 claims the manual's reset table covers general registers; it must say "unchanged/undefined".
13. **Register-operand shift counts > 15** clamp to 16 for logical/arithmetic shifts but rotate mod 16 (ALU:372, 479, 495); untested (T-AR:955; T-ASI:912). Also `MAX_SHIFT_COUNT` (DEF:43) is not enforced on register counts.
14. **Hardware-exception opcode (0xF) from software.** A privileged `COPI #0x1Fxx` would run `handle_exception` as a hardware exception (EEX:416-427); T-FAULT:1321-1323 marks this unresolved.
15. **Aliased auto-update forms.** The assembler rejects them (A-MOD:36-48; TA-CF:294-310) while hardware performs the writes in order source-update then destination (WB:216-236, 258-284); the manual should label the result undefined rather than describe it.
16. **Shift applies to the register, not the immediate.** `ADDI r1, #2, LSL #3` computes `(r1 << 3) + 2` (FD:189-192; T-ASI:507-596), and `ADDR r1, r2, r3, LSL #2` computes `(r2 << 2) + r3` (FD:180-186). GEN-S:8 prints the example without stating this; proofreaders should verify the manual's operand-order wording.
