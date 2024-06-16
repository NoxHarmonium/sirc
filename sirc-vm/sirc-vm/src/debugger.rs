use std::{collections::BTreeMap, sync::mpsc::TryRecvError};

use super::debug_adapter::types::{
    DebuggerMessage, ResumeCondition, VmMessage, VmPauseReason, VmState,
};
use super::{utils::cpu_from_bus::cpu_from_bus, DebugState};
use peripheral_bus::BusPeripheral;
use peripheral_cpu::registers::FullAddressRegisterAccess;

pub fn handle_debug_message(message: DebuggerMessage, debug_state: &mut DebugState) {
    match message {
        DebuggerMessage::UpdateBreakpoints(breakpoints) => {
            debug_state.breakpoints = breakpoints;
        }
        DebuggerMessage::PauseVm => {
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

pub fn yield_to_debugger(bus_peripheral: &mut BusPeripheral, debug_state: &mut DebugState) {
    let cpu = cpu_from_bus(bus_peripheral);
    let pc = cpu.registers.get_full_pc_address();
    let capture_vm_state = || {
        VmState {
            pc,
            // TODO: Expose more CPU state to debugger
            // category=Debugging
            // There is a heap more state than just registers that we could expose to help debug
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
        }
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
                capture_vm_state(),
            ))
            .unwrap();
        debug_state.paused = true;
    } else if debug_state.should_pause_for_init {
        debug_state
            .channels
            .tx
            .send(VmMessage::Paused(VmPauseReason::Init, capture_vm_state()))
            .unwrap();
        debug_state.paused = true;
        debug_state.should_pause_for_init = false;
    } else if debug_state.is_stepping {
        debug_state
            .channels
            .tx
            .send(VmMessage::Paused(VmPauseReason::Step, capture_vm_state()))
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
