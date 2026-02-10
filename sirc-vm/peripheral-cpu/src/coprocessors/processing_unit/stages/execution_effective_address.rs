use log::trace;
use peripheral_bus::device::BusAssertions;

use crate::{
    coprocessors::{
        exception_unit::definitions::Faults, processing_unit::definitions::Instruction,
        shared::ExecutionPhase,
    },
    raise_fault,
    registers::{ExceptionUnitRegisters, Registers, StatusRegisterFields, sr_bit_is_set},
};

use super::{
    alu::{AluOp, perform_alu_operation},
    shared::{DecodedInstruction, IntermediateRegisters, StageExecutor},
};

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum ExecutionStepInstructionType {
    NoOp,
    MemoryRefDisplacement,
    Alu,
}

pub struct ExecutionEffectiveAddressExecutor;

// TODO: Clean up `clippy::match_same_arms` violation in `ExecutionEffectiveAddressExecutor`
// category=Refactoring
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
        registers: &mut Registers,
        eu_registers: &mut ExceptionUnitRegisters,
        intermediate_registers: &mut IntermediateRegisters,
        bus_assertions: BusAssertions,
    ) -> BusAssertions {
        // TODO: Clean up `ExecutionEffectiveAddressExecutor`
        // category=Refactoring
        // These refer to below
        // - Could this decoding be done ahead of time and stored in struct?
        // - Replace unwrap with better error handling
        // - Make 0x7 a constant or put in function
        // - Make 0x8 a constant or put in function
        let alu_code = num::ToPrimitive::to_u8(&decoded.ins).unwrap() & 0x0F;
        // Only the first 3 bits are used to determine the ALU operation, the fourth bit determines whether the result is stored or not
        let alu_op: AluOp = num::FromPrimitive::from_u8(alu_code & 0x7).unwrap();
        // bit 3 determines whether the ALU output is used or not
        // e.g. CMPI is a SUBI without storing ALU output
        // 0xF is special (co processor call) which needs the value to be written
        let simulate = alu_code != 0xF && alu_code & 0x8 == 0x8;

        let execution_step_instruction_type =
            decode_execution_step_instruction_type(decoded.ins, decoded);

        // 4. ====== Execution (EX) ======
        match execution_step_instruction_type {
            ExecutionStepInstructionType::NoOp => {}

            ExecutionStepInstructionType::MemoryRefDisplacement => {
                let (displaced, displacement_overflowed) =
                    decoded.ad_l_.overflowing_add(decoded.sr_b_);
                // This is the original address with the pre/post increment/decrement applied (no offset)
                // we need this value to write back to the source address register to do the inc/dec
                let (incremented_src, _) = decoded.ad_l_.overflowing_add(decoded.addr_inc as u16);
                let (incremented_displacement, displacement_overflowed_after_inc) =
                    displaced.overflowing_add(decoded.addr_inc as u16);

                trace!(
                    "Calculate offset: ad_l_ 0x{:X} sr_b_: 0x{:X} displaced: 0x{displaced:X} displacement_overflowed: {displacement_overflowed}",
                    decoded.ad_l_, decoded.sr_b_
                );

                match decoded.addr_inc {
                    -1 => {
                        // :: Pre-decrement ::
                        // alu_output drives the memory store so it needs to PRE decremented here
                        intermediate_registers.alu_output = incremented_displacement;
                        intermediate_registers.address_output = incremented_src;
                    }
                    0 => {
                        // :: Regular ::
                        intermediate_registers.alu_output = displaced;
                        // If we aren't doing a pre decrement/post increment,
                        // the address we want to write is just the calculated address
                        intermediate_registers.address_output = displaced;
                    }
                    1 => {
                        // :: Post-increment ::
                        intermediate_registers.alu_output = displaced;
                        intermediate_registers.address_output = incremented_src;
                    }
                    _ => panic!("addr_inc should never not be -1, 0 or 1"),
                }

                // Overflow check
                if sr_bit_is_set(StatusRegisterFields::TrapOnAddressOverflow, registers)
                    && (displacement_overflowed || displacement_overflowed_after_inc)
                {
                    eu_registers.pending_fault = raise_fault(
                        eu_registers,
                        Faults::SegmentOverflow,
                        ExecutionPhase::ExecutionEffectiveAddressExecutor,
                        &bus_assertions,
                    );
                }
            }

            ExecutionStepInstructionType::Alu => {
                perform_alu_operation(
                    &alu_op,
                    simulate,
                    decoded.sr_a_,
                    decoded.sr_b_,
                    registers.sr,
                    intermediate_registers,
                );
            }
        }

        BusAssertions::default()
    }
}
