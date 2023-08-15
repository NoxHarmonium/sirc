use peripheral_mem::MemoryPeripheral;

use crate::{
    coprocessors::processing_unit::definitions::{Instruction, StatusRegisterUpdateSource},
    registers::Registers,
};

use super::shared::{DecodedInstruction, IntermediateRegisters, StageExecutor};

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum WriteBackInstructionType {
    NoOp,
    MemoryLoad,
    AluToRegister,
    AluStatusOnly,
    AddressWrite,
    AddressWriteLoadPostDecrement,
    AddressWriteStorePreIncrement,
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
        0x00..=0x07 => WriteBackInstructionType::AluToRegister,
        0x08..=0x0F => WriteBackInstructionType::AluStatusOnly,
        0x10..=0x11 => WriteBackInstructionType::NoOp,
        0x12..=0x13 => WriteBackInstructionType::AddressWriteStorePreIncrement,
        0x14..=0x15 => WriteBackInstructionType::MemoryLoad,
        0x16..=0x17 => WriteBackInstructionType::AddressWriteLoadPostDecrement,
        0x18..=0x1F => WriteBackInstructionType::AddressWrite,
        0x20..=0x27 => WriteBackInstructionType::AluToRegister,
        0x28..=0x2F => WriteBackInstructionType::AluStatusOnly,
        0x30..=0x37 => WriteBackInstructionType::AluToRegister,
        0x38..=0x3F => WriteBackInstructionType::AluStatusOnly,
        _ => panic!("No mapping for [{instruction:?}] to WriteBackInstructionType"),
    }
}

fn update_status_flags(
    decoded: &DecodedInstruction,
    registers: &mut Registers,
    intermediate_registers: &mut IntermediateRegisters,
) {
    // TODO: Should this be done with an instruction type?
    registers.sr = match decoded.sr_src {
        // TODO: Define assembly syntax to define this explicitly if required and make sure it is tested
        StatusRegisterUpdateSource::Alu => {
            // Do not allow updates to the privileged byte of the SR via the ALU!
            (registers.sr & 0xFF00) | (intermediate_registers.alu_status_register & 0x00FF)
        }
        StatusRegisterUpdateSource::Shift => decoded.sr_shift,
        _ => registers.sr,
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
            WriteBackInstructionType::AluStatusOnly => {
                update_status_flags(decoded, registers, intermediate_registers);
            }
            WriteBackInstructionType::AluToRegister => {
                registers[decoded.des] = intermediate_registers.alu_output;
                update_status_flags(decoded, registers, intermediate_registers);
            }
            WriteBackInstructionType::AddressWrite => {
                registers[decoded.des_ad_h] = decoded.ad_h_;
                registers[decoded.des_ad_l] = intermediate_registers.alu_output;
            }
            WriteBackInstructionType::AddressWriteLoadPostDecrement => {
                // TODO: Is there a smarter way to do this that doesn't duplicate MemoryLoad and AddressWriteStorePreIncrement branch
                // also make sure that this is ok to do in hardware
                // TODO: The order of operations matters here which probably doesn't bode well for the hardware
                // implementation. What happens if the destination register is the same as the address source register?
                // I guess the destination register should take precedence
                registers[decoded.ad_h] = decoded.ad_h_;
                registers[decoded.ad_l] = intermediate_registers.address_output;
                registers[decoded.des] = intermediate_registers.lmd;
            }
            WriteBackInstructionType::AddressWriteStorePreIncrement => {
                registers[decoded.ad_h] = decoded.ad_h_;
                registers[decoded.ad_l] = intermediate_registers.address_output;
            }
        }
    }
}
