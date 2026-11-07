use super::common::tick;
use peripheral_bus::device::{BusAccessType, BusAssertions, Device};
use peripheral_cpu::{
    coprocessors::{optional::CoprocessorFault, shared::ExecutionPhase},
    new_cpu_peripheral_with_config,
    registers::Registers,
    CpuConfig, OptionalCoprocessor,
};

struct Probe {
    id: u8,
    finished: bool,
}
impl OptionalCoprocessor for Probe {
    fn id(&self) -> u8 {
        self.id
    }
    fn start(&mut self, _: u16, _: &mut Registers) -> Result<(), CoprocessorFault> {
        self.finished = false;
        Ok(())
    }
    fn step(
        &mut self,
        _: ExecutionPhase,
        _: &mut Registers,
    ) -> Result<BusAssertions, CoprocessorFault> {
        Ok(if self.finished {
            BusAssertions::default()
        } else {
            BusAssertions {
                address: 0x100,
                bus_access_strobe: true,
                bus_access_type: BusAccessType::DmaReadBurst,
                ..Default::default()
            }
        })
    }
    fn complete_access(
        &mut self,
        _: BusAssertions,
        response: BusAssertions,
        registers: &mut Registers,
    ) -> Result<(), CoprocessorFault> {
        registers.r1 = response.data;
        self.finished = true;
        Ok(())
    }
    fn is_finished(&self) -> bool {
        self.finished
    }
    fn reset(&mut self) {
        self.finished = false;
    }
}
fn config(ids: &[u8]) -> CpuConfig {
    CpuConfig {
        optional_coprocessors: ids
            .iter()
            .map(|id| {
                Box::new(Probe {
                    id: *id,
                    finished: false,
                }) as Box<dyn OptionalCoprocessor>
            })
            .collect(),
    }
}
#[test]
fn optional_registration_cannot_replace_required_units_or_duplicate_ids() {
    for ids in [&[0][..], &[1], &[16], &[3, 3]] {
        assert!(new_cpu_peripheral_with_config(config(ids)).is_err());
    }
    assert!(new_cpu_peripheral_with_config(config(&[2, 3])).is_ok());
}
#[test]
fn plugins_share_bus_waits_and_retain_command_ownership_until_the_slot_finishes() {
    for interrupt in [0, 16] {
        let mut cpu = new_cpu_peripheral_with_config(config(&[3])).unwrap();
        cpu.registers.pl = 0x0200;
        cpu.registers.coprocessor_select_register = 0x3000;
        let setup = tick(&mut cpu, 0);
        assert!(!setup.bus_access_strobe);
        assert!(!setup.instruction_sync);
        assert_eq!(tick(&mut cpu, 0).address, 0x100);
        for _ in 0..3 {
            let held = cpu.poll(
                BusAssertions {
                    interrupt_assertion: interrupt,
                    ..Default::default()
                },
                true,
            );
            assert_eq!(held.address, 0x100);
            assert!(!held.instruction_sync);
            assert_eq!(cpu.registers.r1, 0);
        }
        for data in [0xcafe, 0, 0, 0] {
            assert!(!tick(&mut cpu, data).bus_access_strobe);
            assert_eq!(cpu.registers.coprocessor_select_register, 0x3000);
        }
        assert_eq!(cpu.registers.r1, 0xcafe);
        let next = tick(&mut cpu, 0);
        assert_eq!(cpu.registers.coprocessor_select_register, 0);
        assert_eq!(next.address, if interrupt == 0 { 0x0200 } else { 0x20 });
        assert!(next.instruction_sync);
    }
}
#[test]
fn bus_errors_abort_plugins_without_delivering_faulty_data() {
    for protection in [false, true] {
        let mut cpu = new_cpu_peripheral_with_config(config(&[3])).unwrap();
        cpu.registers.coprocessor_select_register = 0x3000;
        tick(&mut cpu, 0);
        tick(&mut cpu, 0);
        let out = cpu.poll(
            BusAssertions {
                bus_error: !protection,
                bus_protection_error: protection,
                bus_acknowledge: true,
                data: 0xcafe,
                ..Default::default()
            },
            true,
        );
        assert!(!out.bus_access_strobe);
        assert_eq!(cpu.registers.r1, 0);
        assert_eq!(cpu.registers.coprocessor_select_register, 0);
        assert_eq!(cpu.eu_registers.link_registers[7].return_address, 0x100);
        while cpu.phase != 0 {
            assert!(!tick(&mut cpu, 0).bus_access_strobe);
        }
        assert_eq!(tick(&mut cpu, 0).address, if protection { 18 } else { 2 });
    }
}
