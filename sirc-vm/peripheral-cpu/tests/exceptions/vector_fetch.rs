use peripheral_bus::device::{BusAccessType, BusAssertions, BusOperation, Device};
use peripheral_cpu::{
    coprocessors::exception_unit::{
        definitions::{
            vectors::{
                BUS_FAULT, LEVEL_FIVE_HARDWARE_EXCEPTION_CONFLICT, LEVEL_ONE_HARDWARE_EXCEPTION,
            },
            ExceptionUnitOpCodes, Faults, PendingFault,
        },
        execution::construct_exception_unit_command,
    },
    new_cpu_peripheral,
    registers::{set_hardware_interrupt_enable, ExceptionLinkRegister, FullAddressRegisterAccess},
};

use super::common::tick;

#[test]
fn control_and_transfer_slots_never_touch_a_faulting_vector_table() {
    for operation in [
        ExceptionUnitOpCodes::ReturnFromException,
        ExceptionUnitOpCodes::TransferFromRegister,
        ExceptionUnitOpCodes::TransferToRegister,
        ExceptionUnitOpCodes::WaitForException,
        ExceptionUnitOpCodes::None,
    ] {
        for protection_error in [false, true] {
            let mut cpu = new_cpu_peripheral();
            cpu.registers.pl = 0x0200;
            cpu.eu_registers.current_exception_level = 1;
            cpu.eu_registers.link_registers[0] = ExceptionLinkRegister {
                return_address: 0x0400,
                return_status_register: 0x0055,
                saved_exception_level: 0,
            };
            cpu.registers.r7 = 0x0066;
            // WAIT must be able to finish its final two phases. This IRQ remains
            // queued until the following slot; no hardware dispatch runs here.
            set_hardware_interrupt_enable(&mut cpu.registers, 1);
            cpu.eu_registers.pending_hardware_exceptions = 1;
            let command = construct_exception_unit_command(&operation, 0x20);
            cpu.registers.coprocessor_select_register = command;
            let mut previous = BusAssertions::default();
            for phase in 0..6 {
                assert_eq!(cpu.phase, phase);
                // A vector device that rejects every access with BERR or BPER.
                // No BACK is supplied: bus-idle phases cannot wait for it.
                let vector_access = previous.bus_access_strobe
                    && previous.bus_access_type == BusAccessType::ExceptionVectorFetch;
                let response = cpu.poll(
                    BusAssertions {
                        bus_error: vector_access && !protection_error,
                        bus_protection_error: vector_access && protection_error,
                        ..Default::default()
                    },
                    true,
                );
                assert!(
                    !response.bus_access_strobe,
                    "operation {operation:?}, phase {phase}"
                );
                assert_eq!(cpu.active_command, command);
                assert_eq!(response.instruction_sync, phase == 0);
                assert!(cpu.eu_registers.pending_fault.is_none());
                previous = response;
            }
            assert_eq!(cpu.phase, 0);
            assert_eq!(cpu.registers.coprocessor_select_register, 0);
            assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
            match operation {
                ExceptionUnitOpCodes::ReturnFromException => {
                    assert_eq!(cpu.registers.pl, 0x0400);
                    assert_eq!(cpu.eu_registers.current_exception_level, 0);
                }
                ExceptionUnitOpCodes::TransferFromRegister => assert_eq!(cpu.registers.r7, 0x0055),
                ExceptionUnitOpCodes::TransferToRegister => assert_eq!(
                    cpu.eu_registers.link_registers[0].return_status_register,
                    0x0066
                ),
                _ => assert_eq!(cpu.registers.pl, 0x0200),
            }
        }
    }
}

#[test]
fn accepted_entries_fetch_exactly_two_ordered_words_and_hold_reads_through_stalls() {
    for operation in [
        ExceptionUnitOpCodes::SoftwareException,
        ExceptionUnitOpCodes::HardwareException,
        ExceptionUnitOpCodes::Fault,
        ExceptionUnitOpCodes::Reset,
    ] {
        let mut cpu = new_cpu_peripheral();
        cpu.registers.pl = 0x0200;
        let vector = match operation {
            ExceptionUnitOpCodes::SoftwareException => {
                cpu.registers.coprocessor_select_register =
                    construct_exception_unit_command(&operation, 0x60);
                0x60
            }
            ExceptionUnitOpCodes::HardwareException => {
                set_hardware_interrupt_enable(&mut cpu.registers, 1);
                cpu.eu_registers.pending_hardware_exceptions = 1;
                LEVEL_ONE_HARDWARE_EXCEPTION
            }
            ExceptionUnitOpCodes::Fault => {
                cpu.eu_registers.pending_fault = Some(PendingFault {
                    fault: Faults::Bus,
                    pc: 0x0120,
                });
                BUS_FAULT
            }
            ExceptionUnitOpCodes::Reset => {
                cpu.reset();
                0
            }
            _ => unreachable!(),
        };
        let first = tick(&mut cpu, 0);
        assert!(first.instruction_sync);
        let mut request = first;
        for (phase, word) in [(1, 0x0012), (2, 0x3456)] {
            assert_eq!(
                request.address,
                u32::from(vector) * 2 + u32::from(phase - 1)
            );
            assert!(request.bus_access_strobe);
            assert!(matches!(request.op, BusOperation::Read));
            assert_eq!(request.bus_access_type, BusAccessType::ExceptionVectorFetch);
            for _ in 0..3 {
                let held = cpu.poll(BusAssertions::default(), true);
                assert_eq!(cpu.phase, phase);
                assert_eq!(held.address, request.address);
                assert!(held.bus_access_strobe);
                assert_eq!(held.bus_access_type, BusAccessType::ExceptionVectorFetch);
            }
            let response = tick(&mut cpu, word);
            if phase == 1 {
                assert!(response.bus_access_strobe);
                assert_eq!(response.address, u32::from(vector) * 2 + 1);
            } else {
                assert!(!response.bus_access_strobe);
            }
            request = response;
        }
        for _ in 3..6 {
            assert!(!tick(&mut cpu, 0).bus_access_strobe);
        }
        assert_eq!(cpu.phase, 0);
        assert_eq!(cpu.registers.get_full_pc_address(), 0x0012_3456);
        assert_eq!(cpu.registers.coprocessor_select_register, 0);
        assert_eq!(
            cpu.eu_registers.current_exception_level,
            match operation {
                ExceptionUnitOpCodes::SoftwareException => 1,
                ExceptionUnitOpCodes::HardwareException => 2,
                ExceptionUnitOpCodes::Fault => 7,
                _ => 0,
            }
        );
    }
}

#[test]
fn l5_conflict_uses_an_idle_slot_then_only_fetches_the_conflict_fault_vector() {
    let mut cpu = new_cpu_peripheral();
    cpu.registers.pl = 0x0200;
    cpu.eu_registers.current_exception_level = 6;
    cpu.eu_registers.link_registers[5].return_address = 0x00ab_cdef;
    cpu.raise_hardware_interrupt(0x10);
    for phase in 0..6 {
        let response = cpu.poll(BusAssertions::default(), true);
        assert!(!response.bus_access_strobe);
        assert_eq!(cpu.phase, (phase + 1) % 6);
        assert_eq!(cpu.eu_registers.current_exception_level, 6);
        assert_eq!(
            cpu.eu_registers.link_registers[5].return_address,
            0x00ab_cdef
        );
        if phase >= 3 {
            let fault = cpu.eu_registers.pending_fault.unwrap();
            assert_eq!(fault.fault, Faults::LevelFiveInterruptConflict);
            assert_eq!(fault.pc, 0x0200);
        }
    }
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 0);
    let first = tick(&mut cpu, 0);
    assert_eq!(
        first.address,
        u32::from(LEVEL_FIVE_HARDWARE_EXCEPTION_CONFLICT) * 2
    );
    assert!(first.bus_access_strobe);
    assert_eq!(first.bus_access_type, BusAccessType::ExceptionVectorFetch);
    let second = tick(&mut cpu, 0);
    assert_eq!(second.address, first.address + 1);
    for data in [0x0800, 0, 0, 0] {
        tick(&mut cpu, data);
    }
    assert_eq!(cpu.eu_registers.current_exception_level, 7);
    assert_eq!(cpu.eu_registers.link_registers[6].return_address, 0x0200);
    assert_eq!(
        cpu.eu_registers.link_registers[5].return_address,
        0x00ab_cdef
    );
}
