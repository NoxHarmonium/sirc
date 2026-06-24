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
        },
        execution::{
            construct_cause_value, deconstruct_cause_value, get_cause_register_value,
            ExceptionUnitExecutor,
        },
    },
    processing_unit::execution::ProcessingUnitExecutor,
    shared::{ExecutionPhase, Executor},
};
use num::ToPrimitive;
use num_traits::FromPrimitive;
use peripheral_bus::device::{BusAccessType, BusAssertions, Device};
use registers::ExceptionUnitRegisters;

use crate::registers::{
    get_hardware_interrupt_enable, sr_bit_is_set, ExceptionLinkRegister, Registers,
    StatusRegisterFields,
};

// The 8th exception link register stores metadata about faults
const FAULT_METADATA_LINK_REGISTER_INDEX: usize = 7;

/// Its always six baby!
pub const CYCLES_PER_INSTRUCTION: u8 = 6;

// Cause register components
pub const COPROCESSOR_ID_MASK: u16 = 0xF000;
pub const COPROCESSOR_ID_LENGTH: u16 = 12;
pub const CAUSE_OPCODE_ID_MASK: u16 = 0x0F00;
pub const CAUSE_OPCODE_ID_LENGTH: u16 = 8;

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

pub struct CpuPeripheral {
    pub registers: Registers,
    pub eu_registers: ExceptionUnitRegisters,
    pub phase: u8,
    pub processing_unit_executor: ProcessingUnitExecutor,
    pub exception_unit_executor: ExceptionUnitExecutor,
    pub cause_register_value: u16,
    // T bit sampled at InstructionFetchLow before any SR writes for the current instruction.
    // Used at WriteBackExecutor to decide whether to raise InstructionTrace.
    // Only set when the processing unit is running (not during exception unit dispatch).
    trace_mode_sampled: bool,
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

pub fn raise_fault(
    eu_registers: &mut ExceptionUnitRegisters,
    fault: Faults,
    bus_assertions: &BusAssertions,
) -> Option<Faults> {
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
    Some(resolved_fault)
}

///
/// Instantiates a new `CpuPeripheral` with default values after doing some checks.
///
/// # Panics
/// Will panic if the specified `program_segment_label` is not a segment defined in the provided `memory_peripheral`
///
#[must_use]
pub fn new_cpu_peripheral(system_ram_offset: u32) -> CpuPeripheral {
    CpuPeripheral {
        registers: Registers {
            system_ram_offset,
            ..Registers::default()
        },
        eu_registers: ExceptionUnitRegisters::default(),
        phase: 0,
        processing_unit_executor: ProcessingUnitExecutor::default(),
        exception_unit_executor: ExceptionUnitExecutor::default(),
        cause_register_value: 0,
        trace_mode_sampled: false,
        pending_bus_request: None,
        is_halted: false,
        reset_pending: false,
    }
}

impl Device for CpuPeripheral {
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
        let result = self.poll_internal(bus_assertions, selected);
        BusAssertions {
            protected_mode_active: sr_bit_is_set(
                StatusRegisterFields::ProtectedMode,
                &self.registers,
            ),
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

        // Allow wake up if there is a hardware interrupt
        if bus_assertions.interrupt_assertion > 0 {
            self.raise_hardware_interrupt(bus_assertions.interrupt_assertion);
        }

        if self.eu_registers.waiting_for_exception {
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
            if let Some(vector_fetch_fault_response) =
                self.handle_vector_fetch_bus_fault(bus_fault, fault_bus_assertions)
            {
                return vector_fetch_fault_response;
            }

            // TODO: Reduce number of mutable references in `CpuPeripheral` poll
            // category=Refactoring
            // Can we do this without have a mut ref to eu_registers? See also `get_cause_register_value`
            self.eu_registers.pending_fault =
                raise_fault(&mut self.eu_registers, bus_fault, &fault_bus_assertions);
        }

        if phase == ExecutionPhase::InstructionFetchLow {
            // Only reset the cause register every full instruction cycle
            let pending_coprocessor_command = self.registers.pending_coprocessor_command;
            let pending_cop_opcode =
                (pending_coprocessor_command & CAUSE_OPCODE_ID_MASK) >> CAUSE_OPCODE_ID_LENGTH;
            let ignored_software_exception = self.eu_registers.current_exception_level != 0
                && Self::decode_processor_id(pending_coprocessor_command)
                    == ExceptionUnitExecutor::COPROCESSOR_ID
                && pending_cop_opcode == ExceptionUnitOpCodes::SoftwareException as u16;
            if ignored_software_exception {
                self.registers.pending_coprocessor_command = 0x0;
            }

            self.cause_register_value =
                get_cause_register_value(&self.registers, &mut self.eu_registers);
        }

        let coprocessor_id = Self::decode_processor_id(self.cause_register_value);

        // Sample the T bit once at InstructionFetchLow, before this instruction can modify SR.
        // In hardware, the trace decision reads the registered (committed) SR value from the
        // previous clock edge, not the combinational new value being written this cycle.
        // Only sample for processing-unit instructions; EU exception dispatch is not traced.
        if phase == ExecutionPhase::InstructionFetchLow {
            self.trace_mode_sampled = coprocessor_id == ProcessingUnitExecutor::COPROCESSOR_ID
                && (sr_bit_is_set(StatusRegisterFields::TraceMode, &self.registers)
                    || bus_assertions.force_trace_mode);
        }

        trace!(
            "coprocessor_id: {coprocessor_id} eu_registers.pending_fault: {:?}",
            self.eu_registers.pending_fault
        );

        let result = match coprocessor_id {
            ProcessingUnitExecutor::COPROCESSOR_ID => self.processing_unit_executor.step(
                &phase,
                self.cause_register_value,
                &mut self.registers,
                &mut self.eu_registers,
                bus_assertions,
            ),
            ExceptionUnitExecutor::COPROCESSOR_ID => self.exception_unit_executor.step(
                &phase,
                self.cause_register_value,
                &mut self.registers,
                &mut self.eu_registers,
                bus_assertions,
            ),
            _ => {
                if phase == ExecutionPhase::InstructionFetchLow {
                    warn!("Invalid COP coprocessor ID detected: {coprocessor_id}");
                    // Can be used for forwards compatibility if co-processors are added in later models
                    self.eu_registers.pending_fault = raise_fault(
                        &mut self.eu_registers,
                        Faults::InvalidOpCode,
                        &bus_assertions,
                    );
                }

                BusAssertions::default()
            }
        };

        if phase == ExecutionPhase::WriteBackExecutor
            && self.eu_registers.pending_fault.is_none()
            && self.trace_mode_sampled
        {
            self.eu_registers.pending_fault = raise_fault(
                &mut self.eu_registers,
                Faults::InstructionTrace,
                &bus_assertions,
            );
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

        // Software RSET: after WriteBack wraps phase back to 0, if pending_coprocessor_command
        // was just set to the reset cause value, signal the reset unit via reset_requested so it starts
        // the 6-cycle RSTO hold before the EU fetches the reset vector.
        let reset_cause = construct_cause_value(&ExceptionUnitOpCodes::Reset, 0x0);
        if self.phase == 0 && self.registers.pending_coprocessor_command == reset_cause {
            return BusAssertions {
                reset_requested: true,
                ..result
            };
        }

        result
    }

    /// Works out which coprocessor should run on this cycle
    fn decode_processor_id(cause_register_value: u16) -> u8 {
        ((cause_register_value & COPROCESSOR_ID_MASK) >> COPROCESSOR_ID_LENGTH) as u8
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

        let (op_code, vector, _) = deconstruct_cause_value(self.cause_register_value);
        // Abort the current exception-unit dispatch. The next dispatch starts at phase 0 and
        // overwrites any partially fetched vector scratch state in the exception-unit executor.
        self.phase = 0;

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
            self.eu_registers.pending_fault = Some(Faults::DoubleFault);
            return Some(BusAssertions::default());
        }

        self.eu_registers.pending_fault =
            raise_fault(&mut self.eu_registers, fault, &fault_bus_assertions);
        Some(BusAssertions::default())
    }

    pub fn raise_hardware_interrupt(&mut self, level: u8) {
        // This level is a bitmask
        if level > 0 {
            // Only mark interrupts as pending if they are enabled
            let hw_interrupt_enable = get_hardware_interrupt_enable(&self.registers);
            let enabled_interrupts = level & hw_interrupt_enable;

            if enabled_interrupts > 0 {
                trace!("Interrupt level [b{enabled_interrupts:b}] raised (masked from [b{level:b}] by enable mask [b{hw_interrupt_enable:b}])");
                // TODO: Clarify what happens when software exception is triggered in interrupt handler
                // category=Hardware
                // By design it should be ignored, or cause a fault. At the moment it might just queue it up?
                // TODO: Use consistent terminology for exceptions
                // category=Refactoring
                // Exception == all exceptions, interrupt = hardware pins, fault = internal exception, software = user triggered exception
                self.eu_registers.pending_hardware_exceptions |= enabled_interrupts;
                self.eu_registers.waiting_for_exception = false;
            }
        }
    }

    fn advance_phase(&mut self) {
        self.phase = (self.phase + 1) % CYCLES_PER_INSTRUCTION;
    }

    pub fn reset(&mut self) {
        self.pending_bus_request = None;
        self.is_halted = false;
        self.eu_registers.waiting_for_exception = false;
        self.eu_registers.pending_fault = None;
        self.eu_registers.pending_hardware_exceptions = 0;
        self.eu_registers.current_exception_level = 0;
        self.phase = 0;
        // Seeds the EU to fetch the reset vector when the RSTO hold expires
        self.registers.pending_coprocessor_command =
            construct_cause_value(&ExceptionUnitOpCodes::Reset, 0x0);
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
