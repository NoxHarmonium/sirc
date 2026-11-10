# Video device design notes

Background for the people building the video device. **This is not part of the manual**: the manual
(`SIRCADA-reference-manual.pdf`, sources in `chapters/`) is written as a period hardware reference and states what the device
does without comparisons or history. This file keeps the rationale, the precedents and the measurements behind it, so the manual does
not have to.

The numbers below were measured with the reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`,
`sirc-vm/device-video`: `iso.rs`, `sprite.rs`, `iso_tests.rs`, `sprite_tests.rs`). The test name that measures each is given so it can
be rerun with `cargo test -p device_video -- --nocapture`.

## Intent

SIRCUS (the SIRC User System) is a fictional console that was very good at one thing and poor at the general case, which is why it faded. The video device
is built around that: Mode 1 is a conventional four-layer tile mode, and Mode 2 is an isometric mode for orthogonal strategy and simulation
maps (the Transport Tycoon / SimCity kind). The weakness comes from the same decision as the strength: Mode 2 fetches a tile word
for nearly every pixel group, which leaves only two isometric layers and no sprite slots in the visible part of a line.

## Precedents

Recalled from memory, not checked against sources.

- **Super Nintendo PPU** (the device's loose model): per-mode fixed fetch-slot schedules (more layers means fewer bits per pixel),
  128 sprites with 32 per line and 34 tile slivers per line, sprites fetched in the horizontal blanking interval, a forced-blank bit that
  opens video memory to the CPU, a separate 4-bit master brightness (15 = full), the NMI for vertical blank and a separate timer IRQ,
  each acknowledged by reading a flag register.
- **Mega Drive VDP**: shadow/highlight (the model for the shading bits and the shadow sprite colour), line and cell scroll, a small
  FIFO for CPU writes (a small queue is period-plausible).
- **Neo Geo fix layer, Game Boy window layer**: an orthogonal status layer is not a new idea (the Mode 2 status layer, Mode 3's orthogonal layers).
- **Amiga copper, SNES HDMA**: table-driven mid-frame register changes. Not adopted; the line-compare interrupt covers raster effects
  with far less hardware.

## Mode 2 geometry and tile format

- Mapping `u = y + (x >> 1)`, `v = y - ((x + 1) >> 1)` is a bijection between pixels and texels (`pixel_to_texel_is_a_bijection`),
  with `u - v = x`. A 16 by 16 texel tile is a diamond 31 pixels wide and 16 high (rows 1, 5, ..., 29, 31, 27, ..., 3 wide). A symmetric
  2, 6, ..., 30, 30, ..., 2 diamond is not a bijection, so the footprint is slightly lopsided. It is 2:1 dimetric, not a 45 degree rotation.
- A tile is 16 by 16 texels at 2:1, i.e. the SimCity 2000 scale (32 by 16 pixel diamonds), half the linear size of Transport Tycoon's
  64 by 32.
- Plain row storage would make four neighbouring pixels sample four different rows (four words per group). The **staircase** format
  stores the four texels `(a,b) (a,b-1) (a+1,b-1) (a+1,b-2)` that a group of four samples in one word, so a group costs one word. 64
  words per tile, perfect packing (`staircase_is_a_perfect_packing`), a group touches at most two cells and uses the same word number in
  both (`group_never_touches_more_than_two_cells_and_shares_one_word_index`).
- **Why a group straddles two cells**: cell edges are diagonal and move two pixels per line, so an edge falls inside a group of four about
  three times in four, one edge about every 16 pixels: 3/16 = 18.75% of groups. Geometry, not data; no tile encoding removes it. Forcing
  edges onto group borders would need 4:1 slopes (too orthogonal) or 2-pixel groups (half of every word wasted).
- Flips cannot be supported by the staircase format (a mirrored staircase is a different shape), so the flip bits mean shading in
  isometric layers.
- **Rotation of the view is not free**: the staircase direction is baked into the data, so each facing needs its own tiles. (An early
  claim that it was nearly free was wrong.)
- Authoring: `bake_layer` slices a screen-space picture into tiles with the layer's scroll; the demo scene round-trips with zero differing
  pixels (`demo_scene_baked_from_screen_space_art_renders_exactly`), ground 15 tiles and buildings 28 (empty tile included).

## Fetch budget

- 8 PPU cycles per group of four pixels; Mode 2 gives each layer four slots (entry A, word A, entry B, word B); B slots only when the
  group straddles. Average 4.75 of 8, worst case 8, so no sprite slots in the visible part of a line.
- Fine X scroll costs nothing: groups are aligned to the plane, a two-group window holds the pixels, and the line starts at offset
  `scroll_x & 3`. Two groups are prefetched in the back porch, 32 master clocks before the first pixel.
- Spare slots: 192 to 224 of 512 per line (mean 208), depending only on scroll
  (`mode_2_visible_slots_left_over_for_sprites`).

### Tilemap entry cache (rejected)

Keeping each layer's last tilemap entry cuts a layer to 88-97 reads per line (mean 92.9) from about 152, two layers to 2.9 of 8 slots per
group, and spare slots to 320-336 per line (`study_tilemap_entry_cache`). But the status-layer queue only shrinks from 4 groups to 3
(`study_status_layer_queue_depth`), about 20 bits, while the cache costs a 16-bit entry, a 10-bit cell number and a comparator per layer,
roughly 55 flip-flops plus logic. Not worth it; a third isometric layer would fit with the queue at depth 3 (2 with the cache) over all
2,097,152 scroll combinations (`study_third_isometric_layer`, release build, `--ignored`) but that is a different feature.

### Status layer queue

An orthogonal layer in Mode 2 takes the slots the isometric layers leave spare. They are not evenly spread, so it runs through a queue. Exhaustive
over every pair of isometric scroll positions (a layer's straddle pattern repeats every 32 plane pixels in x and 16 lines in y, so 128
patterns per layer, 16,384 pairs): required depth 2 for 14,464 pairs, 3 for 1,872, 4 for 48. Depth 4. Two groups fails
(`status_layer_matches_reference_and_the_queue_never_runs_dry`, negative control by shrinking `HUD_QUEUE_GROUPS`).

## 2-bit tiles

Isometric 2-bit tiles pack perfectly in 32 words with groups of eight (a group of eight never touches more than two cells; 43.75% touch
two). A 2-bit layer needs 56-65 reads per line (mean 62.9) against 88-97 for 4-bit (`study_two_bit_layer_reads_per_line`). The ground
of the demo scene in three colours: 14 tiles, 448 words against 896. The bit is `T2` in each layer's palette register; leaving `SIZE` at
four colours is the program's choice (it would only prevent wasted palette space).

## Sprites

- Line-buffer design: scan the table during the visible time (internal RAM), fetch rows in the horizontal blanking interval, paint into a
  256-entry buffer. 170 porch cycles minus 22 prefetch = 148 reads: 74 sprites of 8 by 8 or 37 of 16 by 16 fit, so 32 per line (a register
  limit, as in the SNES) is never time-limited (`capacity_per_line`). Using the visible slots as well needs a second line buffer and was
  not adopted (Mode 1 has no spare slot now it has four backgrounds, Mode 2: spare slots).
- Keeping only the frontmost sprite pixel per column is enough for the S1/B1/S2/B2/S3/B3/B4 order.
- Entry: 8-bit tile number because X (9 bits), Y (8), flips (2), palette (3) and layer (2) leave eight bits.
- Shadow colour (palette 7, value 15, enabled by `SHD`): a shadow never covers a drawn sprite pixel and is always replaced by one, so
  shadows darken backgrounds only. Darkening does not accumulate (one level).
- 16 by 16 sprite tiles use the same quadrant layout as 16 by 16 background tiles so the asset tool exports one format.

## Timing and interrupts

- `CLKI` = master clock / 2: that is how the simulator already polls (one `poll_all` = one CPU phase = one PPU cycle). At 21,477,272 Hz
  that is 10.74 MHz, 6 cycles per instruction (1.79 MIPS), 113 instructions per line, 25,916 cycles of NTSC vertical blanking.
- A one-cycle NMI pulse is one `CLKI` cycle; the architecture manual latches an edge active for at least one clock in every CPU state
  including during DMA, so a DMA transfer delays the NMI rather than losing it. A `DMAT` of 255 words is 522 cycles (about 0.77 line).
- Registers are sampled 228 master clocks (114 cycles) into the line; interrupt latency is 6 to about 12 cycles, so a line-compare handler
  has about 100 cycles (some 16 instructions) to change a register for the same line.
- Vertical sync on the NMI needs `VIE` because the NMI cannot be masked. Line compare is IRQ4 held until a write to `LINE` (the hardware
  manual's IRQ inputs are level-sensitive, so the device must hold the line).
- Forced blank (`GD` = 1) opens video memory to the CPU because the renderer makes no reads; reset leaves `GD` = 1 so a program can load
  graphics immediately (the `basic-video` example already does). Outside forced blank only vertical blanking is safe: the sprite fetch uses
  almost all of the horizontal blanking interval.
- Brightness `(BRT + 1) / 16` follows the SNES; `GD` blanks fully.

## Package, pins and signal timing

Sketched, not yet confirmed (also listed in `STATUS.md`, the working record):

- **Single package, one manual.** The CPU has separate architecture and hardware manuals because the architecture could have several
  implementations. The video device has one, so its pinout and timing are in its own manual (Chapters 2 and 11).
- **Everything on one die.** Video memory (64 KB), palette, sprite table and DACs are on the package. External video memory would
  need about 35 more pins and a wait-state story for the CPU; the manual's "no wait states, every access completes in its cycle"
  is only simple with on-chip memory. (On-chip 64 KB of SRAM is generous for the period; a fictional console can have it.)
- **52-pin PLCC.** Signal count: bus 36 (A0-A15, D0-D15, CS#, BAS#, BRW, BACK#), clock and reset 2, interrupts 2, video 5
  (R, G, B, CSYNC#, IREF), power 6 (VCC1/2, GND1/2, AVCC, AGND) = 51, plus one NC. Digital and analog supplies are separate,
  as a period video DAC would have.
- **CS# instead of segment address pins.** The manual already says the system chooses the segment; a chip select keeps the device
  independent of the address map. No BAT or PROT inputs: the device treats every cycle type alike.
- **CLKI is the only clock.** Every internal interval is a whole number of PPU cycles (= CLKI cycles), so no master-clock pin is needed.
- **BACK#, NMI#, IRQ4# open drain** because the manual says IRQ4 may be shared and BACK may be joined with other devices.
- **Reset hold of six cycles** matches the CPU's RSTO# hold, so RSTO# can drive RST# directly.
- **Composite sync, aligned to the NESdev "NTSC video" page.** CSYNC# low for 100 master clocks (25 pixels, 4.66 us) at the start
  of each line, as the NES hsync. Line order there is picture, right border, front porch, hsync, back porch, left border, so the
  front porch is the part *before* the sync pulse and the back porch is the part between sync and picture. Chapter 10 therefore
  calls the 260 clocks at the start of the line "sync and back porch" (100 sync, 96 back porch where a colour burst would sit,
  64 black border) and the 80 clocks at the end the front porch (44 black border, 36 front porch); the NES line has the same
  85 pixels of blanking and border. The device generates no colour burst (RGB output). Vertical sync copies the NES: three
  lines, one long low pulse in each (318 pixels = 1272 clocks low, 92 high), starting at the hsync position, ending 14 lines
  before the first picture line (NES 245-247 of 262; here 245-247 NTSC, 295-297 PAL). No equalising pulses, as on the NES.
  Our 224-line picture leaves 38 blank lines instead of 22; the extra 16 sit at the bottom as overscan. The earlier
  guess (lines 240-242, broad pulses of 586/96) is gone. The NES also drops a dot on odd frames; this device does not
  (every frame is 262 or 312 whole lines).

## Mode 3 and the slot arithmetic

Orthogonal layers use 2 slots per group (entry, word), an isometric layer up to 4: two orthogonal plus one isometric is 4 + 4 = 8 slots, exactly the
budget with fixed ownership and no queue. The isometric layer is background 3 so that the orthogonal layers sit in front in the natural order
S1, B1, S2, B2, S3, B3. Typical use 6.4 of 8.

## Defects in the original code found on the way

See the tracking issue draft (`issues/00-tracking-issue.md`) for the list measured against `main` (palette off-by-one, status byte
order, Mode 1 layer order and unchecked add, the 8-pixel picture offset in Mode 1, unrestricted video memory access, terminal device clock
accounting).

## Comparison with the official Super NES manual (2026-11-07)

Read from the text of "Super NES Development Manual, Book I" (archive.org `snes_manual1`): Section 2 chapters 2-1 to 2-28 and Appendix A (PPU registers).
Its PPU chapters (OBJ, BG, Mosaic, Rotation, Window, Main/Sub Screen, ...) each use Outline / Function / Setting Example, with an initial-settings flow, then
cross-cutting chapters on CPU clock and address map, functional operation, system flowchart, numbered programming cautions, documented problems, register
clear (initial settings), and the register reference with bit diagrams, `<2105H>`-style cross references and NOTE callouts.

What we already match: register reference with bit diagrams and reset state, block diagrams, per-feature limits (sprites per line with an overflow flag),
VRAM access rules, line and frame timing diagrams, interrupt behaviour, a memory map. What the comparison added, now in the manual (plus the flag below):
- Timing in time units as well as clocks (Table "Video Timing"; the Super NES manual gives 63.5 us per line, which our line also is).
- A programming chapter: start-up order (their "Register Clear" and "System Flowchart"), a frame flow figure, and numbered programming cautions.
- "Programming Mode 1" and "Programming Sprites" setting examples, as Mode 2 and Mode 3 already had.

Found missing and added (2026-11-07): the manual had no vertical-blanking flag and no readable beam position, so a program that did not use the NMI could not
poll. The Super NES has the NMI flag (`$4210`), the blanking status (`$4212`) and the H/V counter latch (`$2137`, `$213C`, `$213D`). SIRCADA now has `STATUS` bit 7
(`VBL`, set lines 224 to the end of the frame) and a read-only `VLINE` register at 0x0018 (current line, 8-0). No horizontal position or latch: the beam position within a
line is not needed for anything the device allows the CPU to do, and a latch would need a trigger input the package does not have.

Features of that chip that SIRCADA deliberately does not have, so the manual does not mention them: interlace and 512-wide modes, mosaic, windows, colour
add/subtract (transparency), Mode 7 rotation/scaling, H-DMA (the line-compare IRQ4 is the substitute for per-line register changes), per-column offset change,
hardware multiply/divide (the CPU has its own). Whether any belongs on a niche chip is a design question, not a documentation gap. Transparency/colour
math and a window are the two a user is most likely to ask about.

Period-style details worth keeping in mind for later chapters: the Super NES manual gives reset values and read/write per register in a summary table,
marks forbidden values ("prohibited"), and gives timing in microseconds; and it lists documented problems (our `STATUS.md` plays that role).

## Mode 1 fourth background and the register repack (2026-11-11)

- Mode 1 had two idle fetch slots per group of four pixels (slots 6 and 7): the eight slots are sized for the worst case of the isometric
  modes (two layers at four slots each), and Mode 1 needs only six. A background costs one tilemap entry and one tile word per group, so a
  fourth background fits exactly in the idle pair, at any bit depth. Nothing else could use them: the CPU is locked out of video memory
  outside vertical blanking and sprites are fetched in the blanking intervals into a line buffer. Mode 1 now uses all eight slots.
- Fixing the new layer at 2 or 1 bit per pixel was rejected. It does not make the layer fit (4-bit already fits), saved reads in alternate
  groups are unusable in a fixed schedule, and 1-bit tiles would be a new tile format. The layer is a normal background with the `T2` bit,
  so the four layer engines are identical. Five backgrounds would need 10 reads per group and do not fit without a tilemap entry cache.
- Background 4 exists in Mode 1 only; Modes 2 and 3 do not draw it, and its registers are ignored there. In priority order it is last
  (behind B3). `B4D` is `CONFIG` bit 11 and `B4S`, `B4X`, `B4Y` are the spare bits 12, 4 and 0 of `TILESIZE`.
- Register map repacked into groups of four (MAP, TILES, SCRX, SCRY, PAL) so every register for background n is at group base + n - 1.
  `INTCTL` moves from `0x0011` to `0x0016`, `LINE` from `0x0015` to `0x0017`, `STILES` from `0x0016` to `0x0018`, `STATUS` from `0x0017` to
  `0x0019` and `VLINE` from `0x0018` to `0x001A`. The simulator, its tests and the wiki still use the old offsets.
- Era check: the Super NES offered a three-layer mode with one 2-bit layer and a four-layer mode of 2-bit layers only, so four 4-bit layers
  is more than that machine did. The trade is die area and cost: four pixel pipelines and the fetch bandwidth to feed them. In the
  fiction SIRCADA is simply the more expensive part, which fits a CPU maker using up surplus stock in a console.
