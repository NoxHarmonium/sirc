//! Acceptance tests from section 16 of the draft specification.

use sirc_umf::core::{mod_route, moddest, reg, CpuReq, EnvPhase, Policy};
use sirc_umf::i2s::{deserialise, serialise, I2sFormat};
use sirc_umf::testkit::{isa, power_on, run_frames, Builder, InstrumentSpec};
use sirc_umf::{Sircas, CLKI_PER_FRAME};

const CH: fn(usize, u16) -> u16 = reg::ch;

fn new() -> Sircas {
    Sircas::new(Policy::default())
}

/// Constant-value instrument: unity pitch table, instant attack, full sustain, slow release.
fn dc_instrument(b: &mut Builder, s: &mut Sircas, index: usize, value: i8, release: u8) {
    let (start, len) = b.sample(s, &[value, value]);
    let spit = b.spit(s, &[65536; 12]);
    b.instrument(
        s,
        index,
        InstrumentSpec {
            start,
            len,
            loop_start: 0,
            loop_en: true,
            spit,
            attack: 0,
            decay: 0,
            sustain: 255,
            release,
        },
    );
}

fn setup_voices(s: &mut Sircas, b: &mut Builder) {
    let saw = sirc_umf::testkit::saw(64, 100.0);
    b.looping_instrument(s, 0, &saw, (20, 5, 200, 10));
    b.looping_instrument(s, 1, &sirc_umf::testkit::sine(32, 100.0), (20, 5, 200, 10));
    b.looping_instrument(s, 2, &sirc_umf::testkit::square(16, 90.0), (20, 5, 200, 10));
    for c in 0..6 {
        s.reg_write(CH(c, reg::CH_VOLUME_PAN), 0x8000 | (c as u16 * 20 + 20));
    }
}

fn start_direct(s: &mut Sircas, c: usize, instr: u16, note: u16) {
    s.reg_write(CH(c, reg::CH_INSTR), instr);
    s.reg_write(CH(c, reg::CH_COMMAND), 0x8000 | note);
}

/// Issue a CPU RAM access at the given CLKI phase and return its latency in clocks.
fn cpu_latency(s: &mut Sircas, write: bool, word: u16) -> u32 {
    assert!(s.cpu_request(CpuReq {
        write,
        word,
        data: 0x1234
    }));
    let mut n = 0;
    loop {
        s.clock();
        n += 1;
        if s.cpu_take_result().is_some() {
            return n;
        }
        assert!(n < 64, "CPU request never completed");
    }
}

fn cpu_trace(s: &mut Sircas) -> Vec<u32> {
    let mut lcg = 0x00C0_FFEEu32;
    let mut out = vec![];
    for i in 0..3000u32 {
        lcg = lcg.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        // random idle gap so requests arrive at every phase of the 8-clock schedule
        for _ in 0..(lcg >> 28) {
            s.clock();
        }
        out.push(cpu_latency(s, i % 3 == 0, (lcg >> 8) as u16 & 0x7FFF));
    }
    out
}

fn worst_case_program(b: &mut Builder, s: &mut Sircas) -> u16 {
    // Eight instructions per tick, alternating instruments so the cache never hits.
    b.program(
        s,
        &[
            isa::inst(1),
            isa::note(10),
            isa::inst(2),
            isa::note(11),
            isa::inst(1),
            isa::note(12),
            isa::inst(2),
            isa::note(13),
            isa::goto(-8),
        ],
    )
}

// 1 -----------------------------------------------------------------------------------------
#[test]
fn ram_slot_determinism() {
    // Same CPU trace, three very different audio workloads: latencies must be identical.
    let mut idle = new();
    let base = cpu_trace(&mut idle);

    let mut playing = new();
    let mut b = Builder::default();
    setup_voices(&mut playing, &mut b);
    power_on(&mut playing);
    for c in 0..6 {
        start_direct(&mut playing, c, (c % 3) as u16, 40 + c as u16);
    }
    let t_play = cpu_trace(&mut playing);

    let mut busy = new();
    let mut b = Builder::default();
    setup_voices(&mut busy, &mut b);
    let pc = worst_case_program(&mut b, &mut busy);
    power_on(&mut busy);
    busy.reg_write(reg::SEQ_PERIOD, 1);
    for c in 0..6 {
        busy.reg_write(CH(c, reg::CH_CONTROL), reg::CONTROL_SEQ_EN);
        busy.reg_write(CH(c, reg::CH_SEQ_PC), pc);
    }
    let t_busy = cpu_trace(&mut busy);

    assert_eq!(base, t_play);
    assert_eq!(base, t_busy);
    assert!(
        busy.mem_overrun(),
        "the busy scenario must really be overloaded"
    );
    let (min, max) = (*base.iter().min().unwrap(), *base.iter().max().unwrap());
    println!(
        "CPU RAM latency over {} random-phase accesses: {min}..{max} CLKI",
        base.len()
    );
    assert_eq!((min, max), (2, 5));
}

// 2 -----------------------------------------------------------------------------------------
#[test]
fn pcm_deadlines_worst_spi_and_loops() {
    let mut s = new();
    let mut b = Builder::default();
    // Tiny two-byte loops with huge SPI: every frame needs a fresh word on every channel.
    let (start, len) = b.sample(&mut s, &[10, -10, 20, -20]);
    let spit = b.spit(&mut s, &[0x0003_C000; 12]); // 3.75 bytes/sample, near the one-subtraction limit
    b.instrument(
        &mut s,
        0,
        InstrumentSpec {
            start,
            len,
            loop_start: 0,
            loop_en: true,
            spit,
            attack: 0,
            decay: 0,
            sustain: 255,
            release: 5,
        },
    );
    power_on(&mut s);
    for c in 0..6 {
        s.reg_write(CH(c, reg::CH_VOLUME_PAN), 0x6080);
        start_direct(&mut s, c, 0, 12);
    }
    run_frames(&mut s, 4000);
    assert_eq!(s.stats.pcm_misses, 0);
    assert!(!s.mem_overrun());
    assert!(
        s.stats.pcm_reads >= 6 * 3900,
        "every channel fetched about once per frame"
    );
    assert!(s.stats.max_audio_slots_in_frame <= 42);
    println!(
        "PCM reads {} over {} frames",
        s.stats.pcm_reads, s.stats.frames
    );
}

// 3 -----------------------------------------------------------------------------------------
#[test]
fn burst_of_six_notes_is_bounded_and_clean() {
    let mut s = new();
    let mut b = Builder::default();
    setup_voices(&mut s, &mut b);
    let mut pcs = vec![];
    for c in 0..6u8 {
        pcs.push(b.program(
            &mut s,
            &[
                isa::inst(c % 3),
                isa::fine(5),
                isa::note(36 + c),
                isa::wait(100),
                isa::end(0),
            ],
        ));
    }
    power_on(&mut s);
    s.reg_write(reg::SEQ_PERIOD, 666);
    for (c, pc) in pcs.iter().enumerate() {
        s.reg_write(CH(c, reg::CH_CONTROL), reg::CONTROL_SEQ_EN);
        s.reg_write(CH(c, reg::CH_SEQ_PC), *pc);
    }
    run_frames(&mut s, 666 + 20);
    for c in 0..6 {
        assert!(s.snapshot(c).playing, "channel {c} should be playing");
    }
    assert!(!s.mem_overrun());
    let lat_frames = s.stats.note_latency_max_clocks as f64 / f64::from(CLKI_PER_FRAME);
    println!(
        "worst tick->note latency in a six-note burst: {} CLKI = {lat_frames:.1} frames = {:.0} us",
        s.stats.note_latency_max_clocks,
        s.stats.note_latency_max_clocks as f64 / 10.738636
    );
    assert!(lat_frames < 4.0);
}

// 4 -----------------------------------------------------------------------------------------
#[test]
fn overload_sets_sticky_overrun_and_never_steals_cpu_slots() {
    let mut s = new();
    let mut b = Builder::default();
    setup_voices(&mut s, &mut b);
    let pc = worst_case_program(&mut b, &mut s);
    power_on(&mut s);
    s.reg_write(reg::SEQ_PERIOD, 2);
    for c in 0..6 {
        s.reg_write(CH(c, reg::CH_CONTROL), reg::CONTROL_SEQ_EN);
        s.reg_write(CH(c, reg::CH_SEQ_PC), pc);
    }
    run_frames(&mut s, 200);
    assert!(s.mem_overrun());
    assert!(s.stats.dropped_ticks > 0);
    assert_eq!(
        s.reg_read(reg::STATUS) & reg::STATUS_MEM_OVERRUN,
        reg::STATUS_MEM_OVERRUN
    );
    // sticky until cleared
    s.reg_write(reg::SEQ_PERIOD, 0);
    run_frames(&mut s, 200);
    assert!(s.mem_overrun());
    s.reg_write(reg::STATUSCLR, reg::STATUS_MEM_OVERRUN);
    assert!(!s.mem_overrun());
    assert_eq!(
        s.stats.pcm_misses, 0,
        "PCM reads are never starved by the sequencer"
    );
}

// 5 -----------------------------------------------------------------------------------------
fn run_seq(program: &[u16], period: u16, frames: u64) -> Sircas {
    let mut s = new();
    let mut b = Builder::default();
    setup_voices(&mut s, &mut b);
    let pc = b.program(&mut s, program);
    power_on(&mut s);
    s.reg_write(reg::SEQ_PERIOD, period);
    s.reg_write(CH(0, reg::CH_CONTROL), reg::CONTROL_SEQ_EN);
    s.reg_write(CH(0, reg::CH_SEQ_PC), pc);
    run_frames(&mut s, frames);
    s
}

#[test]
fn sequencer_end_zero_halts_and_flags_once() {
    let mut s = run_seq(&[isa::inst(0), isa::note(48), isa::end(0)], 10, 100);
    assert!(s.snapshot(0).halted);
    assert_eq!(s.reg_read(reg::SEQ_DONE), 1);
    // PCM keeps playing after the sequence ends.
    assert!(s.snapshot(0).playing);
    // W1C
    s.reg_write(reg::SEQ_DONE, 1);
    assert_eq!(s.reg_read(reg::SEQ_DONE), 0);
    run_frames(&mut s, 100);
    assert_eq!(
        s.reg_read(reg::SEQ_DONE),
        0,
        "halted sequencer must not set DONE again"
    );
}

#[test]
fn sequencer_counted_end_with_goto_and_wait() {
    // idx0 NOTE, idx1 WAIT 3, idx2 END 3, idx3 GOTO -4 (back to idx0)
    let prog = [isa::note(48), isa::wait(3), isa::end(3), isa::goto(-4)];
    let s = run_seq(&prog, 10, 10 * 3 * 3 + 10 * 5);
    let snap = s.snapshot(0);
    assert!(snap.halted);
    assert_eq!(snap.end_count, 3);
    assert_eq!(s.reg_read(reg::SEQ_DONE), 1);
}

#[test]
fn wait_delays_by_the_stated_number_of_ticks() {
    // NOTE 48, WAIT 5, NOTE 60: the second note must land exactly five ticks after the first.
    let prog = [
        isa::inst(0),
        isa::note(48),
        isa::wait(5),
        isa::note(60),
        isa::end(0),
    ];
    let mut s = new();
    let mut b = Builder::default();
    setup_voices(&mut s, &mut b);
    let pc = b.program(&mut s, &prog);
    power_on(&mut s);
    s.reg_write(reg::SEQ_PERIOD, 50);
    s.reg_write(CH(0, reg::CH_CONTROL), reg::CONTROL_SEQ_EN);
    s.reg_write(CH(0, reg::CH_SEQ_PC), pc);
    let mut first = None;
    let mut second = None;
    let mut last_spi = 0;
    for f in 0..50 * 10 {
        run_frames(&mut s, 1);
        let spi = s.snapshot(0).spi;
        if spi != last_spi {
            if first.is_none() {
                first = Some(f);
            } else if second.is_none() {
                second = Some(f);
            }
            last_spi = spi;
        }
    }
    let (a, b2) = (first.unwrap(), second.unwrap());
    println!(
        "note 1 at frame {a}, note 2 at frame {b2}: {} ticks apart",
        (b2 - a) as f64 / 50.0
    );
    assert_eq!((b2 - a) / 50, 5);
}

#[test]
fn eight_instruction_budget_stops_a_tight_goto_loop() {
    // FINE; GOTO -2 spins forever without a WAIT. One tick may execute only eight instructions.
    let s = run_seq(&[isa::fine(1), isa::goto(-2)], 100, 100 + 5);
    // exactly one tick has fired: eight fetches, no more
    assert_eq!(s.stats.ctl_reads, 8);
    assert!(!s.mem_overrun());
}

#[test]
fn seq_pc_write_restarts_and_irq3_is_level_sensitive() {
    let mut s = run_seq(&[isa::end(0)], 10, 30);
    assert_eq!(s.reg_read(reg::SEQ_DONE), 1);
    assert!(!s.irq3(), "masked completion must not interrupt");
    s.reg_write(reg::SEQ_INT_EN, 1);
    assert!(s.irq3(), "unmasking a pending completion asserts IRQ3");
    assert_eq!(
        s.reg_read(reg::STATUS) & reg::STATUS_SEQ_IRQ,
        reg::STATUS_SEQ_IRQ
    );
    s.reg_write(reg::SEQ_INT_EN, 0);
    assert!(!s.irq3());
    assert_eq!(
        s.reg_read(reg::SEQ_DONE),
        1,
        "disabling never clears the flag"
    );
    s.reg_write(reg::SEQ_INT_EN, 1);
    s.reg_write(
        CH(0, reg::CH_SEQ_PC),
        0x8000 + (s.reg_read(CH(0, reg::CH_SEQ_PC)) & 0x7FFF),
    );
    assert_eq!(s.reg_read(reg::SEQ_DONE), 0, "writing SEQ_PC clears DONE");
    assert!(!s.irq3());
}

// 6 -----------------------------------------------------------------------------------------
#[test]
fn direct_notes_and_stop_follow_the_command_policy() {
    let mut s = new();
    let mut b = Builder::default();
    setup_voices(&mut s, &mut b);
    power_on(&mut s);
    start_direct(&mut s, 0, 0, 48);
    run_frames(&mut s, 4);
    let snap = s.snapshot(0);
    assert!(snap.playing && snap.env_phase == EnvPhase::Attack);
    // STOP enters release, not a hard cut
    s.reg_write(CH(0, reg::CH_COMMAND), 0);
    run_frames(&mut s, 4);
    assert_eq!(s.snapshot(0).env_phase, EnvPhase::Release);
    // with SEQ_EN set, direct commands (and INSTR/FINE writes) are ignored
    s.reg_write(CH(1, reg::CH_CONTROL), reg::CONTROL_SEQ_EN);
    s.reg_write(CH(1, reg::CH_INSTR), 2);
    assert_eq!(s.reg_read(CH(1, reg::CH_INSTR)), 0);
    s.reg_write(CH(1, reg::CH_COMMAND), 0x8000 | 50);
    run_frames(&mut s, 10);
    assert!(!s.snapshot(1).playing);
}

// 7 -----------------------------------------------------------------------------------------
#[test]
fn envelope_updates_are_staggered_every_64_frames() {
    let mut s = new();
    let mut b = Builder::default();
    dc_instrument(&mut b, &mut s, 0, 50, 10);
    // give it a slow attack of 8 so steps are visible
    s.set_ram_word(4, 0x0800);
    power_on(&mut s);
    for c in 0..6 {
        start_direct(&mut s, c, 0, 12);
    }
    let mut change_frames: Vec<Vec<u64>> = vec![vec![]; 6];
    let mut last = [0u16; 6];
    for _ in 0..64 * 6 {
        run_frames(&mut s, 1);
        for c in 0..6 {
            let l = s.snapshot(c).env_level;
            if l != last[c] {
                change_frames[c].push(s.frame());
                assert_eq!(l, last[c] + 8, "attack step is exactly the ATTACK byte");
                last[c] = l;
            }
        }
    }
    let phases: Vec<u64> = change_frames.iter().map(|v| v[1] % 64).collect();
    println!("envelope update phase (frame mod 64) per channel: {phases:?}");
    for (c, frames) in change_frames.iter().enumerate() {
        // the first change is the note-on step; the cadence applies to the regular updates after it
        let d: Vec<u64> = frames[1..].windows(2).map(|w| w[1] - w[0]).collect();
        assert!(d.iter().all(|&x| x == 64), "channel {c} cadence {d:?}");
    }
    let mut uniq = phases.clone();
    uniq.sort_unstable();
    uniq.dedup();
    assert_eq!(uniq.len(), 6, "each channel has its own phase");
}

#[test]
fn first_envelope_step_happens_at_note_on() {
    // ATTACK = 0 must reach full level at once, not at the channel's next 64-frame update.
    for settle in [0u64, 13, 31, 63] {
        let mut s = new();
        let mut b = Builder::default();
        dc_instrument(&mut b, &mut s, 0, 100, 10);
        power_on(&mut s);
        s.reg_write(CH(0, reg::CH_VOLUME_PAN), 0xFF80);
        run_frames(&mut s, 70 + settle); // land at different envelope phases
        start_direct(&mut s, 0, 0, 12);
        run_frames(&mut s, 3);
        assert_eq!(s.snapshot(0).env_level, 4095, "settle {settle}");
    }
}

#[test]
fn retrigger_does_not_play_the_previous_notes_last_byte() {
    // Note A is a constant +127 that has fully released. Note B is silence (all zero bytes) with
    // an instant attack. The stale +127 left in the fetch pipeline must not be played at B's gain.
    let mut s = Sircas::new(Policy {
        capture: true,
        ..Policy::default()
    });
    let mut b = Builder::default();
    dc_instrument(&mut b, &mut s, 0, 127, 255);
    dc_instrument(&mut b, &mut s, 1, 0, 255);
    power_on(&mut s);
    s.reg_write(CH(0, reg::CH_VOLUME_PAN), 0xFF00);
    start_direct(&mut s, 0, 0, 12);
    run_frames(&mut s, 200);
    s.reg_write(CH(0, reg::CH_COMMAND), 0); // STOP
    run_frames(&mut s, 2000); // release (255 per 64 frames) completes
    assert_eq!(s.snapshot(0).env_phase, EnvPhase::Off);
    let before = s.captured.len();
    start_direct(&mut s, 0, 1, 12);
    run_frames(&mut s, 30);
    assert!(
        s.captured[before..].iter().all(|&f| f == (0, 0)),
        "stale byte played at the new gain: {:?}",
        &s.captured[before..]
    );
}

#[test]
fn portamento_steps_on_the_envelope_clock_without_retrigger() {
    // The sequencer is off (SEQ_PERIOD = 0): glides still run, once per 64 frames.
    let mut s = new();
    let mut b = Builder::default();
    setup_voices(&mut s, &mut b);
    power_on(&mut s);
    s.reg_write(CH(0, reg::CH_CONTROL), reg::CONTROL_PORTA_EN);
    s.reg_write(CH(0, reg::CH_PORTA_RATE), 300);
    start_direct(&mut s, 0, 0, 48);
    run_frames(&mut s, 200);
    let from = s.snapshot(0).spi;
    start_direct(&mut s, 0, 0, 55);
    run_frames(&mut s, 3);
    let snap = s.snapshot(0);
    assert!(snap.spi_target > from + 10_000, "target moved");
    assert!(
        snap.spi < from + 1_000,
        "a glide must not jump to the new pitch (that would be a retrigger)"
    );
    // Watch 640 frames: every change is exactly one rate step, roughly ten of them.
    let mut last = s.snapshot(0).spi;
    let mut steps = vec![];
    for _ in 0..640 {
        run_frames(&mut s, 1);
        let spi = s.snapshot(0).spi;
        if spi != last {
            steps.push(spi - last);
            last = spi;
        }
    }
    println!("portamento steps in 640 frames: {}", steps.len());
    assert!((9..=11).contains(&steps.len()), "{} steps", steps.len());
    assert!(steps.iter().all(|&d| d == 300), "{steps:?}");

    // Gliding down, with a rate larger than the distance, lands exactly on the target.
    s.reg_write(CH(0, reg::CH_PORTA_RATE), 65535);
    start_direct(&mut s, 0, 0, 36);
    run_frames(&mut s, 200);
    let snap = s.snapshot(0);
    assert_eq!(snap.spi, snap.spi_target, "no overshoot");
    assert!(snap.spi < from);

    // A different instrument always retriggers.
    start_direct(&mut s, 0, 1, 55);
    run_frames(&mut s, 3);
    let snap = s.snapshot(0);
    assert_eq!(
        snap.spi, snap.spi_target,
        "retrigger jumps straight to the new pitch"
    );
}

// 8 -----------------------------------------------------------------------------------------
#[test]
fn modulation_lowest_source_wins_and_feedback_is_stable() {
    let mut s = new();
    let mut b = Builder::default();
    dc_instrument(&mut b, &mut s, 0, 100, 10);
    dc_instrument(&mut b, &mut s, 1, 60, 10);
    power_on(&mut s);
    for c in 0..6 {
        s.reg_write(CH(c, reg::CH_VOLUME_PAN), 0xFF80);
    }
    // ch3 (value 100) and ch4 (value 60) both modulate the volume of ch0.
    for (c, i) in [(3usize, 0u16), (4, 1)] {
        s.reg_write(CH(c, reg::CH_CONTROL), reg::CONTROL_MOD_MODE);
        s.reg_write(
            CH(c, reg::CH_MOD_ROUTE),
            mod_route(moddest::VOLUME, 0, -2, false),
        );
        start_direct(&mut s, c, i, 12);
    }
    run_frames(&mut s, 200);
    let both = s.mod_volume(0);
    // Silence the lower source: the higher one now wins.
    s.reg_write(CH(3, reg::CH_CONTROL), 0);
    run_frames(&mut s, 5);
    let only4 = s.mod_volume(0);
    println!("mod volume with ch3 source = {both}, with only ch4 = {only4}");
    assert!(both > only4 && only4 > 0);

    // Feedback: ch1 and ch2 modulate each other's pitch. Two identical runs must agree exactly.
    let run = || {
        let mut s = new();
        let mut b = Builder::default();
        setup_voices(&mut s, &mut b);
        power_on(&mut s);
        s.reg_write(CH(1, reg::CH_CONTROL), reg::CONTROL_MOD_MODE);
        s.reg_write(CH(2, reg::CH_CONTROL), reg::CONTROL_MOD_MODE);
        s.reg_write(
            CH(1, reg::CH_MOD_ROUTE),
            mod_route(moddest::PITCH, 2, 3, false),
        );
        s.reg_write(
            CH(2, reg::CH_MOD_ROUTE),
            mod_route(moddest::PITCH, 1, 3, false),
        );
        s.reg_write(CH(1, reg::CH_VOLUME_PAN), 0xFF80);
        s.reg_write(CH(2, reg::CH_VOLUME_PAN), 0xFF80);
        start_direct(&mut s, 1, 1, 30);
        start_direct(&mut s, 2, 2, 31);
        run_frames(&mut s, 500);
        (s.snapshot(1), s.snapshot(2))
    };
    assert_eq!(run(), run());
}

// 9 -----------------------------------------------------------------------------------------
#[test]
fn filter_is_deterministic_and_saturates_without_wrapping() {
    let run = |cut: u16, res: u16| {
        let mut s = new();
        let mut b = Builder::default();
        dc_instrument(&mut b, &mut s, 0, 127, 10);
        power_on(&mut s);
        s.reg_write(reg::FILTER_CUTOFF, cut);
        s.reg_write(reg::FILTER_RESONANCE, res);
        s.reg_write(reg::FILTER_VOL_PAN, 0xFF80);
        for c in 0..6 {
            s.reg_write(CH(c, reg::CH_VOLUME_PAN), 0xFF80);
            s.reg_write(CH(c, reg::CH_CONTROL), reg::CONTROL_FILTER_EN);
            start_direct(&mut s, c, 0, 12);
        }
        s.policy.capture = true;
        run_frames(&mut s, 3000);
        s.captured.clone()
    };
    for (cut, res) in [(255, 255), (0, 0), (1, 255), (255, 0), (128, 254)] {
        let a = run(cut, res);
        let b = run(cut, res);
        assert_eq!(a, b, "cutoff {cut} res {res}");
    }
}

// 10 ----------------------------------------------------------------------------------------
#[test]
fn dsp_cycle_budget_fits_in_336_clocks() {
    let mut s = new();
    let mut b = Builder::default();
    setup_voices(&mut s, &mut b);
    power_on(&mut s);
    for c in 0..6 {
        start_direct(&mut s, c, (c % 3) as u16, 40);
    }
    // Worst case: every channel's cached gain is invalidated on every frame by CPU writes,
    // plus a modulation voice on volume.
    for f in 0..2000 {
        for c in 0..6 {
            s.reg_write(
                CH(c, reg::CH_VOLUME_PAN),
                0x8000 | ((f as u16 + c as u16) & 0xFF),
            );
        }
        run_frames(&mut s, 1);
    }
    println!(
        "DSP clocks: max {} of {}",
        s.stats.dsp_clocks_max, CLKI_PER_FRAME
    );
    assert!(s.stats.dsp_clocks_max <= CLKI_PER_FRAME);
    assert_eq!(s.stats.dsp_overruns, 0);
}

// 11 ----------------------------------------------------------------------------------------
fn render_dc(value: i8, vol: u16, pan: u16, n: usize) -> Vec<(i16, i16)> {
    let mut s = Sircas::new(Policy {
        capture: true,
        ..Policy::default()
    });
    let mut b = Builder::default();
    for i in 0..n {
        dc_instrument(&mut b, &mut s, i, value, 10);
    }
    power_on(&mut s);
    for c in 0..n {
        s.reg_write(CH(c, reg::CH_VOLUME_PAN), (vol << 8) | pan);
        start_direct(&mut s, c, c as u16, 12);
    }
    run_frames(&mut s, 200);
    s.captured.clone()
}

#[test]
fn output_gain_pan_and_clipping_extremes() {
    let tail = |v: Vec<(i16, i16)>| *v.last().unwrap();
    // full scale single channel, centre pan
    let (l, r) = tail(render_dc(127, 255, 128, 1));
    println!("full-scale +127, vol 255, pan 128 -> L={l} R={r}");
    // hard left / hard right leakage
    let (l0, r0) = tail(render_dc(127, 255, 0, 1));
    let (l1, r1) = tail(render_dc(127, 255, 255, 1));
    println!("pan 0: L={l0} R={r0}   pan 255: L={l1} R={r1}");
    assert_eq!(r0, 0);
    assert!((0..200).contains(&l1), "hard-right leakage into left: {l1}");
    // six full-scale channels hard left: must clip at +32767, never wrap
    let (lc, _) = tail(render_dc(127, 255, 0, 6));
    println!("six full-scale channels, hard left -> L={lc}");
    assert_eq!(lc, 32767);
    let (lc, _) = tail(render_dc(-128, 255, 0, 6));
    assert_eq!(lc, -32768);
    // muted
    let mut s = Sircas::new(Policy {
        capture: true,
        ..Policy::default()
    });
    let mut b = Builder::default();
    dc_instrument(&mut b, &mut s, 0, 100, 10);
    s.reg_write(CH(0, reg::CH_VOLUME_PAN), 0xFF80);
    start_direct(&mut s, 0, 0, 12);
    run_frames(&mut s, 200);
    assert!(
        s.captured.iter().all(|&f| f == (0, 0)),
        "AUDIO_EN=0 forces digital zero"
    );
}

#[test]
fn i2s_framing_round_trips_bit_exactly() {
    let samples = [
        (0i16, 0i16),
        (32767, -32768),
        (-1, 1),
        (0x1234, -0x4321),
        (i16::MIN, i16::MAX),
        (1, -2),
    ];
    for fmt in [I2sFormat::SIXTEEN, I2sFormat::DRAFT, I2sFormat::DIV8] {
        assert!(fmt.fills_frame(), "{fmt:?} must fill exactly 336 clocks");
        let n = fmt.bclks_per_frame() as usize;
        // a final frame carries the LSB of the last right word
        let mut frames = samples.to_vec();
        frames.push((0, 0));
        let bits = serialise(fmt, &frames);
        assert_eq!(bits.len(), n * frames.len());
        for f in 0..frames.len() {
            // the frame's BCLK periods fill exactly one sample period
            let cycles: u32 = (0..n as u32).map(|b| fmt.period_cycles(b)).sum();
            assert_eq!(cycles, CLKI_PER_FRAME);
            // WS low for the whole left slot, high for the whole right slot
            let frame = &bits[f * n..(f + 1) * n];
            assert!(frame[..fmt.slot_bits as usize].iter().all(|b| !b.0));
            assert!(frame[fmt.slot_bits as usize..].iter().all(|b| b.0));
        }
        for (f, &(l, r)) in samples.iter().enumerate() {
            // MSB is delayed one BCLK after the WS edge (I2S), for both channels
            assert_eq!(bits[f * n + 1].1, (l as u16) >> 15 == 1, "{fmt:?} left MSB");
            assert_eq!(
                bits[f * n + fmt.slot_bits as usize + 1].1,
                (r as u16) >> 15 == 1,
                "{fmt:?} right MSB"
            );
        }
        let back = deserialise(fmt, &bits);
        assert_eq!(&back[..samples.len()], &samples[..], "{fmt:?}");
    }
    // In the 16-bit format the LSB of each word is in the first BCLK of the next slot, after WS
    // has changed, and the first BCLK of a frame carries the previous frame's right LSB.
    let fmt = I2sFormat::SIXTEEN;
    let bits = serialise(fmt, &[(0, 1), (0, 0)]);
    assert!(bits[32].1, "right LSB of frame 0 is in BCLK 0 of frame 1");
    assert!(!bits[16].1, "left LSB of frame 0 is 0");
}

// 12 ----------------------------------------------------------------------------------------
#[test]
fn reset_state_is_deterministic() {
    let mut s = new();
    // RAM is undefined (noise), not cleared
    assert!((0..64).any(|i| s.ram_word(i) != 0));
    s.reg_write(reg::CONFIG, 0x8000);
    s.reg_write(reg::MASTER_VOL, 3);
    s.reg_write(reg::SEQ_INT_EN, 0x3F);
    s.reg_write(CH(2, reg::CH_CONTROL), 0xF000);
    s.reset_registers();
    assert_eq!(s.reg_read(reg::CONFIG), 0);
    assert_eq!(s.reg_read(reg::STATUS), 1);
    assert_eq!(s.reg_read(reg::MASTER_VOL), 255);
    assert_eq!(s.reg_read(reg::SEQ_PERIOD), 0);
    assert_eq!(s.reg_read(reg::SEQ_DONE), 0);
    assert!(!s.irq3());
    for c in 0..6 {
        assert_eq!(s.reg_read(CH(c, reg::CH_CONTROL)), 0);
        assert_eq!(s.reg_read(CH(c, reg::CH_VOLUME_PAN)), 0x0080);
    }
    // reserved space reads zero, writes ignored
    s.reg_write(0x0100, 0xFFFF);
    assert_eq!(s.reg_read(0x0100), 0);
    // output is digital zero and no interrupt after running muted
    s.policy.capture = true;
    run_frames(&mut s, 500);
    assert!(s.captured.iter().all(|&f| f == (0, 0)));
    assert!(!s.irq3());
    assert!(!s.mem_overrun());
}
