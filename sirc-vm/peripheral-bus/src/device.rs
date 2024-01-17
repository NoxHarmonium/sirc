// TODO: Make sure at some point to not have duplicate exception level definitions
// The CPU is technically the source of truth because it exposes the pins that the bus connects to
// and does the actual exception handling. However the bus does not depend on the CPU so it can't
// use constants from there. Will have to figure something out.
pub const LEVEL_ONE_INTERRUPT: u8 = 0x1;
pub const LEVEL_TWO_INTERRUPT: u8 = 0x2;
pub const LEVEL_THREE_INTERRUPT: u8 = 0x4;
pub const LEVEL_FOUR_INTERRUPT: u8 = 0x8;
pub const LEVEL_FIVE_INTERRUPT: u8 = 0x10;

#[derive(Debug, Default)]
pub struct BusAssertions {
    /// Simulates the bus connected to the interrupt pins on the CPU
    /// zero is no interrupt, is a bit field
    /// Interrupt assertions from all devices will be ORed together
    pub interrupt_assertion: u8,
    /// Set to true to cause a bus fault in the CPU
    /// Usually used for invalid access on devices etc.
    pub bus_error: bool,
}

pub trait Device {
    /// Called every clock so the device can do work and raise interrupts etc.
    /// TODO: Allow device to return a value(s) to assert bus lines (e.g. interrupts)
    ///       A return value will avoid having to pass in the parent pmem/bus and cause circular dependencies
    fn poll(&mut self) -> BusAssertions;
}

pub struct StubDevice {}

impl Device for StubDevice {
    fn poll(&mut self) -> BusAssertions {
        // No-op
        BusAssertions::default()
    }
}

#[must_use]
pub fn new_stub_device() -> StubDevice {
    StubDevice {}
}
