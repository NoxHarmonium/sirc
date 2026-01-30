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
        definitions::{ExceptionPriorities, ExceptionUnitOpCodes, Faults},
        execution::{construct_cause_value, get_cause_register_value, ExceptionUnitExecutor},
    },
    processing_unit::execution::ProcessingUnitExecutor,
    shared::{ExecutionPhase, Executor},
};
use num::ToPrimitive;
use num_traits::FromPrimitive;
use peripheral_bus::device::{BusAssertions, Device};
use registers::{get_interrupt_mask, ExceptionUnitRegisters};

use crate::registers::{ExceptionLinkRegister, Registers};

// The 8th exception link register stores metadata about faults
const FAULT_METADATA_LINK_REGISTER_INDEX: usize = 7;

/// Its always six baby!
pub const CYCLES_PER_INSTRUCTION: u32 = 6;

// Cause register components
pub const COPROCESSOR_ID_MASK: u16 = 0xF000;
pub const COPROCESSOR_ID_LENGTH: u16 = 12;
pub const CAUSE_OPCODE_ID_MASK: u16 = 0x0F00;
pub const CAUSE_OPCODE_ID_LENGTH: u16 = 8;

// pub phase: u8, // 3 bits
// pub double_fault: bool, // 1 bit
// pub fault: Faults, // 4 bits (needs to hold values up to 0x8)
// pub original_fault: Faults // 4 bits (needs to hold values up to 0x8)

// Fault metadata components
pub const PHASE_MASK: u16 = 0x7;
pub const PHASE_LENGTH: u16 = 0;
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
}

#[derive(Debug, PartialEq, Eq)]
pub struct FaultMetadataRegister {
    pub phase: u8,          // 3 bits (bits 0-2)
    pub double_fault: bool, // 1 bit (bit 3)
    pub fault: Faults,      // 4 bits (bits 4-7, needs to hold values up to 0x8)
    // If double_fault holds the fault that caused the double fault
    pub original_fault: Faults, // 4 bits (bits 8-11, needs to hold values up to 0x8)
}

pub fn decode_fault_metadata_register(val: u16) -> FaultMetadataRegister {
    FaultMetadataRegister {
        phase: ((val & PHASE_MASK) >> PHASE_LENGTH) as u8,
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
        | u16::from(reg.phase)
}

pub fn raise_fault(
    registers: &Registers,
    eu_registers: &mut ExceptionUnitRegisters,
    fault: Faults,
    phase: ExecutionPhase,
    bus_assertions: &BusAssertions,
) -> Option<Faults> {
    // Disabled during refactor

    if let Some(pending_fault) = eu_registers.pending_fault {
        // TODO: What would happen in hardware if a fault was raised when one was already pending
        // category=Hardware
        // Is this possible in hardware? If so, what would happen?
        panic!("Cannot raise fault when one is pending. Trying to raise {fault:?} but {pending_fault:?} is already pending.");
    }

    debug!(
        "raise_fault: fault: {fault:?} address: 0x{:X} phase: {phase:?}",
        bus_assertions.address,
    );

    let current_interrupt_mask: u8 = get_interrupt_mask(registers);
    let is_double_fault = current_interrupt_mask >= ExceptionPriorities::Fault as u8;
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
        // 1 bit for phase
        // 1 bit for double fault flag
        // 3 bits for fault type
        // 3 bits for original fault type (if double fault)
        return_status_register: encode_fault_metadata_register(&FaultMetadataRegister {
            phase: phase as u8,
            double_fault: is_double_fault,
            fault: resolved_fault,
            original_fault: fault,
        }),
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
    }
}

impl Device for CpuPeripheral {
    #[allow(clippy::cast_possible_truncation)]
    fn poll(&mut self, bus_assertions: BusAssertions, _: bool) -> BusAssertions {
        let phase: ExecutionPhase =
            FromPrimitive::from_u8(self.phase).expect("Expected phase to be between 0-5");

        if bus_assertions.bus_error {
            // TODO: Reduce number of mutable references in `CpuPeripheral` poll
            // category=Refactoring
            // Can we do this without have a mut ref to eu_registers? See also `get_cause_register_value`
            self.eu_registers.pending_fault = raise_fault(
                &self.registers,
                &mut self.eu_registers,
                Faults::Bus,
                phase,
                &bus_assertions,
            );
        }
        if bus_assertions.interrupt_assertion > 0 {
            self.raise_hardware_interrupt(bus_assertions.interrupt_assertion);
        }

        if phase == ExecutionPhase::InstructionFetchLow {
            // Only reset the cause register every full instruction cycle
            self.cause_register_value =
                get_cause_register_value(&self.registers, &mut self.eu_registers);
        }

        let coprocessor_id = Self::decode_processor_id(self.cause_register_value);

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
                warn!("Invalid op code detected");
                // TODO: Double check invalid op code fault handling
                // category=Hardware
                // This doesn't seem to line up with how the other faults are handled
                // because of this check. We need it because the cause register is currently
                // only set on the first phase of the CPU cycles, so this gets run each cycle
                let should_fault = self.eu_registers.pending_fault != Some(Faults::InvalidOpCode);

                if should_fault {
                    // Can be used for forwards compatibility if co-processors are added in later models
                    self.eu_registers.pending_fault = raise_fault(
                        &self.registers,
                        &mut self.eu_registers,
                        Faults::InvalidOpCode,
                        phase,
                        &bus_assertions,
                    );
                }

                BusAssertions::default()
            }
        };

        // TODO: Investigate performance impact of unrolling the six CPU phases
        // category=Performance
        self.phase = (self.phase + 1) % CYCLES_PER_INSTRUCTION as u8;

        result
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

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl CpuPeripheral {
    /// Works out which coprocessor should run on this cycle
    fn decode_processor_id(cause_register_value: u16) -> u8 {
        ((cause_register_value & COPROCESSOR_ID_MASK) >> COPROCESSOR_ID_LENGTH) as u8
    }

    pub fn raise_hardware_interrupt(&mut self, level: u8) {
        // This level is a bitmask
        if level > 0 {
            trace!("Interrupt level [b{level:b}] raised");
            // TODO: Clarify what happens when software exception is triggered in interrupt handler
            // category=Hardware
            // By design it should be ignored, or cause a fault. At the moment it might just queue it up?
            // TODO: Use consistent terminology for exceptions
            // category=Refactoring
            // Exception == all exceptions, interrupt = hardware pins, fault = internal exception, software = user triggered exception
            self.eu_registers.pending_hardware_exceptions |= level;
            self.eu_registers.waiting_for_exception = false;
        }
    }

    pub fn reset(&mut self) {
        // Will cause the exception coprocessor to jump to reset vector
        let reset_cause_value = construct_cause_value(&ExceptionUnitOpCodes::Reset, 0x0);
        self.registers.pending_coprocessor_command = reset_cause_value;
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
        if !self.eu_registers.waiting_for_exception {
            for _ in 0..CYCLES_PER_INSTRUCTION {
                self.poll(BusAssertions::default(), true);
            }
        }
    }
}
