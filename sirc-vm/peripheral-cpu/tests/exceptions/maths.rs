use super::optional_machine::Machine;
use peripheral_bus::{
    device::{BusAccessType, BusAssertions},
    memory_mapped_device::new_stub_memory_mapped_device,
    new_bus_peripheral,
};
use peripheral_cpu::{
    coprocessors::{
        exception_unit::definitions::Faults, maths::MathsCoprocessor,
        processing_unit::definitions::ConditionFlags,
    },
    registers::{set_sr_bit, StatusRegisterFields},
    CpuConfig, CpuPeripheral,
};

const MULU: u16 = 0x3000;
const MULS: u16 = 0x3100;
const DIVU: u16 = 0x3200;
const DIVS: u16 = 0x3300;
const NORMAL_CYCLES: usize = 48;
const EARLY_ERROR_CYCLES: usize = 12;
const COP_PC: u32 = 0xcd00_0200;
const RESUME_PC: u32 = 0xcd00_0202;
const L5_PIN: u8 = 0b1_0000;
const TRACE_VECTOR: u32 = 12;
const L5_VECTOR: u32 = 0x20;
fn machine(command: u16, register_call: bool, condition: ConditionFlags) -> Machine {
    let mut m = Machine::with_config(
        command,
        register_call,
        condition,
        CpuConfig {
            optional_coprocessors: vec![Box::new(MathsCoprocessor::default())],
        },
    );
    m.cpu.registers.r1 = 0;
    m.cpu.registers.r2 = 100;
    m.cpu.registers.r3 = 7;
    m
}

#[test]
fn every_cycle_has_idle_bus_and_atomic_writeback_with_exact_resume_timing() {
    for (command, input, result, elapsed) in [
        (MULU, (0xffff, 0xffff, 0), (0xfffe, 1, 0), NORMAL_CYCLES),
        (MULS, (0x8000, 0xffff, 0), (0, 0x8000, 0), NORMAL_CYCLES),
        (DIVU, (0, 100, 7), (2, 14, 0), NORMAL_CYCLES),
        (
            DIVS,
            (0xffff, 0xff9c, 7),
            (0xfffe, 0xfff2, 0),
            NORMAL_CYCLES,
        ),
        (DIVU, (1, 2, 0), (1, 2, 3), EARLY_ERROR_CYCLES),
        (DIVU, (7, 2, 7), (7, 2, 5), EARLY_ERROR_CYCLES),
        (DIVS, (0, 0x8000, 1), (0, 0x8000, 5), NORMAL_CYCLES),
    ] {
        for register_call in [false, true] {
            for protected in [false, true] {
                let mut m = machine(command, register_call, ConditionFlags::Always);
                (m.cpu.registers.r1, m.cpu.registers.r2, m.cpu.registers.r3) = input;
                if protected {
                    set_sr_bit(StatusRegisterFields::ProtectedMode, &mut m.cpu.registers);
                }
                let mut expected = m.cpu.registers;
                expected.pl = 0x0202;
                let writeback = if elapsed == NORMAL_CYCLES { 46 } else { 11 };
                for cycle in 0..elapsed {
                    let out = m.clock(BusAssertions::default(), false);
                    if cycle >= 6 {
                        assert!(!out.bus_access_strobe && !out.instruction_sync);
                        assert_eq!(m.cpu.registers.coprocessor_select_register, command);
                        if cycle < writeback {
                            assert_eq!(
                                (m.cpu.registers.r1, m.cpu.registers.r2, m.cpu.registers.r3),
                                input
                            );
                        } else {
                            assert_eq!(
                                (m.cpu.registers.r1, m.cpu.registers.r2, m.cpu.registers.r3),
                                result
                            );
                        }
                    }
                    assert!(m.cpu.eu_registers.pending_fault.is_none());
                    assert_eq!(usize::from(m.cpu.phase), (cycle + 1) % 6);
                }
                let out = m.clock(BusAssertions::default(), false);
                assert!(out.instruction_sync && out.bus_access_strobe);
                assert_eq!(out.address, 0x0202);
                expected.coprocessor_select_register = 0;
                (expected.r1, expected.r2, expected.r3) = result;
                assert_eq!(m.cpu.registers, expected);
            }
        }
    }
}

#[test]
fn skipped_reserved_and_absent_commands_keep_normal_dispatch_and_fault_rules() {
    for command in [MULU, MULS, DIVU, DIVS, 0x3f00] {
        let mut m = machine(command, false, ConditionFlags::Never);
        let mut expected = m.cpu.registers;
        expected.pl = 0x0202;
        assert_eq!(m.run_to_resume(), 6);
        assert_eq!(m.cpu.registers, expected);
    }
    for opcode in 4..16 {
        let command = MULU | (opcode << 8);
        let mut m = machine(command, false, ConditionFlags::Always);
        for _ in 0..7 {
            m.clock(BusAssertions::default(), false);
        }
        let fault = m.cpu.eu_registers.pending_fault.unwrap();
        assert_eq!(fault.fault, Faults::InvalidOpCode);
        assert_eq!(fault.pc, COP_PC);
        assert_eq!(
            m.cpu.eu_registers.link_registers[7].return_address,
            u32::from(command)
        );
        assert_eq!(m.cpu.registers.coprocessor_select_register, 0);
    }
    let mut m = Machine::with_config(MULU, false, ConditionFlags::Always, CpuConfig::default());
    for _ in 0..7 {
        m.clock(BusAssertions::default(), false);
    }
    assert_eq!(
        m.cpu.eu_registers.pending_fault.unwrap().fault,
        Faults::InvalidOpCode
    );
}

#[test]
fn halt_at_every_arithmetic_phase_freezes_work_and_retains_l5_until_retirement() {
    for offset in 7..NORMAL_CYCLES {
        let mut m = machine(DIVU, false, ConditionFlags::Always);
        for _ in 0..offset {
            m.clock(BusAssertions::default(), false);
        }
        while m.cpu.phase != 0 {
            m.clock(
                BusAssertions {
                    halt_requested: true,
                    ..Default::default()
                },
                false,
            );
        }
        let halted_at = m.cycles;
        let mut before = m.cpu.registers;
        // A completed unit retires before phase-0 HALT is sampled.
        if halted_at == NORMAL_CYCLES {
            before.coprocessor_select_register = 0;
        }
        for cycle in 0..10 {
            let out = m.clock(
                BusAssertions {
                    halt_requested: true,
                    interrupt_assertion: if cycle == 0 { L5_PIN } else { 0 },
                    ..Default::default()
                },
                false,
            );
            assert!(!out.bus_access_strobe && !out.instruction_sync);
            assert_eq!(m.cpu.registers, before);
        }
        assert!(halted_at - offset <= 5);
        for cycle in halted_at..=NORMAL_CYCLES {
            let out = m.clock(BusAssertions::default(), false);
            if cycle < NORMAL_CYCLES {
                assert_ne!(out.bus_access_type, BusAccessType::ExceptionVectorFetch);
            } else {
                assert_eq!(out.bus_access_type, BusAccessType::ExceptionVectorFetch);
                assert_eq!(out.address, L5_VECTOR);
                assert_eq!(
                    (m.cpu.registers.r1, m.cpu.registers.r2, m.cpu.registers.r3),
                    (2, 14, 0)
                );
                assert_eq!(m.cpu.registers.coprocessor_select_register, 0);
            }
        }
    }
}

#[test]
fn sampled_trace_waits_for_arithmetic_writeback_and_precedes_queued_l5_even_on_status_errors() {
    for (command, divisor, elapsed) in [
        (MULU, 7, NORMAL_CYCLES),
        (DIVU, 7, NORMAL_CYCLES),
        (DIVU, 0, EARLY_ERROR_CYCLES),
    ] {
        for pin_trace in [false, true] {
            let mut m = machine(command, false, ConditionFlags::Always);
            m.cpu.registers.r3 = divisor;
            if !pin_trace {
                set_sr_bit(StatusRegisterFields::TraceMode, &mut m.cpu.registers);
            }
            for cycle in 0..elapsed {
                m.clock(
                    BusAssertions {
                        force_trace_mode: pin_trace && cycle == 0,
                        interrupt_assertion: if cycle == 5 { L5_PIN } else { 0 },
                        ..Default::default()
                    },
                    false,
                );
                assert!(m.cpu.eu_registers.pending_fault.is_none());
            }
            let out = m.clock(BusAssertions::default(), false);
            assert_eq!(out.address, TRACE_VECTOR);
            let fault = m.cpu.eu_registers.pending_fault.unwrap();
            assert_eq!(fault.fault, Faults::InstructionTrace);
            assert_eq!(fault.pc, RESUME_PC);
            assert_eq!(m.cpu.eu_registers.pending_hardware_exceptions, L5_PIN);
        }
    }
}

#[test]
fn reset_mid_calculation_discards_internal_progress_and_fetches_the_reset_vector() {
    let m = machine(DIVU, false, ConditionFlags::Always);
    let mut bus = new_bus_peripheral(Box::new(m.cpu));
    bus.map_segment(
        "PROGRAM",
        0,
        65_536,
        true,
        Box::new(new_stub_memory_mapped_device()),
    );
    for (address, value) in m.memory {
        bus.write_address(address, value);
    }
    bus.write_address(0, 0);
    bus.write_address(1, 0x0400);
    let mut pins = BusAssertions::default();
    for _ in 0..25 {
        pins = bus.poll_all(pins);
    }
    pins.reset_requested = true;
    for _ in 0..6 {
        pins = bus.poll_all(pins);
        assert!(pins.reset_devices_on_bus);
        assert!(!pins.bus_access_strobe);
    }
    for _ in 0..20 {
        pins = bus.poll_all(pins);
        if pins.bus_access_type == BusAccessType::InstructionFetch {
            assert_eq!(pins.address, 0x0400);
            let cpu = bus
                .bus_master
                .as_any()
                .downcast_mut::<CpuPeripheral>()
                .unwrap();
            assert_eq!(
                (cpu.registers.r1, cpu.registers.r2, cpu.registers.r3),
                (0, 100, 7)
            );
            assert_eq!(cpu.registers.coprocessor_select_register, 0);
            return;
        }
    }
    panic!("reset did not resume");
}

#[test]
fn issuing_fetch_waits_extend_total_time_without_changing_arithmetic_steps() {
    for waits in [1, 2, 7] {
        let mut m = machine(DIVU, false, ConditionFlags::Always);
        m.clock(BusAssertions::default(), false);
        let request = m.pending.unwrap();
        for _ in 0..waits {
            let held = m.clock(BusAssertions::default(), true);
            assert_eq!(held.address, request.address);
            assert!(held.instruction_sync);
        }
        assert_eq!(m.run_to_resume(), NORMAL_CYCLES + waits);
        assert_eq!(
            (m.cpu.registers.r1, m.cpu.registers.r2, m.cpu.registers.r3),
            (2, 14, 0)
        );
    }
}

#[test]
fn maths_and_dma_dispatch_back_to_back_without_sharing_hidden_state() {
    use peripheral_bus::conversion::bytes_to_words;
    use peripheral_cpu::coprocessors::{
        dma::DmaCoprocessor,
        processing_unit::{
            definitions::{ImmediateInstructionData, Instruction, InstructionData},
            encoding::encode_instruction,
        },
    };
    const DMAW_ONE_WORD_FROM_A: u16 = 0x2901;
    let mut m = Machine::with_config(
        MULU,
        false,
        ConditionFlags::Always,
        CpuConfig {
            optional_coprocessors: vec![
                Box::new(MathsCoprocessor::default()),
                Box::new(DmaCoprocessor::default()),
            ],
        },
    );
    m.cpu.registers.r1 = 0xffff;
    m.cpu.registers.r2 = 0xffff;
    let words = bytes_to_words(&encode_instruction(&InstructionData::Immediate(
        ImmediateInstructionData {
            op_code: Instruction::CoprocessorCallImmediate,
            register: 0,
            value: DMAW_ONE_WORD_FROM_A,
            condition_flag: ConditionFlags::Always,
            additional_flags: 0,
        },
    )));
    m.memory.insert(0x0202, words[0]);
    m.memory.insert(0x0203, words[1]);
    for cycle in 0..=60 {
        let out = m.clock(BusAssertions::default(), false);
        if cycle == NORMAL_CYCLES {
            assert_eq!(out.address, 0x0202);
        }
        if cycle == 60 {
            assert_eq!(out.bus_access_type, BusAccessType::InstructionFetch);
            assert_eq!(out.address, 0x0204);
        }
        assert!(m.cpu.eu_registers.pending_fault.is_none());
    }
    assert_eq!(m.memory[&0x0001_1000], 0xfffe);
    assert_eq!(m.cpu.registers.al, 0x1001);
    assert_eq!(
        (m.cpu.registers.r1, m.cpu.registers.r2, m.cpu.registers.r3),
        (0xfffe, 1, 0)
    );
}
