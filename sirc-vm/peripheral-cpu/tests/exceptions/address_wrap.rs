use super::common::tick;
use peripheral_bus::conversion::bytes_to_words;
use peripheral_cpu::{
    coprocessors::{
        exception_unit::definitions::Faults,
        processing_unit::{
            definitions::{
                ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
                RegisterInstructionData, ShiftOperand, ShiftType,
            },
            encoding::encode_instruction,
        },
    },
    new_cpu_peripheral,
    registers::{set_sr_bit, AddressRegisterName, RegisterName, StatusRegisterFields},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Destination {
    Load,
    Store,
    Address,
    Link,
    Jump,
}

#[test]
fn effective_address_and_source_update_wrap_independently() {
    use Destination::{Address, Jump, Link, Load, Store};
    use Instruction::{
        LoadEffectiveAddressAndLinkFromIndirectImmediate as LinkImmediate,
        LoadEffectiveAddressAndLinkFromIndirectImmediatePostIncrement as LinkPostImmediate,
        LoadEffectiveAddressAndLinkFromIndirectRegister as LinkRegister,
        LoadEffectiveAddressAndLinkFromIndirectRegisterPostIncrement as LinkPostRegister,
        LoadEffectiveAddressFromIndirectImmediate as AddressImmediate,
        LoadEffectiveAddressFromIndirectImmediatePreDecrement as AddressPreImmediate,
        LoadEffectiveAddressFromIndirectRegister as AddressRegister,
        LoadEffectiveAddressFromIndirectRegisterPreDecrement as AddressPreRegister,
        LoadRegisterFromIndirectImmediate as LoadImmediate,
        LoadRegisterFromIndirectImmediatePostIncrement as LoadPostImmediate,
        LoadRegisterFromIndirectRegister as LoadRegister,
        LoadRegisterFromIndirectRegisterPostIncrement as LoadPostRegister,
        StoreRegisterToIndirectImmediate as StoreImmediate,
        StoreRegisterToIndirectImmediatePreDecrement as StorePreImmediate,
        StoreRegisterToIndirectRegister as StoreRegister,
        StoreRegisterToIndirectRegisterPreDecrement as StorePreRegister,
    };
    // Includes the reported FFFE + #1 post-increment false positive, FFFF +
    // #-1 post-increment missed wrap, and 0 / #2 pre-decrement missed wrap.
    for (immediate, register, destination, increment) in [
        (LoadImmediate, LoadRegister, Load, 0),
        (LoadPostImmediate, LoadPostRegister, Load, 1),
        (StoreImmediate, StoreRegister, Store, 0),
        (StorePreImmediate, StorePreRegister, Store, -1),
        (AddressImmediate, AddressRegister, Address, 0),
        (AddressPreImmediate, AddressPreRegister, Address, -1),
        (LinkImmediate, LinkRegister, Link, 0),
        (LinkPostImmediate, LinkPostRegister, Link, 1),
        // LJSR is the linking address operation with p as destination.
        (LinkPostImmediate, LinkPostRegister, Jump, 1),
    ] {
        for low in [0u16, 1, 0x7fff, 0x8000, 0xfffe, 0xffff] {
            for offset in [i16::MIN, -2, -1, 0, 1, 2, i16::MAX] {
                for register_offset in [false, true] {
                    for trap in [false, true] {
                        for execute in [false, true] {
                            let mut cpu = new_cpu_peripheral();
                            cpu.registers.ph = 0xcd34;
                            cpu.registers.pl = 0x0200;
                            cpu.registers.sh = 0xab12;
                            cpu.registers.sl = low;
                            cpu.registers.ah = 0xef56;
                            cpu.registers.al = 0x1234;
                            cpu.registers.lh = 0x7777;
                            cpu.registers.ll = 0x8888;
                            cpu.registers.r1 = 0xcafe;
                            cpu.registers.r2 = offset.cast_unsigned();
                            if trap {
                                set_sr_bit(
                                    StatusRegisterFields::TrapOnAddressOverflow,
                                    &mut cpu.registers,
                                );
                            }
                            let target = match destination {
                                Load | Store => RegisterName::R1.to_register_index(),
                                Address | Link => AddressRegisterName::Address.to_register_index(),
                                Jump => AddressRegisterName::ProgramCounter.to_register_index(),
                            };
                            let condition_flag = if execute {
                                ConditionFlags::Always
                            } else {
                                ConditionFlags::Never
                            };
                            let source = AddressRegisterName::StackPointer.to_register_index();
                            let instruction = if register_offset {
                                InstructionData::Register(RegisterInstructionData {
                                    op_code: register,
                                    r1: if destination == Store { 0 } else { target },
                                    r2: if destination == Store { target } else { 0 },
                                    r3: RegisterName::R2.to_register_index(),
                                    shift_operand: ShiftOperand::Immediate,
                                    shift_type: ShiftType::None,
                                    shift_count: 0,
                                    condition_flag,
                                    additional_flags: source,
                                })
                            } else {
                                InstructionData::Immediate(ImmediateInstructionData {
                                    op_code: immediate,
                                    register: target,
                                    value: offset.cast_unsigned(),
                                    condition_flag,
                                    additional_flags: source,
                                })
                            };
                            // Reference calculations use a wider signed range, rather than
                            // the production overflowing 16-bit arithmetic.
                            let update = i32::from(low) + increment;
                            let updated = u16::try_from(update.rem_euclid(65_536)).unwrap();
                            let base = if increment == -1 { updated } else { low };
                            let effective = i32::from(base) + i32::from(offset);
                            let ea = u16::try_from(effective.rem_euclid(65_536)).unwrap();
                            let wrapped = !(0..=65_535).contains(&update)
                                || !(0..=65_535).contains(&effective);
                            let fault = execute && trap && wrapped;
                            let words = bytes_to_words(&encode_instruction(&instruction));
                            let mut expected = cpu.registers;
                            expected.pl = 0x0202;
                            for (phase, data) in [0, words[0], words[1], 0, 0, 0xbeef]
                                .into_iter()
                                .enumerate()
                            {
                                let out = tick(&mut cpu, data);
                                let data_access = phase == 4
                                    && execute
                                    && !fault
                                    && matches!(destination, Load | Store);
                                assert_eq!(out.bus_access_strobe, phase < 2 || data_access);
                                if data_access {
                                    assert_eq!(out.address, 0x0012_0000 | u32::from(ea));
                                    if destination == Store {
                                        assert_eq!(out.data, 0xcafe);
                                    }
                                }
                            }
                            assert_eq!(cpu.phase, 0);
                            assert_eq!(cpu.eu_registers.pending_fault.is_some(), fault,
                                "{destination:?} low={low:04x} offset={offset} increment={increment} register={register_offset} trap={trap} execute={execute}");
                            if fault {
                                let pending = cpu.eu_registers.pending_fault.unwrap();
                                assert_eq!(pending.fault, Faults::SegmentOverflow);
                                assert_eq!(pending.pc, 0xcd34_0200);
                            } else if execute {
                                expected.sl = updated;
                                match destination {
                                    Load => expected.r1 = 0xbeef,
                                    Store => {}
                                    Address | Link => {
                                        expected.ah = 0xab12;
                                        expected.al = ea;
                                    }
                                    Jump => {
                                        expected.ph = 0xab12;
                                        expected.pl = ea;
                                    }
                                }
                                if matches!(destination, Link | Jump) {
                                    expected.lh = 0xcd34;
                                    expected.ll = 0x0202;
                                }
                            }
                            assert_eq!(
                                cpu.registers, expected,
                                "faults/skipped calls must not partially write registers"
                            );
                            if execute {
                                assert_eq!(
                                    cpu.processing_unit_executor
                                        .intermediate_registers
                                        .alu_output,
                                    ea
                                );
                                assert_eq!(
                                    cpu.processing_unit_executor
                                        .intermediate_registers
                                        .address_output,
                                    if increment == 0 { ea } else { updated }
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
