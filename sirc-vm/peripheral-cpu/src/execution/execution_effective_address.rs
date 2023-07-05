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
    Alu,
    Branch,
}

pub struct ExecutionEffectiveAddressExecutor;

// TODO: Clean up this match and remove this exclusion
#[allow(clippy::match_same_arms)]
fn decode_execution_step_instruction_type(
    instruction: Instruction,
    decoded_instruction: &DecodedInstruction,
) -> ExecutionStepInstructionType {
    if !decoded_instruction.con_ {
        return ExecutionStepInstructionType::NoOp;
    }

    match instruction {
        Instruction::AddImmediate => ExecutionStepInstructionType::Alu,
        Instruction::AddImmediateWithCarry => ExecutionStepInstructionType::Alu,
        Instruction::SubtractImmediate => ExecutionStepInstructionType::Alu,
        Instruction::SubtractImmediateWithCarry => ExecutionStepInstructionType::Alu,
        Instruction::AndImmediate => ExecutionStepInstructionType::Alu,
        Instruction::OrImmediate => ExecutionStepInstructionType::Alu,
        Instruction::XorImmediate => ExecutionStepInstructionType::Alu,
        Instruction::CompareImmediate => ExecutionStepInstructionType::Alu,
        Instruction::TestAndImmediate => ExecutionStepInstructionType::Alu,
        Instruction::TestXorImmediate => ExecutionStepInstructionType::Alu,
        Instruction::ShiftImmediate => ExecutionStepInstructionType::Alu,
        Instruction::BranchImmediate => ExecutionStepInstructionType::Branch,
        Instruction::BranchToSubroutineImmediate => ExecutionStepInstructionType::Branch,
        Instruction::ShortJumpImmediate => ExecutionStepInstructionType::Alu,
        Instruction::ShortJumpToSubroutineImmediate => ExecutionStepInstructionType::Alu,
        Instruction::Exception => ExecutionStepInstructionType::NoOp,
        Instruction::LoadEffectiveAddressFromIndirectImmediate => {
            ExecutionStepInstructionType::MemoryRefImmDisplacement
        }
        Instruction::LoadEffectiveAddressFromIndirectRegister => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::LongJumpWithImmediateDisplacement => {
            ExecutionStepInstructionType::MemoryRefImmDisplacement
        }
        Instruction::LongJumpWithRegisterDisplacement => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::LongJumpToSubroutineWithImmediateDisplacement => {
            ExecutionStepInstructionType::MemoryRefImmDisplacement
        }
        Instruction::LongJumpToSubroutineWithRegisterDisplacement => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::LoadRegisterFromImmediate => {
            ExecutionStepInstructionType::MemoryRefImmDisplacement
        }
        Instruction::LoadRegisterFromRegister => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::LoadRegisterFromIndirectImmediate => {
            ExecutionStepInstructionType::MemoryRefImmDisplacement
        }
        Instruction::LoadRegisterFromIndirectRegister => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::StoreRegisterToIndirectImmediate => {
            ExecutionStepInstructionType::MemoryRefImmDisplacement
        }
        Instruction::StoreRegisterToIndirectRegister => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::LoadRegisterFromIndirectRegisterPostIncrement => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::StoreRegisterToIndirectRegisterPreDecrement => {
            ExecutionStepInstructionType::MemoryRefRegDisplacement
        }
        Instruction::AddShortImmediate => ExecutionStepInstructionType::Alu,
        Instruction::AddShortImmediateWithCarry => ExecutionStepInstructionType::Alu,
        Instruction::SubtractShortImmediate => ExecutionStepInstructionType::Alu,
        Instruction::SubtractShortImmediateWithCarry => ExecutionStepInstructionType::Alu,
        Instruction::AndShortImmediate => ExecutionStepInstructionType::Alu,
        Instruction::OrShortImmediate => ExecutionStepInstructionType::Alu,
        Instruction::XorShortImmediate => ExecutionStepInstructionType::Alu,
        Instruction::CompareShortImmediate => ExecutionStepInstructionType::Alu,
        Instruction::TestAndShortImmediate => ExecutionStepInstructionType::Alu,
        Instruction::TestXorShortImmediate => ExecutionStepInstructionType::Alu,
        Instruction::ShiftShortImmediate => ExecutionStepInstructionType::Alu,
        Instruction::BranchShortImmediate => ExecutionStepInstructionType::Branch,
        Instruction::BranchToSubroutineShortImmediate => ExecutionStepInstructionType::Branch,
        Instruction::ShortJumpShortImmediate => ExecutionStepInstructionType::Alu,
        Instruction::ShortJumpToSubroutineShortImmediate => ExecutionStepInstructionType::Alu,
        Instruction::ExceptionShort => ExecutionStepInstructionType::NoOp,
        Instruction::AddRegister => ExecutionStepInstructionType::Alu,
        Instruction::AddRegisterWithCarry => ExecutionStepInstructionType::Alu,
        Instruction::SubtractRegister => ExecutionStepInstructionType::Alu,
        Instruction::SubtractRegisterWithCarry => ExecutionStepInstructionType::Alu,
        Instruction::AndRegister => ExecutionStepInstructionType::Alu,
        Instruction::OrRegister => ExecutionStepInstructionType::Alu,
        Instruction::XorRegister => ExecutionStepInstructionType::Alu,
        Instruction::CompareRegister => ExecutionStepInstructionType::Alu,
        Instruction::TestAndRegister => ExecutionStepInstructionType::Alu,
        Instruction::TestXorRegister => ExecutionStepInstructionType::Alu,
        Instruction::ShiftRegister => ExecutionStepInstructionType::Alu,
        Instruction::ReturnFromSubroutine => ExecutionStepInstructionType::Alu,
        Instruction::NoOperation => ExecutionStepInstructionType::NoOp,
        Instruction::WaitForException => ExecutionStepInstructionType::NoOp,
        Instruction::ReturnFromException => ExecutionStepInstructionType::NoOp,
    }
}

#[allow(clippy::cast_sign_loss)]
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
            decode_execution_step_instruction_type(decoded.ins, decoded);

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
                    intermediate_registers.alu_output = (decoded.sr_b_ + decoded.ad_l_)
                        .wrapping_add(sign_extend_small_offset(decoded.addr_inc as u8));
                } else {
                    intermediate_registers.alu_output = decoded.sr_b_ + decoded.ad_l_;
                }
            }
            // d. Register-Register ALU:

            // ALUoutput <- SrA' op SrB'
            // Regs[sr] <- status(SrA' op SrB', Sr)
            ExecutionStepInstructionType::Alu => {
                perform_alu_operation(
                    &alu_op,
                    // TODO: Is this feasible in hardware?
                    // TODO: Why did I do this again?
                    // TODO: Wait a minute is this the same thing we do for the SHFT instruction?
                    if decoded.ins == Instruction::LoadRegisterFromImmediate {
                        0x0
                    } else {
                        decoded.sr_a_
                    },
                    decoded.sr_b_,
                    decoded.sr,
                    intermediate_registers,
                );
            }

            // g. Branch:

            // ALUoutput <- PC + imm
            ExecutionStepInstructionType::Branch => {
                // Update SR?
                // TODO: Overflow?
                perform_alu_operation(
                    &AluOp::Add,
                    registers.pl,
                    decoded.sr_b_,
                    decoded.sr,
                    intermediate_registers,
                );
            }
        }
    }
}
