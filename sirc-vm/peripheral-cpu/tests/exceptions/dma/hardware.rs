use super::support::Machine;
use peripheral_bus::{
    device::{BusAccessType, BusAssertions, Device},
    memory_mapped_device::{new_stub_memory_mapped_device, MemoryMapped, MemoryMappedDevice},
    new_bus_peripheral,
};
use peripheral_cpu::{
    coprocessors::exception_unit::definitions::Faults,
    decode_fault_metadata_register,
    registers::{set_sr_bit, StatusRegisterFields},
    CpuPeripheral,
};
use std::any::Any;

#[test]
fn dma_privilege_checks_precede_dispatch_even_for_zero_length_commands() {
    for command in [0x2800, 0x2807, 0x2900, 0x2907, 0x2a00, 0x2a08] {
        let mut m = Machine::new(command, false);
        set_sr_bit(StatusRegisterFields::ProtectedMode, &mut m.cpu.registers);
        let before = m.cpu.registers;
        for _ in 0..3 {
            m.clock(BusAssertions::default(), false);
        }
        assert_eq!(
            m.cpu.eu_registers.pending_fault.unwrap().fault,
            Faults::PrivilegeViolation
        );
        assert_eq!(m.cpu.registers.r1, before.r1);
        assert_eq!(m.cpu.registers.al, before.al);
        assert_eq!(m.cpu.registers.ll, before.ll);
        assert_eq!(m.cpu.registers.coprocessor_select_register, 0);
        assert!(m.dma_accesses().is_empty());
    }
}

#[test]
fn reserved_dma_commands_fail_without_starting_bus_work() {
    for operation in 0..16 {
        if (8..=10).contains(&operation) {
            continue;
        }
        let command = 0x2000 | operation << 8;
        let mut m = Machine::new(command, false);
        for _ in 0..7 {
            m.clock(BusAssertions::default(), false);
        }
        assert_eq!(
            m.cpu.eu_registers.pending_fault.unwrap().fault,
            Faults::InvalidOpCode
        );
        assert_eq!(
            m.cpu.eu_registers.link_registers[7].return_address,
            u32::from(command)
        );
        assert!(m.dma_accesses().is_empty());
    }
    for command in [0x28c1, 0x29c1] {
        let mut m = Machine::new(command, false);
        for _ in 0..7 {
            m.clock(BusAssertions::default(), false);
        }
        assert_eq!(
            m.cpu.eu_registers.pending_fault.unwrap().fault,
            Faults::InvalidOpCode
        );
        assert!(m.dma_accesses().is_empty());
    }
}

#[test]
fn dma_writes_cannot_bypass_bus_segment_protection() {
    let mut m = Machine::new(0x2901, false);
    m.cpu.registers.al = 0;
    let mut bus = new_bus_peripheral(Box::new(m.cpu));
    bus.map_segment(
        "PROGRAM",
        0,
        65_536,
        true,
        Box::new(new_stub_memory_mapped_device()),
    );
    for (address, value) in m.memory {
        bus.write_address(address, value);
    }
    let mut target = new_stub_memory_mapped_device();
    target.write_address(0, 0x1234);
    bus.map_segment("ROM", 0x0001_0000, 65_536, false, Box::new(target));
    let mut pins = BusAssertions::default();
    for _ in 0..20 {
        pins = bus.poll_all(pins);
        let cpu = bus
            .bus_master
            .as_any()
            .downcast_mut::<CpuPeripheral>()
            .unwrap();
        if let Some(fault) = cpu.eu_registers.pending_fault {
            assert_eq!(fault.fault, Faults::BusProtection);
            assert_eq!(fault.pc, 0xcd00_0200);
            let metadata = cpu.eu_registers.link_registers[7];
            assert_eq!(metadata.return_address, 0xab01_0000);
            assert_eq!(
                decode_fault_metadata_register(metadata.return_status_register).bus_access_type,
                BusAccessType::DmaWriteBurst
            );
            assert_eq!(bus.read_address(0x0001_0000), 0x1234);
            return;
        }
    }
    panic!("ROM write did not fault");
}

struct NeverAck;
impl MemoryMapped for NeverAck {
    fn read_address(&self, _: u32) -> u16 {
        0
    }
    fn write_address(&mut self, _: u32, _: u16) {
        panic!("unacknowledged write must not commit");
    }
}
impl MemoryMappedDevice for NeverAck {}
impl Device for NeverAck {
    fn poll(&mut self, pins: BusAssertions, selected: bool) -> BusAssertions {
        BusAssertions {
            device_was_activated: selected && pins.bus_access_strobe,
            ..Default::default()
        }
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[test]
fn reset_aborts_a_held_dma_access_and_cannot_resurrect_the_command() {
    let m = Machine::new(0x2907, false);
    let mut bus = new_bus_peripheral(Box::new(m.cpu));
    bus.map_segment(
        "PROGRAM",
        0,
        65_536,
        true,
        Box::new(new_stub_memory_mapped_device()),
    );
    for (address, value) in m.memory {
        bus.write_address(address, value);
    }
    bus.write_address(0, 0);
    bus.write_address(1, 0x0400);
    bus.map_segment("WAIT", 0x0001_0000, 65_536, true, Box::new(NeverAck));
    let mut pins = BusAssertions::default();
    for _ in 0..10 {
        pins = bus.poll_all(pins);
    }
    assert_eq!(pins.bus_access_type, BusAccessType::DmaWriteBurst);
    assert!(!pins.bus_acknowledge);
    pins.reset_requested = true;
    for _ in 0..6 {
        pins = bus.poll_all(pins);
        assert!(pins.reset_devices_on_bus);
        assert!(!pins.bus_access_strobe && !pins.instruction_sync);
    }
    let mut vector_words = 0;
    for _ in 0..20 {
        pins = bus.poll_all(pins);
        assert_ne!(pins.bus_access_type, BusAccessType::DmaWriteBurst);
        if pins.bus_access_type == BusAccessType::ExceptionVectorFetch {
            vector_words += 1;
        }
        if pins.bus_access_type == BusAccessType::InstructionFetch {
            assert_eq!(pins.address, 0x0400);
            assert_eq!(vector_words, 2);
            let cpu = bus
                .bus_master
                .as_any()
                .downcast_mut::<CpuPeripheral>()
                .unwrap();
            assert_eq!(cpu.registers.coprocessor_select_register, 0);
            return;
        }
    }
    panic!("reset did not resume at its vector");
}
