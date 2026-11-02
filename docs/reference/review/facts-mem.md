# Findings: G6 memory and control flow (reviewer tag: mem)

### F-mem-1
- file: reference/chapters/15-control-flow.tex
- lines: 590-598
- severity: major
- category: fact
- claim: "if condition:\n    l = address_of_next_instruction\n    dest.high = src.high\n    dest.low  = src.low + displacement"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/stages/write_back.rs:226-245 (dest high = decoded.ad_h_, latched at decode; dest low = alu_output, computed in phase 3 before any write-back). docs/reference/chapters/07-instruction-formats.tex:608-610 says register reads use values latched before write-back. The non-post-increment form with src = l is legal (15-control-flow.tex:568-569, 770-771; not in the rejected list at :73-78).
- resolution: manual-wrong
- fix: Reorder the pseudocode so the target is computed from the original source before l is written: "target = src.high : (src.low + displacement); new_src_low = src.low + 1; l = address_of_next_instruction; dest = target; if post_increment: src.low = new_src_low". Add a sentence: "The source is read before the link is written, so LDEL p, (#0, l) (LJSR l) jumps to the old value of l." Apply the same change to the LJSR operation at 795-803 and the syntax comments at 578-583 and 783-788.
- confidence: high

### F-mem-2
- file: reference/chapters/15-control-flow.tex
- lines: 72-78
- severity: major
- category: fact
- claim: "The assembler rejects exactly these forms: ... and any auto-update of \reg{p} (\texttt{(p)+} or \texttt{-(p)} in any instruction)"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/load.rs:141-149,195-206; store.rs:156-166,212-223,246-255; ldea.rs:133-140,187-196; ldel.rs:135-157 check only register/pair overlap, never p. Only ljsr.rs:160-166,220-226 rejects p. Known gap: general-todo.md item 27.
- resolution: code-wrong
- fix: Keep the manual. Same claim at 14-memory-instructions.tex:78-80 and the src-field notes at 15-control-flow.tex:438, 569. Toolchain todo 27.
- confidence: high

### F-mem-3
- file: reference/chapters/15-control-flow.tex
- lines: 110-113
- severity: major
- category: fact
- claim: "With \texttt{SR.A} set, an instruction in the last slot of a segment also raises a segment-overflow fault if it uses the wrapped next-instruction address, either as the next program counter (condition false) or as the return address saved by \mnemonic{LDEL}, \mnemonic{BRSR}, or \mnemonic{LJSR}; a taken branch or jump that saves no return address does not"
- evidence: sirc-vm/peripheral-cpu/src/coprocessors/processing_unit/execution.rs:99-107,138-148 sets a flag at decode and raises the fault at the next fetch, so it is charged to whatever executes next and is not tied to the link use. Known gap: general-todo.md item 15.
- resolution: code-wrong
- fix: Keep the manual; simulator todo 15.
- confidence: high

### F-mem-4
- file: reference/chapters/14-memory-instructions.tex
- lines: 104-108
- severity: major
- category: contradiction
- claim: "effective-address calculation can raise a segment-overflow fault when \texttt{SR.A} (Trap on Address Overflow) is set and the low-word address calculation wraps"
- evidence: docs/reference/chapters/04-data-representation.tex:152-154 and 06-exceptions.tex:243-245 say the post-increment/pre-decrement auto-update is checked separately and a wrap there also faults. Ch14 (here and at 256-258, 414-416) and Ch15 (98, 108-110, 479, 610) name only the effective-address calculation. Code currently checks the wrong quantities (general-todo.md item 16).
- resolution: manual-wrong
- fix: After "wraps" add: "For post-increment and pre-decrement forms the address-register auto-update is checked separately, and a wrap of either the effective address or the updated low word raises the fault (Section~\ref{sec:arithmetic-and-address-wraparound})." Add the same clause to the LOAD/STOR Exceptions paragraphs and to the Ch15 Exceptions row, the paragraph at 107-110 and the LDEA/LDEL Exceptions paragraphs.
- confidence: high

### F-mem-5
- file: reference/chapters/14-memory-instructions.tex
- lines: 104-117
- severity: major
- category: fact
- claim: "Memory instructions can raise data bus and data bus-protection faults during the memory-access phase. ... effective-address calculation can raise a segment-overflow fault"
- evidence: docs/reference/chapters/06-exceptions.tex:246-256 and 04-data-representation.tex:157-160: with SR.A set, any instruction in the last slot of a segment that uses the advanced PC faults. A LOAD or STOR is such an instruction unless the LOAD writes pl. Ch15:110-113 states this for control flow, but Ch14 never does.
- resolution: manual-wrong
- fix: Add after the segment-overflow sentence: "With \texttt{SR.A} set, a memory instruction in the last instruction slot of a segment also raises a segment-overflow fault, in phase 3 and before its memory access, because it uses the wrapped next-instruction address; a \mnemonic{LOAD} whose destination is \reg{pl} is the exception (Section~\ref{sec:segment-overflow-fault})."
- confidence: high

### F-mem-6
- file: reference/chapters/14-memory-instructions.tex
- lines: 94, 121-122
- severity: major
- category: contradiction
- claim: "Memory instructions do not update status flags. All four condition flags are preserved regardless of addressing form, shift result, or auto-update side effect."
- evidence: 14-memory-instructions.tex:111 and 269-270 say "LOAD into sr updates only bits 7--0" in protected mode, and bits 3-0 are the flags; write_back.rs:300-307 -> set_register_value (write_back.rs:42-59) writes sr directly.
- resolution: manual-wrong
- fix: Replace with "Memory instructions do not update status flags as a side effect. A \mnemonic{LOAD} whose destination is \reg{sr} writes the loaded value to the status register (bits 7--0 only in protected mode; every bit except SR.E in supervisor mode), which replaces the flags." Qualify table row 94 and Table tab:memory-status-flag-effects in the same way.
- confidence: high

### F-mem-7
- file: reference/chapters/14-memory-instructions.tex
- lines: 227-228
- severity: major
- category: fact
- claim: "\textbf{Operands:} \texttt{rD} is the destination general-purpose register; a privileged high address register (\reg{ah}, \reg{lh}, \reg{ph}, or \reg{sh}) raises a privilege-violation fault in protected mode."
- evidence: The rD field is 4 bits and names any of the 16 registers (facts.md:28-35). 06-exceptions.tex:250,256 relies on "a LOAD instruction that writes pl" acting as a jump. write_back.rs:300-307 writes pl after decode has advanced it, so the loaded value wins. Ch14 never describes LOAD into pl or ph, or the odd-pl alignment consequence.
- resolution: manual-wrong
- fix: Replace "general-purpose register" with "any register, including \reg{sr}, the address-register halves and \reg{pl}". Add: "A \mnemonic{LOAD} into \reg{pl} is a jump within the current segment: the loaded value replaces the advanced program counter, and an odd value raises an alignment fault at the next fetch. A supervisor-mode \mnemonic{LOAD} into \reg{ph} changes the segment of the next fetch."
- confidence: medium

### F-mem-8
- file: reference/chapters/14-memory-instructions.tex
- lines: 21-22, 90
- severity: minor
- category: contradiction
- claim: "Condition false     & The instruction is skipped and has no side effects. Registers, memory, address registers, and status flags are preserved."
- evidence: 15-control-flow.tex:110-112 and 06-exceptions.tex:246-249: an instruction whose condition is false still raises a segment-overflow fault in the last slot of a segment when SR.A is set. The same absolute wording appears at 15-control-flow.tex:21-22 and :91.
- resolution: manual-wrong
- fix: Append "(the program-counter advance is still checked for segment overflow; see Section~\ref{sec:segment-overflow-fault})" at all three locations.
- confidence: high

### F-mem-9
- file: reference/chapters/15-control-flow.tex
- lines: 394-404, 432-443
- severity: minor
- category: fact
- claim: "\bitbox{4}{0x0} & \bitbox{4}{rO} & \bitbox{1}{0} & \bitbox{3}{000} & \bitbox{4}{0x0}"
- evidence: 14-memory-instructions.tex:205-207 gives LOAD's register-form R2/SO/ST/SA a rule ("architecturally undefined", expected to be ignored). The LDEA/LDEL/LJMP/LJSR register forms (here, :521-560, :654-668, :750-764) and STOR's R1 field (14-memory-instructions.tex:352, "Unused, encoded as 0") draw fixed zeros but state no rule for a non-zero value. The assembler always writes 0 (ldea.rs:121-125; store.rs:119).
- resolution: manual-wrong
- fix: Add to each register-form field list: "R2 (bits 21--18) and the shift fields (bits 13--6) are reserved and written as zero; a non-zero value is architecturally undefined." For STOR R1, use "reserved, written as zero; a non-zero value is architecturally undefined."
- confidence: medium

### F-mem-10
- file: reference/chapters/14-memory-instructions.tex
- lines: 509
- severity: minor
- category: contradiction
- claim: "\item \textbf{ASL} -- Arithmetic Shift Left (same data result as LSL; sets V on signed overflow)"
- evidence: 14-memory-instructions.tex:94 and :564-566: a shift performed during a STOR cannot update the status register. The STOR AF field selects the address register (:364-365).
- resolution: manual-wrong
- fix: "\item \textbf{ASL} -- Arithmetic Shift Left (same stored value as LSL; no flags are updated)"
- confidence: high

### F-mem-11
- file: reference/chapters/14-memory-instructions.tex
- lines: 6-7
- severity: minor
- category: fact
- claim: "Following the load/store architecture, these are the only instructions that can access memory"
- evidence: facts.md:407-412: DMAR/DMAW/DMAT coprocessor commands transfer words to and from memory (16-coprocessor-instructions.tex; appendix-b-timing.tex:176-181 counts DMA accesses).
- resolution: manual-wrong
- fix: "these are the only processing-unit instructions that access data memory; the optional DMA coprocessor commands (Chapter~\ref{ch:coprocessor-instructions}) also transfer memory"
- confidence: medium

### F-mem-12
- file: reference/chapters/14-memory-instructions.tex
- lines: 95
- severity: minor
- category: fact
- claim: "Memory instructions complete in 6 cycles when the bus access is acknowledged without wait states. Bus wait states extend the memory-access phase."
- evidence: appendix-b-timing.tex:178-181: a load or store makes two instruction-fetch accesses plus one data access, so w wait states add 3w. Ch15:97 says "plus any instruction-fetch wait states", but Ch14 mentions only the data access. The same wording is at :264-265 and :421-422.
- resolution: manual-wrong
- fix: "6 cycles with no wait states. Each wait state on either instruction fetch or on the data access adds one cycle (3w for w wait states per access)."
- confidence: high

### F-mem-13
- file: reference/chapters/15-control-flow.tex
- lines: 41-44
- severity: minor
- category: fact
- claim: "BRAN                 & \texttt{BRAN \#disp} ... BRAN                 & \texttt{BRAN @label}"
- evidence: sirc-vm/toolchain/src/parsers/opcodes/branching.rs:85-117 also accepts `BRAN $placeholder` / `BRSR $placeholder` (an .EQU value).
- resolution: manual-wrong
- fix: Add rows "BRAN \$name" and "BRSR \$name": "displacement from an .EQU placeholder". Alternatively, state that \#disp may be written as a placeholder.
- confidence: high

### F-mem-14
- file: reference/chapters/15-control-flow.tex
- lines: 285-286
- severity: minor
- category: improvement
- claim: "Protected-mode calls must preserve the high words written to \reg{l} and \reg{p}."
- evidence: write_back.rs:226-245 writes lh = ph of the next instruction. write_back.rs:69-86 faults if any written high word differs, so in protected mode BRSR/LDEL/LJSR fault unless lh already equals ph.
- resolution: n/a
- fix: Add: "Because the link write sets lh to the current ph, a protected-mode call faults unless lh already equals ph; protected-mode code must keep l in the program segment." Repeat for LDEL (618-619) and LJSR (824-825).
- confidence: high

### F-mem-15
- file: reference/chapters/15-control-flow.tex
- lines: 674-675
- severity: nit
- category: fact
- claim: "\reg{p} must not be used with an auto-update form."
- evidence: LJMP has no auto-update form (15-control-flow.tex:46-48; ljmp.rs:75-121).
- resolution: manual-wrong
- fix: Delete the sentence from the LJMP src field.
- confidence: high
