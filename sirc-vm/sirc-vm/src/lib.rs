#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    // I don't like this rule
    clippy::module_name_repetitions,
    // Will tackle this at the next clean up
    clippy::too_many_lines,
    // Might be good practice but too much work for now
    clippy::missing_errors_doc,
    // Not stable yet - try again later
    clippy::missing_const_for_fn,
    // I have a lot of temporary panics for debugging that will probably be cleaned up
    clippy::missing_panics_doc
)]
#![deny(warnings)]

pub mod debug_adapter;
mod debugger;
pub mod utils;

use std::{cell::RefCell, collections::HashSet, fs::File, io::Write, path::PathBuf};

use debug_adapter::types::{BreakpointRef, VmChannels};
use debugger::yield_to_debugger;
use log::{error, info};
use peripheral_bus::{
    device::{BusAssertions, Device},
    BusPeripheral,
};
use peripheral_cpu::CpuPeripheral;
use utils::{cpu_from_bus::cpu_from_bus, frame_reporter::start_loop};

#[cfg(feature = "video")]
use device_video::VSYNC_INTERRUPT;
#[cfg(not(feature = "video"))]
const VSYNC_INTERRUPT: u8 = 0x0;

#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct DebugState {
    pub channels: VmChannels,
    pub breakpoints: HashSet<BreakpointRef>,
    pub disconnected: bool,
    // TODO: Collapse multiple bools in DebugState into an enum
    // category=Refactor
    // - Could do some sort of enum state machine type thing
    // - Also while here, `is_stepping` is kind of named in a confusing way, it is set to true when the debugger pauses
    pub paused: bool,
    pub should_pause_for_init: bool,
    pub is_stepping: bool,
}

pub struct Vm {
    pub bus_peripheral: RefCell<BusPeripheral>,
    pub vsync_frequency: f64,
}

#[allow(clippy::borrowed_box)]
fn dump_registers(dump_file: &PathBuf, device: &dyn Device) -> Result<(), std::io::Error> {
    let mut handle = File::create(dump_file)?;

    write!(&mut handle, "{}", device.dump_diagnostic())?;
    Ok(())
}

// Separate from run_vm so that performance is not affected in non-debug mode
// TODO: Deduplicate `run_vm` functions
// category=Refactoring
pub fn run_vm_debug(vm: &Vm, register_dump_file: Option<PathBuf>, channels: VmChannels) {
    // TODO: Check if RefCell is required for VM state
    // category=Refactoring
    // Can we avoid RefCell if we know that `run_vm` is the only consumer of VM?
    let mut bus_peripheral = vm.bus_peripheral.borrow_mut();

    let mut debug_state = DebugState {
        breakpoints: HashSet::new(),
        channels,
        disconnected: false,
        paused: false,
        should_pause_for_init: true,
        is_stepping: false,
    };

    let mut bus_assertions = BusAssertions::default();
    let execute = || {
        let mut clocks = 0;
        loop {
            bus_assertions = bus_peripheral.poll_all(bus_assertions);

            if !debug_state.disconnected && bus_assertions.instruction_fetch {
                yield_to_debugger(&mut bus_peripheral, &mut debug_state);
            }

            clocks += 1;
            if bus_assertions.interrupt_assertion & VSYNC_INTERRUPT > 0
                || bus_assertions.exit_simulation
            {
                return (bus_assertions.exit_simulation, clocks);
            }
        }
    };

    // TODO: CPU should clock every n master clocks
    // category=Hardware
    // E.g. in the SNES the CPU ran 6 times slower than the master clock
    start_loop(vm.vsync_frequency, execute);

    if let Some(register_dump_file) = register_dump_file {
        let cpu: &CpuPeripheral = cpu_from_bus(&mut bus_peripheral);

        info!(
            "Register dump file argument provided. Dumping registers to [{}]...",
            register_dump_file.display()
        );
        if let Err(error) = dump_registers(&register_dump_file, cpu) {
            error!(
                "There was an error dumping registers to [{}].\n{}",
                register_dump_file.display(),
                error
            );
        }
    }
}

pub fn run_vm(vm: &Vm, register_dump_file: Option<PathBuf>) {
    let mut bus_peripheral = vm.bus_peripheral.borrow_mut();
    let mut bus_assertions = BusAssertions::default();
    let execute = || {
        let mut clocks = 0;
        loop {
            bus_assertions = bus_peripheral.poll_all(bus_assertions);

            clocks += 1;
            if bus_assertions.interrupt_assertion & VSYNC_INTERRUPT > 0
                || bus_assertions.exit_simulation
            {
                return (bus_assertions.exit_simulation, clocks);
            }
        }
    };

    start_loop(vm.vsync_frequency, execute);

    if let Some(register_dump_file) = register_dump_file {
        let cpu: &CpuPeripheral = cpu_from_bus(&mut bus_peripheral);

        info!(
            "Register dump file argument provided. Dumping registers to [{}]...",
            register_dump_file.display()
        );
        if let Err(error) = dump_registers(&register_dump_file, cpu) {
            error!(
                "There was an error dumping registers to [{}].\n{}",
                register_dump_file.display(),
                error
            );
        }
    }
}
