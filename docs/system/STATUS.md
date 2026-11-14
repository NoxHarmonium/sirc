# SIRCUS system manual: status, findings and open questions

This file tracks what is not yet decided. The manual's body marks each of these "not yet specified" and invents no value for it
(`README.md`, Voice). Answer an item and the matching places in the manual get a value; the "Where" column says which
chapters change.

Every "reference model" entry below says what the VM in `sirc-vm` does today. It is evidence, not a design decision, and the
manual does not say it.

## Open questions for you

| ID | Question | Where it appears | What exists today |
| --- | --- | --- | --- |
| U1 | **Working RAM and system ROM.** *Answered: 128 KB of working RAM (I took KB as kilobytes, 65,536 words, which is exactly one segment; say if you meant otherwise) and a system ROM.* Still open: RAM type and speed (wait states), ROM size, type and speed, and the segment of each. | Ch. 1 table 1.2, ch. 3, ch. 5, ch. 7, appendix | The VM maps 64K words at segment 0, read-only, with a 2-cycle latency. |
| U2 | **How a program gets in.** *Answered: at reset the CPU fetches its reset vector from the cartridge (the boot overlay maps the cartridge at segment 0). By convention words 0 and 1 of a cartridge hold the address of the system ROM's start-up routine, which sets the system up and jumps to the cartridge program. A game may put its own boot address there.* Still open: the entry point after boot (a header, a fixed address?), what the ROM sets up, and whether the ROM copies the cartridge's own vector table into RAM or writes a default one. | Ch. 3, ch. 7 | The VM loads a binary into the program segment. |
| U3 | **Reset circuit.** What asserts `RSTI` at power-up? Is there a reset button? How long is it held? | Ch. 4, ch. 7 | Nothing. The CPU manual only says what `RSTI` does. |
| U4 | **SIRCAS reset width.** Minimum `RST` pulse (already open in the SIRCAS manual). SIRCADA needs 6 cycles, which the CPU's 6-cycle `RSTO` hold gives. | Ch. 4 | Open in `docs/audio`. |
| U5 | **Chip selects and segments.** SIRCAS at segment `0x80` (your example; any select scheme can give it) and SIRCADA at `0x0C` (its manual). The way the selects are made is open (U26). | Ch. 1, ch. 3, ch. 5, appendix | The VM puts SIRCAS at segment `0x0D`, not `0x80`. |
| U6 | **Unmapped addresses.** Does the system assert `BERR` for a segment nobody owns, and after how long? Without it the CPU waits for ever. | Ch. 3, ch. 5 | The VM warns and raises a bus error. |
| U7 | **Memory protection.** Is there a controller that uses `PROT`, `BAT` and `BPER`? Which regions? Do any devices want the `BAT` burst hints? | Ch. 3, ch. 5 | The VM has a read-only flag on segments (a write raises `BPER`). The program segment uses it. |
| U8 | **`IRQ1` and `IRQ2`.** Are they used (controllers, cartridge)? | Ch. 4, appendix | Unassigned everywhere. |
| U9 | **Pull-up values** for `BACK`, `NMI`, `IRQ3`, `IRQ4`. | Ch. 4, ch. 5 | Each device manual says only that the system supplies them. |
| U10 | **CPU `SYNC`, `HALT`, `TRCE`.** What are they connected to (nothing, a debug header, the reset circuit)? | Ch. 5 table 5.3 | Nothing in any manual. |
| U11 | **Oscillator and clock distribution.** Part, tolerance, the divide-by-two circuit, and the phase of `CLKI` at the three chips. How is a console made NTSC or PAL? | Ch. 2 | Both frequencies are fixed by the video manual. Nothing on the circuit. **Proposal, not a decision:** the NTSC frequency is 6 times the common 3.579545 MHz colour crystal, so a plain crystal oscillator at the master frequency works. The PAL frequency (4.8 times 4.43361875 MHz) is a non-standard value: a crystal can be cut to it as a custom part, or a PLL can multiply the common 4.43 MHz crystal by 24/5. Either way the divide by two is a single flip-flop and no special circuit is needed. This is from memory, not checked against a part list. |
| U12 | **Clock range of the CPU and SIRCADA.** Only SIRCAS states one (9.0 to 12.0 MHz). | Ch. 2 | Nothing. |
| U13 | **Audio memory part.** Two 32,768 by 8 memories or one by 16? Access time and set-up and hold of the interface. | Ch. 5 | The SIRCAS manual asks for an access within 2 cycles. |
| U14 | **Converter and analogue stage.** *Converter answered: the Philips TDA1541A (datasheet supplied, Feb 1991); the chapters use its pins, limits and supplies. It has no `SCK` pin, so `SCK` is left unconnected.* Still open: the output amplifier part and its supplies, the anti-image filter, the output connector, and the edge timing of `SD` and `WS` (set-up of 32 ns needed, SIRCAS leaves it open). | Ch. 1, ch. 5, ch. 7 | The datasheet's reference circuit uses two NE5534 or equivalent with 1.8 kOhm and 2.2 nF in the feedback; the draft quotes it as a reference, not as the design. |
| U15 | **Video output circuit.** Value of the `IREF` reference, the output stage, the connector, signal levels. | Ch. 5, ch. 7 | Nothing. |
| U16 | **Controller ports.** Does the console have any, and how do they connect? | Ch. 7 | None in any manual. |
| U17 | **Cartridge connector and the physical form** of the console. *Cartridges are confirmed.* Open: the connector, which bus signals and interrupt lines it carries, its address window (which segments, how large, any bank switching), how the console detects a cartridge, and whether a cartridge may hold RAM or other hardware. | Ch. 1, ch. 3, ch. 5, ch. 7 | None. |
| U18 | **Power.** Supply voltages for the CPU, SIRCADA and SIRCAS, regulation, decoupling, the amplifiers' supplies, the connector. | Ch. 7 | Answered for the converter only: +5 V, -5 V and -15 V, with currents and ripple rejection (table 7.1). |
| U19 | **Electrical timing between chips.** Set-up and hold, delays. Every manual shows logical timing only. | Ch. 2 | Nothing. |
| U20 | **SIRCAS package and pin numbers** (open in the SIRCAS manual). Table 5.1 shows SIRCAS pins by name only. | Ch. 1, ch. 5 | Open in `docs/audio`. |
| U21 | **Region parts.** The video manual says SIRCADA's region is fixed when it is made. So is a PAL console a PAL SIRCADA plus a 21.28 MHz oscillator, and does the audio chip need no variant? | Ch. 2 | The SIRCAS manual has no region input. |
| U22 | ~~The 16-bit word in a 24-bit slot.~~ **Resolved:** SIRCAS now sends 16-bit slots, 32 bit clocks a frame of 10 and 11 `CLKI` cycles alternately, so nothing follows the 16th bit (`docs/audio`, serial output; the model and its test are changed). | Ch. 2, ch. 5 | Done. |
| U23 | **Installing interrupt handlers.** *Answered: a boot overlay. Your idea of mapping the cartridge at segment 0 is in the manual.* While the overlay is set the cartridge answers at segment 0 (the system ROM if no cartridge is present) and the RAM does not. `RSTO` sets it. A write to the system ROM's own segment clears it, after which RAM answers at segment 0. Words 0 and 1 of the cartridge are the reset vector. A game can clear the overlay itself with the same write, so it can do without the ROM. Still open: the segments of the ROM and cartridge, and confirmation that the clearing trigger is a write to the ROM segment (no extra address needed) and not a register. | Ch. 3, ch. 4, ch. 7 | None. |
| U24 | **System ROM driver routines.** Where is the entry table, what is the calling convention, which services (video set-up, loading the audio memory, input, and so on)? Probably a manual of its own. | Ch. 3 (only mentions them) | Nothing yet. |
| U25 | **BACK for the memories, and wait states.** The devices assert their own `BACK`. Plain RAM, ROM and cartridge memories cannot, so a small circuit must, and must add wait states if the ROM or the cartridge is slow. The instruction counts in ch. 6 assume none. | Ch. 3, ch. 5, ch. 6 | Nothing. |
| U26 | **How the chip selects are made.** Either wire address bits straight to the selects (one inverter for SIRCAS, whose `CS` is active low), or use a decoder. **Recommendation: one PAL or GAL.** It takes A16 to A23, `BAS`, `BRW` and `RSTO`; it gives every `CS` for any segment you like (SIRCADA stays at `0x0C`, SIRCAS at `0x80`), holds the overlay in a registered output, can assert `BERR` when no segment matches (which settles U6 with no timeout), and can generate `BACK` for the memories (U25). Direct bit wiring has overlap and mirror problems: any address with two selecting bits set makes two devices answer and drive the bus at once. A 74-series decoder (a 74LS138 gives active-low outputs, matching the `CS` pins) would do for coarse regions but not for SIRCADA at `0x0C` next to RAM at `0x00` without a second one. | Ch. 3 | Nothing. |
| U27 | **No cartridge.** The overlay maps the cartridge at segment 0 when one is present and the ROM otherwise, so the logic needs to know. A cartridge-present pin on the connector (grounded by the cartridge) would do. What should the ROM do then: show a screen, or just wait? | Ch. 3, ch. 7 | Nothing. |

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
| Selects: no mechanism in the body | You first proposed address bits, then asked about a decoder. The body says only that select logic exists and gives the segments (U26). |
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
