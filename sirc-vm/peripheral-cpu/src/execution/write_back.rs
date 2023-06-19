use peripheral_mem::MemoryPeripheral;

use crate::{instructions::definitions::Instruction, registers::Registers};

use super::shared::{DecodedInstruction, IntermediateRegisters, StageExecutor};

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum WriteBackInstructionType {
    NoOp,
    MemoryLoad,
    AluToRegister,
    LoadEffectiveAddress,
}

pub struct WriteBackExecutor;

fn decode_write_back_step_instruction_type(
    instruction: &Instruction,
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
        // Shifts (Immediate)
        Instruction::LogicalShiftLeftImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::LogicalShiftRightImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::ArithmeticShiftLeftImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::ArithmeticShiftRightImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::RotateLeftImmediate => WriteBackInstructionType::AluToRegister,
        Instruction::RotateRightImmediate => WriteBackInstructionType::AluToRegister,
        // Arithmetic (Register)
        Instruction::AddRegister => WriteBackInstructionType::AluToRegister,
        Instruction::AddRegisterWithCarry => WriteBackInstructionType::AluToRegister,
        Instruction::SubtractRegister => WriteBackInstructionType::AluToRegister,
        Instruction::SubtractRegisterWithCarry => WriteBackInstructionType::AluToRegister,
        // Logic (Register)
        Instruction::AndRegister => WriteBackInstructionType::AluToRegister,
        Instruction::OrRegister => WriteBackInstructionType::AluToRegister,
        Instruction::XorRegister => WriteBackInstructionType::AluToRegister,
        // Shifts (Register)
        Instruction::LogicalShiftLeftRegister => WriteBackInstructionType::AluToRegister,
        Instruction::LogicalShiftRightRegister => WriteBackInstructionType::AluToRegister,
        Instruction::ArithmeticShiftLeftRegister => WriteBackInstructionType::AluToRegister,
        Instruction::ArithmeticShiftRightRegister => WriteBackInstructionType::AluToRegister,
        Instruction::RotateLeftRegister => WriteBackInstructionType::AluToRegister,
        Instruction::RotateRightRegister => WriteBackInstructionType::AluToRegister,

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
            decode_write_back_step_instruction_type(&decoded.ins, decoded);

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
