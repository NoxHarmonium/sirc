use assert_hex::assert_eq_hex;
use peripheral_bus::{
    device::{BusAssertions, Device},
    memory_mapped_device::new_stub_memory_mapped_device,
    new_bus_peripheral,
};
use peripheral_cpu::{
    coprocessors::exception_unit::definitions::{Faults, PendingFault},
    coprocessors::processing_unit::{
        definitions::{ConditionFlags, ImmediateInstructionData, Instruction, InstructionData},
        encoding::encode_instruction,
    },
    new_cpu_peripheral,
    registers::FullAddressRegisterAccess,
    CpuPeripheral, CYCLES_PER_INSTRUCTION,
};

fn set_up_reset_test() -> peripheral_bus::BusPeripheral {
    let cpu = new_cpu_peripheral();
    let mut bus = new_bus_peripheral(Box::new(cpu));
    // Segment at address 0 so the EU can fetch the reset vector (all zeroes → PC = 0x0).
    bus.map_segment(
        "RESET_VEC",
        0x0,
        u32::from(u16::MAX) + 1,
        false,
        Box::new(new_stub_memory_mapped_device()),
    );
    bus
}

fn set_up_reset_test_with_instruction(
    instruction: &InstructionData,
) -> peripheral_bus::BusPeripheral {
    let mut bus = set_up_reset_test();
    let encoded = encode_instruction(instruction);
    // Write the instruction at address 0 (where the CPU starts)
    bus.load_binary_data_into_segment("RESET_VEC", &encoded);
    bus
}

fn get_cpu(bus: &mut peripheral_bus::BusPeripheral) -> &mut CpuPeripheral {
    bus.bus_master
        .as_any()
        .downcast_mut::<CpuPeripheral>()
        .expect("failed to downcast")
}

/// Run n polls, feeding each output back as the input to the next cycle.
fn poll_n_feedback(
    bus: &mut peripheral_bus::BusPeripheral,
    mut assertions: BusAssertions,
    count: u32,
) -> BusAssertions {
    for _ in 0..count {
        assertions = bus.poll_all(assertions);
    }
    assertions
}

const ACK: BusAssertions = BusAssertions {
    address: 0,
    data: 0,
    op: peripheral_bus::device::BusOperation::Read,
    interrupt_assertion: 0,
    bus_access_strobe: false,
    bus_acknowledge: true,
    bus_error: false,
    bus_protection_error: false,
    bus_access_type: peripheral_bus::device::BusAccessType::None,
    device_was_activated: false,
    exit_simulation: false,
    reset_requested: false,
    reset_devices_on_bus: false,
    halt_requested: false,
    force_trace_mode: false,
    instruction_sync: false,
    protected_mode_active: false,
};

const RSTI_ASSERTED: BusAssertions = BusAssertions {
    reset_requested: true,
    ..ACK
};

/// Software RSET: executing the RSET COP instruction causes `poll_all` to emit RSTO for 6 cycles
/// before the EU fetches the reset vector. The internal request's write-back cycle counts as cycle 1.
#[test]
fn test_software_rset_hold() {
    // RSET is encoded as COPI r0, #0x1B00 (EU ID=1, Reset opcode=0xB, vector=0x00)
    let rset_instruction = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::CoprocessorCallImmediate,
        register: 0x0,
        value: 0x1B00,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0x0,
    });
    let mut bus = set_up_reset_test_with_instruction(&rset_instruction);

    // Use a feedback loop (output of each poll becomes input to the next) so that instruction
    // data from the memory device flows correctly to the CPU — without it the CPU decodes zeros.
    let mut assertions = BusAssertions::default();

    // Polls 0-4: PU phases InstructionFetchLow through MemoryAccess. No RSTO yet.
    for i in 0..u32::from(CYCLES_PER_INSTRUCTION) - 1 {
        assertions = bus.poll_all(assertions);
        assert!(
            !assertions.reset_devices_on_bus,
            "RSTO should not be asserted on PU cycle {i}"
        );
    }

    // Poll 5 (WriteBack): PU sets coprocessor_select_register = RESET_COMMAND, phase wraps to 0,
    // poll_all consumes its internal request and emits RSTO cycle 1, never RSTI.
    assertions = bus.poll_all(assertions);
    assert!(
        assertions.reset_devices_on_bus && !assertions.reset_requested,
        "WriteBack must start RSTO without driving RSTI"
    );
    assert!(
        !assertions.bus_access_strobe,
        "no bus activity expected during reset hold"
    );

    // Polls 6-10: 5 countdown cycles (total 6 RSTO cycles).
    for i in 0..5 {
        assertions = bus.poll_all(assertions);
        assert!(
            assertions.reset_devices_on_bus,
            "RSTO should be asserted on countdown cycle {i}"
        );
        assert!(!assertions.bus_access_strobe);
    }

    // Poll 11: hold expired, EU resumes and fetches reset vector from address 0.
    assertions = bus.poll_all(assertions);
    assert!(
        !assertions.reset_devices_on_bus,
        "RSTO should be deasserted after hold"
    );
    assert!(
        assertions.bus_access_strobe,
        "EU should be fetching reset vector"
    );
    // The reset vector table lives at address 0, which is also where the RSET instruction was
    // written. The EU will read those instruction bytes as the vector address.
    // This is a test-setup artifact, not a bug. Checking bus_access_strobe is sufficient to
    // confirm the RSTO hold completed and the EU resumed correctly.
}

/// Hardware RSTI: asserting the RSTI pin causes 6 cycles of RSTO then EU reset, without any
/// instruction needing to execute first.
#[test]
fn test_hardware_rsti_reset() {
    let mut bus = set_up_reset_test();

    // Assert RSTI for 1 cycle — reset unit immediately returns RSTO and calls bus_master.reset().
    let out = bus.poll_all(RSTI_ASSERTED);
    assert!(
        out.reset_devices_on_bus,
        "RSTO should be asserted on RSTI assertion cycle"
    );
    assert!(
        !out.bus_access_strobe,
        "CPU should not be active during reset hold"
    );

    // 5 countdown cycles (RSTI now released) — all RSTO.
    for i in 0..5 {
        let out = bus.poll_all(ACK);
        assert!(
            out.reset_devices_on_bus,
            "RSTO should be asserted on countdown cycle {i}"
        );
        assert!(!out.bus_access_strobe);
    }

    // After 6 RSTO cycles, CPU resumes — EU fetches reset vector at address 0.
    let out = bus.poll_all(ACK);
    assert!(
        !out.reset_devices_on_bus,
        "RSTO should be deasserted after hold"
    );
    assert!(out.bus_access_strobe, "EU should be fetching reset vector");
    assert_eq_hex!(
        0x0,
        out.address,
        "EU should fetch from address 0 (reset vector)"
    );

    // Complete the remaining 5 EU phases.
    poll_n_feedback(&mut bus, ACK, u32::from(CYCLES_PER_INSTRUCTION) - 1);
    assert_eq_hex!(0x0, get_cpu(&mut bus).registers.get_full_pc_address());
}

/// RSTI held continuously keeps RSTO asserted and prevents the CPU from running.
#[test]
fn test_rsti_held_keeps_rsto() {
    let mut bus = set_up_reset_test();

    for i in 0..10 {
        let out = bus.poll_all(RSTI_ASSERTED);
        assert!(
            out.reset_devices_on_bus,
            "RSTO should be asserted on cycle {i} while RSTI held"
        );
        assert!(
            !out.bus_access_strobe,
            "CPU should not advance while RSTI held"
        );
    }

    // After release, 5 countdown cycles of RSTO, then CPU resumes.
    for i in 0..5 {
        let out = bus.poll_all(ACK);
        assert!(
            out.reset_devices_on_bus,
            "RSTO should be asserted on countdown cycle {i} after RSTI release"
        );
    }
    let out = bus.poll_all(ACK);
    assert!(
        !out.reset_devices_on_bus,
        "RSTO should deassert after countdown"
    );
    assert!(out.bus_access_strobe, "EU should be active after hold");
}

/// RSTI clears the `waiting_for_exception` (WFE / low-power) state immediately.
#[test]
fn test_rsti_during_wfe() {
    let mut bus = set_up_reset_test();

    get_cpu(&mut bus).eu_registers.waiting_for_exception = true;

    // Without RSTI, CPU idles.
    let out = bus.poll_all(ACK);
    assert!(!out.bus_access_strobe, "CPU should be idle in WFE mode");

    // RSTI aborts WFE and starts the reset hold.
    let out = bus.poll_all(RSTI_ASSERTED);
    assert!(
        out.reset_devices_on_bus,
        "RSTO should be asserted after RSTI"
    );
    assert!(
        !get_cpu(&mut bus).eu_registers.waiting_for_exception,
        "WFE flag should be cleared by RSTI"
    );
}

/// Reset is a clean exception boundary: pending faults, pending hardware exceptions, and the
/// active exception level are cleared before reset-vector fetch can occur.
#[test]
fn test_reset_clears_pending_exception_state() {
    let mut bus = set_up_reset_test();

    {
        let cpu = get_cpu(&mut bus);
        cpu.eu_registers.pending_fault = Some(PendingFault {
            fault: Faults::BusProtection,
            pc: 0x0,
        });
        cpu.registers.sr = 0xffff;
        cpu.registers.r1 = 0xcafe;
        cpu.registers.ah = 0x1234;
        cpu.eu_registers.link_registers[5].return_address = 0x5566;
        cpu.active_command = 0x1234;
        cpu.processing_unit_executor
            .next_instruction_fetch_is_overflow = true;
        cpu.eu_registers.l5_overrun = true;
        cpu.eu_registers.pending_hardware_exceptions = 0x1F;
        cpu.eu_registers.current_exception_level = 7;
        cpu.eu_registers.waiting_for_exception = true;
    }

    let mut expected_registers = get_cpu(&mut bus).registers;
    expected_registers.sr = 0;
    expected_registers.coprocessor_select_register = 0x1b00;
    let links = get_cpu(&mut bus).eu_registers.link_registers;
    let out = bus.poll_all(RSTI_ASSERTED);
    assert!(
        out.reset_devices_on_bus,
        "RSTO should be asserted after RSTI"
    );

    let cpu = get_cpu(&mut bus);
    assert_eq!(cpu.registers, expected_registers);
    assert_eq!(cpu.eu_registers.link_registers, links);
    assert_eq!(cpu.active_command, 0);
    assert!(
        !cpu.processing_unit_executor
            .next_instruction_fetch_is_overflow
    );
    assert_eq!(cpu.phase, 0);
    assert_eq!(cpu.registers.coprocessor_select_register, 0x1b00);
    assert!(!cpu.eu_registers.l5_overrun);
    assert_eq!(None, cpu.eu_registers.pending_fault);
    assert_eq!(0, cpu.eu_registers.pending_hardware_exceptions);
    assert_eq!(0, cpu.eu_registers.current_exception_level);
    assert!(
        !cpu.eu_registers.waiting_for_exception,
        "WFE flag should be cleared by reset"
    );
}

/// RSTI mid-instruction: the CPU was stalling waiting for bus ack. RSTI should abort the stall
/// and start the reset hold — verified by observing that subsequent polls emit RSTO rather than
/// replaying the stalled request.
#[test]
fn test_rsti_mid_instruction_abort() {
    let mut bus = set_up_reset_test();

    // First poll without ack — CPU issues BAS for InstructionFetchLow.
    let no_ack = BusAssertions::default();
    let out = bus.poll_all(no_ack);
    let stalled_address = out.address;
    assert!(out.bus_access_strobe, "CPU should have issued BAS");

    // Second poll without ack — CPU stalls and re-emits the same request.
    let out2 = bus.poll_all(no_ack);
    assert_eq!(
        stalled_address, out2.address,
        "CPU should still be stalling"
    );

    // Assert RSTI — reset unit calls bus_master.reset(), starts hold, returns RSTO.
    let out = bus.poll_all(RSTI_ASSERTED);
    assert!(
        out.reset_devices_on_bus,
        "RSTO should be asserted after RSTI"
    );
    assert!(
        !out.bus_access_strobe,
        "stall should be aborted — no BAS during hold"
    );

    // Complete the 5-cycle countdown.
    poll_n_feedback(&mut bus, ACK, 5);

    // EU resumes from phase 0 at reset vector address 0, not the stalled address.
    let out = bus.poll_all(ACK);
    assert!(!out.reset_devices_on_bus);
    assert!(out.bus_access_strobe, "EU should be fetching reset vector");
    assert_eq_hex!(
        0x0,
        out.address,
        "EU should fetch from address 0, not the stalled address"
    );
}

#[test]
fn triple_fault_starts_one_six_cycle_hold_without_driving_rsti() {
    for second_word in [false, true] {
        for protection_error in [false, true] {
            let mut bus = set_up_reset_test();
            get_cpu(&mut bus).eu_registers.pending_fault = Some(PendingFault {
                fault: Faults::DoubleFault,
                pc: 0x0200,
            });
            let first = bus.poll_all(ACK);
            assert!(first.bus_access_strobe);
            assert_eq!(first.address, 0x10);
            if second_word {
                assert_eq!(bus.poll_all(ACK).address, 0x11);
            }
            let out = bus.poll_all(BusAssertions {
                bus_error: !protection_error,
                bus_protection_error: protection_error,
                ..ACK
            });
            assert!(out.reset_devices_on_bus);
            assert!(!out.reset_requested);
            assert!(!out.bus_access_strobe);
            assert!(!get_cpu(&mut bus).take_reset_request());
            for _ in 0..5 {
                let held = bus.poll_all(ACK);
                assert!(held.reset_devices_on_bus);
                assert!(!held.reset_requested);
                assert!(!held.bus_access_strobe);
            }
            let fetch = bus.poll_all(ACK);
            assert!(!fetch.reset_devices_on_bus);
            assert!(!fetch.reset_requested);
            assert!(fetch.bus_access_strobe);
            assert_eq!(fetch.address, 0);
        }
    }
}

#[test]
fn software_reset_clears_state_in_the_issuing_write_back_cycle() {
    let mut cpu = new_cpu_peripheral();
    cpu.registers.sr = 0x8005;
    cpu.registers.r1 = 0xcafe;
    cpu.registers.ah = 0x1234;
    cpu.eu_registers.current_exception_level = 7;
    cpu.eu_registers.pending_hardware_exceptions = 31;
    cpu.eu_registers.l5_overrun = true;
    cpu.eu_registers.link_registers[5].return_address = 0x5566;
    let instruction = InstructionData::Immediate(ImmediateInstructionData {
        op_code: Instruction::CoprocessorCallImmediate,
        register: 0,
        value: 0x1b00,
        condition_flag: ConditionFlags::Always,
        additional_flags: 0,
    });
    let words = peripheral_bus::conversion::bytes_to_words(&encode_instruction(&instruction));
    for data in [0, words[0], words[1], 0, 0, 0] {
        assert!(!super::common::tick(&mut cpu, data).reset_requested);
    }
    assert_eq!(cpu.registers.sr, 0);
    assert_eq!(cpu.eu_registers.current_exception_level, 0);
    assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 0);
    assert!(!cpu.eu_registers.l5_overrun);
    assert!(cpu.eu_registers.pending_fault.is_none());
    assert_eq!(cpu.active_command, 0);
    assert_eq!(cpu.phase, 0);
    assert_eq!(cpu.registers.r1, 0xcafe);
    assert_eq!(cpu.registers.ah, 0x1234);
    assert_eq!(cpu.eu_registers.link_registers[5].return_address, 0x5566);
    assert!(cpu.take_reset_request());
}

#[test]
fn reset_vector_failures_repeat_full_holds_and_exclude_interrupt_sampling() {
    for failed_word in [0, 1] {
        for protection_error in [false, true] {
            let mut bus = set_up_reset_test();
            bus.load_binary_data_into_segment("RESET_VEC", &[0, 0, 8, 0]);
            get_cpu(&mut bus).registers.r1 = 0xcafe;
            get_cpu(&mut bus).eu_registers.link_registers[5].return_address = 0x5566;
            let mut out = bus.poll_all(RSTI_ASSERTED);
            for _attempt in 0..3 {
                assert!(out.reset_devices_on_bus);
                assert!(!out.reset_requested);
                assert!(!out.bus_access_strobe);
                for i in 0..5 {
                    out = bus.poll_all(BusAssertions {
                        interrupt_assertion: if i % 2 == 0 { 16 } else { 0 },
                        ..out
                    });
                    assert!(out.reset_devices_on_bus);
                    assert!(!out.bus_access_strobe);
                    assert_eq!(
                        get_cpu(&mut bus).eu_registers.pending_hardware_exceptions,
                        0
                    );
                }
                out = bus.poll_all(BusAssertions {
                    interrupt_assertion: 16,
                    ..out
                });
                assert!(out.bus_access_strobe);
                assert_eq!(out.address, 0);
                if failed_word == 1 {
                    out = bus.poll_all(BusAssertions {
                        interrupt_assertion: 16,
                        ..out
                    });
                }
                assert_eq!(out.address, failed_word);
                let waiting_phase = get_cpu(&mut bus).phase;
                for _ in 0..3 {
                    out = bus.poll_all(BusAssertions {
                        bus_acknowledge: false,
                        interrupt_assertion: 16,
                        ..out
                    });
                    assert!(out.bus_access_strobe);
                    assert_eq!(out.address, failed_word);
                    assert_eq!(get_cpu(&mut bus).phase, waiting_phase);
                    assert_eq!(
                        get_cpu(&mut bus).eu_registers.pending_hardware_exceptions,
                        0
                    );
                }
                out = bus.poll_all(BusAssertions {
                    bus_error: !protection_error,
                    bus_protection_error: protection_error,
                    bus_acknowledge: true,
                    interrupt_assertion: 16,
                    ..out
                });
                assert!(out.reset_devices_on_bus);
                assert!(!out.bus_access_strobe);
                let cpu = get_cpu(&mut bus);
                assert_eq!(cpu.phase, 0);
                assert_eq!(cpu.registers.sr, 0);
                assert_eq!(cpu.eu_registers.current_exception_level, 0);
                assert!(cpu.eu_registers.pending_fault.is_none());
                assert_eq!(cpu.eu_registers.pending_hardware_exceptions, 0);
                assert_eq!(cpu.eu_registers.link_registers[5].return_address, 0x5566);
            }
            // Final attempt succeeds. Pulses throughout all six fetch-slot
            // phases are excluded, including after PC loads in phase 3.
            for _ in 0..5 {
                out = bus.poll_all(out);
                assert!(out.reset_devices_on_bus);
            }
            for phase in 0..6 {
                out = bus.poll_all(BusAssertions {
                    interrupt_assertion: if phase % 2 == 0 { 16 } else { 0 },
                    ..out
                });
                assert!(!out.reset_devices_on_bus);
                assert_eq!(
                    get_cpu(&mut bus).eu_registers.pending_hardware_exceptions,
                    0
                );
                assert_eq!(get_cpu(&mut bus).registers.sr, 0);
            }
            assert_eq!(get_cpu(&mut bus).registers.get_full_pc_address(), 0x0800);
            assert_eq!(get_cpu(&mut bus).registers.r1, 0xcafe);
            assert_eq!(
                get_cpu(&mut bus).eu_registers.link_registers[5].return_address,
                0x5566
            );
            let next = bus.poll_all(BusAssertions {
                interrupt_assertion: 0,
                ..out
            });
            assert!(next.bus_access_strobe);
            assert_eq!(next.address, 0x0800);
        }
    }
}
