use peripheral_bus::{
    conversion::bytes_to_words,
    device::{BusAssertions, Device},
};
use peripheral_cpu::{
    coprocessors::{
        exception_unit::{
            definitions::{
                vectors::{BUS_FAULT, INSTRUCTION_TRACE_FAULT, INVALID_OPCODE_FAULT},
                ExceptionUnitOpCodes, Faults, PendingFault,
            },
            execution::{construct_exception_unit_command, hardware_exception_level_to_vector},
        },
        processing_unit::{
            definitions::{
                ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
                RegisterInstructionData, ShiftOperand, ShiftType,
            },
            encoding::encode_instruction,
        },
    },
    new_cpu_peripheral,
    registers::{set_hardware_interrupt_enable, RegisterName, StatusRegisterFields},
    CpuPeripheral,
};

fn tick(cpu: &mut CpuPeripheral, data: u16, interrupt: u8) -> BusAssertions {
    cpu.poll(
        BusAssertions {
            data,
            interrupt_assertion: interrupt,
            bus_acknowledge: true,
            ..BusAssertions::default()
        },
        true,
    )
}

fn cop_instruction(
    cpu: &mut CpuPeripheral,
    command: u16,
    register_call: bool,
    condition_flag: ConditionFlags,
) -> InstructionData {
    if register_call {
        cpu.registers.r1 = command;
        InstructionData::Register(RegisterInstructionData {
            op_code: Instruction::CoprocessorCallRegister,
            r1: 0,
            r2: 0,
            r3: RegisterName::R1.to_register_index(),
            shift_operand: ShiftOperand::Immediate,
            shift_type: ShiftType::None,
            shift_count: 0,
            condition_flag,
            additional_flags: 0,
        })
    } else {
        InstructionData::Immediate(ImmediateInstructionData {
            op_code: Instruction::CoprocessorCallImmediate,
            register: 0,
            value: command,
            condition_flag,
            additional_flags: 0,
        })
    }
}

fn issue(cpu: &mut CpuPeripheral, command: u16, interrupt: u8, register_call: bool) {
    let instruction = cop_instruction(cpu, command, register_call, ConditionFlags::Always);
    let words = bytes_to_words(&encode_instruction(&instruction));
    cpu.registers.pl = 0x0200;
    for (phase, data) in [0, words[0], words[1], 0, 0, 0].into_iter().enumerate() {
        tick(cpu, data, if phase == 5 { interrupt } else { 0 });
    }
    assert_eq!(cpu.phase, 0);
    assert_eq!(
        cpu.registers.coprocessor_select_register,
        if command & 0xf000 == 0 { 0 } else { command }
    );
}

fn transfer_command() -> u16 {
    // ETFR r7, #0: a visible command effect without changing interrupt masks.
    construct_exception_unit_command(&ExceptionUnitOpCodes::TransferFromRegister, 0x20)
}

#[test]
fn copi_and_copr_complete_once_before_writeback_interrupts() {
    for register_call in [false, true] {
        for (interrupt, line) in [(0b00001, 1), (0b10000, 5)] {
            let mut cpu = new_cpu_peripheral();
            set_hardware_interrupt_enable(&mut cpu.registers, 0b1111);
            cpu.eu_registers.link_registers[0].return_status_register = 0xCAFE;
            let command = transfer_command();
            issue(&mut cpu, command, interrupt, register_call);
            assert_eq!(cpu.registers.r7, 0);
            for phase in 0..6 {
                tick(&mut cpu, 0, 0);
                assert_eq!(cpu.active_command, command);
                assert_eq!(cpu.eu_registers.pending_hardware_exceptions, interrupt);
                assert_eq!(
                    cpu.registers.coprocessor_select_register,
                    if phase == 5 { 0 } else { command }
                );
            }
            assert_eq!(cpu.registers.r7, 0xCAFE);
            cpu.registers.r7 = 0x1234; // A repeated ETFR would overwrite this.
            let vector = hardware_exception_level_to_vector(line);
            let response = tick(&mut cpu, 0, 0);
            assert_eq!(
                cpu.active_command,
                construct_exception_unit_command(&ExceptionUnitOpCodes::HardwareException, vector)
            );
            assert_eq!(response.address, u32::from(vector) * 2);
            assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 0);
            for _ in 1..6 {
                tick(&mut cpu, 0, 0);
            }
            tick(&mut cpu, 0, 0);
            assert_eq!(cpu.active_command, 0);
            assert_eq!(cpu.registers.r7, 0x1234);
        }
    }
}

#[test]
fn rete_completes_before_interrupt_selection_using_restored_masks() {
    for enabled_on_return in [false, true] {
        let mut cpu = new_cpu_peripheral();
        set_hardware_interrupt_enable(&mut cpu.registers, 0b0001);
        cpu.eu_registers.current_exception_level = 1;
        cpu.eu_registers.link_registers[0].return_address = 0x0800;
        cpu.eu_registers.link_registers[0].return_status_register =
            if enabled_on_return { 0x0200 } else { 0 };
        let command =
            construct_exception_unit_command(&ExceptionUnitOpCodes::ReturnFromException, 0);
        issue(&mut cpu, command, 1, false);
        for _ in 0..6 {
            tick(&mut cpu, 0, 0);
            assert_eq!(cpu.active_command, command);
            assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
        }
        assert_eq!(cpu.eu_registers.current_exception_level, 0);
        assert_eq!(cpu.registers.pl, 0x0800);
        assert_eq!(cpu.registers.coprocessor_select_register, 0);
        let response = tick(&mut cpu, 0, 0);
        if enabled_on_return {
            assert_eq!(
                response.address,
                u32::from(hardware_exception_level_to_vector(1)) * 2
            );
            assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 0);
        } else {
            assert_eq!(response.address, 0x0800);
            assert_eq!(cpu.active_command, 0);
            assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
        }
    }
}

#[test]
fn rete_rechecks_restored_exception_level_before_taking_a_queued_interrupt() {
    let mut cpu = new_cpu_peripheral();
    set_hardware_interrupt_enable(&mut cpu.registers, 1);
    cpu.eu_registers.current_exception_level = 3;
    cpu.eu_registers.pending_hardware_exceptions = 1;
    cpu.eu_registers.link_registers[2].saved_exception_level = 2;
    cpu.eu_registers.link_registers[2].return_address = 0x0800;
    cpu.eu_registers.link_registers[2].return_status_register =
        StatusRegisterFields::HardwareInterruptEnable1 as u16;
    let command = construct_exception_unit_command(&ExceptionUnitOpCodes::ReturnFromException, 0);
    issue(&mut cpu, command, 1, false);
    for _ in 0..6 {
        tick(&mut cpu, 0, 0);
    }
    let response = tick(&mut cpu, 0, 0);
    assert_eq!(cpu.eu_registers.current_exception_level, 2);
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
    assert_eq!(response.address, 0x0800);
    assert_eq!(cpu.active_command, 0);
}

#[test]
fn command_bus_stalls_keep_csr_and_interrupt_pending_until_completion() {
    let mut cpu = new_cpu_peripheral();
    set_hardware_interrupt_enable(&mut cpu.registers, 1);
    let command = construct_exception_unit_command(&ExceptionUnitOpCodes::SoftwareException, 0x60);
    issue(&mut cpu, command, 1, false);
    let request = tick(&mut cpu, 0, 0);
    for _ in 0..3 {
        let stalled = cpu.poll(BusAssertions::default(), true);
        assert_eq!(stalled.address, request.address);
        assert_eq!(cpu.phase, 1);
        assert_eq!(cpu.active_command, command);
        assert_eq!(cpu.registers.coprocessor_select_register, command);
        assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
    }
    for _ in 1..6 {
        tick(&mut cpu, 0, 0);
    }
    assert_eq!(cpu.phase, 0);
    assert_eq!(cpu.registers.coprocessor_select_register, 0);
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
}

#[test]
fn missing_command_fault_cancels_csr_and_preserves_the_waiting_interrupt() {
    let mut cpu = new_cpu_peripheral();
    set_hardware_interrupt_enable(&mut cpu.registers, 1);
    issue(&mut cpu, 0x3000, 1, false); // Missing maths unit.
    tick(&mut cpu, 0, 0);
    let fault = cpu.eu_registers.pending_fault.unwrap();
    assert!(matches!(fault.fault, Faults::InvalidOpCode));
    assert_eq!(fault.pc, 0x0200);
    assert_eq!(cpu.eu_registers.link_registers[7].return_address, 0x3000);
    assert_eq!(cpu.registers.coprocessor_select_register, 0);
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
    for _ in 1..6 {
        tick(&mut cpu, 0, 0);
    }
    let response = tick(&mut cpu, 0, 0);
    assert_eq!(response.address, u32::from(INVALID_OPCODE_FAULT) * 2);
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
}

#[test]
fn automatic_eu_dispatch_does_not_consume_an_unrelated_csr_command() {
    let mut cpu = new_cpu_peripheral();
    set_hardware_interrupt_enable(&mut cpu.registers, 1);
    cpu.eu_registers.pending_hardware_exceptions = 1;
    tick(&mut cpu, 0, 0);
    // Probe cleanup ownership: this queued command does not belong to the
    // automatic EU event whose slot is already running.
    let command = transfer_command();
    cpu.registers.coprocessor_select_register = command;
    for _ in 1..6 {
        tick(&mut cpu, 0, 0);
    }
    assert_eq!(cpu.registers.coprocessor_select_register, command);
    tick(&mut cpu, 0, 0);
    assert_eq!(cpu.active_command, command);
}

#[test]
fn id_zero_calls_finish_in_six_cycles_with_empty_csr_and_normal_event_selection() {
    for operation in 0..=15 {
        for payload in [0, 0x23, 0xff] {
            for register_call in [false, true] {
                for condition in [ConditionFlags::Always, ConditionFlags::Never] {
                    // 0: no trace, 1: SR.T, 2: TRCE pulse at instruction start.
                    for trace in 0..=2 {
                        for interrupt in [0, 1] {
                            let mut cpu = new_cpu_peripheral();
                            set_hardware_interrupt_enable(&mut cpu.registers, 1);
                            if trace == 1 {
                                peripheral_cpu::registers::set_sr_bit(
                                    StatusRegisterFields::TraceMode,
                                    &mut cpu.registers,
                                );
                            }
                            cpu.registers.pl = 0x0200;
                            cpu.registers.r2 = 0xcafe;
                            let command = (operation << 8) | payload;
                            let instruction =
                                cop_instruction(&mut cpu, command, register_call, condition);
                            let words = bytes_to_words(&encode_instruction(&instruction));
                            let mut expected_registers = cpu.registers;
                            expected_registers.pl = 0x0202;
                            for (phase, data) in
                                [0, words[0], words[1], 0, 0, 0].into_iter().enumerate()
                            {
                                let out = cpu.poll(
                                    BusAssertions {
                                        data,
                                        bus_acknowledge: true,
                                        force_trace_mode: trace == 2 && phase == 0,
                                        interrupt_assertion: if phase == 5 { interrupt } else { 0 },
                                        ..Default::default()
                                    },
                                    true,
                                );
                                assert_eq!(out.bus_access_strobe, phase < 2);
                                if phase < 2 {
                                    assert_eq!(out.address, 0x0200 + u32::try_from(phase).unwrap());
                                }
                                assert_eq!(cpu.phase, u8::try_from((phase + 1) % 6).unwrap());
                            }
                            // Check before polling the next boundary: no stale ID-0 CSR.
                            assert_eq!(cpu.registers, expected_registers);
                            assert_eq!(cpu.eu_registers.pending_hardware_exceptions, interrupt);
                            if trace != 0 {
                                let fault = cpu.eu_registers.pending_fault.unwrap();
                                assert_eq!(fault.fault, Faults::InstructionTrace);
                                assert_eq!(fault.pc, 0x0202);
                            } else {
                                assert!(cpu.eu_registers.pending_fault.is_none());
                            }
                            if trace != 0 || interrupt != 0 {
                                let vector = if trace != 0 {
                                    INSTRUCTION_TRACE_FAULT
                                } else {
                                    hardware_exception_level_to_vector(1)
                                };
                                super::common::complete_entry(&mut cpu, vector, 0x1000);
                                assert_eq!(
                                    cpu.eu_registers.pending_hardware_exceptions,
                                    if trace != 0 { interrupt } else { 0 }
                                );
                            } else {
                                let fetch = tick(&mut cpu, 0, 0);
                                assert!(fetch.bus_access_strobe);
                                assert_eq!(fetch.address, 0x0202);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn already_pending_interrupt_does_not_deadlock_wait() {
    let mut cpu = new_cpu_peripheral();
    set_hardware_interrupt_enable(&mut cpu.registers, 1);
    let command = construct_exception_unit_command(&ExceptionUnitOpCodes::WaitForException, 0);
    issue(&mut cpu, command, 1, false);
    for _ in 0..6 {
        tick(&mut cpu, 0, 0);
    }
    assert_eq!(cpu.phase, 0);
    assert_eq!(cpu.registers.coprocessor_select_register, 0);
    assert!(!cpu.eu_registers.waiting_for_exception);
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
    let response = tick(&mut cpu, 0, 0);
    assert_eq!(
        response.address,
        u32::from(hardware_exception_level_to_vector(1)) * 2
    );
}

#[test]
fn command_vector_bus_fault_cancels_csr_without_consuming_the_interrupt() {
    for protection_error in [false, true] {
        let mut cpu = new_cpu_peripheral();
        set_hardware_interrupt_enable(&mut cpu.registers, 1);
        issue(
            &mut cpu,
            construct_exception_unit_command(&ExceptionUnitOpCodes::SoftwareException, 0x60),
            1,
            false,
        );
        let request = tick(&mut cpu, 0, 0);
        assert!(request.bus_access_strobe);
        cpu.poll(
            BusAssertions {
                bus_error: !protection_error,
                bus_protection_error: protection_error,
                ..BusAssertions::default()
            },
            true,
        );
        assert_eq!(cpu.registers.coprocessor_select_register, 0);
        assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
        let fault = cpu.eu_registers.pending_fault.unwrap().fault;
        assert_eq!(
            fault,
            if protection_error {
                Faults::BusProtection
            } else {
                Faults::Bus
            }
        );
        tick(&mut cpu, 0, 0);
        assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
    }
}

#[test]
fn existing_fault_and_trace_priority_still_cancels_a_waiting_command() {
    for (fault, vector) in [
        (Faults::Bus, BUS_FAULT),
        (Faults::InstructionTrace, INSTRUCTION_TRACE_FAULT),
    ] {
        let mut cpu = new_cpu_peripheral();
        set_hardware_interrupt_enable(&mut cpu.registers, 1);
        cpu.registers.coprocessor_select_register = transfer_command();
        cpu.eu_registers.pending_hardware_exceptions = 1;
        cpu.eu_registers.pending_fault = Some(PendingFault { fault, pc: 0x1234 });
        tick(&mut cpu, 0, 0);
        assert_eq!(
            cpu.active_command,
            construct_exception_unit_command(&ExceptionUnitOpCodes::Fault, vector)
        );
        assert_eq!(cpu.registers.coprocessor_select_register, 0);
        assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
        assert_eq!(cpu.eu_registers.pending_fault.unwrap().pc, 0x1234);
        assert_eq!(cpu.registers.r7, 0);
    }
}

#[test]
fn wait_retains_command_ownership_while_idle_then_finishes_before_interrupt_entry() {
    let mut cpu = new_cpu_peripheral();
    set_hardware_interrupt_enable(&mut cpu.registers, 1);
    let command = construct_exception_unit_command(&ExceptionUnitOpCodes::WaitForException, 0);
    issue(&mut cpu, command, 0, false);
    for _ in 0..4 {
        tick(&mut cpu, 0, 0);
    }
    assert!(cpu.eu_registers.waiting_for_exception);
    for _ in 0..3 {
        let response = tick(&mut cpu, 0, 0);
        assert!(!response.bus_access_strobe);
        assert_eq!(cpu.phase, 4);
        assert_eq!(cpu.registers.coprocessor_select_register, command);
    }
    tick(&mut cpu, 0, 1);
    assert_eq!(cpu.phase, 5);
    assert_eq!(cpu.registers.coprocessor_select_register, command);
    tick(&mut cpu, 0, 0);
    assert_eq!(cpu.phase, 0);
    assert_eq!(cpu.registers.coprocessor_select_register, 0);
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
    let response = tick(&mut cpu, 0, 0);
    assert_eq!(
        response.address,
        u32::from(hardware_exception_level_to_vector(1)) * 2
    );
}

#[test]
fn wait_uses_dispatch_priority_and_enable_rules_for_pending_and_new_irqs() {
    for current_level in 0..=7 {
        for line in 0..4 {
            for enabled in [false, true] {
                for already_pending in [false, true] {
                    let mut cpu = new_cpu_peripheral();
                    let command = construct_exception_unit_command(
                        &ExceptionUnitOpCodes::WaitForException,
                        0,
                    );
                    issue(&mut cpu, command, 0, false);
                    cpu.eu_registers.current_exception_level = current_level;
                    let pin = 1 << line;
                    set_hardware_interrupt_enable(
                        &mut cpu.registers,
                        if enabled { pin } else { 0 },
                    );
                    if already_pending {
                        cpu.eu_registers.pending_hardware_exceptions = pin;
                    }
                    for _ in 0..4 {
                        tick(&mut cpu, 0, 0);
                    }
                    assert!(cpu.eu_registers.waiting_for_exception);
                    let eligible = enabled && line + 2 > current_level;
                    for _ in 0..3 {
                        let response = tick(&mut cpu, 0, if already_pending { 0 } else { pin });
                        if eligible {
                            break;
                        }
                        assert!(!response.bus_access_strobe);
                        assert_eq!(cpu.phase, 4);
                        assert!(cpu.eu_registers.waiting_for_exception);
                        assert_eq!(cpu.registers.coprocessor_select_register, command);
                    }
                    if eligible {
                        assert!(!cpu.eu_registers.waiting_for_exception);
                        assert_eq!(cpu.phase, 5);
                        // Waking does not accept the interrupt or retire WAIT early.
                        assert_eq!(cpu.eu_registers.pending_hardware_exceptions, pin);
                        tick(&mut cpu, 0, 0);
                        super::common::complete_entry(
                            &mut cpu,
                            hardware_exception_level_to_vector(line + 1),
                            0x1000,
                        );
                        assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 0);
                    } else {
                        assert_eq!(
                            cpu.eu_registers.pending_hardware_exceptions,
                            if already_pending { pin } else { 0 }
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn wait_l5_wake_preserves_level_seven_queue_and_level_six_conflicts() {
    use peripheral_cpu::coprocessors::exception_unit::definitions::vectors::LEVEL_FIVE_HARDWARE_EXCEPTION_CONFLICT;
    for current_level in [0, 6, 7] {
        for overrun in [false, true] {
            let mut cpu = new_cpu_peripheral();
            let command =
                construct_exception_unit_command(&ExceptionUnitOpCodes::WaitForException, 0);
            issue(&mut cpu, command, 0, false);
            cpu.eu_registers.current_exception_level = current_level;
            for _ in 0..4 {
                tick(&mut cpu, 0, 0);
            }
            // A fresh edge must queue at level 7 without waking WAIT.
            if overrun {
                cpu.eu_registers.pending_hardware_exceptions = 16;
            }
            tick(&mut cpu, 0, 16);
            assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 16);
            assert_eq!(cpu.eu_registers.l5_overrun, overrun);
            if current_level == 7 {
                assert!(cpu.eu_registers.waiting_for_exception);
                for _ in 0..3 {
                    assert!(!tick(&mut cpu, 0, 16).bus_access_strobe);
                    assert_eq!(cpu.phase, 4);
                }
                assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 16);
                assert_eq!(cpu.eu_registers.l5_overrun, overrun);
                // Once dispatch eligibility changes, the retained request wakes WAIT.
                cpu.eu_registers.current_exception_level = 6;
                tick(&mut cpu, 0, 16);
            }
            assert!(!cpu.eu_registers.waiting_for_exception);
            assert_eq!(cpu.phase, 5);
            tick(&mut cpu, 0, 16);
            let conflict = current_level >= 6 || overrun;
            let vector = if conflict {
                LEVEL_FIVE_HARDWARE_EXCEPTION_CONFLICT
            } else {
                hardware_exception_level_to_vector(5)
            };
            super::common::complete_entry(&mut cpu, vector, 0x1000);
            assert!(!cpu.eu_registers.l5_overrun);
            assert_eq!(
                cpu.eu_registers.pending_hardware_exceptions,
                if current_level == 0 && overrun { 16 } else { 0 }
            );
        }
    }
}

fn assert_invalid_command(cpu: &mut CpuPeripheral, command: u16) {
    // Input BAT/address belong to the previous bus cycle, not this non-access fault.
    let response = cpu.poll(
        BusAssertions {
            address: 0xfeed,
            bus_access_type: peripheral_bus::device::BusAccessType::DataRead,
            bus_acknowledge: true,
            ..Default::default()
        },
        true,
    );
    assert!(!response.bus_access_strobe);
    let fault = cpu.eu_registers.pending_fault.unwrap();
    assert_eq!(fault.fault, Faults::InvalidOpCode);
    assert_eq!(fault.pc, 0x0200);
    assert_eq!(
        cpu.eu_registers.link_registers[7].return_address,
        u32::from(command)
    );
    let metadata = peripheral_cpu::decode_fault_metadata_register(
        cpu.eu_registers.link_registers[7].return_status_register,
    );
    assert_eq!(
        metadata.bus_access_type,
        peripheral_bus::device::BusAccessType::None
    );
    assert_eq!(cpu.registers.coprocessor_select_register, 0);
    for _ in 1..6 {
        assert!(!tick(cpu, 0, 0).bus_access_strobe);
    }
    super::common::complete_entry(cpu, INVALID_OPCODE_FAULT, 0x1000);
    assert_eq!(cpu.eu_registers.link_registers[6].return_address, 0x0200);
}

#[test]
fn reserved_eu_operations_fault_without_accessing_their_payload_vectors() {
    for operation in 2..=8 {
        for payload in [0, 0xff] {
            for register_call in [false, true] {
                let mut cpu = new_cpu_peripheral();
                // A command fault must also cancel any deferred trace.
                peripheral_cpu::registers::set_sr_bit(
                    StatusRegisterFields::TraceMode,
                    &mut cpu.registers,
                );
                let command = 0x1000 | (operation << 8) | payload;
                issue(&mut cpu, command, 0, register_call);
                assert_invalid_command(&mut cpu, command);
            }
        }
    }
    // Absent units retain the same software-emulation path.
    for command in [0x2000, 0x30ff] {
        let mut cpu = new_cpu_peripheral();
        issue(&mut cpu, command, 0, false);
        assert_invalid_command(&mut cpu, command);
    }
}

#[test]
fn invalid_transfer_indices_fault_before_modifying_registers_or_links() {
    for operation in [0x1c00, 0x1d00] {
        for selection in 0..=3 {
            for index in 8..=15 {
                for register_call in [false, true] {
                    let mut cpu = new_cpu_peripheral();
                    let command = operation | (selection << 4) | index;
                    cpu.registers.ah = 0x1234;
                    cpu.registers.al = 0x5678;
                    cpu.registers.r7 = 0xabcd;
                    // Transfer tracing must also be safe outside a handler (level 0).
                    for (i, link) in cpu.eu_registers.link_registers.iter_mut().enumerate() {
                        link.return_address = u32::try_from(i).unwrap() * 0x100;
                        link.return_status_register = 0x4321;
                    }
                    issue(&mut cpu, command, 0, register_call);
                    let mut expected_registers = cpu.registers;
                    expected_registers.coprocessor_select_register = 0;
                    let links = cpu.eu_registers.link_registers;
                    assert_invalid_command(&mut cpu, command);
                    // Fault entry changes PC/SR and LR6/LR7; no transfer may run.
                    assert_eq!(cpu.registers.ah, expected_registers.ah);
                    assert_eq!(cpu.registers.al, expected_registers.al);
                    assert_eq!(cpu.registers.r7, expected_registers.r7);
                    assert_eq!(&cpu.eu_registers.link_registers[..6], &links[..6]);
                }
            }
        }
    }
}

#[test]
fn legal_transfer_index_boundaries_keep_timing_and_selected_fields() {
    for operation in [0x1c00, 0x1d00] {
        for selection in 0..=3 {
            for index in [0u16, 7] {
                for register_call in [false, true] {
                    let mut cpu = new_cpu_peripheral();
                    cpu.registers.ah = 0x12;
                    cpu.registers.al = 0x3456;
                    cpu.registers.r7 = 0xabcd;
                    let link = &mut cpu.eu_registers.link_registers[usize::from(index)];
                    link.return_address = 0x0065_4321;
                    link.return_status_register = 0x5678;
                    let command = operation | (selection << 4) | index;
                    issue(&mut cpu, command, 0, register_call);
                    for _ in 0..6 {
                        assert!(!tick(&mut cpu, 0, 0).bus_access_strobe);
                        assert!(cpu.eu_registers.pending_fault.is_none());
                    }
                    assert_eq!(cpu.phase, 0);
                    assert_eq!(cpu.registers.pl, 0x0202);
                    assert_eq!(cpu.registers.coprocessor_select_register, 0);
                    let link = cpu.eu_registers.link_registers[usize::from(index)];
                    let from_link = operation == 0x1c00;
                    assert_eq!(
                        cpu.registers.ah,
                        if from_link && selection & 1 != 0 {
                            0x65
                        } else {
                            0x12
                        }
                    );
                    assert_eq!(
                        cpu.registers.al,
                        if from_link && selection & 1 != 0 {
                            0x4321
                        } else {
                            0x3456
                        }
                    );
                    assert_eq!(
                        cpu.registers.r7,
                        if from_link && selection & 2 != 0 {
                            0x5678
                        } else {
                            0xabcd
                        }
                    );
                    assert_eq!(
                        link.return_address,
                        if !from_link && selection & 1 != 0 {
                            0x0012_3456
                        } else {
                            0x0065_4321
                        }
                    );
                    assert_eq!(
                        link.return_status_register,
                        if !from_link && selection & 2 != 0 {
                            0xabcd
                        } else {
                            0x5678
                        }
                    );
                }
            }
        }
    }
}
