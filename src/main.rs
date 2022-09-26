mod executors;
mod instructions;
mod registers;

use std::env::args;

use crate::executors::Executor;
use crate::instructions::{
    decode_instruction, encode_instruction, fetch_instruction, instruction_size,
    AddInstructionData, Instruction, IsLessThanInstructionData, JumpIfInstructionData,
    SetInstructionData,
};
use crate::registers::{new_registers, Registers};

#[derive(Debug)]
pub enum Error {
    ProcessorHalted(Registers),
    InvalidInstruction(Registers),
}

fn step(registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> Result<Registers, Error> {
    use Instruction::*;

    let maybe_instruction = fetch_instruction(rom, registers.pc);
    match maybe_instruction {
        Some(raw_instruction) => {
            let instruction = decode_instruction(raw_instruction);

            match instruction {
                HaltInstruction() => return Err(Error::ProcessorHalted(registers.to_owned())),
                // TODO: There has to be a better way to dispatch these
                SetInstruction(data) => data.execute(registers, rom, ram),
                CopyInstruction(data) => data.execute(registers, rom, ram),
                AddInstruction(data) => data.execute(registers, rom, ram),
                SubtractInstruction(data) => data.execute(registers, rom, ram),
                MultiplyInstruction(data) => data.execute(registers, rom, ram),
                DivideInstruction(data) => data.execute(registers, rom, ram),
                IsEqualInstruction(data) => data.execute(registers, rom, ram),
                IsNotEqualInstruction(data) => data.execute(registers, rom, ram),
                IsLessThanInstruction(data) => data.execute(registers, rom, ram),
                IsGreaterThanInstruction(data) => data.execute(registers, rom, ram),
                IsLessOrEqualThanInstruction(data) => data.execute(registers, rom, ram),
                IsGreaterOrEqualThanInstruction(data) => data.execute(registers, rom, ram),
                JumpInstruction(data) => data.execute(registers, rom, ram),
                JumpIfInstruction(data) => data.execute(registers, rom, ram),
                JumpIfNotInstruction(data) => data.execute(registers, rom, ram),
            };
            return Ok(registers.to_owned());
        }
        None => return Err(Error::InvalidInstruction(registers.to_owned())),
    }
}

fn main() -> Result<(), Error> {
    // let a = args();
    // let arg_vec: Vec<_> = a.collect();
    // println!("Hello, world! {:?}", &arg_vec);

    let mut registers = new_registers();

    // TODO: How to allocate array of zeros (e.g. calloc)
    // let mut ram: [u16; 1024];
    let mut ram_vector = vec![0u16, 1024];
    let ram: &mut [u16] = ram_vector.as_mut_slice();

    let test_instructions = vec![
        Instruction::SetInstruction(SetInstructionData {
            register: 0,
            value: 5,
        }),
        Instruction::SetInstruction(SetInstructionData {
            register: 1,
            value: 3,
        }),
        Instruction::SetInstruction(SetInstructionData {
            register: 2,
            value: 64,
        }),
        Instruction::AddInstruction(AddInstructionData {
            src_register: 0,
            dest_register: 1,
        }),
        Instruction::IsLessThanInstruction(IsLessThanInstructionData {
            src_register: 1,
            dest_register: 2,
        }),
        Instruction::JumpIfInstruction(JumpIfInstructionData { new_pc: 3 }),
        Instruction::HaltInstruction(),
    ]
    .iter()
    .map(encode_instruction)
    .flatten()
    .collect::<Vec<_>>();
    let rom: &[u16] = test_instructions.as_slice();

    loop {
        let original_pc = registers.pc;
        match step(&mut registers, &rom, ram) {
            Err(error) => {
                println!("Execution stopped:\n{:#?}", error);
                return Err(error);
            }
            Ok(registers) => {
                // println!("Step:\n{:#?}", registers);
            }
        }
        if original_pc == registers.pc {
            // If the PC hasn't been modified by the instruction than assume that it isn't
            // a flow control instruction like a jump and just increment it.
            // TODO: Is there a more reliable/elegant way to do this?
            registers.pc += 1;
        }
    }
}
