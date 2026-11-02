# Author decisions on the phase 3 open questions (2026-11-02)

1. Active-low signal names take `\activelow{}` overbars in all three manuals, in prose, tables, timing-diagram row labels and the hardware pinout figure (section headings keep the plain name). This includes RSTI, RSTO, BERR, BPER, BACK, BAS, IRQ1-IRQ4, NMI, HALT and TRCE in the architecture manual.
2. HALT during reset: the six-cycle RSTO hold runs first; if HALT is asserted the CPU then halts at the first phase 0, before the reset-vector fetch.
3. While BAS is deasserted, A0-A23 are driven with an architecturally undefined value (BRW high, BAT 000 as already decided).
4. Keep "any of the 16 bits" for protected-mode writes to a high address register: bits 7-0 are the segment number, which protection guards.
5. Relabel the E1-E4 interrupt lines as "line n (exception level n+1)" instead of "Level n"; line 5 (NMI) runs at exception level 6.
6. Chapter 8 uses "displacement" for addressing-mode values, per STYLE.md; "offset" stays only for structure offsets.
7. Reserved LDEA/LDEL bits 25-24: "written as zero; a non-zero value is architecturally undefined" (same form as STOR's reserved R1 field).
8. The simulator's PROGRAM segment is documented as read-only (enforcement is simulator todo 51).

Orchestrator fixes from the same review: Ch.8 Indirect Immediate box gets the auto-update wrap clause; hardware Ch.5 defines its value-band abbreviations; assembly Table 8.1's .ORG row points to Section 5.1; Ch.9 prose names the shifted operand as the table does; Ch.13 SHFT Operation reads the flags from the [S]/[A]/[N] suffix (default [S]). Backlog: Ch.12 flags column in Chapter 11 symbols.

9. F-con2-1: a traced EXCP's recorded trap is held in the exception unit's own pending-software-exception record, not in the CSR. It is entered before any pending hardware interrupt when the trace handler returns to level 0 (same visible behavior as the F-exc-2 decision; replaces its "held in the CSR" wording).

10. Clock edges (author confirmed 2026-11-02, hardware manual): (1) the CPU changes all its outputs (A0-A23, BRW, BAT,
    BAS, PROT, D0-D15 on writes, SYNC, RSTO) just after the CLKI rising edge that starts a phase; (2) BAS is asserted for
    the whole of a phase that makes a bus access and not in phases that do not; (3) the CPU samples BACK, BERR, BPER and
    read data on the next rising edge, ending that phase, and if none of the three is asserted it repeats the phase (a
    wait state); (4) IRQ1-IRQ4, NMI, HALT and TRCE are sampled on every rising edge except during reset; (5) the falling
    edge of CLKI is not used. Timing figures mark the sample points on the rising edges.
