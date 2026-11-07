//! Divide-only sequencer: normalize, check, shift/subtract, sign and commit.
use super::{Registers, Signs, DIVIDE_BY_ZERO, ERROR, QUOTIENT_OVERFLOW, SIGN_BIT, WORD_BITS};

#[derive(Clone, Copy, PartialEq, Eq)]
enum DivideStage {
    NormalizeDividendLow,
    NormalizeDividendHigh,
    NormalizeDivisor,
    CheckOperands,
    Shift,
    Subtract,
    CheckQuotient,
    SignQuotient,
    SignRemainder,
    WriteBack,
    Done,
}

pub(super) struct DivideWork {
    stage: DivideStage,
    pub(super) remainder: u16,
    pub(super) quotient: u16,
    divisor: u16,
    // Includes the seventeenth bit carried out of the remainder word.
    pub(super) shifted_remainder: u32,
    pub(super) bits_left: u8,
    signs: Signs,
    signed: bool,
    carry: bool,
    status: u16,
}
impl DivideWork {
    /// Latch the operands and signs before advancing the sequencer.
    pub(super) fn new(registers: &Registers, signed: bool) -> Self {
        Self {
            stage: DivideStage::NormalizeDividendLow,
            remainder: registers.r1,
            quotient: registers.r2,
            divisor: registers.r3,
            shifted_remainder: 0,
            bits_left: WORD_BITS,
            signs: Signs::new(registers.r1, registers.r3, signed),
            signed,
            carry: false,
            status: 0,
        }
    }

    /// Advance exactly one arithmetic stage on an advancing CU cycle.
    pub(super) fn clock(&mut self, registers: &mut Registers) {
        self.stage = match self.stage {
            DivideStage::NormalizeDividendLow => {
                if self.signs.left {
                    self.quotient = self.quotient.wrapping_neg();
                    self.carry = self.quotient == 0;
                }
                DivideStage::NormalizeDividendHigh
            }
            DivideStage::NormalizeDividendHigh => {
                if self.signs.left {
                    self.remainder = (!self.remainder).wrapping_add(u16::from(self.carry));
                }
                DivideStage::NormalizeDivisor
            }
            DivideStage::NormalizeDivisor => {
                if self.signs.right {
                    self.divisor = self.divisor.wrapping_neg();
                }
                DivideStage::CheckOperands
            }
            DivideStage::CheckOperands => {
                // Zero wins over overflow. A high word >= divisor already
                // proves a >16-bit quotient; neither error overwrites r1:r2.
                self.status = if self.divisor == 0 {
                    ERROR | DIVIDE_BY_ZERO
                } else if self.remainder >= self.divisor {
                    ERROR | QUOTIENT_OVERFLOW
                } else {
                    0
                };
                if self.status == 0 {
                    DivideStage::Shift
                } else {
                    DivideStage::WriteBack
                }
            }
            DivideStage::Shift => {
                self.shifted_remainder =
                    (u32::from(self.remainder) << 1) | u32::from(self.quotient >> (WORD_BITS - 1));
                self.quotient <<= 1;
                DivideStage::Subtract
            }
            DivideStage::Subtract => {
                // Borrow selects the original shifted remainder; otherwise
                // retain the difference and set the next quotient bit.
                let (difference, borrow) = self
                    .shifted_remainder
                    .overflowing_sub(u32::from(self.divisor));
                self.remainder = if borrow {
                    u16::try_from(self.shifted_remainder)
                        .expect("completed word must fit the datapath")
                } else {
                    self.quotient |= 1;
                    u16::try_from(difference).expect("restored remainder is below the divisor")
                };
                self.bits_left -= 1;
                if self.bits_left == 0 {
                    DivideStage::CheckQuotient
                } else {
                    DivideStage::Shift
                }
            }
            DivideStage::CheckQuotient => {
                if self.signed {
                    let limit = if self.signs.result_negative() {
                        SIGN_BIT
                    } else {
                        SIGN_BIT - 1
                    };
                    if self.quotient > limit {
                        self.status = ERROR | QUOTIENT_OVERFLOW;
                    }
                }
                DivideStage::SignQuotient
            }
            DivideStage::SignQuotient => {
                if self.signs.result_negative() {
                    self.quotient = self.quotient.wrapping_neg();
                }
                DivideStage::SignRemainder
            }
            DivideStage::SignRemainder => {
                // The remainder follows the dividend, independent of divisor.
                if self.signs.left {
                    self.remainder = self.remainder.wrapping_neg();
                }
                DivideStage::WriteBack
            }
            DivideStage::WriteBack => {
                if self.status == 0 {
                    registers.r1 = self.remainder;
                    registers.r2 = self.quotient;
                }
                registers.r3 = self.status;
                DivideStage::Done
            }
            DivideStage::Done => DivideStage::Done,
        };
    }

    /// Report completion only after architectural write-back.
    pub(super) fn is_finished(&self) -> bool {
        self.stage == DivideStage::Done
    }
}
