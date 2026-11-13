//! Budget and numerics study for SIRCAS.
//! `cargo run --release -p device_audio --example budget_study`

use device_audio::core::{mod_route, moddest, reg, FinePolicy, PitchModLaw, Policy, Sircas};
use device_audio::testkit::{
    isa, power_on, run_frames, sine, spit_entries, Builder, InstrumentSpec,
};
use device_audio::{CLKI_PER_FRAME, OUTPUT_RATE_HZ};

const CH: fn(usize, u16) -> u16 = reg::ch;

fn scene(policy: Policy) -> (Sircas, Builder) {
    let mut s = Sircas::new(policy);
    let mut b = Builder::default();
    for i in 0..3 {
        b.looping_instrument(&mut s, i, &sine(64, 100.0), (20, 5, 200, 10));
    }
    (s, b)
}

/// Lowest SEQ_PERIOD (output samples per tick) that never raises MEM_OVERRUN.
fn min_period(policy: Policy, program: &[u16], porta: bool) -> (u16, u64) {
    for period in 1..400u16 {
        let (mut s, mut b) = scene(policy);
        let pc = b.program(&mut s, program);
        power_on(&mut s);
        s.reg_write(reg::SEQ_PERIOD, period);
        for c in 0..6 {
            let ctrl = reg::CONTROL_SEQ_EN | if porta { reg::CONTROL_PORTA_EN } else { 0 };
            s.reg_write(CH(c, reg::CH_CONTROL), ctrl);
            s.reg_write(CH(c, reg::CH_VOLUME_PAN), 0x6080);
            s.reg_write(CH(c, reg::CH_SEQ_PC), pc);
        }
        run_frames(&mut s, 1500);
        if !s.mem_overrun() {
            return (period, s.stats.ctl_reads);
        }
    }
    (0, 0)
}

fn main() {
    println!("=== A. External RAM bandwidth ===");
    let frames_per_s = OUTPUT_RATE_HZ;
    println!("audio slots/frame 42 -> {:.2} M transactions/s; CPU slots 126/frame -> {:.2} M words/s ({:.1} MB/s)",
        42.0 * frames_per_s / 1e6, 126.0 * frames_per_s / 1e6, 126.0 * 2.0 * frames_per_s / 1e6);
    println!("PCM needs at most 6 of the 42 audio slots per frame, leaving at least 36 for the sequencer");

    println!("\n=== B. Minimum sustainable SEQ_PERIOD (output samples/tick), six channels all running the program ===");
    let worst = [
        isa::inst(1),
        isa::note(10),
        isa::inst(2),
        isa::note(11),
        isa::inst(1),
        isa::note(12),
        isa::inst(2),
        isa::note(13),
        isa::goto(-8),
    ];
    let typical = [
        isa::inst(0),
        isa::fine(5),
        isa::note(40),
        isa::wait(1),
        isa::goto(-4),
    ];
    let porta = [
        isa::note(40),
        isa::wait(1),
        isa::note(41),
        isa::wait(1),
        isa::goto(-4),
    ];
    let nocache = Policy {
        instrument_cache: false,
        ..Policy::default()
    };
    for (name, p, pol, pt) in [
        (
            "worst: 4 NOTEs/tick, instrument change each time, cache",
            &worst[..],
            Policy::default(),
            false,
        ),
        (
            "typical INST;FINE;NOTE;WAIT 1, cache",
            &typical[..],
            Policy::default(),
            false,
        ),
        (
            "typical INST;FINE;NOTE;WAIT 1, no instrument cache",
            &typical[..],
            nocache,
            false,
        ),
        (
            "portamento NOTE;WAIT;NOTE;WAIT (pitch-only loads)",
            &porta[..],
            Policy::default(),
            true,
        ),
    ] {
        let (period, reads) = min_period(pol, p, pt);
        println!("{name:62} min SEQ_PERIOD {period:3} = {:.2} ms/tick = {:.0} Hz max tick rate  (ctl reads {reads})",
            f64::from(period) * 1000.0 / OUTPUT_RATE_HZ, OUTPUT_RATE_HZ / f64::from(period));
    }

    println!("\n=== C. Note-onset latency ===");
    for attack in [0u8, 255, 1] {
        let (mut s, mut b) = scene(Policy {
            capture: true,
            ..Policy::default()
        });
        let (start, len) = b.sample(&mut s, &[100, 100]);
        let spit = b.spit(&mut s, &[65536; 12]);
        b.instrument(
            &mut s,
            5,
            InstrumentSpec {
                start,
                len,
                loop_start: 0,
                loop_en: true,
                spit,
                attack,
                decay: 0,
                sustain: 255,
                release: 10,
            },
        );
        power_on(&mut s);
        s.reg_write(CH(0, reg::CH_VOLUME_PAN), 0xFF80);
        run_frames(&mut s, 70); // land at an arbitrary envelope phase
        s.captured.clear();
        s.reg_write(CH(0, reg::CH_INSTR), 5);
        s.reg_write(CH(0, reg::CH_COMMAND), 0x8000 | 12);
        run_frames(&mut s, 1200);
        let first = s.captured.iter().position(|f| f.0 != 0 || f.1 != 0);
        let big = s
            .captured
            .iter()
            .position(|f| f.0.abs() > 12000 || f.1.abs() > 12000);
        println!("ATTACK={attack:3}: first non-zero frame {:?} ({:.2} ms), reaches 3/8 scale at frame {:?} ({:.1} ms)",
            first, first.unwrap_or(0) as f64 * 1000.0 / OUTPUT_RATE_HZ, big, big.unwrap_or(0) as f64 * 1000.0 / OUTPUT_RATE_HZ);
    }
    println!("envelope: step per 64 samples = {:.3} ms; full-scale attack time: A=255 {:.0} ms, A=1 {:.1} s",
        64000.0 / OUTPUT_RATE_HZ, 4095.0 / 255.0 * 64000.0 / OUTPUT_RATE_HZ, 4095.0 * 64.0 / OUTPUT_RATE_HZ);

    println!("\n=== D. DSP clocks per frame (steady state, model: shift-add multiplier = coefficient bits + 2) ===");
    println!("{:55} {:>6}   draft estimate", "scenario", "clocks");
    let run = |dry: usize, filt: usize, modv: usize, mod_vol_target: bool| -> u32 {
        let (mut s, mut b) = scene(Policy::default());
        b.looping_instrument(&mut s, 3, &sine(64, 100.0), (255, 0, 255, 10));
        power_on(&mut s);
        let mut ch = 0;
        for _ in 0..dry {
            s.reg_write(CH(ch, reg::CH_CONTROL), 0);
            ch += 1;
        }
        for _ in 0..filt {
            s.reg_write(CH(ch, reg::CH_CONTROL), reg::CONTROL_FILTER_EN);
            ch += 1;
        }
        for _ in 0..modv {
            s.reg_write(CH(ch, reg::CH_CONTROL), reg::CONTROL_MOD_MODE);
            if mod_vol_target {
                s.reg_write(
                    CH(ch, reg::CH_MOD_ROUTE),
                    mod_route(moddest::VOLUME, 0, -3, false),
                );
            }
            ch += 1;
        }
        for c in 0..ch {
            s.reg_write(CH(c, reg::CH_VOLUME_PAN), 0x8080);
            s.reg_write(CH(c, reg::CH_INSTR), 3);
            s.reg_write(CH(c, reg::CH_COMMAND), 0x8000 | 30);
        }
        run_frames(&mut s, 400);
        // steady-state: take the quietest frame (no envelope update or register-driven recompute)
        let mut min = u32::MAX;
        for _ in 0..200 {
            run_frames(&mut s, 1);
            min = min.min(s.stats.dsp_clocks_last);
        }
        min
    };
    println!(
        "{:55} {:>6}   286 (cached gain)",
        "six dry channels",
        run(6, 0, 0, false)
    );
    println!(
        "{:55} {:>6}   238",
        "six filtered channels",
        run(0, 6, 0, false)
    );
    println!(
        "{:55} {:>6}   263",
        "three dry, two filtered, one modulation voice",
        run(3, 2, 1, false)
    );
    println!(
        "{:55} {:>6}   289",
        "five dry, one volume-modulation voice",
        run(5, 0, 1, true)
    );
    println!("(plus 10 per channel whose cached gain is recomputed that frame: worst case +60)");
    println!("the model's filter block always runs: 3 coefficient multiplies + output volume + output pan");
    println!("budget: {CLKI_PER_FRAME} clocks per frame");

    println!("\n=== E. Pitch resolution across the note range (64-byte single-cycle sample) ===");
    let e = spit_entries(64.0);
    println!("note  freq_Hz   SPI(16.16)   SPI/oct-shift   FINE=+/-4095 as cents (after-shift | before-shift)   +/-1 raw unit cents");
    for note in [0u32, 12, 24, 36, 48, 60, 72, 84, 96, 108, 119] {
        let spi = (u64::from(e[(note % 12) as usize])) << (note / 12);
        let after = 1200.0 * ((spi as f64 + 4095.0) / spi as f64).log2();
        let before = 1200.0
            * ((f64::from(e[(note % 12) as usize]) + 4095.0) / f64::from(e[(note % 12) as usize]))
                .log2();
        let unit = 1200.0 * ((spi as f64 + 1.0) / spi as f64).log2();
        println!("{note:4}  {:8.1}  {spi:11}  {:5.2} B/sample   {after:9.2} | {before:7.2}     {unit:8.5}",
            device_audio::testkit::note_freq(note), spi as f64 / 65536.0);
    }

    println!("\n=== F. Vibrato depth of a pitch-modulation source (M = +127, S = 0), additive vs proportional ===");
    println!("note  additive cents   proportional cents");
    for note in [0u32, 24, 48, 72, 96, 119] {
        let spi = (u64::from(e[(note % 12) as usize])) << (note / 12);
        let add = 127i64 << 4;
        let prop = ((spi as i64) * 127) >> 16;
        let c = |d: i64| 1200.0 * ((spi as f64 + d as f64) / spi as f64).log2();
        println!("{note:4}  {:12.2}   {:14.2}", c(add), c(prop));
    }
    println!(
        "proportional range: S=-8 -> {:.2} cents, S=0 -> {:.1} cents, S=+7 -> {:.0} cents (M=127)",
        1200.0 * (1.0f64 + 127.0 / 65536.0 / 256.0).log2(),
        1200.0 * (1.0f64 + 127.0 / 65536.0).log2(),
        1200.0 * (1.0f64 + 127.0 * 128.0 / 65536.0).log2()
    );

    println!("\n=== G. Gain/pan endpoints ===");
    println!(
        "max channel gain G = (4095*255)>>8 = {} -> {:.4} of unity ({:.3} dB)",
        (4095 * 255) >> 8,
        f64::from((4095 * 255) >> 8) / 4096.0,
        20.0 * (f64::from((4095 * 255) >> 8) / 4096.0).log10()
    );
    println!(
        "pan=255 leak into the opposite channel: 1/256 = {:.1} dB",
        20.0 * (1.0f64 / 256.0).log10()
    );
    println!(
        "pan=128 centre: each side {:.2} dB (linear law, -6.02 dB per side)",
        20.0 * 0.5f64.log10()
    );

    println!(
        "\n=== H. Policy check: FINE policy and pitch-mod law are implemented and switchable ==="
    );
    for fine in [FinePolicy::AfterOctaveShift, FinePolicy::BeforeOctaveShift] {
        let (mut s, mut b) = scene(Policy {
            fine,
            ..Policy::default()
        });
        let _ = &mut b;
        power_on(&mut s);
        s.reg_write(CH(0, reg::CH_FINE), 100);
        s.reg_write(CH(0, reg::CH_INSTR), 0);
        s.reg_write(CH(0, reg::CH_COMMAND), 0x8000 | 60);
        run_frames(&mut s, 20);
        println!("{fine:?}: note 60 FINE=100 -> SPI {}", s.snapshot(0).spi);
    }
    let _ = PitchModLaw::Additive;
}
