//! Filter study: measures the fixed-point SVF against the float reference.
//! `cargo run --release -p device_audio --example filter_study`

use device_audio::filter::{cutoff_hz, damping, Rounding, SvfConfig, SvfFixed, SvfFloat};
use device_audio::OUTPUT_RATE_HZ;

const FS: f64 = OUTPUT_RATE_HZ;

fn cfg(frac: u32, r: Rounding) -> SvfConfig {
    SvfConfig {
        state_bits: 24,
        frac_bits: frac,
        rounding: r,
    }
}

/// Magnitude of the response at `freq` from a long sine run (steady state), in dB re input.
fn gain_db_fixed(c: SvfConfig, cutoff: u8, res: u8, freq: f64, amp: f64, which: u8) -> f64 {
    let mut f = SvfFixed::new(c);
    let n = 6000;
    let (mut si, mut so, mut cnt) = (0.0, 0.0, 0);
    for i in 0..n {
        let x = amp * (std::f64::consts::TAU * freq * i as f64 / FS).sin();
        let xin = (x * f64::from(1u32 << c.frac_bits)).round() as i64;
        let o = f.step(xin, cutoff, res);
        let y = match which {
            0 => o.low,
            1 => o.band,
            _ => o.high,
        } as f64
            / f64::from(1u32 << c.frac_bits);
        if i > n / 2 {
            si += x * x;
            so += y * y;
            cnt += 1;
        }
    }
    let _ = cnt;
    10.0 * ((so + 1e-12) / (si + 1e-12)).log10()
}

fn gain_db_float(cutoff: u8, res: u8, freq: f64, which: u8) -> f64 {
    let mut f = SvfFloat::default();
    let n = 6000;
    let (mut si, mut so) = (0.0, 0.0);
    for i in 0..n {
        let x = (std::f64::consts::TAU * freq * i as f64 / FS).sin();
        let (l, b, h) = f.step(x, cutoff, res);
        let y = match which {
            0 => l,
            1 => b,
            _ => h,
        };
        if i > n / 2 {
            si += x * x;
            so += y * y;
        }
    }
    10.0 * ((so + 1e-12) / (si + 1e-12)).log10()
}

/// Peak of the float band-pass response, found by scanning (true filter centre frequency).
fn bp_peak_hz(cutoff: u8, res: u8) -> (f64, f64) {
    let mut best = (0.0, -200.0);
    let mut fr = 10.0;
    while fr < 15000.0 {
        let g = gain_db_float(cutoff, res, fr, 1);
        if g > best.1 {
            best = (fr, g);
        }
        fr *= 1.02;
    }
    best
}

fn main() {
    println!("=== 1. Cutoff map (float reference) ===");
    println!("reg  nominal_fc_Hz  measured_BP_peak_Hz  step_to_prev_semitones");
    let mut prev = 0.0;
    for c in [
        1u8, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192, 255,
    ] {
        let (pk, _) = bp_peak_hz(c, 100);
        let nom = cutoff_hz(c);
        let step = if prev > 0.0 {
            12.0 * (nom / prev).log2()
        } else {
            0.0
        };
        println!("{c:3}  {nom:12.1}  {pk:18.1}  {step:8.2}");
        prev = nom;
    }
    println!("adjacent-register step in semitones (the zipper size of a cutoff sweep):");
    for c in [1u8, 2, 4, 8, 16, 32, 64, 128, 254] {
        println!(
            "  {c:3}->{:3}: {:6.2} st",
            c + 1,
            12.0 * (cutoff_hz(c + 1) / cutoff_hz(c)).log2()
        );
    }

    println!("\n=== 2. Resonance map: damping and peak gain (cutoff reg 64, float) ===");
    println!("res  q*256  Q=1/q  band-pass peak gain dB");
    for r in [0u8, 64, 128, 192, 224, 240, 250, 254] {
        let q = f64::from(damping(r)) / 256.0;
        let (_, g) = bp_peak_hz(64, r);
        println!("{r:3}  {:5}  {:5.1}  {g:6.1}", damping(r), 1.0 / q);
    }

    println!("\n=== 3. Fixed vs float LP gain at the cutoff, mid amplitude (amp=2000) ===");
    println!("cutoff  float_dB   floor/4  nearest/4  tozero/4  floor/8  nearest/8");
    for c in [2u8, 8, 32, 128, 255] {
        let f = cutoff_hz(c);
        let fl = gain_db_float(c, 40, f, 0);
        let a = gain_db_fixed(cfg(4, Rounding::Floor), c, 40, f, 2000.0, 0);
        let b = gain_db_fixed(cfg(4, Rounding::Nearest), c, 40, f, 2000.0, 0);
        let d = gain_db_fixed(cfg(4, Rounding::TowardZero), c, 40, f, 2000.0, 0);
        let e = gain_db_fixed(cfg(8, Rounding::Floor), c, 40, f, 2000.0, 0);
        let g = gain_db_fixed(cfg(8, Rounding::Nearest), c, 40, f, 2000.0, 0);
        println!("{c:6}  {fl:8.2}  {a:8.2}  {b:8.2}  {d:8.2}  {e:8.2}  {g:8.2}");
    }

    println!("\n=== 4. Low-level signals: passband LP gain at 20 Hz, cutoff reg 4, by input amplitude ===");
    println!("amp   float   floor/4  nearest/4  tozero/4");
    for amp in [4.0, 16.0, 64.0, 256.0, 2000.0] {
        let fl = gain_db_float(4, 40, 20.0, 0);
        let a = gain_db_fixed(cfg(4, Rounding::Floor), 4, 40, 20.0, amp, 0);
        let b = gain_db_fixed(cfg(4, Rounding::Nearest), 4, 40, 20.0, amp, 0);
        let d = gain_db_fixed(cfg(4, Rounding::TowardZero), 4, 40, 20.0, amp, 0);
        println!("{amp:5.0} {fl:6.2} {a:8.2} {b:9.2} {d:9.2}");
    }

    println!("\n=== 5. Return to silence: impulse (amp 30000), then 5 s of zeros; residual |low|,|band| in output LSBs ===");
    println!("rounding / frac    res=0           res=128         res=250         res=254");
    for (name, c) in [
        ("floor/4  ", cfg(4, Rounding::Floor)),
        ("nearest/4", cfg(4, Rounding::Nearest)),
        ("tozero/4 ", cfg(4, Rounding::TowardZero)),
        ("floor/8  ", cfg(8, Rounding::Floor)),
    ] {
        let mut line = format!("{name}        ");
        for res in [0u8, 128, 250, 254] {
            // worst over a few cutoffs
            let mut worst = 0.0f64;
            for cut in [4u8, 32, 128, 255] {
                let mut f = SvfFixed::new(c);
                f.step(30000 << c.frac_bits, cut, res);
                for _ in 0..(5.0 * FS) as usize {
                    f.step(0, cut, res);
                }
                let m = (f.low.abs().max(f.band.abs())) as f64 / f64::from(1u32 << c.frac_bits);
                worst = worst.max(m);
            }
            line += &format!("{worst:10.1}      ");
        }
        println!("{line}");
    }

    println!("\n=== 6. Self-oscillation at res=255 (q=0): amplitude of band after kick, at 0.1 s and 5 s ===");
    println!("cutoff  float(0.1s/5s)         floor/4          nearest/4        tozero/4         floor/8   nearest/8");
    for cut in [4u8, 16, 64, 128, 255] {
        let mut fl = SvfFloat::default();
        fl.step(2000.0, cut, 255);
        let mut res = vec![];
        let mut fx: Vec<SvfFixed> = [
            cfg(4, Rounding::Floor),
            cfg(4, Rounding::Nearest),
            cfg(4, Rounding::TowardZero),
            cfg(8, Rounding::Floor),
            cfg(8, Rounding::Nearest),
        ]
        .iter()
        .map(|c| {
            let mut f = SvfFixed::new(*c);
            f.step(2000 << c.frac_bits, cut, 255);
            f
        })
        .collect();
        let t1 = (0.1 * FS) as usize;
        let t2 = (5.0 * FS) as usize;
        let amp = |a: &mut SvfFloat, n: usize| {
            let mut m = 0.0f64;
            for _ in 0..n {
                let (_, b, _) = a.step(0.0, cut, 255);
                m = m.max(b.abs());
            }
            m
        };
        let f1 = amp(&mut fl, t1);
        let f2 = amp(&mut fl, t2 - t1);
        for f in &mut fx {
            let sc = f64::from(1u32 << f.cfg.frac_bits);
            let mut m1 = 0.0f64;
            for _ in 0..t1 {
                let o = f.step(0, cut, 255);
                m1 = m1.max(o.band.abs() as f64 / sc);
            }
            let mut m2 = 0.0f64;
            for _ in 0..(t2 - t1) {
                let o = f.step(0, cut, 255);
                m2 = m2.max(o.band.abs() as f64 / sc);
            }
            res.push(format!("{m1:6.0}/{m2:<6.0}"));
        }
        println!("{cut:5}   {f1:6.0}/{f2:<6.0}       {}", res.join("   "));
    }

    println!("\n=== 7. Stability at extremes: full-scale square-ish input for 2 s, saturation events ===");
    println!("cutoff res   saturations  final |band|");
    for (cut, res) in [
        (255u8, 255u8),
        (255, 254),
        (128, 255),
        (32, 255),
        (255, 0),
        (0, 0),
        (1, 255),
    ] {
        let mut f = SvfFixed::new(SvfConfig::SPEC);
        for i in 0..(2.0 * FS) as usize {
            let x = if (i / 40) % 2 == 0 { 196_608 } else { -196_608 };
            f.step(x << 4, cut, res);
        }
        println!(
            "{cut:5} {res:3}   {:11}  {:12}",
            f.saturations,
            f.band.abs() >> 4
        );
    }

    println!("\n=== 8. Cutoff register 0: state freeze ===");
    {
        let mut f = SvfFixed::new(SvfConfig::SPEC);
        for _ in 0..2000 {
            f.step(10_000 << 4, 20, 100);
        }
        let before = (f.low, f.band);
        for _ in 0..1000 {
            f.step(0, 0, 100);
        }
        println!(
            "state with cutoff 20 after 2000 samples of DC 10000: low={} band={}",
            before.0 >> 4,
            before.1 >> 4
        );
        println!(
            "after switching to cutoff 0 and input 0 for 1000 samples:   low={} band={}  (frozen)",
            f.low >> 4,
            f.band >> 4
        );
    }

    println!(
        "\n=== 9. Signal-to-noise: 440 Hz sine amp 8000 through LP, fixed vs float, SNR dB ==="
    );
    println!("cutoff   floor/4  nearest/4  tozero/4  floor/8");
    for cut in [8u8, 32, 64, 128, 255] {
        let mut row = format!("{cut:5} ");
        for c in [
            cfg(4, Rounding::Floor),
            cfg(4, Rounding::Nearest),
            cfg(4, Rounding::TowardZero),
            cfg(8, Rounding::Floor),
        ] {
            let mut fx = SvfFixed::new(c);
            let mut fl = SvfFloat::default();
            let (mut sig, mut err) = (0.0f64, 0.0f64);
            let sc = f64::from(1u32 << c.frac_bits);
            for i in 0..(2.0 * FS) as usize {
                let x = 8000.0 * (std::f64::consts::TAU * 440.0 * i as f64 / FS).sin();
                let o = fx.step((x * sc).round() as i64, cut, 40);
                let (l, _, _) = fl.step(x, cut, 40);
                if i > 4000 {
                    let y = o.low as f64 / sc;
                    sig += l * l;
                    err += (y - l) * (y - l);
                }
            }
            row += &format!("{:9.1}", 10.0 * (sig / (err + 1e-9)).log10());
        }
        println!("{row}");
    }
}
