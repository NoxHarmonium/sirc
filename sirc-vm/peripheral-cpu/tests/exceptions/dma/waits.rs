use super::support::{Machine, DMAR, DMAT, DMAW, FAULT_METADATA_LINK};
use peripheral_bus::device::{BusAccessType, BusAssertions, BusOperation};
use peripheral_cpu::coprocessors::exception_unit::definitions::Faults;

fn request_at(m: &mut Machine, index: usize) -> BusAssertions {
    for _ in 0..2000 {
        if let Some(request) = m.pending {
            if matches!(
                request.bus_access_type,
                BusAccessType::DmaReadBurst | BusAccessType::DmaWriteBurst
            ) && m.dma_accesses().len() == index
            {
                return request;
            }
        }
        m.clock(BusAssertions::default(), false);
    }
    panic!("DMA request was not reached");
}

#[test]
fn every_wait_holds_outputs_and_progress_and_adds_one_cycle_without_slot_rounding() {
    for command in [(DMAR | 5), (DMAW | 5), (DMAT | 8)] {
        let accesses = if command == (DMAT | 8) { 16 } else { 5 };
        let baseline = if command == (DMAT | 8) { 24 } else { 12 };
        for index in [0, accesses / 2, accesses - 1] {
            for waits in [1, 2] {
                let mut m = Machine::new(command, false);
                let request = request_at(&mut m, index);
                let registers = m.cpu.registers;
                let phase = m.cpu.phase;
                for _ in 0..waits {
                    let held = m.clock(
                        BusAssertions {
                            data: 0xdead,
                            force_trace_mode: true,
                            ..Default::default()
                        },
                        true,
                    );
                    assert_eq!(held.address, request.address);
                    assert_eq!(held.data, request.data);
                    assert_eq!(held.bus_access_type, request.bus_access_type);
                    assert_eq!(
                        matches!(held.op, BusOperation::Write),
                        matches!(request.op, BusOperation::Write)
                    );
                    assert_eq!(held.protected_mode_active, request.protected_mode_active);
                    assert!(held.bus_access_strobe);
                    assert!(!held.instruction_sync);
                    assert_eq!(m.cpu.registers, registers);
                    assert_eq!(m.cpu.phase, phase);
                    assert_eq!(m.dma_accesses().len(), index);
                }
                assert_eq!(m.run_to_resume(), baseline + waits);
                assert_eq!(m.dma_accesses().len(), accesses);
                assert_eq!(usize::from(m.cpu.phase), (m.cycles - waits) % 6);
            }
        }
    }
}

#[test]
fn an_error_releases_a_held_final_phase_five_access_without_committing_it() {
    for protection in [false, true] {
        let mut m = Machine::new(DMAR | 5, false);
        request_at(&mut m, 4);
        assert_eq!(m.cpu.phase, 0);
        for _ in 0..2 {
            m.clock(BusAssertions::default(), true);
        }
        let out = m.clock(
            BusAssertions {
                bus_error: !protection,
                bus_protection_error: protection,
                ..Default::default()
            },
            true,
        );
        assert_eq!(m.dma_accesses().len(), 4);
        assert_eq!(m.cpu.registers.r5, 0x1104);
        assert_eq!(m.cpu.registers.al, 0x1000);
        assert_eq!(m.cpu.registers.coprocessor_select_register, 0);
        let pending = m.cpu.eu_registers.pending_fault.unwrap();
        assert_eq!(
            pending.fault,
            if protection {
                Faults::BusProtection
            } else {
                Faults::Bus
            }
        );
        assert_eq!(pending.pc, 0xcd00_0200);
        assert_eq!(
            m.cpu.eu_registers.link_registers[FAULT_METADATA_LINK].return_address,
            0xab01_1004
        );
        assert_eq!(out.bus_access_type, BusAccessType::ExceptionVectorFetch);
        assert_eq!(out.address, if protection { 18 } else { 2 });
    }
}
