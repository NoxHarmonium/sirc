use log::trace;
use peripheral_bus::device::{BusAccessType, BusAssertions};

use crate::{
    coprocessors::{
        exception_unit::definitions::Faults, processing_unit::definitions::Instruction,
    },
    raise_fault,
    registers::{
        sr_bit_is_set, ExceptionUnitRegisters, RegisterName, Registers, SegmentedAddress,
        StatusRegisterFields,
    },
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

/// Classify the instruction's intended access, even when overflow prevents it.
fn address_access_type(decoded: &DecodedInstruction) -> BusAccessType {
    match num::ToPrimitive::to_u8(&decoded.ins).unwrap() {
        0x10..=0x13 => BusAccessType::DataWrite,
        0x14..=0x17 => BusAccessType::DataRead,
        _ if decoded.des_ad_l == RegisterName::Pl.to_register_index() => {
            BusAccessType::InstructionFetch
        }
        _ => BusAccessType::None,
    }
}

impl StageExecutor for ExecutionEffectiveAddressExecutor {
    fn execute(
        decoded: &DecodedInstruction,
        registers: &mut Registers,
        eu_registers: &mut ExceptionUnitRegisters,
        intermediate_registers: &mut IntermediateRegisters,
        _bus_assertions: BusAssertions,
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
                // Auto-update changes the source pair by one, independently of
                // the displacement. Pre-decrement uses that updated low word as
                // the EA base; post-increment uses the original low word.
                let (updated_source, source_update_overflowed) = match decoded.addr_inc {
                    -1 => decoded.ad_l_.overflowing_sub(1),
                    0 => (decoded.ad_l_, false),
                    1 => decoded.ad_l_.overflowing_add(1),
                    _ => panic!("addr_inc should never not be -1, 0 or 1"),
                };
                let base = if decoded.addr_inc == -1 {
                    updated_source
                } else {
                    decoded.ad_l_
                };
                let (effective_address, effective_address_overflowed) =
                    base.overflowing_add_signed(decoded.sr_b_.cast_signed());

                trace!("Calculate offset: base: 0x{base:X} displacement: 0x{:X} EA: 0x{effective_address:X} EA wrapped: {effective_address_overflowed} source update: 0x{updated_source:X} update wrapped: {source_update_overflowed}", decoded.sr_b_);

                // Keep both wrapped results available for write-back and the
                // fault metadata, without committing either register write.
                intermediate_registers.alu_output = effective_address;
                intermediate_registers.address_output = if decoded.addr_inc == 0 {
                    effective_address
                } else {
                    updated_source
                };

                // Overflow check
                if sr_bit_is_set(StatusRegisterFields::TrapOnAddressOverflow, registers)
                    && (effective_address_overflowed || source_update_overflowed)
                {
                    // By this phase `registers.pl` has already advanced past this instruction
                    // (it happened back at decode); recover this instruction's own fetch address
                    // from `decoded` instead so the fault is genuinely retryable.
                    let fault_pc = (decoded.pc_h_, decoded.pc_l_).to_full_address();
                    // Diagnostic precedence is EA, then source update, regardless
                    // of the evaluation order needed for pre-decrement arithmetic.
                    let wrapped = if effective_address_overflowed {
                        effective_address
                    } else {
                        updated_source
                    };
                    let fault_access = BusAssertions {
                        address: (decoded.ad_h_, wrapped).to_full_address(),
                        bus_access_type: address_access_type(decoded),
                        ..BusAssertions::default()
                    };
                    raise_fault(
                        eu_registers,
                        Faults::SegmentOverflow,
                        &fault_access,
                        fault_pc,
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
