use super::support::{
    Machine, DMAR, DMAT, DMAW, FAULT_LINK, L1_VECTOR, L5_CONFLICT_VECTOR, L5_VECTOR,
    LEVEL_FIVE_PIN, RETE, TRACE_VECTOR,
};
use peripheral_bus::device::{BusAccessType, BusAssertions};
use peripheral_cpu::{
    coprocessors::exception_unit::definitions::Faults,
    registers::{set_hardware_interrupt_enable, set_sr_bit, StatusRegisterFields},
};

#[test]
fn irq_and_trace_wait_until_the_entire_command_retires_then_keep_normal_priority() {
    for command in [(DMAR | 7), (DMAW | 7), (DMAT | 8)] {
        for (status_trace, pin_trace) in [(false, false), (true, false), (false, true)] {
            let trace_requested = status_trace || pin_trace;
            for irq in [1, LEVEL_FIVE_PIN, 1 | LEVEL_FIVE_PIN] {
                let mut m = Machine::new(command, false);
                set_hardware_interrupt_enable(&mut m.cpu.registers, 1);
                if status_trace {
                    set_sr_bit(StatusRegisterFields::TraceMode, &mut m.cpu.registers);
                }
                for vector in [TRACE_VECTOR, L5_VECTOR, L1_VECTOR] {
                    m.memory.insert(vector, 0);
                    m.memory.insert(vector + 1, 0x0800);
                }
                let elapsed = if command == (DMAT | 8) { 24 } else { 18 };
                for cycle in 0..elapsed {
                    let out = m.clock(
                        BusAssertions {
                            force_trace_mode: pin_trace && cycle == 0,
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
                let irq_vector = if irq & LEVEL_FIVE_PIN != 0 {
                    L5_VECTOR
                } else {
                    L1_VECTOR
                };
                assert_eq!(
                    out.address,
                    if trace_requested {
                        TRACE_VECTOR
                    } else {
                        irq_vector
                    }
                );
                assert_eq!(out.bus_access_type, BusAccessType::ExceptionVectorFetch);
                assert_eq!(
                    m.dma_accesses().len(),
                    if command == (DMAT | 8) { 16 } else { 7 }
                );
                assert_eq!(m.cpu.registers.coprocessor_select_register, 0);
                if trace_requested {
                    let pending = m.cpu.eu_registers.pending_fault.unwrap();
                    assert_eq!(pending.fault, Faults::InstructionTrace);
                    assert_eq!(pending.pc, 0xcd00_0202);
                    assert_eq!(m.cpu.eu_registers.pending_hardware_exceptions, irq);
                    for _ in 0..5 {
                        m.clock(BusAssertions::default(), false);
                    }
                    assert_eq!(
                        m.cpu.eu_registers.link_registers[FAULT_LINK].return_address,
                        0xcd00_0202
                    );
                    m.cpu.registers.coprocessor_select_register = RETE;
                    for _ in 0..6 {
                        m.clock(BusAssertions::default(), false);
                    }
                    assert_eq!(m.clock(BusAssertions::default(), false).address, irq_vector);
                }
                assert_eq!(
                    m.cpu.eu_registers.pending_hardware_exceptions,
                    irq & if irq & LEVEL_FIVE_PIN != 0 {
                        !LEVEL_FIVE_PIN
                    } else {
                        !1
                    }
                );
                for _ in 0..5 {
                    m.clock(BusAssertions::default(), false);
                }
                let link = if irq & LEVEL_FIVE_PIN != 0 { 5 } else { 1 };
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
    let mut m = Machine::new(DMAT | 8, false);
    for _ in 0..8 {
        m.clock(BusAssertions::default(), false);
    }
    for interrupt in [LEVEL_FIVE_PIN, 0, LEVEL_FIVE_PIN] {
        m.clock(
            BusAssertions {
                interrupt_assertion: interrupt,
                ..Default::default()
            },
            true,
        );
    }
    assert!(m.cpu.eu_registers.l5_overrun);
    assert_eq!(
        m.cpu.eu_registers.pending_hardware_exceptions,
        LEVEL_FIVE_PIN
    );
    assert!(m.cpu.eu_registers.pending_fault.is_none());
    let mut dispatched = false;
    for _ in 0..40 {
        let out = m.clock(BusAssertions::default(), false);
        if out.bus_access_type == BusAccessType::ExceptionVectorFetch {
            assert_eq!(out.address, L5_CONFLICT_VECTOR);
            assert_eq!(m.dma_accesses().len(), 16);
            assert_eq!(
                m.cpu.eu_registers.pending_fault.unwrap().fault,
                Faults::LevelFiveInterruptConflict
            );
            assert!(!m.cpu.eu_registers.l5_overrun);
            assert_eq!(
                m.cpu.eu_registers.pending_hardware_exceptions,
                LEVEL_FIVE_PIN
            );
            dispatched = true;
            break;
        }
    }
    assert!(dispatched);
}

#[test]
fn a_dma_fault_cancels_its_deferred_trace_without_consuming_queued_irq() {
    let mut m = Machine::new(DMAR | 7, false);
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
        m.cpu.eu_registers.link_registers[FAULT_LINK].return_address,
        0xcd00_0200
    );
}
