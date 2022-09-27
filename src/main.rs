mod executors;
mod instructions;
mod registers;

use instructions::NullInstructionData;
use registers::StatusRegisterFields;

use crate::executors::Executor;
use crate::instructions::{
    decode_instruction, encode_instruction, fetch_instruction, AddInstructionData, Instruction,
    IsLessThanInstructionData, JumpIfInstructionData, SetInstructionData,
};
use crate::registers::{new_registers, Registers};

#[derive(Debug)]
pub enum Error {
    ProcessorHalted(Registers),
    InvalidInstruction(Registers),
}

fn step(registers: &mut Registers, rom: &[u16], ram: &mut [u16]) -> Result<Registers, Error> {
    let maybe_instruction = fetch_instruction(rom, registers.pc);
    match maybe_instruction {
        Some(raw_instruction) => {
            let instruction = decode_instruction(raw_instruction);
            instruction.execute(registers, rom, ram);

            if (registers.sr & StatusRegisterFields::CpuHalted as u16)
                == StatusRegisterFields::CpuHalted as u16
            {
                return Err(Error::ProcessorHalted(registers.to_owned()));
            }
            Ok(registers.to_owned())
        }
        None => Err(Error::InvalidInstruction(registers.to_owned())),
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
        Instruction::Set(SetInstructionData {
            register: 0,
            value: 5,
        }),
        Instruction::Set(SetInstructionData {
            register: 1,
            value: 3,
        }),
        Instruction::Set(SetInstructionData {
            register: 2,
            value: 64,
        }),
        Instruction::Add(AddInstructionData {
            src_register: 0,
            dest_register: 1,
        }),
        Instruction::IsLessThan(IsLessThanInstructionData {
            src_register: 1,
            dest_register: 2,
        }),
        Instruction::JumpIf(JumpIfInstructionData { new_pc: 3 }),
        Instruction::Halt(NullInstructionData {}),
    ]
    .iter()
    .flat_map(encode_instruction)
    .collect::<Vec<_>>();
    let rom: &[u16] = test_instructions.as_slice();

    loop {
        let original_pc = registers.pc;
        match step(&mut registers, rom, ram) {
            Err(error) => {
                println!("Execution stopped:\n{:#?}", error);
                return Err(error);
            }
            Ok(_registers) => {
                // Debug statements for each execution step can go here
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
