mod executors;
pub mod instructions;
pub mod registers;

use peripheral_mem::MemoryPeripheral;

use crate::executors::Executor;
use crate::instructions::definitions::INSTRUCTION_SIZE_WORDS;
use crate::instructions::encoding::decode_instruction;
use crate::instructions::fetch::fetch_instruction;
use crate::registers::{new_registers, sr_bit_is_set, Registers, StatusRegisterFields};

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
    let initial_pc = match program_segment {
        Some(s) => s.address,
        None => {
            panic!(
                "Could not find '{}' segment in memory peripheral",
                program_segment_label
            );
        }
    };

    CpuPeripheral {
        memory_peripheral,
        registers: new_registers(Some(initial_pc)),
    }
}

fn step<'a>(registers: &'a mut Registers, mem: &MemoryPeripheral) -> Result<&'a Registers, Error> {
    use crate::instructions::definitions::Instruction::*;

    let raw_instruction = fetch_instruction(mem, registers.pc);
    let instruction = decode_instruction(raw_instruction);
    let original_pc = registers.pc;

    match instruction {
        // TODO: There has to be a better way to dispatch these
        // https://gitlab.com/antonok/enum_dispatch worked before the crates were split up
        // but doesn't work now because the shared crate would need to have the executor
        // implementations in scope which would create a circular dependency
        Halt(_) => return Err(Error::ProcessorHalted(registers.to_owned())),
        Set(data) => data.execute(registers, mem),
        Copy(data) => data.execute(registers, mem),
        Add(data) => data.execute(registers, mem),
        Subtract(data) => data.execute(registers, mem),
        Multiply(data) => data.execute(registers, mem),
        Divide(data) => data.execute(registers, mem),
        IsEqual(data) => data.execute(registers, mem),
        IsNotEqual(data) => data.execute(registers, mem),
        IsLessThan(data) => data.execute(registers, mem),
        IsGreaterThan(data) => data.execute(registers, mem),
        IsLessOrEqualThan(data) => data.execute(registers, mem),
        IsGreaterOrEqualThan(data) => data.execute(registers, mem),
        Jump(data) => data.execute(registers, mem),
        JumpIf(data) => data.execute(registers, mem),
        JumpIfNot(data) => data.execute(registers, mem),
    };

    if sr_bit_is_set(StatusRegisterFields::CpuHalted, registers) {
        return Err(Error::ProcessorHalted(registers.to_owned()));
    }

    if original_pc == registers.pc {
        // If the PC hasn't been modified by the instruction than assume that it isn't
        // a flow control instruction like a jump and just increment it.
        // TODO: Is there a more reliable/elegant way to do this?
        registers.pc += INSTRUCTION_SIZE_WORDS;
    }

    Ok(registers)
}

impl CpuPeripheral<'_> {
    pub fn run_cpu(&mut self) -> Result<(), Error> {
        loop {
            match step(&mut self.registers, self.memory_peripheral) {
                Err(error) => {
                    println!("Execution stopped:\n{:#?}", error);
                    return Err(error);
                }
                Ok(_registers) => {
                    // Debug statements for each execution step can go here
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
