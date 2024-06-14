use std::{cell::RefCell, fs, path::PathBuf, time::Duration};

use criterion::{criterion_group, criterion_main, Criterion, SamplingMode};
use device_ram::{new_ram_device_file_mapped, new_ram_device_standard};
use peripheral_bus::new_bus_peripheral;
use peripheral_cpu::new_cpu_peripheral;
use sbrc_vm::{run_vm, Vm};

static PROGRAM_SEGMENT: &str = "PROGRAM";
static FILE_SEGMENT: &str = "FILE";

fn setup_vm(program: &[u8], mapped_file_path: PathBuf) -> Vm {
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    // Jump to reset vector
    cpu_peripheral.reset();

    let mut bus_peripheral = new_bus_peripheral(Box::new(cpu_peripheral));
    let program_ram_device = new_ram_device_standard();

    bus_peripheral.map_segment(
        PROGRAM_SEGMENT,
        0x0,
        0xFFFF,
        false,
        Box::new(program_ram_device),
    );
    bus_peripheral.load_binary_data_into_segment(PROGRAM_SEGMENT, program);

    let mm_ram_device = new_ram_device_file_mapped(mapped_file_path);
    bus_peripheral.map_segment(
        FILE_SEGMENT,
        0x00F00000,
        0xFFFF,
        true,
        Box::new(mm_ram_device),
    );

    Vm {
        bus_peripheral: RefCell::new(bus_peripheral),
        vsync_frequency: 60f64,
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    // Originally from compiling the byte-sieve example with `make all`
    let mut binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    binary_path.push("benches/byte-sieve.bin");
    println!("program binary path: [{:?}]", binary_path);
    let program = fs::read(binary_path).unwrap();
    let mut group = c.benchmark_group("byte-sieve");
    group.sampling_mode(SamplingMode::Flat);
    group.measurement_time(Duration::from_secs(30));
    group.bench_function("byte sieve", |b| {
        b.iter(|| {
            let mut program_scratch_file = tempfile::NamedTempFile::new().unwrap();
            program_scratch_file
                .as_file_mut()
                .set_len(0xFFFF * 2)
                .unwrap();

            // TODO: Can we do the setup once and just reset the CPU on every iteration? Time to setup the test is less important than execution time
            let vm = setup_vm(
                program.as_slice(),
                program_scratch_file.into_temp_path().to_path_buf(),
            );
            run_vm(&vm, None);
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
