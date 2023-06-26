use peripheral_mem::MemoryPeripheral;

use crate::{
    instructions::definitions::Instruction,
    registers::{Registers, SegmentedAddress},
};

use super::shared::{DecodedInstruction, IntermediateRegisters, StageExecutor};

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum MemoryAccessInstructionType {
    NoOp,
    MemoryLoad,
    MemoryStore,
    BranchOrJump,
}

pub struct MemoryAccessExecutor;

fn decode_memory_access_step_instruction_type(
    instruction: &Instruction,
    decoded_instruction: &DecodedInstruction,
) -> MemoryAccessInstructionType {
    if !decoded_instruction.con_ {
        return MemoryAccessInstructionType::NoOp;
    }

    match instruction {
        // Flow Control (Immediate)
        Instruction::ShortJumpImmediate => MemoryAccessInstructionType::BranchOrJump,
        Instruction::ShortJumpToSubroutineImmediate => MemoryAccessInstructionType::BranchOrJump,
        // Flow Control (Immediate)
        Instruction::BranchImmediate => MemoryAccessInstructionType::BranchOrJump,
        Instruction::BranchToSubroutineImmediate => MemoryAccessInstructionType::BranchOrJump,

        // Data Access
        Instruction::LoadRegisterFromIndirectImmediate => MemoryAccessInstructionType::MemoryLoad,
        Instruction::LoadRegisterFromIndirectRegister => MemoryAccessInstructionType::MemoryLoad,
        Instruction::LoadRegisterFromIndirectRegisterPostIncrement => {
            MemoryAccessInstructionType::MemoryLoad
        }
        Instruction::StoreRegisterToIndirectImmediate => MemoryAccessInstructionType::MemoryStore,
        Instruction::StoreRegisterToIndirectRegister => MemoryAccessInstructionType::MemoryStore,
        Instruction::StoreRegisterToIndirectRegisterPreDecrement => {
            MemoryAccessInstructionType::MemoryStore
        }

        Instruction::LongJumpWithImmediateDisplacement => MemoryAccessInstructionType::BranchOrJump,
        Instruction::LongJumpToSubroutineWithRegisterDisplacement => {
            MemoryAccessInstructionType::BranchOrJump
        }
        Instruction::LongJumpWithRegisterDisplacement => MemoryAccessInstructionType::BranchOrJump,
        Instruction::LongJumpToSubroutineWithImmediateDisplacement => {
            MemoryAccessInstructionType::BranchOrJump
        }

        // Flow Control (Address Register Direct)
        _ => MemoryAccessInstructionType::NoOp,
    }
}

impl StageExecutor for MemoryAccessExecutor {
    fn execute(
        decoded: &DecodedInstruction,
        registers: &mut Registers,
        intermediate_registers: &mut IntermediateRegisters,
        mem: &MemoryPeripheral,
    ) {
        // 5. ====== Memory access/branch completion (MEM): ======

        let memory_access_step_instruction_type =
            decode_memory_access_step_instruction_type(&decoded.ins, decoded);

        // TODO: I think this works, because branch will overwrite the PC anyway, otherwise we want to advance.
        // but we might need to think about how this would work in FPGA
        registers.pl = intermediate_registers.npc;

        match memory_access_step_instruction_type {
            // a. No Op
            MemoryAccessInstructionType::NoOp => {}
            // a. Memory load
            // LMD <- Mem[AdrH | ALUOutput]
            MemoryAccessInstructionType::MemoryLoad => {
                intermediate_registers.lmd = mem.read_address(
                    (decoded.ad_h_, intermediate_registers.alu_output).to_full_address(),
                )
            }
            // b. Memory store
            // Mem[AdrH | ALUOutput] <- A?
            MemoryAccessInstructionType::MemoryStore => {
                // A or B?
                mem.write_address(
                    (decoded.ad_h_, intermediate_registers.alu_output).to_full_address(),
                    decoded.sr_a_,
                )
            }
            // c. Branch/Jump
            // if (Cond') PC <- ALUoutput
            // else      PC <- NPC
            MemoryAccessInstructionType::BranchOrJump => {
                // TODO: Long Jump
                registers.pl = intermediate_registers.alu_output;
            }
        }
    }
}
