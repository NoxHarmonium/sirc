use peripheral_bus::{
    conversion::bytes_to_words,
    device::{BusAccessType, BusAssertions, Device},
};
use peripheral_cpu::{
    coprocessors::{
        exception_unit::{
            definitions::{
                vectors::{BUS_FAULT, BUS_PROTECTION_FAULT},
                ExceptionUnitOpCodes, Faults,
            },
            execution::construct_exception_unit_command,
        },
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
        set_hardware_interrupt_enable, sr_bit_is_set, FullAddressRegisterAccess, RegisterName,
        StatusRegisterFields,
    },
    CpuPeripheral,
};

fn tick(cpu: &mut CpuPeripheral, data: u16) -> BusAssertions {
    cpu.poll(
        BusAssertions {
            data,
            bus_acknowledge: true,
            ..Default::default()
        },
        true,
    )
}

fn issue(cpu: &mut CpuPeripheral, command: u16, register_call: bool) {
    cpu.registers.r1 = command;
    let instruction = if register_call {
        InstructionData::Register(RegisterInstructionData {
            op_code: Instruction::CoprocessorCallRegister,
            r1: 0,
            r2: 0,
            r3: RegisterName::R1.to_register_index(),
            shift_operand: ShiftOperand::Immediate,
            shift_type: ShiftType::None,
            shift_count: 0,
            condition_flag: ConditionFlags::Always,
            additional_flags: 0,
        })
    } else {
        InstructionData::Immediate(ImmediateInstructionData {
            op_code: Instruction::CoprocessorCallImmediate,
            register: RegisterName::R1.to_register_index(),
            value: command,
            condition_flag: ConditionFlags::Always,
            additional_flags: 0,
        })
    };
    let words = bytes_to_words(&encode_instruction(&instruction));
    for data in [0, words[0], words[1], 0, 0, 0] {
        tick(cpu, data);
    }
    assert_eq!(cpu.phase, 0);
    assert_eq!(cpu.registers.coprocessor_select_register, command);
}

fn complete_entry(cpu: &mut CpuPeripheral, vector: u8, target: u16) {
    let first = tick(cpu, 0);
    assert!(first.bus_access_strobe);
    assert_eq!(first.address, u32::from(vector) * 2);
    assert!(!first.protected_mode_active);
    let second = tick(cpu, 0);
    assert!(second.bus_access_strobe);
    assert_eq!(second.address, first.address + 1);
    assert!(!second.protected_mode_active);
    for data in [target, 0, 0, 0] {
        tick(cpu, data);
    }
    assert_eq!(cpu.registers.pl, target);
}

#[test]
fn failed_irq_and_trap_vectors_retry_through_the_existing_fault_return_path() {
    // None means EXCP; bits 0..4 exercise every IRQ, including edge-only L5.
    for interrupt in [None, Some(1_u8), Some(2), Some(4), Some(8), Some(16)] {
        for register_call in [false, true] {
            if interrupt.is_some() && register_call {
                continue;
            }
            for failed_word in [0, 1] {
                for protection_error in [false, true] {
                    let mut cpu = new_cpu_peripheral();
                    cpu.registers.set_full_pc_address(0x0034_fffe);
                    cpu.registers.sr = 0x0105; // Protected, with preserved arithmetic flags.
                                               // Retrying an interrupt from inside a software handler also
                                               // verifies restoration of its pre-entry exception level.
                    cpu.eu_registers.current_exception_level = u8::from(interrupt.is_some());
                    set_hardware_interrupt_enable(&mut cpu.registers, 0x0f);
                    let issuing_pc = cpu.registers.get_full_pc_address();
                    let vector = if let Some(mask) = interrupt {
                        cpu.raise_hardware_interrupt(mask);
                        0x50 - u8::try_from(mask.trailing_zeros()).unwrap() * 0x10
                    } else {
                        issue(
                            &mut cpu,
                            construct_exception_unit_command(
                                &ExceptionUnitOpCodes::SoftwareException,
                                0x60,
                            ),
                            register_call,
                        );
                        0x60
                    };
                    let continuation_pc = cpu.registers.get_full_pc_address();
                    let original_sr = cpu.registers.sr;
                    let original_level = cpu.eu_registers.current_exception_level;
                    let old_links = cpu.eu_registers.link_registers;
                    let first = tick(&mut cpu, 0);
                    assert_eq!(first.address, u32::from(vector) * 2);
                    let request = if failed_word == 1 {
                        tick(&mut cpu, 0xabcd)
                    } else {
                        first
                    };
                    assert_eq!(request.bus_access_type, BusAccessType::ExceptionVectorFetch);
                    assert!(!request.protected_mode_active);
                    assert!(sr_bit_is_set(
                        StatusRegisterFields::ProtectedMode,
                        &cpu.registers
                    ));
                    for _ in 0..3 {
                        let held = cpu.poll(BusAssertions::default(), true);
                        assert!(held.bus_access_strobe);
                        assert!(!held.protected_mode_active);
                        assert_eq!(held.address, request.address);
                        assert_eq!(cpu.registers.sr, original_sr);
                    }
                    cpu.poll(
                        BusAssertions {
                            bus_error: !protection_error,
                            bus_protection_error: protection_error,
                            ..Default::default()
                        },
                        true,
                    );
                    let fault = cpu.eu_registers.pending_fault.unwrap();
                    assert_eq!(
                        fault.fault,
                        if protection_error {
                            Faults::BusProtection
                        } else {
                            Faults::Bus
                        }
                    );
                    let return_pc = if interrupt.is_some() {
                        continuation_pc
                    } else {
                        issuing_pc
                    };
                    assert_eq!(fault.pc, return_pc);
                    assert_eq!(cpu.registers.get_full_pc_address(), continuation_pc);
                    assert_eq!(cpu.registers.sr, original_sr);
                    assert_eq!(cpu.eu_registers.current_exception_level, original_level);
                    assert_eq!(cpu.eu_registers.link_registers[..7], old_links[..7]);
                    assert_eq!(
                        cpu.eu_registers.pending_hardware_exceptions,
                        interrupt.unwrap_or(0)
                    );
                    let metadata = cpu.eu_registers.link_registers[7];
                    assert_eq!(metadata.return_address, request.address);
                    let decoded = decode_fault_metadata_register(metadata.return_status_register);
                    assert_eq!(decoded.bus_access_type, BusAccessType::ExceptionVectorFetch);
                    assert_eq!(decoded.fault, fault.fault);
                    super::common::finish_aborted_dispatch(&mut cpu);
                    complete_entry(
                        &mut cpu,
                        if protection_error {
                            BUS_PROTECTION_FAULT
                        } else {
                            BUS_FAULT
                        },
                        0x0800,
                    );
                    let saved = cpu.eu_registers.link_registers[6];
                    assert_eq!(saved.return_address, return_pc);
                    assert_eq!(saved.return_status_register, original_sr);
                    assert_eq!(saved.saved_exception_level, original_level);
                    // A restored interrupt stays pending throughout the fault handler.
                    assert_eq!(
                        cpu.eu_registers.pending_hardware_exceptions,
                        interrupt.unwrap_or(0)
                    );
                    issue(
                        &mut cpu,
                        construct_exception_unit_command(
                            &ExceptionUnitOpCodes::ReturnFromException,
                            0,
                        ),
                        false,
                    );
                    for _ in 0..6 {
                        assert!(!tick(&mut cpu, 0).bus_access_strobe);
                    }
                    assert_eq!(cpu.registers.get_full_pc_address(), return_pc);
                    assert_eq!(cpu.eu_registers.current_exception_level, original_level);
                    // RETE derives EA from the restored level, so compare other SR bits.
                    assert_eq!(
                        cpu.registers.sr & !(StatusRegisterFields::ExceptionActive as u16),
                        original_sr & !(StatusRegisterFields::ExceptionActive as u16)
                    );
                    if interrupt.is_none() {
                        issue(
                            &mut cpu,
                            construct_exception_unit_command(
                                &ExceptionUnitOpCodes::SoftwareException,
                                0x60,
                            ),
                            register_call,
                        );
                    }
                    complete_entry(&mut cpu, vector, 0x1000);
                    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 0);
                    assert!(cpu.eu_registers.pending_fault.is_none());
                    let expected_level = interrupt
                        .map_or(1, |mask| u8::try_from(mask.trailing_zeros()).unwrap() + 2);
                    assert_eq!(cpu.eu_registers.current_exception_level, expected_level);
                    assert_eq!(
                        cpu.eu_registers.link_registers[usize::from(expected_level - 1)]
                            .return_address,
                        continuation_pc
                    );
                    // Returning from the successfully retried event cannot deliver it twice.
                    issue(
                        &mut cpu,
                        construct_exception_unit_command(
                            &ExceptionUnitOpCodes::ReturnFromException,
                            0,
                        ),
                        false,
                    );
                    for _ in 0..6 {
                        tick(&mut cpu, 0);
                    }
                    let next = tick(&mut cpu, 0);
                    assert_eq!(next.address, continuation_pc);
                    assert!(next.protected_mode_active);
                    assert_eq!(cpu.active_command, 0);
                }
            }
        }
    }
}
