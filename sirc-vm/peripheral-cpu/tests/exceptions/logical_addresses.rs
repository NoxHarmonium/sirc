use super::common::{build_rete_instruction, tick};
use peripheral_bus::{
    conversion::bytes_to_words,
    device::{BusAccessType, BusAssertions, Device},
};
use peripheral_cpu::{
    coprocessors::{
        exception_unit::definitions::{Faults, PendingFault},
        processing_unit::{
            definitions::{ConditionFlags, ImmediateInstructionData, Instruction, InstructionData},
            encoding::encode_instruction,
        },
    },
    new_cpu_peripheral,
    registers::{
        AddressRegisterName, FullAddress, FullAddressRegisterAccess, Registers, SegmentedAddress,
        SegmentedRegisterAccess,
    },
};

#[test]
fn all_address_pairs_round_trip_full_logical_addresses() {
    for address in [0, 0x00ff_ffff, 0xab12_3456, u32::MAX] {
        let mut registers = Registers::default();
        registers.set_full_address_address(address);
        registers.set_full_link_address(address);
        registers.set_full_sp_address(address);
        registers.set_full_pc_address(address);
        let pair = address.to_segmented_address();
        assert_eq!(pair.to_full_address(), address);
        assert_eq!(registers.get_segmented_address(), pair);
        assert_eq!(registers.get_segmented_link(), pair);
        assert_eq!(registers.get_segmented_sp(), pair);
        assert_eq!(registers.get_segmented_pc(), pair);
        assert_eq!(registers.get_full_address_address(), address);
        assert_eq!(registers.get_full_link_address(), address);
        assert_eq!(registers.get_full_sp_address(), address);
        assert_eq!(registers.get_full_pc_address(), address);
    }
}

#[test]
fn tagged_vectors_and_saved_return_pcs_survive_entry_rete_and_fetch_waits() {
    // Reset, trap, IRQ and fault all use the same lossless vector scratch/PC path.
    for kind in 0..4 {
        let mut cpu = new_cpu_peripheral();
        cpu.registers.set_full_pc_address(0xcd34_0200);
        let (vector, link) = match kind {
            0 => {
                cpu.reset();
                (0, None)
            }
            1 => {
                cpu.registers.coprocessor_select_register = 0x1160;
                (0x60, Some(0))
            }
            2 => {
                cpu.eu_registers.pending_hardware_exceptions = 1;
                peripheral_cpu::registers::set_hardware_interrupt_enable(&mut cpu.registers, 1);
                (0x50, Some(1))
            }
            _ => {
                cpu.eu_registers.pending_fault = Some(PendingFault {
                    fault: Faults::InvalidOpCode,
                    pc: 0xcd34_0200,
                });
                (4, Some(6))
            }
        };
        let first = tick(&mut cpu, 0);
        assert_eq!(first.address, vector * 2);
        for _ in 0..3 {
            let held = cpu.poll(BusAssertions::default(), true);
            assert_eq!(held.address, first.address);
            assert!(held.instruction_sync);
        }
        assert_eq!(tick(&mut cpu, 0xab12).address, first.address + 1);
        for data in [0x0800, 0, 0, 0] {
            tick(&mut cpu, data);
        }
        assert_eq!(cpu.registers.get_full_pc_address(), 0xab12_0800);
        assert_eq!(cpu.registers.ph, 0xab12);
        if let Some(link) = link {
            assert_eq!(
                cpu.eu_registers.link_registers[link].return_address,
                0xcd34_0200
            );
            let words = bytes_to_words(&encode_instruction(&build_rete_instruction()));
            for (phase, data) in [0, words[0], words[1], 0, 0, 0].into_iter().enumerate() {
                let out = tick(&mut cpu, data);
                if phase < 2 {
                    assert_eq!(out.address, 0x0012_0800 + u32::try_from(phase).unwrap());
                }
            }
            for _ in 0..6 {
                assert!(!tick(&mut cpu, 0).bus_access_strobe);
            }
            assert_eq!(cpu.registers.get_full_pc_address(), 0xcd34_0200);
            assert_eq!(cpu.registers.ph, 0xcd34);
        }
        let first = tick(&mut cpu, 0);
        let physical = if kind == 0 { 0x0012_0800 } else { 0x0034_0200 };
        assert_eq!(first.address, physical);
        for _ in 0..3 {
            let held = cpu.poll(BusAssertions::default(), true);
            assert_eq!(held.address, physical);
            assert!(held.instruction_sync);
        }
    }
}

#[test]
fn stalled_tagged_fetch_fault_reports_the_logical_address() {
    for protection in [false, true] {
        let mut cpu = new_cpu_peripheral();
        cpu.registers.set_full_pc_address(0xab12_0200);
        assert_eq!(tick(&mut cpu, 0).address, 0x0012_0200);
        assert_eq!(
            cpu.poll(BusAssertions::default(), true).address,
            0x0012_0200
        );
        cpu.poll(
            BusAssertions {
                address: 0x0012_0200,
                bus_error: !protection,
                bus_protection_error: protection,
                ..Default::default()
            },
            true,
        );
        assert_eq!(
            cpu.eu_registers.link_registers[7].return_address,
            0xab12_0200
        );
        assert_eq!(cpu.eu_registers.pending_fault.unwrap().pc, 0xab12_0200);
    }
}

#[test]
fn tagged_data_accesses_present_physical_addresses_and_keep_logical_fault_metadata() {
    for store in [false, true] {
        let mut cpu = new_cpu_peripheral();
        cpu.registers.set_full_pc_address(0xab12_0200);
        cpu.registers.set_full_address_address(0xcd34_4560);
        cpu.registers.r1 = 0xcafe;
        let instruction = InstructionData::Immediate(ImmediateInstructionData {
            op_code: if store {
                Instruction::StoreRegisterToIndirectImmediate
            } else {
                Instruction::LoadRegisterFromIndirectImmediate
            },
            register: 1,
            value: 2,
            condition_flag: ConditionFlags::Always,
            additional_flags: AddressRegisterName::Address.to_register_index(),
        });
        let words = bytes_to_words(&encode_instruction(&instruction));
        for data in [0, words[0], words[1], 0] {
            tick(&mut cpu, data);
        }
        let access = tick(&mut cpu, 0);
        assert!(access.bus_access_strobe);
        assert_eq!(access.address, 0x0034_4562);
        assert_eq!(
            access.bus_access_type,
            if store {
                BusAccessType::DataWrite
            } else {
                BusAccessType::DataRead
            }
        );
        assert_eq!(
            cpu.poll(BusAssertions::default(), true).address,
            access.address
        );
        cpu.poll(
            BusAssertions {
                bus_error: true,
                ..Default::default()
            },
            true,
        );
        assert_eq!(
            cpu.eu_registers.link_registers[7].return_address,
            0xcd34_4562
        );
        assert_eq!(cpu.eu_registers.pending_fault.unwrap().pc, 0xab12_0200);
    }
}

#[test]
fn exception_link_transfers_preserve_tagged_addresses() {
    for index in 0..8 {
        let mut cpu = new_cpu_peripheral();
        cpu.registers.set_full_address_address(0xab12_3456);
        cpu.registers.r7 = 0x1234;
        cpu.registers.coprocessor_select_register = 0x1d30 | u16::try_from(index).unwrap();
        for _ in 0..6 {
            assert!(!tick(&mut cpu, 0).bus_access_strobe);
        }
        assert_eq!(
            cpu.eu_registers.link_registers[index].return_address,
            0xab12_3456
        );
        cpu.registers.set_full_address_address(0);
        cpu.registers.r7 = 0;
        cpu.registers.coprocessor_select_register = 0x1c30 | u16::try_from(index).unwrap();
        for _ in 0..6 {
            assert!(!tick(&mut cpu, 0).bus_access_strobe);
        }
        assert_eq!(cpu.registers.get_full_address_address(), 0xab12_3456);
        assert_eq!(cpu.registers.r7, 0x1234);
    }
}

#[test]
fn changing_only_an_address_tag_still_requires_supervisor_mode() {
    let mut cpu = new_cpu_peripheral();
    peripheral_cpu::registers::set_sr_bit(
        peripheral_cpu::registers::StatusRegisterFields::ProtectedMode,
        &mut cpu.registers,
    );
    cpu.registers.ah = 0xab12;
    cpu.registers.sh = 0xcd12;
    let instruction = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::LoadEffectiveAddressFromIndirectImmediate,
        register: AddressRegisterName::Address.to_register_index(),
        value: 0,
        condition_flag: ConditionFlags::Always,
        additional_flags: AddressRegisterName::StackPointer.to_register_index(),
    });
    let words = bytes_to_words(&encode_instruction(&instruction));
    for data in [0, words[0], words[1], 0, 0, 0] {
        tick(&mut cpu, data);
    }
    assert_eq!(
        cpu.eu_registers.pending_fault.unwrap().fault,
        Faults::PrivilegeViolation
    );
    assert_eq!(cpu.registers.ah, 0xab12);
    assert_eq!(cpu.registers.sh, 0xcd12);
}
