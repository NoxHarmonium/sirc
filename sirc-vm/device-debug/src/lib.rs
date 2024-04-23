use std::any::Any;

use log::debug;
use peripheral_bus::{
    device::BusAssertions, device::Device, memory_mapped_device::MemoryMappedDevice,
};

///
/// A device that just exists to do integration tests on the VM
/// Allows programs to assert interrupts etc. when they need to
///
pub struct DebugDevice {
    pub trigger_bus_error: bool,
    pub trigger_interrupt: u8,
}

#[must_use]
pub const fn new_debug_device() -> DebugDevice {
    DebugDevice {
        trigger_bus_error: false,
        trigger_interrupt: 0,
    }
}

impl Device for DebugDevice {
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
        let io_assertions = self.perform_bus_io(bus_assertions, selected);
        // TODO: Bus error should be edge triggered I think so we don't get double faults
        // if devices are slow to stop asserting
        if self.trigger_bus_error {
            debug!("Bus error triggered!");

            self.trigger_bus_error = false;
            return BusAssertions {
                bus_error: true,
                ..io_assertions
            };
        }
        if self.trigger_interrupt > 0 {
            debug!("Interrupt L{} error triggered!", self.trigger_interrupt);

            let interrupt_level = self.trigger_interrupt;
            self.trigger_interrupt = 0;
            return BusAssertions {
                interrupt_assertion: 0x1 << (interrupt_level - 1),
                ..io_assertions
            };
        }
        io_assertions
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl MemoryMappedDevice for DebugDevice {
    fn read_address(&self, address: u32) -> u16 {
        debug!("Reading from address 0x{address:X}");
        match address {
            0x0 => u16::from(self.trigger_bus_error),
            0x1..=0x5 => u16::from(address == u32::from(self.trigger_interrupt)),
            _ => 0x0,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn write_address(&mut self, address: u32, value: u16) {
        debug!("Writing 0x{value:X} to address 0x{address:X}");

        match address {
            0x0 => self.trigger_bus_error = value == 0x1,
            0x1..=0x5 => {
                if value == 0x1 {
                    self.trigger_interrupt = address as u8;
                }
            }
            _ => {}
        }
    }
}
