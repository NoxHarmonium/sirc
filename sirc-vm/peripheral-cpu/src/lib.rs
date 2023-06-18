extern crate num;
#[macro_use]
extern crate num_derive;

// TODO: Can we expose the Executor trait here without exposing the implementations?
// OR can we keep everything private and somehow enable tests to reach inside?
pub mod executors;
pub mod instructions;
pub mod microcode;
pub mod registers;

use executors::IntermediateRegisters;
use instructions::alu::{perform_alu_operation, AluOp};
use instructions::definitions::{
    decode_execution_step_instruction_type, decode_memory_access_step_instruction_type,
    decode_register_fetch, decode_write_back_step_instruction_type, Instruction,
};
use microcode::address::sign_extend_small_offset;
use peripheral_mem::MemoryPeripheral;
use registers::{sr_bit_is_set, FullAddress, SegmentedAddress, StatusRegisterFields};

use crate::instructions::definitions::INSTRUCTION_SIZE_WORDS;
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
) -> Result<(&'a Registers, u32, Instruction), Error> {
    // 1. Instruction Fetch (1/2)
    // 2. Instruction Fetch (2/2)
    let raw_instruction = fetch_instruction(mem, registers.get_segmented_pc());

    // 3. Decode/Register Fetch (ID)
    let decoded = decode_register_fetch(raw_instruction, registers);
    // TODO: Handle exception case instead of unwrap
    let instruction: Instruction = num::FromPrimitive::from_u8(decoded.ins).unwrap();

    // TODO: On the real CPU these might have garbage in them?
    // maybe it should only be zeroed on first run and shared between invocations
    let mut intermediate_registers = IntermediateRegisters {
        alu_output: 0,
        lmd: 0,
        npc: registers.pl.wrapping_add(INSTRUCTION_SIZE_WORDS as u16),
    };

    let alu_code = decoded.ins & 0xF;
    // TODO: Should this be unwrap? - clean this up
    let alu_op: AluOp = num::FromPrimitive::from_u8(alu_code).unwrap();

    let execution_step_instruction_type =
        decode_execution_step_instruction_type(&instruction, &decoded);

    // 4. ====== Execution (EX) ======
    match execution_step_instruction_type {
        // a. No Op
        instructions::definitions::ExecutionStepInstructionType::NoOp => {}
        // b. Memory Reference (reg displacement)

        // if (addr_inc == -1)   ; Pre decrement
        //     ALUoutput <- SrA' + AdL' + addr_inc
        // else
        //     ALUoutput <- SrA' + AdL'
        instructions::definitions::ExecutionStepInstructionType::MemoryRefRegDisplacement => {
            // Update SR?
            // TODO: Overflow for adding SrA' with Adl'?
            if decoded.addr_inc == -1 {
                (intermediate_registers.alu_output, _) = (decoded.sr_a_ + decoded.ad_l_)
                    .overflowing_add(sign_extend_small_offset(decoded.addr_inc as u8));
            } else {
                intermediate_registers.alu_output = decoded.sr_a_ + decoded.ad_l_;
            }
        }
        // c. Memory Reference (imm displacement)

        // if (addr_inc == -1)   ; Pre decrement
        //     ALUoutput <- imm + AdL + addr_inc
        // else
        //     ALUoutput <- imm + AdL
        instructions::definitions::ExecutionStepInstructionType::MemoryRefImmDisplacement => {
            if decoded.addr_inc == -1 {
                intermediate_registers.alu_output = (decoded.imm + decoded.ad_l_)
                    .wrapping_add(sign_extend_small_offset(decoded.addr_inc as u8));
            } else {
                intermediate_registers.alu_output = decoded.imm + decoded.ad_l_;
            }
        }
        // d. Register-Register ALU:

        // ALUoutput <- SrA' op SrB'
        // Regs[sr] <- status(SrA' op SrB', Sr)
        instructions::definitions::ExecutionStepInstructionType::RegisterRegisterAlu => {
            perform_alu_operation(
                alu_op,
                decoded.sr_a_,
                decoded.sr_b_,
                registers,
                &mut intermediate_registers,
            );
        }
        // e. Register-Immediate ALU operation:

        // ALUoutput <- Des' op imm
        // Regs[sr] <- status(Des' op imm, Sr)
        instructions::definitions::ExecutionStepInstructionType::RegisterImmediateAlu => {
            perform_alu_operation(
                alu_op,
                decoded.des_,
                decoded.imm,
                registers,
                &mut intermediate_registers,
            );
        }
        // f. Branch:

        // ALUoutput <- PC + imm
        instructions::definitions::ExecutionStepInstructionType::Branch => {
            // Update SR?
            // TODO: Overflow?
            perform_alu_operation(
                AluOp::Add,
                registers.pl,
                decoded.imm,
                registers,
                &mut intermediate_registers,
            );
        }
    }

    // 5. ====== Memory access/branch completion (MEM): ======

    let memory_access_step_instruction_type =
        decode_memory_access_step_instruction_type(&instruction, &decoded);

    // TODO: I think this works, because branch will overwrite the PC anyway, otherwise we want to advance.
    // but we might need to think about how this would work in FPGA
    registers.pl = intermediate_registers.npc;

    match memory_access_step_instruction_type {
        // a. No Op
        instructions::definitions::MemoryAccessInstructionType::NoOp => {}
        // a. Memory load
        // LMD <- Mem[AdrH | ALUOutput]
        instructions::definitions::MemoryAccessInstructionType::MemoryLoad => {
            intermediate_registers.lmd = mem
                .read_address((decoded.ad_h_, intermediate_registers.alu_output).to_full_address())
        }
        // b. Memory store
        // Mem[AdrH | ALUOutput] <- A?
        instructions::definitions::MemoryAccessInstructionType::MemoryStore => {
            // A or B?
            mem.write_address(
                (decoded.ad_h_, intermediate_registers.alu_output).to_full_address(),
                decoded.sr_b_,
            )
        }
        // c. Branch/Jump
        // if (Cond') PC <- ALUoutput
        // else      PC <- NPC
        instructions::definitions::MemoryAccessInstructionType::BranchOrJump => {
            // TODO: Long Jump
            registers.pl = intermediate_registers.alu_output;
        }
    }

    // ==== 6. Write-back cycle (WB): ====

    let write_back_step_instruction_type =
        decode_write_back_step_instruction_type(&instruction, &decoded);

    match write_back_step_instruction_type {
        instructions::definitions::WriteBackInstructionType::NoOp => {}
        // a. Memory load
        // Regs[Des] <- LMD
        instructions::definitions::WriteBackInstructionType::MemoryLoad => {
            registers[decoded.des] = intermediate_registers.lmd;
        }
        //  b. Register-Register ALU or Register-Immediate ALU:
        // Regs[Des] <- ALUoutput
        instructions::definitions::WriteBackInstructionType::AluToRegister => {
            registers[decoded.des] = intermediate_registers.alu_output;
        }
        //  c. Load Effective Address
        // Regs[DesAdL] <- ALUOutput
        // Regs[DesAdH] <- AdrH
        instructions::definitions::WriteBackInstructionType::LoadEffectiveAddress => {
            registers[decoded.des] = intermediate_registers.alu_output;
        }
    }

    if sr_bit_is_set(StatusRegisterFields::CpuHalted, registers) {
        return Err(Error::ProcessorHalted(registers.to_owned()));
    }

    // TODO: 6 -> constant ITS ALWAYS SIX BABY
    Ok((registers, 6, instruction))
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
