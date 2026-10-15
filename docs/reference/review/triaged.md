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

## Deferred (Gate 2)

### F-data-1
- file: chapters/04-data-representation.tex
- lines: 162-163
- severity: blocker
- category: fact
- claim: "the trap fires only when the low-word calculation wraps, whether the displacement is
  positive or negative" (segment-overflow trap).
- evidence: `execution_effective_address.rs:79-80` uses an *unsigned* `overflowing_add` for the
  displacement, whose carry-out is inverted relative to a signed wrap for a negative displacement.
  `STOR (#-4, s), r2` with `al=0x1000` spuriously faults; `al=0x0000` with `#-1` (a genuine
  underflow) does not trap at all. Chapters 8 and 14 restate the same (correct) manual rule, so
  this is self-consistent manual content, not a manual error.
- resolution: code-wrong
- fix: This is a Rust implementation bug, not a manual fix. `execution_effective_address.rs:79-80`
  needs a signed-wrap check for the displacement add (mirroring the existing pre-decrement fix at
  lines 84-94), plus a regression test for a negative displacement with `SR.A` set.
- confidence: high
- why deferred: `code-wrong` findings go to `implementation-bugs.md` for a separate Rust pass, not
  into this manual edit.

### F-exc-3
- file: chapters/06-exceptions.tex
- lines: 358-359
- severity: blocker
- category: fact
- claim: "Bits 8--11: Original fault type... indicates the fault type that was being handled when
  the double fault occurred."
- evidence: The implementation's `original_fault` field holds the *newly raised, triggering*
  fault, not the one already being serviced (lib.rs:185-203, 530-540); the field's own doc comment
  says "the fault that caused the double fault." The only double-fault test uses the same fault
  type on both sides, so it cannot distinguish the two readings.
- resolution: unclear
- fix: Needs a decision: does bits 8-11 hold the *first* fault (manual's current reading, requires
  a Rust change to carry the in-flight fault forward) or the *second/triggering* fault (code's
  reading, requires a manual wording change)? Either is a legitimate design; nothing currently
  settles which was intended.
- confidence: high (on the contradiction existing; the resolution itself needs a call)
- why deferred: genuine architectural decision, not a fact to look up.

### F-exc-4
- file: chapters/06-exceptions.tex
- lines: 438-441
- severity: major
- category: fact
- claim: Pending lower-priority interrupts "remain latched and are considered again after the
  current handler returns."
- evidence: The implementation re-tests the enable mask at *dispatch* time and only considers the
  single highest pending line; if that line was disabled after being latched, the function returns
  without dispatching anything, so a masked high-priority pending interrupt silently blocks all
  lower-priority ones too. The manual describes masking only at the sampling point.
- resolution: unclear
- fix: Needs a decision on intended behavior for an interrupt latched-then-disabled before
  dispatch: document the implementation's blocking behavior, or rule that latched interrupts
  dispatch regardless of a later mask change (requiring a Rust fix).
- confidence: medium
- why deferred: architectural decision.

### F-exc-5
- file: chapters/06-exceptions.tex
- lines: 139-140
- severity: major
- category: contradiction
- claim: "Invalid Opcode Fault... raised when a coprocessor call targets a non-existent coprocessor
  or requests an operation that the selected coprocessor does not implement."
- evidence: The same chapter (line 804) separately says reserved exception-unit opcodes are
  "architecturally undefined," not an invalid-opcode fault -- i.e. an unimplemented operation on
  an *existing* coprocessor is NOT always this fault. Chapter 16 takes the broader reading for the
  DMA unit. The implementation only raises InvalidOpCode for an unmapped coprocessor ID and panics
  (a TODO, unfinished) on a reserved exception-unit opcode.
- resolution: unclear
- fix: Reconcile: either scope the Invalid Opcode Fault description to "unmapped coprocessor ID or
  implemented-but-unsupported operation" and change line 804 to match, or carve out reserved
  exception-unit opcodes as a named exception.
- confidence: medium
- why deferred: two chapters disagree and the code path for one side is an acknowledged TODO/panic,
  not settled behavior.

### F-cop-9
- file: chapters/16-coprocessor-instructions.tex
- lines: 899-906, 993-1000, 1077-1085
- severity: major
- category: fact
- claim: DMA forward/backward address-register-advance timing and the fault-side-effect asymmetry
  between them.
- evidence: There is no DMA execution code or test anywhere in `sirc-vm/` to check this against --
  only the assembler, bus-access-type constants, and a single fault test exist. This is a case
  where the manual is entirely ahead of any implementation.
- resolution: unclear
- fix: No manual change proposed. Author question: is the DMA unit intended-but-unbuilt (fine as
  spec-ahead-of-code, worth logging as an implementation gap) or should an emulator model exist
  before the manual commits to this level of fault-time detail? Everything here is at least
  internally self-consistent (Fault side effects match the Operation pseudocode; operand encoding
  matches the assembler).
- confidence: low
- why deferred: nothing to verify against; a scope/priority call, not a fact.

### F-sum-4
- file: chapters/appendix-d-examples.tex
- lines: 46-49
- severity: major
- category: fact
- claim: ".EQU... usable anywhere a #immediate operand is accepted."
- evidence: Data directives (`.DW`/`.DB`/`.DQ`) reject `$NAME` outright, even though
  `toolchain/src/data/object.rs` already contains an unreachable `DataType::PlaceHolder` arm --
  looking like an unfinished toolchain feature rather than a deliberate restriction.
- resolution: unclear
- fix: Either narrow the manual claim to "usable anywhere an instruction takes a #immediate
  operand" (data directives take a literal or label only), or keep the broader claim and log the
  parser gap (`parse_data_type` needs a `parse_placeholder` branch) as an implementation bug.
- confidence: high (on the gap; the choice of manual vs. code fix is the open question)
- why deferred: per Source of Truth, the default lean is to log this as an implementation gap
  rather than narrow the manual -- but flagging for confirmation since it changes what the manual
  promises.

### F-sum-5
- file: chapters/appendix-d-examples.tex
- lines: 15-18
- severity: major
- category: fact
- claim: ".ORG addr sets the word address..." (no alignment requirement stated).
- evidence: An odd `.ORG` address is silently rounded down to the containing 2-word slot rather
  than erroring or being explicitly disallowed (verified by assembling/linking a test case).
- resolution: unclear
- fix: State the alignment rule ("the address must be even, since instructions and data
  directives each occupy a 2-word slot"), and/or log the silent-truncation-instead-of-error
  behavior as an implementation gap.
- confidence: high
- why deferred: whether silent truncation is acceptable-as-is or should error is a design call.

### F-sum-6
- file: chapters/appendix-d-examples.tex
- lines: 29-30
- severity: minor
- category: fact
- claim: "@label.r/.l/.u" override suffixes, described without scope restriction.
- evidence: On a data directive, any ref type other than FullAddress currently panics the linker
  ("Only FullAddress ref type currently supported for data directives").
- resolution: unclear
- fix: Either scope the manual text to instruction operands only ("a label reference in a
  .DB/.DW/.DQ directive is always a full address"), or log the linker's missing data-directive
  support for the other ref types as an implementation gap.
- confidence: high
- why deferred: same shape as F-sum-4/5 -- narrow the manual or log a toolchain gap.

### F-cons-13
- file: chapters/appendix-f-external-interface.tex
- lines: 370-374, 739
- severity: minor
- category: contradiction
- claim: "...only an assertion that begins and ends while the enable bit is 0 is lost" (implies
  sub-instruction-boundary pulses are latched).
- evidence: This contradicts the same paragraph's own opening sentence ("sampled at instruction
  boundaries") and makes a guarantee neither Chapter 5 nor Chapter 6 makes about pulses shorter
  than an instruction.
- resolution: unclear
- fix: Decide whether IRQ1-4 are latched continuously or only sampled at boundaries, then align
  Appendix F (both this paragraph and its summary row at line 739) with Chapter 6.
- confidence: medium
- why deferred: architectural decision about interrupt-controller behavior, not a lookup.

---

## Disputed — two reviewers reached different conclusions

### F-exc-2 vs. F-cons-3
- file: chapters/06-exceptions.tex (729-732) vs. chapters/appendix-b-timing.tex (47),
  chapters/16-coprocessor-instructions.tex (534-535)
- claim: Chapter 6 says the reset-vector fetch takes 4 clock cycles total; Appendix B and Chapter
  16 both say 6 (giving an 18-cycle RSET total: 6 call + 6 RSTO hold + 6 fetch).
- F-exc-2's take (resolution: manual-wrong, confidence: high): the implementation hard-codes 6
  cycles per exception-unit slot (`lib.rs:71`, `CYCLES_PER_INSTRUCTION = 6`; phase-advance is mod
  6) and a reset test polls exactly that many cycles, so Chapter 6's "4 cycles" is the outlier --
  fix Chapter 6 to describe one full 6-cycle slot (SR clear and PC load both committing on the
  4th cycle, two more cycles completing the slot), keeping the 18-cycle RSET total everywhere.
- F-cons-3's take (resolution: unclear, confidence: high): flagged the same three-way number
  mismatch without picking a side, since it only compares manual passages against each other and
  doesn't independently trace the Rust source.
- why flagged instead of auto-resolved: F-exc-2's resolution rests on real, specific Rust evidence
  (a named constant and a test's poll count) that F-cons-3 didn't have -- I lean toward accepting
  F-exc-2's fix (edit Chapter 6, not Appendix B/16) precisely because it's the only one of the two
  reports that actually checked the implementation rather than just comparing manual passages. But
  since one report calls this "unclear" and the other "high confidence," it's surfaced here rather
  than silently accepted.
