mod faults;
mod halt;
mod hardware;
mod ownership;
mod support;
mod timing;
mod waits;
use peripheral_bus::device::{BusAccessType, BusAssertions, BusOperation};
use support::{Machine, ADDRESS_PAIR_SHIFT, BACKWARD, DMAR, DMAT, DMAW, FAULT_METADATA_LINK};

#[test]
fn register_bursts_preserve_flags_tags_and_order_for_every_pair_and_direction() {
    for write in [false, true] {
        for pair in 0..3 {
            for backward in [false, true] {
                for count in [0, 1, 7] {
                    let command = (if write { DMAW } else { DMAR })
                        | (pair << ADDRESS_PAIR_SHIFT)
                        | if backward { BACKWARD | count } else { count };
                    let mut m = Machine::new(command, false);
                    let before = m.cpu.registers;
                    let (high, low) = match pair {
                        0 => (before.ah, before.al),
                        1 => (before.lh, before.ll),
                        _ => (before.sh, before.sl),
                    };
                    let start = if backward { low - count } else { low };
                    for offset in 0..count {
                        m.memory.insert(
                            (u32::from(high & 0xff) << 16) | u32::from(start + offset),
                            0x8000 + offset,
                        );
                    }
                    let elapsed = m.run_to_resume();
                    assert_eq!(elapsed, if count < 6 { 12 } else { 18 });
                    let accesses = m.dma_accesses();
                    assert_eq!(accesses.len(), usize::from(count));
                    for (index, access) in accesses.iter().enumerate() {
                        assert_eq!(
                            access.address,
                            (u32::from(high & 0xff) << 16)
                                | u32::from(start + u16::try_from(index).unwrap())
                        );
                        assert!(matches!(
                            (write, access.op),
                            (true, BusOperation::Write) | (false, BusOperation::Read)
                        ));
                        assert_eq!(
                            access.bus_access_type,
                            if write {
                                BusAccessType::DmaWriteBurst
                            } else {
                                BusAccessType::DmaReadBurst
                            }
                        );
                        if write {
                            assert_eq!(
                                m.memory[&access.address],
                                before[u8::try_from(index).unwrap()]
                            );
                        } else {
                            assert_eq!(
                                m.cpu.registers[u8::try_from(index).unwrap()],
                                0x8000 + u16::try_from(index).unwrap()
                            );
                        }
                    }
                    for index in count..7 {
                        assert_eq!(
                            m.cpu.registers[u8::try_from(index).unwrap()],
                            before[u8::try_from(index).unwrap()]
                        );
                    }
                    let mut expected = before;
                    expected.pl = 0x0202;
                    for index in 0..count {
                        if !write {
                            expected[u8::try_from(index).unwrap()] = 0x8000 + index;
                        }
                    }
                    let final_low = if count == 0 {
                        low
                    } else if backward {
                        start
                    } else {
                        low + count
                    };
                    match pair {
                        0 => expected.al = final_low,
                        1 => expected.ll = final_low,
                        _ => expected.sl = final_low,
                    }
                    assert_eq!(m.cpu.registers, expected);
                }
            }
        }
    }
}

#[test]
fn copy_chunks_use_the_real_register_window_including_short_and_overlapping_chunks() {
    for count in [0u16, 1, 7, 8, 15, 255] {
        for overlap in [false, true] {
            let mut m = Machine::new(DMAT | count, false);
            if overlap {
                m.cpu.registers.lh = 0xef01;
                m.cpu.registers.ll = 0x1001;
            }
            for offset in 0..=count {
                m.memory
                    .insert(0x0001_1000 + u32::from(offset), 0x8000 + offset);
            }
            let mut expected_memory = m.memory.clone();
            let mut expected_registers = m.cpu.registers;
            expected_registers.pl = 0x0202;
            let destination = if overlap { 0x0001_1001 } else { 0x0002_2000 };
            let mut transferred = 0;
            while transferred < count {
                let chunk = (count - transferred).min(7);
                let buffer: Vec<_> = (0..chunk)
                    .map(|i| expected_memory[&(0x0001_1000 + u32::from(transferred + i))])
                    .collect();
                for (i, value) in buffer.into_iter().enumerate() {
                    expected_registers[u8::try_from(i).unwrap()] = value;
                    expected_memory.insert(
                        destination + u32::from(transferred) + u32::try_from(i).unwrap(),
                        value,
                    );
                }
                transferred += chunk;
            }
            expected_registers.al += count;
            expected_registers.ll += count;
            assert_eq!(
                m.run_to_resume(),
                6 + 6 * (usize::from(2 * count) + 1).div_ceil(6)
            );
            assert_eq!(m.memory, expected_memory);
            assert_eq!(m.cpu.registers, expected_registers);
            let accesses = m.dma_accesses();
            assert_eq!(accesses.len(), usize::from(2 * count));
            let mut position = 0;
            let mut remaining = usize::from(count);
            while remaining > 0 {
                let chunk = remaining.min(7);
                assert!(accesses[position..position + chunk]
                    .iter()
                    .all(|r| r.bus_access_type == BusAccessType::DmaReadBurst));
                assert!(accesses[position + chunk..position + 2 * chunk]
                    .iter()
                    .all(|r| r.bus_access_type == BusAccessType::DmaWriteBurst));
                remaining -= chunk;
                position += 2 * chunk;
            }
        }
    }
}

#[test]
fn copy_advances_both_pairs_atomically_after_acknowledged_words() {
    use peripheral_cpu::{
        coprocessors::exception_unit::definitions::Faults,
        decode_fault_metadata_register,
        registers::{set_sr_bit, StatusRegisterFields},
    };
    for (source, destination, source_wrap) in [
        (0xffff, 0x1000, true),
        (0x1000, 0xffff, false),
        (0xffff, 0xffff, true),
    ] {
        for trap in [false, true] {
            let mut m = Machine::new(DMAT | 1, false);
            m.cpu.registers.al = source;
            m.cpu.registers.ll = destination;
            if trap {
                set_sr_bit(
                    StatusRegisterFields::TrapOnAddressOverflow,
                    &mut m.cpu.registers,
                );
            }
            m.memory.insert(0x0001_0000 | u32::from(source), 0xcafe);
            if trap {
                for _ in 0..20 {
                    m.clock(BusAssertions::default(), false);
                    if m.cpu.eu_registers.pending_fault.is_some() {
                        break;
                    }
                }
                assert_eq!(
                    m.cpu.eu_registers.pending_fault.unwrap().fault,
                    Faults::SegmentOverflow
                );
                assert_eq!(m.cpu.eu_registers.pending_fault.unwrap().pc, 0xcd00_0200);
                assert_eq!(m.cpu.registers.al, source);
                assert_eq!(m.cpu.registers.ll, destination);
                let link = m.cpu.eu_registers.link_registers[FAULT_METADATA_LINK];
                assert_eq!(
                    link.return_address,
                    if source_wrap {
                        0xab01_0000
                    } else {
                        0xef02_0000
                    }
                );
                assert_eq!(
                    decode_fault_metadata_register(link.return_status_register).bus_access_type,
                    if source_wrap {
                        BusAccessType::DmaReadBurst
                    } else {
                        BusAccessType::DmaWriteBurst
                    }
                );
            } else {
                m.run_to_resume();
                assert_eq!(m.cpu.registers.al, source.wrapping_add(1));
                assert_eq!(m.cpu.registers.ll, destination.wrapping_add(1));
            }
            assert_eq!(m.memory[&(0x0002_0000 | u32::from(destination))], 0xcafe);
            assert_eq!(m.cpu.registers.r1, 0xcafe);
            assert_eq!(m.cpu.registers.ah, 0xab01);
            assert_eq!(m.cpu.registers.lh, 0xef02);
        }
    }
}
