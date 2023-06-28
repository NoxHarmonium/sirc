use peripheral_mem::MemoryPeripheral;

use crate::{
    instructions::definitions::{Instruction, StatusRegisterUpdateSource},
    registers::Registers,
};

use super::shared::{DecodedInstruction, IntermediateRegisters, StageExecutor};

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum WriteBackInstructionType {
    NoOp,
    MemoryLoad,
    AluToRegister,
    LoadEffectiveAddress,
}

pub struct WriteBackExecutor;

// TODO: Clean up this match and remove this warning
#[allow(clippy::match_same_arms)]
fn decode_write_back_step_instruction_type(
    instruction: Instruction,
    decoded_instruction: &DecodedInstruction,
) -> WriteBackInstructionType {
    if !decoded_instruction.con_ {
        return WriteBackInstructionType::NoOp;
    }

    match instruction {
        // Arithmetic (Immediate)
        Instruction::AddImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::AddImmediateWithCarry => WriteBackInstructionType::AluToRegister,
        Instruction::SubtractImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::SubtractImmediateWithCarry => WriteBackInstructionType::AluToRegister,
        // Logic (Immediate)
        Instruction::AndImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::OrImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::XorImmediate => WriteBackInstructionType::AluToRegister,
        // Compare (Immediate)
        Instruction::CompareImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::TestAndImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::TestXorImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::ShiftImmediate => WriteBackInstructionType::AluToRegister,

        // Arithmetic (SHORT Immediate)
        Instruction::AddShortImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::AddShortImmediateWithCarry => WriteBackInstructionType::AluToRegister,
        Instruction::SubtractShortImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::SubtractShortImmediateWithCarry => WriteBackInstructionType::AluToRegister,
        // Logic (SHORT Immediate)
        Instruction::AndShortImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::OrShortImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::XorShortImmediate => WriteBackInstructionType::AluToRegister,
        // Compare (SHORT Immediate)
        Instruction::CompareShortImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::TestAndShortImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::TestXorShortImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::ShiftShortImmediate => WriteBackInstructionType::AluToRegister,

        // Arithmetic (Register)
        Instruction::AddRegister => WriteBackInstructionType::AluToRegister,
        Instruction::AddRegisterWithCarry => WriteBackInstructionType::AluToRegister,
        Instruction::SubtractRegister => WriteBackInstructionType::AluToRegister,
        Instruction::SubtractRegisterWithCarry => WriteBackInstructionType::AluToRegister,
        // Logic (Register)
        Instruction::AndRegister => WriteBackInstructionType::AluToRegister,
        Instruction::OrRegister => WriteBackInstructionType::AluToRegister,
        Instruction::XorRegister => WriteBackInstructionType::AluToRegister,
        // Compare (Register)
        Instruction::CompareRegister => WriteBackInstructionType::AluToRegister,
        Instruction::TestAndRegister => WriteBackInstructionType::AluToRegister,
        Instruction::TestXorRegister => WriteBackInstructionType::AluToRegister,
        Instruction::ShiftRegister => WriteBackInstructionType::AluToRegister,

        // Data Access
        Instruction::LoadRegisterFromImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::LoadRegisterFromRegister => WriteBackInstructionType::AluToRegister,
        Instruction::LoadRegisterFromIndirectImmediate => WriteBackInstructionType::MemoryLoad,
        Instruction::LoadRegisterFromIndirectRegister => WriteBackInstructionType::MemoryLoad,
        Instruction::LoadRegisterFromIndirectRegisterPostIncrement => {
            WriteBackInstructionType::MemoryLoad
        }

        Instruction::LoadEffectiveAddressFromIndirectImmediate => {
            WriteBackInstructionType::LoadEffectiveAddress
        }
        Instruction::LoadEffectiveAddressFromIndirectRegister => {
            WriteBackInstructionType::LoadEffectiveAddress
        }
        _ => WriteBackInstructionType::NoOp,
    }
}

impl StageExecutor for WriteBackExecutor {
    fn execute(
        decoded: &DecodedInstruction,
        registers: &mut Registers,
        intermediate_registers: &mut IntermediateRegisters,
        _: &MemoryPeripheral,
    ) {
        // ==== 6. Write-back cycle (WB): ====

        let write_back_step_instruction_type =
            decode_write_back_step_instruction_type(decoded.ins, decoded);

        match write_back_step_instruction_type {
            WriteBackInstructionType::NoOp => {}
            // a. Memory load
            // Regs[Des] <- LMD
            WriteBackInstructionType::MemoryLoad => {
                registers[decoded.des] = intermediate_registers.lmd;
            }
            //  b. Register-Register ALU or Register-Immediate ALU:
            // Regs[Des] <- ALUoutput
            WriteBackInstructionType::AluToRegister => {
                registers[decoded.des] = intermediate_registers.alu_output;
                // TODO: Should this be done with an instruction type?
                registers.sr = match decoded.sr_src {
                    StatusRegisterUpdateSource::Alu => {
                        // Do not allow updates to the privileged byte of the SR via the ALU!
                        (registers.sr & 0xFF00)
                            | (intermediate_registers.alu_status_register & 0x00FF)
                    }
                    StatusRegisterUpdateSource::Shift => decoded.sr_shift,
                    _ => registers.sr,
                }
            }
            //  c. Load Effective Address
            // Regs[DesAdL] <- ALUOutput
            // Regs[DesAdH] <- AdrH
            WriteBackInstructionType::LoadEffectiveAddress => {
                registers[decoded.des] = intermediate_registers.alu_output;
            }
        }
    }
}
