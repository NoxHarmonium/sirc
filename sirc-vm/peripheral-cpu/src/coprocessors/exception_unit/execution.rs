use peripheral_mem::MemoryPeripheral;

use crate::{
    coprocessors::{
        exception_unit::definitions::{
            vectors::{LEVEL_FIVE_HARDWARE_EXCEPTION, LEVEL_ONE_HARDWARE_EXCEPTION},
            ExceptionPriorities,
        },
        processing_unit::definitions::INSTRUCTION_SIZE_WORDS,
        shared::fetch_instruction,
    },
    registers::{
        get_interrupt_mask, ExceptionLinkRegister, ExceptionUnitRegisters,
        FullAddressRegisterAccess, Registers,
    },
    CYCLES_PER_INSTRUCTION,
};

use super::super::shared::Executor;
use super::{
    super::super::Error, definitions::ExceptionUnitOpCodes,
    encoding::decode_exception_unit_instruction,
};
use crate::registers::FullAddress;
use crate::registers::{set_interrupt_mask, set_sr_bit, StatusRegisterFields};

pub fn hardware_exception_level_to_vector(level: u8) -> u16 {
    assert!(level != ExceptionPriorities::NoException as u8, "Raising a level zero hardware interrupt makes no sense (and would be impossible in hardware)");
    let vector = (u16::from(level) - 1) + LEVEL_ONE_HARDWARE_EXCEPTION;
    assert!(
        (LEVEL_FIVE_HARDWARE_EXCEPTION..=LEVEL_ONE_HARDWARE_EXCEPTION).contains(&vector),
        "Calculated hardware exception vector should be in valid range ({LEVEL_ONE_HARDWARE_EXCEPTION}-{LEVEL_FIVE_HARDWARE_EXCEPTION}) got [{vector}]"
    );
    vector
}

pub fn set_cause_register_if_pending_interrupt(
    registers: &Registers,
    eu_registers: &mut ExceptionUnitRegisters,
) {
    if eu_registers.exception_level > get_interrupt_mask(registers) {
        let vector = hardware_exception_level_to_vector(eu_registers.exception_level);
        eu_registers.hardware_cause_register = 0x1400 | vector;
    };
}

pub fn raise_hardware_interrupt(
    eu_registers: &mut ExceptionUnitRegisters,
    level: ExceptionPriorities,
) {
    assert!(level != ExceptionPriorities::NoException, "Raising a level zero hardware interrupt makes no sense (and would be impossible in hardware)");
    eu_registers.exception_level = level as u8;
}

pub struct ExceptionUnitExecutor {}

impl Executor for ExceptionUnitExecutor {
    const COPROCESSOR_ID: u8 = 1;

    #[allow(clippy::cast_lossless)]
    fn step<'a>(
        registers: &'a mut Registers,
        eu_registers: &'a mut ExceptionUnitRegisters,
        mem: &MemoryPeripheral,
    ) -> Result<(&'a Registers, &'a mut ExceptionUnitRegisters, u32), Error> {
        // TODO: Implement hardware exception triggers
        // TODO: Implement waiting for exception

        let exception_level = eu_registers.exception_level;
        let is_hardware_exception = eu_registers.hardware_cause_register != 0;
        let cause_register_value = if is_hardware_exception {
            let value = eu_registers.hardware_cause_register;
            eu_registers.hardware_cause_register = 0x0;
            value
        } else {
            let value = registers.pending_coprocessor_command;
            registers.pending_coprocessor_command = 0x0;
            value
        };

        println!("cause_register_value: {cause_register_value:X?}");

        let decoded = decode_exception_unit_instruction(cause_register_value);

        // Fetch the vector address
        let vector_address_bytes = fetch_instruction(
            mem,
            (registers.system_ram_offset + (decoded.value as u32 * INSTRUCTION_SIZE_WORDS))
                .to_segmented_address(),
        );
        let vector_address = u32::from_be_bytes(vector_address_bytes);
        let current_interrupt_mask: u8 = get_interrupt_mask(registers);

        // TODO: Real CPU can't panic. Work out what should actually happen here (probably nothing)
        // TODO: Better error handling
        let typed_op_code = num::FromPrimitive::from_u8(decoded.op_code).unwrap_or_else(|| {
            panic!(
                "Unimplemented op code [{:X}] for exception co-processor",
                decoded.op_code
            )
        });

        match typed_op_code {
            ExceptionUnitOpCodes::SoftwareException => {
                handle_exception(
                    current_interrupt_mask,
                    1,
                    registers,
                    eu_registers,
                    vector_address,
                );
            }
            ExceptionUnitOpCodes::WaitForException => {
                // TODO:
                println!("WAIT!");
            }
            ExceptionUnitOpCodes::ReturnFromException => {
                // Store current windowed link register to PC and jump to vector
                let ExceptionLinkRegister {
                    return_address,
                    return_status_register,
                } = eu_registers.link_registers[current_interrupt_mask as usize];

                registers.set_full_pc_address(return_address);
                registers.sr = return_status_register;
            }
            ExceptionUnitOpCodes::Reset => {
                // RESET
                registers.set_full_pc_address(vector_address);
                registers.sr = 0x0;
                set_sr_bit(StatusRegisterFields::SystemMode, registers);
            }
            ExceptionUnitOpCodes::HardwareException => {
                handle_exception(
                    current_interrupt_mask,
                    exception_level,
                    registers,
                    eu_registers,
                    vector_address,
                );
            }
        }

        Ok((registers, eu_registers, CYCLES_PER_INSTRUCTION))
    }
}

fn handle_exception(
    current_interrupt_mask: u8,
    exception_level: u8,
    registers: &mut Registers,
    eu_registers: &mut ExceptionUnitRegisters,
    vector_address: u32,
) {
    println!("current_interrupt_mask: {current_interrupt_mask} exception_level: {exception_level}",);
    // Store current PC in windowed link register and jump to vector
    if current_interrupt_mask >= exception_level {
        // Ignore lower priority exceptions
        return;
    }
    eu_registers.exception_level = exception_level;
    eu_registers.link_registers[exception_level as usize] = ExceptionLinkRegister {
        return_address: registers.get_full_pc_address(),
        return_status_register: registers.sr,
    };
    registers.set_full_pc_address(vector_address);
    set_interrupt_mask(registers, exception_level);
}
