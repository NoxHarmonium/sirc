use peripheral_bus::device::{BusAccessType, BusAssertions, Device};
use peripheral_cpu::{
    coprocessors::exception_unit::{
        definitions::{
            vectors::{
                BUS_FAULT, BUS_PROTECTION_FAULT, DOUBLE_FAULT_VECTOR, INVALID_OPCODE_FAULT,
                LEVEL_ONE_HARDWARE_EXCEPTION,
            },
            ExceptionUnitOpCodes, Faults, PendingFault,
        },
        execution::construct_exception_unit_command,
    },
    new_cpu_peripheral,
    registers::{set_hardware_interrupt_enable, ExceptionLinkRegister, FullAddressRegisterAccess},
    CpuPeripheral,
};

fn tick(cpu: &mut CpuPeripheral, data: u16) -> BusAssertions {
    cpu.poll(
        BusAssertions {
            data,
            bus_acknowledge: true,
            ..Default::default()
        },
        true,
    )
}

#[test]
fn vector_errors_retire_idle_phases_before_the_replacement_fault_dispatch() {
    for software_command in [false, true] {
        for fault_dispatch in [false, true] {
            if software_command && fault_dispatch {
                continue;
            }
            for level in [0, 1] {
                // Traps are accepted only outside a handler.
                if software_command && level != 0 {
                    continue;
                }
                for failed_word in [0, 1] {
                    for protection_error in [false, true] {
                        for waits in [0, 3] {
                            let mut cpu = new_cpu_peripheral();
                            cpu.registers.set_full_pc_address(0x0034_0200);
                            cpu.registers.sr = 0x0055;
                            cpu.eu_registers.current_exception_level = level;
                            cpu.eu_registers.link_registers = [ExceptionLinkRegister {
                                return_address: 0x1122,
                                return_status_register: 0x3344,
                                saved_exception_level: 1,
                            }; 8];
                            let vector = if fault_dispatch {
                                cpu.eu_registers.pending_fault = Some(PendingFault {
                                    fault: Faults::InvalidOpCode,
                                    pc: 0x0034_0100,
                                });
                                INVALID_OPCODE_FAULT
                            } else if software_command {
                                cpu.registers.coprocessor_select_register =
                                    construct_exception_unit_command(
                                        &ExceptionUnitOpCodes::SoftwareException,
                                        0x60,
                                    );
                                0x60
                            } else {
                                set_hardware_interrupt_enable(&mut cpu.registers, 1);
                                cpu.eu_registers.pending_hardware_exceptions = 1;
                                LEVEL_ONE_HARDWARE_EXCEPTION
                            };
                            // A synthetic queued EXCP still needs its issuing decode.
                            cpu.processing_unit_executor.decoded_instruction.pc_h_ = 0x34;
                            cpu.processing_unit_executor.decoded_instruction.pc_l_ = 0x0200;
                            let original_registers = cpu.registers;
                            let original_links = cpu.eu_registers.link_registers;
                            let first = tick(&mut cpu, 0);
                            let request = if failed_word == 1 {
                                tick(&mut cpu, 0xabcd)
                            } else {
                                first
                            };
                            assert!(request.bus_access_strobe);
                            assert_eq!(request.address, u32::from(vector) * 2 + failed_word);
                            assert_eq!(
                                request.bus_access_type,
                                BusAccessType::ExceptionVectorFetch
                            );
                            let waiting_phase = cpu.phase;
                            for _ in 0..waits {
                                let held = cpu.poll(BusAssertions::default(), true);
                                assert_eq!(held.address, request.address);
                                assert!(held.bus_access_strobe);
                                assert_eq!(cpu.phase, waiting_phase);
                            }
                            let error_response = cpu.poll(
                                BusAssertions {
                                    data: 0xffff,
                                    bus_error: !protection_error,
                                    bus_protection_error: protection_error,
                                    // Error wins over BACK if both are asserted.
                                    bus_acknowledge: true,
                                    ..Default::default()
                                },
                                true,
                            );
                            assert!(!error_response.bus_access_strobe);
                            assert!(!error_response.instruction_sync);
                            assert!(!error_response.reset_requested);
                            assert_eq!(cpu.phase, waiting_phase + 1);
                            assert_eq!(
                                cpu.registers.get_full_pc_address(),
                                original_registers.get_full_pc_address()
                            );
                            assert_eq!(cpu.registers.sr, original_registers.sr);
                            assert_eq!(cpu.eu_registers.current_exception_level, level);
                            assert_eq!(cpu.eu_registers.link_registers[..7], original_links[..7]);
                            assert_eq!(cpu.registers.coprocessor_select_register, 0);
                            let replacement = cpu.eu_registers.pending_fault.unwrap();
                            assert_eq!(
                                replacement.fault,
                                if fault_dispatch {
                                    Faults::DoubleFault
                                } else if protection_error {
                                    Faults::BusProtection
                                } else {
                                    Faults::Bus
                                }
                            );
                            assert_eq!(
                                replacement.pc,
                                if fault_dispatch {
                                    0x0034_0100
                                } else {
                                    0x0034_0200
                                }
                            );
                            let registers_after_error = cpu.registers;
                            let eu_after_error = cpu.eu_registers;
                            // Even an error line lingering during idle phases cannot
                            // raise a second fault or clear the replacement at write-back.
                            for phase in cpu.phase..6 {
                                let idle = cpu.poll(
                                    BusAssertions {
                                        bus_error: true,
                                        force_trace_mode: true,
                                        ..Default::default()
                                    },
                                    true,
                                );
                                assert_eq!(cpu.phase, (phase + 1) % 6);
                                assert!(!idle.bus_access_strobe);
                                assert!(!idle.instruction_sync);
                                assert_eq!(cpu.registers, registers_after_error);
                                assert_eq!(cpu.eu_registers, eu_after_error);
                            }
                            let replacement_vector = if fault_dispatch {
                                DOUBLE_FAULT_VECTOR
                            } else if protection_error {
                                BUS_PROTECTION_FAULT
                            } else {
                                BUS_FAULT
                            };
                            let first = tick(&mut cpu, 0);
                            assert!(first.instruction_sync);
                            assert!(first.bus_access_strobe);
                            assert_eq!(first.address, u32::from(replacement_vector) * 2);
                            let second = tick(&mut cpu, 0x0012);
                            assert!(second.bus_access_strobe);
                            assert_eq!(second.address, first.address + 1);
                            for data in [0x3456, 0, 0, 0] {
                                assert!(!tick(&mut cpu, data).bus_access_strobe);
                            }
                            assert_eq!(cpu.phase, 0);
                            assert_eq!(cpu.registers.get_full_pc_address(), 0x0012_3456);
                            assert_eq!(cpu.eu_registers.current_exception_level, 7);
                            assert_eq!(
                                cpu.eu_registers.link_registers[6].return_address,
                                replacement.pc
                            );
                            assert!(cpu.eu_registers.pending_fault.is_none());
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn reset_can_restart_an_aborted_slot_immediately() {
    let mut cpu = new_cpu_peripheral();
    cpu.registers.coprocessor_select_register =
        construct_exception_unit_command(&ExceptionUnitOpCodes::SoftwareException, 0x60);
    tick(&mut cpu, 0);
    cpu.poll(
        BusAssertions {
            bus_error: true,
            ..Default::default()
        },
        true,
    );
    assert_eq!(cpu.phase, 2);
    cpu.reset();
    assert_eq!(cpu.phase, 0);
    assert!(cpu.eu_registers.pending_fault.is_none());
    let first = tick(&mut cpu, 0);
    assert_eq!(first.address, 0);
    assert!(first.bus_access_strobe);
    for data in [0, 0x0800, 0, 0, 0] {
        tick(&mut cpu, data);
    }
    assert_eq!(cpu.registers.pl, 0x0800);
    assert_eq!(cpu.phase, 0);
}

#[test]
fn failed_double_fault_vector_still_requests_reset_immediately() {
    for failed_word in [0, 1] {
        for protection_error in [false, true] {
            let mut cpu = new_cpu_peripheral();
            cpu.eu_registers.pending_fault = Some(PendingFault {
                fault: Faults::DoubleFault,
                pc: 0x0200,
            });
            tick(&mut cpu, 0);
            if failed_word == 1 {
                tick(&mut cpu, 0);
            }
            let response = cpu.poll(
                BusAssertions {
                    bus_error: !protection_error,
                    bus_protection_error: protection_error,
                    ..Default::default()
                },
                true,
            );
            assert!(response.reset_requested);
            assert!(!response.bus_access_strobe);
            assert!(cpu.eu_registers.pending_fault.is_none());
            assert!(tick(&mut cpu, 0).reset_requested);
        }
    }
}
