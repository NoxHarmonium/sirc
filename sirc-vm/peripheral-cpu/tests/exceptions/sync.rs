use super::common::{build_test_instruction, tick};
use peripheral_bus::{
    conversion::bytes_to_words,
    device::{BusAssertions, Device},
    new_bus_peripheral,
};
use peripheral_cpu::{
    coprocessors::{
        exception_unit::definitions::{Faults, PendingFault},
        processing_unit::encoding::encode_instruction,
    },
    new_cpu_peripheral,
    registers::set_hardware_interrupt_enable,
};

#[test]
fn pu_fetch_waits_hold_the_presented_sync_until_acknowledged() {
    let mut cpu = new_cpu_peripheral();
    let words = bytes_to_words(&encode_instruction(&build_test_instruction()));
    cpu.registers.pl = 0x0200;
    let low = tick(&mut cpu, 0);
    assert!(low.instruction_sync);
    for _ in 0..3 {
        // HALT cannot interrupt an unacknowledged request.
        let held = cpu.poll(
            BusAssertions {
                halt_requested: true,
                ..Default::default()
            },
            true,
        );
        assert!(held.instruction_sync);
        assert!(held.bus_access_strobe);
        assert_eq!(held.address, low.address);
        assert_eq!(cpu.phase, 1);
    }
    let high = tick(&mut cpu, words[0]);
    assert!(!high.instruction_sync);
    for _ in 0..3 {
        let held = cpu.poll(BusAssertions::default(), true);
        assert!(!held.instruction_sync);
        assert!(held.bus_access_strobe);
        assert_eq!(held.address, high.address);
        assert_eq!(cpu.phase, 2);
    }
    for data in [words[1], 0, 0, 0] {
        assert!(!tick(&mut cpu, data).instruction_sync);
    }
    assert!(tick(&mut cpu, 0).instruction_sync);
}

#[test]
fn eu_vector_phase_zero_waits_keep_sync_until_ack_error_or_protection_error() {
    for completion in 0..3 {
        let mut cpu = new_cpu_peripheral();
        cpu.eu_registers.pending_fault = Some(PendingFault {
            fault: Faults::InvalidOpCode,
            pc: 0x0200,
        });
        let first = tick(&mut cpu, 0);
        assert!(first.instruction_sync);
        for _ in 0..3 {
            let held = cpu.poll(BusAssertions::default(), true);
            assert!(held.instruction_sync);
            assert_eq!(held.address, first.address);
        }
        let next = cpu.poll(
            BusAssertions {
                bus_acknowledge: completion == 0,
                bus_error: completion == 1,
                bus_protection_error: completion == 2,
                ..Default::default()
            },
            true,
        );
        assert!(!next.instruction_sync);
        for data in [0x0800, 0, 0, 0] {
            assert!(!tick(&mut cpu, data).instruction_sync);
        }
        assert_eq!(cpu.phase, 0);
        assert!(tick(&mut cpu, 0).instruction_sync);
    }
}

#[test]
fn bus_idle_eu_control_slots_still_have_phase_zero_sync() {
    for command in [0x1000, 0x1a00, 0x1c20, 0x1d20, 0x1900] {
        let mut cpu = new_cpu_peripheral();
        cpu.eu_registers.current_exception_level = 1;
        cpu.eu_registers.link_registers[0].return_address = 0x0200;
        cpu.registers.coprocessor_select_register = command;
        for phase in 0..4 {
            let out = cpu.poll(BusAssertions::default(), true);
            assert_eq!(out.instruction_sync, phase == 0);
            assert!(!out.bus_access_strobe);
        }
        if command == 0x1900 {
            for _ in 0..3 {
                let idle = tick(&mut cpu, 0);
                assert!(!idle.instruction_sync);
                assert!(!idle.bus_access_strobe);
                assert_eq!(cpu.phase, 4);
            }
            set_hardware_interrupt_enable(&mut cpu.registers, 1);
            cpu.raise_hardware_interrupt(1);
        }
        for _ in 0..2 {
            assert!(!tick(&mut cpu, 0).instruction_sync);
        }
        assert_eq!(cpu.phase, 0);
    }
}

#[test]
fn rejected_optional_unit_slots_and_halt_do_not_drive_sync() {
    // DMA and maths are absent: these slots reject their commands rather than
    // fetching instructions or dispatching the EU. Future optional executors
    // must retain the same low-SYNC slot policy.
    for command in [0x2000, 0x3000, 0x1200] {
        let mut cpu = new_cpu_peripheral();
        cpu.registers.coprocessor_select_register = command;
        for _ in 0..6 {
            let out = tick(&mut cpu, 0);
            assert!(!out.instruction_sync);
            assert!(!out.bus_access_strobe);
        }
        // The subsequent InvalidOpcode dispatch is an EU slot.
        assert!(tick(&mut cpu, 0).instruction_sync);
    }
    let mut cpu = new_cpu_peripheral();
    for _ in 0..3 {
        let idle = cpu.poll(
            BusAssertions {
                halt_requested: true,
                ..Default::default()
            },
            true,
        );
        assert!(!idle.instruction_sync);
        assert!(!idle.bus_access_strobe);
        assert_eq!(cpu.phase, 0);
    }
    assert!(tick(&mut cpu, 0).instruction_sync);
}

#[test]
fn reset_hold_has_no_sync_and_reset_vector_entry_resumes_eu_sync() {
    let mut bus = new_bus_peripheral(Box::new(new_cpu_peripheral()));
    let first = bus.poll_all(BusAssertions {
        reset_requested: true,
        ..Default::default()
    });
    assert!(first.reset_devices_on_bus);
    assert!(!first.instruction_sync);
    for _ in 0..5 {
        let held = bus.poll_all(BusAssertions::default());
        assert!(held.reset_devices_on_bus);
        assert!(!held.instruction_sync);
    }
    let vector = bus.poll_all(BusAssertions::default());
    assert!(!vector.reset_devices_on_bus);
    assert!(vector.instruction_sync);
    assert!(vector.bus_access_strobe);
    assert_eq!(vector.address, 0);
}
