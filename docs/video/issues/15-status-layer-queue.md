# PPU-15: status layer and spare-slot queue (Mode 2, background 3)

**Milestone:** D. Isometric
**Depends on:** PPU-13, PPU-08
**Labels:** ppu

## Goal

Add background 3 to Mode 2 as an orthogonal layer fed from the fetch slots the isometric layers leave spare, through a queue of four groups.

## Spec

Video Device Reference Manual (`docs/video`):

- Chapter 6: The Status Layer (The Queue), Spare Slots, Tilemap Entry Fetches
- Chapter 8: Fetch Budget

## State on `main`

- Nothing.

## Tasks

- [ ] Orthogonal layer of 8 by 8 tiles (4-bit or 2-bit), 32 by 32 tilemap, no scroll; priority bit puts a tile in front of everything, otherwise it is behind everything; `B3D` disables it.
- [ ] A queue of four groups: in each cycle, if the slot is not needed by an isometric layer and the queue has room (group <= shown + 4), read a tilemap entry (every second group) or a tile word for the next group. Fill the first four groups in the front porch (6 reads).
- [ ] Count underruns (groups needed but not queued); there must be none.
- [ ] The queue depth is a hard requirement: an exhaustive study over all 16,384 pairs of isometric scroll positions needs depth 4.

## Acceptance criteria

- [ ] Test: random content x scroll cases against a reference, zero underruns.
- [ ] Test: an exhaustive model test of the needed depth over all scroll pairs (see the reference).
- [ ] Negative test: a queue of two groups underruns.

## Reference

Reference implementation (uncommitted, branch `claude/video-chip-isometric-mode-f565fa`): `hud_*` functions in `lib.rs`; `study_status_layer_queue_depth` and `status_layer_matches_reference_and_the_queue_never_runs_dry`.
