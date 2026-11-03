use super::common::{build_test_instruction, complete_entry, tick};
use peripheral_bus::{
    conversion::bytes_to_words,
    device::{BusAssertions, Device},
};
use peripheral_cpu::{
    coprocessors::processing_unit::encoding::encode_instruction, new_cpu_peripheral,
};

#[test]
fn l5_pulse_inside_a_bus_wait_is_retained_until_the_next_boundary() {
    let mut cpu = new_cpu_peripheral();
    cpu.registers.pl = 0x0200;
    let words = bytes_to_words(&encode_instruction(&build_test_instruction()));
    tick(&mut cpu, 0);
    for interrupt in [0, 16, 16, 0, 0] {
        let held = cpu.poll(
            BusAssertions {
                interrupt_assertion: interrupt,
                ..Default::default()
            },
            true,
        );
        assert!(held.bus_access_strobe);
        assert_eq!(held.address, 0x0200);
        assert_eq!(cpu.phase, 1);
    }
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 16);
    for data in [words[0], words[1], 0, 0, 0] {
        tick(&mut cpu, data);
    }
    complete_entry(&mut cpu, 0x10, 0x1000);
    assert_eq!(cpu.eu_registers.current_exception_level, 6);
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 0);
    assert_eq!(cpu.eu_registers.link_registers[5].return_address, 0x0202);
}

#[test]
fn l5_pulse_inside_halt_is_retained_without_resuming_the_cpu() {
    let mut cpu = new_cpu_peripheral();
    cpu.registers.pl = 0x0200;
    for interrupt in [0, 16, 16, 0, 0] {
        let idle = cpu.poll(
            BusAssertions {
                interrupt_assertion: interrupt,
                halt_requested: true,
                ..Default::default()
            },
            true,
        );
        assert!(!idle.bus_access_strobe);
        assert_eq!(cpu.phase, 0);
    }
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 16);
    complete_entry(&mut cpu, 0x10, 0x1000);
    assert_eq!(cpu.eu_registers.current_exception_level, 6);
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 0);
    assert_eq!(cpu.eu_registers.link_registers[5].return_address, 0x0200);
}

fn return_from_handler(cpu: &mut peripheral_cpu::CpuPeripheral) {
    // Drive the internal RETE command directly; these tests exercise event
    // selection/queue state, not the already-covered COP instruction encoding.
    cpu.registers.coprocessor_select_register = 0x1a00;
    for _ in 0..6 {
        assert!(!tick(cpu, 0).bus_access_strobe);
    }
    assert_eq!(cpu.phase, 0);
}

#[test]
fn queued_edges_coalesce_into_one_conflict_then_one_l5_delivery() {
    let mut cpu = new_cpu_peripheral();
    cpu.registers.pl = 0x0800;
    cpu.eu_registers.current_exception_level = 7;
    cpu.eu_registers.link_registers[6].return_address = 0x0200;
    cpu.eu_registers.link_registers[5].return_address = 0x0044_5566;
    for level in [16, 16, 0, 16, 0, 16, 0] {
        cpu.raise_hardware_interrupt(level);
    }
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 16);
    assert!(cpu.eu_registers.l5_overrun);
    return_from_handler(&mut cpu);
    assert_eq!(cpu.eu_registers.current_exception_level, 0);
    complete_entry(&mut cpu, 0x07, 0x0800);
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 16);
    assert!(!cpu.eu_registers.l5_overrun);
    assert_eq!(
        cpu.eu_registers.link_registers[5].return_address,
        0x0044_5566
    );
    return_from_handler(&mut cpu);
    complete_entry(&mut cpu, 0x10, 0x1000);
    assert_eq!(cpu.eu_registers.current_exception_level, 6);
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 0);
    assert_eq!(cpu.eu_registers.link_registers[5].return_address, 0x0200);
    return_from_handler(&mut cpu);
    let next = tick(&mut cpu, 0);
    assert_eq!(next.address, 0x0200);
    assert_eq!(cpu.active_command, 0);
}

#[test]
fn reentry_and_overrun_share_one_conflict_and_consume_the_request() {
    let mut cpu = new_cpu_peripheral();
    cpu.registers.pl = 0x0200;
    cpu.eu_registers.current_exception_level = 6;
    cpu.eu_registers.link_registers[5].return_address = 0x0044_5566;
    for level in [16, 0, 16, 0] {
        cpu.raise_hardware_interrupt(level);
    }
    assert!(cpu.eu_registers.l5_overrun);
    complete_entry(&mut cpu, 0x07, 0x0800);
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 0);
    assert!(!cpu.eu_registers.l5_overrun);
    assert_eq!(cpu.eu_registers.link_registers[6].saved_exception_level, 6);
    assert_eq!(
        cpu.eu_registers.link_registers[5].return_address,
        0x0044_5566
    );
    return_from_handler(&mut cpu);
    assert_eq!(cpu.eu_registers.current_exception_level, 6);
    assert_eq!(tick(&mut cpu, 0).address, 0x0200);
    assert_eq!(cpu.active_command, 0);
}

#[test]
fn reset_clears_both_l5_queue_bits() {
    let mut cpu = new_cpu_peripheral();
    for level in [16, 0, 16, 0] {
        cpu.raise_hardware_interrupt(level);
    }
    assert!(cpu.eu_registers.l5_overrun);
    cpu.reset();
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 0);
    assert!(!cpu.eu_registers.l5_overrun);
    complete_entry(&mut cpu, 0, 0x0400);
    assert_eq!(tick(&mut cpu, 0).address, 0x0400);
    assert_eq!(cpu.active_command, 0);
}
