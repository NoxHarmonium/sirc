//! Iterative integer maths hardware sharing the CU clock.
//!
//! A 16-bit word datapath plus carry handles one add/shift or shift/subtract
//! pair per operand bit. No host multiply/divide computes a result in advance.
use super::{
    optional::{CoprocessorFault, OptionalCoprocessor},
    shared::ExecutionPhase,
};
use crate::{registers::Registers, COPROCESSOR_OPCODE_ID_MASK};
use peripheral_bus::device::BusAssertions;

const MULU_OPCODE: u16 = 0x0000;
const MULS_OPCODE: u16 = 0x0100;
const DIVU_OPCODE: u16 = 0x0200;
const DIVS_OPCODE: u16 = 0x0300;
const WORD_BITS: u8 = 16;
const SIGN_BIT: u16 = 0x8000;
const WORD_MASK: u32 = 0xffff;
const ERROR: u16 = 1 << 0;
const DIVIDE_BY_ZERO: u16 = 1 << 1;
const QUOTIENT_OVERFLOW: u16 = 1 << 2;

#[derive(Clone, Copy)]
enum Operation {
    Multiply,
    Divide,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    NormalizeLow,
    NormalizeHigh,
    NormalizeOperand,
    CheckOperands,
    MultiplyAdd,
    MultiplyShift,
    DivideShift,
    DivideSubtract,
    CheckResult,
    SignLow,
    SignHigh,
    WriteBack,
    Done,
}
struct Signs {
    left: bool,
    right: bool,
}
impl Signs {
    fn result_negative(&self) -> bool {
        self.left != self.right
    }
}
struct Work {
    operation: Operation,
    stage: Stage,
    // Word-sized latches. Multiply uses low as the shifting multiplier;
    // division uses high:low as the remainder:quotient register pair.
    high: u16,
    low: u16,
    operand: u16,
    // Holds only a 17-bit sum/remainder, including the datapath carry bit.
    wide: u32,
    bits_left: u8,
    signs: Signs,
    signed: bool,
    carry: bool,
    status: u16,
}
impl Work {
    fn clock(&mut self, registers: &mut Registers) {
        self.stage = match self.stage {
            Stage::NormalizeLow => {
                match self.operation {
                    Operation::Multiply if self.signs.left => self.high = self.high.wrapping_neg(),
                    Operation::Divide if self.signs.left => {
                        self.low = self.low.wrapping_neg();
                        self.carry = self.low == 0;
                    }
                    _ => {}
                }
                Stage::NormalizeHigh
            }
            Stage::NormalizeHigh => {
                match self.operation {
                    Operation::Multiply if self.signs.right => self.low = self.low.wrapping_neg(),
                    Operation::Divide if self.signs.left => {
                        self.high = (!self.high).wrapping_add(u16::from(self.carry));
                    }
                    _ => {}
                }
                Stage::NormalizeOperand
            }
            Stage::NormalizeOperand => {
                match self.operation {
                    Operation::Multiply => self.operand = self.high,
                    Operation::Divide if self.signs.right => {
                        self.operand = self.operand.wrapping_neg();
                    }
                    Operation::Divide => {}
                }
                Stage::CheckOperands
            }
            Stage::CheckOperands => match self.operation {
                Operation::Multiply => Stage::MultiplyAdd,
                Operation::Divide => {
                    // A high word >= divisor already implies a >16-bit quotient.
                    // Zero takes precedence, and neither error overwrites r1:r2.
                    self.status = if self.operand == 0 {
                        ERROR | DIVIDE_BY_ZERO
                    } else if self.high >= self.operand {
                        ERROR | QUOTIENT_OVERFLOW
                    } else {
                        0
                    };
                    if self.status == 0 {
                        Stage::DivideShift
                    } else {
                        Stage::WriteBack
                    }
                }
            },
            Stage::MultiplyAdd => {
                if self.low & 1 != 0 {
                    self.wide += u32::from(self.operand);
                }
                Stage::MultiplyShift
            }
            Stage::MultiplyShift => {
                self.low = (self.low >> 1) | if self.wide & 1 != 0 { SIGN_BIT } else { 0 };
                self.wide >>= 1;
                self.bits_left -= 1;
                if self.bits_left == 0 {
                    Stage::CheckResult
                } else {
                    Stage::MultiplyAdd
                }
            }
            Stage::DivideShift => {
                self.wide = (u32::from(self.high) << 1) | u32::from(self.low >> (WORD_BITS - 1));
                self.low <<= 1;
                Stage::DivideSubtract
            }
            Stage::DivideSubtract => {
                // A subtractor's borrow selects the old shifted remainder on
                // failure (restoring division), or the difference on success.
                let (difference, borrow) = self.wide.overflowing_sub(u32::from(self.operand));
                self.high = if borrow {
                    u16::try_from(self.wide).expect("completed word must fit the datapath")
                } else {
                    self.low |= 1;
                    u16::try_from(difference).expect("restored remainder is below the divisor")
                };
                self.bits_left -= 1;
                if self.bits_left == 0 {
                    Stage::CheckResult
                } else {
                    Stage::DivideShift
                }
            }
            Stage::CheckResult => {
                if matches!(self.operation, Operation::Divide) && self.signed {
                    let limit = if self.signs.result_negative() {
                        SIGN_BIT
                    } else {
                        SIGN_BIT - 1
                    };
                    if self.low > limit {
                        self.status = ERROR | QUOTIENT_OVERFLOW;
                    }
                }
                Stage::SignLow
            }
            Stage::SignLow => {
                if self.signs.result_negative() {
                    self.low = self.low.wrapping_neg();
                    self.carry = self.low == 0;
                }
                Stage::SignHigh
            }
            Stage::SignHigh => {
                match self.operation {
                    Operation::Multiply if self.signs.result_negative() => {
                        self.wide = ((!self.wide & WORD_MASK) + u32::from(self.carry)) & WORD_MASK;
                    }
                    // Remainder has the dividend's sign, independent of divisor.
                    Operation::Divide if self.signs.left => self.high = self.high.wrapping_neg(),
                    _ => {}
                }
                Stage::WriteBack
            }
            Stage::WriteBack => {
                if self.status == 0 {
                    registers.r1 = match self.operation {
                        Operation::Multiply => {
                            u16::try_from(self.wide).expect("completed word must fit the datapath")
                        }
                        Operation::Divide => self.high,
                    };
                    registers.r2 = self.low;
                }
                registers.r3 = self.status;
                Stage::Done
            }
            Stage::Done => Stage::Done,
        };
    }
}

/// Optional integer maths unit with a fixed per-bit sequential datapath.
#[derive(Default)]
pub struct MathsCoprocessor {
    work: Option<Work>,
}
impl MathsCoprocessor {
    /// Standard integer maths hardware ID in the CSR command word.
    pub const COPROCESSOR_ID: u8 = 3;
}
impl OptionalCoprocessor for MathsCoprocessor {
    fn id(&self) -> u8 {
        Self::COPROCESSOR_ID
    }
    fn start(&mut self, command: u16, registers: &mut Registers) -> Result<(), CoprocessorFault> {
        let (operation, signed) = match command & COPROCESSOR_OPCODE_ID_MASK {
            MULU_OPCODE => (Operation::Multiply, false),
            MULS_OPCODE => (Operation::Multiply, true),
            DIVU_OPCODE => (Operation::Divide, false),
            DIVS_OPCODE => (Operation::Divide, true),
            _ => return Err(CoprocessorFault::invalid_command(command)),
        };
        // Unused operand bits do not select an operation. Inputs are latched
        // once and architectural outputs remain unchanged until write-back.
        self.work = Some(Work {
            operation,
            signed,
            stage: Stage::NormalizeLow,
            high: registers.r1,
            low: registers.r2,
            operand: registers.r3,
            wide: 0,
            bits_left: WORD_BITS,
            carry: false,
            status: 0,
            signs: Signs {
                left: signed && registers.r1 & SIGN_BIT != 0,
                right: signed
                    && match operation {
                        Operation::Multiply => registers.r2,
                        Operation::Divide => registers.r3,
                    } & SIGN_BIT
                        != 0,
            },
        });
        Ok(())
    }
    fn step(
        &mut self,
        _: ExecutionPhase,
        registers: &mut Registers,
    ) -> Result<BusAssertions, CoprocessorFault> {
        if let Some(work) = &mut self.work {
            work.clock(registers);
        }
        Ok(BusAssertions::default())
    }
    fn complete_access(
        &mut self,
        _: BusAssertions,
        _: BusAssertions,
        _: &mut Registers,
    ) -> Result<(), CoprocessorFault> {
        unreachable!("integer maths never requests a bus access")
    }
    fn is_finished(&self) -> bool {
        self.work.as_ref().is_none_or(|w| w.stage == Stage::Done)
    }
    fn reset(&mut self) {
        self.work = None;
    }
}

#[cfg(test)]
mod tests;
