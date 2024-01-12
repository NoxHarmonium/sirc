use peripheral_bus::BusPeripheral;

use crate::{
    coprocessors::{
        exception_unit::definitions::{
            vectors::{
                LEVEL_FIVE_HARDWARE_EXCEPTION, LEVEL_ONE_HARDWARE_EXCEPTION,
                USER_EXCEPTION_VECTOR_START,
            },
            ExceptionPriorities,
        },
        processing_unit::definitions::INSTRUCTION_SIZE_WORDS,
        shared::fetch_instruction,
    },
    registers::{
        get_interrupt_mask, ExceptionLinkRegister, ExceptionUnitRegisters,
        FullAddressRegisterAccess, Registers,
    },
    CAUSE_OPCODE_ID_LENGTH, CAUSE_OPCODE_ID_MASK, COPROCESSOR_ID_LENGTH, COPROCESSOR_ID_MASK,
};

use super::super::shared::Executor;
use super::{
    super::super::Error, definitions::ExceptionUnitOpCodes,
    encoding::decode_exception_unit_instruction,
};
use crate::registers::FullAddress;
use crate::registers::{set_interrupt_mask, set_sr_bit, StatusRegisterFields};

///
/// Constructs a value that can be put into the cause register to run an exception unit instruction.
///
/// The cause register is used to communicate between different CPU units.
///
/// The first nibble is the co processor ID (e.g. 1 for the exception unit)
/// The second nibble is the co processor op code (e.g. what command to run on the exception unit)
/// The third and fourth nibbles are an arbitrary value that is optionally used in the instruction.
///
/// ```
/// use peripheral_cpu::coprocessors::exception_unit::execution::construct_cause_value;
/// use peripheral_cpu::coprocessors::exception_unit::definitions::ExceptionUnitOpCodes;
///
/// // Software exeception at vector 0x10
///  let actual = construct_cause_value(&ExceptionUnitOpCodes::SoftwareException, 0x10);
///  assert_eq!(0x1110, actual);
/// ```
///
#[allow(clippy::cast_lossless)]
pub fn construct_cause_value(op_code: &ExceptionUnitOpCodes, value: u8) -> u16 {
    let executor_id: u8 = ExceptionUnitExecutor::COPROCESSOR_ID;
    let op_code_id = num::ToPrimitive::to_u8(op_code)
        .expect("ExceptionUnitOpCodes values should fit into 8 bits");
    ((executor_id as u16) << COPROCESSOR_ID_LENGTH) & COPROCESSOR_ID_MASK
        | ((op_code_id as u16) << CAUSE_OPCODE_ID_LENGTH) & CAUSE_OPCODE_ID_MASK
        | value as u16
}

pub fn hardware_exception_level_to_vector(level: u8) -> u8 {
    assert!(level != ExceptionPriorities::NoException as u8, "Raising a level zero hardware interrupt makes no sense (and would be impossible in hardware)");
    assert!(
        level != ExceptionPriorities::Software as u8,
        "Level one interrupts are reserved for software interrupts"
    );
    // Priority is the second nibble of the vector ID
    // So if we shift the level left by a nibble we get the vector
    let flipped = ExceptionPriorities::LevelFiveHardware as u8 - level + 1;
    let vector = flipped << (u8::BITS / 2);
    assert!(
        (LEVEL_FIVE_HARDWARE_EXCEPTION..=LEVEL_ONE_HARDWARE_EXCEPTION).contains(&vector),
        "Calculated hardware exception vector should be in valid range ({LEVEL_FIVE_HARDWARE_EXCEPTION}-{LEVEL_ONE_HARDWARE_EXCEPTION}) got [{vector}]"
    );
    vector
}

pub fn get_cause_register_value(
    registers: &Registers,
    eu_registers: &ExceptionUnitRegisters,
) -> u16 {
    if eu_registers.pending_hardware_exception_level > get_interrupt_mask(registers) {
        let vector =
            hardware_exception_level_to_vector(eu_registers.pending_hardware_exception_level);
        return construct_cause_value(&ExceptionUnitOpCodes::HardwareException, vector);
    };
    registers.pending_coprocessor_command
}

pub fn raise_hardware_interrupt(
    eu_registers: &mut ExceptionUnitRegisters,
    level: ExceptionPriorities,
) {
    assert!(level != ExceptionPriorities::NoException, "Raising a level zero hardware interrupt makes no sense (and would be impossible in hardware)");
    eu_registers.pending_hardware_exception_level = level as u8;
}

pub struct ExceptionUnitExecutor {}

impl Executor for ExceptionUnitExecutor {
    const COPROCESSOR_ID: u8 = 1;

    #[allow(clippy::cast_lossless)]
    #[allow(clippy::cast_possible_truncation)]
    fn step<'a>(
        cause_register_value: u16,
        registers: &'a mut Registers,
        eu_registers: &'a mut ExceptionUnitRegisters,
        mem: &BusPeripheral,
    ) -> Result<(&'a Registers, &'a mut ExceptionUnitRegisters), Error> {
        // TODO: Implement hardware exception triggers
        // TODO: Implement waiting for exception

        let decoded = decode_exception_unit_instruction(cause_register_value);
        let vector_address_low = decoded.value * (INSTRUCTION_SIZE_WORDS as u8);

        // Fetch the vector address
        let vector_address_bytes = fetch_instruction(
            mem,
            (registers.system_ram_offset | vector_address_low as u32).to_segmented_address(),
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

        assert!(
            typed_op_code != ExceptionUnitOpCodes::SoftwareException || vector_address_low >= USER_EXCEPTION_VECTOR_START,
            "The first section of the exception vector address space is reserved for hardware exceptions. Expected >= 0x{USER_EXCEPTION_VECTOR_START:X} got 0x{vector_address_low:X}",
        );

        println!(
            "XU: typed_op_code: {typed_op_code:#?} cause_register_value: 0x{cause_register_value:X?} vector_address: 0x{vector_address:X?} current_interrupt_mask: {current_interrupt_mask}"
        );

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
                eu_registers.waiting_for_exception = true;
            }
            ExceptionUnitOpCodes::ReturnFromException => {
                // Store current windowed link register to PC and jump to vector
                let ExceptionLinkRegister {
                    return_address,
                    return_status_register,
                } = eu_registers.link_registers[(current_interrupt_mask - 1) as usize];

                registers.set_full_pc_address(return_address);
                registers.sr = return_status_register;
            }
            ExceptionUnitOpCodes::Reset => {
                registers.sr = 0x0;
                set_sr_bit(StatusRegisterFields::SystemMode, registers);
                registers.set_full_pc_address(vector_address);
            }
            ExceptionUnitOpCodes::HardwareException => {
                handle_exception(
                    current_interrupt_mask,
                    eu_registers.pending_hardware_exception_level,
                    registers,
                    eu_registers,
                    vector_address,
                );
            }
        }

        eu_registers.pending_hardware_exception_level = 0x0;
        // TODO: Check if this could mess things up in situations like: 1. User calls to imaginary coprocessor to do something like a floating point calculation 2. there is a HW interrupt before the COP can handle it. 3. The cause register is cleared and the FP COP never executes anything
        registers.pending_coprocessor_command = 0x0;

        Ok((registers, eu_registers))
    }
}

fn handle_exception(
    current_interrupt_mask: u8,
    exception_level: u8,
    registers: &mut Registers,
    eu_registers: &mut ExceptionUnitRegisters,
    vector_address: u32,
) {
    println!("XU: current_interrupt_mask: {current_interrupt_mask} >= exception_level: {exception_level} ?");
    // Store current PC in windowed link register and jump to vector
    if current_interrupt_mask >= exception_level {
        // Ignore lower priority exceptions
        return;
    }
    eu_registers.link_registers[(exception_level - 1) as usize] = ExceptionLinkRegister {
        return_address: registers.get_full_pc_address(),
        return_status_register: registers.sr,
    };
    set_sr_bit(StatusRegisterFields::SystemMode, registers);
    registers.set_full_pc_address(vector_address);
    set_interrupt_mask(registers, exception_level);
}
