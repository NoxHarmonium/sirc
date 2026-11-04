use super::support::Machine;
use peripheral_bus::device::{BusAccessType, BusAssertions};
use peripheral_cpu::{
    coprocessors::exception_unit::definitions::Faults,
    registers::{set_hardware_interrupt_enable, set_sr_bit, StatusRegisterFields},
};

#[test]
fn irq_and_trace_wait_until_the_entire_command_retires_then_keep_normal_priority() {
    for command in [0x2807, 0x2907, 0x2a08] {
        for trace in 0..3 {
            for irq in [1, 16, 17] {
                let mut m = Machine::new(command, false);
                set_hardware_interrupt_enable(&mut m.cpu.registers, 1);
                if trace == 1 {
                    set_sr_bit(StatusRegisterFields::TraceMode, &mut m.cpu.registers);
                }
                for vector in [12, 0x20, 0xa0] {
                    m.memory.insert(vector, 0);
                    m.memory.insert(vector + 1, 0x0800);
                }
                let elapsed = if command == 0x2a08 { 24 } else { 18 };
                for cycle in 0..elapsed {
                    let out = m.clock(
                        BusAssertions {
                            force_trace_mode: trace == 2 && cycle == 0,
                            interrupt_assertion: if cycle == 5 { irq } else { 0 },
                            ..Default::default()
                        },
                        false,
                    );
                    assert!(m.cpu.eu_registers.pending_fault.is_none());
                    if cycle >= 5 {
                        assert_eq!(m.cpu.registers.coprocessor_select_register, command);
                    }
                    if cycle >= 6 {
                        assert!(!out.instruction_sync);
                        assert_ne!(out.bus_access_type, BusAccessType::InstructionFetch);
                        assert_ne!(out.bus_access_type, BusAccessType::ExceptionVectorFetch);
                    }
                }
                let out = m.clock(BusAssertions::default(), false);
                let irq_vector = if irq & 16 != 0 { 0x20 } else { 0xa0 };
                assert_eq!(out.address, if trace == 0 { irq_vector } else { 12 });
                assert_eq!(out.bus_access_type, BusAccessType::ExceptionVectorFetch);
                assert_eq!(
                    m.dma_accesses().len(),
                    if command == 0x2a08 { 16 } else { 7 }
                );
                assert_eq!(m.cpu.registers.coprocessor_select_register, 0);
                if trace != 0 {
                    let pending = m.cpu.eu_registers.pending_fault.unwrap();
                    assert_eq!(pending.fault, Faults::InstructionTrace);
                    assert_eq!(pending.pc, 0xcd00_0202);
                    assert_eq!(m.cpu.eu_registers.pending_hardware_exceptions, irq);
                    for _ in 0..5 {
                        m.clock(BusAssertions::default(), false);
                    }
                    assert_eq!(
                        m.cpu.eu_registers.link_registers[6].return_address,
                        0xcd00_0202
                    );
                    m.cpu.registers.coprocessor_select_register = 0x1a00;
                    for _ in 0..6 {
                        m.clock(BusAssertions::default(), false);
                    }
                    assert_eq!(m.clock(BusAssertions::default(), false).address, irq_vector);
                }
                assert_eq!(
                    m.cpu.eu_registers.pending_hardware_exceptions,
                    irq & if irq & 16 != 0 { !16 } else { !1 }
                );
                for _ in 0..5 {
                    m.clock(BusAssertions::default(), false);
                }
                let link = if irq & 16 != 0 { 5 } else { 1 };
                assert_eq!(
                    m.cpu.eu_registers.link_registers[link].return_address,
                    0xcd00_0202
                );
            }
        }
    }
}

#[test]
fn l5_edges_and_overruns_are_sampled_during_dma_waits_but_dispatch_only_after_completion() {
    let mut m = Machine::new(0x2a08, false);
    for _ in 0..8 {
        m.clock(BusAssertions::default(), false);
    }
    for interrupt in [16, 0, 16] {
        m.clock(
            BusAssertions {
                interrupt_assertion: interrupt,
                ..Default::default()
            },
            true,
        );
    }
    assert!(m.cpu.eu_registers.l5_overrun);
    assert_eq!(m.cpu.eu_registers.pending_hardware_exceptions, 16);
    assert!(m.cpu.eu_registers.pending_fault.is_none());
    let mut dispatched = false;
    for _ in 0..40 {
        let out = m.clock(BusAssertions::default(), false);
        if out.bus_access_type == BusAccessType::ExceptionVectorFetch {
            assert_eq!(out.address, 14);
            assert_eq!(m.dma_accesses().len(), 16);
            assert_eq!(
                m.cpu.eu_registers.pending_fault.unwrap().fault,
                Faults::LevelFiveInterruptConflict
            );
            assert!(!m.cpu.eu_registers.l5_overrun);
            assert_eq!(m.cpu.eu_registers.pending_hardware_exceptions, 16);
            dispatched = true;
            break;
        }
    }
    assert!(dispatched);
}

#[test]
fn a_dma_fault_cancels_its_deferred_trace_without_consuming_queued_irq() {
    let mut m = Machine::new(0x2807, false);
    set_sr_bit(StatusRegisterFields::TraceMode, &mut m.cpu.registers);
    set_hardware_interrupt_enable(&mut m.cpu.registers, 1);
    for cycle in 0..6 {
        m.clock(
            BusAssertions {
                interrupt_assertion: u8::from(cycle == 5),
                ..Default::default()
            },
            false,
        );
    }
    while m.dma_accesses().is_empty() {
        m.clock(BusAssertions::default(), false);
    }
    m.clock(
        BusAssertions {
            bus_error: true,
            ..Default::default()
        },
        false,
    );
    assert_eq!(m.cpu.eu_registers.pending_fault.unwrap().fault, Faults::Bus);
    assert_eq!(m.cpu.eu_registers.pending_hardware_exceptions, 1);
    m.memory.insert(2, 0);
    m.memory.insert(3, 0x0800);
    let mut entered = false;
    for _ in 0..20 {
        let out = m.clock(BusAssertions::default(), false);
        assert_ne!(out.bus_access_type, BusAccessType::DmaReadBurst);
        if out.bus_access_type == BusAccessType::InstructionFetch && out.address == 0x0800 {
            entered = true;
            break;
        }
    }
    assert!(entered);
    assert!(m.cpu.eu_registers.pending_fault.is_none());
    assert_eq!(m.cpu.eu_registers.pending_hardware_exceptions, 1);
    assert_eq!(
        m.cpu.eu_registers.link_registers[6].return_address,
        0xcd00_0200
    );
}
