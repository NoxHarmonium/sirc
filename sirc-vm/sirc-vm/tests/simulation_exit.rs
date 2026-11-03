use device_debug::new_debug_device;
use device_ram::new_ram_device_standard;
use peripheral_bus::{
    device::{BusAccessType, BusAssertions},
    new_bus_peripheral,
};
use peripheral_cpu::{
    coprocessors::processing_unit::{
        definitions::{ConditionFlags, ImmediateInstructionData, Instruction, InstructionData},
        encoding::encode_instruction,
    },
    new_cpu_peripheral,
    registers::{AddressRegisterName, RegisterName},
};

#[test]
fn protected_mode_store_reaches_debug_exit_through_the_bus() {
    let instruction = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::StoreRegisterToIndirectImmediate,
        register: RegisterName::R1.to_register_index(),
        value: 7,
        condition_flag: ConditionFlags::Always,
        additional_flags: AddressRegisterName::Address.to_register_index(),
    });
    let mut cpu = new_cpu_peripheral();
    cpu.registers.sr = 0x0100; // Protected-mode MMIO is allowed by this device mapping.
    cpu.registers.ah = 0xB;
    cpu.registers.al = 0;
    cpu.registers.r1 = 1;
    let mut bus = new_bus_peripheral(Box::new(cpu));
    bus.map_segment(
        "PROGRAM",
        0,
        0xFFFF,
        false,
        Box::new(new_ram_device_standard()),
    );
    bus.load_binary_data_into_segment("PROGRAM", &encode_instruction(&instruction));
    bus.map_segment(
        "DEBUG",
        0x000B_0000,
        0xF,
        true,
        Box::new(new_debug_device()),
    );
    let mut response = BusAssertions::default();
    for _ in 0..24 {
        response = bus.poll_all(response);
        assert!(!response.bus_error);
        assert!(!response.bus_protection_error);
        if response.exit_simulation {
            assert!(response.bus_acknowledge);
            assert_eq!(response.address, 0x000B_0007);
            assert_eq!(response.bus_access_type, BusAccessType::DataWrite);
            return;
        }
    }
    panic!("debug exit store did not terminate within the bounded instruction run");
}
