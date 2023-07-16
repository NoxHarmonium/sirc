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
    BranchOrJumpSubroutine,
}

pub struct MemoryAccessExecutor;

// TODO: Clean up match and remove this warning
#[allow(clippy::match_same_arms)]
fn decode_memory_access_step_instruction_type(
    instruction: Instruction,
    decoded_instruction: &DecodedInstruction,
) -> MemoryAccessInstructionType {
    if !decoded_instruction.con_ {
        return MemoryAccessInstructionType::NoOp;
    }

    match num::ToPrimitive::to_u8(&instruction).unwrap() {
        0x00..=0x0F => MemoryAccessInstructionType::NoOp,
        0x10..=0x13 => MemoryAccessInstructionType::MemoryStore,
        0x14..=0x17 => MemoryAccessInstructionType::MemoryLoad,
        0x18..=0x1B => MemoryAccessInstructionType::NoOp, // LDEA/BRAN
        0x1C..=0x1F => MemoryAccessInstructionType::BranchOrJumpSubroutine, // LJSR/BRSR
        0x20..=0x3F => MemoryAccessInstructionType::NoOp,
        _ => panic!("No mapping for [{instruction:?}] to MemoryAccessInstructionType"),
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
            decode_memory_access_step_instruction_type(decoded.ins, decoded);

        // TODO: I think this works, because branch will overwrite the PC anyway, otherwise we want to advance.
        // but we might need to think about how this would work in FPGA
        registers.pl = decoded.npc_l_;

        match memory_access_step_instruction_type {
            MemoryAccessInstructionType::NoOp => {}

            MemoryAccessInstructionType::MemoryLoad => {
                intermediate_registers.lmd = mem.read_address(
                    (decoded.ad_h_, intermediate_registers.alu_output).to_full_address(),
                );
            }
            MemoryAccessInstructionType::MemoryStore => {
                mem.write_address(
                    (decoded.ad_h_, intermediate_registers.alu_output).to_full_address(),
                    decoded.sr_a_,
                );
            }

            MemoryAccessInstructionType::BranchOrJumpSubroutine => {
                // Also store next instruction in link registers so RETS can jump back to after the branch/jump
                // TODO: This should probably be in the write back stage
                registers.ll = decoded.npc_l_;
                registers.lh = decoded.npc_h_;
            }
        }
    }
}
