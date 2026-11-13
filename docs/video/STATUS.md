# SIRCADA status and open issues

This is the working record for the SIRCADA manual (`docs/video`): what the reference simulator implements, where it
disagrees with the manual or with the older design notes on the project wiki, and the design questions that remain.
It used to be the last chapter of the manual. It lives here instead because the manual is written as a period hardware
reference and must not mention simulators, prototypes, wikis or project status in its chapters. Read this before relying
on anything else in the manual.

The manual describes the intended behaviour. The simulator is `sirc-vm/device-video`; the reference implementation
lives on the branch `claude/video-chip-reference-impl`.

## Implementation status

| Feature | Status | Notes |
| --- | --- | --- |
| Registers, palette and VRAM access | Implemented | Open bus reads zero |
| Package, pins, signal timing, video output | Specified | The simulator has no pins; chapters Package and Pinout, Signal Timing |
| External video memory bus; wait state on video memory reads; entries-before-words slot order in Modes 2 and 3 | Not yet implemented | The simulator has video memory on the device, no wait states and the old slot order |
| Mode 1 backgrounds, 8 by 8 tiles, 32 by 32 maps | Implemented | Layer order corrected with the sprite prototype |
| Mode 1 background 4, register map repacked into groups of four | Not yet implemented | The simulator has three backgrounds and the old register offsets; `STATUS` is at `0x0017`, not `0x0019` |
| Mode 1 scrolling (specified) | Not yet implemented | Registers stored |
| Mode 1 16 by 16 tiles, double tilemaps (specified) | Not yet implemented | Registers stored |
| Orthogonal tile flips (specified), priority bit | Not yet implemented; priority in Mode 3 and the status layer | |
| 2-bit tile data (Mode 1) | Not yet implemented | |
| Graphics disable, brightness (specified) | Not yet implemented | Bits stored |
| Background disables | Modes 2 and 3 only | |
| Sprites, sprite disables and sizes | Prototype | Proposed, both modes; chapter Sprites |
| Vertical sync interrupt (NMI) | Implemented | At line 224; on level 4, not the NMI yet |
| Vertical blanking flag and current line register | Implemented | `VBL` in `STATUS`, `VLINE` at `0x0018` (`0x001A` in the repacked map) |
| PAL timing and the output mode flag (specified) | Not yet implemented | |
| Blanking-only VRAM access (specified) | Not yet implemented | Not enforced |
| Forced blank, graphics disable (specified) | Not yet implemented | `GD` stored only |
| Mode 2 | Prototype | Proposed; chapter Mode 2 |
| Mode 2 status layer (background 3) | Prototype | Proposed; section The Status Layer |
| Mode 2 2-bit isometric tiles | Prototype | Proposed; section 2-bit Tiles |
| Mode 3 | Prototype | Proposed; chapter Mode 3 |
| Isometric shading, shadow sprites | Prototype | Proposed; section Shading |
| 2-bit orthogonal tiles | Mode 3 and Mode 2 only | Not yet implemented in Mode 1 |

## Discrepancies

Known differences between the manual's source, the simulator, and its other sources. Each is a defect to be fixed in the
simulator or a decision to be made; the manual describes the intended behaviour.

1. **Register addresses.** The wiki puts the sprite tile data base at offset `0x0012` and the status at `0x0013`. The
   simulator puts the three background palette configuration registers at `0x0012`–`0x0014`, the sprite tile data base at
   `0x0016` and the status at `0x0017`. The manual followed the simulator until the register map was repacked into groups of four (see the table above), so the simulator's offsets now differ from the manual's.
2. **Palette boundary** (fixed). The simulator used to decode palette offsets `0x6000`–`0x6100` inclusive, one past the
   end of its 256 entries; it now stops at `0x60FF`.
3. **Mode 1 layer order** (fixed with the sprite prototype). The simulator used to show background 3 in front of
   background 1; it now uses the intended order.
4. **Mode 1 palette sum** (fixed with the sprite prototype). The simulator added the palette offset and the tile value
   without wrapping, so a large offset could stop the simulator in a debug build; it now wraps, as Mode 2 does.
5. **Status byte order** (fixed). The simulator read the Status register with its bytes swapped relative to every other
   register; it no longer does.
6. **Vertical sync position** (fixed). The interrupt used to be asserted at line 0; it is now asserted at line 224, the
   start of vertical blanking.
7. **Mode 1 picture offset.** The simulator's Mode 1 shows the first 8 pixels of every line blank and draws the tilemap
   8 pixels to the right of its position, so the last tile column is never shown. The first two groups of a line are
   flushed before any data has been fetched.
8. **Clock accounting.** The simulator polls every device once per `CLKI` cycle, which the video device counts as two
   master clocks; the terminal device counts one poll as one master clock, so its baud rate is off by a factor of two.
9. **VRAM access during rendering.** The specification ignores accesses to video memory and the sprite table outside
   vertical blanking; the simulator allows them.
10. **Blank first line.** The manual makes line 0 always blank (black, no background fetches), with the sprite scan and fetch for line 1 done during it. The simulator draws line 0 as an ordinary line and has not been checked for the sprite preparation timing.

## Open design questions

### General

- **Interrupts.** Specified: vertical sync on the `NMI#` with no acknowledgement, line compare on `IRQ4#`, held until a
  write to `LINE`, both enabled from the Interrupt Control register. The simulator still pulses vertical sync on level 4
  and line compare on level 3 for one cycle, with no enable for vertical sync. A one-cycle NMI pulse is one `CLKI` cycle,
  which the CPU's edge detector accepts.
- **The wiki.** The project wiki's register map (the sprite tile data base and status register offsets) disagrees with the
  manual and the simulator, and should be updated from the manual.
- **Version and region.** `VER` is 1 and the region is fixed when the device is made; the simulator should let both be
  configured at run time.
- **Package and signals.** The package, the pin assignment and the signal timing are a first sketch, to be confirmed: the
  100-pin QFP with an external video memory bus; a chip select instead of the segment address bits; `CLKI` as the only clock; `BACK#`, `NMI#` and `IRQ4#`
  as open-drain outputs; no access-type or protection inputs; a minimum reset of six cycles; the composite sync widths (a
  horizontal pulse of 100 master clocks; vertical sync in three lines, 245 to 247, one long pulse in each, no equalising
  pulses); and the converter and supply pins. The palette and sprite table are on the die and the video memory is external.
  Electrical characteristics (levels, delays, frequency range, current, thermal) are not yet specified, as in the
  hardware manual.
- **DMA.** A DMA transfer into video memory is subject to the same rule as the CPU, and DMA blocks interrupts, the `NMI#`
  included, so programs keep transfers short and away from line 224.

### Mode 2

- **Whether to approve it.** Mode 2 narrows the device to one kind of picture. That is its purpose, but it is a decision,
  not a default.
- **Sprites.** With no free fetch slot, Mode 2 sprites come from a line buffer filled in the blanking interval; the
  prototype shows that 32 sprites per line of either size fit. Whether 32 is enough for a vehicle-heavy game, and whether
  to pay for a second line buffer to double it, is open.
- **The status layer.** Prototyped with a queue of four groups. Open: whether it should scroll, whether its tiles can be
  flipped, and whether to use its slots for a third isometric background instead.
- **2-bit tiles.** Prototyped. Left to the program: the palette size with `T2`. Open: the Mode 1 implementation.
- **Mode 3.** Prototyped. Open: whether the orthogonal backgrounds should have a priority bit to draw in front of
  sprites, and whether another arrangement of orthogonal and isometric backgrounds is wanted.
- **Shading.** Prototyped. Open: whether darkening should have more than one level, and whether a shadow pixel should
  darken sprites behind it as well as backgrounds.
- **The tilemap entry cache.** Measured and left out; worth revisiting if the spare slots are wanted for something else.
- **Rotation of the view.** Ruled out as a register; to show another facing a program needs another set of tiles and map.
- **The symmetric diamond.** The footprint in the manual's footprint table is one pixel narrower at the top than the
  bottom. A conventionally symmetric diamond does not give a bijection (some texels would be shown twice and some never).
  The manual keeps the footprint that does. The slicing tool accepts any picture, but diamonds that are to tessellate must
  be drawn to that footprint.
- **The fixed tilemap size.** A 32 by 32 map is 512 by 512 texels; a transport game would want a larger map and would page
  it by rewriting the tilemap.
