mod executors;

use crate::executors::Executor;
use shared::instructions::{
    decode_instruction, encode_instruction, fetch_instruction, AddInstructionData, Instruction,
    IsLessThanInstructionData, JumpIfInstructionData, NullInstructionData, SetInstructionData,
};
use shared::registers::{new_registers, sr_bit_is_set, Registers, StatusRegisterFields};

#[derive(Debug)]
pub enum Error {
    ProcessorHalted(Registers),
    InvalidInstruction(Registers),
}

fn step<'a>(
    registers: &'a mut Registers,
    rom: &[u16],
    ram: &mut [u16],
) -> Result<&'a Registers, Error> {
    use shared::instructions::Instruction::*;

    let maybe_instruction = fetch_instruction(rom, registers.pc);
    match maybe_instruction {
        Some(raw_instruction) => {
            let instruction = decode_instruction(raw_instruction);
            let original_pc = registers.pc;

            match instruction {
                // TODO: There has to be a better way to dispatch these
                // https://gitlab.com/antonok/enum_dispatch worked before the crates were split up
                // but doesn't work now because the shared crate would need to have the executor
                // implementations in scope which would create a circular dependency
                Halt(_) => return Err(Error::ProcessorHalted(registers.to_owned())),
                Set(data) => data.execute(registers, rom, ram),
                Copy(data) => data.execute(registers, rom, ram),
                Add(data) => data.execute(registers, rom, ram),
                Subtract(data) => data.execute(registers, rom, ram),
                Multiply(data) => data.execute(registers, rom, ram),
                Divide(data) => data.execute(registers, rom, ram),
                IsEqual(data) => data.execute(registers, rom, ram),
                IsNotEqual(data) => data.execute(registers, rom, ram),
                IsLessThan(data) => data.execute(registers, rom, ram),
                IsGreaterThan(data) => data.execute(registers, rom, ram),
                IsLessOrEqualThan(data) => data.execute(registers, rom, ram),
                IsGreaterOrEqualThan(data) => data.execute(registers, rom, ram),
                Jump(data) => data.execute(registers, rom, ram),
                JumpIf(data) => data.execute(registers, rom, ram),
                JumpIfNot(data) => data.execute(registers, rom, ram),
            };

            if sr_bit_is_set(StatusRegisterFields::CpuHalted, registers) {
                return Err(Error::ProcessorHalted(registers.to_owned()));
            }

            if original_pc == registers.pc {
                // If the PC hasn't been modified by the instruction than assume that it isn't
                // a flow control instruction like a jump and just increment it.
                // TODO: Is there a more reliable/elegant way to do this?
                registers.pc += 1;
            }

            Ok(registers)
        }
        None => Err(Error::InvalidInstruction(registers.to_owned())),
    }
}

fn main() -> Result<(), Error> {
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
        match step(&mut registers, rom, ram) {
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
