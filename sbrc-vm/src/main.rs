use std::path::PathBuf;

use peripheral_clock::ClockPeripheral;
use peripheral_cpu::new_cpu_peripheral;
use peripheral_mem::new_memory_peripheral;

static PROGRAM_SEGMENT: &str = "PROGRAM";
static SCRATCH_SEGMENT: &str = "SCRATCH";

use clap::Parser;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, value_name = "FILE")]
    program_file: PathBuf,
}

fn main() {
    let args = Args::parse();

    let clock_peripheral = ClockPeripheral {
        master_clock_freq: 25_000_000,
        cpu_divider: 6,
        hsync_divider: 1600,
    };
    let mut memory_peripheral = new_memory_peripheral();

    memory_peripheral.map_segment(PROGRAM_SEGMENT, 0x0100, 1024, false);
    memory_peripheral.load_binary_data_into_segment(PROGRAM_SEGMENT, args.program_file);
    memory_peripheral.map_segment(SCRATCH_SEGMENT, 0xAAF0, 0x000F, true);

    let mut cpu_peripheral = new_cpu_peripheral(&memory_peripheral, PROGRAM_SEGMENT);

    let execute = |_delta, clock_quota| match cpu_peripheral.run_cpu(clock_quota) {
        Ok(actual_clocks_executed) => {
            println!("actual_clocks_executed: {}", actual_clocks_executed);
        }
        Err(error) => {
            panic!("CPU Error: {:08x?}", error);
        }
    };

    clock_peripheral.start_loop(execute)
}
