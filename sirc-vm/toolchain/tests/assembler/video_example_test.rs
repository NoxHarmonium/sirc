use super::support::assemble_object;
use peripheral_bus::{
    device::BusAssertions, memory_mapped_device::new_stub_memory_mapped_device, new_bus_peripheral,
};
use peripheral_cpu::new_cpu_peripheral;
use toolchain::{data::linker::link_object, types::object::try_merge_object_definitions};

#[test]
fn basic_video_exits_with_a_ram_stack_and_no_bus_faults() {
    const CLOCK: u32 = 21_477_272;
    let objects = [
        assemble_object(include_str!(
            "../../../../examples/basic-video/basic-video.sasm"
        )),
        assemble_object(include_str!(
            "../../../../examples/basic-video/serial-handler.sasm"
        )),
        assemble_object(include_str!("../../../../examples/basic-video/data.sasm")),
    ];
    let (object, _) = try_merge_object_definitions(&objects).unwrap();
    let program = link_object(&object, 0).unwrap();
    let mut cpu = new_cpu_peripheral();
    cpu.reset();
    let mut bus = new_bus_peripheral(Box::new(cpu));
    bus.map_segment(
        "PROGRAM",
        0,
        0xffff,
        false,
        Box::new(new_stub_memory_mapped_device()),
    );
    bus.load_binary_data_into_segment("PROGRAM", &program);
    bus.map_segment(
        "SCRATCH",
        0x10000,
        0xffff,
        true,
        Box::new(new_stub_memory_mapped_device()),
    );
    bus.map_segment(
        "TERMINAL",
        0xa0000,
        0xf,
        true,
        Box::new(device_terminal::new_terminal_device(CLOCK)),
    );
    bus.map_segment(
        "VIDEO",
        0xc0000,
        0xffff,
        true,
        Box::new(device_video::new_video_device_with_renderer(
            CLOCK as usize,
            Box::new(device_video::MockWindow {}),
        )),
    );
    let mut assertions = BusAssertions::default();
    // 120 video frames, twice the example's requested 60. A broken example
    // fails within a deterministic cycle budget rather than hanging CI.
    for _ in 0..(1364 * 262 / 2 * 120) {
        assertions = bus.poll_all(assertions);
        assert!(
            !assertions.bus_error && !assertions.bus_protection_error,
            "{assertions:?}"
        );
        if assertions.exit_simulation {
            assert!((60..=62).contains(&bus.read_address(0x10002)));
            assert_eq!(
                bus.bus_master.dump_diagnostic(),
                include_str!("../../../../examples/basic-video/basic-video.register-dump-expected")
            );
            return;
        }
    }
    panic!(
        "basic-video did not exit within 120 frames: {}",
        bus.bus_master.dump_diagnostic()
    );
}
