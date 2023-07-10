use peripheral_cpu::{
    self,
    instructions::definitions::{
        ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
    },
    registers::{AddressRegisterIndexing, AddressRegisterName, Registers, SegmentedAddress},
};

use crate::instructions::common;

use super::common::{get_address_register_index_range, get_expected_registers};

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
                    |registers: &mut Registers| {
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

// TODO: Test PC offsets
// TODO: Can PC be written to with LDEA?
