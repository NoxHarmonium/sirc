use peripheral_bus::BusPeripheral;
use peripheral_cpu::{
    self,
    coprocessors::processing_unit::definitions::{
        ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
        RegisterInstructionData, ShiftOperand, ShiftType,
    },
    registers::{
        AddressRegisterIndexing, AddressRegisterName, FullAddress, RegisterIndexing, RegisterName,
        Registers, SegmentedAddress,
    },
};

use crate::instructions::common;

use super::common::{
    get_address_register_index_range, get_expected_registers, get_non_address_register_index_range,
};

// TODO: These long running LOAD/STORE tests with lots of permutations could probably be property based tests

#[allow(clippy::cast_sign_loss)]
#[test]
fn test_store_indirect_immediate() {
    for src_address_register_index in get_address_register_index_range() {
        if
        // TODO: Handle PC writes/reads etc.
        // It _should_ be valid to store a memory address offset from the PC but it breaks the test
        src_address_register_index == AddressRegisterName::ProgramCounter.to_register_index() {
            continue;
        }

        for src_register_index in get_non_address_register_index_range() {
            for offset in [
                0i16, -32i16, -64i16, -0x7FFFi16, -32768i16, 32i16, 64i16, 0x7FFFi16,
            ] {
                let instruction_data = InstructionData::Immediate(ImmediateInstructionData {
                    op_code: Instruction::StoreRegisterToIndirectImmediate,
                    register: src_register_index,
                    value: offset as u16,
                    condition_flag: ConditionFlags::Always,
                    additional_flags: src_address_register_index,
                });
                let calculated_address =
                    (0xFAFAu16, 0xFAFAu16.overflowing_add(offset as u16).0).to_full_address();
                let (previous, current) = common::run_instruction(
                    &instruction_data,
                    |registers: &mut Registers, _: &BusPeripheral| {
                        registers
                            .set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
                        registers.set_at_index(src_register_index, 0xCAFE);
                    },
                    0xFACE_0000,
                );
                let expected_registers =
                    get_expected_registers(&previous.registers, |registers: &mut Registers| {
                        registers
                            .set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
                        registers.set_at_index(src_register_index, 0xCAFE);
                    });
                assert_eq!(
                    expected_registers, current.registers,
                    "Not equal:\nleft: {:X?}\nright:{:X?}\n",
                    expected_registers, current.registers
                );
                let segment_relative_address =
                // Multiply by two to get from CPU word addressing to the byte addressing of the memory dump
                    (calculated_address.to_segmented_address().1) as usize * 2;
                let stored_word =
                    &current.memory_dump[segment_relative_address..=segment_relative_address + 1];
                assert_eq!([0xCA, 0xFE], *stored_word);
            }
        }
    }
}

#[allow(clippy::cast_sign_loss)]
#[test]
fn test_store_indirect_register() {
    for src_address_register_index in get_address_register_index_range() {
        for src_register_index in get_non_address_register_index_range() {
            for offset_register_index in get_non_address_register_index_range() {
                if
                // TODO: Handle PC writes/reads etc.
                // It _should_ be valid to load a memory address into the PC and jump, but it breaks the test
                src_address_register_index
                    == AddressRegisterName::ProgramCounter.to_register_index()
                    || src_register_index == RegisterName::Pl.to_register_index()
                    || src_register_index == RegisterName::Ph.to_register_index()
                    || offset_register_index == RegisterName::Pl.to_register_index()
                    || offset_register_index == RegisterName::Ph.to_register_index()
                    || src_register_index == offset_register_index
                {
                    continue;
                }

                for offset in [
                    0i16, -32i16, -64i16, -0x7FFFi16, -32768i16, 32i16, 64i16, 0x7FFFi16,
                ] {
                    let instruction_data = InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::StoreRegisterToIndirectRegister,
                        r1: 0x0,
                        r2: src_register_index,
                        r3: offset_register_index,
                        condition_flag: ConditionFlags::Always,
                        additional_flags: src_address_register_index,
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                    });
                    let calculated_address =
                        (0xFAFAu16, 0xFAFAu16.overflowing_add(offset as u16).0).to_full_address();
                    let (previous, current) = common::run_instruction(
                        &instruction_data,
                        |registers: &mut Registers, _: &BusPeripheral| {
                            registers.set_address_register_at_index(
                                src_address_register_index,
                                0xFAFA_FAFA,
                            );
                            registers.set_at_index(offset_register_index, offset as u16);
                            registers.set_at_index(src_register_index, 0xCAFE);
                        },
                        0xFACE_0000,
                    );
                    let expected_registers =
                        get_expected_registers(&previous.registers, |registers: &mut Registers| {
                            registers.set_address_register_at_index(
                                src_address_register_index,
                                0xFAFA_FAFA,
                            );
                            registers.set_at_index(offset_register_index, offset as u16);
                            registers.set_at_index(src_register_index, 0xCAFE);
                        });
                    assert_eq!(
                        expected_registers, current.registers,
                        "Not equal:\nleft: {:X?}\nright:{:X?}\n",
                        expected_registers, current.registers
                    );
                    let segment_relative_address =
                        // Multiply by two to get from CPU word addressing to the byte addressing of the memory dump
                            (calculated_address.to_segmented_address().1) as usize * 2;
                    let stored_word = &current.memory_dump
                        [segment_relative_address..=segment_relative_address + 1];
                    assert_eq!([0xCA, 0xFE], *stored_word);
                }
            }
        }
    }
}

#[allow(clippy::cast_sign_loss)]
#[test]
fn test_store_indirect_register_pre_decrement() {
    for src_address_register_index in get_address_register_index_range() {
        for src_register_index in get_non_address_register_index_range() {
            for offset_register_index in get_non_address_register_index_range() {
                if
                // TODO: Handle PC writes/reads etc.
                // It _should_ be valid to load a memory address into the PC and jump, but it breaks the test
                src_address_register_index
                    == AddressRegisterName::ProgramCounter.to_register_index()
                    || src_register_index == RegisterName::Pl.to_register_index()
                    || src_register_index == RegisterName::Ph.to_register_index()
                    || offset_register_index == RegisterName::Pl.to_register_index()
                    || offset_register_index == RegisterName::Ph.to_register_index()
                    || src_register_index == offset_register_index
                {
                    continue;
                }

                for offset in [
                    0i16, -32i16, -64i16, -0x7FFFi16, -32768i16, 32i16, 64i16, 0x7FFFi16,
                ] {
                    let instruction_data = InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::StoreRegisterToIndirectRegisterPreDecrement,
                        r1: 0x0,
                        r2: src_register_index,
                        r3: offset_register_index,
                        condition_flag: ConditionFlags::Always,
                        additional_flags: src_address_register_index,
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                    });
                    let calculated_address =
                        (0xFAFAu16, 0xFAFAu16.overflowing_add(offset as u16).0).to_full_address()
                            - 1;
                    let (previous, current) = common::run_instruction(
                        &instruction_data,
                        |registers: &mut Registers, _: &BusPeripheral| {
                            registers.set_address_register_at_index(
                                src_address_register_index,
                                0xFAFA_FAFA,
                            );
                            registers.set_at_index(offset_register_index, offset as u16);
                            registers.set_at_index(src_register_index, 0xCAFE);
                        },
                        0xFACE_0000,
                    );
                    let expected_registers =
                        get_expected_registers(&previous.registers, |registers: &mut Registers| {
                            registers.set_address_register_at_index(
                                src_address_register_index,
                                0xFAFA_FAF9,
                            );
                            registers.set_at_index(offset_register_index, offset as u16);
                            registers.set_at_index(src_register_index, 0xCAFE);
                        });
                    assert_eq!(
                        expected_registers, current.registers,
                        "Not equal:\nleft: {:X?}\nright:{:X?}\n",
                        expected_registers, current.registers
                    );
                    let segment_relative_address =
                        // Multiply by two to get from CPU word addressing to the byte addressing of the memory dump
                            (calculated_address.to_segmented_address().1) as usize * 2;
                    let stored_word = &current.memory_dump
                        [segment_relative_address..=segment_relative_address + 1];
                    assert_eq!([0xCA, 0xFE], *stored_word);
                }
            }
        }
    }
}

#[allow(clippy::cast_sign_loss)]
#[test]
fn test_store_indirect_immediate_pre_decrement() {
    for src_address_register_index in get_address_register_index_range() {
        for src_register_index in get_non_address_register_index_range() {
            if
            // TODO: Handle PC writes/reads etc.
            // It _should_ be valid to load a memory address into the PC and jump, but it breaks the test
            src_address_register_index == AddressRegisterName::ProgramCounter.to_register_index()
                || src_register_index == RegisterName::Pl.to_register_index()
                || src_register_index == RegisterName::Ph.to_register_index()
            {
                continue;
            }

            for offset in [
                0i16, -32i16, -64i16, -0x7FFFi16, -32768i16, 32i16, 64i16, 0x7FFFi16,
            ] {
                let instruction_data = InstructionData::Immediate(ImmediateInstructionData {
                    op_code: Instruction::StoreRegisterToIndirectImmediatePreDecrement,
                    // TODO: Why doesn't this work??
                    // probs the register pre decrement doesnt actually work
                    // // it sets dest register to 0x0 which is SR and probs ignored
                    // // so the test is broken!
                    register: src_register_index,
                    value: offset as u16,
                    condition_flag: ConditionFlags::Always,
                    additional_flags: src_address_register_index,
                });
                let calculated_address =
                    (0xFAFAu16, 0xFAFAu16.overflowing_add(offset as u16).0).to_full_address() - 1;
                let (previous, current) = common::run_instruction(
                    &instruction_data,
                    |registers: &mut Registers, _: &BusPeripheral| {
                        registers
                            .set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
                        registers.set_at_index(src_register_index, 0xCAFE);
                    },
                    0xFACE_0000,
                );
                let expected_registers =
                    get_expected_registers(&previous.registers, |registers: &mut Registers| {
                        registers
                            .set_address_register_at_index(src_address_register_index, 0xFAFA_FAF9);
                        registers.set_at_index(src_register_index, 0xCAFE);
                    });
                assert_eq!(
                    expected_registers, current.registers,
                    "Not equal:\nleft: {:X?}\nright:{:X?}\n",
                    expected_registers, current.registers
                );
                let segment_relative_address =
                        // Multiply by two to get from CPU word addressing to the byte addressing of the memory dump
                            (calculated_address.to_segmented_address().1) as usize * 2;
                let stored_word =
                    &current.memory_dump[segment_relative_address..=segment_relative_address + 1];
                assert_eq!([0xCA, 0xFE], *stored_word);
            }
        }
    }
}
