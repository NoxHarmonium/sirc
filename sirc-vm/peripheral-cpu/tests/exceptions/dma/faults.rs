use super::support::Machine;
use peripheral_bus::device::{BusAccessType, BusAssertions};
use peripheral_cpu::{
    coprocessors::exception_unit::definitions::Faults,
    decode_fault_metadata_register,
    registers::{set_sr_bit, StatusRegisterFields},
};

fn until_request(m: &mut Machine, index: usize) -> BusAssertions {
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
    panic!("request not reached");
}
fn assert_fault(m: &Machine, fault: Faults, address: u32, bat: BusAccessType) {
    let pending = m.cpu.eu_registers.pending_fault.unwrap();
    assert_eq!(pending.fault, fault);
    assert_eq!(pending.pc, 0xcd00_0200);
    assert_eq!(m.cpu.registers.coprocessor_select_register, 0);
    let link = m.cpu.eu_registers.link_registers[7];
    assert_eq!(link.return_address, address);
    let metadata = decode_fault_metadata_register(link.return_status_register);
    assert_eq!(metadata.fault, fault);
    assert_eq!(metadata.bus_access_type, bat);
}

#[test]
fn berr_and_bper_keep_only_acknowledged_words_and_completed_chunk_advances() {
    for command in [0x2807, 0x2827, 0x2907, 0x2927, 0x2a08] {
        let copy = command == 0x2a08;
        let write = command & 0x0f00 == 0x0900;
        let backward = !copy && command & 0x20 != 0;
        let failing_indices = if copy {
            vec![0, 3, 6, 7, 10, 13, 14, 15]
        } else {
            vec![0, 3, 6]
        };
        for failing_index in failing_indices {
            for protection in [false, true] {
                let mut m = Machine::new(command, false);
                let start = if backward { 0x0ff9 } else { 0x1000 };
                for i in 0..8 {
                    m.memory.insert(
                        0x0001_0000 | (start + i),
                        0x8000 + u16::try_from(i).unwrap(),
                    );
                }
                let mut expected_memory = m.memory.clone();
                let mut expected = m.cpu.registers;
                expected.pl = 0x0202;
                expected.coprocessor_select_register = 0;
                if backward {
                    expected.al = 0x0ff9;
                }
                if copy {
                    // Replay only the prefix of successful read/write actions,
                    // committing pointers only after both halves of each chunk.
                    let mut actions = 0;
                    let mut copied = 0u16;
                    'chunks: while copied < 8 {
                        let chunk = (8 - copied).min(7);
                        for i in 0..chunk {
                            if actions == failing_index {
                                break 'chunks;
                            }
                            expected[u8::try_from(i).unwrap()] =
                                expected_memory[&(0x0001_1000 + u32::from(copied + i))];
                            actions += 1;
                        }
                        for i in 0..chunk {
                            if actions == failing_index {
                                break 'chunks;
                            }
                            expected_memory.insert(
                                0x0002_2000 + u32::from(copied + i),
                                expected[u8::try_from(i).unwrap()],
                            );
                            actions += 1;
                        }
                        copied += chunk;
                        expected.al += chunk;
                        expected.ll += chunk;
                    }
                } else {
                    for i in 0..failing_index {
                        let address = 0x0001_0000 | (start + u32::try_from(i).unwrap());
                        if write {
                            expected_memory.insert(address, expected[u8::try_from(i).unwrap()]);
                        } else {
                            expected[u8::try_from(i).unwrap()] = expected_memory[&address];
                        }
                    }
                }
                let request = until_request(&mut m, failing_index);
                // BACK accompanying an error must not commit that word.
                m.clock(
                    BusAssertions {
                        bus_acknowledge: true,
                        bus_error: !protection,
                        bus_protection_error: protection,
                        ..Default::default()
                    },
                    false,
                );
                let high = if copy && request.bus_access_type == BusAccessType::DmaWriteBurst {
                    0xef02
                } else {
                    0xab01
                };
                assert_fault(
                    &m,
                    if protection {
                        Faults::BusProtection
                    } else {
                        Faults::Bus
                    },
                    (high << 16) | (request.address & 0xffff),
                    request.bus_access_type,
                );
                assert_eq!(m.cpu.registers, expected);
                assert_eq!(m.memory, expected_memory);
                assert_eq!(m.dma_accesses().len(), failing_index);
                // Finish fault entry to prove the command cannot resume or issue another word.
                let vector = if protection { 18 } else { 2 };
                m.memory.insert(vector, 0);
                m.memory.insert(vector + 1, 0x0800);
                let mut entered = false;
                for _ in 0..20 {
                    let out = m.clock(BusAssertions::default(), false);
                    assert!(!matches!(
                        out.bus_access_type,
                        BusAccessType::DmaReadBurst | BusAccessType::DmaWriteBurst
                    ));
                    if out.bus_access_type == BusAccessType::InstructionFetch
                        && out.address == 0x0800
                    {
                        entered = true;
                        break;
                    }
                }
                assert!(entered);
                assert_eq!(
                    m.cpu.eu_registers.link_registers[6].return_address,
                    0xcd00_0200
                );
            }
        }
    }
}

#[test]
fn per_word_wraps_and_forward_advances_fault_at_their_own_commit_points() {
    for write in [false, true] {
        for count in [1, 2] {
            let mut m = Machine::new((if write { 0x2900 } else { 0x2800 }) | count, false);
            m.cpu.registers.al = 0xffff;
            set_sr_bit(
                StatusRegisterFields::TrapOnAddressOverflow,
                &mut m.cpu.registers,
            );
            m.memory.insert(0x0001_ffff, 0xcafe);
            for _ in 0..20 {
                m.clock(BusAssertions::default(), false);
                if m.cpu.eu_registers.pending_fault.is_some() {
                    break;
                }
            }
            assert_eq!(m.dma_accesses().len(), 1);
            assert_eq!(m.cpu.registers.al, 0xffff);
            assert_fault(
                &m,
                Faults::SegmentOverflow,
                0xab01_0000,
                if write {
                    BusAccessType::DmaWriteBurst
                } else {
                    BusAccessType::DmaReadBurst
                },
            );
            if write {
                assert_eq!(m.memory[&0x0001_ffff], 0x1100);
            } else {
                assert_eq!(m.cpu.registers.r1, 0xcafe);
            }
        }
        for trap in [false, true] {
            let mut m = Machine::new((if write { 0x2900 } else { 0x2800 }) | 0x27, false);
            m.cpu.registers.al = 3;
            let before = m.cpu.registers;
            if trap {
                set_sr_bit(
                    StatusRegisterFields::TrapOnAddressOverflow,
                    &mut m.cpu.registers,
                );
                for _ in 0..7 {
                    m.clock(BusAssertions::default(), false);
                }
                assert_fault(
                    &m,
                    Faults::SegmentOverflow,
                    0xab01_fffc,
                    if write {
                        BusAccessType::DmaWriteBurst
                    } else {
                        BusAccessType::DmaReadBurst
                    },
                );
                assert_eq!(m.cpu.registers.al, 3);
                assert!(m.dma_accesses().is_empty());
                for i in 0..7 {
                    assert_eq!(m.cpu.registers[i], before[i]);
                }
            } else {
                m.run_to_resume();
                assert_eq!(m.cpu.registers.al, 0xfffc);
                let addresses: Vec<_> = m.dma_accesses().iter().map(|r| r.address).collect();
                assert_eq!(
                    addresses,
                    vec![
                        0x0001_fffc,
                        0x0001_fffd,
                        0x0001_fffe,
                        0x0001_ffff,
                        0x0001_0000,
                        0x0001_0001,
                        0x0001_0002
                    ]
                );
            }
            assert_eq!(m.cpu.registers.ah, 0xab01);
        }
    }
}

#[test]
fn dmat_read_and_write_access_wraps_preserve_the_incomplete_chunk() {
    for writing in [false, true] {
        let mut m = Machine::new(0x2a02, false);
        if writing {
            m.cpu.registers.ll = 0xffff;
        } else {
            m.cpu.registers.al = 0xffff;
        }
        set_sr_bit(
            StatusRegisterFields::TrapOnAddressOverflow,
            &mut m.cpu.registers,
        );
        m.memory.insert(0x0001_ffff, 0xcafe);
        m.memory.insert(0x0001_1000, 0xcafe);
        m.memory.insert(0x0001_1001, 0xbeef);
        for _ in 0..20 {
            m.clock(BusAssertions::default(), false);
            if m.cpu.eu_registers.pending_fault.is_some() {
                break;
            }
        }
        assert_fault(
            &m,
            Faults::SegmentOverflow,
            if writing { 0xef02_0000 } else { 0xab01_0000 },
            if writing {
                BusAccessType::DmaWriteBurst
            } else {
                BusAccessType::DmaReadBurst
            },
        );
        assert_eq!(m.cpu.registers.al, if writing { 0x1000 } else { 0xffff });
        assert_eq!(m.cpu.registers.ll, if writing { 0xffff } else { 0x2000 });
        assert_eq!(m.cpu.registers.r1, 0xcafe);
        if writing {
            assert_eq!(m.cpu.registers.r2, 0xbeef);
            assert_eq!(m.memory[&0x0002_ffff], 0xcafe);
        } else {
            assert_eq!(m.cpu.registers.r2, 0x1101);
            assert_eq!(m.dma_accesses().len(), 1);
        }
    }
}
