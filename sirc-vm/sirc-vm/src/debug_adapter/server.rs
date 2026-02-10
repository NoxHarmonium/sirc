use std::collections::HashMap;
use std::io::{BufReader, BufWriter};
use std::net::TcpListener;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::vec;

use dap::prelude::*;
use events::StoppedEventBody;
use log::info;
use responses::{
    ContinueResponse, ScopesResponse, SetBreakpointsResponse, SetExceptionBreakpointsResponse,
    SourceResponse, StackTraceResponse, ThreadsResponse, VariablesResponse,
};
use thiserror::Error;
use types::{
    Breakpoint, Capabilities, Checksum, ChecksumAlgorithm, Scope, Source, StackFrame,
    StoppedEventReason, Thread, Variable,
};

use crate::debug_adapter::types::{
    BreakpointRef, ResumeCondition, ServerState, VmPauseReason, VmState,
};
use crate::utils::lines::{translate_line_column_to_pc, translate_pc_to_line_column};

use super::types::{
    DebuggerChannels, DebuggerMessage, ProgramDebugInfo, ServerChannels, VmChannels, VmMessage,
};

// TODO: Make debug server options configurable with args or environment variables
// category=Debugging
// E.g. listener address
static LISTENER_ADDRESS: &str = "0.0.0.0:9090";
/// The thread ID for the program running in the VM, not an actual thread on the
/// system the VM is running on
/// The CPU can only has one thread, it is a single core CPU, so this should never change
const DEFAULT_THREAD_ID: i64 = 1;
const DEFAULT_STACK_FRAME_ID: i64 = 1;
const DEFAULT_VARIABLES_ID: i64 = 1;

static INSTRUCTION_REF_PREFIX: &str = "pc:";

// TODO: Consider moving the DAP to a separate module?
// category=Debugging

#[derive(Error, Debug)]
enum DebugAdapterError {
    #[error("Missing command")]
    MissingCommandError,
}

type DynResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[must_use]
pub fn format_instruction_ref(pc: u32) -> String {
    format!("{INSTRUCTION_REF_PREFIX}{pc:X}")
}

#[must_use]
pub fn parse_instruction_ref(instruction_ref: &str) -> u32 {
    assert!(
        instruction_ref.starts_with(INSTRUCTION_REF_PREFIX),
        "Cannot decode instruction ref without prefix."
    );
    let prefix_length = INSTRUCTION_REF_PREFIX.len();
    // Better error handling in debug server
    // category=Refactoring
    // See also: the unwraps below
    u32::from_str_radix(&instruction_ref[prefix_length..], 16).unwrap()
}

#[must_use]
pub fn create_server_channels() -> ServerChannels {
    let (debugger_tx, debugger_rx) = channel::<DebuggerMessage>();
    let (vm_tx, vm_rx) = channel::<VmMessage>();
    ServerChannels {
        vm: VmChannels {
            rx: debugger_rx,
            tx: vm_tx,
        },
        debugger: DebuggerChannels {
            rx: vm_rx,
            tx: debugger_tx,
        },
    }
}

pub fn start_server(
    channels: DebuggerChannels,
    program_debug_info: &ProgramDebugInfo,
) -> DynResult<()> {
    info!("Waiting for socket connection on [{LISTENER_ADDRESS}]...");
    let listener = TcpListener::bind(LISTENER_ADDRESS)?;

    // TODO: Work out what happens if there is more than one connection attempt to the debug server
    // category=Debugging
    let (stream, addr) = listener.accept()?;

    info!("Connection established with {addr:?}");

    let output = BufWriter::new(stream.try_clone().unwrap());
    let input = BufReader::new(stream);
    let mut server = Server::new(input, output);
    let mut server_state = ServerState {
        breakpoints: vec![],
    };
    // TODO: Investigate whether there is a better way to share the VM state than a mutex
    // category=Refactoring
    // Maybe channels would be better?
    let vm_state: Arc<Mutex<Option<VmState>>> = Arc::new(Mutex::new(None));
    let mut next_id: i64 = 1;
    let mut get_new_id = || {
        let id = next_id;
        next_id += 1;
        id
    };
    let vm_state_ref = vm_state.clone();
    let sources: HashMap<String, (Source, String)> = program_debug_info
        .debug_info_map
        .values()
        .map(|d| {
            (
                d.original_filename.clone(),
                (
                    Source {
                        name: Some(d.original_filename.clone()),
                        path: Some(d.original_filename.clone()),
                        source_reference: None,
                        presentation_hint: None,
                        origin: None,
                        sources: None,
                        adapter_data: None,
                        checksums: Some(vec![Checksum {
                            algorithm: types::ChecksumAlgorithm::SHA256,
                            checksum: d.checksum.clone(),
                        }]),
                    },
                    d.original_input.clone(),
                ),
            )
        })
        .collect();

    let server_output = server.output.clone();
    std::thread::spawn(move || {
        loop {
            if let Ok(data) = channels.rx.recv() {
                match data {
                    VmMessage::Paused(reason, new_vm_state) => {
                        let mut vm_state = vm_state_ref.lock().unwrap();
                        *vm_state = Some(new_vm_state);
                        drop(vm_state);

                        server_output
                            .lock()
                            .unwrap()
                            .send_event(Event::Stopped(StoppedEventBody {
                                reason: match reason {
                                    VmPauseReason::Init => StoppedEventReason::Entry,
                                    VmPauseReason::Breakpoint(_) => StoppedEventReason::Breakpoint,
                                    VmPauseReason::Step => StoppedEventReason::Step,
                                },
                                description: match reason {
                                    VmPauseReason::Init => {
                                        Some("Paused at start of program".to_string())
                                    }
                                    VmPauseReason::Breakpoint(_) => {
                                        Some("Paused on breakpoint".to_string())
                                    }
                                    VmPauseReason::Step => Some("Paused after step".to_string()),
                                },
                                thread_id: Some(DEFAULT_THREAD_ID),
                                preserve_focus_hint: None,
                                text: None,
                                all_threads_stopped: Some(true),
                                hit_breakpoint_ids: match reason {
                                    VmPauseReason::Breakpoint(breakpoint) => {
                                        Some(vec![breakpoint.breakpoint_id])
                                    }
                                    VmPauseReason::Step | VmPauseReason::Init => None,
                                },
                            }))
                            .unwrap();
                    }
                }
            } else {
                // The VM has closed it side so we should terminate the debug session
                // TODO: Fix up behaviour when debug server connection closes
                // category=Debugger
                // Why Error occurred in debug server: SendError { .. } is logged when this happens?
                server_output
                    .lock()
                    .unwrap()
                    .send_event(Event::Terminated(None))
                    .unwrap();

                // Stop looping
                return;
            }
        }
    });

    loop {
        let Some(req) = server.poll_request()? else {
            return Err(Box::new(DebugAdapterError::MissingCommandError));
        };

        match req.command {
            Command::Attach(_) => {
                // It is always attached. Maybe in the future there could be some more sophisticated launch vs attach behaviour
                let rsp = req.success(ResponseBody::Attach);
                server.respond(rsp)?;
            }
            Command::BreakpointLocations(_) => todo!(),
            Command::Completions(_) => todo!(),
            Command::ConfigurationDone => todo!(),
            Command::Continue(_) => {
                let rsp = req.success(ResponseBody::Continue(ContinueResponse {
                    all_threads_continued: Some(true),
                }));
                server.respond(rsp)?;

                channels
                    .tx
                    .send(DebuggerMessage::ResumeVm(ResumeCondition::None))?;
            }
            Command::DataBreakpointInfo(_) => todo!(),
            Command::Disassemble(_) => todo!(),
            Command::Disconnect(_) => {
                let rsp = req.success(ResponseBody::Disconnect);
                server.respond(rsp)?;

                channels.tx.send(DebuggerMessage::Disconnect)?;

                break;
            }
            Command::Evaluate(_) => todo!(),
            Command::ExceptionInfo(_) => todo!(),
            Command::Goto(_) => todo!(),
            Command::GotoTargets(_) => todo!(),
            Command::Initialize(_) => {
                let rsp = req.success(ResponseBody::Initialize(Capabilities {
                    supported_checksum_algorithms: Some(vec![ChecksumAlgorithm::SHA256]),
                    ..Capabilities::default()
                }));

                // When you call respond, send_event etc. the message will be wrapped
                // in a base message with a appropriate seq number, so you don't have to keep track of that yourself
                server.respond(rsp)?;

                server.send_event(Event::Initialized)?;
            }
            Command::Launch(_) => {
                // It is always attached so launch doesn't make sense.
                let rsp = req.success(ResponseBody::Launch);
                server.respond(rsp)?;
            }
            Command::LoadedSources => todo!(),
            Command::Modules(_) => todo!(),
            Command::Next(_) => {
                channels
                    .tx
                    .send(DebuggerMessage::ResumeVm(ResumeCondition::UntilNextStep))?;

                let rsp = req.success(ResponseBody::Next);
                server.respond(rsp)?;
            }
            Command::Pause(_) => {
                let rsp = req.success(ResponseBody::Pause);
                server.respond(rsp)?;

                channels.tx.send(DebuggerMessage::PauseVm)?;
            }
            Command::ReadMemory(_) => todo!(),
            Command::Restart(_) => todo!(),
            Command::RestartFrame(_) => todo!(),
            Command::ReverseContinue(_) => todo!(),
            Command::Scopes(_) => {
                let rsp = req.success(ResponseBody::Scopes(ScopesResponse {
                    scopes: vec![
                        Scope {
                            name: "cpu registers".to_string(),
                            presentation_hint: None,
                            variables_reference: DEFAULT_VARIABLES_ID,
                            named_variables: Some(16),
                            indexed_variables: Some(0),
                            expensive: false,
                            source: None,
                            line: None,
                            column: None,
                            end_line: None,
                            end_column: None,
                        },
                        // TODO Misc:
                        // cause
                        // system_ram_offset
                        // pending_coprocessor_command
                        // ExceptionUnitRegisters
                    ],
                }));
                server.respond(rsp)?;
            }
            Command::SetBreakpoints(ref args) => {
                if args.source_modified.is_some_and(|x| x) {
                    unimplemented!("Source modification is not supported");
                }
                let breakpoints: Vec<Breakpoint> = args.breakpoints.as_ref().map_or_else(
                    std::vec::Vec::new,
                    |source_breakpoints| {
                        source_breakpoints
                            .iter()
                            .map(|b| {
                                let pc = args.source.name.as_ref().and_then(|c| {
                                    translate_line_column_to_pc(
                                        program_debug_info,
                                        c.as_str(),
                                        (b.line, b.column.unwrap_or(1)),
                                    )
                                });

                                Breakpoint {
                                    id: Some(get_new_id()),
                                    verified: pc.is_some(),
                                    message: None,
                                    source: Some(args.source.clone()),
                                    line: Some(b.line),
                                    column: b.column,
                                    end_line: None,
                                    end_column: None,
                                    instruction_reference: pc.map(format_instruction_ref),
                                    offset: Some(0),
                                }
                            })
                            .collect()
                    },
                );

                server_state.breakpoints.clone_from(&breakpoints);

                channels.tx.send(DebuggerMessage::UpdateBreakpoints(
                    breakpoints
                        .iter()
                        .map(|b| BreakpointRef {
                            breakpoint_id: b.id.unwrap(),
                            pc: parse_instruction_ref(
                                b.instruction_reference.as_ref().unwrap().as_str(),
                            ),
                        })
                        .collect(),
                ))?;

                let rsp = req.success(ResponseBody::SetBreakpoints(SetBreakpointsResponse {
                    breakpoints: breakpoints.clone(),
                }));
                server.respond(rsp)?;
            }
            Command::SetDataBreakpoints(_) => todo!(),
            Command::SetExceptionBreakpoints(_) => {
                let rsp = req.success(ResponseBody::SetExceptionBreakpoints(
                    SetExceptionBreakpointsResponse { breakpoints: None },
                ));
                server.respond(rsp)?;
            }
            Command::SetExpression(_) => todo!(),
            Command::SetFunctionBreakpoints(_) => todo!(),
            Command::SetInstructionBreakpoints(_) => todo!(),
            Command::SetVariable(_) => todo!(),
            Command::Source(ref args) => {
                let content = args
                    .source
                    .as_ref()
                    .and_then(|s| s.path.as_ref())
                    .and_then(|s| sources.get(s))
                    .map(|(_, content)| content.clone());

                let rsp = req.success(ResponseBody::Source(SourceResponse {
                    content: content.unwrap_or(String::new()),
                    mime_type: None,
                }));
                server.respond(rsp)?;
            }
            Command::StackTrace(_) => {
                if let Some(vm_state) = vm_state.lock().unwrap().as_ref() {
                    if let Some((line, column, original_filename)) =
                        translate_pc_to_line_column(program_debug_info, vm_state.pc)
                    {
                        let source = sources
                            .get(&original_filename)
                            .map(|(source, _)| source.to_owned());

                        let rsp = req.success(ResponseBody::StackTrace(StackTraceResponse {
                            stack_frames: vec![StackFrame {
                                id: DEFAULT_STACK_FRAME_ID,
                                name: "main".to_string(),
                                source,
                                line,
                                column,
                                end_line: None,
                                end_column: None,
                                can_restart: None,
                                instruction_pointer_reference: Some(format_instruction_ref(
                                    vm_state.pc,
                                )),
                                module_id: None,
                                presentation_hint: None,
                            }],
                            total_frames: Some(1),
                        }));
                        server.respond(rsp)?;

                        continue;
                    }
                }

                let rsp = req.success(ResponseBody::StackTrace(StackTraceResponse {
                    stack_frames: vec![],
                    total_frames: None,
                }));
                server.respond(rsp)?;
            }
            Command::StepBack(_) => todo!(),
            Command::StepIn(_) => todo!(),
            Command::StepInTargets(_) => todo!(),
            Command::StepOut(_) => todo!(),
            Command::Terminate(_) => todo!(),
            Command::TerminateThreads(_) => todo!(),
            Command::Threads => {
                // This VM only has a single thread
                let rsp = req.success(ResponseBody::Threads(ThreadsResponse {
                    threads: vec![Thread {
                        id: DEFAULT_THREAD_ID,
                        name: "main".to_string(),
                    }],
                }));
                server.respond(rsp)?;
            }
            Command::Variables(ref args) => {
                assert!(
                    args.variables_reference == DEFAULT_VARIABLES_ID,
                    "Only a single variable reference is supported at this time"
                );

                let rsp = if let Some(vm_state) = vm_state.lock().unwrap().as_ref() {
                    req.success(ResponseBody::Variables(VariablesResponse {
                        variables: vm_state
                            .variables
                            .iter()
                            .map(|(name, value)| Variable {
                                name: name.clone(),
                                value: value.clone(),
                                type_field: None,
                                presentation_hint: None,
                                evaluate_name: None,
                                variables_reference: 0,
                                named_variables: None,
                                indexed_variables: None,
                                memory_reference: None,
                            })
                            .collect(),
                    }))
                } else {
                    req.success(ResponseBody::Variables(VariablesResponse {
                        variables: vec![],
                    }))
                };
                server.respond(rsp)?;
            }
            Command::WriteMemory(_) => todo!(),
            Command::Cancel(_) => todo!(),
        }
    }

    Ok(())
}
