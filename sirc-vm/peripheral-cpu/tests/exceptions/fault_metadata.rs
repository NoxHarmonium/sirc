use super::common::{complete_entry, tick};
use peripheral_bus::device::{BusAccessType, BusAssertions, Device};
use peripheral_cpu::{
    coprocessors::exception_unit::definitions::{vectors::DOUBLE_FAULT_VECTOR, Faults},
    decode_fault_metadata_register, new_cpu_peripheral, raise_fault,
};

#[test]
fn nested_faults_keep_original_and_escalating_causes_including_double_fault_handlers() {
    for original_code in 1..=9 {
        let original: Faults = num::FromPrimitive::from_u8(original_code).unwrap();
        for incoming in [Faults::Bus, Faults::SegmentOverflow, Faults::BusProtection] {
            let mut cpu = new_cpu_peripheral();
            raise_fault(
                &mut cpu.eu_registers,
                original,
                &BusAssertions::default(),
                0x0200,
            );
            let ordinary = decode_fault_metadata_register(
                cpu.eu_registers.link_registers[7].return_status_register,
            );
            assert_eq!(ordinary.original_fault, None);
            assert_eq!(ordinary.escalating_fault, None);
            complete_entry(&mut cpu, original_code, 0x0800);
            let access = BusAssertions {
                address: 0x0012_3456,
                bus_access_type: BusAccessType::DataWrite,
                ..Default::default()
            };
            raise_fault(&mut cpu.eu_registers, incoming, &access, 0x0800);
            let link = cpu.eu_registers.link_registers[7];
            let metadata = decode_fault_metadata_register(link.return_status_register);
            assert!(metadata.double_fault);
            assert_eq!(metadata.fault, Faults::DoubleFault);
            assert_eq!(metadata.original_fault, Some(original));
            assert_eq!(metadata.escalating_fault, Some(incoming));
            assert_eq!(metadata.bus_access_type, access.bus_access_type);
            assert_eq!(link.return_address, access.address);
            complete_entry(&mut cpu, DOUBLE_FAULT_VECTOR, 0x1000);
            assert_eq!(cpu.eu_registers.link_registers[6].return_address, 0x0800);
            // Escalating once more reads Current=DoubleFault, not the older Original.
            raise_fault(&mut cpu.eu_registers, Faults::Alignment, &access, 0x1000);
            let metadata = decode_fault_metadata_register(
                cpu.eu_registers.link_registers[7].return_status_register,
            );
            assert_eq!(metadata.original_fault, Some(Faults::DoubleFault));
            assert_eq!(metadata.escalating_fault, Some(Faults::Alignment));
        }
    }
}

#[test]
fn failed_fault_vector_metadata_identifies_both_causes_and_the_failing_word() {
    for original in [
        Faults::Alignment,
        Faults::InvalidOpCode,
        Faults::BusProtection,
    ] {
        for failed_word in [0, 1] {
            for protection_error in [false, true] {
                for waits in [0, 2] {
                    let mut cpu = new_cpu_peripheral();
                    raise_fault(
                        &mut cpu.eu_registers,
                        original,
                        &BusAssertions::default(),
                        0x0200,
                    );
                    let first = tick(&mut cpu, 0);
                    assert_eq!(first.address, u32::from(original as u8) * 2);
                    if failed_word == 1 {
                        tick(&mut cpu, 0x00ab);
                    }
                    for _ in 0..waits {
                        let held = cpu.poll(BusAssertions::default(), true);
                        assert_eq!(held.address, first.address + failed_word);
                    }
                    let response = cpu.poll(
                        BusAssertions {
                            bus_error: !protection_error,
                            bus_protection_error: protection_error,
                            // The completed request, not these stale fields, identifies the access.
                            address: 0xdead,
                            bus_access_type: BusAccessType::DataRead,
                            ..Default::default()
                        },
                        true,
                    );
                    assert!(!response.bus_access_strobe);
                    let metadata_link = cpu.eu_registers.link_registers[7];
                    let metadata =
                        decode_fault_metadata_register(metadata_link.return_status_register);
                    assert_eq!(metadata.fault, Faults::DoubleFault);
                    assert_eq!(metadata.original_fault, Some(original));
                    assert_eq!(
                        metadata.escalating_fault,
                        Some(if protection_error {
                            Faults::BusProtection
                        } else {
                            Faults::Bus
                        })
                    );
                    assert_eq!(
                        metadata.bus_access_type,
                        BusAccessType::ExceptionVectorFetch
                    );
                    assert_eq!(metadata_link.return_address, first.address + failed_word);
                    assert_eq!(cpu.eu_registers.pending_fault.unwrap().pc, 0x0200);
                    while cpu.phase != 0 {
                        assert!(!tick(&mut cpu, 0).bus_access_strobe);
                    }
                    complete_entry(&mut cpu, DOUBLE_FAULT_VECTOR, 0x1000);
                    assert_eq!(cpu.eu_registers.link_registers[6].return_address, 0x0200);
                }
            }
        }
    }
}
