//! I2S serialiser model (draft section 3.2). Pure function of the frame, so the framing can be
//! tested bit-exactly and alternatives (BCLK divider, slot width) compared.

use crate::CLKI_PER_FRAME;

#[derive(Clone, Copy, Debug)]
pub struct I2sFormat {
    /// CLKI cycles per BCLK.
    pub bclk_div: u32,
    /// BCLK cycles per channel slot.
    pub slot_bits: u32,
}

impl I2sFormat {
    /// The draft's provisional choice: 24-bit slots, `BCLK = CLKI / 7`.
    pub const DRAFT: Self = Self {
        bclk_div: 7,
        slot_bits: 24,
    };
    /// Alternative: `BCLK = CLKI / 8` (exact 50 % duty) with 21-bit slots.
    pub const DIV8: Self = Self {
        bclk_div: 8,
        slot_bits: 21,
    };

    #[must_use]
    pub const fn bclks_per_frame(&self) -> u32 {
        2 * self.slot_bits
    }
    /// Does the format exactly fill one 336-cycle sample period?
    #[must_use]
    pub const fn fills_frame(&self) -> bool {
        self.bclks_per_frame() * self.bclk_div == CLKI_PER_FRAME
    }
    /// Low and high CLKI cycles of one BCLK period (a divide-by-odd gives unequal halves).
    #[must_use]
    pub const fn bclk_duty(&self) -> (u32, u32) {
        (self.bclk_div.div_ceil(2), self.bclk_div / 2)
    }
}

/// `(WS, SD)` for every BCLK of one stereo frame. WS low = left. Standard I2S: WS changes one BCLK
/// before the MSB; data is MSB first, followed by zero padding.
#[must_use]
pub fn serialise(fmt: I2sFormat, left: i16, right: i16) -> Vec<(bool, bool)> {
    let n = fmt.bclks_per_frame();
    (0..n)
        .map(|b| {
            let ws = b >= fmt.slot_bits;
            let slot_start = if ws { fmt.slot_bits } else { 0 };
            let word = if ws { right } else { left } as u16;
            // MSB appears one BCLK after the WS edge.
            let sd = (slot_start + 1..slot_start + 17).contains(&b) && {
                let bit = 15 - (b - slot_start - 1);
                (word >> bit) & 1 == 1
            };
            (ws, sd)
        })
        .collect()
}

/// Inverse of [`serialise`], for round-trip tests.
#[must_use]
pub fn deserialise(fmt: I2sFormat, bits: &[(bool, bool)]) -> (i16, i16) {
    let mut left = 0u16;
    let mut right = 0u16;
    for (b, &(ws, sd)) in bits.iter().enumerate() {
        let b = b as u32;
        let slot_start = if ws { fmt.slot_bits } else { 0 };
        if (slot_start + 1..slot_start + 17).contains(&b) {
            let bit = u16::from(sd) << (15 - (b - slot_start - 1));
            if ws {
                right |= bit;
            } else {
                left |= bit;
            }
        }
    }
    (left as i16, right as i16)
}
