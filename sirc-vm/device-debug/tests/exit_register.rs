use device_debug::{new_debug_device, EXIT_SIMULATION_REGISTER, EXIT_SIMULATION_VALUE};
use peripheral_bus::{
    device::{BusAccessType, BusAssertions, BusOperation, Device},
    memory_mapped_device::MemoryMapped,
};

fn write(value: u16) -> BusAssertions {
    BusAssertions {
        address: 0x000B_0000 | EXIT_SIMULATION_REGISTER,
        data: value,
        op: BusOperation::Write,
        bus_access_strobe: true,
        bus_access_type: BusAccessType::DataWrite,
        ..BusAssertions::default()
    }
}

#[test]
fn exit_store_is_acknowledged_and_signalled_once() {
    let mut device = new_debug_device();
    let response = device.poll(write(EXIT_SIMULATION_VALUE), true);
    assert!(response.exit_simulation);
    assert!(response.bus_acknowledge);
    assert!(response.device_was_activated);
    assert_eq!(device.read_address(EXIT_SIMULATION_REGISTER), 0);
    assert!(!device.poll(BusAssertions::default(), false).exit_simulation);
}

#[test]
fn other_values_reads_and_unselected_requests_do_not_exit() {
    let mut device = new_debug_device();
    for value in [0, 2, u16::MAX] {
        let response = device.poll(write(value), true);
        assert!(response.bus_acknowledge);
        assert!(!response.exit_simulation);
    }
    assert!(
        !device
            .poll(write(EXIT_SIMULATION_VALUE), false)
            .exit_simulation
    );
    assert!(
        !device
            .poll(
                BusAssertions {
                    bus_access_strobe: false,
                    ..write(EXIT_SIMULATION_VALUE)
                },
                true
            )
            .exit_simulation
    );
    let response = device.poll(
        BusAssertions {
            op: BusOperation::Read,
            ..write(EXIT_SIMULATION_VALUE)
        },
        true,
    );
    assert!(response.bus_acknowledge);
    assert_eq!(response.data, 0);
    assert!(!response.exit_simulation);
}

#[test]
fn latched_request_survives_non_triggering_writes_until_poll() {
    let mut device = new_debug_device();
    device.write_address(EXIT_SIMULATION_REGISTER, EXIT_SIMULATION_VALUE);
    device.write_address(EXIT_SIMULATION_REGISTER, 0);
    assert_eq!(device.read_address(EXIT_SIMULATION_REGISTER), 1);
    assert!(device.poll(BusAssertions::default(), false).exit_simulation);
    assert!(!device.poll(BusAssertions::default(), false).exit_simulation);
}

#[test]
fn exit_does_not_suppress_other_pending_debug_signals() {
    for address in [0, 1, 6] {
        let mut device = new_debug_device();
        device.write_address(address, 1);
        let response = device.poll(write(EXIT_SIMULATION_VALUE), true);
        assert!(response.exit_simulation);
        assert!(response.bus_acknowledge);
        match address {
            0 => assert!(response.bus_error),
            1 => assert_eq!(response.interrupt_assertion, 1),
            6 => assert!(response.bus_protection_error),
            _ => unreachable!(),
        }
    }
}
