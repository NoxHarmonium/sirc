use peripheral_mem::MemoryPeripheral;

use crate::{
    coprocessors::{
        processing_unit::definitions::INSTRUCTION_SIZE_WORDS, shared::fetch_instruction,
    },
    registers::{
        get_interrupt_mask, ExceptionLinkRegister, ExceptionUnitRegisters,
        FullAddressRegisterAccess, Registers,
    },
    CYCLES_PER_INSTRUCTION,
};

use super::super::shared::Executor;
use super::{super::super::Error, encoding::decode_exception_unit_instruction};
use crate::registers::FullAddress;
use crate::registers::{set_interrupt_mask, set_sr_bit, StatusRegisterFields};

pub struct ExceptionUnitExecutor {}

impl Executor for ExceptionUnitExecutor {
    const COPROCESSOR_ID: u8 = 1;

    #[allow(clippy::cast_lossless)]
    fn step<'a>(
        registers: &'a mut Registers,
        eu_registers: &'a mut ExceptionUnitRegisters,
        mem: &MemoryPeripheral,
    ) -> Result<(&'a Registers, &'a mut ExceptionUnitRegisters, u32), Error> {
        // TODO: Implement return from exception
        // TODO: Implement hardware exception triggers
        // TODO: Implement waiting for exception

        let decoded = decode_exception_unit_instruction(eu_registers.cause_register);

        // Fetch the vector address
        let vector_address_bytes = fetch_instruction(
            mem,
            (registers.system_ram_offset + (decoded.value as u32 * INSTRUCTION_SIZE_WORDS))
                .to_segmented_address(),
        );
        let vector_address = u32::from_be_bytes(vector_address_bytes);
        let current_interrupt_mask: u8 = get_interrupt_mask(registers);

        match decoded.op_code {
            0x00 => {
                // RESET
                registers.set_full_pc_address(vector_address);
                registers.sr = 0x0;
                set_sr_bit(StatusRegisterFields::SystemMode, registers);
            }
            0x01 => {
                // WAIT
                // TODO: Hardware exceptions
                println!("WAIT!");
            }
            0x02 => {
                // RETE
                // Store current windowed link register to PC and jump to vector
                let ExceptionLinkRegister {
                    return_address,
                    return_status_register,
                } = eu_registers.link_registers[current_interrupt_mask as usize];

                registers.set_full_pc_address(return_address);
                registers.sr = return_status_register;
            }
            0x04 => {
                // EXCP
                // Store the current PC into the windowed interrupt register

                if current_interrupt_mask >= eu_registers.exception_level {
                    // Ignore lower priority exceptions
                    // Note: Interrupt level seven cannot be interrupted
                    return Ok((registers, eu_registers, CYCLES_PER_INSTRUCTION));
                }

                // Store current PC in windowed link register and jump to vector
                eu_registers.link_registers[eu_registers.exception_level as usize] =
                    ExceptionLinkRegister {
                        return_address: registers.get_full_pc_address(),
                        return_status_register: registers.sr,
                    };

                registers.set_full_pc_address(vector_address);

                set_interrupt_mask(registers, eu_registers.exception_level);
            }
            _ => {
                // TODO: Real CPU can't panic. Work out what should actually happen here (probably nothing)
                panic!(
                    "Unimplemented op code [{:X}] for exception co-processor",
                    decoded.op_code
                )
            }
        }

        eu_registers.cause_register = 0x0;

        Ok((registers, eu_registers, CYCLES_PER_INSTRUCTION))
    }
}
