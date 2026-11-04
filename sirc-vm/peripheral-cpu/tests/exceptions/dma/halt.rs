use super::support::Machine;
use peripheral_bus::device::{BusAccessType, BusAssertions};

fn halt() -> BusAssertions {
    BusAssertions {
        halt_requested: true,
        ..Default::default()
    }
}

#[test]
fn halt_at_every_dma_phase_preserves_the_exact_transfer_and_resumes_once() {
    let mut reference = Machine::new(0x2a0f, false);
    reference.run_to_resume();
    for offset in 6..42 {
        let mut m = Machine::new(0x2a0f, false);
        for _ in 0..offset {
            m.clock(BusAssertions::default(), false);
        }
        let mut latency = 0;
        while m.cpu.phase != 0 {
            m.clock(halt(), false);
            latency += 1;
            assert!(latency <= 5);
        }
        // The preceding phase-5 request still needs its ACK before HALT takes effect.
        let out = m.clock(halt(), false);
        assert!(!out.bus_access_strobe && !out.instruction_sync);
        let registers = m.cpu.registers;
        let count = m.dma_accesses().len();
        for _ in 0..10 {
            let out = m.clock(halt(), false);
            assert!(!out.bus_access_strobe && !out.instruction_sync);
            assert_eq!(m.cpu.registers, registers);
            assert_eq!(m.cpu.phase, 0);
        }
        assert_eq!(m.dma_accesses().len(), count);
        m.run_to_resume();
        assert_eq!(m.cpu.registers, reference.cpu.registers);
        assert_eq!(m.memory, reference.memory);
        let actual: Vec<_> = m
            .dma_accesses()
            .iter()
            .map(|r| (r.address, r.data, r.bus_access_type))
            .collect();
        let expected: Vec<_> = reference
            .dma_accesses()
            .iter()
            .map(|r| (r.address, r.data, r.bus_access_type))
            .collect();
        assert_eq!(actual, expected);
    }
}

#[test]
fn halt_cannot_retract_an_unacknowledged_access_and_l5_stays_queued_until_dma_finishes() {
    let mut m = Machine::new(0x2a08, false);
    for _ in 0..8 {
        m.clock(BusAssertions::default(), false);
    }
    let request = m.pending.unwrap();
    for _ in 0..10 {
        let out = m.clock(halt(), true);
        assert!(out.bus_access_strobe);
        assert_eq!(out.address, request.address);
        assert_eq!(out.data, request.data);
        assert_eq!(m.cpu.registers.coprocessor_select_register, 0x2a08);
    }
    while m.cpu.phase != 0 {
        m.clock(halt(), false);
    }
    assert!(!m.clock(halt(), false).bus_access_strobe);
    assert_eq!(m.cpu.eu_registers.pending_hardware_exceptions, 0);
    for cycle in 0..10 {
        let out = m.clock(
            BusAssertions {
                interrupt_assertion: if cycle == 0 { 16 } else { 0 },
                ..halt()
            },
            false,
        );
        assert!(!out.bus_access_strobe);
        assert_eq!(m.cpu.eu_registers.pending_hardware_exceptions, 16);
    }
    for _ in 0..40 {
        let out = m.clock(BusAssertions::default(), false);
        if out.bus_access_type == BusAccessType::ExceptionVectorFetch {
            assert_eq!(out.address, 0x20);
            assert_eq!(m.dma_accesses().len(), 16);
            assert_eq!(m.cpu.registers.coprocessor_select_register, 0);
            return;
        }
    }
    panic!("queued L5 was not dispatched after DMA resumed");
}
