//! Helpers for building scenes in audio RAM: assembler, waveforms, pitch tables, instruments.
//! Used by the acceptance tests and the study programs.

use crate::core::Sircas;
use crate::{CLKI_PER_FRAME, OUTPUT_RATE_HZ};

/// Sequencer assembler: every instruction is one 16-bit word.
pub mod isa {
    #[must_use]
    pub const fn note(n: u8) -> u16 {
        n as u16
    }
    #[must_use]
    pub const fn stop() -> u16 {
        1 << 13
    }
    #[must_use]
    pub const fn wait(ticks: u16) -> u16 {
        (2 << 13) | (ticks & 0x1FFF)
    }
    #[must_use]
    pub const fn inst(i: u8) -> u16 {
        (3 << 13) | i as u16
    }
    #[must_use]
    pub const fn fine(v: i16) -> u16 {
        (4 << 13) | ((v as u16) & 0x1FFF)
    }
    /// Displacement is in instruction words, measured from the instruction after the `GOTO`.
    #[must_use]
    pub const fn goto(disp: i16) -> u16 {
        (5 << 13) | ((disp as u16) & 0x1FFF)
    }
    #[must_use]
    pub const fn end(count: u8) -> u16 {
        (6 << 13) | count as u16
    }
}

#[must_use]
pub fn note_freq(note: u32) -> f64 {
    27.5 * 2f64.powf(f64::from(note) / 12.0)
}

#[must_use]
pub fn sine(len: usize, amp: f64) -> Vec<i8> {
    (0..len)
        .map(|i| (amp * (std::f64::consts::TAU * i as f64 / len as f64).sin()).round() as i8)
        .collect()
}

#[must_use]
pub fn saw(len: usize, amp: f64) -> Vec<i8> {
    (0..len)
        .map(|i| (amp * (2.0 * i as f64 / len as f64 - 1.0)).round() as i8)
        .collect()
}

#[must_use]
pub fn square(len: usize, amp: f64) -> Vec<i8> {
    (0..len)
        .map(|i| if i < len / 2 { amp as i8 } else { -(amp as i8) })
        .collect()
}

/// The 12 16.16 pitch-table entries for a sample holding one cycle in `cycle_bytes` bytes.
#[must_use]
pub fn spit_entries(cycle_bytes: f64) -> [u32; 12] {
    let mut e = [0u32; 12];
    for (n, slot) in e.iter_mut().enumerate() {
        let spi = note_freq(n as u32) * cycle_bytes / OUTPUT_RATE_HZ;
        *slot = (spi * 65536.0).round() as u32;
    }
    e
}

#[derive(Clone, Copy, Debug)]
pub struct InstrumentSpec {
    pub start: u16,
    pub len: u16,
    pub loop_start: u16,
    pub loop_en: bool,
    pub spit: u16,
    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,
}

/// Bump allocator over audio RAM. Instrument table at RAM word 0 (CPU word `0x8000`).
pub struct Builder {
    pub next_byte: usize,
}

pub const INSTR_TABLE_BYTES: usize = 256 * 16;

impl Default for Builder {
    fn default() -> Self {
        Self {
            next_byte: INSTR_TABLE_BYTES,
        }
    }
}

impl Builder {
    fn align(&mut self, to: usize) {
        self.next_byte = self.next_byte.div_ceil(to) * to;
    }

    pub fn sample(&mut self, s: &mut Sircas, data: &[i8]) -> (u16, u16) {
        self.align(2);
        let start = self.next_byte;
        for (i, b) in data.iter().enumerate() {
            s.set_ram_byte(start + i, *b as u8);
        }
        self.next_byte += data.len();
        (start as u16, data.len() as u16)
    }

    pub fn spit(&mut self, s: &mut Sircas, entries: &[u32; 12]) -> u16 {
        self.align(4);
        let at = self.next_byte;
        for (i, e) in entries.iter().enumerate() {
            let w = (at >> 1) + i * 2;
            s.set_ram_word(w, (e >> 16) as u16);
            s.set_ram_word(w + 1, *e as u16);
        }
        self.next_byte += 48;
        at as u16
    }

    pub fn instrument(&mut self, s: &mut Sircas, index: usize, spec: InstrumentSpec) {
        let w = index * 8;
        s.set_ram_word(w, spec.start);
        s.set_ram_word(w + 1, spec.len);
        s.set_ram_word(w + 2, spec.loop_start);
        s.set_ram_word(w + 3, spec.spit);
        s.set_ram_word(w + 4, (u16::from(spec.attack) << 8) | u16::from(spec.decay));
        s.set_ram_word(
            w + 5,
            (u16::from(spec.sustain) << 8) | u16::from(spec.release),
        );
        s.set_ram_word(w + 6, if spec.loop_en { 0x8000 } else { 0 });
        s.set_ram_word(w + 7, 0);
    }

    /// Store a program; returns its CPU word address (for `SEQ_PC`).
    pub fn program(&mut self, s: &mut Sircas, words: &[u16]) -> u16 {
        self.align(2);
        let at = self.next_byte >> 1;
        for (i, w) in words.iter().enumerate() {
            s.set_ram_word(at + i, *w);
        }
        self.next_byte += words.len() * 2;
        0x8000 + at as u16
    }

    /// A looping one-cycle waveform instrument with its own pitch table.
    pub fn looping_instrument(
        &mut self,
        s: &mut Sircas,
        index: usize,
        wave: &[i8],
        adsr: (u8, u8, u8, u8),
    ) {
        let (start, len) = self.sample(s, wave);
        let spit = self.spit(s, &spit_entries(wave.len() as f64));
        self.instrument(
            s,
            index,
            InstrumentSpec {
                start,
                len,
                loop_start: 0,
                loop_en: true,
                spit,
                attack: adsr.0,
                decay: adsr.1,
                sustain: adsr.2,
                release: adsr.3,
            },
        );
    }
}

pub fn run_frames(s: &mut Sircas, frames: u64) {
    for _ in 0..frames * u64::from(CLKI_PER_FRAME) {
        s.clock();
    }
}

/// Enable audio and set master state used by most tests.
pub fn power_on(s: &mut Sircas) {
    use crate::core::reg;
    s.reg_write(reg::CONFIG, 0x8000);
}

#[must_use]
pub fn triangle(len: usize, amp: f64) -> Vec<i8> {
    (0..len)
        .map(|i| {
            let p = i as f64 / len as f64;
            let v = if p < 0.5 {
                4.0 * p - 1.0
            } else {
                3.0 - 4.0 * p
            };
            (amp * v).round() as i8
        })
        .collect()
}

/// Triangle that starts at zero and rises, so a note begins without a step.
#[must_use]
pub fn triangle_from_zero(len: usize, amp: f64) -> Vec<i8> {
    (0..len)
        .map(|i| {
            let p = i as f64 / len as f64;
            let v = if p < 0.25 {
                4.0 * p
            } else if p < 0.75 {
                2.0 - 4.0 * p
            } else {
                4.0 * p - 4.0
            };
            (amp * v).round() as i8
        })
        .collect()
}

/// Zero-mean pulse wave with the given duty cycle (0..1): high level `2*amp*(1-duty)`, low level
/// `-2*amp*duty`, so the level is `amp` at 50 % and there is no DC offset.
#[must_use]
pub fn pulse(len: usize, duty: f64, amp: f64) -> Vec<i8> {
    let hi = (2.0 * amp * (1.0 - duty)).round() as i8;
    let lo = (-2.0 * amp * duty).round() as i8;
    (0..len)
        .map(|i| {
            if (i as f64) < duty * len as f64 {
                hi
            } else {
                lo
            }
        })
        .collect()
}

/// White noise from a 32-bit xorshift generator (deterministic).
#[must_use]
pub fn noise(len: usize, seed: u32, amp: f64) -> Vec<i8> {
    let mut x = seed.max(1);
    (0..len)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            (amp * (f64::from(x >> 8) / f64::from(1u32 << 24) * 2.0 - 1.0)).round() as i8
        })
        .collect()
}

/// Write 16-bit stereo PCM as a WAV file.
///
/// # Errors
/// Returns any I/O error from creating or writing the file.
pub fn write_wav(path: &std::path::Path, frames: &[(i16, i16)], rate: u32) -> std::io::Result<()> {
    use std::io::Write;
    let data_len = (frames.len() * 4) as u32;
    let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + data_len).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?; // PCM
    f.write_all(&2u16.to_le_bytes())?; // channels
    f.write_all(&rate.to_le_bytes())?;
    f.write_all(&(rate * 4).to_le_bytes())?;
    f.write_all(&4u16.to_le_bytes())?;
    f.write_all(&16u16.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&data_len.to_le_bytes())?;
    for (l, r) in frames {
        f.write_all(&l.to_le_bytes())?;
        f.write_all(&r.to_le_bytes())?;
    }
    f.flush()
}
