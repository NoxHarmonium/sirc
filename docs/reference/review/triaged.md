# Triaged findings — scoped editorial pass, 2026-09-16

Source reports: `facts-data.md` (G2), `facts-exc.md` (G3), `facts-cop.md` (G7), `facts-sum.md`
(G8), `consistency.md` (whole manual). 65 raw findings, deduplicated to 56 below. Status assigned
per the skill's default rule, refined so that `resolution: unclear` always defers regardless of
severity (Gate 2 exists specifically for those) and `resolution: code-wrong` always defers to
`implementation-bugs.md` rather than being "fixed" in the manual.

Legend: **status: accepted** = will be applied by a chapter editor without further confirmation.
**status: defer** = shown to the user at Gate 2.

---

## Accepted

### F-cons-1 (merges F-exc-1, F-cop-2)
- file: chapters/06-exceptions.tex
- lines: 636, 640, 644
- severity: blocker
- category: contradiction
- claim: Exception Quick Reference table lists "Next instruction" as the Return Address for Bus
  fault (0x01), Privilege fault (0x05), and Bus protection (0x09).
- evidence: Eight other passages across Ch. 6, Ch. 14, Appendix B, and Ch. 16 all say these three
  faults save the *faulting* instruction's address (e.g. 06-exceptions.tex:91-93, 273-275, 763-765;
  14-memory-instructions.tex:95; appendix-b-timing.tex:71-72: "Alignment, Bus, Bus Protection,
  Invalid Opcode, Privilege Violation, and Segment Overflow all save the address of the faulting
  instruction and are retryable"). Rows 0x02/0x03/0x04 already read "Faulting instruction"; only
  the three named rows are wrong.
- resolution: manual-wrong
- fix: Change the Return Address cell to "Faulting instruction" for vectors 0x01, 0x05, 0x09.
- confidence: high
- status: accepted

### F-cons-2
- file: chapters/09-shift-operations.tex
- lines: 492
- severity: blocker
- category: contradiction
- claim: "A register-supplied shift count greater than 15 is architecturally undefined" (bullet at
  line 492), while the same chapter's own §Shift Count (lines 322-337) gives the exact defined
  result per shift type, and 10 of 12 ALU instruction boxes in Chapter 13 state the defined rule
  (only the SHFT box and Chapter 14's STOR say undefined).
- evidence: 09-shift-operations.tex:322-337; 13-alu-instructions.tex:375,501,620,741,858,981,
  1107,1347,1483,1615 (defined, 10 boxes) vs. 261-262,316-317 (SHFT box, undefined);
  14-memory-instructions.tex:355-356 (undefined).
- resolution: manual-wrong
- fix: Adopt the majority (defined) form. Delete the bullet at 09-shift-operations.tex:492;
  rewrite the SHFT box (13-alu-instructions.tex:261-262, 316-317) and STOR
  (14-memory-instructions.tex:355-356) to match the other ten boxes.
- confidence: high
- status: accepted

### F-cons-4
- file: chapters/07-instruction-formats.tex
- lines: 253-254
- severity: blocker
- category: contradiction
- claim: "\texttt{ADDR r4, r5} uses R1=\texttt{0x4}, R2=\texttt{0x4}, R3=\texttt{0x5}" (reads
  register names as field values).
- evidence: Table~\ref{tab:register-encoding} (03-registers.tex:283-298) and
  generated/register-format-encodings.tex:38 both give r4=0x3, r5=0x4; 0x5 is r6, not an operand
  here at all.
- resolution: manual-wrong
- fix: "\texttt{ADDR r4, r5} uses R1=\texttt{0x3}, R2=\texttt{0x3}, R3=\texttt{0x4}".
- confidence: high
- status: accepted

### F-cons-5 (merges F-data-3, F-sum-14)
- file: chapters/05-status-register.tex, chapters/11-reading-instructions.tex,
  chapters/12-instruction-summary.tex, chapters/13-alu-instructions.tex
- lines: 05:190-193, 11:132-134, 12:101-104, 13:178-180 and 1219-1221
- severity: blocker (elevated from major/minor on the individual reports: three chapters actively
  disagree, not just one being loose)
- category: contradiction
- claim: Three mutually exclusive stories about a LOAD status-update override: Ch.11 implies the
  override may be written and is silently ignored; Ch.5 and Ch.12 say it may be written and yields
  U U U U; Ch.13 (twice) says the override cannot be written in public syntax at all, and only a
  non-public encoding yields U U U U.
- evidence: sirc-vm/toolchain/src/parsers/opcodes/load.rs:29-37 hard-rejects any override on
  LOAD; assembling `LOAD[A] r1, r2` fails. Ch.13's version is the only one consistent with the
  assembler and is stated twice (13-alu-instructions.tex:178-180, 1219-1221); Chapter 8's legality
  table also gives LOAD no override form (08-addressing-modes.tex:93).
- resolution: manual-wrong
- fix: Adopt Ch.13's wording everywhere. Ch.11 (132-134): "\mnemonic{LOAD} always encodes
  \texttt{[N]} and never updates the status flags; its public syntax does not accept a status
  update override." Ch.5 (190-193) and Ch.12 (101-104): "an encoding of the \mnemonic{LOAD}
  opcode with a non-\texttt{[N]} AF field, not reachable from public assembly syntax, leaves all
  four flags architecturally undefined (\texttt{U U U U})."
- confidence: high
- status: accepted

### F-data-2
- file: chapters/05-status-register.tex
- lines: 226-228, 10
- severity: major
- category: fact
- claim: "any direct write to the status register raises a privilege violation fault" in protected
  mode.
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:42-43 checks the
  *destination operand*, not whether the instruction actually writes it — so flag-only `CMPI`,
  `CMPR`, `TSAI`, `TSAR`, `TSXI`, `TSXR` naming `sr` also fault. Chapter 13 already documents one
  instance of this (13-alu-instructions.tex:1530, `TSAI sr, #0`).
- resolution: manual-wrong
- fix: "any instruction that names \reg{sr} as its destination operand raises a privilege
  violation fault -- including the flag-only comparison and test instructions (\mnemonic{CMPI},
  \mnemonic{CMPR}, \mnemonic{TSAI}, \mnemonic{TSAR}, \mnemonic{TSXI}, \mnemonic{TSXR}), which
  fault even though they do not write their destination." Widen the Overview bullet at line 10
  the same way.
- confidence: high
- status: accepted

### F-data-4
- file: chapters/05-status-register.tex
- lines: 100-105
- severity: minor
- category: fact
- claim: Protected-mode restricted-access bullet list has three items (high address-register
  components, high byte of SR, privileged coprocessor operations).
- evidence: A fourth restriction exists and is documented elsewhere:
  exception_unit/execution.rs:429-439 raises PrivilegeViolation for a software exception with
  vector < 0x60 in protected mode; 06-exceptions.tex:159 already documents it.
- resolution: manual-wrong
- fix: Add a fourth bullet: "Software exceptions targeting a vector below \texttt{0x60} (see
  Chapter~\ref{ch:exceptions})".
- confidence: high
- status: accepted

### F-data-5
- file: chapters/10-condition-codes.tex
- lines: 9-11, 111-112
- severity: minor
- category: fact
- claim: "If the condition is not met, the instruction has no architectural effect: no register,
  flag, memory, address-register or coprocessor state changes..."
- evidence: `p` still advances unconditionally at decode regardless of the condition
  (processing_unit/execution.rs:165), so "no register... changes" is misleading as written.
- resolution: manual-wrong
- fix: "...no architectural effect beyond the normal two-word advance of \reg{p}: no
  general-purpose register, flag, memory, address-register or coprocessor state changes..." Same
  fix at the Never (NV) entry, lines 111-112.
- confidence: high
- status: accepted

### F-data-6
- file: chapters/05-status-register.tex
- lines: 296-298
- severity: minor
- category: structure
- claim: The EA-bit-not-restored caveat is stated in full twice in the same chapter (lines
  222-224 and 296-298).
- evidence: Both statements are individually correct; this is duplication left over from this
  session's example-dedup pass.
- resolution: n/a
- fix: Shorten the second occurrence to a back-reference: "As with the stack restore in
  Section~\ref{sec:sr-manual-updates}, this restore does not reproduce bit 13 (\texttt{EA}); only
  \mnemonic{RETE} restores it."
- confidence: medium
- status: accepted

### F-data-7
- file: chapters/05-status-register.tex
- lines: 281-294
- severity: minor
- category: fact
- claim: "Saving and Restoring Status" example (`LOAD r7,sr` / ... / `LOAD sr,r7`) has no mode
  annotation.
- evidence: The example is silently supervisor-only (protected mode: read is masked, write
  faults -- protected_mode_test.rs:98). The neighbouring example is explicitly annotated
  "(from supervisor mode)"; this one is not.
- resolution: manual-wrong
- fix: "; Save status register (supervisor mode only -- see Section~\ref{sec:sr-manual-updates})"
- confidence: medium
- status: accepted

### F-data-8
- file: chapters/05-status-register.tex
- lines: 9-12
- severity: nit
- category: terminology
- claim: "Low Byte (bits 7--0): Condition flags" -- only bits 3-0 are condition flags; bits 4-7
  are reserved (per this chapter's own §Reserved Flags).
- evidence: write_back.rs:171-181, alu.rs:552-600 confirm bits 4-7 read as zero, consistent with
  "reserved," not "condition flags."
- resolution: manual-wrong
- fix: "Low Byte (bits 7--0): Condition flags (bits 3--0) and reserved bits (bits 7--4)"
- confidence: high
- status: accepted

### F-exc-2 / F-cons-3 — see "Disputed" section below (reviewers reached different conclusions)

### F-exc-6
- file: chapters/06-exceptions.tex
- lines: 54-56, 763-766
- severity: minor
- category: contradiction
- claim: "Retryable faults: ...the instruction is cancelled before it commits" (unqualified).
- evidence: The DMA-burst caveats added later in this chapter, and 16-coprocessor-instructions.tex
  :713-738, say this does not hold uniformly for DMAR/DMAW/DMAT.
- resolution: manual-wrong
- fix: Add after "...cancelled before it commits": "The exception is a DMA burst (\mnemonic{DMAR},
  \mnemonic{DMAW}, \mnemonic{DMAT}), which performs several bus accesses per instruction; see the
  individual fault descriptions below and Section~\ref{sec:dma-fault-retry}." Same pointer at
  lines 763-766.
- confidence: high
- status: accepted

### F-exc-7
- file: chapters/06-exceptions.tex
- lines: 402-404
- severity: minor
- category: contradiction
- claim: "A cause value of 0x0000 -- the value left in place at reset and whenever no exception is
  pending -- selects coprocessor ID 0x0."
- evidence: `pending_coprocessor_command` is seeded to `0x1B00` (the reset op) at reset and only
  cleared to 0 once the reset sequence completes (lib.rs:622-624; exception_unit/execution.rs:469).
- resolution: manual-wrong
- fix: "the value left in place once the reset sequence has completed, and whenever no exception
  is pending"
- confidence: high
- status: accepted

### F-exc-8
- file: chapters/06-exceptions.tex
- lines: 528-551
- severity: minor
- category: fact
- claim: Numbered "Exception Handling Flow" list omits two entry side effects that
  §Exception Entry Side Effects (569-579) and the implementation both perform as part of the same
  action.
- evidence: exception_unit/execution.rs:276-279 clears TraceMode and sets ExceptionActive together
  with the Protected Mode clear.
- resolution: manual-wrong
- fix: Add two steps after the Protected Mode step: "The TraceMode bit is cleared..." and "The
  Exception Active bit is set."
- confidence: high
- status: accepted

### F-exc-9
- file: chapters/appendix-c-undocumented.tex
- lines: 71-73, 131-133
- severity: minor
- category: fact
- claim: "The undocumented opcodes with bit 3 set in the ALU operation field compute alu_code &
  0x7... and update all four flags."
- evidence: This literally includes 0x2F (bit 3 set, operation field 0xF), which the
  implementation routes to CoprocessorCall, not the ALU test-variant path
  (execution_effective_address.rs:69; write_back.rs:146-153).
- resolution: manual-wrong
- fix: "whose ALU operation field is \texttt{0x8}--\texttt{0xE}" in both the Observed Behavior
  section and the merged Implementation Behavior subsection.
- confidence: high
- status: accepted

### F-cop-1
- file: chapters/16-coprocessor-instructions.tex
- lines: 780-830
- severity: blocker
- category: contradiction
- claim: The "Retry-Safe DMA Copy Routine" example uses `l` as the DMAT destination register and
  reaches `dma_copy_safe` via a call that also uses `l` as the link register, then returns with
  `RETS` (which reads its target from `l`).
- evidence: `RETS` lowers to `LDEA p, (#0, l)` (17-meta-instructions.tex:26); `BRSR`/`LJSR` write
  the return address into `l` at write-back (write_back.rs:247-268). Whichever way the routine is
  entered, `l` cannot simultaneously hold the return address and the DMA destination, and after
  `DMAT` advances `l`, the closing `RETS` jumps into the destination buffer.
- resolution: manual-wrong
- fix: Rework so the return address is saved/restored around the `DMAT` (e.g. push `lh`/`ll` at
  entry, load `l` with the destination inside the routine, restore `l` before `RETS`), or drop
  `RETS` and restructure as an inline fragment. Same defect at the `:dma_copy_failed` `RETS`
  (line 830). **Flagged for careful chapter-editor attention: this needs the example's structure
  rethought, not a one-line fix.**
- confidence: high
- status: accepted

### F-cop-3
- file: chapters/16-coprocessor-instructions.tex
- lines: 715-716
- severity: minor
- category: fact
- claim: "DMAR, DMAW, and DMAT are the only instructions... whose execution spans more than one
  bus access."
- evidence: every instruction already performs two bus accesses (instruction fetch high/low);
  LOAD/STOR add a third.
- resolution: manual-wrong
- fix: "more than one data-memory bus access"
- confidence: high
- status: accepted

### F-cop-4
- file: chapters/16-coprocessor-instructions.tex
- lines: 728-730
- severity: minor
- category: fact
- claim: Backward-DMA's eager pre-decrement is compared to "the same eager-decrement pattern used
  by pre-decrement addressing elsewhere in this architecture."
- evidence: A pre-decrement STOR's address-register update is only committed at write-back, so a
  fault leaves it untouched -- the opposite of the backward-DMA behavior being described.
- resolution: manual-wrong
- fix: "unlike pre-decrement addressing, whose address-register update is only committed at
  write-back and so is undone by a fault, the DMA decrement is committed before the transfer loop
  begins" -- or drop the comparison.
- confidence: high
- status: accepted

### F-cop-5
- file: chapters/16-coprocessor-instructions.tex
- lines: 772-778
- severity: minor
- category: fact
- claim: Save/restore-around-DMAT header comment warns about a/l pointing at the wrong address "if
  it faults mid-transfer," using `DMAT a, l, #7`.
- evidence: A 7-word transfer is a single chunk; a/l only advance after a full chunk completes, so
  a mid-transfer fault on this exact example can never leave them advanced.
- resolution: manual-wrong
- fix: Raise the example's count above 7 (e.g. #16, matching the usage example at line 1128) or
  reword the comment to say the pattern matters for counts above seven.
- confidence: high
- status: accepted

### F-cop-6
- file: chapters/16-coprocessor-instructions.tex
- lines: 773, 785, 806-809
- severity: minor
- category: fact
- claim: "transfer up to 3 times if a recoverable DMA fault occurs" / "LOAD r1, #3 ; retry budget."
- evidence: The handler decrements first and gives up at zero, so a budget of 3 yields 2 retries
  (3 attempts total), not 3 retries.
- resolution: manual-wrong
- fix: Reword to "retrying the whole transfer up to 2 more times" / "up to 3 attempts in total",
  or change to `LOAD r1, #4`.
- confidence: medium
- status: accepted

### F-cop-7
- file: chapters/16-coprocessor-instructions.tex
- lines: 36-38
- severity: minor
- category: contradiction
- claim: "Opcode 0x2F is Undocumented, in the same class as opcodes 0x08, 0x09, 0x0B, and 0x0D."
- evidence: Appendix C itself says 0x2F is the *exception* -- fully defined behavior, just no
  public syntax -- unlike the truly architecturally-undefined 0x08/09/0B/0D.
- resolution: manual-wrong
- fix: "Opcode 0x2F is listed as Undocumented because it has no public assembly syntax; its
  behavior is defined (see Appendix~\ref{appendix:undocumented})."
- confidence: high
- status: accepted

### F-cop-8
- file: chapters/16-coprocessor-instructions.tex
- lines: 736-738
- severity: minor
- category: fact
- claim: A failed retry "overrun[s] the intended end of the transfer by however many words the
  first attempt already completed."
- evidence: Words inside the faulting chunk do not advance a/l and so do not contribute to the
  overrun -- only fully-completed chunks do.
- resolution: manual-wrong
- fix: "by the number of words in the chunks the first attempt fully completed"
- confidence: high
- status: accepted

### F-cop-10
- file: chapters/16-coprocessor-instructions.tex
- lines: 793-803
- severity: minor
- category: fact
- claim: Comment implies the fault handler is scoped to "this transfer."
- evidence: The fault-metadata link register records only the bus access type and fault
  identity, nothing identifying which instruction issued the DMA burst, so a DMA fault anywhere
  else in the system would also be caught by this handler.
- resolution: manual-wrong
- fix: Note that the example assumes `dma_copy_safe` is the only DMA user, and that a general
  handler must also check the faulting return address (`ETFR a, #6`) against the expected range.
- confidence: medium
- status: accepted

### F-cop-11
- file: chapters/16-coprocessor-instructions.tex
- lines: 1064-1069
- severity: minor
- category: fact
- claim: "Write-back: ...r1-r7 are clobbered" (no mention of `l`).
- evidence: `l` is the subroutine link register elsewhere in the manual; DMAT is the only
  instruction that mandates it as an operand, so every DMAT destroys the enclosing subroutine's
  return address (general form of F-cop-1's defect).
- resolution: manual-wrong
- fix: Add: "Because \reg{l} is also the subroutine link register (Chapter~\ref{ch:registers}), a
  subroutine that issues \mnemonic{DMAT} must save and restore its return address around the
  transfer."
- confidence: high
- status: accepted

### F-sum-1
- file: chapters/appendix-d-examples.tex
- lines: 27, 32-33, 42-45
- severity: blocker
- category: fact
- claim: "...the label's full 32-bit address" / "a stored address must be able to address anywhere
  in the 32-bit address space" (three occurrences).
- evidence: Chapters 1 and 4 both state a 24-bit address space; the linker stores a 24-bit word
  address zero-extended into the 4-byte slot (toolchain/src/types/object.rs:15-16,
  bin/linker.rs:130,137-140). This is an error introduced in this session's own new section.
- resolution: manual-wrong
- fix: Replace "32-bit" with "24-bit" in all three locations.
- confidence: high
- status: accepted

### F-sum-2
- file: chapters/appendix-d-examples.tex
- lines: 15-16
- severity: major
- category: contradiction
- claim: Item headword is `.ORG \#addr`; body sentence says the address takes no `#`.
- evidence: parsers/shared.rs:116-120 rejects `.ORG #0x0010` outright.
- resolution: manual-wrong
- fix: Change the headword to `.ORG addr` to match the body and the grammar.
- confidence: high
- status: accepted

### F-sum-3
- file: chapters/appendix-d-examples.tex
- lines: 21-22
- severity: major
- category: fact
- claim: "@label references a previously declared label."
- evidence: Example D-1 in the same section forward-references `@start` before `:start` is
  declared, and it assembles/links successfully -- forward references are the normal case (e.g.
  every reset vector).
- resolution: manual-wrong
- fix: "References a label declared anywhere in the program, before or after the reference;
  references are resolved at link time."
- confidence: high
- status: accepted

### F-sum-7
- file: chapters/appendix-d-examples.tex
- lines: 24-28
- severity: minor
- category: fact
- claim: "In an immediate-form LOAD, it resolves to the label's low word only."
- evidence: The same default applies to STOR, LDEA, LDEL, LJMP, LJSR -- only BRAN/BRSR differ.
- resolution: manual-wrong
- fix: "In any other instruction that takes a symbol operand (\mnemonic{LOAD}, \mnemonic{STOR},
  \mnemonic{LDEA}, \mnemonic{LDEL}, \mnemonic{LJMP}, \mnemonic{LJSR}), it resolves to the label's
  low word only."
- confidence: high
- status: accepted

### F-sum-8
- file: chapters/appendix-d-examples.tex
- lines: 52-55
- severity: minor
- category: fact
- claim: "Every program in this appendix begins with a .ORG 0x0000 / .DQ @label pair."
- evidence: `faults-high.sasm` (Example D-4) and `hardware-exception/data.sasm` (Example D-7) do
  not, being a separately-mapped segment and a standalone data segment respectively.
- resolution: manual-wrong
- fix: "Every program in this appendix that is loaded at segment zero begins with... the
  separately mapped high segment and the standalone data segment shown later do not need one."
- confidence: high
- status: accepted

### F-cons-10 (merges F-sum-9)
- file: chapters/appendix-d-examples.tex
- lines: 64-65, 73
- severity: minor
- category: contradiction
- claim: Label `:overflow_handler` / `@overflow_handler` at vector 0x40.
- evidence: 06-exceptions.tex:620 assigns 0x40 to the Level 2 hardware exception, not to segment
  overflow (which is vector 0x03).
- resolution: manual-wrong
- fix: Rename to `:level2_handler` (both the declaration and the reference) and reword the
  comment to "vector 0x40: level 2 hardware exception vector".
- confidence: medium
- status: accepted

### F-cons-11 (merges F-sum-13)
- file: chapters/appendix-d-examples.tex
- lines: 132, 155
- severity: minor
- category: contradiction
- claim: "installed at vector 0x0080" / "the handler at vector 0x0500."
- evidence: 0x0080 is the *address* of vector 0x40, and 0x0500 is past the 256-entry vector
  table's end entirely -- it's a handler code address, not a vector.
- resolution: manual-wrong
- fix: "installed at vector \texttt{0x40} (word address \addr{0x0080})" and "the handler at
  address \addr{0x0500}".
- confidence: high
- status: accepted

### F-sum-10
- file: chapters/appendix-d-examples.tex
- lines: 108
- severity: minor
- category: fact
- claim: "Each handler, defined from line 144 onward..."
- evidence: `faults.sasm` gained 66 lines in a later commit; the handler section now starts at
  line 158.
- resolution: manual-wrong
- fix: "Each handler, defined from line 158 onward..."
- confidence: high
- status: accepted

### F-sum-11
- file: chapters/appendix-d-examples.tex
- lines: 109-111
- severity: minor
- category: fact
- claim: "The segment-overflow handler (lines 167--204)..."
- evidence: The segment-overflow handler is actually at lines 197-242 in the current file; 167-204
  spans other handlers.
- resolution: manual-wrong
- fix: "The segment-overflow handler (lines 197--242)..."
- confidence: high
- status: accepted

### F-sum-12
- file: chapters/appendix-d-examples.tex
- lines: 87-89
- severity: minor
- category: fact
- claim: "storing one byte per candidate number in a memory-mapped array."
- evidence: Memory is word-addressed (no narrower unit exists); the sieve stores full 16-bit
  words per candidate.
- resolution: manual-wrong
- fix: "storing one word per candidate number..." (the program's name is historical, from the
  classic byte-per-flag sieve, not a statement about SIRC-1's addressing).
- confidence: medium
- status: accepted

### F-sum-15
- file: chapters/11-reading-instructions.tex
- lines: 19-22
- severity: minor
- category: structure
- claim: "Values come from Chapter 7 and the facts digest; an entry must not invent a value here."
- evidence: "the facts digest" is an internal review artifact, not part of the published manual --
  no reader can consult it.
- resolution: manual-wrong
- fix: "Values come from Chapter~\ref{ch:instruction-formats}; an entry does not introduce a value
  that is not defined there."
- confidence: high
- status: accepted

### F-sum-16
- file: chapters/11-reading-instructions.tex
- lines: 73-75
- severity: nit
- category: fact
- claim: "@label: ...a low-word address otherwise" (the notation table's simplified entry).
- evidence: Not true for data directives, which resolve to a full address; the parenthetical
  partly covers this already.
- resolution: manual-wrong
- fix: "...a low-word address in other instruction operands (see Section~\ref{sec:
  assembler-directives} for the full resolution rules, including data directives)."
- confidence: medium
- status: accepted

### F-cons-6
- file: chapters/notation-and-glossary.tex
- lines: 89-91
- severity: major
- category: contradiction
- claim: Glossary entry for coprocessor dispatch marks only the maths unit "optional," implying
  the DMA unit is required by contrast.
- evidence: Every other file (Ch.1, Ch.16) is consistent that DMA (coprocessor 0x2) is optional;
  the glossary is the one holdout.
- resolution: manual-wrong
- fix: "...the exception unit (0x1), the optional DMA unit (0x2), and the optional integer maths
  unit (0x3)."
- confidence: high
- status: accepted

### F-cons-7
- file: chapters/appendix-e-quick-reference.tex
- lines: 189-190
- severity: major
- category: contradiction
- claim: "See Chapter~\ref{ch:cpu-architecture}, Section \"Pin Descriptions\"..."
- evidence: Chapter 2 has no such section; "Pin Descriptions" is Appendix F's subsection
  (appendix-f-external-interface.tex:152), which also has the PROT-pin example being promised.
  The `\ref` resolves (to Ch.2) so this passes mechanical checks but sends the reader to the wrong
  place.
- resolution: manual-wrong
- fix: "See Appendix~\ref{appendix:external-interface}, Section \"Pin Descriptions\"..."
- confidence: high
- status: accepted

### F-cons-8
- file: chapters/notation-and-glossary.tex
- lines: 65-67
- severity: major
- category: contradiction
- claim: "[reserved]... must write zero unless a chapter states an explicit encoding."
- evidence: This contradicts Chapter 5's advisory-only framing (which permits a supervisor write
  to reserved SR bits) and an already-approved STYLE.md decision explicitly rejecting the
  "must be written as zeros" form.
- resolution: manual-wrong
- fix: "Software must not assume any particular value when reading a reserved field, and must not
  rely on any behaviour it may have. A future CPU revision may assign a reserved field a meaning."
  Drop the write-zero requirement. STYLE.md §2 carries the same wording and should be corrected
  too.
- confidence: high
- status: accepted

### F-cons-9
- file: chapters/12-instruction-summary.tex
- lines: 124-127
- severity: minor
- category: contradiction
- claim: "+0x00 to +0x07: Save result... +0x08 to +0x0E: Test only" (widens both ranges by one
  compared to Ch.7 and Appendix A).
- evidence: This makes 0x27 read as a documented save-result form and 0x08/09/0B/0D as documented
  test-only forms; both are actually undocumented per this same chapter's own table.
- resolution: manual-wrong
- fix: "+0x00 to +0x06: Save result to destination register; +0x07: LOAD (0x27 is undocumented);
  +0x08 to +0x0E: test-only where a documented mnemonic exists; +0x0F: coprocessor call."
- confidence: high
- status: accepted

### F-cons-12
- file: chapters/notation-and-glossary.tex
- lines: 99-100
- severity: minor
- category: contradiction
- claim: "[Fault]... Faults cannot be masked or deferred."
- evidence: Chapter 6 documents two conditional faults: Segment Overflow requires SR.A, and
  Instruction Trace requires SR.T or TRCE.
- resolution: manual-wrong
- fix: Add: "...though two of them are conditional on whether they are raised at all: Segment
  Overflow requires \texttt{SR.A}, and Instruction Trace requires \texttt{SR.T} or the
  \texttt{TRCE} input."
- confidence: high
- status: accepted

### F-cons-14
- file: chapters/13-alu-instructions.tex
- lines: 220, 1348-1349, 1484-1485
- severity: minor
- category: contradiction
- claim: "AF encoding 11 is reserved... the status register is left unchanged" (promises specific
  behavior for a reserved encoding).
- evidence: Chapter 7, Chapter 11, the glossary, and Appendix C all describe AF=11 as reserved and
  therefore unspecified, with Appendix C expecting a future revision to repurpose it.
- resolution: manual-wrong
- fix: Drop the parenthetical everywhere it appears ("AF encoding 11 (binary) is reserved and has
  no assembly syntax."). Grep pattern: `Reserved (status register unchanged)`.
- confidence: medium
- status: accepted

### F-cons-15
- file: chapters/appendix-d-examples.tex, chapters/appendix-e-quick-reference.tex
- lines: d:21, e: (whole file, unreferenced)
- severity: minor
- category: structure
- claim: N/A -- missing cross-references, not a wrong claim.
- evidence: Chapter 11 points to Appendix D's assembler-directives section, but nothing points
  back; `\label{ch:example-programs}` and `\label{ch:quick-reference}` are never referenced from
  anywhere despite the preface advertising both appendices.
- resolution: n/a
- fix: Add a return-leg sentence to Appendix D's `@label` entry pointing back to Chapter 11's
  notation table.
- confidence: high
- status: accepted

### F-cons-16
- file: chapters/appendix-d-examples.tex, chapters/appendix-e-quick-reference.tex
- lines: 2, 2
- severity: nit
- category: latex
- claim: N/A -- label naming inconsistency.
- evidence: Four of six appendices use the `appendix:` label prefix; D and E use `ch:`.
- resolution: n/a
- fix: Rename to `\label{appendix:example-programs}` and `\label{appendix:quick-reference}`. Both
  currently unreferenced (F-cons-15), so no existing `\ref` needs updating.
- confidence: high
- status: accepted

### F-cons-17
- file: chapters/15-control-flow.tex, chapters/16-coprocessor-instructions.tex
- lines: 33-53, 370, 497, 25-26, 33-36, 76
- severity: minor
- category: style
- claim: N/A -- house-style conformance.
- evidence: STYLE.md §3 requires `\opcode{}` for literal opcode bytes; these tables and lines use
  bare `0xNN` text instead.
- resolution: n/a
- fix: Wrap in `\opcode{}`. Grep pattern: `& 0x[0-9A-F][0-9A-F] *&` and `Opcodes:} 0x`.
- confidence: high
- status: accepted

### F-cons-18
- file: chapters/16-coprocessor-instructions.tex
- lines: 131-135, 293-305, 368-377, 438-449, 509-519, 604-616, 872-882, 964-974, 1054-1062,
  1230-1242, 1320-1329
- severity: minor
- category: style
- claim: N/A -- environment choice.
- evidence: Chapter 16 is the only instruction-reference chapter using `lstlisting` (syntax
  highlighted as SIRC assembly) for Operation pseudocode instead of `verbatim`; this makes
  pseudocode constructs (`MULU:`, `else:`, `for i in 1..count:`) render as if they were
  trailing-colon assembler labels, which they are not.
- resolution: n/a
- fix: Convert Chapter 16's Operation blocks from `lstlisting` to `verbatim`, matching Chapters
  13-15.
- confidence: high
- status: accepted

### F-cons-19
- file: chapters/05-status-register.tex, chapters/16-coprocessor-instructions.tex,
  chapters/06-exceptions.tex
- lines: 05:266, 16:812, 06:149
- severity: minor
- category: terminology
- claim: N/A -- house-style conformance (approved decision: spell out "for example", never
  "e.g."; "i.e." also to go).
- evidence: STYLE.md's approved decisions.
- resolution: n/a
- fix: Replace all three "e.g." with "for example" and the one "i.e." with "that is". Grep
  pattern: `e\.g\.` and `i\.e\.`.
- confidence: high
- status: accepted

### F-cons-20
- file: chapters/appendix-c-undocumented.tex, chapters/appendix-e-quick-reference.tex
- lines: c: 7,8,11,13 (+1 more), e: 17
- severity: minor
- category: terminology
- claim: N/A -- spelling consistency.
- evidence: 17 other files use "behavior"; only these two files (6 instances total) use
  "behaviour", and appendix-c mixes both spellings in adjacent sentences.
- resolution: n/a
- fix: Standardize on "behavior". Grep pattern: `behaviour`.
- confidence: high
- status: accepted

### F-cons-21
- file: chapters/16-coprocessor-instructions.tex
- lines: 1104-1106
- severity: nit
- category: terminology
- claim: "...which would make its instruction count unpredictable like every other
  meta-instruction's is not."
- evidence: STYLE.md reserves "unpredictable" as a term never used in this manual (to avoid
  blurring with "architecturally undefined"/"implementation-defined"); this is also the manual's
  only instance and the sentence is hard to parse as written.
- resolution: n/a
- fix: "which would make its instruction count vary with its operand, unlike every other
  meta-instruction."
- confidence: medium
- status: accepted

---

## Resolved at Gate 2 (2026-09-16, author's calls, now accepted)

### F-data-1 — no manual change, logged as implementation bug
- file: chapters/04-data-representation.tex (unchanged)
- Author confirmed this is a real Rust bug, not a manual error, and that the September 14 fix
  (`d6b8eb4`) only covered the pre-decrement special case, not the general displacement path.
  Logged as **IB-1** in `implementation-bugs.md`. No manual edit; the manual's rule is already
  correct and consistent across Chapters 4, 8, and 14.
- status: no manual change (implementation-bugs.md only)

### F-exc-3 — accepted, manual-wrong
- file: chapters/06-exceptions.tex
- lines: 358-359
- severity: blocker
- Author's call: bits 8-11 should hold the fault that *triggered* the double-fault escalation
  (matching current code), not the fault already being serviced. Reasoning: the fault already
  being serviced is always cheaply recoverable by convention (any handler can note its own
  identity on entry, since it has its own cause-register value at hand before anything else can
  go wrong), while the triggering fault is genuinely unrecoverable any other way once inside the
  double-fault handler -- so the code's choice is more informative to preserve, and the manual is
  the side that should change.
- fix: Reword bits 8-11's description from "the fault type that was being handled when the double
  fault occurred" to "the fault that triggered the escalation to a double fault (the second fault,
  not the one already being handled when it occurred)."
- status: accepted, confidence: high

### F-exc-4 — fixed in Rust (IB-3), manual gets one clarifying sentence
- file: chapters/06-exceptions.tex
- lines: 438-441
- severity: major
- Author confirmed this was an oversight, not a deliberate simplification. **Fixed in Rust this
  pass**: `get_cause_register_value` now iterates every pending interrupt line from highest to
  lowest priority and dispatches the first one that is both pending and still enabled, instead of
  only ever considering the single highest pending line (see the commit that added this entry;
  two new unit tests cover it). Logged as **IB-3** (fixed) in `implementation-bugs.md`.
- fix: Add one sentence after the existing text: "If the highest-priority pending interrupt's
  enable bit has been cleared since it was latched, it does not block dispatch of any other
  pending, enabled interrupt of lower priority."
- status: accepted, confidence: high (manual edit is now describing settled, tested behavior)

### F-exc-5 / Invalid Opcode Fault scope — accepted, manual-wrong (both chapters), plus a logged Rust gap
- file: chapters/06-exceptions.tex (139-140), chapters/16-coprocessor-instructions.tex (702-703
  and, if the same pattern repeats, the DMAR/DMAW/DMAT entries' own Exceptions fields)
- severity: major
- Author's call, confirmed against the manual's own existing "Model and Revision Policy" (Chapter
  16, ~line 209-213), which already states the intended design precisely: "Reserved coprocessor
  IDs must raise invalid-opcode faults when they are not implemented by the selected CPU model.
  Reserved operation nibbles and reserved operand values within an implemented coprocessor are
  architecturally undefined." Chapter 6's broad wording and Chapter 16's DMA-specific line
  (702-703, "unsupported DMA operations... raise invalid-opcode faults") both contradict this
  already-settled policy; the policy statement itself is correct and should not change.
- fix (Ch. 6, 139-140): narrow to "Raised when a coprocessor call targets a coprocessor ID not
  implemented by the selected CPU model." Drop "or requests an operation that the selected
  coprocessor does not implement" -- that case is architecturally undefined, not this fault. Keep
  the following paragraph about forward-compatible probing (it's specifically about the
  missing-coprocessor case, still correct under the narrower scope).
- fix (Ch. 16, 702-703): "Missing DMA hardware raises an invalid-opcode fault. Unsupported DMA
  operations and reserved operand encodings are architecturally undefined (see \S16.2, Model and
  Revision Policy)." Check the DMAR/DMAW/DMAT instruction entries' own Exceptions fields for the
  same "unsupported operation -> invalid-opcode fault" phrasing and correct them to match if
  present.
- Rust side: the exception-unit's reserved-opcode panic (`exception_unit/execution.rs:69-77`) is
  already a known gap -- it should become non-crashing undefined behavior, not a fault. Logged as
  **IB-2** in `implementation-bugs.md`, not fixed this pass (author did not ask for this one to be
  fixed now, unlike F-exc-4/IB-3).
- status: accepted, confidence: high

### F-cop-9 — no change, confirmed acceptable as spec-ahead-of-code
- file: chapters/16-coprocessor-instructions.tex (unchanged)
- Author confirmed: "DMA is just a spec for now. As long as it sounds reasonable it's all good."
  No manual change, no implementation-bugs entry (nothing is broken -- there's just no
  implementation yet to check against).
- status: no change

### F-sum-4 — no manual change, logged as unfinished toolchain feature
- file: chapters/appendix-d-examples.tex (unchanged)
- Author's call: keep the manual's broader `.EQU` claim as-is; log the parser gap instead of
  narrowing the manual. Logged as **IB-4** in `implementation-bugs.md`.
- status: no manual change

### F-sum-5 — accepted, manual-wrong
- file: chapters/appendix-d-examples.tex
- lines: 15-18
- severity: major
- Author's call: state the alignment rule directly, matching the architectural fact that every
  instruction and data directive occupies a fixed 2-word slot.
- fix: Add to the `.ORG` entry: "The address must be even (2-word aligned), since instructions and
  data directives each occupy a 2-word slot."
- status: accepted, confidence: high

### F-sum-6 — no manual change, logged as unfinished toolchain feature
- file: chapters/appendix-d-examples.tex (unchanged)
- Author's call: log the linker's missing data-directive ref-type support as an implementation gap
  rather than narrowing the manual's description of the `.r`/`.l`/`.u` suffixes. Logged as **IB-5**
  in `implementation-bugs.md`.
- status: no manual change

### F-cons-13 — accepted, manual-wrong
- file: chapters/appendix-f-external-interface.tex
- lines: 370-374, 739
- severity: minor
- Author's call, on period-appropriate grounds: real level-sensitive IRQ lines on simple 1980s
  CPUs without an external interrupt controller (68000/6502/6800-style designs) put the burden of
  holding the line on the requesting device, not on the CPU to remember a sub-instruction pulse --
  matching what Chapter 5 and Chapter 6 already say about lines 1-4 being plain level-sensitive.
- fix: Replace the "even if released before the next boundary" latching guarantee with: "The
  requesting device must hold the line asserted until it is sampled and serviced; an assertion
  that begins and ends between instruction boundaries, before the CPU samples it, is not
  recognized." Apply the same correction to the summary row at line 739.
- status: accepted, confidence: high

---

## Resolved after Phase 3 (2026-09-16, author's own investigation)

### F-exc-2 vs. F-cons-3
- file: chapters/06-exceptions.tex (729-732) vs. chapters/appendix-b-timing.tex (47),
  chapters/16-coprocessor-instructions.tex (534-535)
- claim: Chapter 6 said the reset-vector fetch takes 4 clock cycles total; Appendix B and Chapter
  16 both say 6 (giving an 18-cycle RSET total: 6 call + 6 RSTO hold + 6 fetch).
- F-exc-2's take (resolution: manual-wrong, confidence: high): the implementation hard-codes 6
  cycles per exception-unit slot (`lib.rs:71`, `CYCLES_PER_INSTRUCTION = 6`; phase-advance is mod
  6) and a reset test polls exactly that many cycles, so Chapter 6's "4 cycles" is the outlier.
- F-cons-3's take (resolution: unclear, confidence: high): flagged the same three-way number
  mismatch without picking a side, since it only compares manual passages against each other and
  doesn't independently trace the Rust source.
- resolution (author-verified 2026-09-16): the author asked whether any existing test traces the
  reset-vector-fetch cycle count directly. It does --
  `peripheral-cpu/tests/exceptions/reset.rs::test_hardware_rsti_reset` polls through the 6-cycle
  RSTO hold, then the vector fetch itself, ending with
  `poll_n_feedback(&mut bus, ACK, CYCLES_PER_INSTRUCTION - 1)` (5 more cycles after the first) --
  6 total -- before asserting the PC landed correctly. Tracing
  `exception_unit/execution.rs`'s `Reset` op-code arm against the six `ExecutionPhase` variants
  confirms why: `InstructionFetchLow`/`InstructionFetchHigh`/`InstructionDecode` fetch and latch
  the two vector words, `ExecutionEffectiveAddressExecutor` commits `sr = 0x0` and the new `p`
  together (matching Chapter 6's "steps 3 and 4 happen together"), but `MemoryAccessExecutor` and
  `WriteBackExecutor` still consume a cycle each even though Reset has no work to do there -- the
  same fixed six-phase pipeline every instruction and coprocessor dispatch runs through
  (Chapter~2). F-exc-2's reading was correct: 6 cycles, not 4.
- fix applied: `06-exceptions.tex` (Reset Vector Fetch) reworded to state the four listed steps
  are "four clock cycles of substantive work" but the dispatch still occupies two further
  pipeline cycles (memory-access, write-back) doing nothing, for six clock cycles total -- not
  four. Appendix B and Chapter 16 needed no change; their 6-cycle / 18-cycle-total figures were
  already correct.
