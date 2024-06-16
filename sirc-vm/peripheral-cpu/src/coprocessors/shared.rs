use peripheral_bus::device::BusAssertions;

use crate::registers::{ExceptionUnitRegisters, Registers};

#[derive(Default, FromPrimitive, ToPrimitive, Debug, PartialEq, Eq, Copy, Clone)]
pub enum ExecutionPhase {
    #[default]
    InstructionFetchLow = 0x0,
    InstructionFetchHigh = 0x1,
    InstructionDecode = 0x2,
    ExecutionEffectiveAddressExecutor = 0x3,
    MemoryAccessExecutor = 0x4,
    WriteBackExecutor = 0x5,
}

pub trait Executor {
    const COPROCESSOR_ID: u8;

    fn step<'a>(
        &mut self,
        phase: &ExecutionPhase,
        cause_register_value: u16,
        registers: &'a mut Registers,
        eu_registers: &'a mut ExceptionUnitRegisters,
        bus_assertions: BusAssertions,
    ) -> BusAssertions;
}
