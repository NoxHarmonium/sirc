use peripheral_bus::BusPeripheral;
use peripheral_cpu::{
    coprocessors::{
        exception_unit::definitions::Faults,
        processing_unit::definitions::{
            ConditionFlags, ImmediateInstructionData, Instruction, InstructionData,
            RegisterInstructionData, ShiftOperand, ShiftType, ShortImmediateInstructionData,
        },
    },
    registers::{
        set_sr_bit, AddressRegisterIndexing, AddressRegisterName, RegisterName, Registers,
        StatusRegisterFields,
    },
    CpuPeripheral, CYCLES_PER_INSTRUCTION,
};

use crate::instructions::common;

struct ProtectedModeResult {
    registers: Registers,
    pending_fault: Option<Faults>,
    pending_coprocessor_command: u16,
}

fn run_instruction<F>(instruction_data: &InstructionData, register_setup: F) -> ProtectedModeResult
where
    F: Fn(&mut Registers, &mut BusPeripheral),
{
    let mut bus = common::set_up_instruction_test(instruction_data, 0x00CC_0000);
    common::setup_test(&mut bus, register_setup, 0x00CC_0000);
    bus.run_full_cycle(CYCLES_PER_INSTRUCTION);

    let cpu: &mut CpuPeripheral = bus
        .bus_master
        .as_any()
        .downcast_mut::<CpuPeripheral>()
        .expect("failed to downcast");

    ProtectedModeResult {
        registers: cpu.registers,
        pending_fault: cpu.eu_registers.pending_fault,
        pending_coprocessor_command: cpu.registers.pending_coprocessor_command,
    }
}

fn enable_protected_mode(registers: &mut Registers) {
    set_sr_bit(StatusRegisterFields::ProtectedMode, registers);
}

fn assert_no_fault(result: &ProtectedModeResult) {
    assert_eq!(None, result.pending_fault);
}

fn assert_privilege_fault(result: &ProtectedModeResult) {
    assert_eq!(Some(Faults::PrivilegeViolation), result.pending_fault);
}

#[test]
fn direct_high_address_register_writes_fault_in_protected_mode() {
    let load_ph = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::LoadRegisterFromImmediate,
        register: RegisterName::Ph.to_register_index(),
        value: 0x1234,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    });
    let result = run_instruction(&load_ph, |registers, _| {
        enable_protected_mode(registers);
        registers.ph = 0x00CC;
    });
    assert_privilege_fault(&result);
    assert_eq!(0x00CC, result.registers.ph);

    let register_load_ph = InstructionData::Register(RegisterInstructionData {
        op_code: Instruction::LoadRegisterFromRegister,
        r1: RegisterName::Ph.to_register_index(),
        r2: RegisterName::R1.to_register_index(),
        r3: RegisterName::R1.to_register_index(),
        shift_operand: ShiftOperand::Immediate,
        shift_type: ShiftType::None,
        shift_count: 0,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    });
    let result = run_instruction(&register_load_ph, |registers, _| {
        enable_protected_mode(registers);
        registers.ph = 0x00CC;
        registers.r1 = 0x1234;
    });
    assert_privilege_fault(&result);
    assert_eq!(0x00CC, result.registers.ph);
}

#[test]
fn conditional_false_privileged_writes_do_not_fault() {
    let load_ph = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::LoadRegisterFromImmediate,
        register: RegisterName::Ph.to_register_index(),
        value: 0x1234,
        condition_flag: ConditionFlags::CarryClear,
        additional_flags: 0x0,
    });
    let result = run_instruction(&load_ph, |registers, _| {
        enable_protected_mode(registers);
        set_sr_bit(StatusRegisterFields::Carry, registers);
        registers.ph = 0x00CC;
    });
    assert_no_fault(&result);
    assert_eq!(0x00CC, result.registers.ph);
}

#[test]
fn ldea_masks_destination_high_word_in_protected_mode() {
    let ldea = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::LoadEffectiveAddressFromIndirectImmediate,
        register: AddressRegisterName::Address.to_register_index(),
        value: 0x0004,
        condition_flag: ConditionFlags::Always,
        additional_flags: AddressRegisterName::StackPointer.to_register_index(),
    });
    let result = run_instruction(&ldea, |registers, _| {
        enable_protected_mode(registers);
        registers.set_address_register_at_index(
            AddressRegisterName::Address.to_register_index(),
            0x1111_2222,
        );
        registers.set_address_register_at_index(
            AddressRegisterName::StackPointer.to_register_index(),
            0xAAAA_1000,
        );
    });
    assert_no_fault(&result);
    assert_eq!(0x0011, result.registers.ah);
    assert_eq!(0x1004, result.registers.al);
}

#[test]
fn ldea_writes_destination_high_word_in_supervisor_mode() {
    let ldea = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::LoadEffectiveAddressFromIndirectImmediate,
        register: AddressRegisterName::Address.to_register_index(),
        value: 0x0004,
        condition_flag: ConditionFlags::Always,
        additional_flags: AddressRegisterName::StackPointer.to_register_index(),
    });
    let result = run_instruction(&ldea, |registers, _| {
        registers.set_address_register_at_index(
            AddressRegisterName::Address.to_register_index(),
            0x1111_2222,
        );
        registers.set_address_register_at_index(
            AddressRegisterName::StackPointer.to_register_index(),
            0xAAAA_1000,
        );
    });
    assert_no_fault(&result);
    assert_eq!(0x00AA, result.registers.ah);
    assert_eq!(0x1004, result.registers.al);
}

#[test]
fn long_jump_masks_program_counter_high_word_in_protected_mode() {
    let ljmp = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::LoadEffectiveAddressFromIndirectImmediate,
        register: AddressRegisterName::ProgramCounter.to_register_index(),
        value: 0x0004,
        condition_flag: ConditionFlags::Always,
        additional_flags: AddressRegisterName::Address.to_register_index(),
    });
    let result = run_instruction(&ljmp, |registers, _| {
        enable_protected_mode(registers);
        registers.ph = 0x00CC;
        registers.set_address_register_at_index(
            AddressRegisterName::Address.to_register_index(),
            0xAAAA_1000,
        );
    });
    assert_no_fault(&result);
    assert_eq!(0x00CC, result.registers.ph);
    assert_eq!(0x1004, result.registers.pl);
}

#[test]
fn branch_masks_program_counter_high_word_in_protected_mode() {
    let branch = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::BranchWithImmediateDisplacement,
        register: AddressRegisterName::ProgramCounter.to_register_index(),
        value: 0x0004,
        condition_flag: ConditionFlags::Always,
        additional_flags: AddressRegisterName::Address.to_register_index(),
    });
    let result = run_instruction(&branch, |registers, _| {
        enable_protected_mode(registers);
        registers.ph = 0x00CC;
        registers.set_address_register_at_index(
            AddressRegisterName::Address.to_register_index(),
            0xAAAA_1000,
        );
    });
    assert_no_fault(&result);
    assert_eq!(0x00CC, result.registers.ph);
    assert_eq!(0x1004, result.registers.pl);
}

#[test]
fn subroutine_calls_mask_link_and_program_counter_high_words_in_protected_mode() {
    let branch_subroutine = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::BranchToSubroutineWithImmediateDisplacement,
        register: AddressRegisterName::ProgramCounter.to_register_index(),
        value: 0x0004,
        condition_flag: ConditionFlags::Always,
        additional_flags: AddressRegisterName::Address.to_register_index(),
    });
    let result = run_instruction(&branch_subroutine, |registers, _| {
        enable_protected_mode(registers);
        registers.ph = 0x00CC;
        registers.lh = 0x7777;
        registers.ll = 0x8888;
        registers.set_address_register_at_index(
            AddressRegisterName::Address.to_register_index(),
            0xAAAA_1000,
        );
    });
    assert_no_fault(&result);
    assert_eq!(0x00CC, result.registers.ph);
    assert_eq!(0x1004, result.registers.pl);
    assert_eq!(0x7777, result.registers.lh);
    assert_eq!(0x0002, result.registers.ll);

    let long_subroutine = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::LongJumpToSubroutineWithImmediateDisplacement,
        register: AddressRegisterName::ProgramCounter.to_register_index(),
        value: 0x0006,
        condition_flag: ConditionFlags::Always,
        additional_flags: AddressRegisterName::Address.to_register_index(),
    });
    let result = run_instruction(&long_subroutine, |registers, _| {
        enable_protected_mode(registers);
        registers.ph = 0x00CC;
        registers.lh = 0x7777;
        registers.ll = 0x8888;
        registers.set_address_register_at_index(
            AddressRegisterName::Address.to_register_index(),
            0xBBBB_4000,
        );
    });
    assert_no_fault(&result);
    assert_eq!(0x00CC, result.registers.ph);
    assert_eq!(0x4006, result.registers.pl);
    assert_eq!(0x7777, result.registers.lh);
    assert_eq!(0x0002, result.registers.ll);
}

#[test]
fn subroutine_calls_write_high_words_in_supervisor_mode() {
    let branch_subroutine = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::BranchToSubroutineWithImmediateDisplacement,
        register: AddressRegisterName::ProgramCounter.to_register_index(),
        value: 0x0004,
        condition_flag: ConditionFlags::Always,
        additional_flags: AddressRegisterName::Address.to_register_index(),
    });
    let result = run_instruction(&branch_subroutine, |registers, _| {
        registers.ph = 0x00CC;
        registers.lh = 0x7777;
        registers.ll = 0x8888;
        registers.set_address_register_at_index(
            AddressRegisterName::Address.to_register_index(),
            0xAAAA_1000,
        );
    });
    assert_no_fault(&result);
    assert_eq!(0x00AA, result.registers.ph);
    assert_eq!(0x1004, result.registers.pl);
    assert_eq!(0x00CC, result.registers.lh);
    assert_eq!(0x0002, result.registers.ll);
}

#[test]
fn rets_restores_only_program_counter_low_word_in_protected_mode() {
    let rets = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::LoadEffectiveAddressFromIndirectImmediate,
        register: AddressRegisterName::ProgramCounter.to_register_index(),
        value: 0x0000,
        condition_flag: ConditionFlags::Always,
        additional_flags: AddressRegisterName::LinkRegister.to_register_index(),
    });
    let result = run_instruction(&rets, |registers, _| {
        enable_protected_mode(registers);
        registers.ph = 0x00CC;
        registers.lh = 0xAAAA;
        registers.ll = 0x4320;
    });
    assert_no_fault(&result);
    assert_eq!(0x00CC, result.registers.ph);
    assert_eq!(0x4320, result.registers.pl);
}

#[test]
fn memory_auto_update_preserves_address_high_word_in_protected_mode() {
    let load_post_increment = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::LoadRegisterFromIndirectImmediatePostIncrement,
        register: RegisterName::R1.to_register_index(),
        value: 0x0000,
        condition_flag: ConditionFlags::Always,
        additional_flags: AddressRegisterName::Address.to_register_index(),
    });
    let result = run_instruction(&load_post_increment, |registers, bus| {
        enable_protected_mode(registers);
        registers.set_address_register_at_index(
            AddressRegisterName::Address.to_register_index(),
            common::SCRATCH_SEGMENT_BEGIN,
        );
        bus.write_address(common::SCRATCH_SEGMENT_BEGIN, 0xCAFE);
    });
    assert_no_fault(&result);
    assert_eq!(0x00FA, result.registers.ah);
    assert_eq!(0x0001, result.registers.al);
    assert_eq!(0xCAFE, result.registers.r1);

    let store_pre_decrement = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::StoreRegisterToIndirectImmediatePreDecrement,
        register: RegisterName::R1.to_register_index(),
        value: 0x0000,
        condition_flag: ConditionFlags::Always,
        additional_flags: AddressRegisterName::Address.to_register_index(),
    });
    let result = run_instruction(&store_pre_decrement, |registers, _| {
        enable_protected_mode(registers);
        registers.set_address_register_at_index(
            AddressRegisterName::Address.to_register_index(),
            common::SCRATCH_SEGMENT_BEGIN + 1,
        );
        registers.r1 = 0xCAFE;
    });
    assert_no_fault(&result);
    assert_eq!(0x00FA, result.registers.ah);
    assert_eq!(0x0000, result.registers.al);
}

#[test]
#[allow(clippy::similar_names)]
fn privileged_coprocessor_calls_fault_only_when_executed() {
    let privileged_copi = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::CoprocessorCallImmediate,
        register: RegisterName::R1.to_register_index(),
        value: 0x1900,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    });
    let result = run_instruction(&privileged_copi, |registers, _| {
        enable_protected_mode(registers);
    });
    assert_privilege_fault(&result);

    let skipped_privileged_copi = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::CoprocessorCallImmediate,
        register: RegisterName::R1.to_register_index(),
        value: 0x1900,
        condition_flag: ConditionFlags::CarryClear,
        additional_flags: 0x0,
    });
    let result = run_instruction(&skipped_privileged_copi, |registers, _| {
        enable_protected_mode(registers);
        set_sr_bit(StatusRegisterFields::Carry, registers);
    });
    assert_no_fault(&result);
    assert_eq!(0x0000, result.pending_coprocessor_command);

    let privileged_copr = InstructionData::Register(RegisterInstructionData {
        op_code: Instruction::CoprocessorCallRegister,
        r1: RegisterName::R1.to_register_index(),
        r2: RegisterName::R1.to_register_index(),
        r3: RegisterName::R2.to_register_index(),
        shift_operand: ShiftOperand::Immediate,
        shift_type: ShiftType::None,
        shift_count: 0,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    });
    let result = run_instruction(&privileged_copr, |registers, _| {
        enable_protected_mode(registers);
        registers.r2 = 0x1900;
    });
    assert_privilege_fault(&result);
}

#[test]
fn short_copi_cannot_encode_privileged_coprocessor_command() {
    let short_copi = InstructionData::ShortImmediate(ShortImmediateInstructionData {
        op_code: Instruction::CoprocessorCallShortImmediate,
        register: RegisterName::R1.to_register_index(),
        value: 0x19,
        shift_operand: ShiftOperand::Immediate,
        shift_type: ShiftType::LogicalLeftShift,
        shift_count: 8,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    });
    let result = run_instruction(&short_copi, |registers, _| {
        enable_protected_mode(registers);
        registers.r1 = 0x1900;
    });
    assert_no_fault(&result);
    assert_eq!(0x0019, result.pending_coprocessor_command);
}
