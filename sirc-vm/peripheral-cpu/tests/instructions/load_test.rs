use peripheral_cpu::{
    self,
    instructions::definitions::{
        ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
        RegisterInstructionData, ShiftOperand, ShiftType,
    },
    registers::{
        AddressRegisterIndexing, AddressRegisterName, RegisterIndexing, RegisterName, Registers,
        SegmentedAddress,
    },
};
use peripheral_mem::MemoryPeripheral;

use crate::instructions::common;

use super::common::{
    get_address_register_index_range, get_expected_registers, get_non_address_register_index_range,
    get_register_index_range,
};

#[allow(clippy::cast_sign_loss)]
#[test]
fn test_load_indirect_immediate() {
    for src_address_register_index in get_address_register_index_range() {
        for dest_register_index in get_register_index_range() {
            if
            // TODO: Handle PC writes/reads etc.
            // It _should_ be valid to load a memory address into the PC and jump, but it breaks the test
            src_address_register_index == AddressRegisterName::ProgramCounter.to_register_index()
                || dest_register_index == RegisterName::Pl.to_register_index()
                || dest_register_index == RegisterName::Ph.to_register_index()
            {
                continue;
            }

            for offset in [
                0i16, -32i16, -64i16, -0x7FFFi16, -32768i16, 32i16, 64i16, 0x7FFFi16,
            ] {
                let instruction_data = InstructionData::Immediate(ImmediateInstructionData {
                    op_code: Instruction::LoadRegisterFromIndirectImmediate,
                    register: dest_register_index,
                    value: offset as u16,
                    condition_flag: ConditionFlags::Always,
                    additional_flags: src_address_register_index,
                });
                let calculated_address =
                    (0xFAFAu16, 0xFAFAu16.overflowing_add(offset as u16).0).to_full_address();
                let (previous, current) = common::run_instruction(
                    &instruction_data,
                    |registers: &mut Registers, memory: &MemoryPeripheral| {
                        memory.write_address(calculated_address, 0xCAFE);
                        registers
                            .set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
                    },
                    0xFACE,
                );
                let expected_registers =
                    get_expected_registers(&previous.registers, |registers: &mut Registers| {
                        registers
                            .set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
                        registers.set_at_index(dest_register_index, 0xCAFE);
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
fn test_load_indirect_register() {
    for src_address_register_index in get_address_register_index_range() {
        for dest_register_index in get_register_index_range() {
            for offset_register_index in get_non_address_register_index_range() {
                if
                // TODO: Handle PC writes/reads etc.
                // It _should_ be valid to load a memory address into the PC and jump, but it breaks the test
                src_address_register_index
                    == AddressRegisterName::ProgramCounter.to_register_index()
                    || dest_register_index == RegisterName::Pl.to_register_index()
                    || dest_register_index == RegisterName::Ph.to_register_index()
                    || offset_register_index == RegisterName::Pl.to_register_index()
                    || offset_register_index == RegisterName::Ph.to_register_index()
                    || dest_register_index == offset_register_index
                {
                    continue;
                }

                for offset in [
                    0i16, -32i16, -64i16, -0x7FFFi16, -32768i16, 32i16, 64i16, 0x7FFFi16,
                ] {
                    let instruction_data = InstructionData::Register(RegisterInstructionData {
                        op_code: Instruction::LoadRegisterFromIndirectRegister,
                        r1: dest_register_index,
                        r2: 0x0,
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
                        |registers: &mut Registers, memory: &MemoryPeripheral| {
                            memory.write_address(calculated_address, 0xCAFE);
                            registers.set_address_register_at_index(
                                src_address_register_index,
                                0xFAFA_FAFA,
                            );
                            registers.set_at_index(offset_register_index, offset as u16);
                        },
                        0xFACE,
                    );
                    let expected_registers =
                        get_expected_registers(&previous.registers, |registers: &mut Registers| {
                            registers.set_address_register_at_index(
                                src_address_register_index,
                                0xFAFA_FAFA,
                            );
                            registers.set_at_index(offset_register_index, offset as u16);
                            registers.set_at_index(dest_register_index, 0xCAFE);
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
