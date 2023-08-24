use peripheral_mem::MemoryPeripheral;

use crate::{
    coprocessors::{
        exception_unit::definitions::vectors, processing_unit::definitions::INSTRUCTION_SIZE_WORDS,
        shared::fetch_instruction,
    },
    registers::{
        get_interrupt_mask, ExceptionLinkRegister, ExceptionUnitRegisters,
        FullAddressRegisterAccess, Registers,
    },
    CYCLES_PER_INSTRUCTION,
};

use super::super::super::Error;
use super::super::shared::Executor;
use crate::registers::FullAddress;
use crate::registers::{set_interrupt_mask, set_sr_bit, StatusRegisterFields};

pub struct ExceptionUnitExecutor {}

impl Executor for ExceptionUnitExecutor {
    #[allow(clippy::cast_lossless)]
    fn step<'a>(
        registers: &'a mut Registers,
        eu_registers: &'a mut ExceptionUnitRegisters,
        mem: &MemoryPeripheral,
    ) -> Result<(&'a Registers, &'a mut ExceptionUnitRegisters, u32), Error> {
        // TODO: Implement return from exception
        // TODO: Implement hardware exception triggers
        // TODO: Implement waiting for exception

        let op = (eu_registers.cause_register & 0x0F00) >> 8;
        let vector = (eu_registers.cause_register & 0xFF) as u8;

        // Fetch the vector address
        let vector_address_bytes = fetch_instruction(
            mem,
            (registers.system_ram_offset + (vector as u32 * INSTRUCTION_SIZE_WORDS))
                .to_segmented_address(),
        );
        let vector_address = u32::from_be_bytes(vector_address_bytes);
        let current_interrupt_mask: u8 = get_interrupt_mask(registers);

        match op {
            0x01 => {
                // WAIT
                println!("WAIT!");
            }
            0x02 => {
                // RETE
                // Store current windowed link register to PC and jump to vector
                let ExceptionLinkRegister {
                    return_address,
                    return_exception_mask,
                } = eu_registers.link_registers[current_interrupt_mask as usize];

                registers.set_full_pc_address(return_address);

                set_interrupt_mask(registers, return_exception_mask);
            }
            0x04 => {
                // EXCP
                // Store the current PC into the windowed interrupt register

                if current_interrupt_mask >= eu_registers.exception_level {
                    // Note: Interrupt level seven cannot be interrupted
                    return Ok((registers, eu_registers, CYCLES_PER_INSTRUCTION));
                }

                // Store current PC in windowed link register and jump to vector
                eu_registers.link_registers[eu_registers.exception_level as usize] =
                    ExceptionLinkRegister {
                        return_address: registers.get_full_pc_address(),
                        return_exception_mask: eu_registers.exception_level,
                    };

                registers.set_full_pc_address(vector_address);

                set_interrupt_mask(registers, eu_registers.exception_level);
            }
            _ => {
                // TODO: Real CPU can't panic. Work out what should actually happen here (probably nothing)
                panic!("Unimplemented op code [{op:X}] for exception co-processor")
            }
        }

        eu_registers.cause_register = 0x0;

        Ok((registers, eu_registers, CYCLES_PER_INSTRUCTION))
    }
}

pub fn jump_to_interrupt(vector_offset: u8, registers: &mut Registers, mem: &MemoryPeripheral) {
    // Store the SR here because we need to flip to system mode to use the system stack
    // which will affect the SR
    // let old_sr = registers.sr;
    // Flip into system mode so we can use the system stack etc.
    set_sr_bit(StatusRegisterFields::SystemMode, registers);

    // Save important registers to restore after the ISR
    // push_address_to_stack(registers, mem, registers.get_full_pc_address());
    // push_value_to_stack(registers, mem, old_sr);

    // Jump to ISR
    let vector_address = registers.system_ram_offset + u32::from(vector_offset);
    (registers.ph, registers.pl) = (
        mem.read_address(vector_address),
        mem.read_address(vector_address + 1),
    );
}

///
/// This function is currently unmaintained and probably unused but will be resurrected when I
/// get around to implementing the exception unit.
///
/// # Panics
/// Will panic if `interupt_level` does not fit in three bits
///
#[allow(clippy::cast_possible_truncation)]
pub fn trigger_hardware_interrupt(
    interrupt_level: u8,
    registers: &mut Registers,
    mem: &MemoryPeripheral,
) {
    assert!(!(interrupt_level == 0 || interrupt_level > 0b111), "Interrupt level (0x{interrupt_level:08x}) must be greater than zero and fit in three bits (max 7 in decimal).");

    let vector_offset_start: u8 = vectors::LEVEL_ONE_INTERRUPT as u8 - 1;
    let vector_offset = vector_offset_start + interrupt_level;

    jump_to_interrupt(vector_offset, registers, mem);

    // TODO: Does it matter that we do this after the jump?
    // Make sure that interrupts of the same or lower priority don't interrupt this ISR
    set_interrupt_mask(registers, interrupt_level);
}
