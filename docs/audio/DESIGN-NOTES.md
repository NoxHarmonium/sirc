# SIRCAS design notes

Rationale, precedents and measurements behind the SIRCAS manual. The manual itself is written as a period hardware reference and
must not carry any of this. `STATUS.md` records the model's implementation status and every decision; this file records why.

## Overall shape

* **Six voices, one filter.** Six eight-bit PCM voices with a single shared resonant filter is a deliberate cut-down of the usual
  console sound chip: the filter is the expensive part (three coefficient multiplications a sample), so it is shared, and a
  voice either feeds it or the dry mix. Any voice can also be a modulation source, so the chip has no dedicated LFO or
  modulator hardware: a voice is spent instead.
* **Dedicated external memory.** 64 KB of static memory, controlled entirely by the chip, keeps the sound chip off the CPU's bus
  for sample data. That removes bus contention from the design: the CPU reaches the memory through a window, in slots reserved
  for it. Two 32K by 8 memories in parallel form the 16-bit word. The memory interface needs 34 pins, which is why the package
  is open (the draft considered 100-pin QFP).
* **Fixed 1:3 slot pattern.** Every eight `CLKI` cycles are four two-cycle slots: one for the sound processor, three for the CPU.
  Neither side can borrow, so CPU access time (two to five cycles) is independent of the audio workload. The cost is that the
  sound processor has 42 transactions per sample period. The model shows this is plenty: sample reads need at most six, leaving
  at least 36 for the sequencers.
* **Sample rate.** 21,477,272 Hz divided by 672 is 31,960.226 Hz: 336 `CLKI` cycles per output sample. A sample period of 336
  cycles divides into exactly 42 patterns, and the 48-bit serial frame at `CLKI`/7 fills it exactly.

## Playback

* **No interpolation.** Nearest-sample lookup needs no multiplier and no second read per voice. The cost is an error that depends
  on waveform length: measured against an ideal waveform, playback error is about -18 dB for 32 entries, -24 dB for 64, -30 dB
  for 128 and -36 dB for 256, with non-harmonic components of about -30 to -48 dB (`resample_study`). The error does not depend
  on the note. Linear interpolation would cost about one 8-bit multiply per voice (about 60 clocks for six) and a second sample
  read. The budget could absorb it; it is not planned, and the lo-fi grit is accepted as character.
* **Pitch table per instrument.** The 12-entry table scales a sample of any recorded rate to the right pitch, and lets one sample
  serve several instruments. Notes are 12 per octave over ten octaves, with the octave a left shift.
* **Raw additive `FINE` and pitch modulation.** Simple and cheap, but the size of a raw unit depends on the octave: a `FINE` of
  4095 is 1313 cents at note 0 and 2 cents at note 119 (64-byte waveform); vibrato with `M = 127`, shift 0 is 773 cents at note 0
  and 1 cent at note 119. The alternatives, `FINE` before the octave shift and proportional pitch modulation
  (`SPI * M * 2^S / 2^16`, about 10 clocks per modulated voice), were measured and rejected for now in favour of the cheap
  form. In practice a vibrato that is set once for a lead voice's range sounds right: a shift of -2 gives about 16 cents at G4
  and 8 at G5.
* **Retrigger.** A hard retrigger restarts the waveform at its first value at the new level. The model flushes the fetch
  pipeline at retrigger so the previous byte is not played at the new gain (a one-frame click found in the demo). A step
  remains by nature; instrument design avoids it (waveform from a zero crossing, decay to silence between notes).

## Envelope and portamento

* **Envelope clock.** One update every 64 samples (2.002 ms) per voice, staggered so no two voices update in the same sample.
  Step 255 gives a 32 ms full-scale attack and step 1 gives 8.2 s.
* **First step at note-on.** The draft left the level at 0 until the voice's next update. Measured first audible frame was 59
  frames (1.85 ms) for an attack of 0, so up to 2 ms of a drum's attack was lost, by a different amount each time. Taking the first
  step at note-on makes an attack of 0 audible after three frames (the output pipeline). Approved departure from the draft.
* **Portamento on the envelope clock.** The draft steps portamento once per sequencer tick. A demo with 125 ms ticks made a
  kick held at 494 Hz for 250 ms (a cowbell) and a lead that never reached its target notes. With a 2 ms step a glide is smooth at
  any tempo, runs with the sequencer stopped, and `PORTA_RATE` is raw units per 2 ms. Longest glide is the distance times 2 ms:
  an octave from A0 is at most 7.2 s, but a semitone from A0 is at most 0.43 s. A 12.4 fixed-point rate would give 16 times
  longer maximums; not done. Approved departure from the draft.

## Filter

* **Topology.** Chamberlin state-variable form, `f = cutoff/256`, which reproduces the draft's 5.3 kHz maximum
  (`fc = (fs/pi) asin(f/2)`). It is cheap: three shift-and-add multiplications a sample.
* **Cutoff is linear and coarse at the bottom.** About 20 Hz a step: the first step is an octave, a step at 636 Hz (register
  32) is half a semitone. Alternatives (a wider internal coefficient through an exponential table, about 24 more clocks) were
  declined as character; decision 1 in `STATUS.md`. Listening to the demo's sweep of registers 1 to 90 found no artefacts or hiss at the start of the sweep:
  it sounded like a high-resonance filter.
* **Arithmetic.** 24-bit state with four fraction bits, truncating shifts, saturating sums. Measured SNR of a 440 Hz sine through
  the low-pass: 54 dB at register 8 and 87 to 103 dB at registers 32 to 255 with bare truncation; round-to-nearest gives 75 dB at
  register 8. Residual after a loud impulse is within 5 LSB. `R = 255` holds a constant-amplitude oscillation for the
  5 seconds tested at every cutoff and rounding mode. Bare truncation was chosen as the cheapest.
* **Cutoff 0** freezes the state. Free, deterministic and recoverable; kept.

## The converter

The target converter is the Philips TDA1541A (product specification, February 1991): a dual 16-bit current-output converter,
time-multiplexed input, MSB first, up to 16 bits, two's-complement mode selected by a pin. Unlike the 1985 TDA1541 it has no
system clock input, so `SCK` is not needed for it; SIRCAS keeps the pin for converters that do need one. Limits are far from the
design: `BCK` up to 6.4 MHz and 46 ns high and low (SIRCAS: 1.53 MHz, 4:3 duty), set-up 32 ns and hold 0 ns. Supplies are +5 V,
-5 V and -15 V, and the outputs are currents that need an I/V stage. One point is open: the datasheet says nothing about bits
after the 16th, and SIRCAS pads each 24-bit slot with seven zeros (see `STATUS.md`). The sample rate is not oversampled, so
the board needs a steep analogue filter. `CLKI`/8 with 21-bit slots would give an exact 50 % duty cycle and is equally
acceptable to this converter; `CLKI`/7 with 24-bit slots was kept.

## Mixer

* Maximum voice gain is 4079/4096 (-0.036 dB) and a pan of 255 leaks 1/256 (-48 dB) to the other side. Both are inaudible and
  fixing them costs an adder; left as they are (decision 8). The pan law is linear: centre is -6 dB per side.
* Cycle budget under the model's shift-add costs (coefficient bits plus two clocks per multiplication): six dry voices 216 of 336
  clocks, six filtered 156, worst case with every cached gain invalidated in one frame 276. The draft's estimates were 286 and
  238; the model is less pessimistic, so its figures are a lower bound. Either way there is room.

## Sequencer

* **Tick work.** A voice reads one word per instruction and up to nine more per new-instrument note. Worst case (four new-instrument
  notes in a tick) is 44 reads a voice, 264 for six, which 36 spare slots a sample period deliver in 8 sample periods. A usual
  `INST FINE NOTE WAIT` voice is 13 reads. Measured, the smallest tick that never overran with six voices all running the
  program was 7 samples for the worst program and 2 for the usual one.
* **Missed tick.** The channel drops the tick (its `WAIT` is not decremented) and `MEM_OVERRUN` is set. Catching up was
  rejected as risking a spiral (decision 7).
* **Counted END.** Compared with `>=`, which removes the "counter already above `N`" case.
