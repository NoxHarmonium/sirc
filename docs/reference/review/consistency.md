# Full-manual consistency review (reviewer tag: `con`)

Scope: internal agreement across `docs/reference/chapters/*.tex`, `docs/reference/generated/*.tex`
and `main.tex`. No claim was checked against the implementation; every `evidence` entry points at
another location in the manual or at `docs/reference/STYLE.md`.

---

### F-con-1
- file: chapters/07-instruction-formats.tex
- lines: 277-280
- severity: blocker
- category: contradiction
- claim: "1100          & <                 & Signed Less Than (N $\neq$ V)              \\ ... 1110          & <<                & Signed Less or Equal (Z = 1 OR N $\neq$ V)"
- evidence: chapters/10-condition-codes.tex:33-35 gives `1100 = <<` (Signed Less Than) and `1110 = <=` (Signed Less or Equal); every use site in the manual follows Chapter 10, not Chapter 7 (chapters/10-condition-codes.tex:154 `BRAN|<<`, chapters/13-alu-instructions.tex:846 `BRAN|<<`, chapters/15-control-flow.tex:525,540 `BRAN|<=`, chapters/appendix-b-timing.tex:136 `BRAN|<=`). The mnemonic `<` is never used anywhere else in the manual (0 hits).
- resolution: manual-wrong
- fix: In `tab:condition-code-encoding` (07-instruction-formats.tex) change the `1100` row mnemonic from `<` to `<<` and the `1110` row mnemonic from `<<` to `<=`, matching `tab:condition-codes` in Chapter 10. Also reconcile row `0000`: Chapter 7 says `(none)`, Chapter 10 says `AL (or none)` — pick `AL (or none)`. Note that appendix-e-quick-reference.tex:185 sends readers to the Chapter 7 table as the authoritative one, so the two tables must either be merged or Chapter 7's must be corrected.
- confidence: high

### F-con-2
- file: chapters/13-alu-instructions.tex
- lines: 857-858
- severity: blocker
- category: contradiction
- claim: "For signed comparisons, use \texttt{>=}, \texttt{<}, \texttt{>>}, \texttt{<<} \\ For unsigned comparisons, use \texttt{>}, \texttt{CS}, \texttt{CC}, \texttt{<=}"
- evidence: chapters/10-condition-codes.tex:30-35 and chapters/10-condition-codes.tex:263-270 — the unsigned conditions are `HI`, `LO`, `CS`, `CC`; `<=` and `>>` are *signed* conditions; `>` is not a condition mnemonic in any table.
- resolution: manual-wrong
- fix: Replace both lines with "For signed comparisons, use \texttt{>=}, \texttt{<<}, \texttt{>>}, \texttt{<=}" and "For unsigned comparisons, use \texttt{HI}, \texttt{LO}, \texttt{CS}, \texttt{CC}".
- confidence: high

### F-con-3
- file: chapters/06-exceptions.tex
- lines: 259-268
- severity: blocker
- category: contradiction
- claim: "\bitbox{3}{BAT} & \bitbox{1}{D} & \bitbox{4}{Current Fault} & \bitbox{4}{Original Fault} & \bitbox{4}{Reserved}"
- evidence: The prose immediately below (chapters/06-exceptions.tex:271-286) says "Bits 0-2: Bus access type", "Bit 3: Double fault flag", "Bits 4-7: Current fault type", "Bits 8-11: Original fault type", "Bits 12-15: Unused". `bytefield` draws the highest-numbered bit on the left, so the figure as written places BAT in bits 15--13 and Reserved in bits 3--0 — exactly reversed. docs/reference/STYLE.md:50 ("MSB is the highest-numbered bit and is drawn on the left in every `bytefield` diagram — do not reverse this") and the correctly-ordered `fig:sr-layout` at chapters/05-status-register.tex:20-35 confirm the convention.
- resolution: manual-wrong
- fix: Reverse the box order in the figure to `\bitbox{4}{Reserved} & \bitbox{4}{Original Fault} & \bitbox{4}{Current Fault} & \bitbox{1}{D} & \bitbox{3}{BAT}`, keeping `\bitheader{0,2,3,4,7,8,11,12,15}`.
- confidence: high

### F-con-4
- file: chapters/14-memory-instructions.tex
- lines: 325-326
- severity: blocker
- category: contradiction
- claim: "When loading from memory, the data read from memory is shifted \textbf{after} being loaded but \textbf{before} being written to the destination register"
- evidence: chapters/09-shift-operations.tex:12-13 "Shift operations are only applied to source operands. A shift cannot be applied to the result of an operation before register writeback."; chapters/09-shift-operations.tex:43-44 "It is applied in the \"Decode and Register Fetch\" phase of instruction execution, before any ALU operations or memory address calculation occurs."; chapters/02-cpu-architecture.tex:136-137 "The shift is applied to the first source operand when fetching the source registers for an instruction. Shifts cannot be applied to the result or any other source operand." Also chapters/14-memory-instructions.tex:23, :53, :55, :144, :151, :156. The generated encodings confirm the conflict: `LOAD r1, (r2, a)` encodes R2=0x0 (generated/register-format-encodings.tex:14), so there is no first source operand for the Chapter 9 rule to shift, while `STOR -(r2, s), r1` encodes R2=0x1 (the stored source) and is consistent with Chapter 9.
- resolution: unclear
- fix: Decide whether the register-offset LOAD shift is a load-result shift (then Chapter 9 line 12-13, Chapter 9 line 43-44 and Chapter 2 line 136-137 must be amended to carve out memory loads and to state that the load-result shift happens in the Memory Access / Write Back phase), or a source-operand shift (then Chapter 14 lines 23, 53, 55, 144, 151, 156, 325-326 must be rewritten). Whichever way it resolves, one sentence must state the phase in which the memory shift is applied.
- confidence: high

### F-con-5
- file: chapters/09-shift-operations.tex
- lines: 328-330
- severity: blocker
- category: contradiction
- claim: "For \textbf{ASL}: Set if the sign bit changes during the shift (signed overflow) \\ For other shifts: Cleared to 0"
- evidence: chapters/09-shift-operations.tex:30 "011 & ASL & Arithmetic Shift Left & Same as LSL (shifts in zeros)" and chapters/09-shift-operations.tex:132 "\textbf{Operation:} Identical to LSL. Shift bits to the left, filling with zeros." If ASL is identical to LSL it cannot have a different V effect. chapters/14-memory-instructions.tex:351 repeats "ASL -- Arithmetic Shift Left (same as LSL)".
- resolution: manual-wrong
- fix: ASL and LSL differ only in flag effect. Change chapters/09-shift-operations.tex:30 to "Same data result as LSL; sets V on signed overflow" and chapters/09-shift-operations.tex:132 to "Produces the same data result as LSL, but sets the overflow flag if the sign bit changes." Apply the same correction at chapters/14-memory-instructions.tex:351.
- confidence: high

### F-con-6
- file: chapters/appendix-c-undocumented.tex
- lines: 109
- severity: blocker
- category: contradiction
- claim: "result = operand1 - operand2 - (1 - C)"
- evidence: chapters/13-alu-instructions.tex:461 gives the documented SBC operation as "result = operand1 - operand2 - SR.C", and chapters/13-alu-instructions.tex:457 says "The incoming \texttt{C} flag is read as the borrow input." Appendix C uses the opposite (inverted-carry) borrow convention.
- resolution: manual-wrong
- fix: Change appendix-c-undocumented.tex:109 to "result = operand1 - operand2 - C" to match the SBC borrow convention defined in Chapter 13.
- confidence: high

### F-con-7
- file: chapters/appendix-e-quick-reference.tex
- lines: 25
- severity: blocker
- category: contradiction
- claim: "\mnemonic{ADCI}   & ALU           & 0x01/0x21/0x31       & NZCV           & Add with carry (immediate, short, or register)"
- evidence: `ADCR` is the register-form mnemonic for opcode 0x31 (chapters/12-instruction-summary.tex:77, chapters/appendix-a-opcode-map.tex:75, chapters/13-alu-instructions.tex:62 "\texttt{ADCR rD, rS1, rS2[, shift]} & 0x31"). Appendix E introduces itself at line 12 as "All instruction mnemonics in alphabetical order", and every other family is split (ADDI/ADDR, ANDI/ANDR, SUBI/SUBR, SBCI/SBCR, ORRI/ORRR, XORI/XORR, CMPI/CMPR, TSAI/TSAR, TSXI/TSXR). `ADCR` is the only mnemonic missing from the index.
- resolution: manual-wrong
- fix: Change the ADCI row opcodes to `0x01/0x21` and insert a new row after it: `\mnemonic{ADCR}   & ALU           & 0x31                 & NZCV           & Add with carry register`.
- confidence: high

### F-con-8
- file: chapters/14-memory-instructions.tex
- lines: 132-139
- severity: blocker
- category: contradiction
- claim: "LOAD rD, (rS, addr)           ; rD = memory[addr + rS]" and (lines 213-220) "STOR (rD, addr), rS           ; memory[addr + rD] = rS"
- evidence: The displacement register is named `rO` in every table: chapters/08-addressing-modes.tex:41 "\texttt{(rO, addr)}", chapters/08-addressing-modes.tex:62-63, chapters/14-memory-instructions.tex:53,55,57,59 ("\texttt{LOAD rD, (rO, addr)[, shift]}"). `rD` is defined at chapters/11-reading-instructions.tex:35 as "Destination general-purpose register" and `rS` at :36 as "Source general-purpose register", so `STOR (rD, addr), rS` names the *displacement* register with the *destination* placeholder while the actual source is `rS`. Chapter 15 has the same drift: chapters/15-control-flow.tex:34,36,38,40,48,51,53,292,295,300,354,357,362,418,458,461,465 all use `rS` for the displacement register, against chapters/08-addressing-modes.tex:66 which uses `rO`.
- resolution: manual-wrong
- fix: Standardize on `rO` (displacement/offset register) in every syntax line and Operands paragraph in Chapters 14 and 15, and add `\texttt{rO}` to the notation table `tab:instruction-description-notation` (chapters/11-reading-instructions.tex:29-47) with the meaning "Register supplying a memory or effective-address displacement". Affected: 14-memory-instructions.tex:132,135,138,139,144,213,216,219,220,225; 15-control-flow.tex:34,36,38,40,48,51,53,292,295,300,354,357,362,418,458,461,465.
- confidence: high

### F-con-9
- file: chapters/07-instruction-formats.tex
- lines: 232-238
- severity: major
- category: contradiction
- claim: "Note: If R3 is not specified, the assembler with use R1 as both destination and first source operand when encoding the instruction: ... ADDR r1, r2      ; r1 = r1 + r2 (R3 implicitly = R1)"
- evidence: The same chapter's own note says the opposite field defaults: chapters/07-instruction-formats.tex:246 "When R3 is not specified in assembly, it defaults to R1 (e.g., \texttt{ADDR r4, r5} uses R1=0x4, R2=0x4, R3=0x5)" — that example shows **R2** taking R1's value and the second operand landing in R3. generated/register-format-encodings.tex:38 confirms: `CMPR r4, r5` encodes R1=0x4, R2=0x4, R3=0x5. chapters/13-alu-instructions.tex:95-97 states it correctly: "When a register-form ALU instruction omits \texttt{rS1}, the assembler encodes \texttt{rD} as both the destination and first source operand. For example, \texttt{ADDR r1, r2} is encoded as \texttt{ADDR r1, r1, r2}."
- resolution: manual-wrong
- fix: At line 232 change "If R3 is not specified" to "If only two register operands are given"; at line 237 change the comment to "; r1 = r1 + r2 (R2 implicitly = R1)"; at line 246 change "When R3 is not specified in assembly, it defaults to R1" to "When only two register operands are given, R2 is encoded as a copy of R1 and the second operand is encoded in R3". Also fix the typo "the assembler with use" -> "the assembler will use".
- confidence: high

### F-con-10
- file: chapters/12-instruction-summary.tex
- lines: 6-7
- severity: major
- category: contradiction
- claim: "The SIRCIS instruction set consists of 64 possible opcodes (6-bit opcode field), of which approximately 52 are documented and assigned."
- evidence: chapters/appendix-c-undocumented.tex:6 "The SIRCIS instruction set includes 14 undocumented instruction opcodes" and appendix-c-undocumented.tex:211 "14 undocumented opcodes"; its table (appendix-c-undocumented.tex:46-59) lists 14 rows. 64 - 14 = 50, not 52. Separately, chapters/appendix-a-opcode-map.tex:134-138 lists only **12** undocumented opcodes (0x08, 0x09, 0x0B, 0x0D, 0x28, 0x29, 0x2B, 0x2D, 0x38, 0x39, 0x3B, 0x3D), omitting 0x27 and 0x2F even though its own table marks both "Undocumented" (appendix-a-opcode-map.tex:62, :70) and its quick-lookup footnote (appendix-a-opcode-map.tex:169) calls them undocumented.
- resolution: manual-wrong
- fix: Change 12-instruction-summary.tex:6-7 to "of which 50 are documented and assigned". Add a fourth bullet to appendix-a-opcode-map.tex:134-138: "\item 0x27, 0x2F (no public assembly syntax)" so the appendix agrees with the 14 listed in Appendix C.
- confidence: high

### F-con-11
- file: chapters/05-status-register.tex
- lines: 45-46
- severity: major
- category: contradiction
- claim: "These flags reflect the results of ALU operations and comparisons. They can be read and written by code running in any privilege level."
- evidence: Contradicted three times in the same chapter and once in Chapter 3: chapters/05-status-register.tex:10 "Condition flags -- readable and writable in both supervisor and protected modes"; chapters/05-status-register.tex:90-91 "These flags control CPU behavior and can only be modified directly in supervisor mode. Attempts to write \reg{sr} in protected mode raise a privilege violation fault."; chapters/05-status-register.tex:207-209 "When running in protected mode, any direct write to the status register raises a privilege violation fault ... they cannot directly load or store \reg{sr}."; chapters/03-registers.tex:192-193 "Direct writes to \reg{sr} raise a privilege violation fault."
- resolution: manual-wrong
- fix: Change line 10 to "Condition flags -- readable in both supervisor and protected modes; modifiable in protected mode only as a side effect of ALU and shift instruction execution" and lines 45-46 to "They can be read at any privilege level and are updated by ordinary instruction execution at any privilege level, but a direct write to \reg{sr} is a privileged operation (see Section \"Manual Updates\")."
- confidence: high

### F-con-12
- file: chapters/14-memory-instructions.tex
- lines: 31-41
- severity: major
- category: contradiction
- claim: "\text{EA} = \text{AddressRegister} + \text{Offset} ... The resulting 24-bit address is used to access memory."
- evidence: The same chapter's own table says the addition is low-word-only: chapters/14-memory-instructions.tex:80 "Immediate-offset forms add the 16-bit encoded offset to the selected address register low word ... The selected address register high word supplies the segment." chapters/08-addressing-modes.tex:201-203 gives the correct form "EA = AddressRegister.high : (AddressRegister.low + Offset)"; chapters/04-data-representation.tex:153-155 "Address calculation ... adds offsets to the low word of the selected address register pair. The high address-register word is not changed"; chapters/08-addressing-modes.tex:120-122.
- resolution: manual-wrong
- fix: Replace the equation with `\text{EA} = \text{AddressRegister.high} : (\text{AddressRegister.low} + \text{Displacement})` and change "The resulting 24-bit address is used to access memory." to "The high word supplies the segment and is not changed by the displacement; only the low word participates in the addition."
- confidence: high

### F-con-13
- file: chapters/06-exceptions.tex
- lines: 442-443
- severity: major
- category: contradiction
- claim: "The vector-table address is computed by multiplying the vector ID by 2 (since vectors are 32-bit addresses stored as two 16-bit words), then adding the system RAM offset."
- evidence: `system_ram_offset` / "system RAM offset" appears only at chapters/06-exceptions.tex:443, :615 and :616 and is never defined anywhere in the manual (grep across chapters/ and generated/ returns only those three hits). Every other statement of the vector address is a flat V*2 with no offset: chapters/01-introduction.tex:157 "The CPU expects the exception vector table to begin at word address \addr{0x000000}"; chapters/04-data-representation.tex:98 "Vector entry \texttt{V} starts at word address \texttt{V * 2}"; chapters/06-exceptions.tex:506 "The exception vectors are stored starting at address 0x0"; chapters/06-exceptions.tex:508 "The high word is stored first at \texttt{vector * 2}".
- resolution: unclear
- fix: Either define `system_ram_offset` (a normative subsection in Chapter 6 stating what it is, its reset value, and how software sets it — and then correct Chapter 1 and Chapter 4 to include it), or delete it from 06-exceptions.tex:443, :615 and :616 and write "\texttt{0x000000}" / "\texttt{0x000001}". Do not leave it undefined.
- confidence: high

### F-con-14
- file: chapters/appendix-e-quick-reference.tex
- lines: 13-14
- severity: major
- category: contradiction
- claim: "\emph{Flags}: flags written by the instruction (\texttt{--} = none; \texttt{*} = determined by the coprocessor)."
- evidence: chapters/11-reading-instructions.tex:60-64 defines the manual-wide symbol set as `*` = "Updated from the instruction result", `-` = "Preserved", `S` = "Updated from the shifter result when \texttt{[S]} is used". Appendix E's own cross-reference table sends readers to Chapter 11 for these symbols (appendix-e-quick-reference.tex:192). Appendix E then uses `*` for SHFT (appendix-e-quick-reference.tex:79), which is not a coprocessor instruction at all — its flags come from the shifter, i.e. symbol `S`.
- resolution: manual-wrong
- fix: Change the legend to "\emph{Flags}: uses the symbols defined in Table~\ref{tab:flag-effect-symbols} (Chapter~\ref{ch:reading-instructions}); \texttt{--} means all four flags preserved and \texttt{COP} means the effect is defined by the selected coprocessor." Replace the `*` entries at lines 38, 39, 42, 43, 61, 62 with `COP`, and change the SHFT row (line 79) from `*` to `S`.
- confidence: high

### F-con-15
- file: chapters/12-instruction-summary.tex
- lines: 29-31
- severity: major
- category: contradiction
- claim: "0x04            & ANDI              & Immediate       & NZ             & AND immediate"
- evidence: The logical instructions also *clear* C and V, which the "NZ" column hides: chapters/13-alu-instructions.tex:150-154 "AND & * & * & 0 & 0"; chapters/13-alu-instructions.tex:135 "Logical (AND, ORR, XOR, TSA, TSX)] Update N and Z; clear C and V"; chapters/05-status-register.tex:180 "Logical Instructions (ANDI, ORRI, XORI): Update Z and N flags; C and V flags are cleared." The same understatement is repeated in chapters/appendix-e-quick-reference.tex:28,29,68,69,85,86,87,88,94,95.
- resolution: manual-wrong
- fix: Change the Flags cell for every logical/test-logical row to `NZ (C,V=0)` in 12-instruction-summary.tex:29,30,31,37,39,63,64,65,71,73,80,81,82,88,90 and in appendix-e-quick-reference.tex:28,29,68,69,85,86,87,88,94,95, and add a legend line under each table pointing at `tab:flag-effect-symbols`.
- confidence: high

### F-con-16
- file: chapters/13-alu-instructions.tex
- lines: 155
- severity: major
- category: contradiction
- claim: "LOAD            & -          & -          & -          & -          & \texttt{[N]}        & Status flags are preserved."
- evidence: chapters/11-reading-instructions.tex:97 "When no override is written, ALU instructions use \texttt{[A]}." LOAD is an ALU-family instruction (chapters/13-alu-instructions.tex:33-36 lists it under "Data Movement"; opcodes 0x07/0x37 sit inside the ALU opcode ranges named at chapters/12-instruction-summary.tex:10).
- resolution: manual-wrong
- fix: Amend chapters/11-reading-instructions.tex:97 to "When no override is written, ALU instructions use \texttt{[A]}, except \mnemonic{LOAD}, which always encodes \texttt{[N]} and never updates the status flags." (Chapter 13 line 166 already states the LOAD exception; Chapter 11 must not contradict it.)
- confidence: high

### F-con-17
- file: chapters/09-shift-operations.tex
- lines: 524
- severity: major
- category: contradiction
- claim: "LOAD[N] r4, (#0, a)+          ; Load next value (flags unchanged)"
- evidence: chapters/14-memory-instructions.tex:104-105 "Memory instructions do not support status update override syntax; the status override field is ignored."; chapters/11-reading-instructions.tex:98 "Instructions that do not support status updates must not use status update override syntax."; chapters/13-alu-instructions.tex:166 "Public \mnemonic{LOAD} forms do not support status update override syntax". The encoding makes it impossible: for memory operations the AF field is the address-register-pair selector, not a status-update selector (chapters/07-instruction-formats.tex:82-88 and :206-212), so `LOAD ..., (#0, a)+` has no spare AF bits.
- resolution: manual-wrong
- fix: Change chapters/09-shift-operations.tex:524 to `LOAD r4, (#0, a)+             ; Load next value (memory loads never touch flags)`. Also correct chapters/14-memory-instructions.tex:104-105, which implies memory instructions *have* a status override field that is "ignored"; they do not — replace with "Memory instructions do not encode a status update source; the AF field selects the address-register pair (see Chapter~\ref{ch:instruction-formats}). Status update override syntax is not accepted on memory instructions."
- confidence: high

### F-con-18
- file: chapters/09-shift-operations.tex
- lines: 27
- severity: major
- category: contradiction
- claim: "000           & NUL (or none)     & None                   & No shift performed"
- evidence: chapters/07-instruction-formats.tex:298 gives the same encoding with mnemonic "--": "000           & None (no shift)        & --". The two tables (`tab:shift-types` at 07-instruction-formats.tex:309 and `tab:shift-encoding` at 09-shift-operations.tex:38) are duplicates of the same normative encoding that have drifted; the mnemonic `NUL` occurs nowhere else in the manual, and `\ref{tab:shift-types}` is what both instruction-format sections cite (07-instruction-formats.tex:145, :204) while appendix-e-quick-reference.tex:186 also points at `tab:shift-types`.
- resolution: manual-wrong
- fix: Delete `tab:shift-encoding` from Chapter 9 and replace it with a reference to `Table~\ref{tab:shift-types}` plus the Description column merged into Chapter 7's table; or, if Chapter 9 keeps its table, make row 000 read "-- (no shift)" in both and decide whether `NUL` is a real assembler mnemonic. Do not ship two encoding tables that disagree.
- confidence: high

### F-con-19
- file: chapters/10-condition-codes.tex
- lines: 245
- severity: major
- category: contradiction
- claim: "ADDI r1, #0, #10         ; i = 10"
- evidence: No `ADDI` form takes two immediates. chapters/13-alu-instructions.tex:57-58 and :253-254 give the only immediate forms: `ADDI rD, #imm16` and `ADDI rD, #imm8, shift`; chapters/08-addressing-modes.tex:59 confirms. The idiom used everywhere else to load a constant is `LOAD` (chapters/13-alu-instructions.tex:721, chapters/09-shift-operations.tex:377). The same broken line appears at chapters/appendix-b-timing.tex:74 "ADDI r7, #0, #10              ; 6 cycles".
- resolution: manual-wrong
- fix: Change chapters/10-condition-codes.tex:245 to `LOAD r1, #10             ; i = 10` and chapters/appendix-b-timing.tex:74 to `LOAD r7, #10                  ; 6 cycles`.
- confidence: high

### F-con-20
- file: chapters/16-coprocessor-instructions.tex
- lines: 366-368
- severity: major
- category: contradiction
- claim: "bits 7--6 select the address register (\texttt{00} = \reg{a}, \texttt{01} = \reg{l}, \texttt{10} = \reg{s}, \texttt{11} reserved)"
- evidence: The architectural 2-bit address-register-pair encoding is `00 = l, 01 = a, 10 = s, 11 = p` — chapters/08-addressing-modes.tex:345-348 (`tab:addr-reg-encoding`), chapters/07-instruction-formats.tex:84-87 and :208-211. The DMA operand byte swaps `l` and `a` with no note that it is a different field with a different assignment, which is an easy trap for an implementer.
- resolution: unclear
- fix: If the DMA operand really uses a different assignment, add an explicit warning sentence after line 368: "Note: this DMA operand field is not the AF address-register field; its \texttt{00}/\texttt{01} assignment differs from Table~\ref{tab:addr-reg-encoding}." If it does not, change the encoding to `00 = l, 01 = a, 10 = s, 11 = reserved`.
- confidence: medium

### F-con-21
- file: chapters/appendix-a-opcode-map.tex
- lines: 126
- severity: major
- category: contradiction
- claim: "\item Bit 1: Determines if both registers in address pair are updated"
- evidence: No auto-update ever writes both registers of a pair. chapters/08-addressing-modes.tex:124-125 "Post-increment and pre-decrement change the selected address register by exactly one word"; chapters/08-addressing-modes.tex:121-122 "The high word supplies the segment and is preserved"; chapters/04-data-representation.tex:153-155; chapters/14-memory-instructions.tex:81-82 ("The address register low word is incremented by one word"). Bit 1 in fact selects the auto-update form (0x10/0x11 vs 0x12/0x13; 0x14/0x15 vs 0x16/0x17).
- resolution: manual-wrong
- fix: Replace with "\item Bit 1: Auto-update form (0 = no address-register update, 1 = pre-decrement for STOR/LDEA or post-increment for LOAD/LDEL; only the low word of the pair changes)".
- confidence: high

### F-con-22
- file: chapters/14-memory-instructions.tex
- lines: 15-21
- severity: major
- category: contradiction
- claim: "Both instructions support four memory addressing forms: ... Post-increment load: \texttt{(\#offset, addr)+} or \texttt{(reg, addr)+} ... Pre-decrement store: \texttt{-(\#offset, addr)} or \texttt{-(reg, addr)}"
- evidence: The same chapter says the opposite 50 lines later: chapters/14-memory-instructions.tex:68-69 "\mnemonic{LOAD} has no pre-decrement form and \mnemonic{STOR} has no post-increment form."; the legal-forms table (chapters/14-memory-instructions.tex:52-59) lists four forms for LOAD and four for STOR, not the same four; chapters/08-addressing-modes.tex:95-97 states the same asymmetry.
- resolution: manual-wrong
- fix: Change the lead-in to "\mnemonic{LOAD} and \mnemonic{STOR} each support four memory addressing forms. Both support indirect immediate-displacement and indirect register-displacement forms; \mnemonic{LOAD} additionally supports post-increment and \mnemonic{STOR} additionally supports pre-decrement." and re-label the last two bullets accordingly.
- confidence: high

### F-con-23
- file: chapters/14-memory-instructions.tex
- lines: 95
- severity: major
- category: contradiction
- claim: "Documented memory forms do not raise privilege-violation or invalid-opcode faults."
- evidence: chapters/06-exceptions.tex:96 lists "Address-register-pair write-back that would change a high address-register word" as a privilege-violation trigger; chapters/08-addressing-modes.tex:412-413 "\textbf{Address-register-pair write-back}: Allowed only when the high word would remain unchanged; otherwise triggers a privilege violation fault \\ \textbf{Post-increment/Pre-decrement}: Allowed in protected mode only when the high word would remain unchanged"; chapters/03-registers.tex:209-211. Post-increment LOAD and pre-decrement STOR are address-register-pair write-backs. Chapter 15 states the rule explicitly for its instructions (chapters/15-control-flow.tex:94); Chapter 14 denies it.
- resolution: unclear
- fix: Either add the privilege rule to Chapter 14 (a `Privilege` row in `tab:memory-common-semantics` mirroring chapters/15-control-flow.tex:94, and an amended line 95), or state in one place why auto-update on memory instructions can never change the high word and therefore never faults. The current flat denial cannot stand alongside Chapter 6, 8 and 3.
- confidence: medium

### F-con-24
- file: chapters/06-exceptions.tex
- lines: 631-637
- severity: major
- category: contradiction
- claim: "Reads the link register corresponding to the current interrupt mask level (minus 1, since the mask is set to the exception level) \\ Restores the status register from the link register (including the interrupt mask and protected mode bit)"
- evidence: The same chapter describes RETE differently 130 lines earlier: chapters/06-exceptions.tex:494-499 "The return address is loaded from the link register into the program counter \\ The saved status register value is restored (including interrupt enable bits) \\ The saved exception level is restored to current\_exception\_level". The term "interrupt mask" / "interrupt mask level" appears nowhere else in the manual; the status register has four *interrupt enable* bits E1--E4 (chapters/05-status-register.tex:107-116) and the exception unit tracks `current_exception_level` (chapters/06-exceptions.tex:377). The second version also omits restoring the exception level.
- resolution: manual-wrong
- fix: Delete the duplicate enumeration at chapters/06-exceptions.tex:631-637 and replace it with a cross-reference to the "Returning from Exceptions" list at chapters/06-exceptions.tex:494-499, or rewrite it using `current_exception_level` and "interrupt enable bits". Do not use "interrupt mask level" anywhere.
- confidence: high

### F-con-25
- file: chapters/06-exceptions.tex
- lines: 163-168
- severity: major
- category: contradiction
- claim: "Each link register stores two pieces of information: \\ \textbf{Return Address:} The 32-bit address to return to ... \\ \textbf{Return Status Register:} The 16-bit status register value at the time of the exception"
- evidence: The figure immediately following stores three fields (chapters/06-exceptions.tex:175 "\bitbox{16}{Return Status Register} & \bitbox{8}{Saved Level} & \bitbox{8}{Reserved}") and the exception-entry flow writes three (chapters/06-exceptions.tex:435-436 "The current program counter, status register, and current exception level are stored in the appropriate link register"; chapters/06-exceptions.tex:466-467 "Writes the selected link register with the current program counter, current status register, and current exception level"; chapters/06-exceptions.tex:497 "The saved exception level is restored").
- resolution: manual-wrong
- fix: Change "two pieces of information" to "three pieces of information" and add a third bullet: "\item \textbf{Saved Exception Level:} The 8-bit \texttt{current\_exception\_level} value at the time of the exception".
- confidence: high

### F-con-26
- file: chapters/09-shift-operations.tex
- lines: 265-267
- severity: major
- category: contradiction
- claim: "; Add (r1 * 4) + 1 to r1 \\ ADDI r1, #1, LSL #2  ; r1 = r1 + (r2 << 2) + 1 \\                           ; SO=0: r2 is shifted by literal 2"
- evidence: chapters/09-shift-operations.tex:60 (`tab:source-operand-examples`) states that `ADDI r1, #2, LSL #1` is interpreted as `ADDI r1, r1, #2, LSL #1` with first source operand **r1**; chapters/09-shift-operations.tex:46-48 "If using the two operand form of instruction, the second operand is inferred to be the same as the destination"; chapters/13-alu-instructions.tex:254 gives the semantics as `ADDI rD, #imm8, shift ; rD = (rD << shift) + imm8`. The example names `r2`, which does not appear in the instruction, and the arithmetic (`r1 + (...) + 1`) double-counts the source.
- resolution: manual-wrong
- fix: Change the comment block to `ADDI r1, #1, LSL #2  ; r1 = (r1 << 2) + 1` / `; SO=0: r1 is shifted by literal 2`, and change the section comment at line 265 to "; Multiply r1 by 4 and add 1".
- confidence: high

### F-con-27
- file: chapters/09-shift-operations.tex
- lines: 95-96
- severity: major
- category: contradiction
- claim: "; Set bit 10 \\ ORRI r3, #1, LSL #10     ; r3 |= (r3 << 10) with immediate 1 = 0x0400"
- evidence: The shift is applied to the register source, never to the immediate: chapters/09-shift-operations.tex:46 "It is applied to the first source operand of an instruction"; chapters/08-addressing-modes.tex:325 "The shift is applied to the source operand, not the constant"; chapters/13-alu-instructions.tex:580 "\texttt{ORRI rD, \#imm8, shift          ; rD = (rD << shift) | imm8}". `ORRI r3, #1, LSL #10` computes `(r3 << 10) | 1`, which does not set bit 10 and is not `0x0400`.
- resolution: manual-wrong
- fix: Replace both lines with `; Set bit 10 (16-bit immediate form)` / `ORRI r3, #0x0400         ; r3 |= 0x0400`.
- confidence: high

### F-con-28
- file: chapters/11-reading-instructions.tex
- lines: 9-25
- severity: major
- category: contradiction
- claim: "\item[Write-back] ... \item[Status Flags] The effect on the condition flags in the status register. \item[Condition Codes] All normal instructions may be conditional."
- evidence: Every actual entry in Chapters 13--17 uses the labels `Flags:` and `Condition codes:` (sentence case), and orders the fields Operation -> Description -> Flags -> Write-back -> Exceptions -> Condition codes -> Timing -> Privilege (chapters/13-alu-instructions.tex:263-292 is representative; also :332-360, :459-486). Chapter 11 lists Write-back before Status Flags, Condition Codes before Exceptions, and omits `Description`, `Example`, and `Notes` entirely. docs/reference/STYLE.md:58-69 rules that the entries win: "Do not use \"Status Flags\" or \"Condition Codes\" (title case) as field labels — use \"Flags\" and \"Condition codes.\""
- resolution: manual-wrong
- fix: Rewrite the Chapter 11 field list in the canonical order and names from STYLE.md:63-65: Opcodes (or Assembles to) -> Syntax -> Operands -> Operation -> Description -> Flags -> Write-back -> Exceptions -> Condition codes -> Timing -> Privilege -> Example/Examples -> Notes. Related but separate drift to fix at the same time: chapters/14-memory-instructions.tex:166-180 puts Write-back before Flags and Timing before Condition codes, and chapters/16-coprocessor-instructions.tex:395-433 puts Description before Operation — bring all boxes into the canonical order.
- confidence: high

### F-con-29
- file: chapters/appendix-b-timing.tex
- lines: 187
- severity: major
- category: contradiction
- claim: "\textbf{At 25 MHz:} $\approx$ 4.17 million instructions/second"
- evidence: chapters/02-cpu-architecture.tex:354 specifies the clock input as "\textbf{Max Rate:} 24 Mhz". A performance figure quoted above the documented maximum clock rate is misleading.
- resolution: manual-wrong
- fix: Change to "\textbf{At 24 MHz (maximum rated clock):} $\approx$ 4.0 million instructions/second". (Also fix the unit spelling at 02-cpu-architecture.tex:354: "Mhz" -> "MHz".)
- confidence: high

### F-con-30
- file: chapters/appendix-c-undocumented.tex
- lines: 213
- severity: major
- category: contradiction
- claim: "\item Reserved address register encoding could enable more address modes"
- evidence: There is no reserved value in the address-register field. chapters/08-addressing-modes.tex:345-348 assigns all four: `00 = l, 01 = a, 10 = s, 11 = p`; chapters/07-instruction-formats.tex:84-87 and :208-211 repeat the full assignment. The only reserved 2-bit value in the architecture is AF `11` when the field is used as a status-update selector (chapters/07-instruction-formats.tex:94, :152, :218; chapters/11-reading-instructions.tex:90).
- resolution: manual-wrong
- fix: Replace with "\item The reserved status-update source encoding (AF = 11) could select a new flag-update source".
- confidence: high

### F-con-31
- file: chapters/appendix-e-quick-reference.tex
- lines: 126
- severity: major
- category: contradiction
- claim: "7--4         & --            & Either          & 0              & Reserved; reads as 0"
- evidence: chapters/05-status-register.tex:84-86 "Bits 4 through 7 of the lower byte are reserved for future use. They should not be relied upon by software." — it does not promise a read value. docs/reference/STYLE.md:127-129 (Gate 1, approved) says "Reserved fields stay advisory. Keep the Chapter 5 wording ... Do *not* adopt the M68000 PRM \"must be written as zeros\" form; the implementation does not promise it." Chapter 6 also lists reserved lower status bits as merely "preserved" on exception entry (chapters/06-exceptions.tex:471).
- resolution: manual-wrong
- fix: Change the Function cell to "Reserved for future use; software must not rely on the value read" and the Reset cell to "--".
- confidence: high

### F-con-32
- file: generated/immediate-format-encodings.tex
- lines: 20-36
- severity: major
- category: contradiction
- claim: "\texttt{LDEA a, (\#4, s)}   & 0x18            & 0x1          & 0x0004                & 0b10        & 0x0           & \texttt{0x60400120}"
- evidence: The Reg field (instruction bits 25--22) is documented as a 4-bit *register identifier* (chapters/07-instruction-formats.tex:80, :138, :196) using the IDs in `tab:register-encoding` (chapters/03-registers.tex:238-253), where `0x1 = r1` and `0x3 = r3`. The generated rows instead put the 2-bit address-register-pair code there (a=0x1, p=0x3): see also generated/immediate-format-encodings.tex:23,26,29,32,35 and generated/register-format-encodings.tex:20,23,26,29,32,35. Nowhere does the manual say that LDEA/LDEL/BRAN/BRSR/LJMP/LJSR encode their destination pair in the Reg/R1 field using the pair code rather than the register ID.
- resolution: manual-wrong
- fix: Add a subsection to Chapter 7 (after `\subsection{Register Operand Semantics}`) stating: "For the effective-address opcodes 0x18--0x1F, the Reg/R1 field holds the *destination address-register pair* code from Table~\ref{tab:addr-reg-encoding} (zero-extended to four bits), not a general-purpose register identifier from Table~\ref{tab:register-encoding}." Reference it from `tab:control-flow-legal-forms` in Chapter 15.
- confidence: high

### F-con-33
- file: generated/register-format-encodings.tex
- lines: 50-51
- severity: minor
- category: contradiction
- claim: "\texttt{SHFT r1, ASL \#3}           & 0x25        & 0x1         & 0x0         & 0x0         & 0           & 011         & 0x3         & 0b10        & 0x0           & \texttt{0x94400CE0}"
- evidence: This row appears in `tab:register-format-examples` (caption "Register Format Encoding Examples", generated/register-format-encodings.tex:54) with R1/R2/R3 column headings, but 0x25 is the *short immediate* opcode (chapters/12-instruction-summary.tex:64, chapters/appendix-a-opcode-map.tex:60) and the hex decodes as a short-immediate word (bits 21--14 are the 8-bit immediate 0x00, not R2/R3). Chapter 7's own note under the register table concedes this: chapters/07-instruction-formats.tex:252 "SHFT is a meta instruction using ORRI opcode (0x25) with AF=0b10". The same instruction is already correctly shown in generated/short-immediate-format-encodings.tex:26.
- resolution: manual-wrong
- fix: Remove the `SHFT r1, ASL #3` row from the register-format table (and note 07-instruction-formats.tex:252, which only exists to explain it); the short-immediate table already carries the `SHFT` example.
- confidence: high

### F-con-34
- file: chapters/02-cpu-architecture.tex
- lines: 133-134
- severity: minor
- category: contradiction
- claim: "The barrel shifters support logical shift left (LSL), logical shift right (LSR), arithmetic shift right (ASR), and rotate operations."
- evidence: Arithmetic Shift Left (ASL, encoding 011) is a documented shift type: chapters/07-instruction-formats.tex:301, chapters/09-shift-operations.tex:30 and §"Arithmetic Shift Left (ASL)" at :130-144, chapters/13-alu-instructions.tex:241 "All shift types are supported: LSL, LSR, ASL, ASR, RTL, and RTR.", chapters/14-memory-instructions.tex:351.
- resolution: manual-wrong
- fix: Change to "logical shift left (LSL), logical shift right (LSR), arithmetic shift left (ASL), arithmetic shift right (ASR), and rotate left/right (RTL, RTR)".
- confidence: high

### F-con-35
- file: chapters/02-cpu-architecture.tex
- lines: 131-137
- severity: minor
- category: contradiction
- claim: "Two barrel shifter units sit between the Register File and the CU ... The shift is applied to the first source operand when fetching the source registers for an instruction. Shifts cannot be applied to the result or any other source operand."
- evidence: If only one operand can ever be shifted, the count "Two barrel shifter units" is unexplained; no other chapter mentions a second shifter (chapters/09-shift-operations.tex:12-13, :46-48 describe a single shift path).
- resolution: unclear
- fix: Either state what the second shifter is for (for example, one per read port with only one enabled per instruction) or change "Two barrel shifter units sit" to "A barrel shifter sits".
- confidence: medium

### F-con-36
- file: chapters/16-coprocessor-instructions.tex
- lines: 140
- severity: minor
- category: contradiction
- claim: "0x1         & Exception Unit       & Yes               & 0x0--0xD            & Handles reset, faults, interrupts, traps, and links"
- evidence: chapters/06-exceptions.tex:684-688 documents exception-unit operations 0xE (Fault) and 0xF (HardwareException) as real, internally-invoked operations of the same coprocessor, plus 0x0 (None).
- resolution: manual-wrong
- fix: Change the Operations cell to "0x0--0xD (software); 0xE--0xF internal" and add a footnote pointing at chapters/06-exceptions.tex §"Internal Instructions".
- confidence: high

### F-con-37
- file: chapters/05-status-register.tex
- lines: 101
- severity: minor
- category: contradiction
- claim: "\item Privileged coprocessor operations (opcodes > 0x07xx)"
- evidence: The rule is stated as an operation-nibble range everywhere else: chapters/01-introduction.tex:119-120 "the first 8 (0x0-0x7) can be called in any mode. The last 8 (0x8-0xF) can only be called in supervisor mode"; chapters/16-coprocessor-instructions.tex:51 "coprocessor-operation nibbles 0x0--0x7 are user-callable and 0x8--0xF are supervisor-only"; chapters/16-coprocessor-instructions.tex:106-107. The notation "opcodes > 0x07xx" is a numeric comparison against a command word and is wrong for any coprocessor other than 0x0.
- resolution: manual-wrong
- fix: Change to "\item Privileged coprocessor operations (operation nibble 0x8--0xF)".
- confidence: high

### F-con-38
- file: chapters/17-meta-instructions.tex
- lines: 14-33
- severity: minor
- category: contradiction
- claim: "\textbf{Meta-Instruction} & \textbf{Primary Documentation}    & \textbf{Lowers To}"
- evidence: `tab:meta-instruction-cross-reference` lists 19 meta-instructions but omits \mnemonic{NOOP}, which is documented in the very next section of the same chapter (chapters/17-meta-instructions.tex:40-44) and is listed in Chapter 12's meta-instruction table (chapters/12-instruction-summary.tex:137) and in Appendix E (chapters/appendix-e-quick-reference.tex:65).
- resolution: manual-wrong
- fix: Insert a row `\mnemonic{NOOP}           & Chapter~\ref{ch:meta-instructions} & \mnemonic{ADDI[N] r1, \#0}  \\` in alphabetical position.
- confidence: high

### F-con-39
- file: chapters/appendix-b-timing.tex
- lines: 156-163
- severity: minor
- category: contradiction
- claim: "; Instead of: \\ LOAD r1, (#0, a) \\ ADDI a, #2 \\ \\ ; Use: \\ LOAD r1, (#0, a)+          ; Same result, fewer cycles"
- evidence: Two problems. (1) `a` is an address-register *pair*, not an ALU-addressable register; the 4-bit register IDs are `al` = 0xB and `ah` = 0xA (chapters/03-registers.tex:248-249), so `ADDI a, #2` has no encoding. (2) Post-increment advances by exactly one word, not two: chapters/08-addressing-modes.tex:124-125 "Post-increment and pre-decrement change the selected address register by exactly one word"; chapters/14-memory-instructions.tex:81. The two sequences therefore do not produce the "Same result".
- resolution: manual-wrong
- fix: Change the "Instead of" block to `LOAD r1, (#0, a)` / `ADDI al, #1` and keep the "Use" block as-is.
- confidence: high

### F-con-40
- file: chapters/appendix-b-timing.tex
- lines: 133-143
- severity: minor
- category: contradiction
- claim: "CMPR r1, r2 \\ BRAN|<= @skip \\ ADDI r3, #1 \\ skip: \\ \\ ; Use: \\ CMPR r1, r2 \\ ADDI|HI r3, #1             ; Saves branch overhead"
- evidence: `<=` is the *signed* less-or-equal condition and `HI` is the *unsigned* higher condition (chapters/10-condition-codes.tex:30-35, :263-270), so the two sequences are not equivalent. chapters/10-condition-codes.tex:293 warns about exactly this: "\textbf{Signed vs. Unsigned}: Using unsigned conditions (\texttt{HI}, \texttt{LO}) for signed values or vice versa produces incorrect results".
- resolution: manual-wrong
- fix: Change `BRAN|<= @skip` to `BRAN|LO @skip` so both halves use unsigned conditions.
- confidence: high

### F-con-41
- file: chapters/08-addressing-modes.tex
- lines: 66
- severity: minor
- category: contradiction
- claim: "\texttt{BRAN/BRSR \#disp}; \texttt{BRAN/BRSR @label}; \texttt{RETS}; \texttt{LJMP src[, \#offset|rO]}; \texttt{LJSR src[, \#offset|rO]}; \texttt{LJSR (\#offset, src)+}."
- evidence: Chapter 15 documents a further legal form that this legality table omits: chapters/15-control-flow.tex:53 "LJSR & \texttt{LJSR (rS, src)+}" and chapters/15-control-flow.tex:461, :514. The register-displacement post-increment call form is missing from the Chapter 8 legality table.
- resolution: manual-wrong
- fix: Append `; \texttt{LJSR (rO, src)+}` to the Legal operand forms cell.
- confidence: high

### F-con-42
- file: chapters/16-coprocessor-instructions.tex
- lines: 328
- severity: minor
- category: contradiction
- claim: "LOAD al, @fixed_address   ; Correct the address"
- evidence: `@label` is defined only as a branch/call operand resolved by the linker to a PC-relative displacement (chapters/15-control-flow.tex:42, :144-145, :194-195; chapters/08-addressing-modes.tex:37 lists `@label` only under "branch meta-instructions"). The legal LOAD forms are `LOAD rD, #imm16` and `LOAD rD, rS` (chapters/08-addressing-modes.tex:61, chapters/13-alu-instructions.tex:78-79).
- resolution: manual-wrong
- fix: Change to `LOAD al, #0x0100          ; Correct the address`, or define an absolute-label immediate form for `LOAD` in Chapter 8 and Chapter 13 if the assembler really supports one.
- confidence: medium

### F-con-43
- file: chapters/05-status-register.tex
- lines: 299
- severity: minor
- category: contradiction
- claim: "ADDI r1, r2, r3"
- evidence: `ADDI` is the immediate form and takes at most `rD, #imm16` or `rD, #imm8, shift` (chapters/13-alu-instructions.tex:57-58, :253-254). The three-register form is `ADDR` (chapters/13-alu-instructions.tex:59, :255).
- resolution: manual-wrong
- fix: Change to `ADDR r1, r2, r3`.
- confidence: high

### F-con-44
- file: chapters/appendix-e-quick-reference.tex
- lines: 91
- severity: minor
- category: contradiction
- claim: "\mnemonic{WAIT}   & Meta          & COPI \#0x1900        & --             & Halt until an interrupt fires"
- evidence: chapters/16-coprocessor-instructions.tex:231 "Puts the CPU into wait state until an exception occurs."; chapters/06-exceptions.tex:667 "Enter wait state until an exception occurs". Faults and traps are exceptions but not interrupts, so "interrupt" is narrower than the normative definition.
- resolution: manual-wrong
- fix: Change the description to "Wait until an exception occurs".
- confidence: high

### F-con-45
- file: chapters/06-exceptions.tex
- lines: 442
- severity: minor
- category: contradiction
- claim: "(since vectors are 32-bit addresses stored as two 16-bit words)"
- evidence: Vectors hold 24-bit addresses stored in 32 bits: chapters/06-exceptions.tex:506 "Each vector is a 24-bit target address stored as two consecutive 16-bit words"; chapters/04-data-representation.tex:97 "Each exception vector table entry is a stored 24-bit address"; chapters/04-data-representation.tex:64-65 "Only bits 7--0 of the high word are significant when the value is loaded as an address." The same slip appears at chapters/06-exceptions.tex:166 "\textbf{Return Address:} The 32-bit address to return to".
- resolution: manual-wrong
- fix: Change line 442 to "(since each vector is a 24-bit address stored as two 16-bit words)" and line 166 to "The 24-bit return address, stored in a 32-bit field".
- confidence: high

### F-con-46
- file: chapters/06-exceptions.tex
- lines: 222-226
- severity: minor
- category: contradiction
- claim: "ETFR #6                   ; Save level 6 exception registers to a and r7 ... ETFR #7                   ; Save level 7 (metadata) exception registers to a and r7"
- evidence: 6 and 7 are *register indices*, not exception levels. chapters/06-exceptions.tex:192-199 (`tab:exception-link-register-assignment`) gives register 6 = Faults at priority level 7 and register 7 = Fault metadata at priority level "n/a". Calling register 7 "level 7" collides with priority level 7, which is the fault priority stored in register 6.
- resolution: manual-wrong
- fix: Change the comments to "; Save fault link register (register 6) to a and r7" and "; Save fault metadata register (register 7) to a and r7". Apply the same change to the matching `ETTR` comments at lines 238 and 242.
- confidence: high

### F-con-47
- file: chapters/appendix-c-undocumented.tex
- lines: 212
- severity: nit
- category: contradiction
- claim: "\item Reserved shift type (0x111) could enable additional shift modes"
- evidence: 111 is the *binary* reserved shift-type code, written bare in both encoding tables (chapters/07-instruction-formats.tex:305 "111 & Reserved"; chapters/09-shift-operations.tex:34 "111 & -- & Reserved"). `0x111` is a hex literal meaning 273. docs/reference/STYLE.md:49 requires bare digit groups for binary literals.
- resolution: manual-wrong
- fix: Change to "\item Reserved shift type (111) could enable additional shift modes".
- confidence: high

### F-con-48
- file: chapters/appendix-e-quick-reference.tex
- lines: 169
- severity: minor
- category: contradiction
- claim: "See Chapter~\ref{ch:cpu-architecture} (section on External Bus Signals) for usage examples including the PROT pin interaction."
- evidence: Chapter 2 has no section named "External Bus Signals". Its sections are "External Interface" -> "Chip Layout", "Pin Descriptions", "Timing Notes" (chapters/02-cpu-architecture.tex:167, :169, :315, :590). The BAT description lives at chapters/02-cpu-architecture.tex:441-474.
- resolution: manual-wrong
- fix: Change to "See Chapter~\ref{ch:cpu-architecture}, Section \"Pin Descriptions\", for usage examples including the PROT pin interaction."
- confidence: high

### F-con-49
- file: chapters/16-coprocessor-instructions.tex
- lines: 209-215
- severity: minor
- category: contradiction
- claim: "\textbf{Exceptions:} Dispatches a software exception through the exception unit. ... \textbf{Privilege:} User-callable in protected mode and supervisor mode."
- evidence: Two normative rules from Chapter 6 are absent from the EXCP entry: chapters/06-exceptions.tex:98 lists "Triggering a software exception with a vector below 0x60" as a privilege-violation fault, and chapters/06-exceptions.tex:142-144 "If an \mnemonic{EXCP} instruction is executed while any exception or fault handler is already active, the software exception is ignored, is not queued, and does not fetch its vector."
- resolution: manual-wrong
- fix: Add to the Exceptions field: "Raises a privilege-violation fault in protected mode if the selected vector is below 0x60. If any exception or fault handler is already active, the software exception is ignored and no vector is fetched (see Chapter~\ref{ch:exceptions})."
- confidence: high

### F-con-50
- file: chapters/12-instruction-summary.tex
- lines: 168-175
- severity: minor
- category: contradiction
- claim: "\item Execute/Address Calculate \\ \item Memory Access (or NOP)"
- evidence: The canonical phase names are given identically in two other places and differ here: chapters/02-cpu-architecture.tex:38 "\textbf{Execute and Address Calculation}" and :44 "\textbf{Memory Access}"; chapters/appendix-b-timing.tex:21-22 repeats Chapter 2 verbatim.
- resolution: manual-wrong
- fix: Change to "Execute and Address Calculation" and "Memory Access" to match Chapter 2 and Appendix B.
- confidence: high

### F-con-51
- file: chapters/13-alu-instructions.tex
- lines: 627-630
- severity: minor
- category: contradiction
- claim: "; Set bit N (variable) \\ ADDI r4, #1 \\ SHFT r4, LSL r5            ; r4 = 1 << r5"
- evidence: `ADDI r4, #1` adds 1 to whatever r4 already holds; the comment on the next line assumes r4 == 1. The equivalent example in Chapter 9 uses the correct instruction: chapters/09-shift-operations.tex:377 "LOAD r3, #1               ; r3 = 1" and :382, :389.
- resolution: manual-wrong
- fix: Change `ADDI r4, #1` to `LOAD r4, #1                ; r4 = 1`.
- confidence: high

### F-con-52
- file: chapters/09-shift-operations.tex
- lines: 163
- severity: minor
- category: contradiction
- claim: "\item Bit 15 (sign bit) is copied into bit 15 and bit 14"
- evidence: Only true for a one-position shift. The same list says the shift is by $n$ positions: chapters/09-shift-operations.tex:164 "Equivalent to signed division by $2^n$", and chapters/09-shift-operations.tex:288 "The shift count is a 4-bit field, allowing shifts of 0--15 positions." chapters/09-shift-operations.tex:148 states the general rule correctly: "Shift bits to the right, preserving the sign bit (bit 15)."
- resolution: manual-wrong
- fix: Change to "\item Bit 15 (sign bit) is replicated into all bits vacated by the shift".
- confidence: high

### F-con-53
- file: chapters/09-shift-operations.tex
- lines: 413
- severity: minor
- category: contradiction
- claim: "SHFT[N] r2, LSR #1    ; Shift low while preserving carry from high word"
- evidence: \mnemonic{SHFT} is defined as exactly `ORRI[S]` — the `[S]` is part of the meta-instruction: chapters/09-shift-operations.tex:535 "SHFT r1, LSL #3               ; ORRI[S] r1, #0, LSL #3"; chapters/13-alu-instructions.tex:192 "\textbf{Assembles to:} \texttt{ORRI[S] rD, \#0, shift}"; chapters/12-instruction-summary.tex:139; chapters/17-meta-instructions.tex:15 "\mnemonic{SHFT} ... \mnemonic{ORRI[S]}". `SHFT[N]` therefore asks for `[S]` and `[N]` at once. The same line is repeated at chapters/09-shift-operations.tex:422.
- resolution: unclear
- fix: Either state explicitly in the \mnemonic{SHFT} entry (chapters/13-alu-instructions.tex:191-243) that an explicit status override suffix on \mnemonic{SHFT} overrides the implied `[S]`, or change both example lines to `ORRI[N] r2, #0, LSR #1`.
- confidence: medium

### F-con-54
- file: chapters/15-control-flow.tex
- lines: 481-482
- severity: minor
- category: contradiction
- claim: "Because \mnemonic{LJSR} implicitly writes \reg{l} and \reg{p}, post-increment source registers cannot be \reg{l} or \reg{p}."
- evidence: The same chapter and Appendix C classify the same situation as architecturally undefined behaviour that the assembler merely rejects, not as an architectural prohibition: chapters/15-control-flow.tex:71-73 "Instructions that write the same address-register pair through more than one write-back path have architecturally undefined behavior. Avoid forms such as ... The assembler rejects known hazardous aliased forms."; chapters/appendix-c-undocumented.tex:20-22, :35-36.
- resolution: manual-wrong
- fix: Change "cannot be" to "must not be \reg{l} or \reg{p}; such forms are aliased address-register writes with architecturally undefined behaviour and are rejected by the assembler (see Appendix~\ref{appendix:undocumented})".
- confidence: medium

### F-con-55
- file: chapters/09-shift-operations.tex
- lines: 248-253
- severity: minor
- category: contradiction
- claim: "0               & Shift by Immediate      & R2 (first source operand) is shifted by a literal amount \\ 1               & Shift Count in Register & R2 is shifted by the amount in the shift count register"
- evidence: `tab:shift-operand` describes SO purely in terms of the Register-format field R2, but SO also exists in the Short Immediate format, where there is no R2 field — the shifted operand is the Reg field (chapters/07-instruction-formats.tex:138-146). chapters/09-shift-operations.tex:60 (`tab:source-operand-examples`) shows the short-immediate case shifting `r1` (the Reg field), not R2.
- resolution: manual-wrong
- fix: Change both Description cells to "The first source operand (Register format: R2; Short Immediate format: the Reg field) is shifted ...".
- confidence: high

---

### F-con-56
- file: chapters/16-coprocessor-instructions.tex
- lines: 293-306
- severity: major
- category: structure
- claim: "\textbf{Assembles to:} \texttt{COPI \#0x1Cxy} or \texttt{COPI \#0x1Dxy} ... \texttt{\#n} selects an exception link register. Optional operands select whether to transfer the saved return address, saved status register, or both."
- evidence: The `xy` operand byte of the ETFR/ETTR command word is never defined anywhere in the manual. Contrast the DMA operand byte, which is fully specified at chapters/16-coprocessor-instructions.tex:366-370. chapters/16-coprocessor-instructions.tex:185-186 gives the ranges 0x1C00--0x1C7F and 0x1D00--0x1D7F without a field breakdown, and chapters/06-exceptions.tex:308 and :645-650 describe the instructions only in prose. An implementer cannot encode `ETFR a, #6` from this manual.
- resolution: manual-wrong
- fix: Add a paragraph to the ETFR/ETTR entry, in the same shape as the DMA operand paragraph, specifying which bits of the low byte hold the link-register index (0--7) and which bits select address-only / status-only / both.
- confidence: high

### F-con-57
- file: chapters/15-control-flow.tex
- lines: 554-558
- severity: minor
- category: structure
- claim: "jump_table: \\     DW case0 - jump_table \\     DW case1 - jump_table"
- evidence: `DW` is used here and nowhere else in the manual; no chapter defines assembler directives. The surrounding example is also misleading: `LDEA p, (r2, p)` (chapters/15-control-flow.tex:552) computes `p = p + r2` and never reads the table, so the `DW` entries are unused.
- resolution: manual-wrong
- fix: Either drop the `DW` lines and re-comment the example as a computed branch into a run of branch instructions, or add a short "Assembler directives used in examples" note to Chapter 11 defining `DW`, `:label` and `@label`.
- confidence: high

### F-con-58
- file: chapters/16-coprocessor-instructions.tex
- lines: 419
- severity: minor
- category: structure
- claim: "\textbf{Clobbers:} Writes \reg{r1} through \reg{rn}."
- evidence: `Clobbers` is used as an instruction-entry field at chapters/16-coprocessor-instructions.tex:419, :467 and :510 but is not among the fields defined in chapters/11-reading-instructions.tex:9-25 and is not in the canonical field order at docs/reference/STYLE.md:63-65.
- resolution: manual-wrong
- fix: Add `\item[Clobbers] Registers that the instruction overwrites beyond its named destination.` to the Chapter 11 field list and place it in the canonical order (after `Write-back`), or fold the information into the `Write-back` field of the three DMA entries.
- confidence: high

### F-con-59
- file: chapters/11-reading-instructions.tex
- lines: 54-69
- severity: minor
- category: structure
- claim: "\texttt{S}      & Updated from the shifter result when \texttt{[S]} is used"
- evidence: `tab:flag-effect-symbols` defines five symbols and omits `U`. docs/reference/STYLE.md:136-139 (Gate 1, approved) requires: "The symbol set is `*` (updated), `0`, `1`, `-` (preserved), `S` (from shifter), and `U` (undefined: the implementation leaves the flag in an unspecified state). Chapter 11's notation table must define `U`." Chapter 11 is the table that every flag table in Chapters 13--15 cites (chapters/13-alu-instructions.tex:162, chapters/14-memory-instructions.tex:121, chapters/15-control-flow.tex:128).
- resolution: manual-wrong
- fix: Add a row `\texttt{U}      & Architecturally undefined; the flag is left in an unspecified state \\` to `tab:flag-effect-symbols`.
- confidence: high

---

### F-con-60
- file: chapters/02-cpu-architecture.tex
- lines: multiple
- severity: minor
- category: terminology
- claim: "DMA Co-processor Read Burst"
- evidence: docs/reference/STYLE.md:22 "coprocessor | co-processor | 155 vs 7; drop the hyphen everywhere, including headings."
- resolution: manual-wrong
- fix: `grep -rl 'co-processor\|Co-processor' docs/reference/chapters/` then `s/co-processor/coprocessor/g; s/Co-processor/Coprocessor/g`. Files: chapters/02-cpu-architecture.tex (4), chapters/06-exceptions.tex (1), chapters/appendix-e-quick-reference.tex (2).
- confidence: high

### F-con-61
- file: chapters/06-exceptions.tex
- lines: multiple
- severity: minor
- category: terminology
- claim: "e.g.\ mapping a page in response to a bus fault"
- evidence: docs/reference/STYLE.md:125-126 (Gate 1, approved) "**\"for example\", never \"e.g.\"** Spell it out in prose and tables alike. Replace every `e.g.` manual-wide."
- resolution: manual-wrong
- fix: `s/(e\.g\.,? ?/(for example, /g` and `s/e\.g\.\\? /for example, /g`, then re-read each site for comma placement. Files: chapters/02-cpu-architecture.tex (1), chapters/05-status-register.tex (1), chapters/06-exceptions.tex (4), chapters/07-instruction-formats.tex (1).
- confidence: high

### F-con-62
- file: chapters/07-instruction-formats.tex
- lines: multiple
- severity: minor
- category: terminology
- claim: "SHFT is a meta instruction using ORRI opcode (0x25) with AF=0b10"
- evidence: docs/reference/STYLE.md:19 "meta-instruction | meta instruction, alias, convenience instruction | Resolved in handover; hyphenated form dominant (73 vs 4)."
- resolution: manual-wrong
- fix: `s/meta instruction/meta-instruction/g`. Files: chapters/02-cpu-architecture.tex:585, chapters/07-instruction-formats.tex:167,252, chapters/09-shift-operations.tex:15.
- confidence: high

### F-con-63
- file: chapters/06-exceptions.tex
- lines: multiple
- severity: minor
- category: terminology
- claim: "For abort exceptions (faults), the return address is the address of the faulting instruction so it can be retried."
- evidence: docs/reference/STYLE.md:17 "fault | abort exception | \"Fault\" dominant (207 vs 4); \"abort exception\" survives only as a stray label, already flagged fixed in handover."
- resolution: manual-wrong
- fix: `s/abort exceptions (faults)/faults/g; s/Faults (abort exceptions)/Faults/g; s/Faults (abort exceptions, highest priority)/Faults (highest priority)/g`. Files: chapters/06-exceptions.tex:182,412,639, chapters/16-coprocessor-instructions.tex:167.
- confidence: high

### F-con-64
- file: chapters/03-registers.tex
- lines: multiple
- severity: minor
- category: terminology
- claim: "\item \textbf{Upper Byte (bits 15--8):} Control and privileged state"
- evidence: docs/reference/STYLE.md:21 "high word / high byte / low word / low byte | upper word / upper byte | 43 \"high\" vs 12 \"upper\"; standardize on \"high\"/\"low\"."
- resolution: manual-wrong
- fix: `s/[Uu]pper [Bb]yte/high byte/g; s/[Uu]pper word/high word/g; s/[Uu]pper nibble/high nibble/g; s/upper 8 bits/high 8 bits/g` (capitalize where the original was a heading or table cell). Files: chapters/01-introduction.tex:149, chapters/02-cpu-architecture.tex:488, chapters/03-registers.tex:77,189,192,238, chapters/05-status-register.tex:11,100,220,221, chapters/09-shift-operations.tex:126, chapters/13-alu-instructions.tex:622,913,922, chapters/14-memory-instructions.tex:386, chapters/appendix-e-quick-reference.tex:107.
- confidence: high

### F-con-65
- file: chapters/16-coprocessor-instructions.tex
- lines: 202
- severity: minor
- category: terminology
- claim: "Used by user-mode programs to invoke system calls or request supervisor-mode services."
- evidence: docs/reference/STYLE.md:15 "protected mode | user mode | Resolved in handover Workstream 1; \"user mode\" is now unused (0 hits)." This is the only surviving instance in the corpus.
- resolution: manual-wrong
- fix: `s/user-mode programs/protected-mode programs/`. One file: chapters/16-coprocessor-instructions.tex:202.
- confidence: high

### F-con-66
- file: generated/immediate-format-encodings.tex
- lines: multiple
- severity: minor
- category: terminology
- claim: "0b01"
- evidence: docs/reference/STYLE.md:49 "Binary literals: bare digit groups in tables (`00`, `10`, `111`), no `0b` prefix. The two existing `0b10` instances in `07-instruction-formats.tex` are the outliers; convert to the bare form used everywhere else." The AF column of all three generated tables uses `0b01`/`0b10`/`0b11` while the parenthetical decode row directly beneath each one uses the bare form `(01)`/`(10)`/`(11)`.
- resolution: manual-wrong
- fix: `s/0b\([01][01]\)/\1/g` across generated/immediate-format-encodings.tex, generated/short-immediate-format-encodings.tex, generated/register-format-encodings.tex and chapters/07-instruction-formats.tex:167,252. Note the three `generated/` files are machine-produced, so the generator must be changed rather than the output patched.
- confidence: high

### F-con-67
- file: chapters/appendix-a-opcode-map.tex
- lines: 140
- severity: minor
- category: terminology
- claim: "These opcodes may execute in hardware but their behavior is implementation-defined and may change between CPU revisions."
- evidence: docs/reference/STYLE.md:24 names this exact line: "`appendix-a-opcode-map.tex:140` still says \"implementation-defined\" for undocumented opcodes — flag for correction to match the resolved definition; true implementation-defined language stays reserved for genuine per-model timing variance." chapters/appendix-c-undocumented.tex:7 gets it right ("their architectural behaviour is undefined"), as does chapters/16-coprocessor-instructions.tex:159-160.
- resolution: manual-wrong
- fix: `s/their behavior is implementation-defined and may change between CPU revisions/their behavior is architecturally undefined and may change between CPU revisions/`. One file: chapters/appendix-a-opcode-map.tex:140.
- confidence: high

### F-con-68
- file: chapters/09-shift-operations.tex
- lines: 43
- severity: minor
- category: terminology
- claim: "A shift postamble can come after any short immediate or register instruction."
- evidence: The same construct is called a "shift suffix" in Chapter 8 and Chapter 15 (chapters/08-addressing-modes.tex:62 "Shift suffixes are only available on register-displacement memory forms", :63, :64, :65; chapters/15-control-flow.tex:75-76 "\textbf{Shift suffixes:} ... Register-offset forms do not accept shift suffixes"), and "shift" or "shift definition" in Chapter 11 (chapters/11-reading-instructions.tex:41). "Postamble" appears once in the whole manual.
- resolution: manual-wrong
- fix: `s/shift postamble/shift suffix/`. One file: chapters/09-shift-operations.tex:43.
- confidence: high

### F-con-69
- file: chapters/06-exceptions.tex
- lines: 648-649
- severity: minor
- category: terminology
- claim: "\textbf{ETFR (Exception Transfer From Register):} Copies the return address and/or status register from a link register to the address register and/or R7"
- evidence: docs/reference/STYLE.md:28 names these exact lines: "`06-exceptions.tex:648–649` uses bare \"R7\" for the real register `r7`; correct to `\reg{r7}` to match the 89-instance convention." docs/reference/STYLE.md:27 reserves bare `R1/R2/R3` for abstract field-position labels only. chapters/16-coprocessor-instructions.tex:311-312 uses `\reg{r7}` correctly.
- resolution: manual-wrong
- fix: `s/\bR7\b/\\reg{r7}/g` on chapters/06-exceptions.tex:648-649 only. While in the same paragraph, wrap the bare mnemonics `ETFR`/`ETTR` (lines 648, 649) in `\mnemonic{}`; the same bare-mnemonic problem occurs at chapters/02-cpu-architecture.tex:585 (RSET), chapters/03-registers.tex:83 (STOR/LOAD), chapters/06-exceptions.tex:138 (EXCP), :308 (\texttt{ETFR}), :492 and :628 (RETE).
- confidence: high

### F-con-70
- file: chapters/01-introduction.tex
- lines: multiple
- severity: minor
- category: terminology
- claim: "vary depending on which SIRC-1 model you have"
- evidence: docs/reference/STYLE.md:75-77 "Voice/tense: third person, present tense ... No first person (\"we\"), no second person (\"you\"). This already matches the manual; treat any surviving \"you\" as a defect to flag."
- resolution: manual-wrong
- fix: Rewrite each site in third person; there is no single mechanical substitution. Sites: chapters/01-introduction.tex:97, chapters/02-cpu-architecture.tex:569, chapters/03-registers.tex:85, chapters/06-exceptions.tex:250, chapters/07-instruction-formats.tex:336, chapters/09-shift-operations.tex:15 and :506, chapters/14-memory-instructions.tex:407, chapters/appendix-c-undocumented.tex:178, :185, :188, :189. Locate with `grep -rnw 'you\|your\|You\|Your' docs/reference/chapters/`.
- confidence: high

---

### F-con-71
- file: chapters/01-introduction.tex
- lines: 141-142
- severity: minor
- category: latex
- claim: "With a 24-bit address space, this allows access to $2^24$ = 16,777,216 words, or 32 megabytes of external byte storage."
- evidence: `$2^24$` renders as $2^2 4$ (superscript 2 followed by a literal 4), not $2^{24}$. The manual gets this right elsewhere: chapters/04-data-representation.tex:150 "$2^{16}$", :157, chapters/16-coprocessor-instructions.tex:386.
- resolution: manual-wrong
- fix: Change `$2^24$` to `$2^{24}$`.
- confidence: high

### F-con-72
- file: chapters/02-cpu-architecture.tex
- lines: 280-309
- severity: minor
- category: latex
- claim: "\begin{table}[] \\ \centering \\ \begin{tabular}{rll} \\ \textbf{Pin} & \textbf{Name} & \textbf{Description}          \\ \hline"
- evidence: docs/reference/STYLE.md:94-95 "Use `booktabs` (`\toprule`/`\midrule`/`\bottomrule`) for every table; no vertical rules, no hand-drawn `\hline`." Three tables in the manual violate this: chapters/02-cpu-architecture.tex:280-309 and :459-474 use `\hline` with no rules, and chapters/appendix-a-opcode-map.tex:11-91 uses both `\hline` and vertical rules (`{|c|l|l|l|}`). Every other table in the corpus uses booktabs. The two Chapter 2 tables also use an empty float specifier `\begin{table}[]` where every other table uses `[H]`.
- resolution: manual-wrong
- fix: Convert all three tables to `booktabs`: replace the header `\hline` with `\midrule`, add `\toprule` after `\begin{tabular}` and `\bottomrule` before `\end{tabular}`, delete the interior `\hline`s, and drop the vertical rules from the Appendix A column spec (`{|c|l|l|l|}` -> `{cll l}`). Change `\begin{table}[]` to `\begin{table}[H]` at 02-cpu-architecture.tex:280 and :459. The two Chapter 2 tables also lack `\label`s while their captions are cross-referenced only informally; add `\label{tab:pinout}` and `\label{tab:bat-encoding}` if they are to be referenced.
- confidence: high

---

## Summary

Findings by category:

| Category | blocker | major | minor | nit | total |
|---|---|---|---|---|---|
| contradiction | 8 | 24 | 22 | 1 | 55 |
| structure | 0 | 1 | 3 | 0 | 4 |
| terminology | 0 | 0 | 11 | 0 | 11 |
| latex | 0 | 0 | 2 | 0 | 2 |
| **total** | **8** | **25** | **38** | **1** | **72** |

Mechanical checks all pass: every `\ref` target has a matching `\label` (0 dangling), every `\label`
is unique (0 duplicates), every `\input` in `main.tex` resolves and every file in
`chapters/` is included, and every `lstlisting`, `instructionbox`, `table`, `tabular`, `figure`,
`bytefield` and `verbatim` environment is balanced across `chapters/` and `generated/`.

### The three most damaging contradictions

1. **F-con-1 — the condition-code table in Chapter 7 disagrees with the one in Chapter 10 on two of
   sixteen encodings.** Chapter 7 calls `1100` = `<` and `1110` = `<<`; Chapter 10 calls them `<<`
   and `<=`. Appendix E's "where to find other tables" points implementers at the Chapter 7 version.
   Every example in the manual follows Chapter 10, so an assembler written from Chapter 7 would
   silently emit signed-less-or-equal where the programmer wrote signed-less-than. This is a
   normative encoding disagreement between two tables that both claim to be complete, and it
   propagates into F-con-2 (Chapter 13's comparison guidance, which invents `<` and `>`).

2. **F-con-4 — the manual cannot decide what a register-offset memory shift shifts.** Chapter 14
   says four times that the *loaded value* is shifted after the memory read and before write-back.
   Chapter 9 and Chapter 2 both state flatly that shifts apply only to the first source operand and
   never to a result, and Chapter 9 pins the shift to the Decode phase, three phases before the data
   exists. The generated encoding tables settle nothing: `LOAD r1, (r2, a)` encodes R2 = 0, so under
   the Chapter 9 rule there is no operand to shift. Hardware built from Chapter 9 and hardware built
   from Chapter 14 will not agree on the result of `LOAD rD, (rO, addr), LSL #2`.

3. **F-con-3 — the fault metadata register bit diagram is drawn backwards.** The figure places BAT
   in bits 15--13 and Reserved in bits 3--0; the prose immediately beneath it places BAT in bits
   2--0 and Reserved in bits 15--12. `bytefield` MSB-left is the manual's own convention (the status
   register figure in Chapter 5 follows it), so the figure is unambiguously the wrong one. A fault
   handler written from the figure decodes every field of every fault at the wrong offset — and
   this is the register a double-fault last-chance handler reads first.
