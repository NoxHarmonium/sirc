# SIRCUS system manual: status, findings and open questions

This file tracks what is not yet decided. The manual's body marks each of these "not yet specified" and invents no value for it
(`README.md`, Voice). Answer an item and the matching places in the manual get a value; the "Where" column says which
chapters change.

Every "reference model" entry below says what the VM in `sirc-vm` does today. It is evidence, not a design decision, and the
manual does not say it.

## Open questions for you

| ID | Question | Where it appears | What exists today |
| --- | --- | --- | --- |
| U1 | **Working RAM and system ROM.** *Answered: 128 KB of working RAM (confirmed: 65,536 words, one segment; the examples' program segment is the cartridge, and their scratch segment is the RAM) and a system ROM.* Region Y0 is the RAM, Y1 the ROM. Still open: RAM type and speed (wait states), ROM size, type and speed. | Ch. 1, ch. 3, ch. 5, ch. 7, appendix | The VM maps 64K words at segment 0, read-only, with a 2-cycle latency, and a scratch segment at 1. |
| U2 | **How a program gets in.** *Answered: at reset the CPU fetches its reset vector from the cartridge (the boot overlay maps the cartridge at segment 0). By convention words 0 and 1 of a cartridge hold the address of the system ROM's start-up routine, which sets the system up and jumps to the cartridge program. A game may put its own boot address there.* Still open: the entry point after boot (a header, a fixed address?), what the ROM sets up, and whether the ROM copies the cartridge's own vector table into RAM or writes a default one. | Ch. 3, ch. 7 | The VM loads a binary into the program segment. |
| U3 | **Reset circuit.** What asserts `RSTI` at power-up? Is there a reset button? How long is it held? | Ch. 4, ch. 7 | Nothing. The CPU manual only says what `RSTI` does. |
| U4 | **SIRCAS reset width.** Minimum `RST` pulse (already open in the SIRCAS manual). SIRCADA needs 6 cycles, which the CPU's 6-cycle `RSTO` hold gives. | Ch. 4 | Open in `docs/audio`. |
| U5 | **Chip selects and segments.** *Answered: one 74LS138 on A23 to A21, one 2M-word region per device. SIRCADA moved from `0x0C` to `0x60` and SIRCAS is at `0x80`. The VM, the examples and both chip manuals now use these.* | Ch. 1, ch. 3, ch. 5, appendix | Done. |
| U6 | **Unmapped addresses.** Does the system assert `BERR` for a segment nobody owns, and after how long? Without it the CPU waits for ever. | Ch. 3, ch. 5 | The VM warns and raises a bus error. |
| U7 | **Memory protection.** Is there a controller that uses `PROT`, `BAT` and `BPER`? Which regions? Do any devices want the `BAT` burst hints? | Ch. 3, ch. 5 | The VM has a read-only flag on segments (a write raises `BPER`). The program segment uses it. |
| U8 | **`IRQ1` and `IRQ2`.** *Answered: `IRQ2` is the serial device (confirmed). The manual's new section on the serial interrupt covers the DUART's eight conditions, the level-sensitive behaviour and the wait behind higher levels.* `IRQ1` stays free: the cartridge, if it needs an interrupt. | Ch. 4, ch. 5, appendix | Done for `IRQ2`. |
| U9 | **Pull-up values** for `BACK`, `NMI`, `IRQ3`, `IRQ4` and the serial `IRQ`. The DUART's open-drain outputs (`DTACK`, `IRQ`) sink only 2.4 mA at 0.4 V, so a pull-up must be at least about 2 kOhm. | Ch. 4, ch. 5 | Each device manual says only that the system supplies them. |
| U10 | **CPU `SYNC`, `HALT`, `TRCE`.** What are they connected to (nothing, a debug header, the reset circuit)? | Ch. 5 table 5.3 | Nothing in any manual. |
| U11 | **Oscillator and clock distribution.** Part, tolerance, the divide-by-two circuit, and the phase of `CLKI` at the three chips. How is a console made NTSC or PAL? | Ch. 2 | Both frequencies are fixed by the video manual. Nothing on the circuit. **Proposal, not a decision:** the NTSC frequency is 6 times the common 3.579545 MHz colour crystal, so a plain crystal oscillator at the master frequency works. The PAL frequency (4.8 times 4.43361875 MHz) is a non-standard value: a crystal can be cut to it as a custom part, or a PLL can multiply the common 4.43 MHz crystal by 24/5. Either way the divide by two is a single flip-flop and no special circuit is needed. This is from memory, not checked against a part list. |
| U12 | **Clock range of the CPU and SIRCADA.** Only SIRCAS states one (9.0 to 12.0 MHz). | Ch. 2 | Nothing. |
| U13 | **Audio memory part.** Two 32,768 by 8 memories or one by 16? Access time and set-up and hold of the interface. | Ch. 5 | The SIRCAS manual asks for an access within 2 cycles. |
| U14 | **Converter and analogue stage.** *Converter answered: the Philips TDA1541A (datasheet supplied, Feb 1991); the chapters use its pins, limits and supplies. It has no `SCK` pin, so `SCK` is left unconnected.* Still open: the output amplifier part and its supplies, the anti-image filter, the output connector, and the edge timing of `SD` and `WS` (set-up of 32 ns needed, SIRCAS leaves it open). | Ch. 1, ch. 5, ch. 7 | The datasheet's reference circuit uses two NE5534 or equivalent with 1.8 kOhm and 2.2 nF in the feedback; the draft quotes it as a reference, not as the design. |
| U15 | **Video output circuit.** Value of the `IREF` reference, the output stage, the connector, signal levels. | Ch. 5, ch. 7 | Nothing. |
| U16 | **Controller ports.** *Answered and confirmed: shift-register controllers (4021 or 74HC165, up to 16 buttons), read by software through the serial device's parallel pins: `OP2` clock, `OP3` latch, `IP2` and `IP3` data, one pass reading both ports in about 50 accesses (0.2 to 0.4% of a frame). Port 1 is controller only; port 2 is a controller or a TTL serial cable on channel B, sharing pins through open-collector gates; a modem or PC uses an adapter cable with the RS-232 level circuit, so the console needs none.* Still open: the connectors and their pins, the button layout and bit order, and whether the pads carry an id. | Ch. 5, ch. 7 | None in any manual. |
| U17 | **Cartridge connector and the physical form** of the console. *Cartridges are confirmed, and a cartridge has the 32-segment region Y2, a window of 2M words (4 MB).* Open: the connector, which bus signals and interrupt lines it carries, bank switching, how the console detects a cartridge (U27), and whether a cartridge may hold RAM or other hardware. | Ch. 1, ch. 3, ch. 5, ch. 7 | None. |
| U18 | **Power.** Supply voltages for the CPU, SIRCADA and SIRCAS, regulation, decoupling, the amplifiers' supplies, the connector. | Ch. 7 | Answered for the converter (+5 V, -5 V, -15 V, with currents and ripple rejection, table 7.1) and the serial device (+5 V, 150 mA maximum). An RS-232 level circuit, if the console has one, would need more (see U16). |
| U19 | **Electrical timing between chips.** Set-up and hold, delays. Every manual shows logical timing only. | Ch. 2 | Nothing. |
| U20 | **SIRCAS package and pin numbers** (open in the SIRCAS manual). Table 5.1 shows SIRCAS pins by name only. | Ch. 1, ch. 5 | Open in `docs/audio`. |
| U21 | **Region parts.** The video manual says SIRCADA's region is fixed when it is made. So is a PAL console a PAL SIRCADA plus a 21.28 MHz oscillator, and does the audio chip need no variant? | Ch. 2 | The SIRCAS manual has no region input. |
| U22 | ~~The 16-bit word in a 24-bit slot.~~ **Resolved:** SIRCAS now sends 16-bit slots, 32 bit clocks a frame of 10 and 11 `CLKI` cycles alternately, so nothing follows the 16th bit (`docs/audio`, serial output; the model and its test are changed). | Ch. 2, ch. 5 | Done. |
| U23 | **Installing interrupt handlers.** *Answered: a boot overlay. Your idea of mapping the cartridge at segment 0 is in the manual.* While the overlay is set the cartridge answers at segment 0 (the system ROM if no cartridge is present) and the RAM does not. `RSTO` sets it. A write to the system ROM's own segment clears it, after which RAM answers at segment 0. Words 0 and 1 of the cartridge are the reset vector. A game can clear the overlay itself with the same write, so it can do without the ROM. Still open: the segments of the ROM and cartridge, and confirmation that the clearing trigger is a write to the ROM segment (no extra address needed) and not a register. | Ch. 3, ch. 4, ch. 7 | None. |
| U24 | **System ROM driver routines.** Where is the entry table, what is the calling convention, which services (video set-up, loading the audio memory, input, and so on)? Probably a manual of its own. | Ch. 3 (only mentions them) | Nothing yet. |
| U25 | **BACK for the memories, and wait states.** The devices assert their own `BACK`. Plain RAM, ROM and cartridge memories cannot, so a small circuit must, and must add wait states if the ROM or the cartridge is slow. The instruction counts in ch. 6 assume none. | Ch. 3, ch. 5, ch. 6 | Nothing. |
| U26 | ~~How the chip selects are made.~~ **Resolved: one 74LS138** (see U5). Spare outputs Y6 and Y7 can drive `BERR` for unmapped regions (U6). The overlay needs a 74LS74, clocked by `CLKI` so a decode glitch cannot clear it, and a few gates: Y0 is gated by the overlay (RAM when clear, cartridge or ROM when set). | Ch. 3, ch. 5 | Done. |
| U27 | ~~No cartridge.~~ **Resolved:** fall back to the ROM. A cartridge grounds a pin on its connector, with a pull-up on the console; the overlay maps the ROM at segment 0 while the pin is high. By default the ROM shows the SIRCUS logo and a missing-cartridge icon; holding a controller button at reset enters a simple monitor that loads a program over the serial port (U30). | Ch. 3, ch. 7 | Done. |
| U28 | **The serial I/O device: MC68681 DUART.** *Answered: pins, register addresses, bus timing, the crystal, the reset and the glue are in chapter 5, from the original Motorola advance information (the scan you supplied). The 1996 manual is the CMOS MC68HC681, a pin-compatible later part, used only to cross-check.* Findings: (1) `RESET` needs 1.0 us and `RSTO` gives 559 ns, so a pulse stretcher is needed (U29). (2) `DTACK` follows the DUART's own clock, not `CLKI`, so a cycle takes 2 to 9 cycles and `BACK` is re-timed by a flip-flop and an open-collector gate. (3) 3.6864 MHz crystal; deriving it from the master clock would be 3% off at NTSC and 4% at PAL. (4) Its outputs sink 2.4 mA at 0.4 V, which limits the pull-ups (U9). (5) It draws up to 150 mA (U18). (6) The original's timing figure draws `RESET` as a high pulse, but its table and the 1996 manual say active low; I followed those. Still open: the connectors, the handshake lines, the speed range, which of `IRQ1` and `IRQ2`, and the RS-232 level circuit or an all-TTL design (U16). | Ch. 3, ch. 5, ch. 7 | The VM has a `TERMINAL` device at `0x0A` and a `DEBUG` device at `0x0B`; you said not to treat them as the source of truth. They fall inside the RAM region in the new map. |
| U29 | **Reset pulse stretcher.** The DUART needs a 1.0 us `RESET`. `RSTO` after a software reset is 6 cycles, 559 ns (NTSC). SIRCAS's own minimum is unspecified (U4), and SIRCADA needs 6 cycles. One stretcher on `RSTO` (a monostable or a short counter) could serve all three. | Ch. 4, ch. 5 | Nothing. |
| U30 | **The ROM monitor.** Which button, the load protocol (a simple framed format or an existing one such as XMODEM), the speed (the DUART's standard rates reach 38,400 baud), where the program goes in the 128 KB of RAM and how it starts, and whether the monitor offers anything else (peek, poke, run). Also the logo and icon (drawn by the ROM with SIRCADA). | Ch. 7 | Nothing. |
| U31 | **Simulating and building the vendor parts.** See `DESIGN-NOTES.md`. The VM needs a model of the DUART only. The 74LS138 is the segment map and the converter is SIRCAS's output stream, which the VM already has. For an FPGA build the vendor parts stay as real chips beside the FPGA, or a register-compatible subset of the DUART goes into the FPGA. | Whole manual | The VM has no DUART model yet. |

## Findings while writing

* **The PAL clock is already fixed.** The SIRCADA manual gives a PAL master clock of 21,281,370 Hz (the PAL subcarrier times
  4.8). With the divide by two, `CLKI` is 10,640,685 Hz, which is inside SIRCAS's 9.0 to 12.0 MHz range and makes every note
  15.9 cents flat of NTSC. The SIRCAS manual still says "the frequency of `CLKI` in a PAL system is not yet specified"
  (`docs/audio/chapters/01b-package-and-pinout.tex`) and lists the PAL clock as open in `README.md`. **Done at your request**: the pinout
  chapter, the clock-rate and timing sections, the device summary, `README.md` and `STATUS.md` of `docs/audio` now give
  10,640,685 Hz.
* **The converter datasheet supplied is the TDA1541A (1991), not the TDA1541 (1985) I had read before.** The 1985 text needed an `SCK` input; the 1991 part runs its own oscillator and has none. The SIRCAS notes are corrected (`docs/audio/STATUS.md`, Target DAC).
* **Frame rate rounding.** The video manual quotes 50.00 Hz for PAL. At 212,784 cycles a frame and 10,640,685 Hz it is
  50.007 Hz, so this manual says 50.01 Hz. NTSC is 60.10 Hz in both.
* **Video line length.** The video manual uses 1364 master clocks a line (60.10 Hz frames). A broadcast NTSC line is 1365.0
  clocks, so the video runs 0.07% fast against broadcast. This does not affect any figure here; noted in case you want a
  broadcast-exact frame.
* **Devices that exist only in the VM.** `TERMINAL` at segment `0x0A` and `DEBUG` at `0x0B` are simulator devices. They are
  not part of the console, so the manual does not mention them. If you want them reserved on the real map, say so.

## Choices I made in the draft

All easy to change.

| Choice | Why |
| --- | --- |
| The serial part is the original NMOS MC68681, not the CMOS MC68HC681 | The era is 1993 (`DESIGN-NOTES.md`) and the NMOS part dates from 1985. The CMOS part is a drop-in. |
| `DTACK` is re-timed by a flip-flop on `CLKI` | `DTACK` follows the DUART's own clock. Without it the CPU could sample `BACK` at a bad moment. |
| The serial device is in region Y5 (`0xA0`) | It is the next free output of the decoder. Y6 and Y7 stay free. |
| One 2M-word region per device | You said space is not a constraint, and it needs one decoder. |
| Selects: 74LS138 named, layout open | You chose the 74LS138. The body says the logic is made of 74LS138 decoders and gives the segments that are fixed (U26). |
| Cartridge at the low words at reset | Your proposal: the overlay maps the cartridge at segment 0, and its words 0 and 1 point at the ROM start-up routine. |
| The manual says SIRCAS's segment is unspecified, not `0x0D` | The SIRCAS manual says so. The VM's `0x0D` is only evidence (U5). |
| Both regions given in every table | The PAL clock is fixed by the video manual, so there is nothing to withhold. |
| Block diagram has one clock rail and no interrupt lines | The lines are in ch. 4 and the net tables of ch. 5. |
| `BACK` drawn as a shared open-drain net | Both devices' manuals say it is open drain. Whether the program memory takes part is open (U1). |

## Not done

* No timing diagrams of bus cycles to each chip. The CPU manual has the cycle diagrams and each device manual has its own, so
  this manual refers to them. A diagram of a CPU access to the audio window next to the sample schedule might be worth adding.
* No electrical specification of any net.
* The manual has had no review pass.
