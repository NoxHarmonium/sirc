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
}

#[must_use]
pub fn new_debug_device() -> DebugDevice {
    DebugDevice {
        trigger_bus_error: false,
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
            0x0 => {
                if self.trigger_bus_error {
                    0x1
                } else {
                    0x0
                }
            }
            _ => 0x0,
        }
    }

    #[allow(clippy::single_match)]
    fn write_address(&mut self, address: u32, value: u16) {
        debug!("Writing 0x{value:X} to address 0x{address:X}");

        match address {
            0x0 => self.trigger_bus_error = value == 0x1,
            _ => {}
        }
    }
}
