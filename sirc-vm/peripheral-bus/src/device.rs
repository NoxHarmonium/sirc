use std::any::Any;

// TODO: Make sure at some point to not have duplicate exception level definitions
// The CPU is technically the source of truth because it exposes the pins that the bus connects to
// and does the actual exception handling. However the bus does not depend on the CPU so it can't
// use constants from there. Will have to figure something out.
pub const LEVEL_ONE_INTERRUPT: u8 = 0x1;
pub const LEVEL_TWO_INTERRUPT: u8 = 0x2;
pub const LEVEL_THREE_INTERRUPT: u8 = 0x4;
pub const LEVEL_FOUR_INTERRUPT: u8 = 0x8;
pub const LEVEL_FIVE_INTERRUPT: u8 = 0x10;

#[derive(Debug, Default, Clone, Copy)]
pub enum BusOperation {
    #[default]
    Read,
    Write,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BusAssertions {
    pub address: u32,
    pub data: u16,
    pub op: BusOperation,

    /// Simulates the bus connected to the interrupt pins on the CPU
    /// zero is no interrupt, is a bit field
    /// Interrupt assertions from all devices will be merged using additively with the || operator
    pub interrupt_assertion: u8,
    /// Set to true to cause a bus fault in the CPU
    /// Usually used for invalid access on devices etc.
    pub bus_error: bool,

    /// Set to true to exit the simulation with no error code
    /// Something that only exists in software and required because the hardware never stops
    /// Used to distinguish between programs that run successfully to completion and errors
    pub exit_simulation: bool,
}

pub trait Device {
    /// Called every clock so the device can do work and raise interrupts etc.
    /// TODO: Allow device to return a value(s) to assert bus lines (e.g. interrupts)
    ///       A return value will avoid having to pass in the parent pmem/bus and cause circular dependencies
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions;
    fn dump_diagnostic(&self) -> String {
        String::from("TODO")
    }
    // TODO: This is a hopefully temporary hack that should only be used for testing - remove once a proper way to access CPU registers has been found
    fn as_any(&mut self) -> &mut dyn Any;
}

pub struct StubDevice {}

impl Device for StubDevice {
    fn poll(&mut self, _: BusAssertions, _: bool) -> BusAssertions {
        // No-op
        BusAssertions::default()
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[must_use]
pub fn new_stub_device() -> StubDevice {
    StubDevice {}
}
