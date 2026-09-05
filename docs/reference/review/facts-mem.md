# Fact-check findings: chapter group G6 (memory and control flow)

Reviewer tag: `mem`. Chapters reviewed: `docs/reference/chapters/14-memory-instructions.tex`,
`docs/reference/chapters/15-control-flow.tex`. Digest: `docs/reference/review/facts.md`.

### F-mem-1
- file: chapters/14-memory-instructions.tex
- lines: 95
- severity: blocker
- category: fact
- claim: "Documented memory forms do not raise privilege-violation or invalid-opcode faults."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:21-27 and :42-43 (`PRIVILEGED_REGISTERS.contains(&instruction.des)` for every non-COP opcode); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:158 (`des` is the register field of *all three* formats, so it is the LOAD destination and the STOR source register)
- resolution: manual-wrong
- fix: Replace with: "A memory instruction raises a privilege-violation fault in protected mode when its register field names a privileged register (\reg{sr}, \reg{ah}, \reg{lh}, \reg{ph}, or \reg{sh}). This applies to the \mnemonic{LOAD} destination register and to the \mnemonic{STOR} source register, even though \mnemonic{STOR} only reads it. Documented memory forms do not raise invalid-opcode faults." The `Privilege:` fields of the LOAD entry (line 180) and the STOR entry (line 260) need the same qualification.
- confidence: high

### F-mem-2
- file: chapters/14-memory-instructions.tex
- lines: 104-105
- severity: major
- category: contradiction
- claim: "Memory instructions do not support status update override syntax; the status override field is ignored."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:198-199 and :217 (the AF field is decoded simultaneously as `sr_src` and as the address-register pair index); sirc-vm/toolchain/src/parsers/opcodes/store.rs:48 and load.rs:58 (the assembler puts the address register in `additional_flags`)
- resolution: manual-wrong
- fix: Replace the second clause with: "the AF field (bits 5--4) is not a status update source for these opcodes; it selects the address-register pair used to form the effective address." As written, an implementer or assembler author could conclude that AF is a don't-care for memory opcodes and encode zero, which silently retargets every memory access to \reg{l}.
- confidence: high

### F-mem-3
- file: chapters/14-memory-instructions.tex
- lines: 29-41
- severity: major
- category: fact
- claim: "\text{EA} = \text{AddressRegister} + \text{Offset}" ... "The resulting 24-bit address is used to access memory."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:77-78 (`decoded.ad_l_.overflowing_add(decoded.sr_b_)`, a 16-bit add); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/memory_access.rs:58 and :67 (`(decoded.ad_h_, alu_output).to_full_address()`, the high word is taken unmodified)
- resolution: manual-wrong
- fix: The displacement is added to the low word only, modulo 2^16; the high word of the address register is concatenated unchanged and carry never propagates into it. Replace the equation with two lines, e.g. `EA[23:16] = AddressRegister.high` and `EA[15:0] = AddressRegister.low + Offset (mod 2^16)`, and state that a memory access can never leave the 64K segment selected by the high word. The table row at line 80 says this correctly; the equation contradicts it.
- confidence: high

### F-mem-4
- file: chapters/14-memory-instructions.tex
- lines: 142-144
- severity: major
- category: fact
- claim: "\texttt{rD} is the destination general-purpose register. \texttt{addr} is the address-register pair that supplies the segment and base low word."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/load.rs:240-246, :297-303, :331-337 and store.rs:160-166, :217-223 (`reject_aliased_address_register_write`); sirc-vm/toolchain/tests/assembler/control_flow_test.rs:305-308 (`LOAD al, (#0, a)+`, `LOAD ah, (r3, a)+`, `STOR -(#0, a), al`, `STOR -(r3, a), ah` all fail to assemble)
- resolution: manual-wrong
- fix: Chapter 14 never states the aliased-write restriction that Chapter 15 states for control flow. Add a normative sentence to the Legal Forms section: "In post-increment \mnemonic{LOAD} forms the destination register must not be either half of the auto-updated address-register pair; in pre-decrement \mnemonic{STOR} forms the source register must not be either half of the auto-updated pair. The result of such an encoding is architecturally undefined and the assembler rejects it."
- confidence: high

### F-mem-5
- file: chapters/14-memory-instructions.tex
- lines: 94-95
- severity: blocker
- category: contradiction
- claim: "Effective-address calculation can raise a segment-overflow fault when \texttt{SR.A} (Trap on Address Overflow) is set and the 16-bit low-word address calculation wraps."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:81-83 and :110-115. For pre-decrement forms `addr_inc` is `-1`, which is added as `0xFFFF`: `displaced.overflowing_add(0xFFFF)` reports overflow for every `displaced != 0` and reports *no* overflow for `displaced == 0`. With SR.A set, `STOR -(#0, s), r1` with `sl = 0x1000` therefore faults although 0x1000-1 does not wrap, while `sl = 0x0000` (the genuine wrap) does not fault. No test covers a pre-decrement with SR.A set (only sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:640-693, a positive-displacement LOAD).
- resolution: unclear
- fix: Manual side: the stated rule ("raise only when the calculation wraps") is the sane architectural rule. Code side: the borrow out of the `-1` is being fed into the same overflow term as the carry out of the displacement add, inverting the test for every pre-decrement form. Either the implementation must stop treating the decrement borrow as an overflow, or the manual must say that with SR.A set every pre-decrement memory or effective-address instruction faults unless the computed low word is zero. Do not publish the current wording until this is decided. The same sentence appears at chapters/15-control-flow.tex:103-105 and covers \mnemonic{LDEA} pre-decrement (opcodes 0x1A/0x1B), which uses the same code path.
- confidence: medium

### F-mem-6
- file: chapters/14-memory-instructions.tex
- lines: 94-95
- severity: major
- category: fact
- claim: "Effective-address calculation can raise a segment-overflow fault when \texttt{SR.A} (Trap on Address Overflow) is set and the 16-bit low-word address calculation wraps."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/execution_effective_address.rs:77-78 and :110-115 (the overflow test is an unsigned 16-bit add of the raw displacement field); docs/reference/review/facts.md Open question 8
- resolution: unclear
- fix: The manual never says whether the 16-bit displacement is signed. Because the trap fires on unsigned carry, a "negative" displacement such as `LOAD r1, (#-2, a)` faults with SR.A set whenever `al >= 2` -- that is, negative displacements are unusable with the trap enabled. State this explicitly (either "displacements are unsigned and SR.A traps any carry out of bit 15" or "negative displacements must not be used while SR.A is set"), and give the same warning for backward \mnemonic{BRAN}/\mnemonic{BRSR} displacements in Chapter 15.
- confidence: medium

### F-mem-7
- file: chapters/14-memory-instructions.tex
- lines: 86
- severity: major
- category: fact
- claim: "If a fault aborts execution before write-back, those write-back effects do not occur."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:73-76 (the whole phase is skipped once a fault is pending) confirms the quoted sentence; but sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:684-693 shows the fault return address for a faulting LOAD is the *following* instruction (`0x00AB_CDE2` for an instruction at `0x00AB_CDE0`)
- resolution: n/a
- fix: The statement is correct but incomplete in the way an implementer needs. Add: "A memory instruction aborted by a fault is not restartable: the saved return address is the address of the next instruction, so returning from the handler resumes after the faulting access." (See docs/reference/review/facts.md Open question 1 for the general abort-fault return-address question, which the exception chapter must settle.)
- confidence: medium

### F-mem-8
- file: chapters/14-memory-instructions.tex
- lines: 103-114
- severity: major
- category: fact
- claim: "Memory instructions do not update status flags. All four condition flags are preserved regardless of addressing form, shift result, or auto-update side effect." (table row: "LOAD & - & - & - & -")
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:310-316 (memory loads never call `update_status_flags`) supports the claim for opcodes 0x14--0x17; but sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/alu.rs:307-317 and tests/instructions/arithmetic_register_test.rs:497-534 show the non-memory `LOAD` op with AF=Alu sets N and Z; docs/reference/review/facts.md Open question 9
- resolution: unclear
- fix: The table row is labelled only "LOAD", which a reader will apply to `LOAD rD, #imm16` (0x07) and `LOAD rD, rS` (0x37) as well. Relabel the rows "LOAD (memory forms 0x14--0x17)" and "STOR (0x10--0x13)", and add a cross-reference to whichever chapter rules on LOAD-immediate/LOAD-register flag behaviour. The three observable behaviours (hardware AF=Alu sets N/Z; assembler always emits AF=None; the immediate test uses AF=Shift and clears everything) must be settled once, manual-wide.
- confidence: medium

### F-mem-9
- file: chapters/14-memory-instructions.tex
- lines: 296
- severity: major
- category: fact
- claim: "loop:"
- evidence: sirc-vm/toolchain/src/parsers/shared.rs:112 (`parse_label_` is `preceded(char(':'), parse_label_name_)`, i.e. the colon is a *prefix*); examples/byte-sieve/byte-sieve.sasm:10 (`:main`)
- resolution: manual-wrong
- fix: Label definitions in SIRC assembly are written `:loop`, not `loop:`. Every trailing-colon label in these chapters is rejected by the assembler. Occurrences: 14-memory-instructions.tex:296; 15-control-flow.tex:177, 276, 529, 532, 538, 544, 554.
- confidence: high

### F-mem-10
- file: chapters/14-memory-instructions.tex
- lines: 15-21
- severity: minor
- category: fact
- claim: "Both instructions support four memory addressing forms:" followed by a list including "Post-increment load" and "Pre-decrement store"
- evidence: docs/reference/review/facts.md opcode map 0x10--0x17; chapters/14-memory-instructions.tex:68-69 states the correct rule
- resolution: manual-wrong
- fix: Rewrite as "Each instruction supports four addressing forms: indirect with immediate displacement, indirect with register displacement, and one auto-update form -- post-increment for \mnemonic{LOAD}, pre-decrement for \mnemonic{STOR} -- with immediate or register displacement." As written, "Both instructions support four" contradicts the list, in which two of the four items are single-instruction forms.
- confidence: high

### F-mem-11
- file: chapters/14-memory-instructions.tex
- lines: 53-59
- severity: minor
- category: terminology
- claim: "LOAD & \texttt{LOAD rD, (rO, addr)[, shift]} & 0x15 & Register offset; optional load-result shift"
- evidence: chapters/14-memory-instructions.tex:132 (`LOAD rD, (rS, addr)`), :213 (`STOR (rD, addr), rS`)
- resolution: manual-wrong
- fix: One placeholder name per operand role, chapter-wide. The Legal Forms table calls the displacement register `rO`, the LOAD syntax block calls it `rS`, and the STOR syntax block calls it `rD` while using `rS` for the store source -- so `rD` denotes a destination in one entry and a displacement in the next. Pick one (suggest `rO` for the displacement register everywhere) and apply it at lines 53, 55, 57, 59, 132, 135, 138-139, 213, 216, 219-220, 225.
- confidence: high

### F-mem-12
- file: chapters/15-control-flow.tex
- lines: 144-145
- severity: blocker
- category: fact
- claim: "\texttt{@label} is resolved by the linker to a PC-relative displacement from the next instruction address."
- evidence: sirc-vm/toolchain/src/bin/linker.rs:86-102 (`full_offset = target_offset_words - program_offset_words`, where `program_offset_words` is the address of the branch instruction itself); sirc-vm/peripheral-cpu/tests/instructions/ljmp_test.rs:224-232 (pl = 0xFAC0, displacement 0x000E lands at 0xFACE, not 0xFACC); execution.rs:151 (pl advances only after the decoded `npc_l_` is captured)
- resolution: manual-wrong
- fix: Replace with "resolved by the linker to a word displacement from the address of the branch instruction itself." The same wrong statement appears at 15-control-flow.tex:194-195 (BRSR). Also add to the BRAN/BRSR Description blocks: the displacement base is the branching instruction's own address, so `BRAN #0` branches to itself and `BRAN #2` falls through to the next instruction, while the \mnemonic{BRSR} link value is that address plus 2 (ljmp_test.rs:292-300).
- confidence: high

### F-mem-13
- file: chapters/15-control-flow.tex
- lines: 60-62
- severity: major
- category: fact
- claim: "\texttt{src} and \texttt{dest} are address-register pairs: \reg{l}, \reg{a}, \reg{s}, or \reg{p}."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/ldel.rs:51-57, :106-113, :135-142, :206-213 (every \mnemonic{LDEL} form rejects `l` as the destination, not just the post-increment forms); sirc-vm/toolchain/tests/assembler/control_flow_test.rs:299 (`LDEL l, (#0, a)` is an error)
- resolution: manual-wrong
- fix: Add: "\reg{l} is never a legal \mnemonic{LDEL} destination, because \mnemonic{LDEL} always writes the return address to \reg{l}; the destination and the link write would alias. \mnemonic{LDEL} destinations are \reg{a}, \reg{s}, or \reg{p}." The Legal Forms rows at lines 37-40 need the same restriction, since as written they admit `LDEL l, (#offset, src)`.
- confidence: high

### F-mem-14
- file: chapters/15-control-flow.tex
- lines: 71-73
- severity: major
- category: fact
- claim: "Instructions that write the same address-register pair through more than one write-back path have architecturally undefined behavior. Avoid forms such as \texttt{LDEA a, -(\#0, a)} and \texttt{LDEL l, (\#0, l)+}. The assembler rejects known hazardous aliased forms."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/ldea.rs:135-141, :191-197; ldel.rs:51-57, :135-157, :206-228; ljsr.rs:154-167, :214-227; load.rs:240-246; store.rs:160-166; tests/assembler/control_flow_test.rs:295-310; docs/reference/review/facts.md Open question 15
- resolution: unclear
- fix: "Known hazardous" is not implementable: an assembler author cannot derive the rejection list, and a CPU implementer cannot tell which encodings must be diagnosed. Replace with the exhaustive list the toolchain enforces: \mnemonic{LDEA} pre-decrement with destination equal to source; \mnemonic{LDEL} with destination \reg{l} (any form); \mnemonic{LDEL} post-increment with source \reg{l} or with destination equal to source; \mnemonic{LJSR} post-increment from \reg{l} or \reg{p}; \mnemonic{LOAD} post-increment whose destination is a half of the auto-updated pair; \mnemonic{STOR} pre-decrement whose source is a half of the auto-updated pair. Keep the "architecturally undefined" label rather than describing the order the current implementation happens to use (write_back.rs:216-236 applies the source update first and the destination write last).
- confidence: high

### F-mem-15
- file: chapters/15-control-flow.tex
- lines: 506
- severity: major
- category: fact
- claim: "LJSR a, #function_offset      ; p = a + function_offset"
- evidence: sirc-vm/toolchain/src/parsers/shared.rs:99 (`parse_number_` = `#` followed by a hex, binary, or decimal literal only), :138 (symbol references are `@name`), :103 (placeholders are `$name`)
- resolution: manual-wrong
- fix: `#identifier` is not assembler syntax; the listing does not assemble. Use `LJSR a, @function_offset` (a symbol reference, resolved as the symbol's lower word by ljsr.rs:108-119) or define the constant with `.EQU $function_offset #N` and write `LJSR a, $function_offset`.
- confidence: high

### F-mem-16
- file: chapters/15-control-flow.tex
- lines: 550-552
- severity: major
- category: fact
- claim: "ADDI r2, #0, LSL #1           ; r2 = (r2 << 1) = r2 * 2 (word size)" / "LDEA p, (r2, p)               ; Jump via table"
- evidence: sirc-vm/peripheral-cpu/tests/instructions/ljmp_test.rs:224-232 (the PC-relative base is the \mnemonic{LDEA}'s own address, not the following word)
- resolution: manual-wrong
- fix: The example does not work. `LDEA p, (r2, p)` computes `p = address_of_this_LDEA + r2`, and `jump_table` begins two words later, so index 0 jumps back to the \mnemonic{LDEA} itself (an infinite loop) and every other index lands two words short. The scaling is also wrong for the table content: the entries are one-word data words, so an index must be scaled by 1 to address them and by 2 only if the table holds branch instructions. Rewrite the example so the table holds two-word branch instructions (`ADDI r2, #2, LSL #1` to skip the \mnemonic{LDEA}, table of `BRAN @caseN`), or load the displacement first with `LOAD r3, (r2, p)` and then branch.
- confidence: high

### F-mem-17
- file: chapters/15-control-flow.tex
- lines: 554-558
- severity: major
- category: fact
- claim: "    DW case0 - jump_table"
- evidence: sirc-vm/toolchain/src/types/data.rs:5-7 (`DB_TOKEN = ".DB"`, `DW_TOKEN = ".DW"`, `DQ_TOKEN = ".DQ"`); sirc-vm/toolchain/src/parsers/data.rs:36-47, :49-51 (a data directive operand is a `#`-prefixed number or an `@` symbol reference -- there is no expression grammar)
- resolution: manual-wrong
- fix: The directive is `.DW` (leading dot) and `case0 - jump_table` is not parseable: the assembler has no arithmetic on symbols. Use `.DW @case0` (which the linker resolves) or hand-written literals, and note that a `.DW` symbol reference resolves as a full address, not a difference.
- confidence: high

### F-mem-18
- file: chapters/15-control-flow.tex
- lines: 94, 330-331
- severity: major
- category: fact
- claim: "In protected mode, address-register-pair write-back is allowed only when every written high word would remain unchanged. If any high word would change, the instruction raises a privilege-violation fault instead of completing write-back."
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:21-27 and :42-43 (the decode-time check tests `instruction.des` against register indexes 0, 8, 10, 12, 14); fetch_and_decode.rs:158 and :200-201 (for \mnemonic{LDEA}/\mnemonic{LDEL} the register field is the 2-bit destination *pair* index, 0 = \reg{l}, so `des == 0` collides with the \reg{sr} index used by the privilege check). No test covers `LDEA l, (...)` in protected mode (tests/instructions/protected_mode_test.rs:152-438 uses destinations \reg{a} and \reg{p} only).
- resolution: unclear
- fix: As implemented, any \mnemonic{LDEA} whose destination pair is \reg{l} (register field 0) raises a privilege-violation fault in protected mode before write-back, regardless of the high words -- an extra rule the chapter does not state. Either the decode-time check must exclude the effective-address opcode class (0x18--0x1F), in which case the chapter is right and the code is wrong, or the chapter must add "\mnemonic{LDEA} with destination \reg{l} is privileged." Present both sides to the author; do not publish either wording until it is settled.
- confidence: medium

### F-mem-19
- file: chapters/15-control-flow.tex
- lines: 144
- severity: minor
- category: fact
- claim: "\texttt{\#disp} is a 16-bit PC-relative word displacement."
- evidence: sirc-vm/toolchain/src/parsers/opcodes/branching.rs:41-52 (`BRAN|!= #-4` encodes 0xFFFC); sirc-vm/toolchain/src/bin/linker.rs:105-114 (the linker range-checks the label displacement as `i16`); sirc-vm/peripheral-cpu/tests/instructions/ljmp_test.rs:271-286 (the add wraps within the segment)
- resolution: manual-wrong
- fix: State the signedness and the wrap: "\texttt{\#disp} is a signed 16-bit word displacement (--32768 to +32767) added to the low word of \reg{p} modulo 2^16. A branch cannot change the program segment; the displacement wraps within the current 64K segment." The same sentence is repeated for \mnemonic{BRSR} at line 194.
- confidence: high

### F-mem-20
- file: chapters/15-control-flow.tex
- lines: 501-503
- severity: minor
- category: contradiction
- claim: "LOAD ah, #0x0040" / "LOAD al, #0x0000              ; a = 0x00400000"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:21-27 (`ah` is a privileged destination); chapters/15-control-flow.tex:94 ("Direct writes to high address registers also raise a privilege-violation fault")
- resolution: manual-wrong
- fix: The example contradicts the chapter's own privilege rule when read in protected mode. Add a comment to the listing, for example `; supervisor mode only -- writing ah is privileged`, or build the segment through a supervisor-provided pointer.
- confidence: high
