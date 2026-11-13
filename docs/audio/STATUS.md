# SIRCAS status, findings and open questions

Working record for the future SIRCAS manual (`docs/audio`). Like `docs/video/STATUS.md`, it holds everything that must
stay out of the manual's chapters: what the reference model implements, which open points the model resolved and how,
and what it measured. The source of truth for the intended design is the draft specification
(`SIRCAS-draft-specification.md`, draft 0.1); this file records where the prototype confirms, refines or disagrees with it.

## The prototype

`sirc-vm/device-audio` is a behavioural reference model, **clocked at `CLKI` resolution** (one bus `poll` = one `CLKI`).
It is not RTL and produces no sound on the host yet; it captures the stereo output frames for inspection.

| Part | File | What it models |
| --- | --- | --- |
| Engine | `src/core.rs` | 8-clock SRAM schedule (audio slot + three CPU slots), 336-clock frames, registers, sequencer, instrument/SPIT loads, PCM fetch, 12-bit envelopes (staggered), portamento, modulation, mixer, DSP cycle accounting, `MEM_OVERRUN`, `IRQ3` |
| Filter | `src/filter.rs` | Chamberlin state-variable filter, fixed point with selectable width, fractional bits and rounding, plus a float reference |
| I²S | `src/i2s.rs` | Frame serialiser/deserialiser for any BCLK divider and slot width |
| Bus | `src/lib.rs` | `AudioDevice`: registers acknowledged at once, RAM held for a CPU slot, `IRQ3` on the bus |
| Test kit | `src/testkit.rs` | Sequencer assembler, waveforms, pitch tables, RAM builder |
| Tests | `tests/acceptance.rs`, `tests/bus.rs` | All twelve acceptance tests of draft section 16, plus the real bus path |
| Studies | `examples/filter_study.rs`, `examples/budget_study.rs` | The measurements quoted below |

Run it:

```bash
cd sirc-vm
cargo test -p device_audio
cargo run --release -p device_audio --example filter_study
cargo run --release -p device_audio --example budget_study
cargo run -p sirc_vm -- --enable-audio ...   # maps SIRCAS at 0x000D0000, 0x10000 words
```

## Acceptance tests (draft section 16)

| # | Test | Result |
| --- | --- | --- |
| 1 | RAM slot determinism | **Pass.** 3000 random-phase CPU accesses give identical latencies idle, with six voices playing, and with the sequencer overloaded. CPU RAM latency is 2 to 5 `CLKI`. |
| 2 | PCM deadlines | **Pass.** Six channels at 3.75 bytes/sample over 2-byte loops, 4000 frames: no misses. PCM needs at most 6 of the 42 audio slots per frame. |
| 3 | Burst of six `INST;FINE;NOTE;WAIT` | **Pass.** Worst tick-to-note latency 568 `CLKI` (1.7 frames, 53 µs). No overrun at `SEQ_PERIOD` 666. |
| 4 | Overload | **Pass.** Sticky `MEM_OVERRUN`, dropped ticks counted, CPU slots untouched, no PCM miss. |
| 5 | Sequencer | **Pass.** `WAIT`, signed `GOTO`, 8-instruction cap, `END 0`, counted `END`, `SEQ_PC` rewrite, `IRQ3` level and W1C. |
| 6 | Direct notes | **Pass** under the policy chosen below. |
| 7 | ADSR and portamento | **Pass.** Updates every 64 frames per channel at six distinct phases; exact `ATTACK` step; glide once per tick without retrigger. |
| 8 | Modulation | **Pass.** Lowest source wins; mutual feedback is deterministic. |
| 9 | Filter | **Pass** for determinism and saturation. Numerics: see findings. |
| 10 | DSP cycles | **Pass.** Maximum 276 of 336 with every cached gain invalidated every frame. |
| 11 | Output formatting | **Pass.** Clips at +32767 / -32768; I²S round trip is bit-exact for both formats tested. |
| 12 | Reset | **Pass.** |

## Findings

### Memory scheduling is comfortable

* The 1:3 slot ratio gives the audio engine 1.34 M transactions/s and the CPU 4.03 M words/s (8.1 MB/s).
* PCM needs at most one word per channel per frame, so six of 42 slots. The sequencer always has at least 36.
* Smallest sustainable `SEQ_PERIOD` with all six channels running the same program:

| Program | Min `SEQ_PERIOD` | Max tick rate |
| --- | ---: | ---: |
| Worst case: four `NOTE`s per tick, different instrument each time | 7 | 4.6 kHz |
| `INST;FINE;NOTE;WAIT 1`, instrument cache on or off | 2 | 16 kHz |
| Portamento `NOTE;WAIT;NOTE;WAIT` | 2 | 16 kHz |

  A realistic tick (50 to 200 Hz) uses well under 1 % of the audio slots. The instrument cache halves the control reads
  but does not change feasibility, so the tag comparison could be left out of the hardware.
* CPU latency to RAM is 2 to 5 `CLKI` whatever the audio does, and register accesses cost no wait.

### The DSP budget has headroom

Under the model (shift-add multiplier = coefficient bits + 2 clocks, filter block always running):

| Scenario | Model | Draft estimate |
| --- | ---: | ---: |
| Six dry | 216 | 286 |
| Six filtered | 156 | 238 |
| Three dry, two filtered, one modulator | 186 | 263 |
| Five dry, one volume modulator | 206 | 289 |

Add 10 clocks per channel whose cached gain is recomputed in that frame (worst case 60, giving the 276 maximum above).
The model is less pessimistic than the draft, so its absolute numbers are a lower bound. Even adding the draft's overhead
the sums stay below 336, leaving roughly 50 to 120 clocks to spend.

### The filter

* **Topology.** Chamberlin with `f = cutoff/256` reproduces the draft's 5.3 kHz maximum exactly
  (`fc = fs/π · asin(f/2)`). Update order used: `low += f·band; high = in − low − q·band; band += f·high`.
* **Cutoff resolution is the weak point.** `f` is linear in the register, about 20 Hz per step. One register step is an octave
  at the bottom (1 to 2), a fifth from 2 to 3, a semitone at 16, 0.5 semitone at 32. Cutoff sweeps below about 600 Hz will
  step audibly, and an 8-bit modulation target makes it worse.
* **Resonance** behaves as intended. `Q` runs from 1.2 (R = 0) to 16 (R = 254), peaking +24 dB. `R = 255` (zero damping) holds a
  constant-amplitude oscillation for the full 5 s tested at every cutoff and every rounding mode, with no decay or growth.
* **Quantisation.** Four fractional bits are acceptable. SNR on a 440 Hz sine:

| Cutoff reg | Floor, 4 bits | Nearest, 4 bits | Floor, 8 bits |
| ---: | ---: | ---: | ---: |
| 8 | 54 dB | 75 dB | 78 dB |
| 32 | 87 dB | 99 dB | 111 dB |
| 128 | 103 dB | 99 dB | 127 dB |

  The worst case is a low cutoff with bare truncation (54 dB).
* **Truncation bias is small.** After a loud impulse and 5 s of zeros, the output settles within 5 output LSBs (bare shifts), 2 LSBs
  (round to nearest). There is no runaway drift.
* **Dead zone.** At very low levels and a low cutoff, truncation toward zero swallows the signal (a 4-LSB input at register 4 is
  lost entirely). Round-to-nearest keeps it. Bare floor gives a spurious +2.9 dB.
* **Saturation** works as specified. A full-scale square wave into `R = 255` saturates the states thousands of times but never
  wraps and stays bounded.
* **Cutoff 0 freezes the state.** The low-pass output holds its last value forever (a sample-and-hold). That is deterministic,
  but a program that parks the cutoff at 0 can latch a DC offset. See the questions.
* **Width trade-off.** 24 bits with 4 fractional bits leaves 20 integer bits, enough for the sum of six full-scale channels.
  Eight fractional bits in 24 bits leaves only ±32768 of headroom, which the six-channel sum exceeds. Better numerics need a
  wider state (28 bits).

### Pitch law: raw additive units are not musical

* `FINE` as raw 16.16 units added after the octave shift spans ±1313 cents at note 0 but only ±2 cents at note 119. A fixed
  register value means a different detune in every octave.
* Applying `FINE` before the octave shift gives the same ±1313 cents (more than an octave) in every octave, with 0.25 to 0.48 cents
  per step. Fine resolution, but a much larger range than detuning needs.
* Additive pitch modulation (`M << (S+4)`) at `M = 127, S = 0` is 773 cents of vibrato at note 0 and 1 cent at note 119.
  Proportional modulation (`SPI·M·2^S/2^16`, one extra 8-bit multiply per pitch-modulated channel, 10 clocks) gives a constant
  3.4 cents at `S = 0`, from 0.01 cent (`S = −8`) to 384 cents (`S = +7`), whatever the note.
* Portamento is linear in SPI, so it is a linear glide in Hz, not in semitones.

### Smaller points

* **Note onset (resolved).** The draft left the envelope at level 0 until the channel's next 64-frame update, so up to 2 ms of a
  sample's attack transient played into silence (measured first audible frame: 59 frames for `ATTACK = 0`). The model now applies the
  first envelope step at note-on: `ATTACK = 0` and `255` are audible after 3 frames (0.09 ms, the PCM pipeline). `ATTACK = 1` still
  takes 59 frames to become audible because level 1 gives a cached gain of 0.
* **Unity gain is unreachable.** Maximum `G = (4095·255) >> 8 = 4079`, so −0.036 dB.
* **Pan endpoints leak.** `pan = 255` leaks 1/256 (−48 dB) into the other side (measured 126 of 32250). Centre is −6.02 dB per side
  (linear law, constant amplitude).
* **`BCLK = CLKI/7` has a 4:3 duty cycle.** `CLKI/8` with 21-bit slots fills the 336-clock frame exactly with a 50 % duty cycle.
  Both round-trip bit-exactly, and the TDA1541 accepts either (see Decisions).
* **Loop shorter than `SPI`.** With the single-subtraction loop, any loop span shorter than the per-sample step is undefined. This
  is harmless for single-cycle waveforms below Nyquist, but it bites on short multi-cycle loops.

## Choices the model made for the open points

Every item here is my pick, made so tests could be deterministic. They are all easy to change.

| Open point | Prototype behaviour |
| --- | --- |
| `WAIT n` | Sets a counter and ends the tick; the next instruction runs on the tick where the counter reaches 0. `WAIT 0` is a no-op. |
| `END N` comparison | Halts when the shared counter is `>=` N, which removes the "already greater" case. Counter saturates at 255. |
| 8-instruction budget | Every fetched instruction counts. On exhaustion the channel stops for the tick and resumes at its PC on the next one. |
| Missed tick | A channel still busy at the next tick drops that tick (its `WAIT` is not decremented), and `MEM_OVERRUN` sets. |
| `SEQ_PC` write while busy | Aborts any in-flight fetch or load; effects already committed stay. |
| Direct commands | One pending `COMMAND` per channel, last write wins. With `SEQ_EN = 1` the direct `COMMAND`, `INSTR` and `FINE` writes are discarded. |
| Instrument load | Words 0 to 6, then two `SPIT` words. Reloaded only when the instrument number changes (`instrument_cache`, switchable). |
| Instrument layout | ADSR bytes MSB first (`ATTACK` high). `LOOP_ENABLE` is bit 15 of word 6. |
| `SPIT` entry | High word first, at `SPIT + 4·(note mod 12)`; `SPI = entry << (note / 12)`. |
| `FINE` | Applied when the `NOTE` is processed, after the octave shift (`FinePolicy`, switchable). |
| Portamento | Glide only if same instrument, `PORTA_EN`, and the channel is sounding. Otherwise a full retrigger. |
| Retrigger | Position 0, envelope level 0, attack phase, then the first envelope step is taken immediately. |
| Envelope | Zero attack, decay or release step means instantaneous. Sustain 8 to 12 bits as `(S<<4) | (S>>4)`. |
| Modulation scale | Internal `M` is the top 8 bits of the post-gain sample. `K` = 4 for pitch and cutoff, 0 otherwise. One frame of delay. |
| Reserved `FILTER_CONFIG` `11` | Behaves as low-pass. |
| SRAM power-up | Seeded noise, so no test can depend on zeros. |

## Decisions

Rule of thumb from the project owner: take the cheapest or simplest option now, and revisit it once the prototype has been
heard if it sounds really bad. A little character in the sound is fine.

| # | Topic | Decision |
| --- | --- | --- |
| 1 | Cutoff resolution | Keep the 8-bit linear cutoff. The coarse low end is part of the character; revisit after listening. |
| 2 | `FINE` and pitch modulation | Keep raw additive units. |
| 3 | Filter arithmetic | Bare truncation. |
| 4 | Cutoff register 0 | Keep the freeze: the state holds, `LP` outputs a constant. Costs nothing; documented behaviour. |
| 5 | Note onset | Apply the first envelope step at note-on (implemented). `ATTACK = 0` now reaches full level in about 0.1 ms. |
| 6 | DAC | Philips TDA1541. See the next section. |
| 7 | Missed tick | Drop the tick (as modelled). |
| 8 | Unity gain and hard pan | Leave as is: maximum gain 99.6 % of unity, hard right leaks -48 dB. Document in the manual. |

### Target DAC: Philips TDA1541 (product specification, November 1985)

Read from the datasheet (OCR of a scanned copy; the timing waveforms, Figs. 3 to 5, are images and were **not** read):

* Dual 16-bit current-output DAC (4 mA full scale). It needs an external I/V stage per channel and three supplies: +5 V, -5 V
  and -15 V.
* Time-multiplexed serial input on `WS`, `BCK` and `DATA`, MSB first, "any bit length". The pin `OB/TWC` selects offset binary
  (ground) or **two's complement (+5 V)**. SIRCAS outputs signed 16-bit, so it ties the pin to +5 V.
* **A separate system clock input `SCK` is required**, for jitter-free output timing. The datasheet shows two time-multiplexed
  formats: `fSCK = fBCK` (labelled I2S, Fig. 3) and `fSCK = 2 x fBCK` (Fig. 4). SIRCAS therefore needs an `SCK` output pin.
* Limits: `BCK` 6 MHz maximum, `SCK` 12 MHz, `WS` 200 kHz. `BCK` period 160 ns minimum, high and low 48 ns minimum. Data set-up
  32 ns, hold 0 ns. Fixed phase relations between `SCK` and `BCK` edges (32 ns and 50 ns margins).
* Channel separation 80 dB, signal-to-noise 95 dB, inter-channel delay up to 1 us.

What this means for the draft:

* **Both `CLKI/7` and `CLKI/8` are fine.** `BCK` at 1.53 MHz or 1.34 MHz is far below the limit, and the DAC only needs 48 ns high
  and low, so the 4:3 duty cycle of `CLKI/7` does not matter. Keep the draft's `CLKI/7` with 24-bit slots.
* Add an `SCK` output equal to the `BCK` frequency (Fig. 3 format), offset in phase by one or two `CLKI` cycles (93 ns each) from `BCK`
  to meet the edge margins. The phase offsets must be checked against Figs. 3 and 4 in the datasheet before the pinout is frozen.
* The sample rate (31.96 kHz) is not oversampled, so the board needs a steep analogue anti-image filter after the DAC. That is a
  board matter, not a chip matter.

## Not done

* No audible output (no host audio backend yet), no WAV export.
* No SRAM electrical timing (chip enable, output enable, turnaround). Only the schedule is modelled.
* No assembly-driven test through the SIRC-1 CPU; the bus path is covered by a scripted master.
* The manual itself (`docs/audio`, LaTeX chapters) has not been started. The video manual is the template once these questions close.
