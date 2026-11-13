//! The SIRCAS shared two-pole state-variable filter.
//!
//! The specification (section 12) leaves the exact recurrence open. The prototype uses the
//! Chamberlin form, which matches the figures in the draft (maximum cutoff of about 5.3 kHz at
//! register `0xFF`, shift-and-add coefficients of `register / 256`):
//!
//! ```text
//! low  += f * band
//! high  = in - low - q * band
//! band += f * high
//! ```
//!
//! The fixed-point version is parametrised on state width, fractional bits and rounding mode so
//! that the study programs can find out which choices the hardware can live with.

use crate::OUTPUT_RATE_HZ;

/// How the product of a state value and an 8-bit coefficient is reduced back to state precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rounding {
    /// Arithmetic shift right (what a bare shift-and-add datapath does). Biased towards -infinity.
    Floor,
    /// Add half an LSB before the shift. Costs one extra adder input.
    Nearest,
    /// Truncate the magnitude (sign-magnitude style). Costs a conditional negate.
    TowardZero,
}

#[derive(Clone, Copy, Debug)]
pub struct SvfConfig {
    /// Total signed state width in bits (the draft says 24).
    pub state_bits: u32,
    /// Fractional bits in the state (the draft says 4).
    pub frac_bits: u32,
    pub rounding: Rounding,
}

impl SvfConfig {
    /// What the draft specification says: 24-bit state, four fractional bits, bare shifts.
    pub const SPEC: Self = Self {
        state_bits: 24,
        frac_bits: 4,
        rounding: Rounding::Floor,
    };

    #[must_use]
    pub const fn max(&self) -> i64 {
        (1i64 << (self.state_bits - 1)) - 1
    }
    #[must_use]
    pub const fn min(&self) -> i64 {
        -(1i64 << (self.state_bits - 1))
    }
}

impl Default for SvfConfig {
    fn default() -> Self {
        Self::SPEC
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterMode {
    LowPass,
    BandPass,
    HighPass,
}

impl FilterMode {
    /// `FILTER_CONFIG[1:0]`. The reserved encoding `11` behaves as low-pass in the prototype.
    #[must_use]
    pub const fn from_bits(bits: u16) -> Self {
        match bits & 3 {
            1 => Self::BandPass,
            2 => Self::HighPass,
            _ => Self::LowPass,
        }
    }
}

/// Damping numerator (over 256) for a resonance register value (draft section 12.2).
#[must_use]
pub const fn damping(resonance: u8) -> u32 {
    if resonance == 255 {
        0
    } else {
        16 + (3 * (255 - resonance as u32)) / 4
    }
}

/// Nominal cutoff in Hz for a cutoff register value, from `f = 2 sin(pi fc / fs)`.
#[must_use]
pub fn cutoff_hz(cutoff: u8) -> f64 {
    let f = f64::from(cutoff) / 256.0;
    OUTPUT_RATE_HZ / std::f64::consts::PI * (f / 2.0).asin()
}

fn shift_round(x: i64, shift: u32, mode: Rounding) -> i64 {
    match mode {
        Rounding::Floor => x >> shift,
        Rounding::Nearest => (x + (1i64 << (shift - 1))) >> shift,
        Rounding::TowardZero => {
            if x < 0 {
                -((-x) >> shift)
            } else {
                x >> shift
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SvfOut {
    pub low: i64,
    pub band: i64,
    pub high: i64,
}

impl SvfOut {
    #[must_use]
    pub const fn select(&self, mode: FilterMode) -> i64 {
        match mode {
            FilterMode::LowPass => self.low,
            FilterMode::BandPass => self.band,
            FilterMode::HighPass => self.high,
        }
    }
}

/// Fixed-point filter. All values are in state units (`frac_bits` below the integer LSB).
#[derive(Clone, Debug)]
pub struct SvfFixed {
    pub cfg: SvfConfig,
    pub low: i64,
    pub band: i64,
    /// Number of times a state or intermediate value hit the saturation limit.
    pub saturations: u64,
}

impl SvfFixed {
    #[must_use]
    pub const fn new(cfg: SvfConfig) -> Self {
        Self {
            cfg,
            low: 0,
            band: 0,
            saturations: 0,
        }
    }

    fn sat(&mut self, x: i64) -> i64 {
        if x > self.cfg.max() {
            self.saturations += 1;
            self.cfg.max()
        } else if x < self.cfg.min() {
            self.saturations += 1;
            self.cfg.min()
        } else {
            x
        }
    }

    /// One output sample. `input` is in state units. Three coefficient multiplications.
    pub fn step(&mut self, input: i64, cutoff: u8, resonance: u8) -> SvfOut {
        let f = i64::from(cutoff);
        let q = i64::from(damping(resonance));
        let r = self.cfg.rounding;

        let fb = shift_round(f * self.band, 8, r);
        self.low = self.sat(self.low + fb);
        let qb = shift_round(q * self.band, 8, r);
        let high = self.sat(input - self.low - qb);
        let fh = shift_round(f * high, 8, r);
        self.band = self.sat(self.band + fh);
        SvfOut {
            low: self.low,
            band: self.band,
            high,
        }
    }
}

/// Double-precision reference with the same recurrence.
#[derive(Clone, Debug, Default)]
pub struct SvfFloat {
    pub low: f64,
    pub band: f64,
}

impl SvfFloat {
    pub fn step(&mut self, input: f64, cutoff: u8, resonance: u8) -> (f64, f64, f64) {
        let f = f64::from(cutoff) / 256.0;
        let q = f64::from(damping(resonance)) / 256.0;
        self.low += f * self.band;
        let high = input - self.low - q * self.band;
        self.band += f * high;
        (self.low, self.band, high)
    }
}
