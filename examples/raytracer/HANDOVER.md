# Raytracer example — handover notes

Goal: a SIRC assembly program that renders a 128x96 sphere to a PGM (P5
grayscale) image via a file-mapped memory segment, for use as the cover
image of the SIRC reference manual.

Scope rule from the project owner: **only modify files under `examples/`**.
Do not modify `sirc-vm` (simulator/toolchain) source. If a simulator/toolchain
bug or manual issue is found, stop and report it instead of patching it.

## Status

**DONE** — `raytracer.sasm` builds, runs, and produces a correct 128×96 sphere
PGM image. Committed and pushed.

## Known toolchain caveats (not bugs to fix here)

1. **`.DW` packing**: the assembler always advances the data offset by a
   full instruction-width slot (4 bytes / 2 words) per data token,
   regardless of the declared size (`.DB`=1, `.DW`=2, `.DQ`=4 bytes). So a
   `.DW` array's *logical* element `i` is not at `base + i`, it occupies a
   2-word slot: `base + i*2` (always zero, the padding word) and
   `base + i*2 + 1` (the actual value, confirmed empirically with isolated
   probes — see scratchpad probes in the session this was discovered).
   This matches the existing convention seen in `hardware-exception.sasm`
   and `basic-video/serial-handler.sasm` ("Increment by two because of lack
   of packing (a word actually is padded to a DW)"), except those examples
   only ever read/write addresses they computed themselves at runtime (so
   the +1 offset doesn't come up) — `sq_table` is the first case here that
   needs to read back **statically declared** `.DW` values by computed
   index, so the `+1` to land on the real (non-padding) word matters.
   - Fix applied: index sq_table as `base + 1 + index*2`, i.e. set
     the address register's low word to `sq_table_base + 1` once, then use
     `index * 2` as the displacement for every lookup.
   - Real fix (owner says they'll do this in a separate session): assembler
     should pack `.DW`/`.DB` tightly instead of padding every data token to
     instruction width.
2. **DMA instructions unimplemented**: `DMAT`, `DMAR`, `DMAW` are not
   implemented in the simulator. Do not use them in this example.
3. **Don't run the `basic-video` example** (per owner instruction, unrelated
   to this work — just avoid it during testing/exploration).
4. **`.ORG` placement / code-size budget**: the render loop is ~273
   instructions (2 words each), so code spans words 0x0200–0x0421. The
   `sq_table` must live above that; it is placed at `.ORG 0x0600` and `l`
   is initialised to `#0x0601` (base+1 for the +1 offset above). Do not
   move `sq_table` below 0x0450 without re-checking the instruction count.

## Plan / progress

- [x] Feasibility check, ISA review
- [x] Write `raytracer.sasm`: sphere distance test via `sq_table` +
      binary-search isqrt for shading, PGM header + pixel writes to a
      file-mapped segment
- [x] Diagnosed the `.DW` packing issue via isolated scratchpad probes
      (not committed — those lived in the scratchpad dir, outside the repo)
- [x] Applied `base + 1 + index*2` indexing fix to all `sq_table` lookups
      (6 call sites in the two isqrt loops + 2 for the direct d_sq lookups)
- [x] Fixed linker error caused by code growing past `.ORG 0x0400`;
      moved `sq_table` to `.ORG 0x0600`
- [x] Re-assembled/re-linked/re-ran; `render.pgm` shows correct sphere
      (smooth shading, circular boundary, black background)
- [x] Committed and pushed final working version

## How to build/run (for whoever picks this up)

```
cd examples/raytracer
make run        # builds raytracer.bin, truncates render.pgm, executes
```
`render.pgm`, `raytracer.bin`, `raytracer.bin.dbg`, `raytracer.o` are all
gitignored / generated — do not commit them.
