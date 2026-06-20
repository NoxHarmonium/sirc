use crate::device::{BusAssertions, Device};

pub struct ResetUnit {
    hold_cycles: u8,
}

impl Default for ResetUnit {
    fn default() -> Self {
        Self::new()
    }
}

impl ResetUnit {
    #[must_use]
    pub const fn new() -> Self {
        Self { hold_cycles: 0 }
    }

    pub fn should_reset(&mut self, assertions: BusAssertions, bus_master: &mut dyn Device) -> bool {
        if assertions.reset_requested {
            bus_master.reset();
            self.hold_cycles = 5;
            return true;
        }
        if self.hold_cycles > 0 {
            self.hold_cycles -= 1;
            return true;
        }
        false
    }
}
