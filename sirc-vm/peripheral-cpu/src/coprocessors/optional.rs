//! Runtime-installed optional hardware shares the CU's clock and held bus request.
use crate::{coprocessors::exception_unit::definitions::Faults, registers::Registers};
use peripheral_bus::device::{BusAccessType, BusAssertions};

/// Fault raised by an optional unit, before a request or while accepting its result.
/// The CU supplies the issuing COP PC; addresses remain logical until bus presentation.
#[derive(Debug, Clone, Copy)]
pub struct CoprocessorFault {
    pub fault: Faults,
    pub address: u32,
    pub bus_access_type: BusAccessType,
}

impl CoprocessorFault {
    #[must_use]
    pub fn invalid_command(command: u16) -> Self {
        Self {
            fault: Faults::InvalidOpCode,
            address: u32::from(command),
            bus_access_type: BusAccessType::None,
        }
    }
}

/// A compiled-in hardware plugin sharing the CU phase clock.
///
/// The CU calls `start` once in setup phase 0,
/// `complete_access` once per successful request, and `next_request`
/// only when the bus is free. A finished command retires at the next slot boundary.
/// Implementations must not maintain their own phase clock or poll the bus.
pub trait OptionalCoprocessor {
    fn id(&self) -> u8;
    fn start(&mut self, command: u16, registers: &mut Registers) -> Result<(), CoprocessorFault>;
    /// Consume a successful ACK only; the CU handles bus errors before this call.
    fn complete_access(
        &mut self,
        request: BusAssertions,
        response: BusAssertions,
        registers: &mut Registers,
    ) -> Result<(), CoprocessorFault>;
    fn next_request(&mut self, registers: &Registers) -> Result<BusAssertions, CoprocessorFault>;
    fn is_finished(&self) -> bool;
    /// Abort hidden progress without rolling back architectural side effects.
    fn reset(&mut self);
}

/// Empty by default: missing hardware remains available for software emulation.
#[derive(Default)]
pub struct CpuConfig {
    pub optional_coprocessors: Vec<Box<dyn OptionalCoprocessor>>,
}
