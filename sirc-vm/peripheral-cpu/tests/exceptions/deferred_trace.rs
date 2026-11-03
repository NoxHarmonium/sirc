use peripheral_bus::{
    conversion::bytes_to_words,
    device::{BusAssertions, Device},
};
use peripheral_cpu::{
    coprocessors::{
        exception_unit::{
            definitions::{
                vectors::{INSTRUCTION_TRACE_FAULT, INVALID_OPCODE_FAULT},
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
    registers::{
        set_hardware_interrupt_enable, set_sr_bit, FullAddressRegisterAccess, RegisterName,
        StatusRegisterFields,
    },
    CpuPeripheral,
};

fn tick(cpu: &mut CpuPeripheral, data: u16, force_trace: bool, irq: u8) -> BusAssertions {
    cpu.poll(
        BusAssertions {
            data,
            force_trace_mode: force_trace,
            interrupt_assertion: irq,
            bus_acknowledge: true,
            ..BusAssertions::default()
        },
        true,
    )
}

fn immediate(
    op_code: Instruction,
    register: &RegisterName,
    value: u16,
    condition_flag: ConditionFlags,
) -> InstructionData {
    InstructionData::Immediate(ImmediateInstructionData {
        op_code,
        register: register.to_register_index(),
        value,
        condition_flag,
        additional_flags: 0,
    })
}

fn call(command: u16) -> InstructionData {
    immediate(
        Instruction::CoprocessorCallImmediate,
        &RegisterName::R1,
        command,
        ConditionFlags::Always,
    )
}

fn run_instruction(
    cpu: &mut CpuPeripheral,
    instruction: &InstructionData,
    force_at_start: bool,
    irq_at_writeback: u8,
) -> BusAssertions {
    let words = bytes_to_words(&encode_instruction(instruction));
    let mut response = BusAssertions::default();
    for (phase, data) in [0, words[0], words[1], 0, 0, 0].into_iter().enumerate() {
        response = tick(
            cpu,
            data,
            force_at_start && phase == 0,
            if phase == 5 { irq_at_writeback } else { 0 },
        );
    }
    assert_eq!(cpu.phase, 0);
    response
}

fn run_slot(cpu: &mut CpuPeripheral, target: u16, force_trace: bool) -> BusAssertions {
    let first = tick(cpu, 0, force_trace, 0);
    for data in [0, target, 0, 0, 0] {
        tick(cpu, data, force_trace, 0);
    }
    assert_eq!(cpu.phase, 0);
    first
}

fn traced_cpu() -> CpuPeripheral {
    let mut cpu = new_cpu_peripheral();
    cpu.registers.pl = 0x0200;
    set_sr_bit(StatusRegisterFields::TraceMode, &mut cpu.registers);
    cpu
}

fn assert_trace(cpu: &CpuPeripheral, pc: u32) {
    let fault = cpu
        .eu_registers
        .pending_fault
        .expect("trace should be ready");
    assert_eq!(fault.fault, Faults::InstructionTrace);
    assert_eq!(fault.pc, pc);
}

#[test]
fn traced_copi_and_copr_complete_the_command_before_trace_and_interrupts() {
    for register_call in [false, true] {
        for irq in [1, 0b10000] {
            let mut cpu = traced_cpu();
            set_hardware_interrupt_enable(&mut cpu.registers, 1);
            let command =
                construct_exception_unit_command(&ExceptionUnitOpCodes::TransferFromRegister, 0x20);
            cpu.eu_registers.link_registers[0].return_status_register = 0xCAFE;
            let instruction = if register_call {
                cpu.registers.r1 = command;
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
                call(command)
            };
            run_instruction(&mut cpu, &instruction, false, irq);
            assert!(cpu.eu_registers.pending_fault.is_none());
            assert_eq!(cpu.registers.coprocessor_select_register, command);
            assert_eq!(cpu.registers.r7, 0);
            for phase in 0..6 {
                tick(&mut cpu, 0, false, 0);
                assert_eq!(cpu.active_command, command);
                if phase != 5 {
                    assert!(cpu.eu_registers.pending_fault.is_none());
                }
                assert_eq!(cpu.eu_registers.pending_hardware_exceptions, irq);
            }
            assert_eq!(cpu.registers.r7, 0xCAFE);
            assert_eq!(cpu.registers.coprocessor_select_register, 0);
            assert_trace(&cpu, 0x0202);
            let response = run_slot(&mut cpu, 0x0800, true);
            assert_eq!(response.address, u32::from(INSTRUCTION_TRACE_FAULT) * 2);
            assert_eq!(cpu.eu_registers.pending_hardware_exceptions, irq);
            assert!(cpu.eu_registers.pending_fault.is_none());
            assert_eq!(cpu.eu_registers.link_registers[6].return_address, 0x0202);
        }
    }
}

#[test]
fn command_fault_suppresses_deferred_trace_and_keeps_the_issuing_pc() {
    let mut cpu = traced_cpu();
    run_instruction(&mut cpu, &call(0x3000), false, 0); // Missing maths unit.
    assert!(cpu.eu_registers.pending_fault.is_none());
    for _ in 0..6 {
        tick(&mut cpu, 0, false, 0);
    }
    let fault = cpu.eu_registers.pending_fault.unwrap();
    assert_eq!(fault.fault, Faults::InvalidOpCode);
    assert_eq!(fault.pc, 0x0200);
    assert_eq!(cpu.registers.coprocessor_select_register, 0);
    let response = run_slot(&mut cpu, 0x0800, true);
    assert_eq!(response.address, u32::from(INVALID_OPCODE_FAULT) * 2);
    assert!(cpu.eu_registers.pending_fault.is_none());
}

#[test]
fn fault_in_the_call_suppresses_trace_without_issuing_a_command() {
    let mut cpu = traced_cpu();
    set_sr_bit(StatusRegisterFields::ProtectedMode, &mut cpu.registers);
    let command =
        construct_exception_unit_command(&ExceptionUnitOpCodes::TransferFromRegister, 0x20);
    run_instruction(&mut cpu, &call(command), false, 0);
    let fault = cpu.eu_registers.pending_fault.unwrap();
    assert_eq!(fault.fault, Faults::PrivilegeViolation);
    assert_eq!(fault.pc, 0x0200);
    assert_eq!(cpu.registers.coprocessor_select_register, 0);
}

#[test]
fn traced_rete_records_the_restored_pc_even_when_it_restores_t_clear() {
    let mut cpu = traced_cpu();
    cpu.eu_registers.current_exception_level = 2;
    cpu.eu_registers.link_registers[1].return_address = 0x0012_3456;
    cpu.eu_registers.link_registers[1].return_status_register = 0;
    let command = construct_exception_unit_command(&ExceptionUnitOpCodes::ReturnFromException, 0);
    run_instruction(&mut cpu, &call(command), false, 0);
    assert!(cpu.eu_registers.pending_fault.is_none());
    run_slot(&mut cpu, 0, false);
    assert_eq!(cpu.registers.get_full_pc_address(), 0x0012_3456);
    assert_trace(&cpu, 0x0012_3456);
    run_slot(&mut cpu, 0x0800, false);
    assert_eq!(
        cpu.eu_registers.link_registers[6].return_address,
        0x0012_3456
    );
}

#[test]
fn command_stalls_do_not_make_trace_ready_and_a_force_trace_pulse_is_retained() {
    let mut cpu = new_cpu_peripheral();
    cpu.registers.pl = 0x0200;
    let command =
        construct_exception_unit_command(&ExceptionUnitOpCodes::TransferFromRegister, 0x20);
    run_instruction(&mut cpu, &call(command), true, 0);
    tick(&mut cpu, 0, false, 0);
    for _ in 0..3 {
        cpu.poll(BusAssertions::default(), true);
        assert_eq!(cpu.phase, 1);
        assert_eq!(cpu.registers.coprocessor_select_register, command);
        assert!(cpu.eu_registers.pending_fault.is_none());
    }
    for _ in 1..6 {
        tick(&mut cpu, 0, false, 0);
    }
    assert_trace(&cpu, 0x0202);
}

#[test]
fn command_vector_bus_error_cancels_its_deferred_trace() {
    for protection_error in [false, true] {
        let mut cpu = traced_cpu();
        let command =
            construct_exception_unit_command(&ExceptionUnitOpCodes::TransferFromRegister, 0x20);
        run_instruction(&mut cpu, &call(command), false, 0);
        tick(&mut cpu, 0, false, 0);
        cpu.poll(
            BusAssertions {
                bus_error: !protection_error,
                bus_protection_error: protection_error,
                ..BusAssertions::default()
            },
            true,
        );
        assert_eq!(cpu.registers.coprocessor_select_register, 0);
        assert_eq!(
            cpu.eu_registers.pending_fault.unwrap().fault,
            if protection_error {
                Faults::BusProtection
            } else {
                Faults::Bus
            }
        );
        run_slot(&mut cpu, 0x0800, false);
        assert!(cpu.eu_registers.pending_fault.is_none());
    }
}

#[test]
fn id_zero_and_false_condition_calls_trace_without_a_command_slot() {
    for condition in [ConditionFlags::Always, ConditionFlags::Equal] {
        let mut cpu = traced_cpu();
        let command = if condition == ConditionFlags::Always {
            0x0042
        } else {
            0x3000
        };
        let instruction = immediate(
            Instruction::CoprocessorCallImmediate,
            &RegisterName::R1,
            command,
            condition,
        );
        run_instruction(&mut cpu, &instruction, false, 0);
        assert_trace(&cpu, 0x0202);
        let response = tick(&mut cpu, 0, false, 0);
        assert_eq!(response.address, u32::from(INSTRUCTION_TRACE_FAULT) * 2);
        assert_eq!(cpu.registers.coprocessor_select_register, 0);
    }
}

#[test]
fn trace_is_sampled_before_an_instruction_changes_t() {
    for initially_enabled in [false, true] {
        let mut cpu = new_cpu_peripheral();
        cpu.registers.pl = 0x0200;
        if initially_enabled {
            set_sr_bit(StatusRegisterFields::TraceMode, &mut cpu.registers);
        }
        let instruction = immediate(
            if initially_enabled {
                Instruction::AndImmediate
            } else {
                Instruction::OrImmediate
            },
            &RegisterName::Sr,
            if initially_enabled {
                !(StatusRegisterFields::TraceMode as u16)
            } else {
                StatusRegisterFields::TraceMode as u16
            },
            ConditionFlags::Always,
        );
        run_instruction(&mut cpu, &instruction, false, 0);
        if initially_enabled {
            assert_trace(&cpu, 0x0202);
        } else {
            assert!(cpu.eu_registers.pending_fault.is_none());
            run_instruction(
                &mut cpu,
                &immediate(
                    Instruction::AddImmediate,
                    &RegisterName::R1,
                    1,
                    ConditionFlags::Always,
                ),
                false,
                0,
            );
            assert_trace(&cpu, 0x0204);
        }
    }
}

#[test]
fn fault_handler_and_its_rete_do_not_trace_even_when_trce_is_held_high() {
    let mut cpu = traced_cpu();
    run_instruction(
        &mut cpu,
        &immediate(
            Instruction::AddImmediate,
            &RegisterName::R1,
            1,
            ConditionFlags::Always,
        ),
        false,
        0,
    );
    run_slot(&mut cpu, 0x0800, true);
    set_sr_bit(StatusRegisterFields::TraceMode, &mut cpu.registers);
    run_instruction(
        &mut cpu,
        &immediate(
            Instruction::AddImmediate,
            &RegisterName::R1,
            1,
            ConditionFlags::Always,
        ),
        true,
        0,
    );
    assert!(cpu.eu_registers.pending_fault.is_none());
    let command = construct_exception_unit_command(&ExceptionUnitOpCodes::ReturnFromException, 0);
    run_instruction(&mut cpu, &call(command), true, 0);
    assert!(cpu.eu_registers.pending_fault.is_none());
    run_slot(&mut cpu, 0, true);
    assert_eq!(cpu.eu_registers.current_exception_level, 0);
    assert_eq!(cpu.registers.pl, 0x0202);
    assert!(cpu.eu_registers.pending_fault.is_none());
    run_instruction(
        &mut cpu,
        &immediate(
            Instruction::AddImmediate,
            &RegisterName::R1,
            1,
            ConditionFlags::Always,
        ),
        false,
        0,
    );
    assert_trace(&cpu, 0x0204);
}

#[test]
fn excp_suppresses_t_and_trce_and_enters_before_waiting_irq() {
    for force_trace in [false, true] {
        let mut cpu = traced_cpu();
        if force_trace {
            cpu.registers.sr = 0;
        }
        set_hardware_interrupt_enable(&mut cpu.registers, 1);
        let command =
            construct_exception_unit_command(&ExceptionUnitOpCodes::SoftwareException, 0x60);
        let original_sr = cpu.registers.sr;
        run_instruction(&mut cpu, &call(command), force_trace, 1);
        assert!(cpu.eu_registers.pending_fault.is_none());
        let response = run_slot(&mut cpu, 0x0800, force_trace);
        assert_eq!(response.address, 0x60 * 2);
        assert_eq!(cpu.registers.pl, 0x0800);
        assert_eq!(cpu.eu_registers.current_exception_level, 1);
        assert_eq!(cpu.eu_registers.link_registers[0].return_address, 0x0202);
        assert_eq!(
            cpu.eu_registers.link_registers[0].return_status_register,
            original_sr
        );
        assert_eq!(cpu.registers.sr & 0x8000, 0);
        assert!(cpu.eu_registers.pending_fault.is_none());
        assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
    }
}

#[test]
fn ignored_handler_excp_suppresses_trace_only_for_that_instruction() {
    for force_trace in [false, true] {
        let mut cpu = traced_cpu();
        cpu.eu_registers.current_exception_level = 2;
        if force_trace {
            cpu.registers.sr = 0;
        }
        let command =
            construct_exception_unit_command(&ExceptionUnitOpCodes::SoftwareException, 0x60);
        run_instruction(&mut cpu, &call(command), force_trace, 0);
        assert!(cpu.eu_registers.pending_fault.is_none());
        // The ignored command consumes no slot. The next PU instruction
        // is traced normally, proving suppression does not leak forward.
        run_instruction(
            &mut cpu,
            &immediate(
                Instruction::AddImmediate,
                &RegisterName::R1,
                1,
                ConditionFlags::Always,
            ),
            force_trace,
            0,
        );
        assert_trace(&cpu, 0x0204);
        assert_eq!(cpu.eu_registers.current_exception_level, 2);
    }
}

#[test]
fn reset_discards_a_deferred_trace() {
    let mut cpu = traced_cpu();
    let command =
        construct_exception_unit_command(&ExceptionUnitOpCodes::TransferFromRegister, 0x20);
    run_instruction(&mut cpu, &call(command), false, 0);
    cpu.reset();
    run_slot(&mut cpu, 0x0400, false);
    assert_eq!(cpu.registers.pl, 0x0400);
    assert!(cpu.eu_registers.pending_fault.is_none());
    run_instruction(
        &mut cpu,
        &immediate(
            Instruction::AddImmediate,
            &RegisterName::R1,
            1,
            ConditionFlags::Always,
        ),
        false,
        0,
    );
    assert!(cpu.eu_registers.pending_fault.is_none());
}

#[test]
fn traced_rset_requests_reset_instead_of_scheduling_trace() {
    let mut cpu = traced_cpu();
    let command = construct_exception_unit_command(&ExceptionUnitOpCodes::Reset, 0);
    let response = run_instruction(&mut cpu, &call(command), false, 0);
    assert!(response.reset_requested);
    assert!(cpu.eu_registers.pending_fault.is_none());
}

#[test]
fn wait_keeps_trace_deferred_while_idle_then_traces_before_the_waking_interrupt() {
    let mut cpu = traced_cpu();
    set_hardware_interrupt_enable(&mut cpu.registers, 1);
    let command = construct_exception_unit_command(&ExceptionUnitOpCodes::WaitForException, 0);
    run_instruction(&mut cpu, &call(command), false, 0);
    for _ in 0..4 {
        tick(&mut cpu, 0, false, 0);
    }
    for _ in 0..3 {
        tick(&mut cpu, 0, false, 0);
        assert!(cpu.eu_registers.waiting_for_exception);
        assert!(cpu.eu_registers.pending_fault.is_none());
        assert_eq!(cpu.registers.coprocessor_select_register, command);
    }
    tick(&mut cpu, 0, false, 1);
    tick(&mut cpu, 0, false, 0);
    assert_trace(&cpu, 0x0202);
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
    let response = tick(&mut cpu, 0, false, 0);
    assert_eq!(response.address, u32::from(INSTRUCTION_TRACE_FAULT) * 2);
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 1);
}

#[test]
fn automatic_interrupt_dispatch_is_not_traced_but_its_handler_can_be() {
    let mut cpu = new_cpu_peripheral();
    set_hardware_interrupt_enable(&mut cpu.registers, 1);
    cpu.eu_registers.pending_hardware_exceptions = 1;
    run_slot(&mut cpu, 0x0800, true);
    assert!(cpu.eu_registers.pending_fault.is_none());
    assert_eq!(cpu.eu_registers.current_exception_level, 2);
    run_instruction(
        &mut cpu,
        &immediate(
            Instruction::AddImmediate,
            &RegisterName::R1,
            1,
            ConditionFlags::Always,
        ),
        true,
        0,
    );
    assert_trace(&cpu, 0x0802);
}
