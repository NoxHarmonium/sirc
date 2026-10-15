# Facts review: chapter group G7 (coprocessor and meta-instructions)

Reviewer tag: `cop`. Chapters reviewed: `chapters/16-coprocessor-instructions.tex`,
`chapters/17-meta-instructions.tex`.

Note on method: no `docs/reference/review/facts.md` digest existed for this pass, so every
normative claim below was checked directly against `sirc-vm/` and, where relevant, against the
extracted listings under `examples/manual-examples/extracted/` (all 69 listings re-assembled
cleanly during this review, so no finding below is a syntax finding).

### F-cop-1
- file: chapters/16-coprocessor-instructions.tex
- lines: 780-791
- severity: blocker
- category: contradiction
- claim: ":dma_copy_safe\n    STOR -(#0, s), ah          ; save the original source/dest addresses"
  ... "    DMAT a, l, #7              ; may fault partway through\n    ADDI sl, #5                ; success: discard the saved state\n    RETS"
- evidence: chapters/17-meta-instructions.tex:26 (`RETS` lowers to `LDEA p, (#0, l)`);
  sirc-vm/toolchain/src/parsers/opcodes/meta.rs:66-79 (`RETS` -> `LoadEffectiveAddressFromIndirectImmediate`,
  `additional_flags: AddressRegisterName::LinkRegister`);
  sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:247-268
  (`AddressWriteSubroutine` writes `Lh`/`Ll` with the next PC, i.e. `BRSR`/`LJSR` put the return
  address in `l`); chapters/16-coprocessor-instructions.tex:1067 ("A successful non-zero
  \mnemonic{DMAT} advances \reg{a} and \reg{l}")
- resolution: manual-wrong
- fix: The routine cannot work as written. `l` is the link register: if `dma_copy_safe` is
  reached with `BRSR`/`LJSR`, `l` holds the return address at entry (so `DMAT a, l, #7` copies
  into the caller's code, not into a destination buffer); if the caller instead loads `l` with a
  destination address, the `BRSR` that calls the routine overwrites it. Either way, after the
  `DMAT` `l` is the destination address advanced by 7, so the closing `RETS` jumps into the
  destination buffer rather than returning. Rework the example so the return address is saved
  and restored around the `DMAT` (for example, push `lh`/`ll` at entry *as the return address*,
  load `l` with the destination inside the routine, and restore `l` from the stack immediately
  before `RETS`), or restructure it as an inline code fragment with no `RETS` at all. The same
  defect applies to the trailing `RETS` at line 830 (`:dma_copy_failed`).
- confidence: high

### F-cop-2
- file: chapters/16-coprocessor-instructions.tex
- lines: 713-718
- severity: major
- category: contradiction
- claim: "Bus Fault, Bus Protection Fault, and Segment Overflow Fault are all classified \textit{Retryable}
  (Chapter~\ref{ch:exceptions}), and for an ordinary single-access instruction that classification means \mnemonic{RETE}
  safely re-executes the whole instruction from a clean, unstarted state."
- evidence: chapters/06-exceptions.tex:634 and :644 (Exception Quick Reference lists "Next
  instruction" as the Return Address for Bus fault and Bus protection) versus
  chapters/06-exceptions.tex:88-95, :106-112 and :273-275 (prose: "the link register stores the
  address of the \textit{faulting} instruction"), and
  sirc-vm/peripheral-cpu/src/lib.rs:341-352 (bus/bus-protection faults raised outside the two
  instruction-fetch phases recover `decoded_instruction.pc_h_/pc_l_`, i.e. the faulting
  instruction's own address)
- resolution: manual-wrong
- fix: The new retry section and the whole retry example depend on `RETE` returning to the
  faulting `DMAT`. Chapter 6's prose and the implementation agree with Chapter 16; the two
  Quick-Reference rows are the outlier. Change `chapters/06-exceptions.tex:634` and `:644`
  Return Address from "Next instruction" to "Faulting instruction". No change needed in
  Chapter 16. (Flagged here because Chapter 16's new text is unusable if the table is right.)
- confidence: high

### F-cop-3
- file: chapters/16-coprocessor-instructions.tex
- lines: 715-716
- severity: minor
- category: fact
- claim: "\mnemonic{DMAR}, \mnemonic{DMAW}, and \mnemonic{DMAT} are the only instructions in the SIRCIS instruction set whose execution spans more than one bus access"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:100-125
  (every instruction performs two `InstructionFetch` bus accesses, `InstructionFetchLow` and
  `InstructionFetchHigh`); `LOAD`/`STOR` add a third
- resolution: manual-wrong
- fix: Replace "more than one bus access" with "more than one data-memory bus access". Chapter 6
  states the same idea correctly at 06-exceptions.tex:100-101 ("one instruction that performs
  multiple bus accesses in sequence" is in the context of a DMA burst).
- confidence: high

### F-cop-4
- file: chapters/16-coprocessor-instructions.tex
- lines: 728-730
- severity: minor
- category: fact
- claim: "the same eager-decrement pattern used by pre-decrement addressing elsewhere in this architecture (Chapter~\ref{ch:addressing-modes})"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:140-147
  and :295-297 (`StoreRegisterToIndirectImmediatePreDecrement` = `AddressWriteStorePreDecrement`,
  applied in the write-back stage) together with
  sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:73-76 (write-back is
  skipped entirely when a fault is pending)
- resolution: manual-wrong
- fix: The analogy is inexact in exactly the dimension the paragraph is about: a pre-decrement
  `STOR` computes the decremented address before the access but only *commits* it at write-back,
  so a fault leaves its address register untouched -- the opposite of the backward-DMA behaviour
  being described. Either drop the parenthetical comparison or qualify it, e.g. "unlike
  pre-decrement addressing, whose address-register update is only committed at write-back and so
  is undone by a fault, the DMA decrement is committed before the transfer loop begins".
- confidence: high

### F-cop-5
- file: chapters/16-coprocessor-instructions.tex
- lines: 772-778
- severity: minor
- category: fact
- claim: "DMAT clobbers r1-r7 and can leave a/l pointing at the wrong\n; address if it faults mid-transfer (see above)"
- evidence: chapters/16-coprocessor-instructions.tex:1054-1062 (Operation: `a.low`/`l.low` are
  advanced only after a full chunk has been read *and* written) and :1077-1079 ("a fault leaves
  both address registers at the last fully completed chunk boundary")
- resolution: manual-wrong
- fix: For `DMAT a, l, #7` the whole transfer is a single 7-word chunk, so a mid-transfer fault
  can never advance `a` or `l` -- the exact hazard the example claims to guard against cannot
  occur in the example's own code. Either raise the example's count above 7 (e.g. `#16`, matching
  the DMAT usage example at line 1128) so a partial chunk boundary is reachable, or reword the
  header comment to say the save/restore is a general pattern that matters for counts above
  seven. The same mismatch appears in the handler comment at lines 811-813 ("may already be
  wrong, e.g. double-decremented by a backward transfer"): `DMAT` has no direction bit and no
  backward form at all (line 1041-1042), so that rationale does not apply to this listing.
- confidence: high

### F-cop-6
- file: chapters/16-coprocessor-instructions.tex
- lines: 773, 785, 806-809
- severity: minor
- category: fact
- claim: "; transfer up to 3 times if a recoverable DMA fault occurs partway" ... "    LOAD r1, #3                ; retry budget"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:140-147
  (`LoadRegisterFromIndirectImmediate` = `MemoryLoad` and `StoreRegisterToIndirectImmediate` =
  `NoOp` for status purposes, so only `SUBI` sets Z here);
  sirc-vm/toolchain/src/parsers/opcodes/arithmetic_immediate.rs:98 (plain `SUBI` defaults to
  `StatusRegisterUpdateSource::Alu`)
- resolution: manual-wrong
- fix: The handler decrements first and gives up when the result is zero, so a budget of 3 yields
  two retries (3 total attempts), not three retries. Either change the comment at line 773 to
  "retrying the whole transfer up to 2 more times" / "up to 3 attempts in total", or change
  `LOAD r1, #3` to `LOAD r1, #4`.
- confidence: medium

### F-cop-7
- file: chapters/16-coprocessor-instructions.tex
- lines: 36-38
- severity: minor
- category: contradiction
- claim: "Opcode 0x2F is Undocumented, in the same class as opcodes \opcode{08}, \opcode{09}, \opcode{0B}, and \opcode{0D} (see Appendix~\ref{appendix:undocumented}); it has no public assembly syntax."
- evidence: chapters/appendix-c-undocumented.tex:6-12 ("Two of the fourteen, \opcode{27} and
  \opcode{2F}, are the exception -- each has fully defined, tested behaviour ... but no public
  assembly syntax, and they are listed here for that reason rather than because their behaviour
  is undefined"); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:264
  (`CoprocessorCallShortImmediate = 0x2F`, a real decoded instruction, unlike
  `_Undocumented0x08/09/0B/0D`); sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:39-48
  (0x2F participates in the coprocessor privilege check; with an 8-bit command its
  coprocessor ID is always 0, so it is never privileged)
- resolution: manual-wrong
- fix: Replace "in the same class as opcodes \opcode{08}, \opcode{09}, \opcode{0B}, and
  \opcode{0D}" with something that does not group 0x2F with the architecturally-undefined
  opcodes, e.g. "Opcode 0x2F is listed as Undocumented because it has no public assembly syntax;
  its behaviour is defined (see Appendix~\ref{appendix:undocumented})."
- confidence: high

### F-cop-8
- file: chapters/16-coprocessor-instructions.tex
- lines: 736-738
- severity: minor
- category: fact
- claim: "overrunning the intended end of the transfer by however many words the first attempt already completed."
- evidence: chapters/16-coprocessor-instructions.tex:1077-1079 ("a fault leaves both address
  registers at the last fully completed chunk boundary, not at the exact word") and :733-735
  (same paragraph: "\reg{a} and \reg{l} only advance once a complete chunk ... has been both read
  and written")
- resolution: manual-wrong
- fix: Change to "overrunning the intended end of the transfer by the number of words in the
  chunks the first attempt fully completed" -- words transferred inside the faulting chunk do not
  advance \reg{a}/\reg{l} and so do not contribute to the overrun.
- confidence: high

### F-cop-9
- file: chapters/16-coprocessor-instructions.tex
- lines: 899-906, 993-1000, 1077-1085
- severity: major
- category: fact
- claim: "for a positive count, the address-register advance is the last step of the operation and does not run until the whole transfer completes, so a fault leaves it at its original, pre-transfer value no matter how many words were read. For a negative count, the address register is decremented to its final position \textit{before} the transfer loop begins"
- evidence: none -- there is no DMA unit in the implementation to check against. `sirc-vm/peripheral-cpu/src/lib.rs:400-447`
  routes every coprocessor ID other than 0 and 1 straight to an invalid-opcode fault, and the
  only DMA-related code in the tree is the assembler
  (`sirc-vm/toolchain/src/parsers/opcodes/coprocessor_meta.rs`), the bus access types
  (`sirc-vm/peripheral-bus/src/device.rs:26-27`), and a single fault test
  (`sirc-vm/peripheral-cpu/tests/exceptions/faults.rs:1273`). There is no DMA execution code and
  no DMA execution test anywhere under `sirc-vm/`.
- resolution: unclear
- fix: No manual change proposed. Question for the author: the forward/backward asymmetry, the
  chunked `DMAT` semantics, and the `12 + max(|n|-1,0)` / `12 + max(2n-1,0)` timing formulas are
  entirely manual-side design with no implementation or test to corroborate them -- is the DMA
  unit intended-but-unbuilt (in which case these are spec-ahead-of-code and fine, and worth
  recording as an implementation gap), or should an emulator model be added before the manual
  commits to this level of fault-time detail? Everything checked here is at least internally
  self-consistent: the Fault side effects fields match the Operation pseudocode at lines 872-882
  and 964-974, and the operand encoding matches the assembler at
  `sirc-vm/toolchain/src/parsers/opcodes/coprocessor_meta.rs:82-91, 140-163`
  (00=a, 01=l, 10=s in bits 7-6; direction bit 5; magnitude bits 2-0; range -7..7).
- confidence: low

### F-cop-10
- file: chapters/16-coprocessor-instructions.tex
- lines: 793-803
- severity: minor
- category: fact
- claim: "; Install at the Bus Fault, Bus Protection Fault, and Segment Overflow\n; Fault vectors (Chapter 6). Falls through to whatever other handling\n; those vectors need for faults unrelated to this transfer."
- evidence: sirc-vm/peripheral-cpu/src/lib.rs:193-203 and :150-155 (the fault-metadata link
  register records only the bus access type, the current and original fault, and the double-fault
  flag -- nothing identifying which instruction issued the DMA burst)
- resolution: manual-wrong
- fix: The bus-access-type test (`BAT = 0x5`/`0x6`) distinguishes DMA faults from non-DMA faults,
  but not *this* routine's `DMAT` from any other DMA instruction in the system; a DMA fault raised
  anywhere else would make this handler decrement and rewrite whatever happens to be on top of the
  stack. Add a sentence to the paragraph at lines 764-768, or a comment in the listing, noting
  that the example assumes `dma_copy_safe` is the only DMA user, and that a general system must
  also check the faulting return address (via `ETFR a, #6`) against the `dma_copy_attempt` range.
- confidence: medium

### F-cop-11
- file: chapters/16-coprocessor-instructions.tex
- lines: 1064-1069
- severity: minor
- category: fact
- claim: "\textbf{Write-back:} A successful non-zero \mnemonic{DMAT} advances \reg{a} and \reg{l} by the number of words transferred. ... \reg{r1}--\reg{r7} are clobbered"
- evidence: chapters/03-registers.tex:146 ("The link register pair stores return addresses for
  subroutine calls"); chapters/17-meta-instructions.tex:23, :26 (`BRSR` lowers to `LDEL p, ...`,
  `RETS` to `LDEA p, (#0, l)`)
- resolution: manual-wrong
- fix: `DMAT` is the only instruction in the manual that *mandates* the link register as an
  operand, so every `DMAT` destroys the enclosing subroutine's return address. The entry warns
  about \reg{r1}--\reg{r7} being clobbered but says nothing about \reg{l}. Add a sentence to
  Write-back or Notes, e.g. "Because \reg{l} is also the subroutine link register
  (Chapter~\ref{ch:registers}), a subroutine that issues \mnemonic{DMAT} must save and restore its
  return address around the transfer." This is the general form of the defect reported in F-cop-1.
- confidence: high

## Items checked and found correct

Recorded so the next pass does not re-derive them.

- Optional-coprocessor phrasing is consistent across all three DMA entries
  (lines 837-838, 929-930, 1023-1024) and matches the Integer Maths Unit entries (1185-1186,
  1274-1275); Table~\ref{tab:standard-coprocessor-ids} line 194 correctly shows
  Required = No for 0x2. Invalid-opcode-on-absence is stated at lines 12-14, 182-183, 655-656,
  702-703, 895-897, 989-991, 1073-1075 and matches `sirc-vm/peripheral-cpu/src/lib.rs:415-446`.
- `DMAT` is documented as immediate-only with no register form anywhere in the manual (checked
  every `DMAT` occurrence across `chapters/` -- 12-instruction-summary.tex:161,
  17-meta-instructions.tex:35, appendix-b-timing.tex:44, appendix-e-quick-reference.tex:64 all
  say `COPI #0x2Ann`), matching
  `sirc-vm/toolchain/src/parsers/opcodes/coprocessor_meta.rs:199-232`, which accepts only
  `DMAT a, l, #n` with a literal immediate 0-255 and rejects symbol operands and any other
  register pair. The Notes block at lines 1101-1120 describes the
  `ANDI[N]`/`ORRI[N]`/`COPR` technique as ordinary `COPR` usage, not as a `DMAT` form.
- The zero-operand-trailing-comment parser gap is not reintroduced: the `NOOP` example
  (17-meta-instructions.tex:102-107) puts the comment on its own line, and all 69 extracted
  listings assemble (`make -C examples/manual-examples objects`). `RETS`/`NOOP` are the affected
  mnemonics (`sirc-vm/toolchain/src/parsers/opcodes/meta.rs:50-59` requires a newline immediately
  after the tag); `WAIT`/`RETE`/`RSET` go through
  `sirc-vm/toolchain/src/parsers/opcodes/exception.rs` and do accept a same-line comment, which is
  why lines 400, 472, 542 and 815 assemble.
- Opcode assignments 0x0F / 0x2F / 0x3F match
  `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/definitions.rs:228, 264, 282`.
- Both `COPI` and `COPR` bytefields sum to 32 bits, and the Instruction Fields bit ranges
  (25-22, 25-18, 17-14, 21-6, 13-6, 5-4, 3-0) are consistent with them.
- Privilege rule "nibbles 0x0--0x7 user-callable, 0x8--0xF supervisor-only, decided on the
  run-time `rS` value for `COPR`" matches
  `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:39-50`
  (`cop_opcode = sr_b_ & 0x0F00`, privileged iff `> 0x0700`, evaluated at `InstructionDecode`).
- "A protected-mode `COPR sr` reads only the low byte of `sr`" matches
  `sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/fetch_and_decode.rs:44` with
  `SR_REDACTION_MASK = 0x00FF` (`sirc-vm/peripheral-cpu/src/registers.rs:11`).
- Exception-unit operation table (0x1, 0x9-0xD software; 0x0, 0xE-0xF internal; 0x2-0x8
  undefined) matches `ExceptionUnitOpCodes` at
  `sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/definitions.rs:149-162`, and the
  16 nibbles are fully accounted for.
- "A pending coprocessor command is discarded if a fault or hardware exception is dispatched
  before the coprocessor's dispatch slot begins" matches
  `sirc-vm/peripheral-cpu/src/coprocessors/exception_unit/execution.rs:469` and
  `sirc-vm/peripheral-cpu/src/lib.rs:442`.
- "The invalid-opcode fault records the offending 16-bit coprocessor command word in the
  `return_address` field of link register 7" matches `sirc-vm/peripheral-cpu/src/lib.rs:420,
  433-441, 193-196` with `FAULT_METADATA_LINK_REGISTER_INDEX = 7`
  (`sirc-vm/peripheral-cpu/src/lib.rs:65`).
- The retry example's `ANDI r6, #0x0007` / `CMPI r6, #0x0005` / `#0x0006` sequence is correct:
  the fault metadata register holds the bus access type in its low three bits
  (`sirc-vm/peripheral-cpu/src/lib.rs:150-155`) and `DmaReadBurst = 0b101`,
  `DmaWriteBurst = 0b110` (`sirc-vm/peripheral-bus/src/device.rs:26-27`). `ETFR r7, #7` reads the
  metadata (return status) and `ETFR a, #6` reads the fault return address (level-7 fault ->
  link register 6), consistent with the existing example at line 637.
- The example's stack discipline is correct: five words pushed (`ah`, `al`, `lh`, `ll`, `r1`),
  read back at offsets `(#0..#4, s)` in the right order, and released with `ADDI sl, #5`.
  `STOR` does not disturb the Z flag set by `SUBI`
  (`.../stages/write_back.rs:140-147, 295-297`), so the `BRAN|==` retry test is sound.
- `NOOP` lowering (`ADDI[N] r1, #0`, opcode 0x00, register field 0x0, immediate 0, AF `00`)
  matches `sirc-vm/toolchain/src/parsers/opcodes/meta.rs:82-95`,
  `StatusRegisterUpdateSource::None = 0b00`
  (`.../processing_unit/definitions.rs:96-102`) and `RegisterName::R1 = 0`
  (`sirc-vm/peripheral-cpu/src/registers.rs:47-48`). Three `NOOP`s at 6 cycles each is 18 cycles.
- The Ch. 17 Overview's new "exactly one real instruction, never a sequence" rule holds for every
  row of Table~\ref{tab:meta-instruction-cross-reference}: the assembler's `InstructionToken`
  carries exactly one `InstructionData`, and each meta parser returns one
  (`meta.rs`, `exception.rs`, `coprocessor_meta.rs`, `ljmp.rs`, `ldel.rs`, `branching.rs`).
- DMA timing formulas in Chapter 16 (lines 693-700, 911-914, 1005-1008, 1090-1094) agree with
  `chapters/appendix-b-timing.tex:43-45, 55-57`.
- `DMAW s, #-n` followed by `DMAR s, #n` does restore `r1`-`rn`, per the two Operation blocks.
