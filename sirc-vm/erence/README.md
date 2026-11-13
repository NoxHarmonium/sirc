# ERENCE

**E**vent **R**ecording, **E**diting, **N**otation and **C**omposition **E**nvironment: a tracker-style terminal editor
for SIRCAS sequences, part of SIRCUMF (the SIRC Unified Music Framework).

You write music in a table of rows and six channel columns, set up the instruments, and play it back through the SIRCAS
reference model. You export it as SIRC assembly that links into a program, like sirc-tiledit does for graphics.

```bash
cargo run --release -p erence -- song.erence          # edit (starts a new song if the file does not exist)
cargo run --release -p erence -- --render out.wav song.erence
cargo run --release -p erence -- --export out.sasm song.erence
```

## Using it

| Key | Pattern tab (F1) |
| --- | --- |
| `z s x d c v g b h n j m` | Notes, one octave from C. `q 2 w 3 e r 5 t 6 y 7 u i 9 o 0 p` is the octave above |
| `1` | Note off (`===`, a `STOP`) |
| `[` `]` | Octave down / up |
| `;` `'` | Current instrument down / up |
| Arrows, Tab, PgUp/PgDn, Home/End | Move. Left and right step through the note, instrument and fine-pitch fields |
| Digits | In the instrument field, hex; in the fine field, decimal (`-` makes it negative) |
| Delete or Backspace | Clear the field |
| Insert, Ctrl-D, Ctrl-A | Insert a row, delete a row, append 16 rows |
| `l` | Loop the song back to this row (press again to remove the loop) |
| `\` and `/` | Mute / solo the channel |
| Space, F5, F6, F7 or Esc | Play, play from the top, play from the cursor, stop |

F2 is the instrument editor: choose a generated waveform (sine, saw, square, triangle, pulse, noise) or import a WAV
file, and set the envelope, loop and pitch divider. Piano keys audition the instrument. F3 holds the song settings: tick
length, speed, master volume, the filter, and each channel's volume, pan, filter routing, portamento and modulation.

Ctrl-S saves, Ctrl-O opens, Ctrl-E exports assembly, Ctrl-W renders a WAV, Ctrl-Q quits.

Playback renders the song through the model first (about 35 times faster than real time), then plays the result, so
there is a short pause before the sound starts and a looping song repeats from the loop row without a gap. Edits made
while it plays are heard the next time you press play.

## How a song becomes SIRCAS code

A row lasts `speed` sequencer ticks, and a tick is `tick` output samples. Each channel column compiles to one
instruction stream: a note on becomes `INST`, `FINE` and `NOTE` (only the ones that changed), a note off is `STOP`, and
empty rows are merged into `WAIT`s (split at the 8191-tick limit). A loop ends the stream with a `GOTO` back to the loop
row; without a loop it ends with `END 0`. The tracker has no pattern re-use, because the chip's sequencer has none: a
repeated bar is repeated in audio memory, and the status line shows how much of the 64 KB is used.

At the loop row the compiler forgets which instrument and fine value are selected, so the first note after it states
them again and the loop plays the same on every pass.

An instrument needs a pitch table for its waveform length. Identical tables are shared. The pitch divider (`Pitch div`)
slows an instrument down by a whole factor: 5 makes note A0 sound at 5.5 Hz instead of 27.5 Hz, which is how a
low-frequency modulation source is made. Set a channel to modulate pitch, volume, pan or the filter cutoff on the song
tab to turn it into one.

## Exporting

`--export` (or Ctrl-E) writes two tables of 16-bit words, packed two to a `.DQ` slot so they are consecutive words in
memory:

* `:erence_image`: a count, then the audio memory words to copy into the device window at offset `0x8000`.
* `:erence_regs`: a count, then (register offset, value) pairs to write to the device in order. The last writes start
  the sequencers.

`examples/erence` has a loader (`erence_load`) that copies both and a program that waits for the sequences to finish.
`make check` there exports a song, links it, runs it in sirc-vm with `--audio-wav`, and compares what the device played
with ERENCE's own render.

## The song file

Songs are plain text so they diff well. The first line is `ERENCE 1`, then settings (`tick`, `speed`, `master`,
`filter`, `loop`, `channel`, `inst`), then a `rows` section with one line per row and six cells separated by `|`. A cell
is `note instrument fine`: `C-4 01 +0000`, with dots for an empty field and `===` for a note off. Imported samples are
stored as paths relative to the song file. See `examples/erence/demo.erence`.

## Structure

* `sirc-umf`: `song` (model and file format), `compile` (rows to sequences, audio memory image, render, assembly
  export), `wav` (sample import), plus the SIRCAS model itself.
* `erence`: `app` (state and keys, testable without a terminal), `ui` (ratatui drawing), `audio` (cpal playback).

## Limits

* Notes, instrument and fine pitch are the only events. Portamento, volume, pan and the filter are per channel
  settings, not per row; the filter cutoff does not change during a song unless a modulation voice drives it.
* Samples are limited to 65,535 bytes after decimation and share the 64 KB of audio memory with everything else.
* The chip is still a draft; ERENCE follows the reference model, so spec changes will need a matching change here.
