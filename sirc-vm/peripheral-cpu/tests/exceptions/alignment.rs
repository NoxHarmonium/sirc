use super::common::{build_rete_instruction, complete_entry, tick};
use peripheral_bus::{
    conversion::bytes_to_words,
    device::{BusAccessType, BusAssertions, Device},
};
use peripheral_cpu::{
    coprocessors::{
        exception_unit::{
            definitions::{vectors::ALIGNMENT_FAULT, Faults},
            execution::hardware_exception_level_to_vector,
        },
        processing_unit::encoding::encode_instruction,
    },
    decode_fault_metadata_register, new_cpu_peripheral,
    registers::{
        set_hardware_interrupt_enable, set_sr_bit, FullAddressRegisterAccess, StatusRegisterFields,
    },
    CpuPeripheral,
};

#[test]
fn alignment_dispatch_starts_immediately_with_precise_metadata_even_when_stalled() {
    for waits in [0, 3] {
        let mut cpu = new_cpu_peripheral();
        cpu.registers.set_full_pc_address(0x0012_0345);
        set_sr_bit(StatusRegisterFields::ProtectedMode, &mut cpu.registers);
        set_sr_bit(StatusRegisterFields::TraceMode, &mut cpu.registers);
        // Incoming bus fields are stale and must not contaminate fault metadata.
        let first = cpu.poll(
            BusAssertions {
                address: 0xdead,
                bus_access_type: BusAccessType::DataWrite,
                ..Default::default()
            },
            true,
        );
        assert!(first.bus_access_strobe);
        assert!(first.instruction_sync);
        assert!(!first.protected_mode_active);
        assert_eq!(first.address, u32::from(ALIGNMENT_FAULT) * 2);
        assert_eq!(first.bus_access_type, BusAccessType::ExceptionVectorFetch);
        assert_eq!(cpu.phase, 1);
        let metadata = cpu.eu_registers.link_registers[7];
        assert_eq!(metadata.return_address, 0x0012_0345);
        let decoded = decode_fault_metadata_register(metadata.return_status_register);
        assert_eq!(decoded.fault, Faults::Alignment);
        assert_eq!(decoded.bus_access_type, BusAccessType::InstructionFetch);
        for _ in 0..waits {
            let held = cpu.poll(BusAssertions::default(), true);
            assert!(held.bus_access_strobe);
            assert_eq!(held.address, first.address);
            assert_eq!(held.bus_access_type, BusAccessType::ExceptionVectorFetch);
            assert!(!held.protected_mode_active);
            assert_eq!(cpu.phase, 1);
            assert_eq!(cpu.eu_registers.link_registers[7], metadata);
        }
        let second = tick(&mut cpu, 0);
        assert!(second.bus_access_strobe);
        assert_eq!(second.address, first.address + 1);
        for data in [0x0800, 0, 0, 0] {
            assert!(!tick(&mut cpu, data).bus_access_strobe);
        }
        assert_eq!(cpu.phase, 0);
        assert_eq!(cpu.registers.get_full_pc_address(), 0x0800);
        assert_eq!(
            cpu.eu_registers.link_registers[6].return_address,
            0x0012_0345
        );
        // Alignment entry must not create a deferred instruction trace.
        assert_eq!(tick(&mut cpu, 0).address, 0x0800);
    }
}

fn execute_rete(cpu: &mut CpuPeripheral) {
    let words = bytes_to_words(&encode_instruction(&build_rete_instruction()));
    for data in [0, words[0], words[1], 0, 0, 0] {
        tick(cpu, data);
    }
    for _ in 0..6 {
        assert!(!tick(cpu, 0).bus_access_strobe);
    }
    assert_eq!(cpu.phase, 0);
    assert_eq!(cpu.eu_registers.current_exception_level, 0);
}

#[test]
fn rete_to_uncorrected_odd_pc_immediately_repeats_alignment_entry() {
    let mut cpu = new_cpu_peripheral();
    cpu.registers.pl = 0x0201;
    complete_entry(&mut cpu, ALIGNMENT_FAULT, 0x0800);
    execute_rete(&mut cpu);
    assert_eq!(cpu.registers.pl, 0x0201);
    complete_entry(&mut cpu, ALIGNMENT_FAULT, 0x0800);
    assert_eq!(cpu.eu_registers.link_registers[6].return_address, 0x0201);
}

#[test]
fn alignment_preflight_runs_only_when_arbitration_selects_a_pu_fetch() {
    for enabled in [false, true] {
        let mut cpu = new_cpu_peripheral();
        cpu.registers.pl = 0x0201;
        set_hardware_interrupt_enable(&mut cpu.registers, u8::from(enabled));
        cpu.eu_registers.pending_hardware_exceptions = 1;
        if enabled {
            complete_entry(&mut cpu, hardware_exception_level_to_vector(1), 0x1000);
            assert_eq!(cpu.eu_registers.link_registers[1].return_address, 0x0201);
            assert_eq!(cpu.eu_registers.link_registers[7].return_address, 0);
            assert!(cpu.eu_registers.pending_fault.is_none());
            execute_rete(&mut cpu);
        }
        complete_entry(&mut cpu, ALIGNMENT_FAULT, 0x0800);
        assert_eq!(cpu.eu_registers.link_registers[6].return_address, 0x0201);
        assert_eq!(
            cpu.eu_registers.pending_hardware_exceptions,
            u8::from(!enabled)
        );
    }
}
