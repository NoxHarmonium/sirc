use peripheral_bus::device::BusAssertions;

use crate::{
    coprocessors::processing_unit::{
        definitions::{Instruction, ShiftOperand, StatusRegisterUpdateSource},
        stages::{alu::perform_shift, shared::ShiftParameters},
    },
    registers::{
        ExceptionUnitRegisters, RegisterName, Registers, SR_PRIVILEGED_MASK, SR_REDACTION_MASK,
        StatusRegisterFields, sr_bit_is_set,
    },
};

use super::shared::{DecodedInstruction, IntermediateRegisters, StageExecutor};

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
enum WriteBackInstructionType {
    NoOp,
    MemoryLoad,
    AluToRegister,
    AluStatusOnly,
    AddressWrite,
    AddressWriteLoadPostDecrement,
    AddressWriteStorePreIncrement,
    CoprocessorCall,
}

pub struct WriteBackExecutor;

fn get_register_value(registers: &Registers, index: u8) -> u16 {
    let should_redact_sr = sr_bit_is_set(StatusRegisterFields::ProtectedMode, registers);
    let sr_register_index = RegisterName::Sr as u8;
    if should_redact_sr && index == sr_register_index {
        registers[index] & SR_REDACTION_MASK
    } else {
        registers[index]
    }
}

fn set_register_value(registers: &mut Registers, index: u8, value: u16) {
    let should_redact_sr = sr_bit_is_set(StatusRegisterFields::ProtectedMode, registers);
    let sr_register_index = RegisterName::Sr as u8;
    if index == sr_register_index {
        // Always preserve the ExceptionActive bit (bit 13) - only the EU can modify it
        let exception_active_mask = StatusRegisterFields::ExceptionActive as u16;
        let exception_active_preserved = registers[index] & exception_active_mask;

        if should_redact_sr {
            // In protected mode: preserve privileged byte, but allow new value in lower byte
            registers[index] =
                (registers[index] & SR_PRIVILEGED_MASK) | (value & SR_REDACTION_MASK);
        } else {
            // In supervisor mode: allow new value everywhere except ExceptionActive bit
            registers[index] = (value & !exception_active_mask) | exception_active_preserved;
        }
    } else {
        registers[index] = value;
    }
}

fn do_shift(
    registers: &Registers,
    sr_a_before_shift: u16,
    shift_params: &ShiftParameters,
) -> (u16, u16) {
    let ShiftParameters {
        shift_count,
        shift_operand,
        shift_type,
    } = *shift_params;
    match shift_operand {
        ShiftOperand::Immediate => {
            perform_shift(sr_a_before_shift, shift_type, u16::from(shift_count))
        }
        ShiftOperand::Register => {
            let dereferenced_shift_count = get_register_value(registers, shift_count);
            perform_shift(sr_a_before_shift, shift_type, dereferenced_shift_count)
        }
    }
}

// TODO: Clean up `clippy::match_same_arms` violation in `WriteBackExecutor`
// category=Refactoring
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
        0x08..=0x0E => WriteBackInstructionType::AluStatusOnly,
        0x0F => WriteBackInstructionType::CoprocessorCall,
        0x10..=0x11 => WriteBackInstructionType::NoOp,
        0x12..=0x13 => WriteBackInstructionType::AddressWriteStorePreIncrement,
        0x14..=0x15 => WriteBackInstructionType::MemoryLoad,
        0x16..=0x17 => WriteBackInstructionType::AddressWriteLoadPostDecrement,
        0x18..=0x1F => WriteBackInstructionType::AddressWrite,
        0x20..=0x27 => WriteBackInstructionType::AluToRegister,
        0x28..=0x2E => WriteBackInstructionType::AluStatusOnly,
        0x2F => WriteBackInstructionType::CoprocessorCall,
        0x30..=0x37 => WriteBackInstructionType::AluToRegister,
        0x38..=0x3E => WriteBackInstructionType::AluStatusOnly,
        0x3F => WriteBackInstructionType::CoprocessorCall,
        _ => panic!("No mapping for [{instruction:?}] to WriteBackInstructionType"),
    }
}

fn update_status_flags(
    decoded: &DecodedInstruction,
    registers: &mut Registers,
    intermediate_registers: &IntermediateRegisters,
) {
    // TODO: Investigate whether `update_status_flags` could be cleaned up
    // category=Refactoring
    // Should this be done with an instruction type?
    // Not sure what this was referring to but worth a second look

    registers.sr = match decoded.sr_src {
        // TODO: Allow specifying explicit status register update source via assembly
        // category=Features
        // Currently it is set implicitly by the assembler but it could be handy in
        // some cases to specify it explicitly (especially if you don't want to
        // the existing value in blat the SR)
        StatusRegisterUpdateSource::Alu => {
            // Do not allow updates to the privileged byte of the SR via the ALU!
            (registers.sr & SR_PRIVILEGED_MASK)
                | (intermediate_registers.alu_status_register & SR_REDACTION_MASK)
        }
        StatusRegisterUpdateSource::Shift => {
            (registers.sr & SR_PRIVILEGED_MASK) | (decoded.sr_shift & SR_REDACTION_MASK)
        }
        _ => registers.sr,
    }
}

impl StageExecutor for WriteBackExecutor {
    fn execute(
        decoded: &DecodedInstruction,
        registers: &mut Registers,
        _: &mut ExceptionUnitRegisters,
        intermediate_registers: &mut IntermediateRegisters,
        bus_assertions: BusAssertions,
    ) -> BusAssertions {
        let write_back_step_instruction_type =
            decode_write_back_step_instruction_type(decoded.ins, decoded);

        // Internal Registers
        match write_back_step_instruction_type {
            WriteBackInstructionType::NoOp | WriteBackInstructionType::MemoryLoad => {}
            WriteBackInstructionType::AluStatusOnly => {
                update_status_flags(decoded, registers, intermediate_registers);
            }
            WriteBackInstructionType::AluToRegister => {
                set_register_value(registers, decoded.des, intermediate_registers.alu_output);
                update_status_flags(decoded, registers, intermediate_registers);
            }
            WriteBackInstructionType::AddressWrite => {
                registers[decoded.des_ad_h] = decoded.ad_h_;
                registers[decoded.des_ad_l] = intermediate_registers.address_output;
            }
            WriteBackInstructionType::AddressWriteLoadPostDecrement
            | WriteBackInstructionType::AddressWriteStorePreIncrement => {
                registers[decoded.ad_h] = decoded.ad_h_;
                registers[decoded.ad_l] = intermediate_registers.address_output;
            }
            WriteBackInstructionType::CoprocessorCall => {
                registers.pending_coprocessor_command = intermediate_registers.alu_output;
            }
        }

        // Load from Memory
        match write_back_step_instruction_type {
            WriteBackInstructionType::MemoryLoad
            | WriteBackInstructionType::AddressWriteLoadPostDecrement => {
                // LOAD instructions never update the status register, so status register updates are ignored, regardless of the status register update source parameter
                let (shifted, _) = do_shift(registers, bus_assertions.data, &decoded.shift_params);

                set_register_value(registers, decoded.des, shifted);
            }
            WriteBackInstructionType::NoOp
            | WriteBackInstructionType::AluToRegister
            | WriteBackInstructionType::AluStatusOnly
            | WriteBackInstructionType::AddressWrite
            | WriteBackInstructionType::AddressWriteStorePreIncrement
            | WriteBackInstructionType::CoprocessorCall => {}
        }
        BusAssertions::default()
    }
}
