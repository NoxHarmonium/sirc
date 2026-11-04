//! Multiply-only sequencer: normalize, add/shift, correct sign, commit.
use super::{Registers, Signs, SIGN_BIT, WORD_BITS, WORD_MASK};

#[derive(Clone, Copy, PartialEq, Eq)]
enum MultiplyStage {
    NormalizeMultiplicand,
    NormalizeMultiplier,
    LatchAddend,
    PrepareIterations,
    Add,
    Shift,
    FinishIterations,
    SignLow,
    SignHigh,
    WriteBack,
    Done,
}

pub(super) struct MultiplyWork {
    stage: MultiplyStage,
    multiplicand: u16,
    addend: u16,
    pub(super) multiplier: u16,
    // A 16-bit accumulator plus the carry entering the next shift.
    pub(super) accumulator: u32,
    pub(super) bits_left: u8,
    signs: Signs,
    carry: bool,
}
impl MultiplyWork {
    /// Latch the operands and signs before advancing the sequencer.
    pub(super) fn new(registers: &Registers, signed: bool) -> Self {
        Self {
            stage: MultiplyStage::NormalizeMultiplicand,
            multiplicand: registers.r1,
            multiplier: registers.r2,
            addend: 0,
            accumulator: 0,
            bits_left: WORD_BITS,
            signs: Signs::new(registers.r1, registers.r2, signed),
            carry: false,
        }
    }

    /// Advance exactly one arithmetic stage on an advancing CU cycle.
    pub(super) fn clock(&mut self, registers: &mut Registers) {
        self.stage = match self.stage {
            MultiplyStage::NormalizeMultiplicand => {
                if self.signs.left {
                    self.multiplicand = self.multiplicand.wrapping_neg();
                }
                MultiplyStage::NormalizeMultiplier
            }
            MultiplyStage::NormalizeMultiplier => {
                if self.signs.right {
                    self.multiplier = self.multiplier.wrapping_neg();
                }
                MultiplyStage::LatchAddend
            }
            MultiplyStage::LatchAddend => {
                self.addend = self.multiplicand;
                MultiplyStage::PrepareIterations
            }
            // These allocated cycles preserve the reference unit's schedule;
            // multiplication needs no operand or quotient-range check.
            MultiplyStage::PrepareIterations => MultiplyStage::Add,
            MultiplyStage::Add => {
                if self.multiplier & 1 != 0 {
                    self.accumulator += u32::from(self.addend);
                }
                MultiplyStage::Shift
            }
            MultiplyStage::Shift => {
                self.multiplier = (self.multiplier >> 1)
                    | if self.accumulator & 1 != 0 {
                        SIGN_BIT
                    } else {
                        0
                    };
                self.accumulator >>= 1;
                self.bits_left -= 1;
                if self.bits_left == 0 {
                    MultiplyStage::FinishIterations
                } else {
                    MultiplyStage::Add
                }
            }
            MultiplyStage::FinishIterations => MultiplyStage::SignLow,
            MultiplyStage::SignLow => {
                if self.signs.result_negative() {
                    self.multiplier = self.multiplier.wrapping_neg();
                    self.carry = self.multiplier == 0;
                }
                MultiplyStage::SignHigh
            }
            MultiplyStage::SignHigh => {
                if self.signs.result_negative() {
                    self.accumulator =
                        ((!self.accumulator & WORD_MASK) + u32::from(self.carry)) & WORD_MASK;
                }
                MultiplyStage::WriteBack
            }
            MultiplyStage::WriteBack => {
                registers.r1 =
                    u16::try_from(self.accumulator).expect("completed word must fit the datapath");
                registers.r2 = self.multiplier;
                registers.r3 = 0;
                MultiplyStage::Done
            }
            MultiplyStage::Done => MultiplyStage::Done,
        };
    }

    /// Report completion only after architectural write-back.
    pub(super) fn is_finished(&self) -> bool {
        self.stage == MultiplyStage::Done
    }
}
