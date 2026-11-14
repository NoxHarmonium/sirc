//! I2S serialiser model. A pure function of the sample stream, so the framing can be tested
//! bit-exactly and alternatives (BCLK periods, slot width) compared.
//!
//! The format is standard I2S: `WS` changes one BCLK before the MSB of the word it announces, so
//! with a slot exactly as wide as the word the LSB of a word falls in the first BCLK of the next
//! slot, after `WS` has changed.

use crate::CLKI_PER_FRAME;

/// Width of a word in bits.
pub const WORD_BITS: u32 = 16;

#[derive(Clone, Copy, Debug)]
pub struct I2sFormat {
    /// CLKI cycles of the BCLK periods of the frame, alternating: even-numbered periods first.
    pub periods: (u32, u32),
    /// BCLK periods per channel slot.
    pub slot_bits: u32,
}

impl I2sFormat {
    /// The format in the manual: 16-bit slots, 32 BCLK periods of 10 and 11 cycles alternately.
    pub const SIXTEEN: Self = Self {
        periods: (10, 11),
        slot_bits: 16,
    };
    /// Earlier alternative: 24-bit slots, `BCLK = CLKI / 7`, 16 data bits and 7 padding bits.
    pub const DRAFT: Self = Self {
        periods: (7, 7),
        slot_bits: 24,
    };
    /// Alternative: `BCLK = CLKI / 8` (exact 50 % duty) with 21-bit slots.
    pub const DIV8: Self = Self {
        periods: (8, 8),
        slot_bits: 21,
    };

    #[must_use]
    pub const fn bclks_per_frame(&self) -> u32 {
        2 * self.slot_bits
    }

    /// CLKI cycles in BCLK period `b` of the frame.
    #[must_use]
    pub const fn period_cycles(&self, b: u32) -> u32 {
        if b.is_multiple_of(2) {
            self.periods.0
        } else {
            self.periods.1
        }
    }

    /// CLKI cycles in the whole frame.
    #[must_use]
    pub const fn frame_cycles(&self) -> u32 {
        let n = self.bclks_per_frame();
        // n is even, so there are n/2 periods of each length
        (n / 2) * (self.periods.0 + self.periods.1)
    }

    /// Does the format exactly fill one 336-cycle sample period?
    #[must_use]
    pub const fn fills_frame(&self) -> bool {
        self.frame_cycles() == CLKI_PER_FRAME
    }
}

/// `(WS, SD)` for every BCLK of the stream of stereo frames `(left, right)`. WS low is left. The
/// output has `bclks_per_frame` entries for each input frame; the LSB of the last right word falls
/// in the first BCLK of the frame that follows, so a final frame is needed to carry it.
#[must_use]
pub fn serialise(fmt: I2sFormat, frames: &[(i16, i16)]) -> Vec<(bool, bool)> {
    let slot = fmt.slot_bits;
    let n = fmt.bclks_per_frame();
    let periods = frames.len() as u32 * n;
    // Bit `k` of a slot is BCLK `k` counted from the slot's WS edge; the MSB is k = 1.
    let word_bit = |slot_index: i64, k: u32| -> bool {
        if slot_index < 0 || !(1..=WORD_BITS).contains(&k) {
            return false;
        }
        let frame = (slot_index / 2) as usize;
        let Some(&(l, r)) = frames.get(frame) else {
            return false;
        };
        let word = if slot_index % 2 == 0 { l } else { r } as u16;
        (word >> (WORD_BITS - k)) & 1 == 1
    };
    (0..periods)
        .map(|g| {
            let j = i64::from(g / slot);
            let k = g % slot;
            let ws = j % 2 == 1;
            // The bit comes from this slot, or is the tail of the previous slot's word.
            let sd = word_bit(j, k) || word_bit(j - 1, k + slot);
            (ws, sd)
        })
        .collect()
}

/// Inverse of [`serialise`], for round-trip tests. One frame is returned for every whole frame of
/// `bits`. The LSB of the last right word is lost if it lies beyond the end, so a test appends a
/// final frame to the input and compares the others.
#[must_use]
pub fn deserialise(fmt: I2sFormat, bits: &[(bool, bool)]) -> Vec<(i16, i16)> {
    let slot = fmt.slot_bits;
    let n = fmt.bclks_per_frame();
    let frames = bits.len() as u32 / n;
    let mut words = vec![0u16; 2 * frames as usize];
    for (g, &(_, sd)) in bits.iter().enumerate() {
        let g = g as u32;
        // This period holds bit k of slot j (k from 1 to 16) for the latest slot whose WS edge it
        // follows, or bit k + slot of the slot before.
        for back in 0..=1u32 {
            let j = (g / slot).checked_sub(back);
            let Some(j) = j else { continue };
            let k = g - j * slot;
            if (1..=WORD_BITS).contains(&k) && (j as usize) < words.len() && sd {
                words[j as usize] |= 1 << (WORD_BITS - k);
            }
        }
    }
    words
        .chunks(2)
        .map(|w| (w[0] as i16, w[1] as i16))
        .collect()
}
