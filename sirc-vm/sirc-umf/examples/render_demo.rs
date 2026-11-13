//! Render a short demo piece through the SIRCAS model to a WAV file.
//! `cargo run --release -p sirc_umf --example render_demo -- demo.wav`
//!
//! Six voices: filtered saw bass, portamento pulse lead with LFO vibrato, plucked arpeggio,
//! sine-sweep kick (portamento), noise hat and snare. The CPU role is played by this program:
//! it sets up the registers, then only sweeps the filter cutoff while the sequencers run.

use sirc_umf::core::{mod_route, moddest, reg, Policy, Sircas};
use sirc_umf::testkit::{
    isa, noise, pulse, run_frames, saw, sine, spit_entries, write_wav, Builder, InstrumentSpec,
};
use sirc_umf::OUTPUT_RATE_HZ;

const CH: fn(usize, u16) -> u16 = reg::ch;
// 400 samples is 12.5 ms (80 Hz); a sixteenth note at about 120 BPM is ten ticks. (Portamento
// runs on the envelope clock, so the tick rate no longer affects how smooth a glide is.)
const TICK_FRAMES: u16 = 400;
const STEP_TICKS: u16 = 10;
const BAR: usize = 16;
const INTRO_BARS: usize = 4;
const MAIN_BARS: usize = 8;
const ARP_ATTACK: u8 = 0;
const ARP_DECAY: u8 = 20;
const ARP_SUSTAIN: u8 = 70;

type Events = Vec<Vec<u16>>;

fn empty_bar() -> Events {
    vec![vec![]; BAR]
}

/// One bar of instructions: each step's events, then a WAIT that makes the step `STEP_TICKS` long
/// (less any WAITs the events already contain).
fn emit_bar(events: &Events) -> Vec<u16> {
    let mut out = vec![];
    for step in events {
        out.extend(step);
        let inner: u16 = step
            .iter()
            .filter(|w| *w >> 13 == 2)
            .map(|w| w & 0x1FFF)
            .sum();
        out.push(isa::wait(STEP_TICKS - inner));
    }
    out
}

/// A triangle that starts at its zero crossing, so a retrigger does not start at a peak.
fn triangle_from_zero(len: usize, amp: f64) -> Vec<i8> {
    (0..len)
        .map(|i| {
            let p = (i as f64 / len as f64 + 0.25).fract();
            let v = if p < 0.5 {
                4.0 * p - 1.0
            } else {
                3.0 - 4.0 * p
            };
            (amp * v).round() as i8
        })
        .collect()
}

/// Place notes (note, length in sixteenths). Each note gets a gate one sixteenth shorter than its
/// length. With `legato`, the gate is only closed before a rest, so consecutive notes glide into
/// each other instead of each one starting from a released envelope.
fn melody(bar: &mut Events, notes: &[(Option<u8>, usize)], legato: bool) {
    let mut t = 0;
    for (i, &(n, len)) in notes.iter().enumerate() {
        if let Some(n) = n {
            bar[t].push(isa::note(n));
            let next_is_note = matches!(notes.get(i + 1), Some((Some(_), _)));
            if !(legato && next_is_note) {
                let gate = if len >= 2 { len - 1 } else { 1 };
                if t + gate < BAR {
                    bar[t + gate].push(isa::stop());
                }
            }
        }
        t += len;
    }
    assert_eq!(t, BAR, "bar must be 16 sixteenths");
}

#[derive(Clone, Copy)]
struct Opts {
    /// Only this channel is audible. The others keep running at volume 0, so timing is identical.
    solo: Option<usize>,
    /// Vibrato from the LFO on the lead.
    vibrato: bool,
    /// Reproduces the original abrupt cutoff jump at the end of the intro.
    cutoff_jump: bool,
    /// Arp envelope that decays to silence before the next note (no retrigger step).
    arp_pluck: bool,
}

struct Rendered {
    sircas: Sircas,
    /// First note (SPI change) per channel after the intro, in ms after the start of bar 5.
    first_main_note_ms: [Option<f64>; 5],
    /// Every change of a channel's SPI: (ms from the start of the render, SPI).
    spi_log: Vec<Vec<(f64, u32)>>,
}

fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir).expect("create output dir");
    let full = Opts {
        solo: None,
        vibrato: true,
        cutoff_jump: false,
        arp_pluck: false,
    };
    let jobs: Vec<(&str, Opts)> = vec![
        ("sircas-demo.wav", full),
        (
            "sircas-demo-original-cutoff-jump.wav",
            Opts {
                cutoff_jump: true,
                ..full
            },
        ),
        (
            "stem-0-bass.wav",
            Opts {
                solo: Some(0),
                ..full
            },
        ),
        (
            "stem-0-bass-original-cutoff-jump.wav",
            Opts {
                solo: Some(0),
                cutoff_jump: true,
                ..full
            },
        ),
        (
            "stem-1-lead-vibrato.wav",
            Opts {
                solo: Some(1),
                ..full
            },
        ),
        (
            "stem-1-lead-no-vibrato.wav",
            Opts {
                solo: Some(1),
                vibrato: false,
                ..full
            },
        ),
        (
            "stem-2-arp.wav",
            Opts {
                solo: Some(2),
                ..full
            },
        ),
        (
            "stem-2-arp-pluck.wav",
            Opts {
                solo: Some(2),
                arp_pluck: true,
                ..full
            },
        ),
        (
            "stem-3-kick.wav",
            Opts {
                solo: Some(3),
                ..full
            },
        ),
        (
            "stem-4-hat-snare.wav",
            Opts {
                solo: Some(4),
                ..full
            },
        ),
    ];
    for (name, opts) in jobs {
        let r = render(opts);
        let s = &r.sircas;
        write_wav(&dir.join(name), &s.captured, OUTPUT_RATE_HZ.round() as u32).expect("write wav");
        let peak = s
            .captured
            .iter()
            .map(|&(l, r)| i32::from(l).abs().max(i32::from(r).abs()))
            .max()
            .unwrap_or(0);
        println!(
            "{name:40} {:.1} s  peak {peak:5}  overruns {}  pcm misses {}",
            s.captured.len() as f64 / OUTPUT_RATE_HZ,
            s.stats.overruns,
            s.stats.pcm_misses
        );
        if let (Some(c), None) = (
            std::env::var("TRACE_CH")
                .ok()
                .and_then(|v| v.parse::<usize>().ok()),
            opts.solo,
        ) {
            if !opts.cutoff_jump {
                println!("  SPI changes of channel {c} (ms, SPI, bytes/sample):");
                for (t, spi) in r.spi_log[c]
                    .iter()
                    .filter(|(t, _)| (8100.0..9700.0).contains(t))
                {
                    println!("    {t:8.1}  {spi:8}  {:.3}", f64::from(*spi) / 65536.0);
                }
            }
        }
        if opts.solo.is_none() && !opts.cutoff_jump {
            println!(
                "  first main-section note per channel, ms after the start of bar 5: {:?}",
                r.first_main_note_ms
            );
            println!(
                "  DSP max {} of 336, worst note latency {} CLKI, filter saturations {}",
                s.stats.dsp_clocks_max, s.stats.note_latency_max_clocks, s.stats.filter_saturations
            );
        }
    }
}

fn render(opts: Opts) -> Rendered {
    let mut s = Sircas::new(Policy {
        capture: true,
        ..Policy::default()
    });
    let mut b = Builder::default();

    // --- instruments: (sample, ADSR) -----------------------------------------------------
    b.looping_instrument(&mut s, 0, &saw(256, 105.0), (0, 8, 140, 40)); // bass
    b.looping_instrument(&mut s, 1, &pulse(64, 0.30, 60.0), (40, 5, 200, 30)); // lead
    b.looping_instrument(
        &mut s,
        2,
        &triangle_from_zero(256, 105.0),
        if opts.arp_pluck {
            (0, 80, 0, 60)
        } else {
            (ARP_ATTACK, ARP_DECAY, ARP_SUSTAIN, 60)
        },
    ); // arp
    b.looping_instrument(&mut s, 3, &sine(64, 115.0), (0, 60, 0, 20)); // kick
    b.looping_instrument(&mut s, 4, &noise(4096, 1, 100.0), (0, 255, 0, 255)); // hat
    b.looping_instrument(&mut s, 5, &noise(4096, 7, 100.0), (0, 40, 0, 60)); // snare
                                                                             // LFO: a 64-byte sine whose pitch table is scaled to give 5.5 Hz at note 0
    let (lfo_start, lfo_len) = b.sample(&mut s, &sine(64, 120.0));
    let lfo_spit = b.spit(&mut s, &spit_entries(64.0 / 5.0));
    b.instrument(
        &mut s,
        6,
        InstrumentSpec {
            start: lfo_start,
            len: lfo_len,
            loop_start: 0,
            loop_en: true,
            spit: lfo_spit,
            attack: 0,
            decay: 0,
            sustain: 255,
            release: 10,
        },
    );

    // --- the song: A minor, Am F C G, A0 = note 0 --------------------------------------------
    let roots = [12u8, 8, 15, 10]; // A1 F1 C2 G1
    let chords: [[u8; 3]; 4] = [[36, 39, 43], [32, 36, 39], [39, 43, 46], [34, 38, 41]];
    let lead: [&[(Option<u8>, usize)]; 4] = [
        &[
            (Some(55), 3),
            (Some(51), 1),
            (Some(48), 2),
            (Some(51), 2),
            (Some(55), 4),
            (None, 4),
        ],
        &[
            (Some(56), 3),
            (Some(53), 1),
            (Some(48), 2),
            (Some(53), 2),
            (Some(56), 4),
            (None, 4),
        ],
        &[
            (Some(58), 3),
            (Some(55), 1),
            (Some(51), 2),
            (Some(55), 2),
            (Some(58), 4),
            (None, 4),
        ],
        &[
            (Some(53), 3),
            (Some(50), 1),
            (Some(46), 2),
            (Some(50), 2),
            (Some(53), 4),
            (None, 4),
        ],
    ];

    let intro_ticks = INTRO_BARS as u16 * BAR as u16 * STEP_TICKS;
    let mut progs: Vec<Vec<u16>> = (0..5).map(|_| vec![]).collect();

    // bass plays the whole piece (intro included, so the filter sweep is heard on its own)
    progs[0].push(isa::inst(0));
    for bar in 0..INTRO_BARS + MAIN_BARS {
        let mut ev = empty_bar();
        let r = roots[bar % 4];
        let notes: Vec<(Option<u8>, usize)> = (0..8)
            .map(|i| (Some(if i % 2 == 0 { r } else { r + 12 }), 2))
            .collect();
        melody(&mut ev, &notes, false);
        progs[0].extend(emit_bar(&ev));
    }
    // other voices wait out the intro
    for p in progs.iter_mut().skip(1) {
        p.push(isa::wait(intro_ticks));
    }
    progs[1].push(isa::inst(1));
    progs[2].push(isa::inst(2));
    progs[3].push(isa::inst(3));
    for bar in 0..MAIN_BARS {
        // lead
        let mut ev = empty_bar();
        melody(&mut ev, lead[bar % 4], true);
        progs[1].extend(emit_bar(&ev));
        // arpeggio: three chord notes and the middle one again, every sixteenth
        let mut ev = empty_bar();
        let c = chords[bar % 4];
        for (step, e) in ev.iter_mut().enumerate() {
            e.push(isa::note(c[[0, 1, 2, 1][step % 4]]));
        }
        progs[2].extend(emit_bar(&ev));
        // kick: pitch drops from note 50 to note 14 by portamento
        let mut ev = empty_bar();
        // each hit is stopped before the next one: a note on a sounding portamento voice glides
        // rather than retriggers
        for s0 in [0usize, 8, 11] {
            ev[s0].extend([isa::note(36), isa::wait(1), isa::note(12)]);
            ev[s0 + 2].push(isa::stop());
        }
        progs[3].extend(emit_bar(&ev));
        // hat and snare share a channel
        let mut ev = empty_bar();
        for s0 in [2usize, 6, 10, 14, 15] {
            ev[s0].extend([isa::inst(4), isa::note(100)]);
        }
        for s0 in [4usize, 12] {
            ev[s0].extend([isa::inst(5), isa::note(62)]);
        }
        progs[4].extend(emit_bar(&ev));
    }
    // tail: let everything ring out
    for p in progs.iter_mut() {
        p.extend([isa::stop(), isa::wait(80), isa::end(0)]);
    }

    let pcs: Vec<u16> = progs.iter().map(|p| b.program(&mut s, p)).collect();

    // --- CPU setup ----------------------------------------------------------------------
    s.reg_write(reg::MASTER_VOL, 224);
    s.reg_write(reg::FILTER_CONFIG, 0); // low-pass
    s.reg_write(reg::FILTER_CUTOFF, 1);
    s.reg_write(reg::FILTER_RESONANCE, 215);
    s.reg_write(reg::FILTER_VOL_PAN, 0xB080);
    let seq = reg::CONTROL_SEQ_EN;
    let setup: [(usize, u16, u16); 5] = [
        (0, seq | reg::CONTROL_FILTER_EN, 0xFF80),
        (1, seq | reg::CONTROL_PORTA_EN, 0x60B8),
        (2, seq, 0x5848),
        (3, seq | reg::CONTROL_PORTA_EN, 0xB080),
        (4, seq, 0x5080),
    ];
    for (c, ctrl, volpan) in setup {
        s.reg_write(CH(c, reg::CH_CONTROL), ctrl);
        let muted = opts.solo.is_some_and(|only| only != c);
        let volpan = if muted { volpan & 0x00FF } else { volpan };
        s.reg_write(CH(c, reg::CH_VOLUME_PAN), volpan);
    }
    // raw SPI units per envelope update (2 ms): about 50 ms for a typical melodic step
    s.reg_write(CH(1, reg::CH_PORTA_RATE), 640);
    s.reg_write(CH(3, reg::CH_PORTA_RATE), 1120);
    // LFO voice on channel 5: modulation source, vibrato on the lead's pitch
    s.reg_write(CH(5, reg::CH_CONTROL), reg::CONTROL_MOD_MODE);
    s.reg_write(CH(5, reg::CH_VOLUME_PAN), 0xFF80);
    if opts.vibrato {
        s.reg_write(
            CH(5, reg::CH_MOD_ROUTE),
            mod_route(moddest::PITCH, 1, -2, false),
        );
    }
    s.reg_write(CH(5, reg::CH_INSTR), 6);
    s.reg_write(CH(5, reg::CH_COMMAND), 0x8000);

    s.reg_write(reg::CONFIG, 0x8000);
    s.reg_write(reg::SEQ_PERIOD, TICK_FRAMES);
    for (c, pc) in pcs.iter().enumerate() {
        s.reg_write(CH(c, reg::CH_SEQ_PC), *pc);
    }

    // --- run: the CPU only sweeps the filter ------------------------------------------------
    let bar_frames = u64::from(TICK_FRAMES) * u64::from(STEP_TICKS) * BAR as u64;
    let total = bar_frames * (INTRO_BARS + MAIN_BARS) as u64 + bar_frames / 2 + 3 * 32_000;
    let intro_frames = bar_frames * INTRO_BARS as u64;
    let mut f = 0u64;
    let mut last_spi = [0u32; 5];
    let mut first_main_note_ms: [Option<f64>; 5] = [None; 5];
    let mut spi_log: Vec<Vec<(f64, u32)>> = vec![vec![]; 5];
    while f < total {
        let cutoff = if f < intro_frames {
            // slow linear sweep through the coarse low end: registers 1..90
            1.0 + 89.0 * f as f64 / intro_frames as f64
        } else {
            let bars = (f - intro_frames) as f64 / bar_frames as f64;
            if opts.cutoff_jump {
                // original: starts at register 45, a drop of 45 registers at the intro boundary
                45.0 + 85.0 * (0.5 + 0.5 * (std::f64::consts::TAU * bars / 4.0 - 1.57).sin())
            } else {
                // continuous: starts at 90 where the sweep ended, then moves around it
                90.0 + 45.0 * (std::f64::consts::TAU * bars / 4.0).sin()
            }
        };
        s.reg_write(reg::FILTER_CUTOFF, cutoff as u16);
        run_frames(&mut s, 16);
        f += 16;
        for c in 0..5 {
            let spi = s.snapshot(c).spi;
            if spi != last_spi[c] {
                last_spi[c] = spi;
                spi_log[c].push((f as f64 * 1000.0 / OUTPUT_RATE_HZ, spi));
                if f >= intro_frames && first_main_note_ms[c].is_none() {
                    first_main_note_ms[c] =
                        Some((f - intro_frames) as f64 * 1000.0 / OUTPUT_RATE_HZ);
                }
            }
        }
    }

    Rendered {
        sircas: s,
        first_main_note_ms,
        spi_log,
    }
}
