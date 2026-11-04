use super::common::{complete_entry, tick};
use peripheral_bus::{
    conversion::bytes_to_words,
    device::{BusAccessType, BusAssertions, Device},
};
use peripheral_cpu::{
    coprocessors::{
        exception_unit::definitions::{vectors::SEGMENT_OVERFLOW_FAULT, Faults},
        processing_unit::{
            definitions::{
                ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
                RegisterInstructionData, ShiftOperand, ShiftType,
            },
            encoding::encode_instruction,
        },
    },
    decode_fault_metadata_register, new_cpu_peripheral,
    registers::{
        set_hardware_interrupt_enable, set_sr_bit, AddressRegisterName, FullAddressRegisterAccess,
        RegisterName, Registers, StatusRegisterFields,
    },
    CpuPeripheral,
};

fn immediate(
    op_code: Instruction,
    register: u8,
    value: u16,
    source: &AddressRegisterName,
    condition_flag: ConditionFlags,
) -> InstructionData {
    InstructionData::Immediate(ImmediateInstructionData {
        op_code,
        register,
        value,
        additional_flags: source.to_register_index(),
        condition_flag,
    })
}

fn cpu_at_segment_end() -> CpuPeripheral {
    let mut cpu = new_cpu_peripheral();
    cpu.registers.set_full_pc_address(0xcd34_fffe);
    cpu.registers.sh = 0xab12;
    cpu.registers.sl = 0x0800;
    cpu.registers.r1 = 0x1900;
    cpu.registers.lh = 0x7777;
    cpu.registers.ll = 0x8888;
    set_sr_bit(
        StatusRegisterFields::TrapOnAddressOverflow,
        &mut cpu.registers,
    );
    cpu
}

fn run_through_phase_three(
    cpu: &mut CpuPeripheral,
    instruction: &InstructionData,
    force_trace: bool,
) {
    let words = bytes_to_words(&encode_instruction(instruction));
    for (phase, data) in [0, words[0], words[1], 0].into_iter().enumerate() {
        let out = cpu.poll(
            BusAssertions {
                data,
                force_trace_mode: force_trace && phase == 0,
                bus_acknowledge: true,
                address: 0xdead,
                bus_access_type: BusAccessType::Reserved,
                ..Default::default()
            },
            true,
        );
        assert_eq!(out.bus_access_strobe, phase < 2);
        if phase < 2 {
            assert_eq!(out.address, 0x0034_fffe + u32::try_from(phase).unwrap());
        }
        if phase == 2 {
            assert!(cpu.eu_registers.pending_fault.is_none());
        }
    }
    assert_eq!(cpu.phase, 4);
}

fn assert_cancelled(
    cpu: &mut CpuPeripheral,
    before: Registers,
    address: u32,
    bat: BusAccessType,
    irq: u8,
) {
    let pending = cpu.eu_registers.pending_fault.unwrap();
    assert_eq!(pending.fault, Faults::SegmentOverflow);
    assert_eq!(pending.pc, 0xcd34_fffe);
    let link = cpu.eu_registers.link_registers[7];
    assert_eq!(link.return_address, address);
    assert_eq!(
        decode_fault_metadata_register(link.return_status_register).bus_access_type,
        bat
    );
    for phase in 4..6 {
        let out = cpu.poll(
            BusAssertions {
                bus_acknowledge: true,
                interrupt_assertion: if phase == 5 { irq } else { 0 },
                ..Default::default()
            },
            true,
        );
        assert!(!out.bus_access_strobe);
    }
    let mut expected = before;
    expected.pl = 0;
    assert_eq!(cpu.registers, expected);
    assert!(!cpu.eu_registers.waiting_for_exception);
    assert!(!cpu.take_reset_request());
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, irq);
    complete_entry(cpu, SEGMENT_OVERFLOW_FAULT, 0x1000);
    assert_eq!(
        cpu.eu_registers.link_registers[6].return_address,
        0xcd34_fffe
    );
    // Neither trace nor a pending IRQ may inherit a stale wrap at handler fetch.
    assert_eq!(tick(cpu, 0).address, 0x1000);
    assert!(cpu.eu_registers.pending_fault.is_none());
}

#[test]
fn pc_wrap_cancels_the_owning_instruction_before_effects_and_before_trace_or_irq() {
    use AddressRegisterName::{Address, ProgramCounter, StackPointer};
    use ConditionFlags::{Always, Never};
    use Instruction::{
        AddImmediate, CompareImmediate, CoprocessorCallImmediate,
        LoadEffectiveAddressAndLinkFromIndirectImmediate as Link,
        LoadEffectiveAddressAndLinkFromIndirectImmediatePostIncrement as LinkPost,
        LoadEffectiveAddressFromIndirectImmediate as Branch,
        LoadRegisterFromIndirectImmediate as Load, MoveRegisterFromImmediate as Move,
        StoreRegisterToIndirectImmediate as Store,
    };
    let r1 = RegisterName::R1.to_register_index();
    let pl = RegisterName::Pl.to_register_index();
    let p = ProgramCounter.to_register_index();
    let cases = [
        immediate(
            AddImmediate,
            r1,
            1,
            &AddressRegisterName::LinkRegister,
            Always,
        ),
        immediate(Store, r1, 0, &StackPointer, Always),
        immediate(Load, r1, 0, &StackPointer, Always),
        immediate(
            CoprocessorCallImmediate,
            0,
            0x1900,
            &AddressRegisterName::LinkRegister,
            Always,
        ),
        immediate(
            CoprocessorCallImmediate,
            0,
            0x1b00,
            &AddressRegisterName::LinkRegister,
            Always,
        ),
        immediate(
            CoprocessorCallImmediate,
            0,
            0x1160,
            &AddressRegisterName::LinkRegister,
            Always,
        ),
        immediate(
            CoprocessorCallImmediate,
            0,
            0x2000,
            &AddressRegisterName::LinkRegister,
            Always,
        ),
        immediate(Branch, p, 0xfffe, &ProgramCounter, Never),
        immediate(Link, p, 0xfffe, &ProgramCounter, Always), // BRSR
        immediate(Link, Address.to_register_index(), 0, &StackPointer, Always), // LDEL
        immediate(LinkPost, p, 0, &StackPointer, Always),    // LJSR
        immediate(
            CompareImmediate,
            pl,
            0,
            &AddressRegisterName::LinkRegister,
            Always,
        ), // reads pl only
        immediate(Move, pl, 0x0200, &AddressRegisterName::LinkRegister, Never),
        immediate(Load, pl, 0, &StackPointer, Never),
        InstructionData::Register(RegisterInstructionData {
            op_code: Instruction::CoprocessorCallRegister,
            r1: 0,
            r2: 0,
            r3: r1,
            shift_operand: ShiftOperand::Immediate,
            shift_type: ShiftType::None,
            shift_count: 0,
            condition_flag: Always,
            additional_flags: 0,
        }),
    ];
    for instruction in cases {
        for trace in 0..3 {
            for irq in [0, 1] {
                let mut cpu = cpu_at_segment_end();
                set_hardware_interrupt_enable(&mut cpu.registers, irq);
                if trace == 1 {
                    set_sr_bit(StatusRegisterFields::TraceMode, &mut cpu.registers);
                }
                let before = cpu.registers;
                run_through_phase_three(&mut cpu, &instruction, trace == 2);
                assert_cancelled(&mut cpu, before, 0xcd34_0000, BusAccessType::None, irq);
            }
        }
    }
}

#[test]
fn executed_non_link_jumps_and_alu_or_load_writes_to_pl_do_not_use_advanced_pc() {
    use AddressRegisterName::{LinkRegister, ProgramCounter, StackPointer};
    use ConditionFlags::Always;
    let pl = RegisterName::Pl.to_register_index();
    for (instruction, high, low, memory) in [
        (
            immediate(
                Instruction::LoadEffectiveAddressFromIndirectImmediate,
                ProgramCounter.to_register_index(),
                0xfffe,
                &ProgramCounter,
                Always,
            ),
            0xcd34,
            0xfffc,
            false,
        ),
        (
            immediate(
                Instruction::LoadEffectiveAddressFromIndirectImmediate,
                ProgramCounter.to_register_index(),
                0,
                &StackPointer,
                Always,
            ),
            0xab12,
            0x0800,
            false,
        ),
        (
            immediate(
                Instruction::MoveRegisterFromImmediate,
                pl,
                0x0200,
                &LinkRegister,
                Always,
            ),
            0xcd34,
            0x0200,
            false,
        ),
        (
            immediate(
                Instruction::LoadRegisterFromIndirectImmediate,
                pl,
                0,
                &StackPointer,
                Always,
            ),
            0xcd34,
            0x0200,
            true,
        ),
        (
            immediate(
                Instruction::LoadRegisterFromIndirectImmediatePostIncrement,
                pl,
                0,
                &StackPointer,
                Always,
            ),
            0xcd34,
            0x0200,
            true,
        ),
    ] {
        let mut cpu = cpu_at_segment_end();
        run_through_phase_three(&mut cpu, &instruction, false);
        assert!(cpu.eu_registers.pending_fault.is_none());
        assert_eq!(tick(&mut cpu, 0).bus_access_strobe, memory);
        tick(&mut cpu, 0x0200);
        assert!(cpu.eu_registers.pending_fault.is_none());
        assert_eq!(cpu.registers.ph, high);
        assert_eq!(cpu.registers.pl, low);
        assert_eq!(
            tick(&mut cpu, 0).address,
            (u32::from(high & 0xff) << 16) | u32::from(low)
        );
        assert!(cpu.eu_registers.pending_fault.is_none());
    }
}

#[test]
fn effective_address_then_update_then_pc_is_the_single_wrap_fault_order() {
    use AddressRegisterName::StackPointer;
    use ConditionFlags::Always;
    let r1 = RegisterName::R1.to_register_index();
    for (op, low, offset, address, bat) in [
        (
            Instruction::LoadRegisterFromIndirectImmediate,
            0,
            0xffff,
            0xab12_ffff,
            BusAccessType::DataRead,
        ),
        (
            Instruction::LoadRegisterFromIndirectImmediatePostIncrement,
            0xffff,
            0xffff,
            0xab12_0000,
            BusAccessType::DataRead,
        ),
        (
            Instruction::StoreRegisterToIndirectImmediatePreDecrement,
            0,
            1,
            0xab12_0000,
            BusAccessType::DataWrite,
        ),
    ] {
        let mut cpu = cpu_at_segment_end();
        cpu.registers.sl = low;
        let instruction = immediate(op, r1, offset, &StackPointer, Always);
        let before = cpu.registers;
        run_through_phase_three(&mut cpu, &instruction, false);
        assert_cancelled(&mut cpu, before, address, bat, 0);
    }
}

#[test]
fn pc_advance_wraps_normally_when_trapping_is_disabled() {
    let mut cpu = cpu_at_segment_end();
    cpu.registers.sr = 0;
    let instruction = immediate(
        Instruction::AddImmediate,
        RegisterName::R1.to_register_index(),
        1,
        &AddressRegisterName::LinkRegister,
        ConditionFlags::Always,
    );
    run_through_phase_three(&mut cpu, &instruction, false);
    tick(&mut cpu, 0);
    tick(&mut cpu, 0);
    assert!(cpu.eu_registers.pending_fault.is_none());
    assert_eq!(cpu.registers.r1, 0x1901);
    assert_eq!(cpu.registers.get_full_pc_address(), 0xcd34_0000);
    assert_eq!(tick(&mut cpu, 0).address, 0x0034_0000);
    assert!(cpu.eu_registers.pending_fault.is_none());
}
