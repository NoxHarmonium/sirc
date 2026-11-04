mod support;
use peripheral_bus::device::{BusAccessType, BusOperation};
use support::Machine;

#[test]
fn register_bursts_preserve_flags_tags_and_order_for_every_pair_and_direction() {
    for write in [false, true] {
        for pair in 0..3 {
            for backward in [false, true] {
                for count in [0, 1, 7] {
                    let command = (if write { 0x2900 } else { 0x2800 })
                        | (pair << 6)
                        | if backward { 0x20 | count } else { count };
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
