# SIRCAS — Sound Processor Reference Specification

**Status:** Working architectural specification, draft 0.1  
**Target:** SIRCUS console, originally circa 1987  
**Scope:** Software-visible architecture and intended implementation model  
**Not yet:** An RTL implementation or verified gate-level timing model

This document consolidates the SIRCAS design decisions from the architecture discussion. Sections labelled **PROVISIONAL** describe intended behaviour that still needs precise numeric definition or simulator verification. Sections labelled **OPEN** are not yet agreed architectural decisions. Do not silently treat them as established behaviour.

## 1. System overview

SIRCAS is a six-channel PCM sound processor with autonomous sequencers, shared modulation and a shared resonant digital filter. Each PCM channel supports signed 8-bit playback, linear ADSR, pitch tables, portamento, individual volume and pan, and an optional modulation-source mode. Channels may be mixed dry or routed through one shared mono filter. Output is signed 16-bit stereo over an I²S-compatible serial interface to an external DAC.

The chip is memory-mapped on SIRC-1's 16-bit-wide, word-addressed peripheral bus. Audio RAM is **64 KiB external SRAM**, dedicated to SIRCAS. Its memory interface is controlled by SIRCAS; the CPU accesses audio RAM only through SIRCAS's CPU-facing bus window. Neither the sequencer nor PCM playback uses the system CPU bus to fetch sample data.

Audio RAM may be implemented by two 32K × 8-bit SRAMs operating in parallel as 32K × 16 bits. The previously discussed 52-pin PLCC assignment applied to *integrated RAM* and is superseded by the external-SRAM decision; the external-memory pinout/package must be finalised.

## 2. Clocks and fixed timing

- Source oscillator (NTSC): 21,477,272 Hz.
- SIRCAS input `CLKI`: 10,738,636 Hz (oscillator / 2).
- Output sample period: **336 CLKI cycles**.
- Output sample rate: **31,960.226 Hz** (oscillator / 672).
- Output period: approximately **31.29 microseconds**.
- Audio processing and memory slot scheduler operate continuously while unmuted *or* muted.

The sample period is a hard recurring deadline for PCM fetch and production of the next stereo output frame. Sequencer operations may be deferred between output sample periods, subject to the next sequencer-tick deadline.

### 2.1 External audio RAM arbitration — selected design

SRAM transactions consume **two CLKI cycles** each. A repeating **eight-clock schedule** contains four non-borrowable two-clock slots:

| Slot | Clocks modulo 8 | Owner |
|---|---:|---|
| 0 | 0–1 | SIRCAS audio engine |
| 1 | 2–3 | CPU-facing port |
| 2 | 4–5 | CPU-facing port |
| 3 | 6–7 | CPU-facing port |

This makes **42 internal transactions** and **126 CPU transactions** available per 336-clock audio sample. No owner can borrow the other's slots. A CPU RAM request is held pending until an assigned CPU slot and acknowledged by `BACK`; resulting RAM wait time is bounded and independent of audio activity. Register accesses are acknowledged without waiting for RAM slots.

The initial phase between a CPU transaction and the slot scheduler may affect the exact latency. DMA time is predictable from schedule plus the SIRC-1 controller's actual bus protocol, not from the audio workload. Do not assume a fixed transfer time independent of initial phase and main-bus overhead.

The audio engine must prioritise reads needed to emit each PCM sample. Remaining audio slots handle instructions, descriptors and pitch tables. `MEM_OVERRUN` sets on a missed PCM sample-fetch or sequencer-tick deadline; ordinary work extending over more than one output sample does **not** by itself constitute an overrun.

**PROVISIONAL:** Allocation of individual audio slots, internal buffering, read/write turn-around, and precise deadline-failure behaviour require implementation. The scheduling ratio and non-borrowing guarantee are settled.

## 3. CPU and physical interfaces

### 3.1 CPU-facing peripheral interface

SIRCAS uses the established SIRC peripheral interface:

- 16-bit address pins `A0`–`A15` (word addresses within the decoded segment).
- 16-bit bidirectional data pins `D0`–`D15`.
- `CLKI`, `RST`, chip select `CS`, bus active `BAS`, read/write indication `BRW`.
- Open-drain bus acknowledge `BACK`. CPU RAM requests may wait for assigned RAM slots.
- Open-drain, level-sensitive **IRQ3** for enabled sequence-completion notifications.

The exact external SRAM interface needs 15 address signals, 16 data signals, and suitable control signals. The external SRAM bus is separate from the CPU's bus; SIRCAS is the memory arbiter.

**OPEN:** Physical package, full pin allocation and electrical details of SRAM outputs (including chip enable/output enable/write enable timing). A 100-pin QFP is a plausible candidate, not a final requirement.

### 3.2 DAC interface

- SIRCAS is the I²S clock master.
- Outputs `BCLK`, `WS` (left/right word select), and serial data `SD`.
- Two-channel signed 16-bit stereo output.
- **PROVISIONAL:** 24-bit serial slots, one per channel, permit 48 bit clocks per stereo sample, with `BCLK = CLKI / 7` (approximately 1.534 MHz). Define I²S MSB delay, pad bits, edge timing, clock duty cycle and external DAC compatibility before freezing the pinout. A separate DAC master-clock output may be necessary for the chosen DAC.

## 4. Memory map and addressing

All addresses below are **16-bit word offsets inside the SIRCAS peripheral segment**.

| Word addresses | Purpose |
|---|---|
| `0x0000–0x000F` | Global registers |
| `0x0010–0x001F` | Channel 0 registers |
| `0x0020–0x002F` | Channel 1 registers |
| `0x0030–0x003F` | Channel 2 registers |
| `0x0040–0x004F` | Channel 3 registers |
| `0x0050–0x005F` | Channel 4 registers |
| `0x0060–0x006F` | Channel 5 registers |
| `0x0070–0x7FFF` | Reserved; reads return 0, writes ignored |
| `0x8000–0xFFFF` | 32,768 16-bit SRAM words (64 KiB) |

CPU word address `0x8000 + n` selects RAM word `n`. PCM data and selected descriptor fields instead use **byte offsets** `0x0000–0xFFFF` relative to audio RAM. Thus byte offset `b` corresponds to CPU word address `0x8000 + (b >> 1)`.

**MSB-first byte packing:** For RAM word `0x1234`, PCM byte at even offset is `0x12` (bits 15:8) and PCM byte at odd offset is `0x34` (bits 7:0). PCM bytes are interpreted as signed two's-complement values.

`INSTR_BASE` and `SEQ_PC` hold CPU-style word addresses in `0x8000–0xFFFF`. `SAMPLE_START` and `SPIT_ADDRESS` hold byte offsets relative to RAM base. `GOTO` displacements are measured in 16-bit **instruction words**.

**PROVISIONAL:** Behaviour of CPU writes racing the audio engine's read of the same SRAM word (slot ordering defines an observable winner, but exact timing remains to be specified).

## 5. Global register file

| Word | Name | Access | Meaning |
|---|---|---|---|
| `0x0000` | `CONFIG` | R/W | Global enable/mute |
| `0x0001` | `STATUS` | R | Revision, sequencer IRQ pending, memory overrun |
| `0x0002` | `STATUSCLR` | W | Write-one-to-clear diagnostic flags |
| `0x0003` | `SEQ_PERIOD` | R/W | Output samples per global sequencer tick; 0 disables ticks |
| `0x0004` | `SEQ_DONE` | R / W1C | Six sticky sequence completion flags |
| `0x0005` | `SEQ_INT_EN` | R/W | Six interrupt enable bits |
| `0x0006` | `INSTR_BASE` | R/W | CPU-style word address of instrument table |
| `0x0007` | `MASTER_VOL` | R/W | Master volume in bits 7:0 |
| `0x0008` | `FILTER_CONFIG` | R/W | Shared filter output mode |
| `0x0009` | `FILTER_CUTOFF` | R/W | Unsigned 8-bit cutoff coefficient |
| `0x000A` | `FILTER_RESONANCE` | R/W | Unsigned 8-bit resonance |
| `0x000B` | `FILTER_VOL_PAN` | R/W | Volume (15:8) and pan (7:0) |
| `0x000C–0x000F` | — | — | Reserved |

### 5.1 `CONFIG` (`0x0000`)

- Bit 15 `AUDIO_EN`: `0` forces both DAC output channels to digital zero; `1` enables audio output.
- Bits 14:0 reserved, read zero, writes ignored.
- Muting does not stop envelopes, sample position, modulation, sequencer clocks or filter calculations.
- Reset value `0x0000`.

### 5.2 `STATUS` and `STATUSCLR`

- `STATUS[3:0]`: revision `VERSION`, initial proposed value 1.
- `STATUS[4]`: `SEQ_IRQ` / sequence interrupt pending, combinationally derived from `(SEQ_DONE & SEQ_INT_EN) != 0`.
- `STATUS[5]`: sticky `MEM_OVERRUN`.
- Other bits return zero.
- Writing `1` to `STATUSCLR[5]` clears `MEM_OVERRUN`; other bits ignored.

### 5.3 Sequence interrupt registers

- `SEQ_DONE[5:0]`: sticky bits, one per channel. Reading does not clear. Writing a 1 clears the corresponding bit.
- `SEQ_INT_EN[5:0]`: interrupt enable mask. Disabling an interrupt never clears completion flags.
- `IRQ3` asserted (open-drain low) whenever `(SEQ_DONE & SEQ_INT_EN) != 0`; deasserted when expression becomes zero. A completion that happened while masked can assert IRQ3 if subsequently unmasked.
- Restarting a sequencer by writing its `SEQ_PC` also clears its `SEQ_DONE` bit.
- Completion does not itself interrupt PCM playback or ADSR.

### 5.4 `FILTER_CONFIG`

- Bits 1:0: `00` low-pass, `01` band-pass, `10` high-pass, `11` reserved.
- All other bits reserved, read zero.
- Filter bypass occurs at **channel routing** via `FILTER_EN`; there is no global filter bypass/enable bit.

### 5.5 Volume and pan registers

- `MASTER_VOL[7:0]`: unsigned master output gain. Bits 15:8 reserved.
- `FILTER_VOL_PAN[15:8]`: filter output volume; `[7:0]`: filter output pan.
- `VOLUME_PAN[15:8]`: channel volume; `[7:0]`: channel pan.
- Gain arithmetic is designed for shift-friendly `/256` scaling; exact full-scale endpoint treatment and round/truncate policy still need specification.

## 6. Per-channel register file

Channel `c` has base word offset `0x0010 + (c << 4)`, for `c = 0..5`.

| Offset | Name | Access | Meaning |
|---|---|---|---|
| `+0` | `CONTROL` | R/W | Sequencer enable, modulation source, filter routing, portamento |
| `+1` | `VOLUME_PAN` | R/W | 8-bit volume and pan |
| `+2` | `MOD_ROUTE` | R/W | Modulation destination, target and signed shift |
| `+3` | `INSTR` | R/W | Pending instrument selection; low 8 bits |
| `+4` | `FINE` | R/W | Signed 13-bit raw 16.16 increment adjustment |
| `+5` | `PORTA_RATE` | R/W | Unsigned 16-bit pitch-increment adjustment per sequencer tick |
| `+6` | `COMMAND` | W | Direct `NOTE` or `STOP` command |
| `+7` | `SEQ_PC` | R/W | Word-addressed sequencer PC; writing restarts channel sequence |
| `+8..+F` | — | — | Reserved |

### 6.1 `CONTROL`

- Bit 15 `SEQ_EN`: sequencer commands enabled for this channel. CPU direct `INSTR/FINE/NOTE/STOP` playback commands are accepted only when sequencer disabled (**PROVISIONAL:** whether CPU writes to `INSTR`/`FINE` are discarded or stored while enabled requires precise definition).
- Bit 14 `MOD_MODE`: channel becomes a modulation source and is removed from audible mixer and filter input.
- Bit 13 `FILTER_EN`: if audible, channel goes to shared mono filter-input sum rather than dry stereo mixer.
- Bit 12 `PORTA_EN`: an ordinary subsequent `NOTE` glides from current SPI to target SPI; a newly selected instrument forces a full retrigger.
- Bits 11:0 reserved. Reset `0x0000`.
- `MOD_MODE` takes precedence over `FILTER_EN`.
- Toggling `SEQ_EN` does not retrigger or restart playback or the sequencer PC.

### 6.2 `COMMAND` (direct CPU playback)

- Bit 15 = 1: `NOTE` with note number bits 6:0 (valid 0–119).
- Bit 15 = 0: `STOP`, starts ADSR release rather than hard cut.
- Other bits reserved.
- Direct note/stop ignored with `SEQ_EN=1`.
- A CPU `NOTE` may require instrument/SPIT loads and therefore need more than one output sample to become audible. The audio engine retains the command until it can safely apply it. **OPEN:** Depth and overwrite policy for multiple CPU commands issued before a prior command takes effect.

### 6.3 `SEQ_PC`

Writing `SEQ_PC` sets the address of the next instruction, clears the channel's `WAIT` state, clears its eight-bit counted-END counter, clears its `SEQ_DONE` bit and unhalts the sequencer. Execution resumes on the next eligible global tick **if `SEQ_EN=1`**. Reading returns the current stored PC. **OPEN:** Detailed interaction between restarting a channel and any queued/in-flight instruction or descriptor fetch.

## 7. Instrument data and pitch tables

Instruments are eight consecutive **16-bit words** (16 bytes), starting at `INSTR_BASE + instrument_index * 8` in CPU-style word addressing:

| Word | Contents |
|---|---|
| 0 | `SAMPLE_START`: unsigned byte offset into RAM |
| 1 | `SAMPLE_LENGTH`: unsigned byte length |
| 2 | `LOOP_START`: unsigned offset in bytes from sample start |
| 3 | `SPIT_ADDRESS`: unsigned byte offset to pitch table |
| 4 | `ATTACK` and `DECAY`: eight bits each |
| 5 | `SUSTAIN` and `RELEASE`: eight bits each |
| 6–7 | Flags, reserved, including loop-enable flag |

**OPEN:** Exact high/low byte assignment for the packed ADSR bytes and bit position of `LOOP_ENABLE`; alignment rules and flag semantics. Default convention should be MSB-first in each word.

Pitch table, `SPIT`, contains **12 unsigned 32-bit 16.16 SPI entries**, one semitone per entry, for A0 through G♯1; the octave is applied using shifts to select notes 0–119. Entries are four-byte aligned and use a fixed word order (**OPEN:** specify high-word first and confirm range/overflow for octave shifting). A pitch value determines advance in **sample bytes per output sample**.

The channel's `INSTR` is a pending selection. A new instrument is loaded on the next `NOTE`, and changing instrument forces a retrigger regardless of portamento. Writing the same instrument again should not imply a new instrument unless specified (**OPEN:** pending-selection semantics for identical numbers).

## 8. PCM playback

- Six homogeneous channels, signed 8-bit PCM, nearest-neighbour sampling, no interpolator.
- Per-channel `SPA` (sample-playback accumulator) and `SPI` (increment), both **16.16 fixed-point**.
- On each output sample: fetch sample at the selected current integer `SPA` position, then advance `SPA += SPI` (sample/fetch ordering **PROVISIONAL** to be made exact).
- For a looping sample, when the position reaches/exceeds `SAMPLE_LENGTH`, subtract one `(SAMPLE_LENGTH - LOOP_START)` period, preserving overshoot.
- If `SPI` can advance more than one loop span in a sample, behaviour is undefined under this one-subtraction implementation.
- For non-looping samples, output becomes silent at sample end but envelope state continues.
- A RAM sample address crossing `0xFFFF` wraps naturally modulo 64 KiB.
- `SAMPLE_LENGTH = 0` is undefined. With looping enabled, `LOOP_START >= SAMPLE_LENGTH` is undefined.
- On a new instrument `NOTE`, sample position and ADSR retrigger. For an existing instrument with portamento enabled, subsequent `NOTE` changes the target pitch without retriggering.

## 9. Envelope generator

- Per-channel **12-bit unsigned level** from `0` to `4095`.
- Linear attack, decay, sustain and release phases, with eight-bit A/D/S/R parameters.
- Each channel envelope updates once every **64 output samples**, approximately 499.38 updates/sec.
- **Staggered updates:** each of six channels uses a different update phase, reducing simultaneous arithmetic load; cadence remains 64 output samples per channel.
- Attack, decay and release are unsigned step sizes applied to level; attack value zero means instantaneous attack.
- Sustain is an unsigned eight-bit level requiring conversion to the twelve-bit envelope scale (**OPEN:** exact mapping and endpoint).
- `STOP` enters release; it is not a hard mute.

**PROVISIONAL:** Exact update order at trigger/release boundaries, effects of zero decay/release steps, envelope saturation/rounding and phase assignment on reset.

## 10. Portamento and fine tuning

- `FINE` is **signed 13-bit two's-complement** and adds directly to target `SPI` in *raw 16.16 increment units*, not cents.
- `PORTA_RATE` unsigned 16-bit; if portamento is active, add or subtract this rate **once per global sequencer tick** toward the target SPI, clamping to avoid overshoot.
- Portamento ticks continue even for a channel in CPU direct mode (global sequencer clock must be running).
- A new instrument triggers immediately; subsequent notes can glide. `PORTA_RATE = 0` holds the current SPI until an event overrides it.

**OPEN:** Signed SPI saturation, order of `FINE` application versus octave shift and modulation, and exact handling of `SEQ_PERIOD = 0` for an active portamento glide.

## 11. Modulation

Any of the six channels can be used as a **modulation source** (`MOD_MODE=1`), consuming a full PCM voice but generating no audible dry or filter output. Its PCM samples, envelope and volume determine the signed modulation source magnitude.

Targets supported: channel SPI, channel volume, channel pan, global filter cutoff, global filter pan, global master volume. Filter resonance is not a modulation target.

`MOD_ROUTE` **proposed 16-bit encoding**:

| Bits | Meaning |
|---|---|
| 15:13 | Destination: `000` off, `001` pitch, `010` volume, `011` pan, `100` filter cutoff, `101` master volume, `110` filter pan, `111` reserved |
| 12:10 | Channel destination 0–5, ignored for global destinations |
| 9:6 | Signed 4-bit shift exponent `S = -8..+7` |
| 5 | `MOD_INVERT` |
| 4:0 | Reserved |

- At most one modulation source affects any given destination; **lowest numbered source channel wins**.
- Output to a destination has **one output-sample delay**. Cascading and feedback routes therefore cannot introduce a combinational loop.
- The destination's programmed base value remains unchanged; the modulation contribution is added and clamped to its valid range.
- Modulation scale uses shifts: `ΔP = M × 2^(S + K_target)` with target-specific exponents. **PROVISIONAL:** the target exponent constants (e.g. +4 for SPI and cutoff, 0 for volume and pan), exact internal M width, rounding/saturation and wrap prevention must be verified.
- ADSR is applied before modulation; modulation must not resurrect an inactive envelope.

## 12. Shared filter

One mono **two-pole state-variable filter** receives the sum of audible channels whose `FILTER_EN` bits are set. Filtered channels do **not** also contribute to the dry mixer. The filter's output is routed to the stereo mixer with its own volume and pan controls.

Selected intended numerical characteristics:

- **24-bit signed internal state, four fractional bits** (20 integer bits including sign).
- **8-bit unsigned cutoff** and resonance registers.
- Low-pass, band-pass and high-pass output selection.
- Saturate internal filter states; never use wraparound arithmetic.
- No automatic gain limiter or suppression of distortion, quantisation noise or limit cycles.
- Shared shift-and-add arithmetic; estimated three coefficient multiplications per output sample.

### 12.1 Cutoff

Nominal coefficient `f = FILTER_CUTOFF / 256`. Maximum cutoff occurs at `0xFF`, approximately 5.3 kHz at the selected output sample rate, for the previously modelled filter topology. **PROVISIONAL:** Exact recurrence and effective frequency curve need to be frozen from an executable reference implementation; register `0` needs explicit DC/stall behaviour.

### 12.2 Resonance

Let `R` be the unsigned eight-bit resonance value. The discussed damping map was:

```
q(R) = (16 + floor(3*(255-R)/4)) / 256,   for 0 <= R <= 254
q(255) = 0
```

This makes `0xFF` a special **zero-damping** setting intended to permit sustained or quantisation-limited self-oscillation. Other high settings remain very resonant. Self-oscillation at every cutoff/resonance combination is **not guaranteed** without further testing.

**OPEN:** Precise SVF equation update order, fixed-point shift/truncation, intermediate widths, overflow and cutoff/resonance behaviour at pathological parameter values. The previous sound test establishes desirable character, not exhaustive numeric correctness.

## 13. Mixer and DSP arithmetic

- PCM source is signed eight-bit, expanded for arithmetic.
- Each audible dry channel has an individual envelope, 8-bit volume and 8-bit pan.
- Filter bus sums enabled channels before filtering; the filtered result has separate volume and pan.
- Two signed **20-bit stereo accumulators** collect dry and filtered contributions.
- Master volume is applied after accumulation.
- Final output saturates to **signed 16-bit** samples before I²S serialization.
- All overflows intended to saturate, never wrap.

### 13.1 Accepted optimisation: cached gain

Store one approximately 12-bit **effective envelope × volume gain per channel**, recalculated when envelope level or volume changes:

```
G = (Envelope * Volume) >> 8
PCM16 = sign_extend(PCM8) << 8
ScaledSample = (PCM16 * G) >> 12
```

This avoids separate envelope and volume multiplications every output sample. For panning, one channel's stereo contribution can be obtained by calculating one pan product and deriving the complementary contribution by subtraction; exact scaling and endpoint definitions remain **OPEN**.

Envelope updates are staggered by channel. Recomputations caused by simultaneous notes, CPU writes or modulation may be queued, but must have defined ordering and bounded effect latency.

### 13.2 Cycle-budget evidence (non-normative)

Prior analytical schedules assumed a shared shift-and-add multiplier using one coefficient bit per clock plus two setup/output clocks. Illustrative counts within the 336-clock deadline:

| Scenario | Estimated DSP clocks |
|---|---:|
| Six dry channels, naive envelope and gain products | 346 (fails) |
| Six dry channels with cached gain | 286 |
| Six filtered channels | 238 |
| Three dry, two filtered, one modulation voice | 263 |
| Five dry, one modulation voice targeting volume | 289 |

These numbers are **hypotheses for implementation**, not guarantees that any RTL or software clock model will meet timing. Memory scheduling, deferred control operations, control-register update pressure and stage interactions must be jointly simulated.

## 14. Autonomous sequencer

Six independent sequencer channels share one instruction decoder and global tick source. Each channel has a word-addressed PC, a WAIT counter, a counted-END counter, and halted/running state. One channel can execute at most **eight instructions per sequencer tick**; operations may complete over several output samples, while preserving channel instruction order.

`SEQ_PERIOD` is the number of output samples per global sequencer tick (unsigned 16-bit); `0` disables tick generation. For example, 666 samples/tick is approximately 20.84 ms/tick.

All instructions are **one 16-bit word** with bits `[15:13]` for the opcode and bits `[12:0]` for the operand.

| Opcode | Instruction | Operand | Effect |
|---|---|---|---|
| `000` | `NOTE` | 7-bit note 0–119 | Start note / glide depending on instrument and portamento |
| `001` | `STOP` | Unused | Enter envelope release |
| `010` | `WAIT` | Unsigned 13-bit, 1–8191 | Delay track by indicated sequencer ticks |
| `011` | `INST` | Unsigned 8-bit instrument index | Set pending instrument for next NOTE |
| `100` | `FINE` | Signed 13-bit | Set pitch-increment offset |
| `101` | `GOTO` | Signed 13-bit, −4096..+4095 | PC-relative word displacement from following instruction |
| `110` | `END` | Unsigned 8-bit count in bits 7:0 | Finish immediately if 0; otherwise increment count and finish on reaching operand |
| `111` | — | — | Reserved |

Unused operand bits are reserved; invalid note numbers, `WAIT 0`, `END` values above 255, and reserved opcode behaviour must be clearly marked undefined or ignored in the final ISA.

### 14.1 Counted `END`

- Each sequencer channel has its own eight-bit END counter, initially zero.
- `END 0` halts immediately.
- `END N` for `1 <= N <= 255` increments the channel counter; if updated count reaches `N`, the sequencer halts, otherwise execution continues to the next instruction.
- All `END` instructions on one channel share this single counter. No nesting and no branch-specific counters.
- `GOTO` does not reset the counter.
- Writing `SEQ_PC` resets the counter and unhalts/restarts execution.
- On halt, `SEQ_DONE` bit sets exactly once; optional IRQ3 depends on enable mask.
- On halt, current PCM playback and ADSR continue.

**OPEN:** Behaviour if a sequence visits `END N` with a counter already greater than N (equal-versus-greater comparison, wraparound); reset/overflow on count 255; recommend an explicit comparison rule in final draft. The current intended behaviour is completion *upon reaching the count*, not per-loop separate state.

### 14.2 Example

```asm
start:
    NOTE 48       ; A4 under the selected note numbering
    WAIT 10
    NOTE 50       ; B4
    WAIT 10
    END 5
    GOTO start
```

The sequence repeats five passages and sets its `SEQ_DONE` bit, after which it is halted until its `SEQ_PC` is rewritten.

### 14.3 Sequencer bounded execution

- Maximum eight instructions executed per channel per tick prevents an unbounded single-tick `GOTO` loop.
- PCM reads have priority in allocated audio RAM slots. Instrument, pitch table and instruction reads may extend over several 336-clock output frames.
- All work from a tick must finish before the next tick, else `MEM_OVERRUN` sets.
- **OPEN:** Exact meaning of eight instructions when a `GOTO` loop reaches the budget without `WAIT`, whether a new tick resumes immediately, and timing/ordering of deferred notes across channels. A queue/backpressure policy must not allow unbounded outstanding work.

### 14.4 Direct CPU control and sound effects

No separate SFX engines or shadow playback registers. For sound effects, software sets `SEQ_EN=0` on a voice and uses `INSTR`, `FINE` and `COMMAND`. Other registers (volume, pan, filter, modulation, portamento) may be adjusted in either sequencer or direct mode. Re-enabling `SEQ_EN` does not automatically reset the track; write `SEQ_PC` explicitly to begin a chosen sequence.

## 15. Reset and initial state

| Item | Reset state |
|---|---|
| `CONFIG` | 0; output muted |
| `MASTER_VOL` | 255 |
| `SEQ_PERIOD` | 0; tick generator disabled |
| `SEQ_DONE`, `SEQ_INT_EN`, `MEM_OVERRUN` | 0 |
| All channel `CONTROL` | 0 |
| All channel volume | 0 |
| All channel pan | 128 (approx. centred) |
| Envelopes | Inactive |
| Playback and modulation states | Inactive |
| WAIT and counted-END counters | 0 |
| External audio RAM | Undefined; software must initialise before use |
| Audio sample generator and RAM scheduler | Run after reset, even while muted |

**OPEN:** Full reset register defaults and whether the filter state resets to zero. Audio-RAM contents are not cleared by reset.

## 16. Reference-simulator validation plan

Before treating this draft as implementation-ready, build an executable SIRCAS device model in the existing SIRC simulator. The software model should explicitly account for `CLKI`, the two-clock external SRAM transactions, and 336-clock output boundaries. Prefer this reference model over a premature complete RTL implementation.

Minimum acceptance tests:

1. **RAM slot determinism:** A fixed CPU transfer trace must receive the same `BACK` timing whether SIRCAS is muted, playing six dry channels, or processing six simultaneous notes (given identical starting phase).
2. **PCM deadlines:** Six active channels with worst valid SPI and loop cases meet every sample deadline.
3. **Burst handling:** Six `INST; FINE; NOTE; WAIT` tracks begin playing with bounded deferred latency and no false overrun at normal `SEQ_PERIOD`.
4. **Stress/overrun:** An intentionally unsustainable tick rate sets sticky `MEM_OVERRUN`; the chip never steals CPU slots.
5. **Sequencer correctness:** `WAIT`, signed relative `GOTO`, eight-instruction tick cap, `END 0`, `END N`, shared-per-channel END counter, PC rewrite and IRQ3 W1C all have deterministic behaviour.
6. **Direct note timing:** Writes while playback runs produce predictable effects or overwrite/queue outcomes according to the final command policy.
7. **ADSR/portamento:** Staggered 64-sample envelope updates and global-tick glide match the intended register semantics.
8. **Modulation:** One-sample latency, lowest-channel-wins conflicts and feedback loops do not create combinational dependencies.
9. **Filter:** Saturation, cutoff extremes, resonance `0xFF`, prolonged silence and self-oscillation/limit cycles behave deterministically.
10. **DSP cycle accounting:** Every sample frame can finish arithmetic plus any bounded admitted control updates in <=336 CLKI, including multiply shift/setup overhead and fixed SRAM schedule.
11. **Output formatting:** PCM clipping, gain extremes, pan extremes, I²S word-select boundaries and signed 16-bit serialization are bit-exact.
12. **Reset:** RAM undefined; registers deterministic; output digital zero until enabled; no spurious IRQ after reset.

**Important:** Existing Python simulations established a promising RAM and arithmetic budget under selected costs. They are not an integrated device simulation. The reference SIRC device will be the authoritative behavioural prototype; RTL can follow when the spec is stable.

## 17. Decisions still to close

The design is feature-complete at the architectural level, but the following should be frozen before calling the manual **v1.0**:

1. **External SRAM electrical interface** (pins/package, access timing and read/write setup, bus turnaround).
2. **Instruction edge cases and deadline behaviour** (`END` comparisons/wrap, `WAIT` exact semantics, eight-instruction budget exhaustion, missed tick response).
3. **CPU command and sequencing interactions** (pending direct command queue; when notes commit; PC writes vs queued work).
4. **Exact fixed-point reference maths** (SVF recurrence/update order, truncation, round/saturate, resonance extremes; channel pan and filter pan endpoints).
5. **Exact modulation numeric mapping** (`K_target`, signed shifts, clamps and modulation/gain cache update scheduling).
6. **Instrument and SPIT binary layout details**, including flags, table integer ranges and high/low word order.
7. **I²S electrical and framing details** for an era-plausible external DAC.
8. **Full register reset table** and invalid operand treatment.

These are implementation and numerical-semantics decisions, not reasons to reconsider the major feature set.
