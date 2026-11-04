use super::support::{Machine, BACKWARD, DMAR, DMAT, DMAW, RESERVED_DMA_OPCODE};
use peripheral_bus::device::{BusAccessType, BusAssertions};
use peripheral_cpu::{
    coprocessors::processing_unit::definitions::ConditionFlags,
    registers::{set_sr_bit, StatusRegisterFields},
};

#[test]
fn setup_accesses_padding_and_continuations_obey_the_documented_cycle_formula() {
    for (operation, counts) in [
        (DMAR, vec![0, 1, 5, 6, 7]),
        (DMAW, vec![0, 1, 5, 6, 7]),
        (DMAT, vec![0, 1, 3, 7, 8, 255]),
    ] {
        for count in counts {
            for register_call in [false, true] {
                let command = operation | count;
                let mut m = Machine::new(command, register_call);
                let accesses = usize::from(if operation == DMAT { 2 * count } else { count });
                let elapsed = 6 + 6 * (accesses + 1).div_ceil(6);
                for cycle in 0..=elapsed {
                    let out = m.clock(BusAssertions::default(), false);
                    if cycle < 6 {
                        assert_eq!(out.bus_access_strobe, cycle < 2);
                    } else if cycle < elapsed {
                        assert!(!out.instruction_sync);
                        assert_eq!(
                            out.bus_access_strobe,
                            (7..7 + accesses).contains(&cycle),
                            "operation={operation:04x} count={count} cycle={cycle}"
                        );
                        assert_eq!(m.cpu.registers.coprocessor_select_register, command);
                        if out.bus_access_strobe {
                            assert!(matches!(
                                out.bus_access_type,
                                BusAccessType::DmaReadBurst | BusAccessType::DmaWriteBurst
                            ));
                        }
                    } else {
                        assert!(out.instruction_sync);
                        assert!(out.bus_access_strobe);
                        assert_eq!(out.address, 0x0202);
                        assert_eq!(m.cpu.registers.coprocessor_select_register, 0);
                    }
                    assert_eq!(usize::from(m.cpu.phase), (cycle + 1) % 6);
                }
                assert_eq!(m.dma_accesses().len(), accesses);
            }
        }
    }
}

#[test]
fn skipped_dma_commands_use_only_the_call_slot_even_in_protected_mode() {
    for command in [
        (DMAR | 7),
        (DMAW | BACKWARD | 7),
        (DMAT | 255),
        (RESERVED_DMA_OPCODE | 1),
    ] {
        for register_call in [false, true] {
            let mut m = Machine::conditional(command, register_call, ConditionFlags::Never);
            set_sr_bit(StatusRegisterFields::ProtectedMode, &mut m.cpu.registers);
            let mut expected = m.cpu.registers;
            expected.pl = 0x0202;
            assert_eq!(m.run_to_resume(), 6);
            assert!(m.dma_accesses().is_empty());
            assert_eq!(m.cpu.registers, expected);
        }
    }
}
