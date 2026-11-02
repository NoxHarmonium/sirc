# Changes: reference/chapters/notation-and-glossary.tex

## Findings applied
F-sum-5, F-sum-6, F-sum-7, F-con-27, F-con-18, F-con-38, F-asm-5 (reference side).

## Applied differently or not applied
- F-asm-5: its fix targets the assembly manual. Here, the sentence after Table "Operand Forms" now says only some instructions accept a label reference and points to the `@label` entry, which F-sum-5 corrected.
- F-con-38: the macros `\archmanual`, `\hwmanual` and `\asmmanual` already exist in the preamble. Used `\asmmanual{}` and `\hwmanual{}`; the "-1" in the title now comes from the macro.
- Not touched (simulator): F-sum-4, F-asm-9, F-asm-14. F-asm-8 agrees with the existing text.
- F-con-32 (overbar for ARCH signal names): the glossary still writes signals in `\texttt`; see open questions.

## Copy edits
"hex" to "hexadecimal"; "Can be" to "May be"; `Table~\ref` fix; caption "Operand Forms" in Title Case.

## Open questions
- Should `\activelow{}` extend to the ARCH signal names (Signals, Requests and Pins, and other ARCH chapters)? The chapter currently says they are in typewriter font.
- The `.EQU` entry says a constant is "usable anywhere a immediate operand is accepted". The toolchain rejects constants in shift counts, EXCP and ETFR/ETTR (F-asm-9, simulator status). Left unchanged.

## Round 2

Applied F-con2-9, F-con2-10 (entry shortened, per-object rule and pointer to the Assembly manual added), F-con2-11, and the signal-name sentence (now overbar, not typewriter). No open questions.
