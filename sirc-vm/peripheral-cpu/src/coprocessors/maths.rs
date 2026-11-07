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

mod divide;
mod multiply;
use divide::DivideWork;
use multiply::MultiplyWork;

struct Signs {
    left: bool,
    right: bool,
}
impl Signs {
    fn new(left: u16, right: u16, signed: bool) -> Self {
        Self {
            left: signed && left & SIGN_BIT != 0,
            right: signed && right & SIGN_BIT != 0,
        }
    }
    fn result_negative(&self) -> bool {
        self.left != self.right
    }
}

// Select the operation once. Each variant owns only the states and latches
// relevant to its algorithm, while both advance on the same CU clock.
enum Work {
    Multiply(MultiplyWork),
    Divide(DivideWork),
}
impl Work {
    fn clock(&mut self, registers: &mut Registers) {
        match self {
            Self::Multiply(work) => work.clock(registers),
            Self::Divide(work) => work.clock(registers),
        }
    }
    fn is_finished(&self) -> bool {
        match self {
            Self::Multiply(work) => work.is_finished(),
            Self::Divide(work) => work.is_finished(),
        }
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
        // Inputs are latched once; results remain private until write-back.
        // Unused operand bits do not select a different operation.
        self.work = Some(match command & COPROCESSOR_OPCODE_ID_MASK {
            MULU_OPCODE => Work::Multiply(MultiplyWork::new(registers, false)),
            MULS_OPCODE => Work::Multiply(MultiplyWork::new(registers, true)),
            DIVU_OPCODE => Work::Divide(DivideWork::new(registers, false)),
            DIVS_OPCODE => Work::Divide(DivideWork::new(registers, true)),
            _ => return Err(CoprocessorFault::invalid_command(command)),
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
        self.work.as_ref().is_none_or(Work::is_finished)
    }
    fn reset(&mut self) {
        self.work = None;
    }
}

#[cfg(test)]
mod tests;
