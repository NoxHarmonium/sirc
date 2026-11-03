use peripheral_bus::conversion::bytes_to_words;
use peripheral_cpu::{
    coprocessors::{
        exception_unit::{
            definitions::{
                vectors::{DOUBLE_FAULT_VECTOR, PRIVILEGE_VIOLATION_FAULT},
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
    new_cpu_peripheral,
    registers::{set_sr_bit, FullAddressRegisterAccess, RegisterName, StatusRegisterFields},
};

use super::common::tick;

fn call(command: u16, register: bool, condition: ConditionFlags) -> InstructionData {
    if register {
        InstructionData::Register(RegisterInstructionData {
            op_code: Instruction::CoprocessorCallRegister,
            r1: 0,
            r2: 0,
            r3: RegisterName::R1.to_register_index(),
            shift_operand: ShiftOperand::Immediate,
            shift_type: ShiftType::None,
            shift_count: 0,
            condition_flag: condition,
            additional_flags: 0,
        })
    } else {
        InstructionData::Immediate(ImmediateInstructionData {
            op_code: Instruction::CoprocessorCallImmediate,
            register: 0,
            value: command,
            condition_flag: condition,
            additional_flags: 0,
        })
    }
}

fn instruction_data(instruction: &InstructionData) -> [u16; 6] {
    let words = bytes_to_words(&encode_instruction(instruction));
    [0, words[0], words[1], 0, 0, 0]
}

#[test]
fn reserved_excp_faults_in_phase_three_before_dispatch_with_issuing_pc() {
    for register in [false, true] {
        for vector in [0x00, 0x10, 0x5f] {
            for level in [0, 1, 3, 7] {
                for pc in [0, 0x0034_fffe] {
                    let mut cpu = new_cpu_peripheral();
                    cpu.registers.set_full_pc_address(pc);
                    cpu.registers.r1 = construct_exception_unit_command(
                        &ExceptionUnitOpCodes::SoftwareException,
                        vector,
                    );
                    set_sr_bit(StatusRegisterFields::ProtectedMode, &mut cpu.registers);
                    // Fault must win over the issuing instruction's sampled trace.
                    set_sr_bit(StatusRegisterFields::TraceMode, &mut cpu.registers);
                    cpu.eu_registers.current_exception_level = level;
                    let instruction = call(cpu.registers.r1, register, ConditionFlags::Always);
                    for (phase, data) in instruction_data(&instruction).into_iter().enumerate() {
                        let response = tick(&mut cpu, data);
                        assert_eq!(response.bus_access_strobe, phase < 2);
                        assert_eq!(cpu.registers.coprocessor_select_register, 0);
                        if phase < 3 {
                            assert!(cpu.eu_registers.pending_fault.is_none());
                        } else {
                            let fault = cpu.eu_registers.pending_fault.unwrap();
                            assert_eq!(
                                fault.fault,
                                if level == 7 {
                                    Faults::DoubleFault
                                } else {
                                    Faults::PrivilegeViolation
                                }
                            );
                            assert_eq!(fault.pc, pc);
                        }
                    }
                    let vector = if level == 7 {
                        DOUBLE_FAULT_VECTOR
                    } else {
                        PRIVILEGE_VIOLATION_FAULT
                    };
                    let response = tick(&mut cpu, 0);
                    assert!(response.bus_access_strobe);
                    assert_eq!(response.address, u32::from(vector) * 2);
                    let response = tick(&mut cpu, 0);
                    assert!(response.bus_access_strobe);
                    assert_eq!(response.address, u32::from(vector) * 2 + 1);
                    for data in [0x0800, 0, 0, 0] {
                        tick(&mut cpu, data);
                    }
                    assert_eq!(cpu.eu_registers.link_registers[6].return_address, pc);
                    assert_eq!(cpu.eu_registers.current_exception_level, 7);
                }
            }
        }
    }
}

#[test]
fn legal_handler_excp_is_ignored_without_vector_reads_or_queued_trap() {
    for register in [false, true] {
        for (protected, vector) in [(false, 0x10), (false, 0x5f), (true, 0x60), (true, 0xff)] {
            for level in [1, 3, 7] {
                let mut cpu = new_cpu_peripheral();
                cpu.registers.pl = 0x0200;
                cpu.eu_registers.current_exception_level = level;
                if protected {
                    set_sr_bit(StatusRegisterFields::ProtectedMode, &mut cpu.registers);
                }
                let command = construct_exception_unit_command(
                    &ExceptionUnitOpCodes::SoftwareException,
                    vector,
                );
                cpu.registers.r1 = command;
                for data in instruction_data(&call(command, register, ConditionFlags::Always)) {
                    tick(&mut cpu, data);
                }
                assert!(cpu.eu_registers.pending_fault.is_none());
                // The validated command keeps an idle six-cycle dispatch slot.
                let mut clocks = 0;
                super::common::run_idle_dispatch(&mut cpu, &mut clocks);
                let response = tick(&mut cpu, 0);
                assert!(response.bus_access_strobe);
                assert_eq!(response.address, 0x0202);
                assert_eq!(cpu.registers.coprocessor_select_register, 0);
                assert_eq!(cpu.active_command, 0);
                assert_eq!(cpu.eu_registers.current_exception_level, level);
                assert!(cpu.eu_registers.pending_fault.is_none());
            }
        }
    }
}

#[test]
fn legal_level_zero_excp_dispatches_normally_in_both_modes() {
    for register in [false, true] {
        for (protected, vector) in [
            (false, 0x00),
            (false, 0x10),
            (false, 0x5f),
            (true, 0x60),
            (true, 0xff),
        ] {
            let mut cpu = new_cpu_peripheral();
            cpu.registers.pl = 0x0200;
            if protected {
                set_sr_bit(StatusRegisterFields::ProtectedMode, &mut cpu.registers);
            }
            set_sr_bit(StatusRegisterFields::TraceMode, &mut cpu.registers);
            let command =
                construct_exception_unit_command(&ExceptionUnitOpCodes::SoftwareException, vector);
            cpu.registers.r1 = command;
            for data in instruction_data(&call(command, register, ConditionFlags::Always)) {
                tick(&mut cpu, data);
            }
            assert!(
                cpu.eu_registers.pending_fault.is_none(),
                "register={register} protected={protected} vector={vector:x} fault={:?}",
                cpu.eu_registers.pending_fault
            );
            assert_eq!(cpu.registers.coprocessor_select_register, command);
            let response = tick(&mut cpu, 0);
            assert_eq!(response.address, u32::from(vector) * 2);
            for data in [0, 0x0800, 0, 0, 0] {
                tick(&mut cpu, data);
            }
            assert_eq!(cpu.eu_registers.current_exception_level, 1);
            assert_eq!(cpu.eu_registers.link_registers[0].return_address, 0x0202);
            assert!(cpu.eu_registers.pending_fault.is_none());
        }
    }
}

#[test]
fn false_condition_reserved_excp_does_not_fault_or_dispatch() {
    for register in [false, true] {
        for level in [0, 3, 7] {
            let mut cpu = new_cpu_peripheral();
            cpu.registers.pl = 0x0200;
            cpu.eu_registers.current_exception_level = level;
            set_sr_bit(StatusRegisterFields::ProtectedMode, &mut cpu.registers);
            let command =
                construct_exception_unit_command(&ExceptionUnitOpCodes::SoftwareException, 0x10);
            cpu.registers.r1 = command;
            for data in instruction_data(&call(command, register, ConditionFlags::Equal)) {
                tick(&mut cpu, data);
            }
            assert!(cpu.eu_registers.pending_fault.is_none());
            assert_eq!(cpu.registers.coprocessor_select_register, 0);
            let response = tick(&mut cpu, 0);
            assert_eq!(response.address, 0x0202);
            assert_eq!(cpu.eu_registers.current_exception_level, level);
        }
    }
}
