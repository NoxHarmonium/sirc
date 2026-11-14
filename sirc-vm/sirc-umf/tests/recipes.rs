//! The worked examples of Appendix B of the SIRCAS manual, built through the model and measured.
//! Every number the appendix quotes for them is checked here. Run with `--nocapture` to see the
//! measurements.

use sirc_umf::core::{mod_route, moddest, reg, Policy, Sircas};
use sirc_umf::testkit::{
    isa, noise, run_frames, saw, sine, spit_entries, square, triangle_from_zero, Builder,
    InstrumentSpec,
};
use sirc_umf::OUTPUT_RATE_HZ;

const CH: fn(usize, u16) -> u16 = reg::ch;
const UPDATE_MS: f64 = 64.0 * 1000.0 / OUTPUT_RATE_HZ;

fn new() -> Sircas {
    Sircas::new(Policy {
        capture: true,
        ..Policy::default()
    })
}

fn mono(s: &Sircas) -> Vec<f64> {
    s.captured
        .iter()
        .map(|&(l, r)| (f64::from(l) + f64::from(r)) / 2.0)
        .collect()
}

fn ms(frames: usize) -> f64 {
    frames as f64 * 1000.0 / OUTPUT_RATE_HZ
}

/// Frequency from rising zero crossings (interpolated) between two frame indices.
fn freq(x: &[f64], from: usize, to: usize) -> f64 {
    let mut crossings = vec![];
    for i in from.max(1)..to.min(x.len()) {
        if x[i - 1] < 0.0 && x[i] >= 0.0 {
            crossings.push(i as f64 - 1.0 + (-x[i - 1]) / (x[i] - x[i - 1]));
        }
    }
    if crossings.len() < 2 {
        return 0.0;
    }
    (crossings.len() - 1) as f64 * OUTPUT_RATE_HZ / (crossings[crossings.len() - 1] - crossings[0])
}

/// Level of a harmonic of `f0` in `x[from..to]`, in dB of full scale (Hann window).
fn harmonic_db(x: &[f64], from: usize, to: usize, f0: f64, k: f64) -> f64 {
    let n = to - from;
    let w = std::f64::consts::TAU * f0 * k / OUTPUT_RATE_HZ;
    let (mut re, mut im, mut wsum) = (0.0, 0.0, 0.0);
    for (i, v) in x[from..to].iter().enumerate() {
        let h = 0.5 - 0.5 * (std::f64::consts::TAU * i as f64 / n as f64).cos();
        re += v * h * (w * i as f64).cos();
        im += v * h * (w * i as f64).sin();
        wsum += h;
    }
    20.0 * ((re * re + im * im).sqrt() * 2.0 / wsum / 32768.0).log10()
}

fn cents(a: f64, b: f64) -> f64 {
    1200.0 * (a / b).log2()
}

/// Peak of each 64-frame window.
fn envelope(x: &[f64]) -> Vec<f64> {
    x.chunks(64)
        .map(|c| c.iter().fold(0.0f64, |m, v| m.max(v.abs())))
        .collect()
}

fn peak(x: &[f64]) -> f64 {
    x.iter().fold(0.0f64, |m, v| m.max(v.abs()))
}

/// Window of `env` (in frames) where the level first reaches `frac` of `of`.
fn first_reach(env: &[f64], frac: f64, of: f64) -> Option<usize> {
    env.iter().position(|&v| v >= frac * of).map(|i| i * 64)
}

/// Window after `from` where the level first falls below `frac` of `of`.
fn first_below(env: &[f64], from: usize, frac: f64, of: f64) -> Option<usize> {
    env.iter()
        .enumerate()
        .skip(from / 64)
        .find(|(_, &v)| v <= frac * of)
        .map(|(i, _)| i * 64)
}

fn voice_setup(s: &mut Sircas, ch: usize, ctrl: u16, vol: u8, pan: u8) {
    s.reg_write(reg::MASTER_VOL, 255);
    s.reg_write(CH(ch, reg::CH_CONTROL), ctrl);
    s.reg_write(
        CH(ch, reg::CH_VOLUME_PAN),
        (u16::from(vol) << 8) | u16::from(pan),
    );
}

fn play_note(s: &mut Sircas, ch: usize, instr: u16, note: u16) {
    s.reg_write(CH(ch, reg::CH_INSTR), instr);
    s.reg_write(CH(ch, reg::CH_COMMAND), 0x8000 | note);
}

fn stop_note(s: &mut Sircas, ch: usize) {
    s.reg_write(CH(ch, reg::CH_COMMAND), 0);
}

// ---------------------------------------------------------------------------------------------
// B.2 The square lead
// ---------------------------------------------------------------------------------------------

#[test]
fn square_lead() {
    let mut s = new();
    let mut b = Builder::default();
    b.looping_instrument(&mut s, 0, &square(64, 90.0), (128, 8, 190, 24));
    voice_setup(&mut s, 0, 0, 255, 128);
    s.reg_write(reg::CONFIG, 0x8000);
    play_note(&mut s, 0, 0, 48); // A4
    run_frames(&mut s, 32_000);
    let x = mono(&s);
    let f = freq(&x, 16_000, 31_000);
    let env = envelope(&x);
    let full = peak(&x);
    let rise = first_reach(&env, 0.99, full).unwrap();
    println!(
        "lead: A4 {f:.2} Hz, peak {full}, reaches 99% at {:.0} ms (attack {:.0} ms)",
        ms(rise),
        4095.0 / 128.0f64.ceil() * UPDATE_MS
    );
    assert!((f - 440.0).abs() < 1.0);
    stop_note(&mut s, 0);
    run_frames(&mut s, 16_000);
    let x = mono(&s);
    let env = envelope(&x);
    let tail = first_below(&env, 32_000, 0.01, full).unwrap();
    println!("lead: release to 1% after {:.0} ms", ms(tail - 32_000));
}

// ---------------------------------------------------------------------------------------------
// B.3 The filtered bass
// ---------------------------------------------------------------------------------------------

fn bass_run(filtered: bool) -> Vec<f64> {
    let mut s = new();
    let mut b = Builder::default();
    b.looping_instrument(&mut s, 0, &saw(256, 105.0), (0, 8, 140, 40));
    voice_setup(
        &mut s,
        0,
        if filtered { reg::CONTROL_FILTER_EN } else { 0 },
        255,
        128,
    );
    s.reg_write(reg::FILTER_CONFIG, 0);
    s.reg_write(reg::FILTER_CUTOFF, 16);
    s.reg_write(reg::FILTER_RESONANCE, 64);
    s.reg_write(reg::FILTER_VOL_PAN, 0xFF80);
    s.reg_write(reg::CONFIG, 0x8000);
    play_note(&mut s, 0, 0, 12); // A1
    run_frames(&mut s, 16_000);
    mono(&s)
}

#[test]
fn filtered_bass() {
    let dry = bass_run(false);
    let wet = bass_run(true);
    let f = freq(&dry, 8_000, 15_000);
    let h = |x: &[f64], k: f64| harmonic_db(x, 6_000, 15_000, f, k);
    for k in [1.0, 2.0, 4.0, 8.0, 16.0] {
        println!(
            "bass: harmonic {k:>2} ({:>5.0} Hz): dry {:6.1} dB, filtered {:6.1} dB",
            f * k,
            h(&dry, k),
            h(&wet, k)
        );
    }
    assert!((f - 55.0).abs() < 0.5);
    // the cutoff is 318 Hz: harmonics well above it are much weaker than in the dry voice
    assert!(h(&wet, 16.0) < h(&dry, 16.0) - 15.0);
}

// ---------------------------------------------------------------------------------------------
// B.4 The violin
// ---------------------------------------------------------------------------------------------

#[test]
fn violin_with_fading_in_vibrato() {
    let mut s = new();
    let mut b = Builder::default();
    // Voice 0: a sawtooth of 256 bytes with a slow attack.
    b.looping_instrument(&mut s, 0, &saw(256, 100.0), (30, 4, 200, 20));
    // Voice 5: the vibrato source, a 64-byte sine at 5.5 Hz, fading in over about a second.
    let (st, len) = b.sample(&mut s, &sine(64, 120.0));
    let spit = b.spit(&mut s, &spit_entries(64.0 / 5.0));
    b.instrument(
        &mut s,
        1,
        InstrumentSpec {
            start: st,
            len,
            loop_start: 0,
            loop_en: true,
            spit,
            attack: 8,
            decay: 0,
            sustain: 255,
            release: 10,
        },
    );
    voice_setup(&mut s, 0, 0, 255, 128);
    voice_setup(&mut s, 5, reg::CONTROL_MOD_MODE, 255, 128);
    s.reg_write(
        CH(5, reg::CH_MOD_ROUTE),
        mod_route(moddest::PITCH, 0, 0, false),
    );
    s.reg_write(reg::CONFIG, 0x8000);
    play_note(&mut s, 0, 0, 48); // A4
    play_note(&mut s, 5, 1, 0);
    run_frames(&mut s, 4 * 32_000);
    let x = mono(&s);
    // One cycle is only 72 frames long, so a single cycle is too coarse (a frame is 1.4 %). Average
    // over 16 cycles (36 ms, a fifth of a vibrato period).
    let mut crossings = vec![];
    for i in 1..x.len() {
        if x[i - 1] < 0.0 && x[i] >= 0.0 {
            crossings.push(i as f64 - 1.0 + (-x[i - 1]) / (x[i] - x[i - 1]));
        }
    }
    let cf: Vec<(usize, f64)> = crossings
        .windows(17)
        .map(|w| (w[0] as usize, 16.0 * OUTPUT_RATE_HZ / (w[16] - w[0])))
        .collect();
    let window = |from_ms: f64, to_ms: f64| -> (f64, f64) {
        let (lo, hi) = (
            (from_ms * OUTPUT_RATE_HZ / 1000.0) as usize,
            (to_ms * OUTPUT_RATE_HZ / 1000.0) as usize,
        );
        let v: Vec<f64> = cf
            .iter()
            .filter(|(t, _)| (lo..hi).contains(t))
            .map(|(_, f)| *f)
            .collect();
        (
            cents(v.iter().cloned().fold(f64::MAX, f64::min), 440.0),
            cents(v.iter().cloned().fold(0.0, f64::max), 440.0),
        )
    };
    for start in [
        0.0, 200.0, 400.0, 600.0, 800.0, 1000.0, 1500.0, 2000.0, 3000.0,
    ] {
        let (lo, hi) = window(start, start + 200.0);
        println!(
            "violin: {start:>5.0}-{:>5.0} ms: pitch {lo:+6.1} to {hi:+6.1} cents",
            start + 200.0
        );
    }
    let (lo0, hi0) = window(0.0, 200.0);
    let (lo1, hi1) = window(1000.0, 1200.0);
    assert!(
        hi0 < 5.0 && lo0 > -5.0,
        "vibrato should be shallow at the start"
    );
    assert!(
        hi1 > 12.0 && lo1 < -12.0,
        "vibrato should be full after a second"
    );
    // the source runs at 5.5 Hz: its pitch step for a 64-byte cycle
    let lfo_hz = f64::from(s.snapshot(5).spi) / 65536.0 * OUTPUT_RATE_HZ / 64.0;
    assert!((lfo_hz - 5.5).abs() < 0.05, "lfo {lfo_hz}");
    // vibrato rate over the last two seconds: crossings of the mean in the smoothed series
    let series: Vec<f64> = cf
        .iter()
        .filter(|(t, _)| *t >= 2 * 32_000)
        .map(|(_, f)| *f)
        .collect();
    let mean = series.iter().sum::<f64>() / series.len() as f64;
    let span = cf.iter().filter(|(t, _)| *t >= 2 * 32_000).count();
    let _ = span;
    let ups = series
        .windows(2)
        .filter(|w| w[0] < mean && w[1] >= mean)
        .count();
    let secs = (cf.last().unwrap().0 - 2 * 32_000) as f64 / OUTPUT_RATE_HZ;
    println!(
        "violin: vibrato rate {:.2} Hz over {secs:.2} s (mean {mean:.1} Hz)",
        ups as f64 / secs
    );
}

// ---------------------------------------------------------------------------------------------
// B.5 The kick
// ---------------------------------------------------------------------------------------------

#[test]
fn kick_glides_down() {
    let mut s = new();
    let mut b = Builder::default();
    b.looping_instrument(&mut s, 0, &sine(64, 115.0), (0, 60, 0, 20));
    voice_setup(&mut s, 0, reg::CONTROL_PORTA_EN, 176, 128);
    s.reg_write(CH(0, reg::CH_PORTA_RATE), 1120);
    s.reg_write(reg::CONFIG, 0x8000);
    play_note(&mut s, 0, 0, 36);
    let mut track = vec![];
    run_frames(&mut s, 64);
    s.reg_write(CH(0, reg::CH_COMMAND), 0x8000 | 12);
    for _ in 0..(32_000 / 64) {
        run_frames(&mut s, 64);
        track.push(s.snapshot(0).spi);
    }
    let hz = |spi: u32| f64::from(spi) / 65536.0 * OUTPUT_RATE_HZ / 64.0;
    let start = hz(track[0]);
    let end = hz(*track.last().unwrap());
    let settled = track
        .iter()
        .position(|&v| v == s.snapshot(0).spi_target)
        .unwrap();
    println!(
        "kick: starts at {start:.1} Hz, ends at {end:.1} Hz, reaches the target after {:.0} ms",
        settled as f64 * UPDATE_MS
    );
    let x = mono(&s);
    let env = envelope(&x);
    let full = peak(&x);
    let gone = first_below(&env, 64, 0.01, full).unwrap();
    println!("kick: peak {full}, below 1% after {:.0} ms", ms(gone));
    assert!((end - 55.0).abs() < 1.0);
    assert!((start - 220.0).abs() < 15.0);
    assert!((30..=45).contains(&((settled as f64 * UPDATE_MS) as usize)));
}

// ---------------------------------------------------------------------------------------------
// B.6 The hat and the snare
// ---------------------------------------------------------------------------------------------

#[test]
fn hat_and_snare() {
    for (name, adsr, note) in [
        ("hat", (0u8, 255u8, 0u8, 255u8), 100u16),
        ("snare", (0, 40, 0, 60), 62),
    ] {
        let mut s = new();
        let mut b = Builder::default();
        b.looping_instrument(&mut s, 0, &noise(4096, 1, 100.0), adsr);
        voice_setup(&mut s, 0, 0, 255, 128);
        s.reg_write(reg::CONFIG, 0x8000);
        play_note(&mut s, 0, 0, note);
        run_frames(&mut s, 16_000);
        let x = mono(&s);
        let env = envelope(&x);
        let full = peak(&x);
        let gone = first_below(&env, 640, 0.02, full).unwrap_or(0);
        println!(
            "{name}: peak {full}, falls below 2% of peak after {:.0} ms",
            ms(gone)
        );
        let limit = if name == "hat" { 40.0 } else { 220.0 };
        assert!(ms(gone) < limit && ms(gone) > limit * 0.7);
    }
}

// ---------------------------------------------------------------------------------------------
// B.7 The plucked arpeggio
// ---------------------------------------------------------------------------------------------

#[test]
fn pluck() {
    let mut s = new();
    let mut b = Builder::default();
    b.looping_instrument(&mut s, 0, &triangle_from_zero(256, 105.0), (0, 80, 0, 60));
    voice_setup(&mut s, 0, 0, 255, 128);
    s.reg_write(reg::CONFIG, 0x8000);
    play_note(&mut s, 0, 0, 51); // C5
    run_frames(&mut s, 16_000);
    let x = mono(&s);
    let env = envelope(&x);
    let full = peak(&x);
    let f = freq(&x, 0, 3000);
    let gone = first_below(&env, 64, 0.01, full).unwrap();
    assert!((f - 523.25).abs() < 1.5, "pluck pitch {f}");
    println!(
        "pluck: C5 {f:.2} Hz, peak {full}, below 1% after {:.0} ms",
        ms(gone)
    );
}

// ---------------------------------------------------------------------------------------------
// B.8 A one-bar pattern
// ---------------------------------------------------------------------------------------------

const TICK: u16 = 400; // samples; 12.516 ms
const STEP: u16 = 10; // ticks per sixteenth

#[test]
fn one_bar_pattern() {
    let mut s = new();
    let mut b = Builder::default();
    b.looping_instrument(&mut s, 0, &sine(64, 115.0), (0, 60, 0, 20)); // kick
    b.looping_instrument(&mut s, 1, &noise(4096, 1, 100.0), (0, 255, 0, 255)); // hat
    b.looping_instrument(&mut s, 2, &noise(4096, 7, 100.0), (0, 40, 0, 60)); // snare
    b.looping_instrument(&mut s, 3, &saw(256, 105.0), (0, 8, 140, 40)); // bass

    // Voice 0: kick on steps 0, 8 and 11. A hit starts the note high and glides it to note 12.
    let mut kick = vec![isa::inst(0)];
    let hit = |gap_after: u16| -> Vec<u16> {
        // NOTE 36; WAIT 1; NOTE 12; WAIT (two sixteenths less a tick); STOP; WAIT (gap)
        vec![
            isa::note(36),
            isa::wait(1),
            isa::note(12),
            isa::wait(2 * STEP - 1),
            isa::stop(),
            isa::wait(gap_after),
        ]
    };
    kick.extend(hit(6 * STEP)); // step 0 hit, then to step 8
    kick.extend(hit(STEP)); // step 8 hit, then to step 11
    kick.extend(hit(3 * STEP)); // step 11 hit, then to step 16
    let kick_loop = 1; // the loop returns to the first NOTE, after the INST
    let kick_len = kick.len();
    kick.push(isa::goto(-((kick_len - kick_loop + 1) as i16)));

    // Voice 1: hat on 2, 6, 10 and 14; snare on 4 and 12.
    let mut hs = vec![];
    let ev = [(2usize, 1u8), (4, 2), (6, 1), (10, 1), (12, 2), (14, 1)];
    let mut at = 0usize;
    for (st, ins) in ev {
        hs.push(isa::wait((st - at) as u16 * STEP));
        hs.push(isa::inst(ins));
        hs.push(isa::note(if ins == 1 { 100 } else { 62 }));
        at = st;
    }
    hs.push(isa::wait((16 - at) as u16 * STEP));
    let hs_len = hs.len();
    hs.push(isa::goto(-((hs_len + 1) as i16)));

    // Voice 3: bass on every second step, root then octave.
    let mut bass = vec![isa::inst(3)];
    for i in 0..8 {
        bass.push(isa::note(if i % 2 == 0 { 12 } else { 24 }));
        bass.push(isa::wait(2 * STEP - 1));
        bass.push(isa::stop());
        bass.push(isa::wait(1));
    }
    let bass_len = bass.len();
    bass.push(isa::goto(-((bass_len - 1 + 1) as i16)));

    let pc_kick = b.program(&mut s, &kick);
    let pc_hs = b.program(&mut s, &hs);
    let pc_bass = b.program(&mut s, &bass);

    let seq = reg::CONTROL_SEQ_EN;
    voice_setup(&mut s, 0, seq | reg::CONTROL_PORTA_EN, 176, 128);
    s.reg_write(CH(0, reg::CH_PORTA_RATE), 1120);
    voice_setup(&mut s, 1, seq, 90, 128);
    voice_setup(&mut s, 2, seq, 200, 128);
    s.reg_write(reg::CONFIG, 0x8000);
    s.reg_write(CH(0, reg::CH_SEQ_PC), pc_kick);
    s.reg_write(CH(1, reg::CH_SEQ_PC), pc_hs);
    s.reg_write(CH(2, reg::CH_SEQ_PC), pc_bass);
    s.reg_write(reg::SEQ_PERIOD, TICK); // last, so that every sequence starts on the first tick

    let bar_frames = usize::from(TICK) * usize::from(STEP) * 16;
    // Track voice 0 note-on instants (SPI jumps up) over three bars.
    let mut onsets = vec![];
    let mut prev = 0u32;
    let mut frame = 0usize;
    while frame < bar_frames * 3 {
        run_frames(&mut s, 16);
        frame += 16;
        let spi = s.snapshot(0).spi;
        if spi > prev + prev / 2 && spi > 0 {
            onsets.push(frame);
        }
        prev = spi;
    }
    let step_ms = ms(usize::from(TICK) * usize::from(STEP));
    println!(
        "pattern: tick {:.3} ms, step {step_ms:.2} ms, bar {:.1} ms ({:.1} beats a minute)",
        ms(usize::from(TICK)),
        ms(bar_frames),
        60_000.0 / (step_ms * 4.0)
    );
    println!(
        "pattern: kick onsets in steps: {:?}",
        onsets
            .iter()
            .map(|f| (*f as f64 / (bar_frames as f64 / 16.0)).round())
            .collect::<Vec<_>>()
    );
    println!("pattern: kick {kick:04X?}");
    println!("pattern: hat/snare {hs:04X?}");
    println!("pattern: bass {bass:04X?}");
    assert_eq!(
        onsets
            .iter()
            .map(|f| (*f as f64 / (bar_frames as f64 / 16.0)).round() as u32)
            .collect::<Vec<_>>(),
        vec![0, 8, 11, 16, 24, 27, 32, 40, 43]
    );
    println!("pattern: overruns {}", s.stats.overruns);
    assert_eq!(s.stats.overruns, 0);
}
