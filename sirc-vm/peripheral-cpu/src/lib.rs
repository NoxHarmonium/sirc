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

// TODO: Can we expose the Executor trait here without exposing the implementations?
// OR can we keep everything private and somehow enable tests to reach inside?
pub mod coprocessors;
pub mod registers;
pub mod util;

use std::{any::Any, fmt::Write};

use log::{error, trace, warn};

use coprocessors::{
    exception_unit::{
        definitions::{ExceptionPriorities, ExceptionUnitOpCodes, Faults},
        execution::{construct_cause_value, get_cause_register_value, ExceptionUnitExecutor},
    },
    processing_unit::execution::ProcessingUnitExecutor,
    shared::{ExecutionPhase, Executor},
};
use num_traits::FromPrimitive;
use peripheral_bus::device::{BusAssertions, Device};
use registers::{
    get_interrupt_mask, ExceptionLinkRegister, ExceptionUnitRegisters,
    FAULT_METADATA_LINK_REGISTER_INDEX,
};

use crate::registers::Registers;

/// Its always six baby!
pub const CYCLES_PER_INSTRUCTION: u32 = 6;

// Cause register components
pub const COPROCESSOR_ID_MASK: u16 = 0xF000;
pub const COPROCESSOR_ID_LENGTH: u16 = 12;
pub const CAUSE_OPCODE_ID_MASK: u16 = 0x0F00;
pub const CAUSE_OPCODE_ID_LENGTH: u16 = 8;

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

pub fn raise_fault(
    registers: &Registers,
    eu_registers: &mut ExceptionUnitRegisters,
    fault: Faults,
    phase: ExecutionPhase,
    bus_assertions: &BusAssertions,
) -> Option<Faults> {
    // Disabled during refactor

    if let Some(pending_fault) = eu_registers.pending_fault {
        // TODO: Is this possible in hardware? If so, what would happen?
        panic!("Cannot raise fault when one is pending. Trying to raise {fault:?} but {pending_fault:?} is already pending.");
    }

    let current_interrupt_mask: u8 = get_interrupt_mask(registers);
    if current_interrupt_mask >= ExceptionPriorities::Fault as u8 {
        error!("Double fault! [{fault:?}] raised when a fault was already being serviced ");
        panic!("Double faults are unhandled right now and crash the VM. In the future they would halt the CPU.");
    }

    eu_registers.link_registers[FAULT_METADATA_LINK_REGISTER_INDEX] = ExceptionLinkRegister {
        return_address: bus_assertions.address,
        return_status_register: phase as u16, // TODO: phase only takes up u8 (or less), could fit more data in here - what is useful?
    };

    Some(fault)
}

///
/// Instantiates a new `CpuPeripheral` with default values after doing some checks.
///
/// # Panics
/// Will panic if the specified `program_segment_label` is not a segment defined in the provided `memory_peripheral`
///
#[must_use]
pub fn new_cpu_peripheral(system_ram_offset: u32) -> CpuPeripheral {
    // let program_segment = memory_peripheral.get_segment_for_label(program_segment_label);
    // let (ph, pl) = program_segment.map_or_else(
    //     || panic!("Could not find '{program_segment_label}' segment in memory peripheral"),
    //     |s| s.address.to_segmented_address(),
    // );

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
            // TODO: Can we do this without have a mut ref to eu_registers?
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
            // TODO: Can we do this without have a mut ref to eu_registers?
            self.cause_register_value =
                get_cause_register_value(&self.registers, &mut self.eu_registers);
        }

        let coprocessor_id = Self::decode_processor_id(self.cause_register_value);

        trace!(
            "coprocessor_id: {coprocessor_id} eu_registers.pending_fault: {:?}",
            self.eu_registers.pending_fault
        );

        // TODO: Do something with error results, instead of unwrap
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
                // TODO: This doesn't seem to line up with how the other faults are handled
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
            // TODO: WHAT AM I DOING
            // This pending exception level needs to be a combination of every pending interrupt
            // so this should be an OR I guess
            // Also it seems like some code treats this as a regular number, when it is actually a bit mask. Fix that so that L5 interrupts actually work
            // If a lower level interrupt comes in while a higher level is being serviced, it should be set as pending. WHen the higher level exception returns, the CPU will immediately service the next highest level interrupt
            // (could probably test this in the hardware exception example by queuing up all 5 exception levels)
            //  TODO: What happens when a software exception is triggered in an interrupt handler? By design it should be ignored, or cause a fault. At the moment it might just queue it up?
            // I think L2 interrupts were working because they fit into the link register array - it accidently worked. Probably should have tested all the interrupt levels
            // TODO: Use consistent terminology. Exception == all exceptions, interrupt = hardware pins, fault = internal exception, software = user triggered exception
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
        // TODO: Remove this function and do the polling via the bus
        // TODO: It isn't ideal to spin the cpu when waiting for exception. Should probably come up with something more clever
        if !self.eu_registers.waiting_for_exception {
            for _ in 0..CYCLES_PER_INSTRUCTION {
                // TODO: this needs a bus
                self.poll(BusAssertions::default(), true);
            }
        }
    }
}
