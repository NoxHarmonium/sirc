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
        AddressRegisterIndexing, AddressRegisterName, FullAddress, RegisterIndexing, RegisterName,
        Registers, SegmentedAddress,
    },
};
use quickcheck::{Arbitrary, Gen, TestResult};

use crate::instructions::common;

use super::common::{
    get_address_register_index_range, get_expected_registers, get_non_address_register_index_range,
};

#[derive(Debug, PartialEq, Eq, Clone)]
struct StoreTestData {
    src_address_register_index: u8,
    src_register_index: u8,
    offset_register_index: u8, // Only used for register addressing
    offset: i16,
}

impl Arbitrary for StoreTestData {
    fn arbitrary(g: &mut Gen) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        Self {
            src_address_register_index: u8::arbitrary(g)
                % get_address_register_index_range().len() as u8,
            src_register_index: (u8::arbitrary(g)
                % get_non_address_register_index_range().len() as u8)
                + 1,
            offset_register_index: (u8::arbitrary(g)
                % get_non_address_register_index_range().len() as u8)
                + 1,
            offset: i16::arbitrary(g),
        }
    }
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StoreTestShiftParameters {
    pub shift_count: u8,
    pub shift_operand: ShiftOperand,
    pub shift_type: ShiftType,
}

impl Arbitrary for StoreTestShiftParameters {
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
fn test_store_indirect_immediate(test_data: StoreTestData) -> TestResult {
    let StoreTestData {
        src_address_register_index,
        src_register_index,
        offset,
        ..
    } = test_data;

    // Why is this check here? What happens if you try to store from the PC?
    if src_address_register_index == AddressRegisterName::ProgramCounter.to_register_index() {
        return TestResult::discard();
    }

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
        |registers: &mut Registers, _: &mut BusPeripheral| {
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
            registers.set_at_index(src_register_index, 0xCAFE);
        },
        0xFACE_0000,
    );
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
            registers.set_at_index(src_register_index, 0xCAFE);
        });
    let segment_relative_address =
                // Multiply by two to get from CPU word addressing to the byte addressing of the memory dump
                    (calculated_address.to_segmented_address().1) as usize * 2;
    let stored_word = &current.memory_dump[segment_relative_address..=segment_relative_address + 1];

    let test_successful = expected_registers == current.registers && [0xCA, 0xFE] == *stored_word;
    TestResult::from_bool(test_successful)
}

#[allow(clippy::cast_sign_loss, clippy::needless_pass_by_value)]
#[quickcheck()]
fn test_store_indirect_register(
    test_data: StoreTestData,
    shift_params: StoreTestShiftParameters,
) -> TestResult {
    let StoreTestData {
        src_address_register_index,
        src_register_index,
        offset,
        offset_register_index,
    } = test_data;
    let StoreTestShiftParameters {
        shift_count,
        shift_operand,
        shift_type,
    } = shift_params;

    if src_address_register_index == AddressRegisterName::ProgramCounter.to_register_index()
        || src_register_index == RegisterName::Pl.to_register_index()
        || src_register_index == RegisterName::Ph.to_register_index()
        || offset_register_index == RegisterName::Pl.to_register_index()
        || offset_register_index == RegisterName::Ph.to_register_index()
        || src_register_index == offset_register_index
    {
        return TestResult::discard();
    }

    let instruction_data = InstructionData::Register(RegisterInstructionData {
        op_code: Instruction::StoreRegisterToIndirectRegister,
        r1: 0x0,
        r2: src_register_index,
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
        |registers: &mut Registers, _: &mut BusPeripheral| {
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
            registers.set_at_index(offset_register_index, offset as u16);
            registers.set_at_index(src_register_index, 0xCAFE);
        },
        0xFACE_0000,
    );
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
            registers.set_at_index(offset_register_index, offset as u16);
            registers.set_at_index(src_register_index, 0xCAFE);
        });

    let segment_relative_address =
                        // Multiply by two to get from CPU word addressing to the byte addressing of the memory dump
                            (calculated_address.to_segmented_address().1) as usize * 2;
    let stored_word = &current.memory_dump[segment_relative_address..=segment_relative_address + 1];
    let (shifted, _) = perform_shift(0xCAFE, shift_type, shift_count.into());
    let expected_word = u16::to_be_bytes(shifted);

    let test_successful = expected_registers == current.registers && expected_word == *stored_word;
    TestResult::from_bool(test_successful)
}

#[allow(clippy::cast_sign_loss, clippy::needless_pass_by_value)]
#[quickcheck()]
fn test_store_indirect_register_pre_decrement(
    test_data: StoreTestData,
    shift_params: StoreTestShiftParameters,
) -> TestResult {
    let StoreTestData {
        src_address_register_index,
        src_register_index,
        offset,
        offset_register_index,
    } = test_data;
    let StoreTestShiftParameters {
        shift_count,
        shift_operand,
        shift_type,
    } = shift_params;
    if src_address_register_index == AddressRegisterName::ProgramCounter.to_register_index()
        || src_register_index == RegisterName::Pl.to_register_index()
        || src_register_index == RegisterName::Ph.to_register_index()
        || offset_register_index == RegisterName::Pl.to_register_index()
        || offset_register_index == RegisterName::Ph.to_register_index()
        || src_register_index == offset_register_index
    {
        return TestResult::discard();
    }

    let instruction_data = InstructionData::Register(RegisterInstructionData {
        op_code: Instruction::StoreRegisterToIndirectRegisterPreDecrement,
        r1: 0x0,
        r2: src_register_index,
        r3: offset_register_index,
        condition_flag: ConditionFlags::Always,
        additional_flags: src_address_register_index,
        shift_operand,
        shift_type,
        shift_count,
    });
    let calculated_address =
        (0xFAFAu16, 0xFAFAu16.overflowing_add(offset as u16).0).to_full_address() - 1;
    let (previous, current) = common::run_instruction(
        &instruction_data,
        |registers: &mut Registers, _: &mut BusPeripheral| {
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
            registers.set_at_index(offset_register_index, offset as u16);
            registers.set_at_index(src_register_index, 0xCAFE);
        },
        0xFACE_0000,
    );
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAF9);
            registers.set_at_index(offset_register_index, offset as u16);
            registers.set_at_index(src_register_index, 0xCAFE);
        });
    let segment_relative_address =
                        // Multiply by two to get from CPU word addressing to the byte addressing of the memory dump
                            (calculated_address.to_segmented_address().1) as usize * 2;
    let stored_word = &current.memory_dump[segment_relative_address..=segment_relative_address + 1];
    let (shifted, _) = perform_shift(0xCAFE, shift_type, shift_count.into());
    let expected_word = u16::to_be_bytes(shifted);

    let test_successful = expected_registers == current.registers && expected_word == *stored_word;
    TestResult::from_bool(test_successful)
}

#[allow(clippy::cast_sign_loss, clippy::needless_pass_by_value)]
#[quickcheck()]
fn test_store_indirect_immediate_pre_decrement(test_data: StoreTestData) -> TestResult {
    let StoreTestData {
        src_address_register_index,
        src_register_index,
        offset,
        ..
    } = test_data;
    if src_address_register_index == AddressRegisterName::ProgramCounter.to_register_index()
        || src_register_index == RegisterName::Pl.to_register_index()
        || src_register_index == RegisterName::Ph.to_register_index()
    {
        return TestResult::discard();
    }

    let instruction_data = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::StoreRegisterToIndirectImmediatePreDecrement,
        register: src_register_index,
        value: offset as u16,
        condition_flag: ConditionFlags::Always,
        additional_flags: src_address_register_index,
    });
    let calculated_address =
        (0xFAFAu16, 0xFAFAu16.overflowing_add(offset as u16).0).to_full_address() - 1;
    let (previous, current) = common::run_instruction(
        &instruction_data,
        |registers: &mut Registers, _: &mut BusPeripheral| {
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAFA);
            registers.set_at_index(src_register_index, 0xCAFE);
        },
        0xFACE_0000,
    );
    let expected_registers =
        get_expected_registers(&previous.registers, |registers: &mut Registers| {
            registers.set_address_register_at_index(src_address_register_index, 0xFAFA_FAF9);
            registers.set_at_index(src_register_index, 0xCAFE);
        });
    let segment_relative_address =
                        // Multiply by two to get from CPU word addressing to the byte addressing of the memory dump
                            (calculated_address.to_segmented_address().1) as usize * 2;
    let stored_word = &current.memory_dump[segment_relative_address..=segment_relative_address + 1];

    let test_successful = expected_registers == current.registers && [0xCA, 0xFE] == *stored_word;
    TestResult::from_bool(test_successful)
}
