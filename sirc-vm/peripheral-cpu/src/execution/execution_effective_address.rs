use peripheral_mem::MemoryPeripheral;

use crate::{
    instructions::definitions::Instruction, microcode::address::sign_extend_small_offset,
    registers::Registers,
};

use super::{
    alu::{perform_alu_operation, AluOp},
    shared::{DecodedInstruction, IntermediateRegisters, StageExecutor},
};

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum ExecutionStepInstructionType {
    NoOp,
    MemoryRefRegDisplacement,
    MemoryRefImmDisplacement,
    RegisterRegisterAlu,
    RegisterImmediateAlu,
    Branch,
}

pub struct ExecutionEffectiveAddressExecutor;

fn decode_execution_step_instruction_type(
    instruction: &Instruction,
    decoded_instruction: &DecodedInstruction,
) -> ExecutionStepInstructionType {
    if !decoded_instruction.con_ {
        return ExecutionStepInstructionType::NoOp;
    }

    match instruction {
        // Arithmetic (Immediate)
        Instruction::AddImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        Instruction::AddImmediateWithCarry => ExecutionStepInstructionType::RegisterImmediateAlu,
        Instruction::SubtractImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        Instruction::SubtractImmediateWithCarry => {
            ExecutionStepInstructionType::RegisterImmediateAlu
        }
        // Logic (Immediate)
        Instruction::AndImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        Instruction::OrImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        Instruction::XorImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        // Shifts (Immediate)
        Instruction::LogicalShiftLeftImmediate => {
            ExecutionStepInstructionType::RegisterImmediateAlu
        }
        Instruction::LogicalShiftRightImmediate => {
            ExecutionStepInstructionType::RegisterImmediateAlu
        }
        Instruction::ArithmeticShiftLeftImmediate => {
            ExecutionStepInstructionType::RegisterImmediateAlu
        }
        Instruction::ArithmeticShiftRightImmediate => {
            ExecutionStepInstructionType::RegisterImmediateAlu
        }
        Instruction::RotateLeftImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        Instruction::RotateRightImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        // Comparison (Immediate)
        Instruction::CompareImmediate => ExecutionStepInstructionType::RegisterImmediateAlu,
        // Flow Control (Immediate)
        Instruction::ShortJumpImmediate => ExecutionStepInstructionType::Branch,
        Instruction::ShortJumpToSubroutineImmediate => ExecutionStepInstructionType::Branch,

        // Arithmetic (Register)
        Instruction::AddRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::AddRegisterWithCarry => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::SubtractRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::SubtractRegisterWithCarry => ExecutionStepInstructionType::RegisterRegisterAlu,
        // Logic (Register)
        Instruction::AndRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::OrRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::XorRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        // Shifts (Register)
        Instruction::LogicalShiftLeftRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::LogicalShiftRightRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::ArithmeticShiftLeftRegister => {
            ExecutionStepInstructionType::RegisterRegisterAlu
        }
        Instruction::ArithmeticShiftRightRegister => {
            ExecutionStepInstructionType::RegisterRegisterAlu
        }
        Instruction::RotateLeftRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::RotateRightRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        // Comparison (Register)
        Instruction::CompareRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        // NOOP (Register)
        Instruction::NoOperation => ExecutionStepInstructionType::NoOp,

        // Flow Control (Immediate)
        Instruction::BranchImmediate => ExecutionStepInstructionType::Branch,
        Instruction::BranchToSubroutineImmediate => ExecutionStepInstructionType::Branch,
        Instruction::WaitForException => ExecutionStepInstructionType::NoOp, // Handled by Exception Unit
        Instruction::ReturnFromException => ExecutionStepInstructionType::NoOp, // Handled by Exception Unit
        Instruction::Exception => ExecutionStepInstructionType::NoOp, // Handled by Exception Unit

        // Data Access
        Instruction::LoadRegisterFromImmediate => {
            ExecutionStepInstructionType::RegisterImmediateAlu
        }
        Instruction::LoadRegisterFromRegister => ExecutionStepInstructionType::RegisterRegisterAlu,
        Instruction::LoadRegisterFromIndirectImmediate => {
            ExecutionStepInstructionType::MemoryRefImmDisplacement
        }
        Instruction::LoadRegisterFromIndirectRegister => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::LoadRegisterFromIndirectRegisterPostIncrement => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::StoreRegisterToIndirectImmediate => {
            ExecutionStepInstructionType::MemoryRefImmDisplacement
        }
        Instruction::StoreRegisterToIndirectRegister => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::StoreRegisterToIndirectRegisterPreDecrement => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }

        Instruction::LongJumpWithImmediateDisplacement => {
            ExecutionStepInstructionType::MemoryRefImmDisplacement
        }
        Instruction::LongJumpToSubroutineWithRegisterDisplacement => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::ReturnFromSubroutine => ExecutionStepInstructionType::MemoryRefImmDisplacement, // Encoded as zero offset from link register
        Instruction::LoadEffectiveAddressFromIndirectImmediate => {
            ExecutionStepInstructionType::MemoryRefImmDisplacement
        }
        Instruction::LoadEffectiveAddressFromIndirectRegister => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::LongJumpWithRegisterDisplacement => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::LongJumpToSubroutineWithImmediateDisplacement => {
            ExecutionStepInstructionType::MemoryRefImmDisplacement
        }
    }
}

impl StageExecutor for ExecutionEffectiveAddressExecutor {
    fn execute(
        decoded: &DecodedInstruction,
        registers: &mut Registers,
        intermediate_registers: &mut IntermediateRegisters,
        _: &MemoryPeripheral,
    ) {
        // TODO: Replace unwrap with something better
        let alu_code = num::ToPrimitive::to_u8(&decoded.ins).unwrap() & 0x0F;
        // TODO: Should this be unwrap? - clean this up
        let alu_op: AluOp = num::FromPrimitive::from_u8(alu_code).unwrap();

        let execution_step_instruction_type =
            decode_execution_step_instruction_type(&decoded.ins, decoded);

        // 4. ====== Execution (EX) ======
        match execution_step_instruction_type {
            // a. No Op
            ExecutionStepInstructionType::NoOp => {}
            // b. Memory Reference (reg displacement)

            // if (addr_inc == -1)   ; Pre decrement
            //     ALUoutput <- SrA' + AdL' + addr_inc
            // else
            //     ALUoutput <- SrA' + AdL'
            ExecutionStepInstructionType::MemoryRefRegDisplacement => {
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
            ExecutionStepInstructionType::MemoryRefImmDisplacement => {
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
            ExecutionStepInstructionType::RegisterRegisterAlu => {
                perform_alu_operation(
                    alu_op,
                    // TODO: Is this feasible in hardware?
                    if decoded.ins == Instruction::LoadRegisterFromImmediate {
                        0x0
                    } else {
                        decoded.sr_a_
                    },
                    decoded.sr_b_,
                    registers,
                    intermediate_registers,
                );
            }
            // e. Register-Immediate ALU operation:

            // ALUoutput <- Des' op imm
            // Regs[sr] <- status(Des' op imm, Sr)
            ExecutionStepInstructionType::RegisterImmediateAlu => {
                // TODO: Should the status register only be updated on writeback?
                perform_alu_operation(
                    alu_op,
                    // TODO: Is this feasible in hardware?
                    if decoded.ins == Instruction::LoadRegisterFromImmediate {
                        0x0
                    } else {
                        decoded.des_
                    },
                    decoded.imm,
                    registers,
                    intermediate_registers,
                );
            }

            // f. Branch:

            // ALUoutput <- PC + imm
            ExecutionStepInstructionType::Branch => {
                // Update SR?
                // TODO: Overflow?
                perform_alu_operation(
                    AluOp::Add,
                    registers.pl,
                    decoded.imm,
                    registers,
                    intermediate_registers,
                );
            }
        }
    }
}
