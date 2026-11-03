#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    // I don't like this rule
    clippy::module_name_repetitions,
    // Not sure what this is, will have to revisit
    clippy::must_use_candidate,
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

extern crate num;
#[macro_use]
extern crate num_derive;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

// TODO: Try to hide more implementation details of the CPU module
// category: Refactoring
// Can we expose the Executor trait here without exposing the implementations?
// OR can we keep everything private and somehow enable tests to reach inside?
pub mod coprocessors;
pub mod registers;
pub mod util;

use std::{any::Any, fmt::Write};

use log::{debug, error, trace, warn};

use coprocessors::{
    exception_unit::{
        definitions::{
            vectors::DOUBLE_FAULT_VECTOR, ExceptionPriorities, ExceptionUnitOpCodes, Faults,
            PendingFault,
        },
        execution::{
            construct_exception_unit_command, deconstruct_exception_unit_command,
            select_pending_event, ExceptionUnitExecutor,
        },
    },
    processing_unit::execution::ProcessingUnitExecutor,
    shared::{ExecutionPhase, Executor},
};
use num::ToPrimitive;
use num_traits::FromPrimitive;
use peripheral_bus::device::{BusAccessType, BusAssertions, Device};
use registers::ExceptionUnitRegisters;
use util::mask_bits_below_index;

use crate::registers::{
    get_hardware_interrupt_enable, sr_bit_is_set, ExceptionLinkRegister, FullAddressRegisterAccess,
    Registers, SegmentedAddress, StatusRegisterFields,
};

// The 8th exception link register stores metadata about faults
const FAULT_METADATA_LINK_REGISTER_INDEX: usize = 7;

// Bit 4 of the hardware-interrupt bitmask passed to `raise_hardware_interrupt` -- Level 5 (NMI).
const LEVEL_FIVE_INTERRUPT_PIN_BIT: u8 = 0b1_0000;

/// Its always six baby!
pub const CYCLES_PER_INSTRUCTION: u8 = 6;

// Coprocessor select register components
pub const COPROCESSOR_ID_MASK: u16 = 0xF000;
pub const COPROCESSOR_ID_LENGTH: u16 = 12;
pub const COPROCESSOR_OPCODE_ID_MASK: u16 = 0x0F00;
pub const COPROCESSOR_OPCODE_ID_LENGTH: u16 = 8;

// pub bus_access_type: BusAccessType, // 3 bits (BAT0-BAT2)
// pub double_fault: bool, // 1 bit
// pub fault: Faults, // 4 bits (needs to hold values up to 0x8)
// pub original_fault: Faults // 4 bits (needs to hold values up to 0x8)

// Fault metadata components
pub const BUS_ACCESS_TYPE_MASK: u16 = 0x7;
pub const BUS_ACCESS_TYPE_LENGTH: u16 = 0;
pub const DOUBLE_FAULT_FLAG_MASK: u16 = 0x8;
pub const DOUBLE_FAULT_FLAG_LENGTH: u16 = 3;
pub const CURRENT_FAULT_MASK: u16 = 0xF0;
pub const CURRENT_FAULT_LENGTH: u16 = 4;
pub const PREVIOUS_FAULT_MASK: u16 = 0xF00;
pub const PREVIOUS_FAULT_LENGTH: u16 = 8;

#[derive(Debug)]
pub enum Error {
    ProcessorHalted(Registers),
    InvalidInstruction(Registers),
}

/// Why the current slot is running, selected at phase 0 and retained through
/// the slot and bus stalls. Both CSR commands and automatic events can run the
/// EU, so the executor's coprocessor ID alone cannot determine CSR ownership.
#[derive(Debug, PartialEq, Eq)]
enum SlotOwner {
    /// A fetched PU instruction, including COPI/COPR issuing a command for a later slot.
    ProcessingUnit,
    /// A command accepted into CSR. The CU clears it on completion or cancellation.
    CsrCommand,
    /// An automatically selected fault or interrupt. Its dispatch does not own a queued CSR command.
    EuEvent,
    /// A vector fetch failed. No executor owns the remaining idle phases;
    /// its replacement fault is selected at the next phase-0 boundary.
    Aborted,
}

/// Trace belongs to the logical PU instruction, including any command it issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstructionTrace {
    /// No trace is owed: disabled, canceled, or already made ready for dispatch.
    None,
    /// T/TRCE requested trace when the current PU instruction began.
    Sampled,
    /// The call committed; its command must succeed before trace can become ready.
    DeferredCommand,
}

pub struct CpuPeripheral {
    pub registers: Registers,
    pub eu_registers: ExceptionUnitRegisters,
    pub phase: u8,
    pub processing_unit_executor: ProcessingUnitExecutor,
    pub exception_unit_executor: ExceptionUnitExecutor,
    /// Command latched for the active slot, including automatic EU events.
    /// Slot ownership determines whether completion consumes CSR.
    pub active_command: u16,
    // Selected at phase 0 and retained through the slot, including bus stalls.
    slot_owner: SlotOwner,
    // Sample once at PU instruction start; retain eligibility until its command completes.
    instruction_trace: InstructionTrace,
    // When the CPU asserts bus_access_strobe, the request is held here until bus_acknowledge
    // (or a bus error) is received. During the wait, the request is re-asserted each cycle
    // without advancing the phase, simulating the CPU stalling on a slow device.
    pending_bus_request: Option<BusAssertions>,
    // Set when halt_requested is asserted at an instruction boundary. Cleared when deasserted.
    is_halted: bool,
    // Set by a triple fault (bus error during double-fault vector fetch). Gates all CPU logic
    // and re-asserts reset_requested every cycle until reset() clears it. Models the hardware
    // flip-flop that a triple fault would set to hold the reset line.
    reset_pending: bool,
    // Tracks the Level 5 (NMI) pin's state as of the last poll cycle, so a rising edge can be
    // detected. Unlike levels 1-4 (level-sensitive), Level 5 only latches on a 0->1 transition.
    l5_interrupt_pin_previously_asserted: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FaultMetadataRegister {
    pub bus_access_type: BusAccessType, // 3 bits (bits 0-2, BAT0-BAT2)
    pub double_fault: bool,             // 1 bit (bit 3)
    pub fault: Faults,                  // 4 bits (bits 4-7, needs to hold values up to 0x8)
    // If double_fault: holds the fault that caused the double fault
    pub original_fault: Faults, // 4 bits (bits 8-11, needs to hold values up to 0x8)
}

pub fn decode_fault_metadata_register(val: u16) -> FaultMetadataRegister {
    FaultMetadataRegister {
        bus_access_type: BusAccessType::from(
            ((val & BUS_ACCESS_TYPE_MASK) >> BUS_ACCESS_TYPE_LENGTH) as u8,
        ),
        double_fault: ((val & DOUBLE_FAULT_FLAG_MASK) >> DOUBLE_FAULT_FLAG_LENGTH) == 0x1,
        fault: Faults::from_u16((val & CURRENT_FAULT_MASK) >> CURRENT_FAULT_LENGTH).unwrap(),
        original_fault: Faults::from_u16((val & PREVIOUS_FAULT_MASK) >> PREVIOUS_FAULT_LENGTH)
            .unwrap(),
    }
}

pub fn encode_fault_metadata_register(reg: &FaultMetadataRegister) -> u16 {
    (reg.original_fault.to_u16().unwrap()) << PREVIOUS_FAULT_LENGTH
        | (reg.fault.to_u16().unwrap()) << CURRENT_FAULT_LENGTH
        | u16::from(reg.double_fault) << DOUBLE_FAULT_FLAG_LENGTH
        | u16::from(reg.bus_access_type as u8)
}

/// Raises `fault`, setting `eu_registers.pending_fault` (escalating to `DoubleFault` if one is
/// already being serviced).
///
/// `fault_pc` is the address `handle_exception` should later store as the fault's return
/// address: the faulting instruction's own address for a retryable fault, or the following
/// instruction's address otherwise. This is the sole place `pending_fault` is set, other than
/// the double-fault-during-vector-fetch escalation in `handle_vector_fetch_bus_fault`, which
/// must preserve the original fault's `pc` rather than compute a new one.
pub fn raise_fault(
    eu_registers: &mut ExceptionUnitRegisters,
    fault: Faults,
    bus_assertions: &BusAssertions,
    fault_pc: u32,
) {
    if let Some(pending_fault) = eu_registers.pending_fault {
        // TODO: What would happen in hardware if a fault was raised when one was already pending
        // category=Hardware
        // Is this possible in hardware? If so, what would happen?
        panic!("Cannot raise fault when one is pending. Trying to raise {fault:?} but {pending_fault:?} is already pending.");
    }

    debug!(
        "raise_fault: fault: {fault:?} address: 0x{:X} bus_access_type: {:?}",
        bus_assertions.address, bus_assertions.bus_access_type,
    );

    let current_exception_level = eu_registers.current_exception_level;
    let is_double_fault = current_exception_level >= ExceptionPriorities::Fault as u8;
    let resolved_fault = if is_double_fault {
        error!("Double fault! [{fault:?}] raised when a fault was already being serviced. Jumping to double fault vector.");
        Faults::DoubleFault
    } else {
        fault
    };
    // Is always clobbered, so you lose the original fault metadata register if there is a double fault
    // Make sure you save it somewhere if you're being careful
    eu_registers.link_registers[FAULT_METADATA_LINK_REGISTER_INDEX] = ExceptionLinkRegister {
        // Not actually a return address, just using the same data structure as a link register for now
        return_address: bus_assertions.address,
        // TODO: Find a use for unused bits in `return_status_register`
        // category=Hardware
        return_status_register: encode_fault_metadata_register(&FaultMetadataRegister {
            bus_access_type: bus_assertions.bus_access_type,
            double_fault: is_double_fault,
            fault: resolved_fault,
            original_fault: fault,
        }),
        saved_exception_level: 0, // Not used for fault metadata
    };
    eu_registers.pending_fault = Some(PendingFault {
        fault: resolved_fault,
        pc: fault_pc,
    });
}

///
/// Instantiates a new `CpuPeripheral` with default values after doing some checks.
///
/// # Panics
/// Will panic if the specified `program_segment_label` is not a segment defined in the provided `memory_peripheral`
///
#[must_use]
pub fn new_cpu_peripheral() -> CpuPeripheral {
    CpuPeripheral {
        registers: Registers::default(),
        eu_registers: ExceptionUnitRegisters::default(),
        phase: 0,
        processing_unit_executor: ProcessingUnitExecutor::default(),
        exception_unit_executor: ExceptionUnitExecutor::default(),
        active_command: 0,
        slot_owner: SlotOwner::ProcessingUnit,
        instruction_trace: InstructionTrace::None,
        pending_bus_request: None,
        is_halted: false,
        reset_pending: false,
        l5_interrupt_pin_previously_asserted: false,
    }
}

impl Device for CpuPeripheral {
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
        let result = self.poll_internal(bus_assertions, selected);
        BusAssertions {
            // Vector-table reads are supervisor accesses even before entry
            // clears live SR.P. Apply this to held requests as well.
            protected_mode_active: result.bus_access_type != BusAccessType::ExceptionVectorFetch
                && sr_bit_is_set(StatusRegisterFields::ProtectedMode, &self.registers),
            ..result
        }
    }

    fn dump_diagnostic(&self) -> String {
        let register_text = format!("{:#x?}", self.registers);
        let eu_register_text = format!("{:#x?}", self.eu_registers);

        let mut s = String::new();
        writeln!(s, "===REGISTERS===").unwrap();
        writeln!(s, "{register_text}").unwrap();
        writeln!(s, "===EXCEPTION UNIT REGISTERS===").unwrap();
        writeln!(s, "{eu_register_text}").unwrap();

        s
    }

    fn reset(&mut self) {
        Self::reset(self);
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl CpuPeripheral {
    #[allow(clippy::cast_possible_truncation)]
    fn poll_internal(&mut self, bus_assertions: BusAssertions, _: bool) -> BusAssertions {
        if self.reset_pending {
            return BusAssertions {
                reset_requested: true,
                ..BusAssertions::default()
            };
        }

        // Allow wake up if there is a hardware interrupt. Called every cycle (not just when
        // asserted) so Level 5's edge-detection can observe the pin being released, not only
        // asserted.
        self.raise_hardware_interrupt(bus_assertions.interrupt_assertion);

        // If we have a pending bus request, stall until the device acknowledges (or errors).
        // Bus errors and protection errors also release the stall so fault handling can proceed.
        let completed_bus_request = if let Some(pending_request) = self.pending_bus_request {
            let bus_access_complete = bus_assertions.bus_acknowledge
                || bus_assertions.bus_error
                || bus_assertions.bus_protection_error;
            if !bus_access_complete {
                return BusAssertions {
                    instruction_sync: self.phase == ExecutionPhase::InstructionFetchLow as u8,
                    ..pending_request
                };
            }
            // Access complete: advance to the next phase and fall through to run it
            self.pending_bus_request = None;
            Some(pending_request)
        } else {
            None
        };

        let phase: ExecutionPhase =
            FromPrimitive::from_u8(self.phase).expect("Expected phase to be between 0-5");

        // Check for halt at instruction boundaries (phase 0). Latch halt when first asserted;
        // stall each subsequent cycle while still asserted; resume when deasserted.
        if phase == ExecutionPhase::InstructionFetchLow {
            if bus_assertions.halt_requested {
                self.is_halted = true;
            }
            if self.is_halted {
                if bus_assertions.halt_requested {
                    return BusAssertions::default();
                }
                self.is_halted = false;
            }
        }

        // Command-first dispatch may enter WAIT with an interrupt already
        // latched. Wake for that eligible event so the WAIT slot can finish;
        // the event is accepted only at the following phase-0 boundary.
        if self.eu_registers.waiting_for_exception
            && select_pending_event(&self.registers, &self.eu_registers).is_some()
        {
            self.eu_registers.waiting_for_exception = false;
        }
        if self.eu_registers.waiting_for_exception {
            return BusAssertions::default();
        }

        // A newly aborting vector fault differs from the pending fault whose
        // handler a normal EU slot is entering. Retire the cancelled slot's
        // remaining phases without running EU execution or write-back cleanup.
        if self.slot_owner == SlotOwner::Aborted && phase != ExecutionPhase::InstructionFetchLow {
            self.advance_phase();
            return BusAssertions::default();
        }

        let fault_bus_assertions =
            completed_bus_request.map_or(bus_assertions, |request| BusAssertions {
                address: request.address,
                op: request.op,
                bus_access_type: request.bus_access_type,
                ..bus_assertions
            });

        if let Some(bus_fault) = Self::fault_from_bus_assertions(fault_bus_assertions) {
            if self.slot_owner == SlotOwner::CsrCommand {
                self.clear_coprocessor_select_register();
                self.instruction_trace = InstructionTrace::None;
            }
            if let Some(vector_fetch_fault_response) =
                self.handle_vector_fetch_bus_fault(bus_fault, fault_bus_assertions)
            {
                // The error completes this bus phase; preserve the rest of
                // the slot rather than restarting dispatch asynchronously.
                self.advance_phase();
                return vector_fetch_fault_response;
            }

            // An instruction-fetch bus error means the fetch itself is what's failing, so `pl`
            // is still the faulting address. Any later phase means the error came from a bus
            // request issued during this instruction's own execution (in practice only a
            // `MemoryAccessExecutor` data read/write) -- by then `pl` has already been advanced
            // to the next instruction, so recover this instruction's own fetch address instead.
            let fault_pc = match phase {
                ExecutionPhase::InstructionFetchLow | ExecutionPhase::InstructionFetchHigh => {
                    self.registers.get_full_pc_address()
                }
                _ => (
                    self.processing_unit_executor.decoded_instruction.pc_h_,
                    self.processing_unit_executor.decoded_instruction.pc_l_,
                )
                    .to_full_address(),
            };

            // TODO: Reduce number of mutable references in `CpuPeripheral` poll
            // category=Refactoring
            // Can we do this without have a mut ref to eu_registers?
            raise_fault(
                &mut self.eu_registers,
                bus_fault,
                &fault_bus_assertions,
                fault_pc,
            );
        }

        if phase == ExecutionPhase::InstructionFetchLow {
            // Select the active command once per slot, retaining it through bus stalls.
            // An aborting fault cancels pending work. ID-0
            // commands have no dispatch slot and must not block pending events.
            if self.eu_registers.pending_fault.is_some()
                || Self::decode_processor_id(self.registers.coprocessor_select_register)
                    == ProcessingUnitExecutor::COPROCESSOR_ID
            {
                self.clear_coprocessor_select_register();
            }

            (self.active_command, self.slot_owner) = if self.registers.coprocessor_select_register
                != 0
            {
                (
                    self.registers.coprocessor_select_register,
                    SlotOwner::CsrCommand,
                )
            } else if let Some(event) = select_pending_event(&self.registers, &self.eu_registers) {
                (
                    event.accept(&mut self.eu_registers, self.registers.get_full_pc_address()),
                    SlotOwner::EuEvent,
                )
            } else {
                (0, SlotOwner::ProcessingUnit)
            };
        }

        let coprocessor_id = Self::decode_processor_id(self.active_command);

        // Sample T/TRCE only when a PU instruction begins. A command retains its
        // issuing instruction's decision; automatic EU dispatches never generate trace.
        if phase == ExecutionPhase::InstructionFetchLow {
            match self.slot_owner {
                SlotOwner::ProcessingUnit => {
                    let enabled = self.eu_registers.current_exception_level
                        != ExceptionPriorities::Fault as u8
                        && (sr_bit_is_set(StatusRegisterFields::TraceMode, &self.registers)
                            || bus_assertions.force_trace_mode);
                    self.instruction_trace = if enabled {
                        InstructionTrace::Sampled
                    } else {
                        InstructionTrace::None
                    };
                }
                SlotOwner::CsrCommand => {}
                SlotOwner::EuEvent | SlotOwner::Aborted => {
                    self.instruction_trace = InstructionTrace::None;
                }
            }
        }

        trace!(
            "coprocessor_id: {coprocessor_id} eu_registers.pending_fault: {:?}",
            self.eu_registers.pending_fault
        );

        let result = match coprocessor_id {
            ProcessingUnitExecutor::COPROCESSOR_ID => self.processing_unit_executor.step(
                &phase,
                self.active_command,
                &mut self.registers,
                &mut self.eu_registers,
                bus_assertions,
            ),
            ExceptionUnitExecutor::COPROCESSOR_ID => self.exception_unit_executor.step(
                &phase,
                self.active_command,
                &mut self.registers,
                &mut self.eu_registers,
                bus_assertions,
            ),
            _ => {
                if self.eu_registers.pending_fault.is_none()
                    && phase == ExecutionPhase::InstructionFetchLow
                {
                    warn!("Invalid COP coprocessor ID detected: {coprocessor_id}");
                    let invalid_coprocessor_command = self.active_command;
                    // This arm only runs at InstructionFetchLow, one poll cycle after the
                    // faulting instruction's own WriteBackExecutor wrote
                    // `coprocessor_select_register` -- since the coprocessor ID has been invalid
                    // for the whole intervening cycle, `processing_unit_executor.step()` (and so
                    // its decode) was never invoked, so `decoded_instruction` is still exactly
                    // that faulting instruction's own decode, untouched.
                    let fault_pc = (
                        self.processing_unit_executor.decoded_instruction.pc_h_,
                        self.processing_unit_executor.decoded_instruction.pc_l_,
                    )
                        .to_full_address();
                    // Can be used for forwards compatibility if co-processors are added in later models
                    raise_fault(
                        &mut self.eu_registers,
                        Faults::InvalidOpCode,
                        &BusAssertions {
                            address: u32::from(invalid_coprocessor_command),
                            ..bus_assertions
                        },
                        fault_pc,
                    );
                    self.clear_coprocessor_select_register();
                    self.instruction_trace = InstructionTrace::None;
                }

                BusAssertions::default()
            }
        };

        // Executors do not own CSR cleanup. Only the command whose slot just
        // completed (or aborted) consumes CSR; an automatic EU event does not.
        if phase == ExecutionPhase::WriteBackExecutor && self.slot_owner == SlotOwner::CsrCommand {
            self.clear_coprocessor_select_register();
        }

        if phase == ExecutionPhase::WriteBackExecutor {
            self.finish_instruction_trace(bus_assertions);
        }

        // TODO: Investigate performance impact of unrolling the six CPU phases
        // category=Performance
        if result.bus_access_strobe {
            self.pending_bus_request = Some(result);
        }

        self.advance_phase();

        // instruction_sync uses the phase captured before advance_phase() above.
        let result = BusAssertions {
            instruction_sync: phase == ExecutionPhase::InstructionFetchLow,
            ..result
        };

        // Software RSET: after WriteBack wraps phase back to 0, if coprocessor_select_register
        // was just set to the reset command, signal the reset unit via reset_requested so it starts
        // the 6-cycle RSTO hold before the EU fetches the reset vector.
        let reset_command = construct_exception_unit_command(&ExceptionUnitOpCodes::Reset, 0x0);
        if self.phase == 0 && self.registers.coprocessor_select_register == reset_command {
            return BusAssertions {
                reset_requested: true,
                ..result
            };
        }

        result
    }

    /// Make trace eligible only after the entire instruction succeeds, including
    /// its command. Capture the resulting PC here (not at call write-back), so
    /// RETE's restored PC is the trace handler's return address.
    fn finish_instruction_trace(&mut self, bus_assertions: BusAssertions) {
        if self.eu_registers.pending_fault.is_some() {
            self.instruction_trace = InstructionTrace::None;
            return;
        }

        let trace_due = match (self.instruction_trace, &self.slot_owner) {
            (InstructionTrace::Sampled, SlotOwner::ProcessingUnit) => {
                let command = self.registers.coprocessor_select_register;
                if command == construct_exception_unit_command(&ExceptionUnitOpCodes::Reset, 0)
                    || Self::is_software_exception_command(command)
                {
                    self.instruction_trace = InstructionTrace::None;
                    false
                } else if Self::decode_processor_id(command)
                    != ProcessingUnitExecutor::COPROCESSOR_ID
                {
                    self.instruction_trace = InstructionTrace::DeferredCommand;
                    false
                } else {
                    // Ordinary instructions and ID-0 calls complete here.
                    self.instruction_trace = InstructionTrace::None;
                    true
                }
            }
            (InstructionTrace::DeferredCommand, SlotOwner::CsrCommand) => {
                self.instruction_trace = InstructionTrace::None;
                true
            }
            _ => {
                self.instruction_trace = InstructionTrace::None;
                false
            }
        };
        if trace_due {
            raise_fault(
                &mut self.eu_registers,
                Faults::InstructionTrace,
                &bus_assertions,
                self.registers.get_full_pc_address(),
            );
        }
    }

    fn is_software_exception_command(command: u16) -> bool {
        Self::decode_processor_id(command) == ExceptionUnitExecutor::COPROCESSOR_ID
            && (command & COPROCESSOR_OPCODE_ID_MASK) >> COPROCESSOR_OPCODE_ID_LENGTH
                == ExceptionUnitOpCodes::SoftwareException as u16
    }

    /// Clear the CSR command word on completion, cancellation, or discard.
    fn clear_coprocessor_select_register(&mut self) {
        self.registers.coprocessor_select_register = 0;
    }

    /// Works out which coprocessor should run on this cycle
    fn decode_processor_id(command: u16) -> u8 {
        ((command & COPROCESSOR_ID_MASK) >> COPROCESSOR_ID_LENGTH) as u8
    }

    fn fault_from_bus_assertions(bus_assertions: BusAssertions) -> Option<Faults> {
        if bus_assertions.bus_error {
            Some(Faults::Bus)
        } else if bus_assertions.bus_protection_error {
            Some(Faults::BusProtection)
        } else {
            None
        }
    }

    fn handle_vector_fetch_bus_fault(
        &mut self,
        fault: Faults,
        fault_bus_assertions: BusAssertions,
    ) -> Option<BusAssertions> {
        if fault_bus_assertions.bus_access_type != BusAccessType::ExceptionVectorFetch {
            return None;
        }

        let (op_code, vector, level) = deconstruct_exception_unit_command(self.active_command);
        // Stop this slot without resetting phase. In particular, an aborted
        // Fault slot must not clear the replacement fault at EU write-back.
        // The next dispatch overwrites partial vector scratch state normally.
        self.slot_owner = SlotOwner::Aborted;
        self.instruction_trace = InstructionTrace::None;

        if op_code == ExceptionUnitOpCodes::Fault && vector == DOUBLE_FAULT_VECTOR {
            error!("Triple fault! [{fault:?}] raised while fetching the double fault vector. Requesting reset.");
            self.eu_registers.pending_fault = None;
            self.reset_pending = true;
            return Some(BusAssertions {
                reset_requested: true,
                ..BusAssertions::default()
            });
        }

        if op_code == ExceptionUnitOpCodes::Fault {
            error!("Double fault! [{fault:?}] raised while fetching a fault vector. Jumping to double fault vector.");
            self.eu_registers.link_registers[FAULT_METADATA_LINK_REGISTER_INDEX] =
                ExceptionLinkRegister {
                    return_address: fault_bus_assertions.address,
                    return_status_register: encode_fault_metadata_register(
                        &FaultMetadataRegister {
                            bus_access_type: fault_bus_assertions.bus_access_type,
                            double_fault: true,
                            fault: Faults::DoubleFault,
                            original_fault: fault,
                        },
                    ),
                    saved_exception_level: 0,
                };
            // Escalating to a double fault, not raising a new one: preserve the *original*
            // fault's return address rather than computing a new one, so the double-fault
            // handler still returns to the instruction that started the whole chain. This is
            // the one place `pending_fault` is constructed outside `raise_fault`.
            let original_pc = self
                .eu_registers
                .pending_fault
                .expect("a fault must already be pending when its own vector fetch bus-faults")
                .pc;
            self.eu_registers.pending_fault = Some(PendingFault {
                fault: Faults::DoubleFault,
                pc: original_pc,
            });
            return Some(BusAssertions::default());
        }

        // Entry has not committed: live SR/level still hold the state the
        // failed event would have saved. Normal fault entry saves that state.
        let return_pc = if op_code == ExceptionUnitOpCodes::SoftwareException {
            // No PU instruction can run between issuing EXCP and completing
            // its command slot, so its decode still owns the issuing PC.
            (
                self.processing_unit_executor.decoded_instruction.pc_h_,
                self.processing_unit_executor.decoded_instruction.pc_l_,
            )
                .to_full_address()
        } else {
            if op_code == ExceptionUnitOpCodes::HardwareException && (2..=6).contains(&level) {
                // Requeue L5 through the same logical enqueue as pin edges:
                // a new edge already in its bit means the failed request overran it.
                if level == ExceptionPriorities::LevelFiveHardware as u8 {
                    self.enqueue_l5();
                } else {
                    self.eu_registers.pending_hardware_exceptions |= 1 << (level - 2);
                }
            }
            self.registers.get_full_pc_address()
        };
        raise_fault(
            &mut self.eu_registers,
            fault,
            &fault_bus_assertions,
            return_pc,
        );
        Some(BusAssertions::default())
    }

    /// Called every poll cycle (not just when `level > 0`) so Level 5's edge-detection can
    /// observe the pin being released, not only asserted.
    pub fn raise_hardware_interrupt(&mut self, level: u8) {
        // This level is a bitmask
        // Only mark interrupts as pending if they are enabled
        let hw_interrupt_enable = get_hardware_interrupt_enable(&self.registers);
        let enabled_interrupts = level & hw_interrupt_enable;

        // Latch a level-sensitive line only below its handler's exception level.
        // Do not discard requests that were already pending before entering a handler.
        let first_handler_level = ExceptionPriorities::LevelOneHardware as u8;
        let first_eligible_pin = self
            .eu_registers
            .current_exception_level
            .saturating_sub(first_handler_level - 1);
        let eligible_lines =
            mask_bits_below_index(LEVEL_FIVE_INTERRUPT_PIN_BIT - 1, first_eligible_pin);
        let level_sensitive_interrupts = enabled_interrupts & eligible_lines;
        if level_sensitive_interrupts > 0 {
            trace!("Interrupt level [b{level_sensitive_interrupts:b}] raised (masked from [b{level:b}] by enable mask [b{hw_interrupt_enable:b}])");
            // TODO: Clarify what happens when software exception is triggered in interrupt handler
            // category=Hardware
            // By design it should be ignored, or cause a fault. At the moment it might just queue it up?
            // TODO: Use consistent terminology for exceptions
            // category=Refactoring
            // Exception == all exceptions, interrupt = hardware pins, fault = internal exception, software = user triggered exception
            self.eu_registers.pending_hardware_exceptions |= level_sensitive_interrupts;
            self.eu_registers.waiting_for_exception = false;
        }

        // Level 5 (NMI) is edge-triggered: only latch on a 0->1 transition of the pin, and it
        // won't latch again until it's released and reasserted.
        let l5_asserted = (level & LEVEL_FIVE_INTERRUPT_PIN_BIT) != 0;
        if l5_asserted && !self.l5_interrupt_pin_previously_asserted {
            trace!("Level 5 (NMI) interrupt raised on rising edge");
            self.enqueue_l5();
        }
        self.l5_interrupt_pin_previously_asserted = l5_asserted;
    }

    /// Enqueue a logical L5 request without touching the pin edge detector.
    /// Several lost requests coalesce into one overrun report.
    fn enqueue_l5(&mut self) {
        self.eu_registers.l5_overrun |=
            self.eu_registers.pending_hardware_exceptions & LEVEL_FIVE_INTERRUPT_PIN_BIT != 0;
        self.eu_registers.pending_hardware_exceptions |= LEVEL_FIVE_INTERRUPT_PIN_BIT;
        self.eu_registers.waiting_for_exception = false;
    }

    fn advance_phase(&mut self) {
        self.phase = (self.phase + 1) % CYCLES_PER_INSTRUCTION;
    }

    pub fn reset(&mut self) {
        self.instruction_trace = InstructionTrace::None;
        self.slot_owner = SlotOwner::ProcessingUnit;
        self.pending_bus_request = None;
        self.is_halted = false;
        self.reset_pending = false;
        self.l5_interrupt_pin_previously_asserted = false;
        self.eu_registers.waiting_for_exception = false;
        self.eu_registers.pending_fault = None;
        self.eu_registers.pending_hardware_exceptions = 0;
        self.eu_registers.l5_overrun = false;
        self.eu_registers.current_exception_level = 0;
        self.phase = 0;
        // Seeds the EU to fetch the reset vector when the RSTO hold expires
        self.registers.coprocessor_select_register =
            construct_exception_unit_command(&ExceptionUnitOpCodes::Reset, 0x0);
    }

    /// Runs the CPU for six cycles. Only to keep tests functioning at the moment. Will be removed
    ///
    /// # Panics
    /// Will panic if a coprocessor instruction is executed with a COP ID of neither 0 or 1
    #[cfg(test)]
    pub fn run_cpu(&mut self) {
        // TODO: Remove `run_cpu` function and do the polling via the bus
        // category=Testing
        // It was created as an interim to make refactoring easier but there is probably a better way to do it
        // TODO: Investigate if spinning is the best way to wait for exception
        // category=Performance
        for _ in 0..CYCLES_PER_INSTRUCTION {
            // Simulate an always-ready bus so the CPU never stalls waiting for bus_acknowledge
            self.poll(
                BusAssertions {
                    bus_acknowledge: true,
                    ..BusAssertions::default()
                },
                true,
            );
        }
    }
}
