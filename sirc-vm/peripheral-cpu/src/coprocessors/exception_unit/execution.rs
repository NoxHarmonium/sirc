use log::{debug, trace};
use peripheral_bus::device::{BusAccessType, BusAssertions, BusOperation};

use super::{
    super::shared::Executor,
    definitions::vectors::{
        ALIGNMENT_FAULT, BUS_FAULT, BUS_PROTECTION_FAULT, INSTRUCTION_TRACE_FAULT,
        INVALID_OPCODE_FAULT, LEVEL_FIVE_HARDWARE_EXCEPTION_CONFLICT, PRIVILEGE_VIOLATION_FAULT,
        SEGMENT_OVERFLOW_FAULT,
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
                DOUBLE_FAULT_VECTOR, LEVEL_FIVE_HARDWARE_EXCEPTION, LEVEL_ONE_HARDWARE_EXCEPTION,
            },
            ExceptionPriorities, Faults, PendingFault, EXCEPTION_UNIT_TRANSFER_EU_REGISTER_LENGTH,
            EXCEPTION_UNIT_TRANSFER_EU_REGISTER_MASK, EXCEPTION_UNIT_TRANSFER_REGISTER_SELECT_MASK,
        },
        processing_unit::definitions::INSTRUCTION_SIZE_WORDS,
        shared::ExecutionPhase,
    },
    registers::{
        clear_sr_bit, get_hardware_interrupt_enable, set_sr_bit, toggle_sr_bit,
        ExceptionLinkRegister, ExceptionUnitRegisters, FullAddressRegisterAccess, Registers,
    },
    util::find_highest_active_bit,
    COPROCESSOR_ID_LENGTH, COPROCESSOR_ID_MASK, COPROCESSOR_OPCODE_ID_LENGTH,
    COPROCESSOR_OPCODE_ID_MASK,
};
use crate::{raise_fault, registers::StatusRegisterFields};

/// Encode an exception-unit command word for CSR or automatic EU dispatch.
///
/// The CU keeps the queued CSR command separate from the active slot command.
///
/// The first nibble is the coprocessor ID (e.g. 1 for the exception unit)
/// The second nibble is the coprocessor op code (e.g. what command to run on the exception unit)
/// The third and fourth nibbles are an arbitrary value that is optionally used in the instruction.
///
/// ```
/// use peripheral_cpu::coprocessors::exception_unit::execution::construct_exception_unit_command;
/// use peripheral_cpu::coprocessors::exception_unit::definitions::ExceptionUnitOpCodes;
///
/// // Software exception at vector 0x10
///  let actual = construct_exception_unit_command(&ExceptionUnitOpCodes::SoftwareException, 0x10);
///  assert_eq!(0x1110, actual);
/// ```
///
#[allow(clippy::cast_lossless)]
pub fn construct_exception_unit_command(op_code: &ExceptionUnitOpCodes, vector: u8) -> u16 {
    let executor_id: u8 = ExceptionUnitExecutor::COPROCESSOR_ID;
    let op_code_id = num::ToPrimitive::to_u8(op_code)
        .expect("ExceptionUnitOpCodes values should fit into 8 bits");

    ((executor_id as u16) << COPROCESSOR_ID_LENGTH) & COPROCESSOR_ID_MASK
        | ((op_code_id as u16) << COPROCESSOR_OPCODE_ID_LENGTH) & COPROCESSOR_OPCODE_ID_MASK
        | vector as u16
}

/// Validate the operation before a software command can execute or access vectors.
#[must_use]
pub fn is_valid_exception_unit_command(command: u16) -> bool {
    let instruction = decode_exception_unit_instruction(command);
    num::FromPrimitive::from_u8(instruction.op_code).is_some_and(|_: ExceptionUnitOpCodes| true)
}

pub fn deconstruct_exception_unit_command(command: u16) -> (ExceptionUnitOpCodes, u8, u8) {
    let instruction = decode_exception_unit_instruction(command);
    // The CU rejects reserved operations before dispatch. Keep decoding total for
    // callers inspecting raw command words; decoding alone does not validate them.
    let op_code =
        num::FromPrimitive::from_u8(instruction.op_code).unwrap_or(ExceptionUnitOpCodes::None);
    let vector = instruction.value;
    let level = match op_code {
        ExceptionUnitOpCodes::SoftwareException => 1,
        // A no-op's payload is not a vector. Deriving a level from e.g. 0xFF
        // would underflow in debug builds, despite the result never being used.
        ExceptionUnitOpCodes::None => 0,
        _ => 7 - (vector >> 4),
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

/// An EU event eligible for dispatch, not yet consumed from the pending state.
/// If dispatch is deferred, discard this candidate and select again at the next boundary.
#[derive(Debug, PartialEq, Eq)]
pub struct PendingEvent {
    command: u16,
    // Zero for a fault: its metadata is retained until fault entry completes.
    interrupt_mask: u8,
    // A selected L5 conflict creates its fault and clears overrun only on acceptance.
    l5_conflict: bool,
}

/// Select an EU event without consuming it or considering software commands in CSR.
pub fn select_pending_event(
    registers: &Registers,
    eu_registers: &ExceptionUnitRegisters,
) -> Option<PendingEvent> {
    if let Some(PendingFault { fault, .. }) = eu_registers.pending_fault {
        let vector = match fault {
            Faults::Bus => BUS_FAULT,
            Faults::Alignment => ALIGNMENT_FAULT,
            Faults::SegmentOverflow => SEGMENT_OVERFLOW_FAULT,
            Faults::InvalidOpCode => INVALID_OPCODE_FAULT,
            Faults::PrivilegeViolation => PRIVILEGE_VIOLATION_FAULT,
            Faults::InstructionTrace => INSTRUCTION_TRACE_FAULT,
            Faults::LevelFiveInterruptConflict => LEVEL_FIVE_HARDWARE_EXCEPTION_CONFLICT,
            Faults::DoubleFault => DOUBLE_FAULT_VECTOR,
            Faults::BusProtection => BUS_PROTECTION_FAULT,
        };

        return Some(PendingEvent {
            command: construct_exception_unit_command(&ExceptionUnitOpCodes::Fault, vector),
            interrupt_mask: 0,
            l5_conflict: false,
        });
    }
    trace!(
        "eu_registers.pending_hardware_exceptions: 0x[{:X}] hw_interrupt_enable: 0x[{:X}] current_exception_level: 0x[{:X}]",
        eu_registers.pending_hardware_exceptions,
        get_hardware_interrupt_enable(registers),
        eu_registers.current_exception_level
    );

    // Walk every pending line from highest to lowest priority (exception_level 6 down to 2,
    // mapping to interrupt lines 5 down to 1) and dispatch the first one that is both pending
    // and still enabled. This used to only ever look at the single highest *pending* line
    // (via interrupt_flags_to_exception_level/find_highest_active_interrupt): if that one line's
    // enable bit had been cleared since it was latched, the function gave up for the whole
    // cycle and never considered any other pending line, even a lower-priority one that was
    // still enabled -- see docs/reference/review/implementation-bugs.md, IB-3.
    let hw_interrupt_enable = get_hardware_interrupt_enable(registers);
    for exception_level in (2..=6).rev() {
        let interrupt_line = exception_level - 2;
        let is_pending = (eu_registers.pending_hardware_exceptions & (1 << interrupt_line)) != 0;
        if !is_pending {
            continue;
        }

        // Also check that this exception is higher priority than the current level
        // (higher priority = higher exception level number).
        // L5 at level 6 is eligible for a conflict report, not handler re-entry.
        // At level 7 neither request nor overrun is consumed.
        let is_nmi_reentry = exception_level == ExceptionPriorities::LevelFiveHardware as u8
            && eu_registers.current_exception_level == ExceptionPriorities::LevelFiveHardware as u8;
        if !(exception_level > eu_registers.current_exception_level || is_nmi_reentry) {
            continue;
        }

        // Check if this specific hardware interrupt line is still enabled -- it may have been
        // latched while enabled and then masked before its turn to dispatch.
        let is_enabled = (hw_interrupt_enable & (1 << interrupt_line)) != 0;
        if !is_enabled {
            continue;
        }

        if exception_level == ExceptionPriorities::LevelFiveHardware as u8
            && (is_nmi_reentry || eu_registers.l5_overrun)
        {
            return Some(PendingEvent {
                command: construct_exception_unit_command(
                    &ExceptionUnitOpCodes::Fault,
                    LEVEL_FIVE_HARDWARE_EXCEPTION_CONFLICT,
                ),
                // Re-entry consumes the request. Overrun alone keeps it queued
                // for delivery after the conflict handler returns.
                interrupt_mask: if is_nmi_reentry {
                    1 << interrupt_line
                } else {
                    0
                },
                l5_conflict: true,
            });
        }

        trace!("eligible interrupt candidate: exception_level {exception_level}");
        let vector = hardware_exception_level_to_vector(exception_level - 1);
        return Some(PendingEvent {
            command: construct_exception_unit_command(
                &ExceptionUnitOpCodes::HardwareException,
                vector,
            ),
            interrupt_mask: 1 << interrupt_line,
            l5_conflict: false,
        });
    }
    None
}

impl PendingEvent {
    /// Accept this candidate when the CU grants dispatch, returning its command word.
    /// Taking ownership prevents the same candidate from being accepted twice.
    /// Fault metadata stays pending for the existing fault-entry/completion path.
    pub fn accept(self, eu_registers: &mut ExceptionUnitRegisters, pc: u32) -> u16 {
        if self.interrupt_mask != 0 {
            eu_registers.pending_hardware_exceptions &= !self.interrupt_mask;
            debug!(
                "eu_registers.pending_hardware_exceptions: {}",
                eu_registers.pending_hardware_exceptions
            );
        }
        if self.l5_conflict {
            eu_registers.l5_overrun = false;
            raise_fault(
                eu_registers,
                Faults::LevelFiveInterruptConflict,
                &BusAssertions::default(),
                pc,
            );
        }
        self.command
    }
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

    let link_register = eu_registers
        .link_registers
        .get_mut(eu_register as usize)
        .expect("ETFR Expected target register value between 0-7");
    (register_select, link_register)
}

/// Faults can enter at any level; other exceptions must outrank the handler.
/// In particular, L5 re-entry is a conflict, not a second handler entry.
fn accepts_exception_entry(current_level: u8, requested_level: u8) -> bool {
    requested_level == ExceptionPriorities::Fault as u8 || requested_level > current_level
}

fn handle_exception(
    current_exception_level: u8,
    exception_level: u8,
    registers: &mut Registers,
    eu_registers: &mut ExceptionUnitRegisters,
    vector_address: u32,
) {
    trace!("HANDLE EXCEPTION: current_exception_level: 0x{current_exception_level:X} exception_level: 0x{exception_level:X} pc: 0x{:X} vector_address: 0x{vector_address:X}", registers.get_full_pc_address() );

    // Store current PC in windowed link register and jump to vector
    if !accepts_exception_entry(current_exception_level, exception_level) {
        // TODO: Should this just be done when selecting the pending EU event?
        // Ignore lower priority exceptions
        return;
    }

    // For a Fault dispatch, `pending_fault` was set by `raise_fault` with the correct return
    // address for that specific fault (the faulting instruction's own address if retryable, the
    // following instruction's otherwise) and hasn't been cleared yet -- it's cleared later, in
    // `WriteBackExecutor`, once this dispatch has committed. For a HardwareException or
    // SoftwareException dispatch, no fault is pending, so fall back to the current PC as before.
    let return_address = eu_registers.pending_fault.map_or_else(
        || registers.get_full_pc_address(),
        |pending_fault| pending_fault.pc,
    );

    eu_registers.link_registers[(exception_level - 1) as usize] = ExceptionLinkRegister {
        return_address,
        return_status_register: registers.sr,
        saved_exception_level: current_exception_level,
    };

    clear_sr_bit(StatusRegisterFields::ProtectedMode, registers);
    clear_sr_bit(StatusRegisterFields::TraceMode, registers);
    // Set ExceptionActive bit since we're entering an exception handler
    set_sr_bit(StatusRegisterFields::ExceptionActive, registers);

    registers.set_full_pc_address(vector_address);
    eu_registers.current_exception_level = exception_level;
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
        command: u16,
        registers: &'a mut Registers,
        eu_registers: &'a mut ExceptionUnitRegisters,
        bus_assertions: BusAssertions,
    ) -> BusAssertions {
        trace!(
            "PHASE '{phase:?}' :: FAULT '{:?}'",
            eu_registers.pending_fault
        );

        let instruction = decode_exception_unit_instruction(command);
        let (op_code, vector, level) = deconstruct_exception_unit_command(command);

        trace!("op_code: {op_code:?} vector: 0x{vector:X} level: 0x{level:X}");

        // Only handler entry reads the vector table. This decision remains
        // stable through fetch phases/stalls: command and exception level are
        // unchanged until execution in phase 3. Control/transfer operations,
        // ignored traps and L5 conflicts retain their slot with idle bus phases.
        let fetches_vector = matches!(op_code, ExceptionUnitOpCodes::Reset)
            || (matches!(
                op_code,
                ExceptionUnitOpCodes::Fault
                    | ExceptionUnitOpCodes::HardwareException
                    | ExceptionUnitOpCodes::SoftwareException
            ) && accepts_exception_entry(eu_registers.current_exception_level, level));

        match phase {
            ExecutionPhase::InstructionFetchLow if fetches_vector => {
                self.vector_address = vector as u32 * INSTRUCTION_SIZE_WORDS;
                return BusAssertions {
                    address: self.vector_address,
                    op: BusOperation::Read,
                    bus_access_strobe: true,
                    bus_access_type: BusAccessType::ExceptionVectorFetch,
                    ..BusAssertions::default()
                };
            }
            ExecutionPhase::InstructionFetchHigh if fetches_vector => {
                self.vector_value = (bus_assertions.data as u32) << u16::BITS;
                return BusAssertions {
                    address: self.vector_address + 1,
                    op: BusOperation::Read,
                    bus_access_strobe: true,
                    bus_access_type: BusAccessType::ExceptionVectorFetch,
                    ..BusAssertions::default()
                };
            }
            ExecutionPhase::InstructionDecode if fetches_vector => {
                self.vector_value |= bus_assertions.data as u32;

                debug!("0x{:X}: {:?}", registers.get_full_pc_address(), op_code);
            }
            ExecutionPhase::ExecutionEffectiveAddressExecutor => {
                // TODO: Some of these may have to be spread into the other execution phases depending on things like memory access (e.g. to make it like the CPU - one op per phase)
                let current_exception_level: u8 = eu_registers.current_exception_level;

                // TODO: Sort these cases?
                match op_code {
                    ExceptionUnitOpCodes::None => {
                        // No-op
                    }
                    ExceptionUnitOpCodes::WaitForException => {
                        debug!("Setting waiting_for_exception flag. CPU will go into low power mode until an interrupt is triggered.");
                        eu_registers.waiting_for_exception = true;
                    }
                    ExceptionUnitOpCodes::ReturnFromException => {
                        // Store current windowed link register to PC and jump to vector
                        // TODO: Is it safe to use the current_exception_level here? Could that be user editable? Does that matter?
                        // TODO: Overflow error when current_exception_level is zero. What should actually happen on hardware if you try to RTE when you're not in an exception
                        let ExceptionLinkRegister {
                            return_address,
                            return_status_register,
                            saved_exception_level,
                        } = eu_registers.link_registers[(current_exception_level - 1) as usize];

                        registers.set_full_pc_address(return_address);
                        registers.sr = return_status_register;
                        eu_registers.current_exception_level = saved_exception_level;

                        // EA follows the restored level, even if ETTR edited the saved SR.
                        toggle_sr_bit(
                            StatusRegisterFields::ExceptionActive,
                            saved_exception_level != 0,
                            registers,
                        );
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

                        trace!("ETFR! register_select: {register_select} link_register: {link_register:X?} lri: {} | <-- Registers: a: [{:X}] r7: [{:X}]", current_exception_level - 1, registers.get_full_address_address(), registers.r7);
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

                        trace!("ETTR! register_select: {register_select} link_register: {link_register:X?} lri: {} | --> Registers: a: [{:X}] r7: [{:X}]", current_exception_level - 1, registers.get_full_address_address(), registers.r7);
                    }
                    // Software commands were privilege-checked by their issuing
                    // PU instruction before CSR write-back.
                    ExceptionUnitOpCodes::Fault
                    | ExceptionUnitOpCodes::HardwareException
                    | ExceptionUnitOpCodes::SoftwareException => {
                        handle_exception(
                            current_exception_level,
                            level,
                            registers,
                            eu_registers,
                            self.vector_value,
                        );
                    }
                }
            }
            ExecutionPhase::InstructionFetchLow
            | ExecutionPhase::InstructionFetchHigh
            | ExecutionPhase::InstructionDecode
            | ExecutionPhase::MemoryAccessExecutor => {}
            ExecutionPhase::WriteBackExecutor => {
                // For Fault ops, pending_fault was set before this cycle and has now been
                // dispatched at phase 3; clear it as cleanup.
                // For HardwareException/SoftwareException ops, pending_fault starts as None
                // but may have been raised by handle_exception mid-cycle (e.g. when a second
                // L5 interrupt arrives while already at level 6, triggering
                // LevelFiveInterruptConflict). In that case we must NOT clear it — the next
                // InstructionFetchLow needs to see it and dispatch the fault.
                if matches!(op_code, ExceptionUnitOpCodes::Fault) {
                    eu_registers.pending_fault = None;
                }
            }
        }

        BusAssertions::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registers::set_hardware_interrupt_enable;
    use crate::registers::sr_bit_is_set;

    #[test]
    fn l5_overrun_selection_is_read_only_until_dispatch_acceptance() {
        let registers = Registers::default();
        let mut eu = ExceptionUnitRegisters {
            pending_hardware_exceptions: 16,
            l5_overrun: true,
            ..Default::default()
        };
        let before = eu;
        let event = select_pending_event(&registers, &eu).unwrap();
        assert_eq!(
            select_pending_event(&registers, &eu),
            Some(PendingEvent {
                command: construct_exception_unit_command(
                    &ExceptionUnitOpCodes::Fault,
                    LEVEL_FIVE_HARDWARE_EXCEPTION_CONFLICT
                ),
                interrupt_mask: 0,
                l5_conflict: true,
            })
        );
        assert_eq!(eu, before);
        event.accept(&mut eu, 0x1234);
        assert!(!eu.l5_overrun);
        assert_eq!(eu.pending_hardware_exceptions, 16);
        assert_eq!(
            eu.pending_fault,
            Some(PendingFault {
                fault: Faults::LevelFiveInterruptConflict,
                pc: 0x1234
            })
        );
    }

    #[test]
    fn rete_derives_ea_from_restored_level_after_ettr() {
        for saved_level in [0, 2] {
            for saved_ea in [false, true] {
                let mut registers = Registers::default();
                let mut eu = ExceptionUnitRegisters {
                    current_exception_level: 3,
                    ..Default::default()
                };
                eu.link_registers[2].saved_exception_level = saved_level;
                let saved_sr = 0x0055
                    | if saved_ea {
                        StatusRegisterFields::ExceptionActive as u16
                    } else {
                        0
                    };
                registers.r7 = saved_sr;
                let mut executor = ExceptionUnitExecutor {
                    vector_address: 0,
                    vector_value: 0,
                };
                // ETTR #2, r7 edits the saved status without changing its level.
                executor.step(
                    &ExecutionPhase::ExecutionEffectiveAddressExecutor,
                    0x1d22,
                    &mut registers,
                    &mut eu,
                    BusAssertions::default(),
                );
                assert_eq!(eu.link_registers[2].return_status_register, saved_sr);
                executor.step(
                    &ExecutionPhase::ExecutionEffectiveAddressExecutor,
                    0x1a00,
                    &mut registers,
                    &mut eu,
                    BusAssertions::default(),
                );
                assert_eq!(eu.current_exception_level, saved_level);
                assert_eq!(
                    registers.sr & !(StatusRegisterFields::ExceptionActive as u16),
                    0x0055
                );
                assert_eq!(
                    sr_bit_is_set(StatusRegisterFields::ExceptionActive, &registers),
                    saved_level != 0
                );
            }
        }
    }

    // IB-3 (docs/reference/review/implementation-bugs.md): dispatch used to only ever look at
    // the single highest-priority *pending* interrupt line. If that one line's enable bit had
    // been cleared since it was latched, dispatch gave up for the whole cycle instead of
    // considering any other pending line -- even a lower-priority one that was still enabled.
    // Line 4 (bit 3, the highest-priority maskable line) and line 1 (bit 0, the lowest) are both
    // latched as pending; only line 1 is enabled -- as if line 4's enable bit was cleared after
    // it was latched but before its turn to dispatch.
    const LINE_1_AND_LINE_4_PENDING: u8 = 0b0_1001;
    const LINE_4_PENDING_ONLY: u8 = 0b0_1000;

    #[test]
    fn masked_high_priority_pending_interrupt_does_not_block_a_lower_priority_one() {
        let mut registers = Registers::default();
        let mut eu_registers = ExceptionUnitRegisters {
            pending_hardware_exceptions: LINE_1_AND_LINE_4_PENDING,
            ..Default::default()
        };
        set_hardware_interrupt_enable(&mut registers, 0b0001);

        let event = select_pending_event(&registers, &eu_registers).unwrap();
        assert_eq!(
            select_pending_event(&registers, &eu_registers).as_ref(),
            Some(&event),
            "repeated selection must not consume the candidate"
        );
        assert_eq!(
            eu_registers.pending_hardware_exceptions,
            LINE_1_AND_LINE_4_PENDING
        );
        let command = event.accept(&mut eu_registers, 0);
        let (op_code, vector, _level) = deconstruct_exception_unit_command(command);

        assert_eq!(op_code, ExceptionUnitOpCodes::HardwareException);
        assert_eq!(
            vector,
            hardware_exception_level_to_vector(1),
            "line 1 (the only enabled pending line) should be dispatched"
        );
        assert_eq!(
            eu_registers.pending_hardware_exceptions, LINE_4_PENDING_ONLY,
            "only line 1's pending bit should be cleared; line 4 stays latched, still masked"
        );
    }

    #[test]
    fn nothing_dispatches_when_every_pending_line_is_masked() {
        let mut registers = Registers::default();
        let eu_registers = ExceptionUnitRegisters {
            pending_hardware_exceptions: LINE_1_AND_LINE_4_PENDING,
            ..Default::default()
        };
        set_hardware_interrupt_enable(&mut registers, 0b0000);

        assert_eq!(select_pending_event(&registers, &eu_registers), None);
        assert_eq!(
            eu_registers.pending_hardware_exceptions, LINE_1_AND_LINE_4_PENDING,
            "both lines stay latched, since neither is enabled"
        );
    }

    #[test]
    fn eu_selection_does_not_return_a_software_command() {
        let command =
            construct_exception_unit_command(&ExceptionUnitOpCodes::ReturnFromException, 0);
        let registers = Registers {
            coprocessor_select_register: command,
            ..Default::default()
        };
        let eu_registers = ExceptionUnitRegisters::default();
        assert_eq!(select_pending_event(&registers, &eu_registers), None);
        assert_eq!(registers.coprocessor_select_register, command);
    }

    #[test]
    fn fault_selection_and_acceptance_preserve_fault_metadata_and_interrupts() {
        let command =
            construct_exception_unit_command(&ExceptionUnitOpCodes::ReturnFromException, 0);
        let mut registers = Registers {
            coprocessor_select_register: command,
            ..Default::default()
        };
        set_hardware_interrupt_enable(&mut registers, 0b1001);
        let mut eu_registers = ExceptionUnitRegisters {
            pending_fault: Some(PendingFault {
                fault: Faults::Bus,
                pc: 0x1234,
            }),
            pending_hardware_exceptions: LINE_1_AND_LINE_4_PENDING,
            ..Default::default()
        };
        let event = select_pending_event(&registers, &eu_registers).unwrap();
        assert_eq!(
            event,
            select_pending_event(&registers, &eu_registers).unwrap()
        );
        let selected_command = event.accept(&mut eu_registers, 0);
        assert_eq!(
            selected_command,
            construct_exception_unit_command(&ExceptionUnitOpCodes::Fault, BUS_FAULT)
        );
        let fault = eu_registers.pending_fault.unwrap();
        assert!(matches!(fault.fault, Faults::Bus));
        assert_eq!(fault.pc, 0x1234);
        assert_eq!(
            eu_registers.pending_hardware_exceptions,
            LINE_1_AND_LINE_4_PENDING
        );
        assert_eq!(registers.coprocessor_select_register, command);
    }

    #[test]
    fn deferred_interrupt_is_reselected_using_current_masks_and_priority() {
        let mut registers = Registers::default();
        let mut eu_registers = ExceptionUnitRegisters {
            pending_hardware_exceptions: LINE_1_AND_LINE_4_PENDING,
            ..Default::default()
        };
        set_hardware_interrupt_enable(&mut registers, 0b1001);
        let event = select_pending_event(&registers, &eu_registers).unwrap();
        assert_eq!(event.interrupt_mask, LINE_4_PENDING_ONLY);
        // No dispatch was granted. Discard the candidate; do not carry a stale
        // decision across a boundary where masking or exception level can change.
        set_hardware_interrupt_enable(&mut registers, 0b0001);
        eu_registers.current_exception_level = ExceptionPriorities::LevelOneHardware as u8;
        assert_eq!(select_pending_event(&registers, &eu_registers), None);
        eu_registers.current_exception_level = ExceptionPriorities::NoException as u8;
        let event = select_pending_event(&registers, &eu_registers).unwrap();
        let selected_command = event.accept(&mut eu_registers, 0);
        assert_eq!(
            selected_command,
            construct_exception_unit_command(
                &ExceptionUnitOpCodes::HardwareException,
                hardware_exception_level_to_vector(1),
            )
        );
        assert_eq!(
            eu_registers.pending_hardware_exceptions,
            LINE_4_PENDING_ONLY
        );
    }

    #[test]
    fn validates_operation_nibbles_without_interpreting_reserved_payloads() {
        for op in 0u16..=15 {
            for payload in 0..=255 {
                let command = 0x1000 | (op << 8) | payload;
                assert_eq!(
                    is_valid_exception_unit_command(command),
                    !(2..=8).contains(&op)
                );
            }
        }
    }
}
