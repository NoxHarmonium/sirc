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
    clippy::missing_const_for_fn
)]

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
pub mod execution;
pub mod instructions;
pub mod microcode;
pub mod registers;

use instructions::definitions::Instruction;
use peripheral_mem::MemoryPeripheral;
use registers::{sr_bit_is_set, FullAddress, StatusRegisterFields};

use crate::execution::execution_effective_address::ExecutionEffectiveAddressExecutor;
use crate::execution::fetch_and_decode::decode_and_register_fetch;
use crate::execution::memory_access::MemoryAccessExecutor;
use crate::execution::shared::{IntermediateRegisters, StageExecutor};
use crate::execution::write_back::WriteBackExecutor;
use crate::instructions::fetch::fetch_instruction;
use crate::registers::{Registers, SegmentedRegisterAccess};

/// Its always six baby!
pub const CYCLES_PER_INSTRUCTION: u32 = 6;

#[derive(Debug)]
pub enum Error {
    ProcessorHalted(Registers),
    InvalidInstruction(Registers),
}

pub struct CpuPeripheral<'a> {
    pub memory_peripheral: &'a MemoryPeripheral,
    pub registers: Registers,
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
    }
}

#[allow(clippy::cast_possible_truncation)]
fn step<'a>(
    registers: &'a mut Registers,
    mem: &MemoryPeripheral,
) -> Result<(&'a Registers, u32, Instruction), Error> {
    // 1. Instruction Fetch (1/2)
    // 2. Instruction Fetch (2/2)
    let raw_instruction = fetch_instruction(mem, registers.get_segmented_pc());

    // 3. Decode/Register Fetch (ID)
    let decoded_instruction = decode_and_register_fetch(raw_instruction, registers);

    println!("{decoded_instruction:#X?}");

    // Special instruction just for debugging purposes. Probably won't be in hardware
    assert!(
        !(decoded_instruction.ins == Instruction::Exception && decoded_instruction.sr_b_ == 0xFFFF),
        "Execution was halted due to 0xFFFF exception"
    );

    // TODO: On the real CPU these might have garbage in them?
    // maybe it should only be zeroed on first run and shared between invocations
    let mut intermediate_registers = IntermediateRegisters {
        alu_output: 0,
        alu_status_register: 0,
        lmd: 0,
    };

    ExecutionEffectiveAddressExecutor::execute(
        &decoded_instruction,
        registers,
        &mut intermediate_registers,
        mem,
    );
    MemoryAccessExecutor::execute(
        &decoded_instruction,
        registers,
        &mut intermediate_registers,
        mem,
    );
    WriteBackExecutor::execute(
        &decoded_instruction,
        registers,
        &mut intermediate_registers,
        mem,
    );

    if sr_bit_is_set(StatusRegisterFields::CpuHalted, registers) {
        return Err(Error::ProcessorHalted(registers.clone()));
    }

    println!("step: {:X?} {:X?}", decoded_instruction.ins, registers);

    Ok((registers, CYCLES_PER_INSTRUCTION, decoded_instruction.ins))
}

impl CpuPeripheral<'_> {
    pub fn run_cpu(&mut self, clock_quota: u32) -> Result<u32, Error> {
        let mut clocks: u32 = 0;
        loop {
            match step(&mut self.registers, self.memory_peripheral) {
                Err(error) => {
                    println!("Execution stopped:\n{error:08x?}");
                    return Err(error);
                }
                Ok((_registers, instruction_clocks, _instruction)) => {
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
