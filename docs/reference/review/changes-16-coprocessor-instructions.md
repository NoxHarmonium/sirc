# Changes — chapters/16-coprocessor-instructions.tex

## Findings applied
- F-cop-1: Reworked the "Retry-Safe DMA Copy Routine" example (`ex:dma-copy-safe`). The
  routine previously used `l` as both DMAT's destination register and the incoming
  subroutine link register, so RETS would jump into the copied data instead of returning to
  the caller. New calling convention: the caller passes the source address in `a`
  (unchanged) and the destination in `r1`/`r2` (high/low word); the routine saves the
  incoming return address (`lh`/`ll`) to the stack before touching `l`, loads `l` from the
  destination parameter, and restores the saved return address into `l` immediately before
  every RETS (success path and `:dma_copy_failed`). Updated the prose paragraph before the
  example and the listing's header/calling-convention comments to match. Raised the transfer
  count from `#7` to `#16` (see F-cop-5 below) so the "may fault mid-transfer" comment is
  physically possible (a 7-word transfer is a single DMAT chunk and can't fault
  mid-transfer). Extracted the corrected listing verbatim from the manual and assembled it
  with `sirc-vm/target/debug/assembler`; it assembles cleanly (exit 0, no errors).
- F-cop-3: "whose execution spans more than one bus access" → "more than one data-memory bus
  access" (Section~\ref{sec:dma-fault-retry} intro).
- F-cop-4: Replaced the backward-DMA-decrement/pre-decrement-addressing comparison, which
  had the fault-recovery direction backwards, with the corrected contrast (pre-decrement's
  update is undone by a fault because it commits only at write-back; the DMA decrement
  commits before the transfer loop begins and is not undone).
- F-cop-5: Folded into the F-cop-1 rewrite — raised the example's DMAT count from `#7` to
  `#16` rather than only rewording the comment, since `#16` also demonstrates the
  chunk-boundary retry behavior more realistically.
- F-cop-6: "retrying the whole transfer up to 3 times" → "attempting the whole transfer up
  to 3 times in total"; added "(3 attempts in total)" to the `LOAD r1, #3` retry-budget
  comment.
- F-cop-7: Reworded the Legal Forms section so opcode `0x2F` is no longer grouped with the
  architecturally-undefined opcodes 0x08/0x09/0x0B/0x0D; it now reads as Appendix C's stated
  exception (defined behavior, no public syntax).
- F-cop-8: "however many words the first attempt already completed" → "the number of words
  in the chunks the first attempt fully completed."
- F-cop-10: Added to the prose before the example and to the listing's fault-handler header
  comment: the handler assumes `dma_copy_safe` is the only DMA user in the system, and a
  handler shared with other DMA callers would also need to check the faulting return address
  (`ETFR a, #6`) against the expected range.
- F-cop-11: Added a sentence to DMAT's Write-back field noting that `l` is also the
  subroutine link register and that a subroutine issuing DMAT must save/restore its return
  address around the transfer.
- Invalid Opcode Fault scope (this session's resolution): Fixed the Section 16.2-adjacent
  paragraph (~line 702) to "Missing DMA hardware raises an invalid-opcode fault. Unsupported
  DMA operations and reserved operand encodings are architecturally undefined..." per the
  approved wording, and corrected the same "unsupported operation(s) ... invalid-opcode
  fault" pattern everywhere else it appeared in this file for internal consistency with the
  chapter's own Model and Revision Policy: the Overview section, the Coprocessor
  Compatibility intro, the Common Semantics prose and table row, the COPI/COPR entry's
  Exceptions field, and the DMAR/DMAW/DMAT and MULU/MULS/DIVU/DIVS entries' Exceptions
  fields. Added a `\label{sec:coprocessor-model-revision-policy}` to the Model and Revision
  Policy subsection (it had none) so these could cross-reference it properly.
- F-cons-17: Wrapped bare opcode literals in `\opcode{}`: Table~\ref{tab:coprocessor-legal-forms}
  (`0x0F`/`0x3F`), the Legal Forms prose (`0x0F`/`0x3F`/`0x2F`), and the COPI/COPR entry's
  Opcodes field.
- F-cons-18: Converted all 11 `\textbf{Operation:}` pseudocode blocks from `lstlisting` to
  `verbatim`, matching Chapters 13–15.
- F-cons-19: Replaced the one "e.g." inside the DMA example's listing comments with "for
  example" (kept natural: "may already be wrong, for example / double-decremented by a
  backward transfer").
- F-cons-21: "which would make its instruction count unpredictable like every other
  meta-instruction's is not" → "which would make its instruction count vary with its
  operand, unlike every other meta-instruction" (removes the reserved-word "unpredictable").

## Findings I could not apply as written
- None of the findings tagged to this chapter required deviating from their stated fix; the
  Invalid Opcode Fault scope fix was extended beyond its literally-scoped grep (DMAR/DMAW/DMAT
  Exceptions fields only) to the Overview, Coprocessor Compatibility, Common Semantics, and
  maths-unit passages that shared the exact same "unsupported operation → invalid-opcode
  fault" contradiction with this chapter's own Model and Revision Policy — this is a direct,
  same-document application of the author's already-settled ruling, not a new technical
  decision, but is called out here since it goes past the literal instruction scope.

## Open questions
- The DMAR/DMAW Exceptions fields also list "invalid counts" as a cause of invalid-opcode
  faults (counts outside `#-7`–`#7`). The 3-bit count-magnitude field can only encode
  0–7, so it's unclear whether "invalid counts" describes a real runtime CPU fault or only
  an assembler-time syntax rejection that can never reach hardware. Left this phrase alone
  (it wasn't part of any triaged finding) rather than guess; flagging in case a technical
  reviewer wants to tighten it.
