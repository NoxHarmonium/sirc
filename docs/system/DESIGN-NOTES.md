# SIRCUS system manual: design notes

Rationale and context behind the system manual. The manual itself is written as a period hardware reference and says none of this.

## The era

SIRCUS is a console for first release in **1993**. SIRC missed the boat: the Mega Drive (1988 in Japan, 1989 in North America) and
the SNES (1990 in Japan, 1991 in North America) were already on sale, so SIRCUS arrived second and later. Parts are chosen to have
been available by 1993, and the manuals cite datasheets of that time where they can. Where only a later datasheet exists, it is used
for pins and registers and not for ratings (the MC68HC681 user's manual is dated 1996).

## Parts and when they were available

| Part | Role | Availability |
| --- | --- | --- |
| Motorola MC68681 DUART | Serial port | Advance information datasheet dated September 1985, derived from the Signetics 2681. The MC68HC681 is a CMOS version in the same pins. |
| Philips TDA1541A | Audio converter | The TDA1541 dates from 1985. The datasheet used is the A version, February 1991. |
| 74LS138 | Select decoder | Standard TTL part, decades old by 1993. |

## Sources used for the serial device

* `MC68681P.pdf`: scan of the Motorola advance information in the *M68000 Family Reference Manual*, pages 7-33 to 7-67. Source of the
  pins, the register map, the DC and AC figures and the bus timing in chapter 5.
* `MC68681UM.pdf`: the MC68HC681 user's manual (1996, CMOS). Used only to cross-check. Its differences from the original: it
  latches write data when `CS` is negated and not at the `DTACK` edge, its minimum `CS` negation after `DTACK` is 0 ns and
  not 20 ns, and it allows faster clocks.
* **A discrepancy in the original:** its signal table says `RESET` is active low, and so does the 1996 manual, but the original's
  Figure 3 draws a high pulse. The system manual follows the table and the later manual: active low, 1.0 us.
