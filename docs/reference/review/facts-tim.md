# G9 (timing) fact-check findings — reviewer tag `tim`

Chapter file reviewed: `docs/reference/chapters/appendix-b-timing.tex` (213 lines).
Digest: `docs/reference/review/facts.md`. Code paths below are relative to the repository root.

### F-tim-1
- file: chapters/appendix-b-timing.tex
- lines: 99-102
- severity: blocker
- category: fact
- claim: "ADDR|HI r3, r4, r5            ; 6 cycles (exec if r1 > r2)" / "ADDR|LO r3, r6, r7            ; 6 cycles (exec if r1 <= r2)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:195-206 (`a.overflowing_sub(b)` — C is set on *borrow*, i.e. when `a < b`); sirc-vm/peripheral-cpu/tests/instructions/arithmetic_immediate_test.rs:260-273; sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:129-135 (`UnsignedHigher` = C && !Z, `UnsignedLowerOrSame` = !C || Z); docs/reference/manual-handover.md:19 ("subtraction sets C when a borrow occurs"); facts.md:171, 248
- resolution: manual-wrong
- fix: After `CMPR r1, r2` the carry is a borrow, so `HI` (C && !Z) is true exactly when r1 < r2 and `LO` (!C || Z) is true exactly when r1 >= r2 — the comments are inverted. Replace with "ADDR|HI r3, r4, r5            ; 6 cycles (exec if r1 < r2)" and "ADDR|LO r3, r6, r7            ; 6 cycles (exec if r1 >= r2)". The same inversion drives line 176 (`SUBI r7, #4` / `BRAN|HI @loop` loops only while r7 < 4, so the unrolled-loop example never iterates); change that branch to `BRAN|!= @loop`. Note that `10-condition-codes.tex:74,76,82,138,181-184,249,263` states the ARM reading ("HI tests if first > second") throughout, so the fix must be coordinated with that chapter's reviewer; if the architects instead intend the ARM polarity, the defect is in the ALU carry output and this becomes `code-wrong`.
- confidence: high

### F-tim-2
- file: chapters/appendix-b-timing.tex
- lines: 74
- severity: blocker
- category: fact
- claim: "ADDI r7, #0, #10              ; 6 cycles"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_immediate.rs:274-279 (only `[DirectRegister, Immediate]`, `[DirectRegister, Immediate, ShiftDefinition]` accepted); assembling this line with `sirc-vm/target/debug/assembler` fails: "The [ADDI] opcode only supports immediate->register addressing mode (e.g. ADDI y1, #1) at line 1, column 6"
- resolution: manual-wrong
- fix: The third operand of the short-immediate form must be a shift definition, not a second immediate. Replace the line with "LOAD r7, #10                  ; 6 cycles" (opcode 0x07, `A-LOAD:41-49`), which is what the surrounding comment ("Process 10 elements") intends.
- confidence: high

### F-tim-3
- file: chapters/appendix-b-timing.tex
- lines: 146-153
- severity: blocker
- category: fact
- claim: "; Instead of:\nADDR r1, r2, r2\nADDI r1, #0, LSL #1\n\n; Use:\nADDR r1, r2, r2, LSL #1    ; Multiply by 3 in one instruction"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:180-192 (register form computes `shift(S1) op S2`; short-immediate form computes `shift(D) op imm8`); facts.md:153, 348
- resolution: manual-wrong
- fix: The two sequences are not equivalent. The "Instead of" block computes `r1 = (r2 + r2) << 1 = 4 * r2`; the "Use" line computes `(r2 << 1) + r2 = 3 * r2`. Replace the "Instead of" block with the two-instruction sequence that really does yield 3*r2:
  "ADDR r1, r2, r2            ; r1 = 2 * r2" / "ADDR r1, r1, r2            ; r1 = 3 * r2".
- confidence: high

### F-tim-4
- file: chapters/appendix-b-timing.tex
- lines: 159
- severity: blocker
- category: fact
- claim: "ADDI a, #2"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/arithmetic_immediate.rs:274-279 with sirc-vm/toolchain/src/parsers/instruction.rs:85-106 (an address-register *pair* name `a` parses as `DirectAddressRegister`, which no `ADDI` arm accepts); assembling the line fails with the same "only supports immediate->register addressing mode" error; facts.md:211-212
- resolution: manual-wrong
- fix: ALU instructions take a 4-bit register field, so the operand must be a half of the pair: write "ADDI al, #1" (see also F-tim-5 for the count).
- confidence: high

### F-tim-5
- file: chapters/appendix-b-timing.tex
- lines: 156-163
- severity: blocker
- category: fact
- claim: "LOAD r1, (#0, a)+          ; Same result, fewer cycles"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:81, 100-104 (`addr_inc` is exactly 1); sirc-vm/peripheral-cpu/tests/instructions/load_test.rs:250-294; memory is word-addressed (sirc-vm/peripheral-cpu/tests/instructions/store_test.rs:105-107); facts.md:155, 326
- resolution: manual-wrong
- fix: Post-increment advances the address register's low word by one *word*, not by two, so `ADDI a, #2` is not the same result. Change the "Instead of" block to "LOAD r1, (#0, a)" / "ADDI al, #1". The same word-vs-byte error appears at line 78, where `LOAD r1, (#0, a)+` followed by `STOR (#-2, a), r1` writes two words below the word just read; that store must be `STOR (#-1, a), r1`.
- confidence: high

### F-tim-6
- file: chapters/appendix-b-timing.tex
- lines: 75
- severity: major
- category: fact
- claim: "loop:"
- evidence: sirc-vm/toolchain/src/parsers/shared.rs:112-114 (`parse_label_` = `preceded(char(':'), cut(parse_label_name_))`); assembling a file containing `loop:` fails at line 1 column 1 with "expected ';' ... in section \"instruction\""; the `:label` form assembles
- resolution: manual-wrong
- fix: Label definitions take a leading colon in this assembler. Replace `loop:` with `:loop` here and at line 169, and `skip:` with `:skip` at line 138. (The trailing-colon form also appears in `08-addressing-modes.tex:376`, `10-condition-codes.tex:246`, `15-control-flow.tex:177,276,529,532,538,544,554`, and `16-coprocessor-instructions.tex:326`; a manual-wide sweep is needed, but only the three occurrences in this appendix are mine to report.)
- confidence: high

### F-tim-7
- file: chapters/appendix-b-timing.tex
- lines: 133-143
- severity: major
- category: fact
- claim: "CMPR r1, r2\nBRAN|<= @skip\nADDI r3, #1\nskip:\n\n; Use:\nCMPR r1, r2\nADDI|HI r3, #1             ; Saves branch overhead"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:129-135, 144-152 (`LessThanOrEqual` = Z || (N != V) — signed; `UnsignedHigher` = C && !Z — unsigned); facts.md:240-245
- resolution: manual-wrong
- fix: The rewrite is not equivalent even before F-tim-1 is applied: `<=` is the signed condition, so its complement is `>>` (GreaterThan), not the unsigned `HI`. Use one domain consistently — with the borrow convention (F-tim-1) the unsigned pair is `BRAN|HI @skip` / `ADDI|LO r3, #1`, or keep the signed test as `BRAN|<= @skip` / `ADDI|>> r3, #1`.
- confidence: high

### F-tim-8
- file: chapters/appendix-b-timing.tex
- lines: 6-9, 28-48
- severity: major
- category: fact
- claim: "DMA meta-instructions are the main exception: after the normal coprocessor dispatch they hold instruction fetch until the DMA unit has completed its bus transfers."
- evidence: facts.md:317 ("Every COP instruction costs two 6-cycle slots: the PU instruction, then the coprocessor dispatch at the following phase 0"), citing sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:781-791 and sirc-vm/peripheral-cpu/tests/exceptions/software_exceptions.rs:41-51; dispatch slot structure at sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:302-446 and tests/exceptions/common.rs:125-167
- resolution: manual-wrong
- fix: DMA is not the only exception to "6 cycles". Every coprocessor call that actually dispatches — `COPI`, `COPR`, and the exception-unit meta-instructions `EXCP`, `WAIT`, `RETE`, `RSET`, `ETFR`, `ETTR` — costs the 6-cycle processing-unit instruction plus a further 6-cycle coprocessor dispatch slot beginning at the next phase 0, i.e. 12 cycles before the next instruction fetch (and unbounded for `WAIT`). Add rows to Table "Instruction Timing" for `COPI`/`COPR` (6 + 6), the exception-unit meta-instructions (6 + 6), and `WAIT` (6 + 6 + idle until an enabled interrupt or reset), and reword the Overview sentence to "Coprocessor calls are the exception: the dispatched coprocessor occupies at least one further slot after the six-phase call." Chapter 16's per-entry Timing fields (`16-coprocessor-instructions.tex:213,241,264,287,320`) say only "6 cycles ... plus exception-unit follow-up work" and should be given the same number.
- confidence: high

### F-tim-9
- file: chapters/appendix-b-timing.tex
- lines: 26-54
- severity: major
- category: fact
- claim: "\section{Timing Characteristics}" (table lists only ALU/load/store/branch/conditional/NOOP/DMA rows)
- evidence: facts.md:286, 327 (EU dispatch = 6 cycles: vector high word at phase 0, low word at phase 1, entry at phase 3; "a software exception therefore starts its handler 12 cycles after the EXCP fetch began; a data bus fault detected at phase 4/5 starts its handler 6 cycles later"), citing sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:303-334, tests/exceptions/common.rs:133-153, tests/exceptions/faults.rs:124-184; reset hold at sirc-vm/peripheral-bus/src/reset_unit.rs:22-26 and tests/exceptions/reset.rs:84-147
- resolution: n/a
- fix: The appendix gives no exception-entry or interrupt-latency figures, and no other chapter quantifies them (`06-exceptions.tex` mentions cycles only for the reset hold). Add a subsection "Exception Timing" stating: a hardware-exception or fault dispatch occupies one 6-cycle slot (vector high word at phase 0, low word at phase 1, entry committed at phase 3), a latched interrupt is dispatched at the next phase 0 so worst-case latency from assertion to the handler's first instruction fetch is 6 (finish current instruction) + 6 (dispatch) cycles, and reset holds RSTO for 6 cycles before the reset-vector fetch.
- confidence: high

### F-tim-10
- file: chapters/appendix-b-timing.tex
- lines: 58-64
- severity: major
- category: contradiction
- claim: "All execution phases run normally" / "The write-back stage is disabled (no register or memory modification)"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:33-35 (false condition -> `ExecutionStepInstructionType::NoOp`, no effective address and no `SR.A` overflow check); stages/memory_access.rs:26-28 (false condition -> `MemoryAccessInstructionType::NoOp`, so no `BAS`/`DataRead`/`DataWrite` cycle is driven at phase 4); stages/write_back.rs:128-130; facts.md:182
- resolution: manual-wrong
- fix: The phases still occur, but three of them are suppressed, and the suppression of the memory-access phase is externally visible (a not-executed `LOAD`/`STOR` drives no bus cycle and therefore cannot incur a data wait state). Replace the bullets with: "All six phases still occur and consume one cycle each"; "The execute, memory-access, and write-back phases perform no operation: no effective address is computed, no `SR.A` overflow check is made, and no data bus cycle is driven"; "No register, memory, or address-register update occurs"; "Status flags are not updated".
- confidence: high

### F-tim-11
- file: chapters/appendix-b-timing.tex
- lines: 26-54
- severity: major
- category: fact
- claim: "The cycle counts in this appendix assume that every bus operation is acknowledged without a wait state."
- evidence: facts.md Open question 1 (EDEF:121-131 says abort faults save the faulting instruction's address for retry, but pl is advanced unconditionally at decode, PU:151, even when the decode-time privilege check has just faulted at PU:141-145; data bus faults arrive after decode, LIB:312-324; only alignment, PU:83-86, and PC-wrap segment overflow, PU:92-97, act before decode); no test asserts the link register for PrivilegeViolation, Bus, BusProtection, or Alignment
- resolution: unclear
- fix: The appendix never says what a faulting instruction costs — whether it runs all six phases and then dispatches, or is aborted at the phase that detects the fault. The digest's open question 1 shows the code and the definitions comment disagree on the closely related question of which address is saved. State explicitly, once the architects rule: at which phase each fault is detected (alignment before phase 0's fetch; privilege violation at phase 2; segment overflow at phase 2/3; bus faults at the acknowledging phase), whether the remaining phases still run, and how many cycles elapse before the exception-unit dispatch slot begins.
- confidence: medium

### F-tim-12
- file: chapters/appendix-b-timing.tex
- lines: 121-127
- severity: minor
- category: fact
- claim: "Each wait state adds one clock cycle to the instruction"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:89-117 (two instruction-fetch bus cycles per instruction, phases 0 and 1); stages/memory_access.rs:56-74 (one further bus cycle only for opcodes 0x10--0x17); facts.md:322-324
- resolution: manual-wrong
- fix: True per wait state but not enough to compute an instruction's cost. Add the bus-access count per class: every instruction performs two instruction-fetch accesses; loads and stores add one data access; a coprocessor dispatch adds two vector-fetch accesses. So a system inserting *w* wait states per access adds 2w cycles to an ALU instruction and 3w to a load or store.
- confidence: high

### F-tim-13
- file: chapters/appendix-b-timing.tex
- lines: 204-205
- severity: minor
- category: fact
- claim: "ARM6           & 1.0              & Single-cycle with pipeline" / "MIPS R2000     & 1.0              & Single-cycle with pipeline"
- evidence: none (external historical claim; not checkable against this repository)
- resolution: unclear
- fix: An average CPI of exactly 1.0 is not attainable on either part: ARM6 loads take 3 cycles and taken branches 3 cycles, and R2000 loads/branches cost extra cycles on a cache or interlock stall. Either give a range in the same style as the 6502 and 68000 rows (for example "1--3" and "1--2" with the note "1 cycle for register ALU operations; loads and taken branches cost more"), or mark the column "Best-case CPI" so the SIRC-1 row (a true fixed 6.0) is not compared against best cases.
- confidence: medium

### F-tim-14
- file: chapters/appendix-b-timing.tex
- lines: 43-44, 202-203
- severity: nit
- category: style
- claim: "DMAR/DMAW                  & $6 + \\max(|n|, 1)$" / "MOS 6502       & 2-7" / "Motorola 68000 & 4-158"
- evidence: docs/reference/STYLE.md:51 (numeric ranges take an en dash `--`), STYLE.md:52 (mnemonics always inside `\mnemonic{}`)
- resolution: n/a
- fix: Write `2--7` and `4--158`, and set the mnemonics in the timing table as `\mnemonic{DMAR}/\mnemonic{DMAW}` and `\mnemonic{DMAT}` (the prose at lines 51--52 already uses the macro). No semantic effect.
- confidence: high

## Checks performed that found no defect

- Six-phase list (lines 17-24): matches `CSH:6-14` and `LIB:66-67` (`CYCLES_PER_INSTRUCTION = 6`) and Chapter 2's list. The appendix does **not** repeat Chapter 2's claim that the PC increments by 1 after each instruction-word fetch — it makes no PC statement at all, so the sibling reviewer's finding does not extend here. If Chapter 2's phase list is corrected, no corresponding change is needed in this appendix.
- Fetch order "High Word" then "Low Word": correct in behaviour — phase 0 reads the word at PC into instruction bits 31:16 and phase 1 reads PC+1 into bits 15:0 (`PU:89-108`), even though the code's enum names those phases `InstructionFetchLow` and `InstructionFetchHigh`.
- DMA rows `6 + max(|n|, 1)` and `6 + max(2n, 1)`, the "at least one post-dispatch cycle" prose, and the false-condition rule are exactly the ruled decision at `docs/reference/manual-handover.md:555-559` and match `16-coprocessor-instructions.tex:429,477,523`. `DMAR`/`DMAW` counts are 0--7 and `DMAT` counts 0--255 (facts.md:313), so both formulas are in range.
- Arithmetic in the loop example: 5 instructions x 6 = 30 cycles per iteration, 6 + (10 x 30) = 306. Correct.
- `LJSR a` and `RETS` are accepted by the assembler (`A-LJSR:83-92`; `A-META`), and the 12-cycle call overhead is right.
- `HI`/`LO` are exact complements (`C && !Z` vs `!C || Z`), so "one conditional executes, one doesn't" holds — only the direction is wrong (F-tim-1).
- `f_clock / 6`: 10 MHz -> 1.67 MIPS, 25 MHz -> 4.17 MIPS. Both correct to the stated precision.
- Branch taken and not taken both 6 cycles: branches are `LDEA`/`LDEL` with `p` as destination, no extra phases (facts.md:157-158).
- "one execution phase per 16-bit word transferred" and the BACK/BERR/BPER hold rule match `LIB:106-110, 259-276` and facts.md:324.
- 6502 (2--7) and 68000 (4--158) CPI ranges are consistent with the published figures for those parts.
