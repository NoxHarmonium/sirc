use std::{
    collections::{BTreeMap, HashSet},
    sync::mpsc::{Receiver, Sender},
};

use dap::types::Breakpoint;
use serde::{Deserialize, Serialize};

pub type ProgramPosition = u32;
pub type InputPosition = usize;
pub type ObjectDebugInfoMap = BTreeMap<ProgramPosition, ObjectDebugInfo>;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
pub struct ObjectDebugInfo {
    /// SHA256
    pub checksum: String,
    pub original_filename: String,
    pub original_input: String,
    pub program_to_input_offset_mapping: BTreeMap<ProgramPosition, InputPosition>,
}

#[derive(Debug, Clone, Eq, PartialEq, Default, Hash)]
pub struct BreakpointRef {
    pub breakpoint_id: i64,
    pub pc: u32,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct VmState {
    pub pc: u32,
    // btree map to keep variables in some kind of order by name
    // maybe could sort them some other way
    pub variables: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
pub struct ProgramDebugInfo {
    pub debug_info_map: ObjectDebugInfoMap,
}

pub enum ResumeCondition {
    None,
    UntilNextStep,
}

pub enum DebuggerMessage {
    UpdateBreakpoints(HashSet<BreakpointRef>),
    PauseVm,
    ResumeVm(ResumeCondition),
    Disconnect,
}

pub enum VmPauseReason {
    Init,
    Breakpoint(BreakpointRef),
    Step,
}

pub enum VmMessage {
    Paused(VmPauseReason, VmState),
}

#[derive(Debug)]
pub struct VmChannels {
    pub rx: Receiver<DebuggerMessage>,
    pub tx: Sender<VmMessage>,
}

#[derive(Debug)]
pub struct DebuggerChannels {
    pub rx: Receiver<VmMessage>,
    pub tx: Sender<DebuggerMessage>,
}

#[derive(Debug)]
pub struct ServerChannels {
    pub vm: VmChannels,
    pub debugger: DebuggerChannels,
}

pub struct ServerState {
    pub breakpoints: Vec<Breakpoint>,
}
