#[cfg(test)]
use num::FromPrimitive;
use peripheral_bus::BusPeripheral;
use peripheral_cpu::{
    coprocessors::processing_unit::{
        definitions::{
            ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
            RegisterInstructionData, ShiftOperand, ShiftType,
        },
        stages::alu::perform_shift,
    },
    registers::{
        AddressRegisterIndexing, AddressRegisterName, RegisterIndexing, RegisterName, Registers,
        SegmentedAddress,
    },
};
use quickcheck::{Arbitrary, Gen, TestResult};

use crate::instructions::common;

use super::common::get_expected_registers;

#[derive(Debug, PartialEq, Eq, Clone)]
struct LoadTestData {
    src_address_register_index: u8,
    dest_register_index: u8,
    offset_register_index: u8, // Only used for register addressing
    offset: i16,
}

impl Arbitrary for LoadTestData {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            src_address_register_index: u8::arbitrary(g) % 4,
            dest_register_index: (u8::arbitrary(g) % 13) + 1,
            offset_register_index: (u8::arbitrary(g) % 7) + 1,
            offset: i16::arbitrary(g),
        }
    }
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LoadTestShiftParameters {
    pub shift_count: u8,
    pub shift_operand: ShiftOperand,
    pub shift_type: ShiftType,
}

impl Arbitrary for LoadTestShiftParameters {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            shift_count: u8::arbitrary(g) % 15,
            // TODO: Test the register shift operand (too complicated for now)
            shift_operand: ShiftOperand::Immediate,
            shift_type: ShiftType::from_u8(u8::arbitrary(g) % 7).unwrap(),
        }
    }
}

#[allow(clippy::cast_sign_loss, clippy::needless_pass_by_value)]
#[quickcheck()]
fn test_load_indirect_immediate(test_data: LoadTestData) -> TestResult {
    let LoadTestData {
        src_address_register_index,
        dest_register_index,
        offset,
        ..
    } = test_data;
    if src_address_register_index == AddressRegisterName::ProgramCounter.to_register_index()
        || dest_register_index == RegisterName::Pl.to_register_index()
        || dest_register_index == RegisterName::Ph.to_register_index()
    {
        return TestResult::discard();
    }

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
        |registers: &mut Registers, bus: &mut BusPeripheral| {
            bus.write_address(calculated_address, 0xCAFE);
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
        },
        0xFACE_0000,
    );
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
            registers.set_at_index(dest_register_index, 0xCAFE);
        });

    let test_successful = expected_registers == current.registers;
    if !test_successful {
        println!(
            "test_load_indirect_immediate: Final register state does not match expected:\nexpected: {:X?}\nactual:{:X?}\n",
            expected_registers, current.registers
        );
    }
    TestResult::from_bool(test_successful)
}

#[allow(clippy::cast_sign_loss, clippy::needless_pass_by_value)]
#[quickcheck()]
fn test_load_indirect_register(
    test_data: LoadTestData,
    shift_params: LoadTestShiftParameters,
) -> TestResult {
    let LoadTestData {
        src_address_register_index,
        dest_register_index,
        offset_register_index,
        offset,
    } = test_data;
    let LoadTestShiftParameters {
        shift_count,
        shift_operand,
        shift_type,
    } = shift_params;

    if src_address_register_index == AddressRegisterName::ProgramCounter.to_register_index()
        || dest_register_index == RegisterName::Pl.to_register_index()
        || dest_register_index == RegisterName::Ph.to_register_index()
        || offset_register_index == RegisterName::Pl.to_register_index()
        || offset_register_index == RegisterName::Ph.to_register_index()
        || dest_register_index == offset_register_index
    {
        return TestResult::discard();
    }

    let instruction_data = InstructionData::Register(RegisterInstructionData {
        op_code: Instruction::LoadRegisterFromIndirectRegister,
        r1: dest_register_index,
        r2: 0x0,
        r3: offset_register_index,
        condition_flag: ConditionFlags::Always,
        additional_flags: src_address_register_index,
        shift_operand,
        shift_type,
        shift_count,
    });
    let calculated_address =
        (0xFAFAu16, 0xFAFAu16.overflowing_add(offset as u16).0).to_full_address();
    let (previous, current) = common::run_instruction(
        &instruction_data,
        |registers: &mut Registers, memory: &mut BusPeripheral| {
            memory.write_address(calculated_address, 0xCAFE);
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
            registers.set_at_index(offset_register_index, offset as u16);
        },
        0xFACE_0000,
    );
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
            registers.set_at_index(offset_register_index, offset as u16);
            let (shifted, _) = perform_shift(0xCAFE, shift_type, shift_count.into());
            registers.set_at_index(dest_register_index, shifted);
        });
    let test_successful = expected_registers == current.registers;
    if !test_successful {
        println!(
                    "test_load_indirect_register: Final register state does not match expected:\nexpected: {:X?}\nactual:{:X?}\n",
                    expected_registers, current.registers
                );
    }
    TestResult::from_bool(test_successful)
}

#[allow(clippy::cast_sign_loss, clippy::needless_pass_by_value)]
#[quickcheck()]
fn test_load_indirect_register_post_increment(
    test_data: LoadTestData,
    shift_params: LoadTestShiftParameters,
) -> TestResult {
    let LoadTestData {
        src_address_register_index,
        dest_register_index,
        offset_register_index,
        offset,
    } = test_data;
    let LoadTestShiftParameters {
        shift_count,
        shift_operand,
        shift_type,
    } = shift_params;

    if src_address_register_index == AddressRegisterName::ProgramCounter.to_register_index()
        || dest_register_index == RegisterName::Pl.to_register_index()
        || dest_register_index == RegisterName::Ph.to_register_index()
        || offset_register_index == RegisterName::Pl.to_register_index()
        || offset_register_index == RegisterName::Ph.to_register_index()
        || dest_register_index == offset_register_index
    {
        return TestResult::discard();
    }

    // TODO: Try to deduplicate common code in `load_test.rs`
    // category=Testing
    // - Deduplicate this with the non post-increment version
    // - See also test_load_indirect_immediate_post_increment
    let instruction_data = InstructionData::Register(RegisterInstructionData {
        op_code: Instruction::LoadRegisterFromIndirectRegisterPostIncrement,
        r1: dest_register_index,
        r2: 0x0,
        r3: offset_register_index,
        condition_flag: ConditionFlags::Always,
        additional_flags: src_address_register_index,
        shift_operand,
        shift_type,
        shift_count,
    });
    let calculated_address =
        (0xFAFAu16, 0xFAFAu16.overflowing_add(offset as u16).0).to_full_address();
    let (previous, current) = common::run_instruction(
        &instruction_data,
        |registers: &mut Registers, memory: &mut BusPeripheral| {
            memory.write_address(calculated_address, 0xCAFE);
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
            registers.set_at_index(offset_register_index, offset as u16);
        },
        0xFACE_0000,
    );
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAFB);
            registers.set_at_index(offset_register_index, offset as u16);
            let (shifted, _) = perform_shift(0xCAFE, shift_type, shift_count.into());
            registers.set_at_index(dest_register_index, shifted);
        });
    let test_successful = expected_registers == current.registers;
    if !test_successful {
        println!(
                        "test_load_indirect_register_post_increment: Final register state does not match expected:\nexpected: {:X?}\nactual:{:X?}\n",
                        expected_registers, current.registers
                    );
    }
    TestResult::from_bool(test_successful)
}

#[allow(clippy::cast_sign_loss, clippy::needless_pass_by_value)]
#[quickcheck()]
fn test_load_indirect_immediate_post_increment(test_data: LoadTestData) -> TestResult {
    let LoadTestData {
        src_address_register_index,
        dest_register_index,
        offset,
        ..
    } = test_data;
    if src_address_register_index == AddressRegisterName::ProgramCounter.to_register_index()
        || dest_register_index == RegisterName::Pl.to_register_index()
        || dest_register_index == RegisterName::Ph.to_register_index()
    {
        return TestResult::discard();
    }

    let instruction_data = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::LoadRegisterFromIndirectImmediatePostIncrement,
        register: dest_register_index,
        value: offset as u16,
        condition_flag: ConditionFlags::Always,
        additional_flags: src_address_register_index,
    });
    let calculated_address =
        (0xFAFAu16, 0xFAFAu16.overflowing_add(offset as u16).0).to_full_address();
    let (previous, current) = common::run_instruction(
        &instruction_data,
        |registers: &mut Registers, memory: &mut BusPeripheral| {
            memory.write_address(calculated_address, 0xCAFE);
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
        },
        0xFACE_0000,
    );
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAFB);
            registers.set_at_index(dest_register_index, 0xCAFE);
        });
    let test_successful = expected_registers == current.registers;
    if !test_successful {
        println!(
                        "test_load_indirect_immediate_post_increment: Final register state does not match expected:\nexpected: {:X?}\nactual:{:X?}\n",
                        expected_registers, current.registers
                    );
    }
    TestResult::from_bool(test_successful)
}
