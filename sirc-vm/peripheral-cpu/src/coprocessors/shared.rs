use peripheral_mem::MemoryPeripheral;

use crate::{registers::Registers, Error};

use super::exception_unit::registers::ExceptionUnitRegisters;

pub trait Executor {
    fn step<'a>(
        registers: &'a mut Registers,
        eu_registers: &'a mut ExceptionUnitRegisters,
        mem: &MemoryPeripheral,
    ) -> Result<(&'a Registers, &'a mut ExceptionUnitRegisters, u32), Error>;
}
