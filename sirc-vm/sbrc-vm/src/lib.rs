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

use std::{cell::RefCell, fs::File, io::Write, path::PathBuf};

use log::{error, info};

use peripheral_bus::{
    device::{BusAssertions, Device},
    BusPeripheral,
};
use peripheral_clock::ClockPeripheral;
use peripheral_cpu::CpuPeripheral;

pub struct Vm {
    pub bus_peripheral: RefCell<BusPeripheral>,
    pub clock_peripheral: RefCell<ClockPeripheral>,
}

#[allow(clippy::borrowed_box)]
fn dump_registers(dump_file: &PathBuf, device: &dyn Device) -> Result<(), std::io::Error> {
    let mut handle = File::create(dump_file)?;

    write!(&mut handle, "{}", device.dump_diagnostic())?;
    Ok(())
}

pub fn run_vm(vm: &Vm, register_dump_file: Option<PathBuf>) {
    // TODO: Can we avoid RefCell if we know that `run_vm` is the only consumer of VM?
    let mut bus_peripheral = vm.bus_peripheral.borrow_mut();
    let clock_peripheral = vm.clock_peripheral.borrow();

    let mut bus_assertions = BusAssertions::default();
    // TODO: Profile and make this actually performant (currently is ,less than 1 fps in a tight loop)
    let execute = |_| {
        bus_assertions = bus_peripheral.poll_all(bus_assertions);
        !bus_assertions.exit_simulation
    };

    clock_peripheral.start_loop(execute);

    if let Some(register_dump_file) = register_dump_file {
        // TODO: This is terrible - again
        let cpu: &CpuPeripheral = bus_peripheral
            .bus_master
            .as_any()
            .downcast_ref::<CpuPeripheral>()
            .expect("failed to downcast");

        info!(
                "Register dump file argument provided. Dumping registers to [{register_dump_file:?}]..."
            );
        if let Err(error) = dump_registers(&register_dump_file, cpu) {
            error!(
                "There was an error dumping registers to [{}].\n{}",
                register_dump_file.display(),
                error
            );
        };
    }
}
