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

## Simulating and building it

The system manual names real parts, but a simulator or an FPGA does not have to reproduce a vendor chip's silicon. It has to
reproduce what a program can see.

* **Simulator.** Of the three vendor parts, only the MC68681 DUART needs a model: its 16 registers, the two channels'
  asynchronous transmit and receive with their buffers, the baud-rate generator, the counter/timer, the parallel input and
  output ports (which read the controllers) and the interrupt. The model can follow the datasheet. It should include the access
  time of 2 to 9 cycles (`DTACK` follows the DUART's own clock), which the VM already supports as bus latency. The 74LS138 is
  the VM's segment map. The TDA1541A is the sample stream that SIRCAS's model already produces. The serial channels would
  connect to the host as a pipe, a terminal or a TCP socket, so two VMs could be linked for the multiplayer link.
* **FPGA.** The SIRC-1, SIRCADA and SIRCAS are the project's own designs and would go in the FPGA as RTL. The vendor parts can
  stay real, on a board next to the FPGA, with the bus brought out to them. That needs level translation between the FPGA's
  3.3 V and the parts' 5 V. Or a register-compatible subset of the DUART goes in the FPGA, which is small, since a UART is
  one of the simplest peripherals. Exar still makes a CMOS part (the XR68C681) that is compatible with the MC68681.
* **Custom serial chip.** If a fully custom console is wanted, a serial chip in the SIRC family, with its own manual, would
  be the easiest thing to simulate and to put in an FPGA, at the price of designing it. The manual's chapter 5 would then
  describe it in place of the DUART, and the controller design would not change.

## Why SIRCUS was expensive

SIRCUS came out in 1993 after the Mega Drive and the SNES, and it cost more than they did. The design explains why. None of
these was a mistake: each was the simplest or the best-sounding choice, and together they set the price.

* **Fast video memory.** The renderer reads a word every 93 ns, so the video memory is two 35 ns static RAMs, the cache-class parts
  of 1993 PCs, dearer than the 70 to 100 ns parts that cheaper consoles used. A wider bus would have allowed cheaper memory, but
  needed the fetch schedule redesigned, so it was not taken (`docs/video/DESIGN-NOTES.md`).
* **Static RAM everywhere.** The working memory (128 KB), the video memory and the audio memory are all static RAM. DRAM would have
  been cheaper per bit but needs refresh and a controller, and makes timing less exact.
* **A hi-fi converter.** The audio converter is a 16-bit part made for CD players, with three supplies and two output amplifiers,
  and a console that carries it also needs the extra supplies.
* **Big custom chips.** SIRCADA and SIRCAS each have about 77 to 79 signal pins plus power, so their packages are large.
* **Wait states.** The bus is fast compared with the memory of the time, which costs speed more than money, but a console
  with a faster cartridge ROM would need dearer cartridges.
