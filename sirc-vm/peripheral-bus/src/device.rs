use std::any::Any;
use std::ops::BitOr;

// TODO: Make sure at some point to not have duplicate exception level definitions
// category=Refactoring
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

impl BitOr for BusOperation {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        match (self, rhs) {
            // In hardware, the bus access strobe output (BRW) is active high where high is "write"
            (Self::Write, _) | (_, Self::Write) => Self::Write,
            _ => Self::Read,
        }
    }
}

impl BitOr for BusAssertions {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self {
            // Ensure that any new fields added to `BusAssertions` are added here.
            address: self.address | rhs.address,
            data: self.data | rhs.data,
            op: self.op | rhs.op,
            interrupt_assertion: self.interrupt_assertion | rhs.interrupt_assertion,
            bus_access_strobe: self.bus_access_strobe | rhs.bus_access_strobe,
            bus_acknowledge: self.bus_acknowledge | rhs.bus_acknowledge,
            bus_error: self.bus_error | rhs.bus_error,
            bus_protection_error: self.bus_protection_error | rhs.bus_protection_error,
            instruction_fetch: self.instruction_fetch | rhs.instruction_fetch,
            device_was_activated: self.device_was_activated | rhs.device_was_activated,
            exit_simulation: self.exit_simulation | rhs.exit_simulation,
            reset_requested: self.reset_requested | rhs.reset_requested,
            reset_devices_on_bus: self.reset_devices_on_bus | rhs.reset_devices_on_bus,
        }
    }
}

/// External interaction with the bus by devices.
///
/// To simulate that all the devices in the simulator are connected to the bus, they return this
/// struct after they are "polled" by the bus and all the assertion are merged together to form
/// the state of the bus, which is then passed back into every device on the next poll.
///
/// The CPU is the "bus master" so some of the fields should only be asserted by the CPU.
/// However, there is no protection around what is asserted because in a real system there is
/// nothing stopping someone from soldering a pin to whatever they want and we might want to
/// simulate what happens with conflicting devices etc.
///
/// Note that the electrical state is _not_ modelled here. Some pins are active high and some are
/// active low, but in the simulator they are all active high (true or 1 means asserted).
/// This means that if you capture the state of the bus in a debugger it is not going to electrically
/// match what the bus on real hardware would look like. This will probably be fixed up at some point
/// now that the reference manual specifies active high/active low for different pins.
#[derive(Debug, Default, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct BusAssertions {
    /// Pins: A0-A23
    pub address: u32,
    /// Pins: D0-D15
    pub data: u16,
    /// Pin: BRW
    pub op: BusOperation,

    /// Simulates the bus connected to the interrupt pins on the CPU
    /// zero is no interrupt, is a bit field
    /// Interrupt assertions from all devices will be merged using additively with the || operator
    /// Pins: IRQ1-IRQ4, NMI
    pub interrupt_assertion: u8,

    /// Asserted by the CPU to indicate that a bus operation should occur.
    /// When a device handles that bus operation it should acknowledge that it was successful by
    /// asserting `bus_acknowledge` (BACK) (or if it is invalid with a `bus_error` (BERR)
    /// or `bus_protection_error` (BPER)).
    /// Pin: BAS
    pub bus_access_strobe: bool,
    /// Set to true by a device responding to a `bus_access_strobe` to indicate the operation
    /// is complete so the CPU can continue. E.g. when a memory read/write is complete.
    /// If the system only has "fast" devices that take one cycle to perform I/O, `bus_acknowledge`
    /// can be permanently asserted.
    /// Pin: BACK
    pub bus_acknowledge: bool,
    /// Set to true to cause a bus fault in the CPU
    /// Usually used for invalid access on devices etc.
    pub bus_error: bool,
    /// Set to true to cause a bus protection fault in the CPU
    /// Used when the address is valid but the device disallowed the access (e.g. memory protection)
    pub bus_protection_error: bool,
    /// Set to true in the first cycle of the execution unit instruction fetch
    /// Could be used as a hint to a memory controller that the next fetch will be sequential
    /// At the moment, just used as a checkpoint for the debugger
    /// TODO: Replace with output of BAT (bus access type) CPU pins
    pub instruction_fetch: bool,
    /// Set to true if any device was mapped to the address during polling
    /// Currently used to warn if no device is mapped for an address range,
    /// probably wouldn't have an equivalent in hardware
    pub device_was_activated: bool,

    /// Set to true to exit the simulation with no error code
    /// Something that only exists in software and required because the hardware never stops
    /// Used to distinguish between programs that run successfully to completion and errors
    pub exit_simulation: bool,

    /// External reset input
    /// When asserted, the reset unit immediately halts the CPU and begins the 6-cycle RSTO hold.
    /// Can be driven by external devices or by the CPU itself (software RSET signals its own RSTI
    /// output to trigger the same reset unit path).
    /// Pin: RSTI
    pub reset_requested: bool,
    /// Asserted during the 6-cycle post-reset hold.
    /// External devices should treat this as a notification to reset their own state.
    /// If 6 cycles is not enough for external devices to reset, you'll either have to have glue
    /// logic to hold rsti active, or have the program add some delays in software.
    /// Pin: RSTO
    pub reset_devices_on_bus: bool,
}

/// Something that interacts with the bus.
///
/// Every clock cycle, the bus will "poll" each device with the current state of the bus, and
/// each device will return their "bus assertions" which will determine the new state of the bus.
///
/// This loop drives the simulation.
///
/// In the hardware, there would not be any polling loop. The devices would just make their
/// assertions via electrical signals (pull down/pull up) and latch state at each clock cycle.
/// This would be very hard to simulate, as much as I want this to be a realistic simulator,
/// we are making the assumption that each device has "settled" by the time it is polled.
pub trait Device {
    /// Called every clock so the device can do work and raise interrupts etc.
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions;
    /// Called by the reset unit to immediately halt the device and prepare it for a reset sequence.
    /// For the CPU: aborts any pending bus transaction, resets phase to 0, seeds the reset cause
    /// value so the EU fetches the reset vector when the hold expires.
    /// Default is a no-op; non-CPU devices typically don't need to do anything here.
    fn reset(&mut self) {}
    fn dump_diagnostic(&self) -> String {
        String::from("TODO")
    }
    // TODO: Refactor bus device interfaces to not need `Any`
    // category=Refactoring
    // This is a hopefully temporary hack that should only be used for testing - remove once a proper way to access CPU registers has been found
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
