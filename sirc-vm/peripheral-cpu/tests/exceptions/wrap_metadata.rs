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
    registers::{set_sr_bit, AddressRegisterName, RegisterName, StatusRegisterFields},
    CpuPeripheral,
};

fn instruction(
    immediate: Instruction,
    register: Instruction,
    target: u8,
    offset: i16,
    use_register: bool,
) -> InstructionData {
    let additional_flags = AddressRegisterName::StackPointer.to_register_index();
    if use_register {
        InstructionData::Register(RegisterInstructionData {
            op_code: register,
            r1: target,
            r2: if matches!(
                register,
                Instruction::StoreRegisterToIndirectRegister
                    | Instruction::StoreRegisterToIndirectRegisterPreDecrement
            ) {
                target
            } else {
                0
            },
            r3: RegisterName::R2.to_register_index(),
            shift_operand: ShiftOperand::Immediate,
            shift_type: ShiftType::None,
            shift_count: 0,
            condition_flag: ConditionFlags::Always,
            additional_flags,
        })
    } else {
        InstructionData::Immediate(ImmediateInstructionData {
            op_code: immediate,
            register: target,
            value: offset.cast_unsigned(),
            condition_flag: ConditionFlags::Always,
            additional_flags,
        })
    }
}

fn detect_wrap(instruction: &InstructionData, low: u16, offset: i16) -> CpuPeripheral {
    let mut cpu = new_cpu_peripheral();
    cpu.registers.ph = 0xcd34;
    cpu.registers.pl = 0x0200;
    cpu.registers.sh = 0xab12;
    cpu.registers.sl = low;
    cpu.registers.r1 = 0xcafe;
    cpu.registers.r2 = offset.cast_unsigned();
    set_sr_bit(
        StatusRegisterFields::TrapOnAddressOverflow,
        &mut cpu.registers,
    );
    let before = cpu.registers;
    let words = bytes_to_words(&encode_instruction(instruction));
    for data in [0, words[0], words[1]] {
        tick(&mut cpu, data);
    }
    let response = cpu.poll(
        BusAssertions {
            address: 0xdead,
            bus_access_type: BusAccessType::Reserved,
            bus_acknowledge: true,
            ..Default::default()
        },
        true,
    );
    assert!(!response.bus_access_strobe);
    assert_eq!(cpu.phase, 4);
    assert_eq!(
        cpu.eu_registers.pending_fault.unwrap().fault,
        Faults::SegmentOverflow
    );
    assert_eq!(cpu.eu_registers.pending_fault.unwrap().pc, 0xcd34_0200);
    for _ in 0..2 {
        assert!(!tick(&mut cpu, 0).bus_access_strobe);
    }
    let mut expected = before;
    expected.pl = 0x0202;
    assert_eq!(cpu.registers, expected);
    cpu
}

fn assert_metadata(cpu: &mut CpuPeripheral, wrapped: u16, bat: BusAccessType) {
    let link = cpu.eu_registers.link_registers[7];
    assert_eq!(link.return_address, 0xab12_0000 | u32::from(wrapped));
    let metadata = decode_fault_metadata_register(link.return_status_register);
    assert_eq!(metadata.bus_access_type, bat);
    assert_eq!(metadata.fault, Faults::SegmentOverflow);
    complete_entry(cpu, SEGMENT_OVERFLOW_FAULT, 0x1000);
    assert_eq!(
        cpu.eu_registers.link_registers[6].return_address,
        0xcd34_0200
    );
}

#[test]
fn effective_address_wrap_metadata_describes_the_intended_access() {
    use Instruction::{
        LoadEffectiveAddressAndLinkFromIndirectImmediate as LinkI,
        LoadEffectiveAddressAndLinkFromIndirectImmediatePostIncrement as PostI,
        LoadEffectiveAddressAndLinkFromIndirectRegister as LinkR,
        LoadEffectiveAddressAndLinkFromIndirectRegisterPostIncrement as PostR,
        LoadEffectiveAddressFromIndirectImmediate as AddressI,
        LoadEffectiveAddressFromIndirectRegister as AddressR,
        LoadRegisterFromIndirectImmediate as LoadI, LoadRegisterFromIndirectRegister as LoadR,
        StoreRegisterToIndirectImmediate as StoreI, StoreRegisterToIndirectRegister as StoreR,
    };
    let a = AddressRegisterName::Address.to_register_index();
    let p = AddressRegisterName::ProgramCounter.to_register_index();
    for (immediate, register, target, bat) in [
        (
            LoadI,
            LoadR,
            RegisterName::R1.to_register_index(),
            BusAccessType::DataRead,
        ),
        (
            StoreI,
            StoreR,
            RegisterName::R1.to_register_index(),
            BusAccessType::DataWrite,
        ),
        (AddressI, AddressR, a, BusAccessType::None),
        // LDEA p and BRAN use the same instruction encoding.
        (AddressI, AddressR, p, BusAccessType::InstructionFetch),
        (LinkI, LinkR, a, BusAccessType::None),
        // Linking to p covers BRSR/LJSR, with or without source update.
        (LinkI, LinkR, p, BusAccessType::InstructionFetch),
        (PostI, PostR, a, BusAccessType::None),
        (PostI, PostR, p, BusAccessType::InstructionFetch),
    ] {
        for (low, offset, wrapped) in [(0, -1, 0xffff), (0xffff, 1, 0)] {
            for use_register in [false, true] {
                let data = instruction(immediate, register, target, offset, use_register);
                let mut cpu = detect_wrap(&data, low, offset);
                assert_metadata(&mut cpu, wrapped, bat);
            }
        }
    }
}

#[test]
fn source_wrap_metadata_uses_updated_pair_unless_the_effective_address_also_wraps() {
    use Instruction::{
        LoadEffectiveAddressAndLinkFromIndirectImmediatePostIncrement as LinkI,
        LoadEffectiveAddressAndLinkFromIndirectRegisterPostIncrement as LinkR,
        LoadEffectiveAddressFromIndirectImmediatePreDecrement as AddressI,
        LoadEffectiveAddressFromIndirectRegisterPreDecrement as AddressR,
        LoadRegisterFromIndirectImmediatePostIncrement as LoadI,
        LoadRegisterFromIndirectRegisterPostIncrement as LoadR,
        StoreRegisterToIndirectImmediatePreDecrement as StoreI,
        StoreRegisterToIndirectRegisterPreDecrement as StoreR,
    };
    let a = AddressRegisterName::Address.to_register_index();
    let p = AddressRegisterName::ProgramCounter.to_register_index();
    for (immediate, register, target, pre, bat) in [
        (
            LoadI,
            LoadR,
            RegisterName::R1.to_register_index(),
            false,
            BusAccessType::DataRead,
        ),
        (
            StoreI,
            StoreR,
            RegisterName::R1.to_register_index(),
            true,
            BusAccessType::DataWrite,
        ),
        (AddressI, AddressR, a, true, BusAccessType::None),
        (AddressI, AddressR, p, true, BusAccessType::InstructionFetch),
        (LinkI, LinkR, a, false, BusAccessType::None),
        (LinkI, LinkR, p, false, BusAccessType::InstructionFetch),
    ] {
        let cases = if pre {
            // First: source only. Second: both wrap, with distinguishable
            // wrapped values (EA=0, source=FFFF), so EA must win.
            [(0, 0, 0xffff), (0, 1, 0)]
        } else {
            [(0xffff, -1, 0), (0xffff, 1, 0)]
        };
        for (low, offset, wrapped) in cases {
            for use_register in [false, true] {
                let data = instruction(immediate, register, target, offset, use_register);
                let mut cpu = detect_wrap(&data, low, offset);
                assert_metadata(&mut cpu, wrapped, bat);
            }
        }
    }
}
