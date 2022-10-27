// TODO: Can we expose the Executor trait here without exposing the implementations?
// OR can we keep everything private and somehow enable tests to reach inside?
pub mod executors;
pub mod instructions;
pub mod interrupts;
pub mod registers;

use instructions::definitions::get_clocks_for_instruction;
use peripheral_mem::MemoryPeripheral;
use registers::{sr_bit_is_set, FullAddress, StatusRegisterFields};

use crate::executors::Executor;
use crate::instructions::definitions::INSTRUCTION_SIZE_WORDS;
use crate::instructions::encoding::decode_instruction;
use crate::instructions::fetch::fetch_instruction;
use crate::registers::{Registers, SegmentedRegisterAccess};

#[derive(Debug)]
pub enum Error {
    ProcessorHalted(Registers),
    InvalidInstruction(Registers),
}

pub struct CpuPeripheral<'a> {
    pub memory_peripheral: &'a MemoryPeripheral,
    pub registers: Registers,
}

pub fn new_cpu_peripheral<'a>(
    memory_peripheral: &'a MemoryPeripheral,
    program_segment_label: &str,
) -> CpuPeripheral<'a> {
    let program_segment = memory_peripheral.get_segment_for_label(program_segment_label);
    let (ph, pl) = match program_segment {
        Some(s) => s.address.to_segmented_address(),
        None => {
            panic!(
                "Could not find '{}' segment in memory peripheral",
                program_segment_label
            );
        }
    };

    CpuPeripheral {
        memory_peripheral,
        registers: Registers {
            ph,
            pl,
            ..Registers::default()
        },
    }
}

fn step<'a>(
    registers: &'a mut Registers,
    mem: &MemoryPeripheral,
) -> Result<(&'a Registers, u32), Error> {
    let raw_instruction = fetch_instruction(mem, registers.get_segmented_pc());
    let instruction = decode_instruction(raw_instruction);

    let original_pc = registers.get_segmented_pc();

    instruction.execute(registers, mem);

    if original_pc == registers.get_segmented_pc() {
        // If the PC hasn't been modified by the instruction than assume that it isn't
        // a flow control instruction like a jump and just increment it.
        // TODO: Is there a more reliable/elegant way to do this?
        // TODO TODO: Need to handle pl overflow into ph (maybe segment overflow should be an exception?)
        registers.pl += INSTRUCTION_SIZE_WORDS as u16;
    }

    let clocks = get_clocks_for_instruction(&instruction);

    if sr_bit_is_set(StatusRegisterFields::CpuHalted, registers) {
        return Err(Error::ProcessorHalted(registers.to_owned()));
    }

    Ok((registers, clocks))
}

impl CpuPeripheral<'_> {
    pub fn run_cpu(&mut self, clock_quota: u32) -> Result<u32, Error> {
        let mut clocks: u32 = 0;
        loop {
            match step(&mut self.registers, self.memory_peripheral) {
                Err(error) => {
                    println!("Execution stopped:\n{:08x?}", error);
                    return Err(error);
                }
                Ok((_registers, instruction_clocks)) => {
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
