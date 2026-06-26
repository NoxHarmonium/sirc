use peripheral_bus::{
    memory_mapped_device::new_stub_memory_mapped_device, new_bus_peripheral, BusPeripheral,
};
use peripheral_cpu::{
    coprocessors::{
        exception_unit::definitions::Faults,
        processing_unit::{
            definitions::{
                ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
            },
            encoding::encode_instruction,
        },
    },
    decode_fault_metadata_register, new_cpu_peripheral,
    registers::{FullAddressRegisterAccess, Registers},
    CpuPeripheral,
};

const PROGRAM_SEGMENT: &str = "PROGRAM";
const SCRATCH_SEGMENT: &str = "SCRATCH";
const PROGRAM_ADDRESS: u32 = 0x0000_0100;
const SCRATCH_ADDRESS: u32 = 0x00FA_0000;

fn setup_dma_test(command: u16) -> BusPeripheral {
    let cpu = new_cpu_peripheral(0x0);
    let mut bus = new_bus_peripheral(Box::new(cpu));

    bus.map_segment(
        PROGRAM_SEGMENT,
        PROGRAM_ADDRESS,
        u16::MAX as u32 + 1,
        false,
        Box::new(new_stub_memory_mapped_device()),
    );
    bus.map_segment(
        SCRATCH_SEGMENT,
        SCRATCH_ADDRESS,
        u16::MAX as u32 + 1,
        true,
        Box::new(new_stub_memory_mapped_device()),
    );

    let program = encode_instruction(&InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::CoprocessorCallImmediate,
        register: 0,
        value: command,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0,
    }));
    bus.load_binary_data_into_segment(PROGRAM_SEGMENT, &program);

    with_cpu(&mut bus, |cpu| {
        cpu.registers.set_full_pc_address(PROGRAM_ADDRESS);
    });

    bus
}

fn with_cpu<F>(bus: &mut BusPeripheral, callback: F)
where
    F: FnOnce(&mut CpuPeripheral),
{
    let cpu = bus
        .bus_master
        .as_any()
        .downcast_mut::<CpuPeripheral>()
        .expect("failed to downcast CPU");
    callback(cpu);
}

fn cpu_registers(bus: &mut BusPeripheral) -> Registers {
    let mut registers = Registers::default();
    with_cpu(bus, |cpu| {
        registers = cpu.registers;
    });
    registers
}

#[test]
fn dmar_reads_memory_into_register_window_and_advances_address() {
    let mut bus = setup_dma_test(0x2803);
    bus.write_address(SCRATCH_ADDRESS, 0x1111);
    bus.write_address(SCRATCH_ADDRESS + 1, 0x2222);
    bus.write_address(SCRATCH_ADDRESS + 2, 0x3333);

    with_cpu(&mut bus, |cpu| {
        cpu.registers.set_full_address_address(SCRATCH_ADDRESS);
    });

    bus.run_full_cycle(16);
    let registers = cpu_registers(&mut bus);

    eprintln!("{registers:#x?}");
    assert_eq!(registers.r1, 0x1111);
    assert_eq!(registers.r2, 0x2222);
    assert_eq!(registers.r3, 0x3333);
    assert_eq!(registers.get_full_address_address(), SCRATCH_ADDRESS + 3);
    assert_eq!(registers.pending_coprocessor_command, 0);
}

#[test]
fn dmaw_writes_register_window_and_advances_address() {
    let mut bus = setup_dma_test(0x2943);

    with_cpu(&mut bus, |cpu| {
        cpu.registers.set_full_link_address(SCRATCH_ADDRESS + 0x10);
        cpu.registers.r1 = 0xAAAA;
        cpu.registers.r2 = 0xBBBB;
        cpu.registers.r3 = 0xCCCC;
    });

    bus.run_full_cycle(16);
    let registers = cpu_registers(&mut bus);

    assert_eq!(bus.read_address(SCRATCH_ADDRESS + 0x10), 0xAAAA);
    assert_eq!(bus.read_address(SCRATCH_ADDRESS + 0x11), 0xBBBB);
    assert_eq!(bus.read_address(SCRATCH_ADDRESS + 0x12), 0xCCCC);
    assert_eq!(registers.get_full_link_address(), SCRATCH_ADDRESS + 0x13);
}

#[test]
fn negative_dmaw_then_positive_dmar_behaves_like_bulk_stack_push_pop() {
    let mut bus = setup_dma_test(0x29A3);

    with_cpu(&mut bus, |cpu| {
        cpu.registers.set_full_sp_address(SCRATCH_ADDRESS + 0x20);
        cpu.registers.r1 = 0x1001;
        cpu.registers.r2 = 0x1002;
        cpu.registers.r3 = 0x1003;
    });

    bus.run_full_cycle(16);
    let registers_after_push = cpu_registers(&mut bus);
    assert_eq!(
        registers_after_push.get_full_sp_address(),
        SCRATCH_ADDRESS + 0x1D
    );
    assert_eq!(bus.read_address(SCRATCH_ADDRESS + 0x1D), 0x1001);
    assert_eq!(bus.read_address(SCRATCH_ADDRESS + 0x1E), 0x1002);
    assert_eq!(bus.read_address(SCRATCH_ADDRESS + 0x1F), 0x1003);

    let mut bus = setup_dma_test(0x2883);
    with_cpu(&mut bus, |cpu| {
        cpu.registers.set_full_sp_address(SCRATCH_ADDRESS + 0x1D);
    });
    bus.write_address(SCRATCH_ADDRESS + 0x1D, 0x1001);
    bus.write_address(SCRATCH_ADDRESS + 0x1E, 0x1002);
    bus.write_address(SCRATCH_ADDRESS + 0x1F, 0x1003);

    bus.run_full_cycle(16);
    let registers_after_pop = cpu_registers(&mut bus);
    assert_eq!(registers_after_pop.r1, 0x1001);
    assert_eq!(registers_after_pop.r2, 0x1002);
    assert_eq!(registers_after_pop.r3, 0x1003);
    assert_eq!(
        registers_after_pop.get_full_sp_address(),
        SCRATCH_ADDRESS + 0x20
    );
}

#[test]
fn dmat_copies_memory_to_memory_using_a_to_l() {
    let mut bus = setup_dma_test(0x2A08);
    for index in 0..8 {
        bus.write_address(SCRATCH_ADDRESS + index, 0xAA00 | index as u16);
    }

    with_cpu(&mut bus, |cpu| {
        cpu.registers.set_full_address_address(SCRATCH_ADDRESS);
        cpu.registers.set_full_link_address(SCRATCH_ADDRESS + 0x100);
    });

    bus.run_full_cycle(40);
    let registers = cpu_registers(&mut bus);

    for index in 0..8 {
        assert_eq!(
            bus.read_address(SCRATCH_ADDRESS + 0x100 + index),
            0xAA00 | index as u16
        );
    }
    assert_eq!(registers.get_full_address_address(), SCRATCH_ADDRESS + 8);
    assert_eq!(registers.get_full_link_address(), SCRATCH_ADDRESS + 0x108);
    assert_eq!(registers.r1, 0xAA07);
}

#[test]
fn zero_count_dma_command_clears_without_memory_access() {
    let mut bus = setup_dma_test(0x2800);
    bus.write_address(SCRATCH_ADDRESS, 0xCAFE);

    with_cpu(&mut bus, |cpu| {
        cpu.registers.set_full_address_address(SCRATCH_ADDRESS);
        cpu.registers.r1 = 0x1234;
    });

    bus.run_full_cycle(8);
    let registers = cpu_registers(&mut bus);

    assert_eq!(registers.r1, 0x1234);
    assert_eq!(registers.get_full_address_address(), SCRATCH_ADDRESS);
    assert_eq!(bus.read_address(SCRATCH_ADDRESS), 0xCAFE);
    assert_eq!(registers.pending_coprocessor_command, 0);
}

#[test]
fn reserved_dma_operand_raises_invalid_opcode_fault() {
    let mut bus = setup_dma_test(0x2818);

    bus.run_full_cycle(8);

    with_cpu(&mut bus, |cpu| {
        assert_eq!(cpu.eu_registers.pending_fault, Some(Faults::InvalidOpCode));
        assert_eq!(cpu.registers.pending_coprocessor_command, 0);
    });
}

#[test]
fn dma_bus_error_records_dma_bat_in_fault_metadata() {
    let mut bus = setup_dma_test(0x2801);

    with_cpu(&mut bus, |cpu| {
        cpu.registers.set_full_address_address(0x00FC_0000);
    });

    bus.run_full_cycle(10);

    with_cpu(&mut bus, |cpu| {
        assert_eq!(cpu.eu_registers.pending_fault, Some(Faults::Bus));
        let metadata = decode_fault_metadata_register(
            cpu.eu_registers.link_registers[7].return_status_register,
        );
        assert_eq!(
            metadata.bus_access_type,
            peripheral_bus::device::BusAccessType::DmaReadBurst
        );
        assert_eq!(metadata.fault, Faults::Bus);
    });
}
