# SIRCUS system manual: status, findings and open questions

This file tracks what is not yet decided. The manual's body marks each of these "not yet specified" and invents no value for it
(`README.md`, Voice). Answer an item and the matching places in the manual get a value; the "Where" column says which
chapters change.

Every "reference model" entry below says what the VM in `sirc-vm` does today. It is evidence, not a design decision, and the
manual does not say it.

## Open questions for you

| ID | Question | Where it appears | What exists today |
| --- | --- | --- | --- |
| U1 | **Program memory.** What is it: RAM, ROM, both? Size, type, speed (wait states), and which words of the address space it fills. | Ch. 1 table 1.2, ch. 3 (map, cycle lengths), ch. 5, ch. 7 | The VM maps 64K words at segment 0, read-only, with a 2-cycle latency. The architecture manual puts the vectors in the low words, so something must answer there. |
| U2 | **How a program gets in.** Cartridge slot, boot ROM, or something else? Where does the reset vector live? | Ch. 3, ch. 7 | The VM loads a binary into the program segment. Nothing in the manuals says. |
| U3 | **Reset circuit.** What asserts `RSTI` at power-up? Is there a reset button? How long is it held? | Ch. 4, ch. 7 | Nothing. The CPU manual only says what `RSTI` does. |
| U4 | **SIRCAS reset width.** Minimum `RST` pulse (already open in the SIRCAS manual). SIRCADA needs 6 cycles, which the CPU's 6-cycle `RSTO` hold gives. | Ch. 4 | Open in `docs/audio`. |
| U5 | **Chip-select decoder and the SIRCAS segment.** What decodes A16 to A23? Which segment does SIRCAS take? | Ch. 1, ch. 3 (address map), ch. 5, appendix | The VM puts SIRCAS at segment `0x0D`. SIRCADA is at `0x0C` (in the video manual). The SIRCAS manual says "not yet specified". |
| U6 | **Unmapped addresses.** Does the system assert `BERR` for a segment nobody owns, and after how long? Without it the CPU waits for ever. | Ch. 3, ch. 5 | The VM warns and raises a bus error. |
| U7 | **Memory protection.** Is there a controller that uses `PROT`, `BAT` and `BPER`? Which regions? Do any devices want the `BAT` burst hints? | Ch. 3, ch. 5 | The VM has a read-only flag on segments (a write raises `BPER`). The program segment uses it. |
| U8 | **`IRQ1` and `IRQ2`.** Are they used (controllers, cartridge)? | Ch. 4, appendix | Unassigned everywhere. |
| U9 | **Pull-up values** for `BACK`, `NMI`, `IRQ3`, `IRQ4`. | Ch. 4, ch. 5 | Each device manual says only that the system supplies them. |
| U10 | **CPU `SYNC`, `HALT`, `TRCE`.** What are they connected to (nothing, a debug header, the reset circuit)? | Ch. 5 table 5.3 | Nothing in any manual. |
| U11 | **Oscillator and clock distribution.** Part, tolerance, the divide-by-two circuit, and the phase of `CLKI` at the three chips. How is a console made NTSC or PAL? | Ch. 2 | Both frequencies are fixed by the video manual. Nothing on the circuit. **Proposal, not a decision:** the NTSC frequency is 6 times the common 3.579545 MHz colour crystal, so a plain crystal oscillator at the master frequency works. The PAL frequency (4.8 times 4.43361875 MHz) is a non-standard value: a crystal can be cut to it as a custom part, or a PLL can multiply the common 4.43 MHz crystal by 24/5. Either way the divide by two is a single flip-flop and no special circuit is needed. This is from memory, not checked against a part list. |
| U12 | **Clock range of the CPU and SIRCADA.** Only SIRCAS states one (9.0 to 12.0 MHz). | Ch. 2 | Nothing. |
| U13 | **Audio memory part.** Two 32,768 by 8 memories or one by 16? Access time and set-up and hold of the interface. | Ch. 5 | The SIRCAS manual asks for an access within 2 cycles. |
| U14 | **Converter, `SCK`, analogue stage.** *Converter answered: the TDA1541.* Still open: the frequency and phase of `SCK` (needs the datasheet's timing figures 3 to 5, which were images I could not read), the I/V stage, the filter and the connector. Should the body name the converter? The SIRCAS manual names no real part, so the draft does not either. | Ch. 5, ch. 7 | The SIRCAS notes (`docs/audio/STATUS.md`) already record the datasheet text: two's complement input (pin tied high), `SCK` required, `BCK` up to 6 MHz. |
| U15 | **Video output circuit.** Value of the `IREF` reference, the output stage, the connector, signal levels. | Ch. 5, ch. 7 | Nothing. |
| U16 | **Controller ports.** Does the console have any, and how do they connect? | Ch. 7 | None in any manual. |
| U17 | **Expansion or cartridge connector, and the physical form** of the console. | Ch. 7 | None. |
| U18 | **Power.** Supply voltages, regulation, decoupling, the converter's supplies, the connector. | Ch. 7 | Pin names only. The converter datasheet text says it needs +5 V, -5 V and -15 V. |
| U19 | **Electrical timing between chips.** Set-up and hold, delays. Every manual shows logical timing only. | Ch. 2 | Nothing. |
| U20 | **SIRCAS package and pin numbers** (open in the SIRCAS manual). Table 5.1 shows SIRCAS pins by name only. | Ch. 1, ch. 5 | Open in `docs/audio`. |
| U21 | **Region parts.** The video manual says SIRCADA's region is fixed when it is made. So is a PAL console a PAL SIRCADA plus a 21.28 MHz oscillator, and does the audio chip need no variant? | Ch. 2 | The SIRCAS manual has no region input. |

## Findings while writing

* **The PAL clock is already fixed.** The SIRCADA manual gives a PAL master clock of 21,281,370 Hz (the PAL subcarrier times
  4.8). With the divide by two, `CLKI` is 10,640,685 Hz, which is inside SIRCAS's 9.0 to 12.0 MHz range and makes every note
  15.9 cents flat of NTSC. The SIRCAS manual still says "the frequency of `CLKI` in a PAL system is not yet specified"
  (`docs/audio/chapters/01b-package-and-pinout.tex`) and lists the PAL clock as open in `README.md`. **Done at your request**: the pinout
  chapter, the clock-rate and timing sections, the device summary, `README.md` and `STATUS.md` of `docs/audio` now give
  10,640,685 Hz.
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
| Program memory at segment `0x00` | The vectors are in the low words of the address space, so something must answer there. Size is left open. |
| The manual says SIRCAS's segment is unspecified, not `0x0D` | The SIRCAS manual says so. The VM's `0x0D` is only evidence (U5). |
| Both regions given in every table | The PAL clock is fixed by the video manual, so there is nothing to withhold. |
| Block diagram has one clock rail and no interrupt lines | The lines are in ch. 4 and the net tables of ch. 5. |
| `BACK` drawn as a shared open-drain net | Both devices' manuals say it is open drain. Whether the program memory takes part is open (U1). |

## Not done

* No timing diagrams of bus cycles to each chip. The CPU manual has the cycle diagrams and each device manual has its own, so
  this manual refers to them. A diagram of a CPU access to the audio window next to the sample schedule might be worth adding.
* No electrical specification of any net.
* The manual has had no review pass.
