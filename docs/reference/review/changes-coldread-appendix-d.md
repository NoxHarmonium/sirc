# Changes: appendix-d-examples.tex (cold-read follow-up)

Added two-to-four sentences of orienting prose before each `\lstinputlisting`, grounded in a
reading of the corresponding `.sasm` file under `examples/`. No files under `examples/` were
modified.

## Byte Sieve (Example D-1)
Explained that the program is a Sieve of Eratosthenes storing one byte per candidate in a
memory-mapped array, noted the addressing-register setup, and pointed to the nested loop at
labels `:2`/`:3` (lines 35-50) as the core algorithm.

## Faults (Examples D-2 and D-3)
For `faults.sasm`: listed the full sequence of fault types deliberately triggered (bus,
alignment, two segment-overflow, instruction-trace, bus-protection, invalid-opcode/double
fault, privilege violation, level-five interrupt), noted each handler increments a distinct
register for verification, and flagged the segment-overflow handler (lines 167-204) as
worth close reading because it disambiguates instruction-fetch vs. effective-address
overflow via `ETFR`.
For `faults-high.sasm`: explained it is a separate segment used only so the PC-overflow test
can fault without executing the real vector table, and noted the trampoline back at 0xFFF0.

## Hardware Exception (Examples D-4 and D-5)
For `hardware-exception.sasm`: described the serial-device interrupt flow (print help, wait,
match expected character in r7) and pointed to the exception handler and its
read/write-pending-byte subroutines as the poll-and-acknowledge pattern to study.
For `data.sasm`: noted it supplies the two zero-terminated messages used by Example D-4.

## Software Exception (Example D-6)
Explained the EXCP-triggered handler and the nested-exception-suppression check via the final
value of r1, and the decoy handler at 0x0500 that should never run.

## Store Load (Example D-7)
Described the store/load round-trip block (lines 8-19) and the exhaustive LJMP/LJSR
addressing-mode exercise, explaining the FAIL!-commented decoy jumps and the pass/fail labels
used to self-check.

## Open questions
None. All descriptions were derived directly from reading the listings; no technical
decisions were required.
