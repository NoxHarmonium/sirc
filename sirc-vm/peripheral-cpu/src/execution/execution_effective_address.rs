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
    MemoryRefDisplacement,
    Alu,
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

    match num::ToPrimitive::to_u8(&instruction).unwrap() {
        0x00..=0x0F => ExecutionStepInstructionType::Alu,
        0x10..=0x1F => ExecutionStepInstructionType::MemoryRefDisplacement,
        0x20..=0x3F => ExecutionStepInstructionType::Alu,
        _ => panic!("No mapping for [{instruction:?}] to ExecutionStepInstructionType"),
    }
}

#[allow(clippy::cast_sign_loss)]
impl StageExecutor for ExecutionEffectiveAddressExecutor {
    fn execute(
        decoded: &DecodedInstruction,
        _: &mut Registers,
        intermediate_registers: &mut IntermediateRegisters,
        _: &MemoryPeripheral,
    ) {
        // TODO: Replace unwrap with something better
        let alu_code = num::ToPrimitive::to_u8(&decoded.ins).unwrap() & 0x0F;
        // TODO: Should this be unwrap? - clean this up - make 0x7 a constant or put in function
        // Only the first 3 bits are used to determine the ALU operation, the fourth bit determines whether the result is stored or not
        let alu_op: AluOp = num::FromPrimitive::from_u8(alu_code & 0x7).unwrap();
        // TODO : Clean this up - make 0x8 a constant or put in function
        // bit 3 determines whether the ALU output is used or not
        // e.g. CMPI is a SUBI without storing ALU output
        let simulate = alu_code & 0x8 == 0x8;

        let execution_step_instruction_type =
            decode_execution_step_instruction_type(decoded.ins, decoded);

        // 4. ====== Execution (EX) ======
        match execution_step_instruction_type {
            ExecutionStepInstructionType::NoOp => {}

            ExecutionStepInstructionType::MemoryRefDisplacement => {
                let (displaced, _) = decoded.ad_l_.overflowing_add(decoded.sr_b_);

                (intermediate_registers.alu_output, _) =
                    displaced.overflowing_add(sign_extend_small_offset(decoded.addr_inc as u8));
            }

            ExecutionStepInstructionType::Alu => {
                perform_alu_operation(
                    &alu_op,
                    simulate,
                    decoded.sr_a_,
                    decoded.sr_b_,
                    decoded.sr,
                    intermediate_registers,
                );
            }
        }
    }
}
