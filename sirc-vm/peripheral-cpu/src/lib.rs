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

use coprocessors::{
    exception_unit::{
        definitions::ExceptionUnitOpCodes,
        execution::{construct_cause_value, get_cause_register_value, ExceptionUnitExecutor},
    },
    processing_unit::execution::ProcessingUnitExecutor,
    shared::Executor,
};
use peripheral_mem::MemoryPeripheral;
use registers::{ExceptionUnitRegisters, FullAddress};

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

#[derive(Clone)]
pub struct CpuPeripheral<'a> {
    pub memory_peripheral: &'a MemoryPeripheral,
    pub registers: Registers,
    pub eu_registers: ExceptionUnitRegisters,
}

///
/// Instantiates a new `CpuPeripheral` with default values after doing some checks.
///
/// # Panics
/// Will panic if the specified `program_segment_label` is not a segment defined in the provided `memory_peripheral`
///
#[must_use]
pub fn new_cpu_peripheral<'a>(
    memory_peripheral: &'a MemoryPeripheral,
    program_segment_label: &str,
) -> CpuPeripheral<'a> {
    let program_segment = memory_peripheral.get_segment_for_label(program_segment_label);
    let (ph, pl) = program_segment.map_or_else(
        || panic!("Could not find '{program_segment_label}' segment in memory peripheral"),
        |s| s.address.to_segmented_address(),
    );

    CpuPeripheral {
        memory_peripheral,
        registers: Registers {
            ph,
            pl,
            ..Registers::default()
        },
        eu_registers: ExceptionUnitRegisters::default(),
    }
}

impl CpuPeripheral<'_> {
    /// Works out which coprocessor should run on this cycle
    fn decode_processor_id(cause_register_value: u16) -> u8 {
        ((cause_register_value & COPROCESSOR_ID_MASK) >> COPROCESSOR_ID_LENGTH) as u8
    }

    pub fn raise_hardware_interrupt(&mut self, level: u8) {
        if level > self.eu_registers.pending_hardware_exception_level {
            self.eu_registers.pending_hardware_exception_level = level;
        }
    }

    pub fn reset(&mut self) {
        // Will cause the exception coprocessor to jump to reset vector
        let reset_cause_value = construct_cause_value(&ExceptionUnitOpCodes::Reset, 0x0);
        self.registers.pending_coprocessor_command = reset_cause_value;
    }

    ///
    /// # Panics
    /// Will panic if a coprocessor instruction is executed with a COP ID of neither 0 or 1
    pub fn run_cpu(&mut self, clock_quota: u32) -> Result<u32, Error> {
        let mut clocks: u32 = 0;
        loop {
            // TODO: use a better name than "cause" register and make it consistent across everywhere
            let cause_register_value =
                get_cause_register_value(&self.registers, &self.eu_registers);
            let coprocessor_id = CpuPeripheral::decode_processor_id(cause_register_value);
            let result = match coprocessor_id {
                ProcessingUnitExecutor::COPROCESSOR_ID => ProcessingUnitExecutor::step(
                    cause_register_value,
                    &mut self.registers,
                    &mut self.eu_registers,
                    self.memory_peripheral,
                ),
                ExceptionUnitExecutor::COPROCESSOR_ID => ExceptionUnitExecutor::step(
                    cause_register_value,
                    &mut self.registers,
                    &mut self.eu_registers,
                    self.memory_peripheral,
                ),
                _ => {
                    // TODO: Work out what would happen in hardware here
                    // Do we want another coprocessor (multiplication?)
                    panic!("Coprocessor ID [{coprocessor_id}] not implemented yet")
                }
            };

            match result {
                Err(error) => {
                    println!("Execution stopped:\n{error:08x?}");
                    return Err(error);
                }
                Ok((_registers, _eu_registers, instruction_clocks)) => {
                    // println!("{:?} {:?}", instruction, registers);
                    clocks += instruction_clocks;

                    if clocks >= clock_quota {
                        // Exit if quota is reached to allow other devices to run
                        return Ok(clocks);
                    }
                }
            }
        }
    }
}
