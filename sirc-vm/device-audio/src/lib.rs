//! SIRCAS (SIRC Audio System) prototype device.
//!
//! This crate is a *behavioural reference model* of the audio chip described in the draft
//! specification, clocked at `CLKI` resolution so that memory slot scheduling, deadlines and
//! DSP cycle budgets can be measured. It is not RTL. See `docs/audio/STATUS.md`.

pub use sirc_umf::*;
pub use sirc_umf::{core, filter, i2s, testkit};

use std::any::Any;

use peripheral_bus::device::{BusAssertions, BusOperation, Device, LEVEL_THREE_INTERRUPT};
use peripheral_bus::memory_mapped_device::{MemoryMapped, MemoryMappedDevice};

/// SIRCAS on the SIRC peripheral bus. One `poll` is one `CLKI` cycle.
pub struct AudioDevice {
    pub core: Sircas,
    ram_busy: bool,
}

#[must_use]
pub fn new_audio_device(policy: Policy) -> AudioDevice {
    AudioDevice {
        core: Sircas::new(policy),
        ram_busy: false,
    }
}

impl Device for AudioDevice {
    fn poll(&mut self, bus: BusAssertions, selected: bool) -> BusAssertions {
        let mut out = BusAssertions::default();
        let mut register_ack = false;
        if selected && bus.bus_access_strobe && !self.ram_busy {
            let addr = (bus.address & 0xFFFF) as u16;
            if addr < RAM_WINDOW {
                // Register accesses are acknowledged without waiting for RAM slots.
                match bus.op {
                    BusOperation::Read => out.data = self.core.reg_read(addr),
                    BusOperation::Write => self.core.reg_write(addr, bus.data),
                }
                register_ack = true;
            } else {
                let accepted = self.core.cpu_request(CpuReq {
                    write: matches!(bus.op, BusOperation::Write),
                    word: addr - RAM_WINDOW,
                    data: bus.data,
                });
                self.ram_busy = accepted;
            }
        }

        self.core.clock();

        if register_ack {
            out.bus_acknowledge = true;
            out.device_was_activated = true;
        } else if self.ram_busy {
            out.device_was_activated = true;
            if let Some(data) = self.core.cpu_take_result() {
                self.ram_busy = false;
                out.bus_acknowledge = true;
                out.data = data;
            }
        }
        if self.core.irq3() {
            out.interrupt_assertion |= LEVEL_THREE_INTERRUPT;
        }
        out
    }

    fn reset(&mut self) {
        self.core.reset_registers();
        self.ram_busy = false;
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl MemoryMapped for AudioDevice {
    /// Debugger backdoor: no timing, no side effects.
    fn read_address(&self, address: u32) -> u16 {
        let a = (address & 0xFFFF) as u16;
        if a < RAM_WINDOW {
            self.core.reg_read(a)
        } else {
            self.core.ram_word(usize::from(a - RAM_WINDOW))
        }
    }
    fn write_address(&mut self, address: u32, value: u16) {
        let a = (address & 0xFFFF) as u16;
        if a < RAM_WINDOW {
            self.core.reg_write(a, value);
        } else {
            self.core.set_ram_word(usize::from(a - RAM_WINDOW), value);
        }
    }
}

impl MemoryMappedDevice for AudioDevice {
    /// Raw reads and writes address the audio RAM image from word zero (not the register block).
    fn read_raw_bytes(&self, limit: u32) -> Vec<u8> {
        self.core
            .ram_image()
            .iter()
            .take(limit as usize)
            .flat_map(|w| w.to_be_bytes())
            .collect()
    }
    fn write_raw_bytes(&mut self, binary_data: &[u8]) {
        for (i, pair) in binary_data.chunks(2).enumerate() {
            let hi = pair[0];
            let lo = pair.get(1).copied().unwrap_or(0);
            self.core.set_ram_word(i, u16::from_be_bytes([hi, lo]));
        }
    }
}
