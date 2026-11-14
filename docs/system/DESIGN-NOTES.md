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
| Philips TDA1543 | Audio converter | The economy version of Philips' 16-bit converters. The datasheet used is dated February 1991. An earlier draft used the hi-fi TDA1541A (datasheet of the same month), which needs three supplies and costs more. |
| 74ALS138 | Select decoder | Advanced low-power Schottky TTL, pin-compatible with the 74LS138 and about twice as fast (20 ns against 40 ns); common and cheap by 1993. |

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
  time of 2 to 9 cycles (`DTACK` follows the DUART's own clock), which the VM already supports as bus latency. The 74ALS138 is
  the VM's segment map. The TDA1543 is the sample stream that SIRCAS's model already produces. The serial channels would
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
* **A composite video encoder.** The picture leaves as composite video on a yellow jack, which needs the MC1377 encoder, a +12 V
  rail for it, a tuned chroma transformer and a 400 ns luminance delay line, because the picture has detail well above 1.5 MHz.
* **Big custom chips.** SIRCADA and SIRCAS each have about 77 to 79 signal pins plus power, so their packages are large.
* **Wait states.** The bus is fast compared with the memory of the time, which costs speed more than money, but a console
  with a faster cartridge ROM would need dearer cartridges.

An earlier draft listed a hi-fi converter among these. The converter is now the economy TDA1543, which runs from the +5 V supply
alone and also takes standard I2S, so it is not a cost driver.

## The audio output stage

The converter's datasheet shows each output current turned into a voltage by an op-amp with 1.2 kOhm and 3.3 nF in the feedback, which swings
2.76 V either side of the 2.2 V reference. That needs a supply wider than the console's +5 V, so the values are changed.

* **Current to voltage.** The top of the swing must stay at least 0.3 V under the supply and the bottom at least 0.3 V above ground. With
  the reference at its lowest, 2.10 V, that allows 1.8 V either side, and the converter's largest full-scale current is 2.65 mA, so the
  feedback resistor is at most 680 Ohm (1.8 V divided by 2.65 mA). 5.6 nF in parallel keeps the corner near 40 kHz, 41.8 kHz.
* **Low-pass filter.** The output is not oversampled, so images of the signal sit around 31.96 kHz and start just above 16 kHz, which no
  cheap filter removes. A second-order Sallen-Key filter is cheap (one op-amp section for each channel) and takes the top off:
  10 kOhm, 10 kOhm, 1.5 nF and 820 pF give a corner of 14.4 kHz and a Q of 0.68. Its response is -1.2 dB at 10 kHz, -3.2 dB at 14 kHz,
  -4.4 dB at 16 kHz, -8.4 dB at 22 kHz (the image of a 10 kHz tone) and -13 dB at 30 kHz. The images are only partly removed, which
  gives the console some of its character. A steeper filter would cost more op-amp sections and still lose the top of the band.
* **Coupling.** The output rests at the 2.2 V reference, so a 10 uF capacitor removes it: -3 dB at 1.6 Hz into 10 kOhm.
* **Amplifiers.** Two sections for each channel make a quad op-amp. It needs an output that swings to within 0.3 V of each rail on 5 V.
  The datasheet's NE5532 needs a wider supply, so a rail-to-rail type is required.

## The power budget

A rough 5 V budget, from typical figures of the period (to be replaced when the custom chips' currents are known): working RAM 100 mA,
video memory 200 mA, audio memory 50 mA, DUART 150 mA, converter 50 mA, glue logic 100 mA, cartridge 250 mA (room for a coprocessor and its memory), the three custom chips
150 mA each (assumed), and 40 mA of controllers and op-amps: about 1.39 A, 6.95 W. A linear regulator from 12 V would lose 9.7 W.
A switching regulator at 82 percent loses 1.5 W and draws 0.7 A at 12 V. Adding the encoder's 0.72 W gives about 9 W, so a 12 V 1 A
adapter. It is regulated because the MC1377 must not see more than 14 V, which an unregulated adapter gives at light load.

## Where the timing figures come from

* 74LS138: the National Semiconductor datasheet of June 1989 (supplied). Select or enable to output, three levels of logic, high to low: 40 ns at 50 pF and 28 ns at 15 pF. Outputs sink 8 mA.
* 74ALS138: 20 ns, from memory of the Texas Instruments datasheet. Not yet checked.
* CPU, SIRCADA and SIRCAS: targets, not measurements: outputs valid in 25 ns, inputs need 10 ns of set-up. The CPU's are in `docs/hardware`, chapter 6. They are in the range of early-1990s CMOS CPUs, as I recall them (a 68020-class part at 25 MHz was about 25 ns and 5 to 10 ns). An FPGA implementation will meet them inside the chip, but its pins and any level shifting to 5 V parts add a few nanoseconds.
