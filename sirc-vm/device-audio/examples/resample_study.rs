//! Nearest-neighbour playback quality: how much error and how large a spurious tone does the
//! chip's no-interpolation playback add, by table length and pitch?
//! `cargo run --release -p device_audio --example resample_study`

use device_audio::testkit::{note_freq, spit_entries};
use device_audio::OUTPUT_RATE_HZ;

fn tri(p: f64) -> f64 {
    let p = p - p.floor();
    if p < 0.5 {
        4.0 * p - 1.0
    } else {
        3.0 - 4.0 * p
    }
}
fn sin(p: f64) -> f64 {
    (std::f64::consts::TAU * p).sin()
}

/// Returns (SNR dB of error vs ideal, strongest non-harmonic component in dB re the fundamental).
fn measure(wave: fn(f64) -> f64, len: usize, note: u32) -> (f64, f64) {
    let amp = 105.0;
    let table: Vec<f64> = (0..len)
        .map(|i| (amp * wave(i as f64 / len as f64)).round())
        .collect();
    let e = spit_entries(len as f64);
    let spi = (u64::from(e[(note % 12) as usize]) << (note / 12)) as f64 / 65536.0;
    let n = 4096;
    let (mut sig, mut err) = (0.0, 0.0);
    let mut out = Vec::with_capacity(n);
    let mut pos = 0.0f64;
    for _ in 0..n {
        let ideal = amp * wave(pos / len as f64);
        let nn = table[(pos.floor() as usize) % len];
        sig += ideal * ideal;
        err += (nn - ideal) * (nn - ideal);
        out.push(nn);
        pos += spi;
    }
    // spectrum with a Hann window; compare strongest bin away from harmonics of f0
    let f0 = note_freq(note);
    let bins = n / 2;
    let mut mag = vec![0.0; bins];
    for (k, m) in mag.iter_mut().enumerate().skip(1) {
        let (mut re, mut im) = (0.0, 0.0);
        for (i, v) in out.iter().enumerate() {
            let w = 0.5 - 0.5 * (std::f64::consts::TAU * i as f64 / n as f64).cos();
            let ph = std::f64::consts::TAU * k as f64 * i as f64 / n as f64;
            re += v * w * ph.cos();
            im -= v * w * ph.sin();
        }
        *m = (re * re + im * im).sqrt();
    }
    let hz = |k: usize| k as f64 * OUTPUT_RATE_HZ / n as f64;
    let fund = mag.iter().cloned().fold(0.0, f64::max);
    let mut spur = 0.0f64;
    for (k, m) in mag.iter().enumerate().skip(2) {
        let h = hz(k) / f0;
        let off = (h - h.round()).abs() * f0; // distance in Hz from the nearest harmonic
        if off > 3.0 * OUTPUT_RATE_HZ / n as f64 {
            spur = spur.max(*m);
        }
    }
    (10.0 * (sig / err).log10(), 20.0 * (spur / fund).log10())
}

fn main() {
    println!("Nearest-neighbour playback error vs an ideal waveform (amp 105, 8-bit table).");
    println!("SNR = signal / (playback error); spur = strongest non-harmonic component re the fundamental.\n");
    for (name, wave) in [("triangle", tri as fn(f64) -> f64), ("sine", sin)] {
        println!("== {name} ==");
        println!("note  freq_Hz  bytes/sample |  len 32 SNR/spur | len 64 | len 128 | len 256");
        for note in [24u32, 32, 36, 39, 43, 46, 48, 55, 60, 72] {
            let spi = (u64::from(spit_entries(64.0)[(note % 12) as usize]) << (note / 12)) as f64
                / 65536.0;
            let mut row = format!("{note:4}  {:7.0}  {spi:11.3}  |", note_freq(note));
            for len in [32usize, 64, 128, 256] {
                let (snr, spur) = measure(wave, len, note);
                row += &format!(" {snr:5.1} / {spur:6.1} |");
            }
            println!("{row}");
        }
        println!();
    }
}
