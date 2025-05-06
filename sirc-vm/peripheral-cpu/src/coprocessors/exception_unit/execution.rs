use log::{debug, trace};
use peripheral_bus::device::{BusAssertions, BusOperation};

use super::{
    super::shared::Executor,
    definitions::vectors::{
        ALIGNMENT_FAULT, BUS_FAULT, INSTRUCTION_TRACE_FAULT, INVALID_OPCODE_FAULT,
        LEVEL_FIVE_HARDWARE_EXCEPTION_CONFLICT, PRIVILEGE_VIOLATION_FAULT, SEGMENT_OVERFLOW_FAULT,
    },
};
use super::{
    definitions::ExceptionUnitOpCodes,
    encoding::{decode_exception_unit_instruction, ExceptionUnitInstruction},
};
use crate::{
    coprocessors::{
        exception_unit::definitions::{
            vectors::{
                LEVEL_FIVE_HARDWARE_EXCEPTION, LEVEL_ONE_HARDWARE_EXCEPTION,
                USER_EXCEPTION_VECTOR_START,
            },
            ExceptionPriorities, Faults, EXCEPTION_UNIT_TRANSFER_EU_REGISTER_LENGTH,
            EXCEPTION_UNIT_TRANSFER_EU_REGISTER_MASK, EXCEPTION_UNIT_TRANSFER_REGISTER_SELECT_MASK,
        },
        processing_unit::definitions::INSTRUCTION_SIZE_WORDS,
        shared::ExecutionPhase,
    },
    registers::{
        clear_sr_bit, get_interrupt_mask, ExceptionLinkRegister, ExceptionUnitRegisters,
        FullAddressRegisterAccess, Registers,
    },
    util::find_highest_active_bit,
    CAUSE_OPCODE_ID_LENGTH, CAUSE_OPCODE_ID_MASK, COPROCESSOR_ID_LENGTH, COPROCESSOR_ID_MASK,
};
use crate::{
    raise_fault,
    registers::{set_interrupt_mask, StatusRegisterFields},
};

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
pub fn construct_cause_value(op_code: &ExceptionUnitOpCodes, vector: u8) -> u16 {
    let executor_id: u8 = ExceptionUnitExecutor::COPROCESSOR_ID;
    let op_code_id = num::ToPrimitive::to_u8(op_code)
        .expect("ExceptionUnitOpCodes values should fit into 8 bits");

    ((executor_id as u16) << COPROCESSOR_ID_LENGTH) & COPROCESSOR_ID_MASK
        | ((op_code_id as u16) << CAUSE_OPCODE_ID_LENGTH) & CAUSE_OPCODE_ID_MASK
        | vector as u16
}

pub fn deconstruct_cause_value(cause_register_value: u16) -> (ExceptionUnitOpCodes, u8, u8) {
    let instruction = decode_exception_unit_instruction(cause_register_value);
    // TODO: Define what happens when exception CoP gets an unimplemented opcode
    // category=Hardware
    // Real CPU can't panic. Work out what should actually happen here (probably nothing)
    let op_code = num::FromPrimitive::from_u8(instruction.op_code).unwrap_or_else(|| {
        panic!(
            "Unimplemented op code [{:X}] for exception co-processor",
            instruction.op_code
        )
    });
    let vector = instruction.value;
    let level = if op_code == ExceptionUnitOpCodes::SoftwareException {
        1
    } else {
        // TODO: Possible overflow?
        7 - (vector >> 4)
    };
    // Make this a struct?
    (op_code, vector, level)
}

pub fn find_highest_active_interrupt(active_interrupt_bitfield: u8) -> u8 {
    // There are only 5 interrupt lines
    let valid_bits = 0x1F;
    find_highest_active_bit(active_interrupt_bitfield & valid_bits)
}

#[allow(clippy::cast_possible_truncation)]
pub fn interrupt_flags_to_exception_level(active_interrupt_bitfield: u8) -> u8 {
    if active_interrupt_bitfield == 0 {
        return 0;
    }
    // Even though multiple interrupts can be active at once,
    // we should service the highest priority one
    let highest_active_interrupt = find_highest_active_interrupt(active_interrupt_bitfield);
    // +1 because hardware interrupts start at 2 (1 is software exception)
    (highest_active_interrupt.ilog2()) as u8 + 2
}

///
/// Takes an CPU interrupt bit mask (e.g. a bit for each active interrupt) and
/// returns the vector number. It will always use the MSB to determine the level.
///
/// ```
/// use assert_hex::assert_eq_hex;
/// use peripheral_cpu::coprocessors::exception_unit::execution::hardware_exception_level_to_vector;
///
/// assert_eq_hex!(0x50 ,hardware_exception_level_to_vector(0x01));
/// assert_eq_hex!(0x40 ,hardware_exception_level_to_vector(0x02));
/// assert_eq_hex!(0x30, hardware_exception_level_to_vector(0x03));
/// assert_eq_hex!(0x20, hardware_exception_level_to_vector(0x04));
/// assert_eq_hex!(0x10, hardware_exception_level_to_vector(0x05));
///
/// ```
pub fn hardware_exception_level_to_vector(exception_level: u8) -> u8 {
    assert!(exception_level != ExceptionPriorities::NoException as u8, "Raising a level zero hardware interrupt makes no sense (and would be impossible in hardware)");
    assert!(
        exception_level < ExceptionPriorities::LevelFiveHardware as u8,
        "There is no interrupt level above level five"
    );

    // Priority is the second nibble of the vector ID
    // So if we shift the level left by a nibble we get the vector
    // We also invert it because vectors with lower memory addresses are higher priority
    let flipped = ExceptionPriorities::LevelFiveHardware as u8 - exception_level;
    let vector = flipped << (u8::BITS / 2);
    assert!(
        (LEVEL_FIVE_HARDWARE_EXCEPTION..=LEVEL_ONE_HARDWARE_EXCEPTION).contains(&vector),
        "Calculated hardware exception vector should be in valid range ({LEVEL_FIVE_HARDWARE_EXCEPTION}-{LEVEL_ONE_HARDWARE_EXCEPTION}) got [{vector}]"
    );
    trace!("hardware_exception_level_to_vector  {exception_level} vector: 0x{vector:X}");
    vector
}

pub fn get_cause_register_value(
    registers: &Registers,
    eu_registers: &mut ExceptionUnitRegisters,
) -> u16 {
    if let Some(pending_fault) = eu_registers.pending_fault {
        let vector = match pending_fault {
            Faults::Bus => BUS_FAULT,
            Faults::Alignment => ALIGNMENT_FAULT,
            Faults::SegmentOverflow => SEGMENT_OVERFLOW_FAULT,
            Faults::InvalidOpCode => INVALID_OPCODE_FAULT,
            Faults::PrivilegeViolation => PRIVILEGE_VIOLATION_FAULT,
            Faults::InstructionTrace => INSTRUCTION_TRACE_FAULT,
            Faults::LevelFiveInterruptConflict => LEVEL_FIVE_HARDWARE_EXCEPTION_CONFLICT,
        };

        return construct_cause_value(&ExceptionUnitOpCodes::Fault, vector);
    }
    let exception_level =
        interrupt_flags_to_exception_level(eu_registers.pending_hardware_exceptions);

    trace!(
        "eu_registers.pending_hardware_exceptions: 0x[{:X}] interrupt exception level: 0x[{exception_level:X}] current mask: 0x[{:X}]",
        eu_registers.pending_hardware_exceptions,
        get_interrupt_mask(registers)
    );

    // TODO: Why no fault when L5 interrupt called twice?
    // Probably because its masked
    if exception_level > get_interrupt_mask(registers) || exception_level == 6 {
        trace!("servicing interrupt by injecting cause value. exception_level {exception_level}");
        let vector = hardware_exception_level_to_vector(exception_level - 1);
        // Clear highest interrupt because we will now service it
        eu_registers.pending_hardware_exceptions &=
            !find_highest_active_interrupt(eu_registers.pending_hardware_exceptions);

        debug!(
            "eu_registers.pending_hardware_exceptions: {}",
            eu_registers.pending_hardware_exceptions
        );
        return construct_cause_value(&ExceptionUnitOpCodes::HardwareException, vector);
    }
    trace!(
        "returning registers.pending_coprocessor_command 0x[{:X}]",
        registers.pending_coprocessor_command
    );

    registers.pending_coprocessor_command
}

fn extract_transfer_instruction_parameters<'a>(
    exception_unit_instruction: &ExceptionUnitInstruction,
    eu_registers: &'a mut ExceptionUnitRegisters,
) -> (u8, &'a mut ExceptionLinkRegister) {
    let eu_register = exception_unit_instruction.value & EXCEPTION_UNIT_TRANSFER_EU_REGISTER_MASK;
    let register_select = (exception_unit_instruction.value
        >> EXCEPTION_UNIT_TRANSFER_EU_REGISTER_LENGTH)
        & EXCEPTION_UNIT_TRANSFER_REGISTER_SELECT_MASK;

    trace!("----> eu_register: {eu_register}");

    // TODO: Again, what would happen in hardware if this was out of range in hardware?
    let link_register = eu_registers
        .link_registers
        .get_mut(eu_register as usize)
        .expect("ETFR Expected target register value between 0-7");
    (register_select, link_register)
}

fn handle_exception(
    current_interrupt_mask: u8,
    exception_level: u8,
    registers: &mut Registers,
    eu_registers: &mut ExceptionUnitRegisters,
    vector_address: u32,
    bus_assertions: &BusAssertions,
) {
    trace!("HANDLE EXCEPTION: current_interrupt_mask: 0x{current_interrupt_mask:X} exception_level: 0x{exception_level:X} pc: 0x{:X} vector_address: 0x{vector_address:X}", registers.get_full_pc_address() );

    // TODO: Magic numbers
    if current_interrupt_mask == 6 && exception_level == 6 {
        eu_registers.pending_fault = raise_fault(
            registers,
            eu_registers,
            Faults::LevelFiveInterruptConflict,
            // TODO: Don't hardcode this
            ExecutionPhase::ExecutionEffectiveAddressExecutor,
            bus_assertions,
        );
        return;
    }

    // Store current PC in windowed link register and jump to vector
    if current_interrupt_mask >= exception_level {
        // TODO: Should this just be done when determining the cause register?
        // Ignore lower priority exceptions
        return;
    }
    eu_registers.link_registers[(exception_level - 1) as usize] = ExceptionLinkRegister {
        return_address: registers.get_full_pc_address(),
        return_status_register: registers.sr,
    };

    clear_sr_bit(StatusRegisterFields::ProtectedMode, registers);

    registers.set_full_pc_address(vector_address);
    set_interrupt_mask(registers, exception_level);
}

#[derive(Default)]
pub struct ExceptionUnitExecutor {
    pub vector_address: u32,
    pub vector_value: u32,
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

        trace!(
            "PHASE '{phase:?}' :: FAULT '{:?}'",
            eu_registers.pending_fault
        );

        let instruction = decode_exception_unit_instruction(cause_register_value);
        let (op_code, vector, level) = deconstruct_cause_value(cause_register_value);

        trace!("op_code: {op_code:?} vector: 0x{vector:X} level: 0x{level:X}");

        match phase {
            ExecutionPhase::InstructionFetchLow => {
                let vector_address_offset = vector as u32 * INSTRUCTION_SIZE_WORDS;
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
                let vector_address_high = self.vector_address as u8 & u8::MAX;

                debug!("0x{:X}: {:?}", registers.get_full_pc_address(), op_code);

                assert!(
                    op_code != ExceptionUnitOpCodes::SoftwareException || vector_address_high >= USER_EXCEPTION_VECTOR_START,
                    "The first section of the exception vector address space is reserved for hardware exceptions. Expected >= 0x{USER_EXCEPTION_VECTOR_START:X} got 0x{vector_address_high:X}",
                );
            }
            ExecutionPhase::ExecutionEffectiveAddressExecutor => {
                // TODO: Some of these may have to be spread into the other execution phases depending on things like memory access (e.g. to make it like the CPU - one op per phase)
                let current_interrupt_mask: u8 = get_interrupt_mask(registers);

                // TODO: Sort these cases?
                match op_code {
                    ExceptionUnitOpCodes::None => {
                        // No-op
                    }
                    ExceptionUnitOpCodes::WaitForException => {
                        eu_registers.waiting_for_exception = true;
                    }
                    ExceptionUnitOpCodes::ReturnFromException => {
                        // Store current windowed link register to PC and jump to vector
                        // TODO: Is it safe to use the current_interrupt_mask here? Could that be user editable? Does that matter?
                        // TODO: Overflow error when current_interrupt_mask is zero. What should actually happen on hardware if you try to RTE when you're not in an exception
                        let ExceptionLinkRegister {
                            return_address,
                            return_status_register,
                        } = eu_registers.link_registers[(current_interrupt_mask - 1) as usize];

                        registers.set_full_pc_address(return_address);
                        registers.sr = return_status_register;
                    }
                    ExceptionUnitOpCodes::Reset => {
                        registers.sr = 0x0;
                        registers.set_full_pc_address(self.vector_value);
                    }
                    ExceptionUnitOpCodes::TransferFromRegister => {
                        let (register_select, link_register) =
                            extract_transfer_instruction_parameters(&instruction, eu_registers);

                        match register_select {
                            0 => {}
                            1 => {
                                registers.set_full_address_address(link_register.return_address);
                            }
                            2 => registers.r7 = link_register.return_status_register,
                            3 => {
                                registers.set_full_address_address(link_register.return_address);
                                registers.r7 = link_register.return_status_register;
                            }
                            _ => panic!("ETFR register select can only be in the range of 0-3"),
                        }

                        trace!("ETFR! register_select: {register_select} link_register: {link_register:X?} lri: {} | <-- Registers: a: [{:X}] r7: [{:X}]", current_interrupt_mask - 1, registers.get_full_address_address(), registers.r7);
                        trace!("ALL: {:X?}", eu_registers.link_registers);
                    }
                    ExceptionUnitOpCodes::TransferToRegister => {
                        let (register_select, link_register) =
                            extract_transfer_instruction_parameters(&instruction, eu_registers);

                        trace!(
                            "B4! Registers: a: [{:X}] r7: [{:X}]",
                            registers.get_full_address_address(),
                            registers.r7
                        );

                        match register_select {
                            0 => {}
                            1 => {
                                link_register.return_address = registers.get_full_address_address();
                            }
                            2 => link_register.return_status_register = registers.r7,
                            3 => {
                                link_register.return_address = registers.get_full_address_address();
                                link_register.return_status_register = registers.r7;
                            }
                            _ => panic!("ETTR register select can only be in the range of 0-3"),
                        }

                        trace!("ETTR! register_select: {register_select} link_register: {link_register:X?} lri: {} | --> Registers: a: [{:X}] r7: [{:X}]", current_interrupt_mask - 1, registers.get_full_address_address(), registers.r7);
                    }
                    ExceptionUnitOpCodes::Fault
                    | ExceptionUnitOpCodes::HardwareException
                    | ExceptionUnitOpCodes::SoftwareException => {
                        handle_exception(
                            current_interrupt_mask,
                            level,
                            registers,
                            eu_registers,
                            self.vector_value,
                            &bus_assertions,
                        );
                    }
                }
            }
            ExecutionPhase::MemoryAccessExecutor => {}
            ExecutionPhase::WriteBackExecutor => {
                // TODO: Where should these go?
                eu_registers.pending_fault = None;

                // TODO: Check if this could mess things up in situations like: 1. User calls to imaginary coprocessor to do something like a floating point calculation 2. there is a HW interrupt before the COP can handle it. 3. The cause register is cleared and the FP COP never executes anything
                registers.pending_coprocessor_command = 0x0;
            }
        }

        BusAssertions::default()
    }
}
