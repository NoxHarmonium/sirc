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
pub mod utils;

use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    fs::File,
    io::Write,
    path::PathBuf,
    sync::mpsc::TryRecvError,
};

use debug_adapter::types::{
    BreakpointRef, DebuggerMessage, ResumeCondition, VmChannels, VmMessage, VmPauseReason, VmState,
};
use device_video::VSYNC_INTERRUPT;
use log::{error, info};
use peripheral_bus::{
    device::{BusAssertions, Device},
    BusPeripheral,
};
use peripheral_clock::ClockPeripheral;
use peripheral_cpu::{registers::FullAddressRegisterAccess, CpuPeripheral};

#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct DebugState {
    pub channels: VmChannels,
    pub breakpoints: HashSet<BreakpointRef>,
    pub disconnected: bool,
    // TODO: some sort of enum state machine type thing
    pub paused: bool,
    pub should_pause_for_init: bool,
    pub is_stepping: bool,
}

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

pub fn cpu_from_bus(bus_peripheral: &mut BusPeripheral) -> &CpuPeripheral {
    // TODO: This is weird. Is there a better way?
    bus_peripheral
        .bus_master
        .as_any()
        .downcast_ref::<CpuPeripheral>()
        .expect("failed to downcast")
}

pub fn handle_debug_message(message: DebuggerMessage, debug_state: &mut DebugState) {
    match message {
        DebuggerMessage::UpdateBreakpoints(breakpoints) => {
            debug_state.breakpoints = breakpoints;
        }
        DebuggerMessage::PauseVm => {
            // TODO: This naming is confusing
            debug_state.is_stepping = true;
        }
        DebuggerMessage::ResumeVm(condition) => {
            debug_state.paused = false;
            match condition {
                ResumeCondition::UntilNextStep => {
                    debug_state.is_stepping = true;
                }
                ResumeCondition::None => {}
            }
        }
        DebuggerMessage::Disconnect => debug_state.disconnected = true,
    };
}

pub fn notify_debugger(bus_peripheral: &mut BusPeripheral, debug_state: &mut DebugState) {
    let cpu = cpu_from_bus(bus_peripheral);
    let pc = cpu.registers.get_full_pc_address();
    let vm_state = VmState {
        pc,
        // TODO: More variables and move this somewhere
        variables: BTreeMap::from([
            ("sr".to_string(), format!("0x{:X}", cpu.registers.sr)),
            ("r1".to_string(), format!("0x{:X}", cpu.registers.r1)),
            ("r2".to_string(), format!("0x{:X}", cpu.registers.r2)),
            ("r3".to_string(), format!("0x{:X}", cpu.registers.r3)),
            ("r4".to_string(), format!("0x{:X}", cpu.registers.r4)),
            ("r5".to_string(), format!("0x{:X}", cpu.registers.r5)),
            ("r6".to_string(), format!("0x{:X}", cpu.registers.r6)),
            ("r7".to_string(), format!("0x{:X}", cpu.registers.r7)),
            ("lh".to_string(), format!("0x{:X}", cpu.registers.lh)),
            ("ll".to_string(), format!("0x{:X}", cpu.registers.ll)),
            ("ah".to_string(), format!("0x{:X}", cpu.registers.ah)),
            ("al".to_string(), format!("0x{:X}", cpu.registers.al)),
            ("sh".to_string(), format!("0x{:X}", cpu.registers.sh)),
            ("sl".to_string(), format!("0x{:X}", cpu.registers.sl)),
            ("ph".to_string(), format!("0x{:X}", cpu.registers.ph)),
            ("pl".to_string(), format!("0x{:X}", cpu.registers.pl)),
        ]),
    };

    // Check for pending messages (e.g. update breakpoints)
    loop {
        match debug_state.channels.rx.try_recv() {
            Ok(data) => {
                handle_debug_message(data, debug_state);
            }
            Err(TryRecvError::Empty) => {
                // No more messages
                break;
            }
            Err(TryRecvError::Disconnected) => {
                debug_state.disconnected = true;
                return;
            }
        }
    }

    // Check if any conditions are met
    if let Some(breakpoint) = debug_state.breakpoints.iter().find(|b| b.pc == pc) {
        debug_state
            .channels
            .tx
            .send(VmMessage::Paused(
                VmPauseReason::Breakpoint(breakpoint.clone()),
                vm_state,
            ))
            .unwrap();
        debug_state.paused = true;
    } else if debug_state.should_pause_for_init {
        debug_state
            .channels
            .tx
            .send(VmMessage::Paused(VmPauseReason::Init, vm_state))
            .unwrap();
        debug_state.paused = true;
        debug_state.should_pause_for_init = false;
    } else if debug_state.is_stepping {
        debug_state
            .channels
            .tx
            .send(VmMessage::Paused(VmPauseReason::Step, vm_state))
            .unwrap();
        debug_state.paused = true;
        debug_state.is_stepping = false;
    }

    // Block waiting for debug adapter to resume
    while !debug_state.disconnected && debug_state.paused {
        if let Ok(data) = debug_state.channels.rx.recv() {
            handle_debug_message(data, debug_state);
        } else {
            debug_state.disconnected = true;
            return;
        }
    }
}

// Separate from run_vm so that performance is not affected in non-debug mode
// TODO: Come up with a way to have less duplicated
pub fn run_vm_debug(vm: &Vm, register_dump_file: Option<PathBuf>, channels: VmChannels) {
    // TODO: Can we avoid RefCell if we know that `run_vm` is the only consumer of VM?
    let mut bus_peripheral = vm.bus_peripheral.borrow_mut();
    let clock_peripheral = vm.clock_peripheral.borrow();

    let mut debug_state = DebugState {
        breakpoints: HashSet::new(),
        channels,
        disconnected: false,
        paused: false,
        should_pause_for_init: true,
        is_stepping: false,
    };

    let mut bus_assertions = BusAssertions::default();
    // TODO: Profile and make this actually performant (currently is ,less than 1 fps in a tight loop)
    let execute = |clocks_until_vsync| {
        let mut clocks = 0;
        loop {
            bus_assertions = bus_peripheral.poll_all(bus_assertions);

            if !debug_state.disconnected && bus_assertions.instruction_fetch {
                notify_debugger(&mut bus_peripheral, &mut debug_state);
            }

            clocks += 1;

            if clocks >= clocks_until_vsync || bus_assertions.exit_simulation {
                bus_assertions.interrupt_assertion |= VSYNC_INTERRUPT;

                return (bus_assertions.exit_simulation, clocks);
            }
        }
    };

    clock_peripheral.start_loop(execute);

    if let Some(register_dump_file) = register_dump_file {
        let cpu: &CpuPeripheral = cpu_from_bus(&mut bus_peripheral);

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

// TODO: Deduplicate stuff with run_vm_debug
pub fn run_vm(vm: &Vm, register_dump_file: Option<PathBuf>) {
    // TODO: Can we avoid RefCell if we know that `run_vm` is the only consumer of VM?
    let mut bus_peripheral = vm.bus_peripheral.borrow_mut();
    let clock_peripheral = vm.clock_peripheral.borrow();

    let mut bus_assertions = BusAssertions::default();
    // TODO: Profile and make this actually performant (currently is ,less than 1 fps in a tight loop)
    let execute = |clocks_until_vsync| {
        let mut clocks = 0;
        loop {
            bus_assertions = bus_peripheral.poll_all(bus_assertions);

            clocks += 1;
            if clocks >= clocks_until_vsync || bus_assertions.exit_simulation {
                bus_assertions.interrupt_assertion |= VSYNC_INTERRUPT;

                return (bus_assertions.exit_simulation, clocks);
            }
        }
    };

    clock_peripheral.start_loop(execute);

    if let Some(register_dump_file) = register_dump_file {
        let cpu: &CpuPeripheral = cpu_from_bus(&mut bus_peripheral);

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
