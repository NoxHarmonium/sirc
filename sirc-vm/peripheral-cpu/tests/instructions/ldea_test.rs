use peripheral_bus::BusPeripheral;
use peripheral_cpu::{
    self,
    coprocessors::processing_unit::definitions::{
        ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
        RegisterInstructionData, ShiftOperand, ShiftType,
    },
    registers::{
        AddressRegisterIndexing, AddressRegisterName, RegisterIndexing, Registers, SegmentedAddress,
    },
};

use crate::instructions::common;

use super::common::{
    get_address_register_index_range, get_expected_registers, get_non_address_register_index_range,
};

#[allow(clippy::cast_sign_loss)]
#[test]
fn test_ldea_indirect_immediate() {
    for src_address_register_index in get_address_register_index_range() {
        for dest_address_register_index in get_address_register_index_range() {
            if src_address_register_index == dest_address_register_index
            // TODO: Handle PC writes/reads etc.
                || src_address_register_index
                    == AddressRegisterName::ProgramCounter.to_register_index()
                || dest_address_register_index
                    == AddressRegisterName::ProgramCounter.to_register_index()
            {
                continue;
            }

            for offset in [
                0i16, -32i16, -64i16, -0x7FFFi16, -32768i16, 32i16, 64i16, 0x7FFFi16,
            ] {
                let instruction_data = InstructionData::Immediate(ImmediateInstructionData {
                    op_code: Instruction::LoadEffectiveAddressFromIndirectImmediate,
                    register: dest_address_register_index,
                    value: offset as u16,
                    condition_flag: ConditionFlags::Always,
                    additional_flags: src_address_register_index,
                });
                let (previous, current) = common::run_instruction(
                    &instruction_data,
                    |registers: &mut Registers, _: &BusPeripheral| {
                        registers
                            .set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
                    },
                    0xFACE,
                );
                let expected_registers =
                    get_expected_registers(&previous.registers, |registers: &mut Registers| {
                        registers
                            .set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
                        registers.set_address_register_at_index(
                            dest_address_register_index,
                            (0xFAFAu16, 0xFAFAu16.overflowing_add(offset as u16).0)
                                .to_full_address(),
                        );
                    });
                assert_eq!(
                    expected_registers, current.registers,
                    "Not equal:\nleft: {:X?}\nright:{:X?}\n",
                    expected_registers, current.registers
                );
            }
        }
    }
}

#[allow(clippy::cast_sign_loss)]
#[test]
fn test_ldea_indirect_register() {
    for src_address_register_index in get_address_register_index_range() {
        for dest_address_register_index in get_address_register_index_range() {
            if src_address_register_index == dest_address_register_index
            // TODO: Handle PC writes/reads etc.
                || src_address_register_index
                    == AddressRegisterName::ProgramCounter.to_register_index()
                || dest_address_register_index
                    == AddressRegisterName::ProgramCounter.to_register_index()
            {
                continue;
            }

            for offset in [
                0i16, -32i16, -64i16, -0x7FFFi16, -32768i16, 32i16, 64i16, 0x7FFFi16,
            ] {
                for offset_register in get_non_address_register_index_range() {
                    let instruction_data = InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadEffectiveAddressFromIndirectRegister,
                        r1: dest_address_register_index,
                        r2: 0x0,
                        r3: offset_register,
                        shift_operand: ShiftOperand::Immediate,
                        shift_type: ShiftType::None,
                        shift_count: 0,
                        condition_flag: ConditionFlags::Always,
                        additional_flags: src_address_register_index,
                    });
                    let (previous, current) = common::run_instruction(
                        &instruction_data,
                        |registers: &mut Registers, _: &BusPeripheral| {
                            registers.set_at_index(offset_register, offset as u16);
                            registers.set_address_register_at_index(
                                src_address_register_index,
                                0xFAFA_FAFA,
                            );
                        },
                        0xFACE,
                    );
                    let expected_registers =
                        get_expected_registers(&previous.registers, |registers: &mut Registers| {
                            registers.set_at_index(offset_register, offset as u16);
                            registers.set_address_register_at_index(
                                src_address_register_index,
                                0xFAFA_FAFA,
                            );
                            registers.set_address_register_at_index(
                                dest_address_register_index,
                                (0xFAFAu16, 0xFAFAu16.overflowing_add(offset as u16).0)
                                    .to_full_address(),
                            );
                        });
                    assert_eq!(
                        expected_registers, current.registers,
                        "Not equal:\nleft: {:X?}\nright:{:X?}\n",
                        expected_registers, current.registers
                    );
                }
            }
        }
    }
}

// TODO: Test PC offsets
// TODO: Can PC be written to with LDEA?
