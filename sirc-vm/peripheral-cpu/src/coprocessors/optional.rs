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
    /// Report an unsupported command with the raw command word as fault metadata.
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
    /// Return the fixed hardware ID used to route CSR commands to this unit.
    /// IDs 0/1 belong to PU/EU; optional IDs must be unique and in 2..=15.
    fn id(&self) -> u8;

    /// Accept a command once, in its initial setup phase 0, and latch operands.
    ///
    /// May update architectural state required at command start. Return an error
    /// to abort the command; the CU supplies the issuing COP PC for the fault.
    fn start(&mut self, command: u16, registers: &mut Registers) -> Result<(), CoprocessorFault>;

    /// Commit one successful bus access, using the saved logical request and ACK data.
    ///
    /// The CU calls this exactly once per ACK, before requesting further work.
    /// Bus errors bypass this method and abort the command. An error returned
    /// here aborts subsequent work without rolling back already committed state.
    fn complete_access(
        &mut self,
        request: BusAssertions,
        response: BusAssertions,
        registers: &mut Registers,
    ) -> Result<(), CoprocessorFault>;

    /// Produce the next request when the shared bus is free, or idle assertions.
    ///
    /// The CU holds a presented request through waits; the unit must not advance
    /// transfer progress until `complete_access` accepts it. An error aborts the
    /// command before a new request reaches the bus. Keep addresses logical so
    /// the CU can retain tags for fault metadata and mask only physical pins.
    fn next_request(&mut self, registers: &Registers) -> Result<BusAssertions, CoprocessorFault>;

    /// Report whether all command work, including final state updates, has finished.
    ///
    /// The CU releases ownership at the next phase-0 boundary; until then this
    /// unit must return idle assertions rather than issuing further accesses.
    fn is_finished(&self) -> bool;

    /// Discard hidden command progress on CPU reset or command abortion.
    ///
    /// Already committed architectural side effects remain. The unit stays
    /// installed and must be ready for a later `start` without stale requests.
    fn reset(&mut self);
}

/// Empty by default: missing hardware remains available for software emulation.
#[derive(Default)]
pub struct CpuConfig {
    pub optional_coprocessors: Vec<Box<dyn OptionalCoprocessor>>,
}
