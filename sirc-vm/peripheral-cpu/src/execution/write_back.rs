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
    AddressWrite,
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

    match num::ToPrimitive::to_u8(&instruction).unwrap() {
        0x00..=0x0F => WriteBackInstructionType::AluToRegister,
        0x10..=0x12 => WriteBackInstructionType::NoOp,
        0x13..=0x15 => WriteBackInstructionType::MemoryLoad,
        0x16..=0x1F => WriteBackInstructionType::AddressWrite,
        0x20..=0x3C => WriteBackInstructionType::AluToRegister,
        0x3D..=0x3F => WriteBackInstructionType::NoOp,

        _ => panic!("No mapping for [{instruction:?}] to WriteBackInstructionType"),
    }
}

impl StageExecutor for WriteBackExecutor {
    fn execute(
        decoded: &DecodedInstruction,
        registers: &mut Registers,
        intermediate_registers: &mut IntermediateRegisters,
        _: &MemoryPeripheral,
    ) {
        let write_back_step_instruction_type =
            decode_write_back_step_instruction_type(decoded.ins, decoded);

        match write_back_step_instruction_type {
            WriteBackInstructionType::NoOp => {}
            WriteBackInstructionType::MemoryLoad => {
                registers[decoded.des] = intermediate_registers.lmd;
            }
            WriteBackInstructionType::AluToRegister => {
                registers[decoded.des] = intermediate_registers.alu_output;
                // TODO: Should this be done with an instruction type?
                registers.sr = match decoded.sr_src {
                    // TODO: Define assembly syntax to define this explicitly if required and make sure it is tested
                    StatusRegisterUpdateSource::Alu => {
                        // Do not allow updates to the privileged byte of the SR via the ALU!
                        (registers.sr & 0xFF00)
                            | (intermediate_registers.alu_status_register & 0x00FF)
                    }
                    StatusRegisterUpdateSource::Shift => decoded.sr_shift,
                    _ => registers.sr,
                }
            }
            WriteBackInstructionType::AddressWrite => {
                registers[decoded.des_ad_h] = decoded.ad_h_;
                registers[decoded.des_ad_l] = intermediate_registers.alu_output;
            }
        }
    }
}
