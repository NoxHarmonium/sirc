use log::trace;
use peripheral_bus::device::{BusAssertions, BusOperation};

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
        shared::ExecutionPhase,
    },
    registers::{
        get_interrupt_mask, ExceptionLinkRegister, ExceptionUnitRegisters,
        FullAddressRegisterAccess, Registers,
    },
    util::find_highest_active_bit,
    CAUSE_OPCODE_ID_LENGTH, CAUSE_OPCODE_ID_MASK, COPROCESSOR_ID_LENGTH, COPROCESSOR_ID_MASK,
};

use super::super::shared::Executor;
use super::{
    definitions::ExceptionUnitOpCodes,
    encoding::{decode_exception_unit_instruction, ExceptionUnitInstruction},
};
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

///
/// Takes an CPU interrupt bit mask (e.g. a bit for each active interrupt) and
/// returns the vector number. It will always use the MSB to determine the level.
///
/// ```
/// use assert_hex::assert_eq_hex;
/// use peripheral_cpu::coprocessors::exception_unit::execution::hardware_exception_level_to_vector;
///
/// assert_eq_hex!(0x50, hardware_exception_level_to_vector(0x01));
/// assert_eq_hex!(0x40 ,hardware_exception_level_to_vector(0x02));
/// assert_eq_hex!(0x30, hardware_exception_level_to_vector(0x04));
/// assert_eq_hex!(0x20, hardware_exception_level_to_vector(0x08));
/// assert_eq_hex!(0x10, hardware_exception_level_to_vector(0x10));
///
/// assert_eq_hex!(0x10, hardware_exception_level_to_vector(0xFF));
/// assert_eq_hex!(0x10 ,hardware_exception_level_to_vector(0x12));
/// assert_eq_hex!(0x20, hardware_exception_level_to_vector(0x0F));
/// assert_eq_hex!(0x40, hardware_exception_level_to_vector(0x03));
/// assert_eq_hex!(0x30, hardware_exception_level_to_vector(0x06));
///
/// ```
#[allow(clippy::cast_possible_truncation)]
pub fn hardware_exception_level_to_vector(active_interrupt_bitfield: u8) -> u8 {
    // There are only 5 interrupt lines
    let valid_bits = 0x1F;
    // Even though multiple interrupts can be active at once,
    // we should service the highest priority one
    let highest_active_interrupt = find_highest_active_bit(active_interrupt_bitfield & valid_bits);
    // +1 to make bit zero into interrupt one, +1 again because hardware interrupts start at 2 (1 is software exception)
    let exception_level: u8 = (highest_active_interrupt.ilog2()) as u8 + 2;

    assert!(exception_level != ExceptionPriorities::NoException as u8, "Raising a level zero hardware interrupt makes no sense (and would be impossible in hardware)");
    assert!(
        exception_level != ExceptionPriorities::Software as u8,
        "Level one interrupts are reserved for software interrupts"
    );
    assert!(
        exception_level < ExceptionPriorities::Fault as u8,
        "There is no interrupt level above level five"
    );

    // Priority is the second nibble of the vector ID
    // So if we shift the level left by a nibble we get the vector
    // We also invert it because vectors with lower memory addresses are higher priority
    let flipped = ExceptionPriorities::LevelFiveHardware as u8 - exception_level + 1;
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
    // let fault_can_occur = get_interrupt_mask(registers) < ExceptionPriorities::Fault as u8;

    // WHAT I WAS UP TO. Trying to get the first fault (bus fault) to actually work
    // TODO: The bus error is stuck in a loop or something
    // Actually! :-( bus error can occur mid instruction (.e.g IF/mem read/writeback) so I think we need to make the bus clock more granular
    // if fault_can_occur {
    //     if let Some(pending_fault) = eu_registers.pending_fault {
    //         let vector = match pending_fault {
    //             super::definitions::Faults::Bus => BUS_FAULT,
    //             super::definitions::Faults::Alignment => ALIGNMENT_FAULT,
    //             super::definitions::Faults::SegmentOverflow => SEGMENT_OVERFLOW_FAULT,
    //             super::definitions::Faults::InvalidOpCode => INVALID_OPCODE_FAULT,
    //             super::definitions::Faults::PrivilegeViolation => PRIVILEGE_VIOLATION_FAULT,
    //             super::definitions::Faults::InstructionTrace => INSTRUCTION_TRACE_FAULT,
    //             super::definitions::Faults::LevelFiveInterruptConflict => {
    //                 LEVEL_FIVE_HARDWARE_EXCEPTION_CONFLICT
    //             }
    //         };

    //         return construct_cause_value(&ExceptionUnitOpCodes::HardwareException, vector);
    //     }
    // }
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

#[derive(Default)]
pub struct ExceptionUnitExecutor {
    pub exception_unit_instruction: ExceptionUnitInstruction,
    pub vector_address: u32,
    pub vector_value: u32,
    pub exception_unit_opcode: ExceptionUnitOpCodes,
    // TODO: Use cell?
}

impl Executor for ExceptionUnitExecutor {
    const COPROCESSOR_ID: u8 = 1;

    #[allow(clippy::cast_lossless)]
    #[allow(clippy::cast_possible_truncation)]
    fn step<'a>(
        &mut self,
        phase: &ExecutionPhase,
        cause_register_value: u16,
        registers: &'a mut Registers,
        eu_registers: &'a mut ExceptionUnitRegisters,
        bus_assertions: BusAssertions,
    ) -> BusAssertions {
        // TODO: Implement faults
        // TODO: P5 interrupts should be edge triggered and if one is triggered while another is being serviced, it should be a fault

        match phase {
            ExecutionPhase::InstructionFetchLow => {
                self.exception_unit_instruction =
                    decode_exception_unit_instruction(cause_register_value);
                let vector_address_offset =
                    self.exception_unit_instruction.value as u32 * INSTRUCTION_SIZE_WORDS;
                self.vector_address = registers.system_ram_offset | vector_address_offset;
                return BusAssertions {
                    address: self.vector_address,
                    op: BusOperation::Read,
                    ..BusAssertions::default()
                };
            }
            ExecutionPhase::InstructionFetchHigh => {
                self.vector_value = (bus_assertions.data as u32) << u16::BITS;
                return BusAssertions {
                    address: self.vector_address + 1,
                    op: BusOperation::Read,
                    ..BusAssertions::default()
                };
            }
            ExecutionPhase::InstructionDecode => {
                self.vector_value |= bus_assertions.data as u32;

                // TODO: Real CPU can't panic. Work out what should actually happen here (probably nothing)
                // TODO: Better error handling
                self.exception_unit_opcode =
                    num::FromPrimitive::from_u8(self.exception_unit_instruction.op_code)
                        .unwrap_or_else(|| {
                            panic!(
                                "Unimplemented op code [{:X}] for exception co-processor",
                                self.exception_unit_instruction.op_code
                            )
                        });

                let vector_address_high = self.vector_address as u8 & u8::MAX;

                assert!(
                    self.exception_unit_opcode != ExceptionUnitOpCodes::SoftwareException || vector_address_high >= USER_EXCEPTION_VECTOR_START,
                    "The first section of the exception vector address space is reserved for hardware exceptions. Expected >= 0x{USER_EXCEPTION_VECTOR_START:X} got 0x{vector_address_high:X}",
                );
            }
            ExecutionPhase::ExecutionEffectiveAddressExecutor => {
                // TODO: Some of these may have to be spread into the other execution phases depending on things like memory access
                let current_interrupt_mask: u8 = get_interrupt_mask(registers);

                match self.exception_unit_opcode {
                    ExceptionUnitOpCodes::None => {
                        // No-op
                    }
                    ExceptionUnitOpCodes::SoftwareException => {
                        handle_exception(
                            current_interrupt_mask,
                            1,
                            registers,
                            eu_registers,
                            self.vector_value,
                        );
                    }
                    ExceptionUnitOpCodes::WaitForException => {
                        eu_registers.waiting_for_exception = true;
                    }
                    ExceptionUnitOpCodes::ReturnFromException => {
                        // Store current windowed link register to PC and jump to vector
                        // TODO: Is it safe to use the current_interrupt_mask here? Could that be user editable? Does that matter?
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
                        registers.set_full_pc_address(self.vector_value);
                    }
                    ExceptionUnitOpCodes::HardwareException => {
                        handle_exception(
                            current_interrupt_mask,
                            eu_registers.pending_hardware_exception_level,
                            registers,
                            eu_registers,
                            self.vector_value,
                        );
                    }
                }
            }
            ExecutionPhase::MemoryAccessExecutor => {}
            ExecutionPhase::WriteBackExecutor => {
                // TODO: Where should these go?
                eu_registers.pending_hardware_exception_level = 0x0;
                // TODO: Check if this could mess things up in situations like: 1. User calls to imaginary coprocessor to do something like a floating point calculation 2. there is a HW interrupt before the COP can handle it. 3. The cause register is cleared and the FP COP never executes anything
                registers.pending_coprocessor_command = 0x0;
            }
        }

        BusAssertions::default()
    }
}

fn handle_exception(
    current_interrupt_mask: u8,
    exception_level: u8,
    registers: &mut Registers,
    eu_registers: &mut ExceptionUnitRegisters,
    vector_address: u32,
) {
    trace!("HANDLE EXCEPTION: current_interrupt_mask: 0x{current_interrupt_mask:X} exception_level: 0x{exception_level:X} pc: 0x{:X} vector_address: 0x{vector_address:X}", registers.get_full_pc_address() );

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
