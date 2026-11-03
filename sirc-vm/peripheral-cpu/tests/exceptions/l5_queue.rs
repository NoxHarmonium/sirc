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
