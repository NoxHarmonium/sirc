use peripheral_bus::device::{BusAccessType, BusAssertions, BusOperation};

use crate::{
    coprocessors::processing_unit::definitions::Instruction,
    registers::{ExceptionUnitRegisters, Registers, SegmentedAddress},
};

use super::shared::{DecodedInstruction, IntermediateRegisters, StageExecutor};

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
enum MemoryAccessInstructionType {
    NoOp,
    MemoryLoad,
    MemoryStore,
}

pub struct MemoryAccessExecutor;

// TODO: Clean up `clippy::match_same_arms` violation in `MemoryAccessExecutor`
// category=Refactoring
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
        0x18..=0x1F => MemoryAccessInstructionType::NoOp,
        0x20..=0x3F => MemoryAccessInstructionType::NoOp,
        _ => panic!("No mapping for [{instruction:?}] to MemoryAccessInstructionType"),
    }
}

impl StageExecutor for MemoryAccessExecutor {
    fn execute(
        decoded: &DecodedInstruction,
        _registers: &mut Registers,
        _: &mut ExceptionUnitRegisters,
        intermediate_registers: &mut IntermediateRegisters,
        _: BusAssertions,
    ) -> BusAssertions {
        // 5. ====== Memory access/branch completion (MEM): ======

        let memory_access_step_instruction_type =
            decode_memory_access_step_instruction_type(decoded.ins, decoded);

        match memory_access_step_instruction_type {
            MemoryAccessInstructionType::NoOp => {}

            MemoryAccessInstructionType::MemoryLoad => {
                return BusAssertions {
                    address: (decoded.ad_h_, intermediate_registers.alu_output).to_full_address(),
                    op: BusOperation::Read,
                    bus_access_strobe: true,
                    bus_access_type: BusAccessType::DataRead,
                    ..BusAssertions::default()
                };
            }
            MemoryAccessInstructionType::MemoryStore => {
                return BusAssertions {
                    address: (decoded.ad_h_, intermediate_registers.alu_output).to_full_address(),
                    data: decoded.sr_a_,
                    op: BusOperation::Write,
                    bus_access_strobe: true,
                    bus_access_type: BusAccessType::DataWrite,
                    ..BusAssertions::default()
                };
            }
        }
        BusAssertions::default()
    }
}
