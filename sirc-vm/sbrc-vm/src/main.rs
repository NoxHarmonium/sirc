#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    // I don't like this rule
    clippy::module_name_repetitions,
    // Will tackle this at the next clean up
    clippy::too_many_lines,
    // Might be good practice but too much work for now
    clippy::missing_errors_doc,
    // Not stable yet - try again later
    clippy::missing_const_for_fn,
    // I have a lot of temporary panics for debugging that will probably be cleaned up
    clippy::missing_panics_doc
)]
#![deny(warnings)]

use std::{fs::File, io::Write, path::PathBuf, process::exit};

use device_debug::new_debug_device;
use log::{error, info, Level};

use device_ram::{new_ram_device_file_mapped, new_ram_device_standard};
use device_terminal::new_terminal_device;
use peripheral_bus::{device::Device, new_bus_peripheral};
use peripheral_clock::ClockPeripheral;
use peripheral_cpu::{new_cpu_peripheral, CpuPeripheral};

static PROGRAM_SEGMENT: &str = "PROGRAM";
static TERMINAL_SEGMENT: &str = "TERMINAL";
static DEBUG_SEGMENT: &str = "DEBUG";

use clap::Parser;

fn segment_arg_parser(s: &str) -> Result<SegmentArg, String> {
    let segment_args: Vec<_> = s.split(':').collect();
    if segment_args.len() < 3 || segment_args.len() > 4 {
        return Err(format!(
            "Incorrect format for segment args [${s}] . Should in the format <label>:<offset>:<length>:<optional_mapped_file>:<writable>.",
        ));
    }
    match segment_args.as_slice() {
        [label, offset_str, length_str] => {
            // TODO: Surely there is a cleaner idomatic way to do this
            let offset = match u32::from_str_radix(offset_str, 16) {
                Ok(x) => x,
                Err(error) => return Err(error.to_string()),
            };
            let length = match u32::from_str_radix(length_str, 16) {
                Ok(x) => x,
                Err(error) => return Err(error.to_string()),
            };

            Ok(SegmentArg {
                label: (*label).to_string(),
                offset,
                length,
                mapped_file: None,
                writeable: true,
            })
        }
        [label, offset_str, length_str, file] => {
            // TODO: Surely there is a cleaner idomatic way to do this
            let offset = match u32::from_str_radix(offset_str, 16) {
                Ok(x) => x,
                Err(error) => return Err(error.to_string()),
            };
            let length = match u32::from_str_radix(length_str, 16) {
                Ok(x) => x,
                Err(error) => return Err(error.to_string()),
            };

            Ok(SegmentArg {
                label: (*label).to_string(),
                offset,
                length,
                mapped_file: Some(PathBuf::from(file)),
                writeable: true,
            })
        }
        _ => Err("Error".to_string()),
    }
}

#[derive(Clone, Debug)]
struct SegmentArg {
    pub label: String,
    pub offset: u32,
    pub length: u32,
    pub mapped_file: Option<PathBuf>,
    pub writeable: bool,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser, value_name = "FILE")]
    program_file: PathBuf,

    #[clap(short, long, value_parser = segment_arg_parser)]
    segment: Vec<SegmentArg>,

    #[clap(short, long, value_parser, value_name = "FILE")]
    register_dump_file: Option<PathBuf>,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

#[allow(clippy::borrowed_box)]
fn dump_registers(dump_file: &PathBuf, device: &dyn Device) -> Result<(), std::io::Error> {
    let mut handle = File::create(dump_file)?;

    write!(&mut handle, "{}", device.dump_diagnostic())?;
    Ok(())
}

fn main() {
    let args = Args::parse();

    stderrlog::new()
        .module(module_path!())
        // TODO: Is there a way to get this from the dependency list?
        .modules(vec![
            "device_debug",
            "device_ram",
            "device_terminal",
            "peripheral_bus",
            "peripheral_clock",
            "peripheral_cpu",
        ])
        .quiet(args.verbose.is_silent())
        .verbosity(args.verbose.log_level().unwrap_or(Level::Error))
        .timestamp(stderrlog::Timestamp::Millisecond)
        .show_module_names(true)
        .init()
        .unwrap();

    let master_clock_freq = 8_000_000;

    let clock_peripheral = ClockPeripheral {
        master_clock_freq,
        vsync_frequency: 50,
    };
    let mut cpu_peripheral = new_cpu_peripheral(0x0);
    // Jump to reset vector
    cpu_peripheral.reset();

    let mut bus_peripheral = new_bus_peripheral(Box::new(cpu_peripheral));
    let program_ram_device = new_ram_device_standard();
    let terminal_device = new_terminal_device(master_clock_freq);
    let debug_device = new_debug_device();

    bus_peripheral.map_segment(
        TERMINAL_SEGMENT,
        0x000A_0000,
        0xF,
        true,
        Box::new(terminal_device),
    );
    bus_peripheral.map_segment(
        DEBUG_SEGMENT,
        0x000B_0000,
        0xF,
        true,
        Box::new(debug_device),
    );

    bus_peripheral.map_segment(
        PROGRAM_SEGMENT,
        0x0,
        0xFFFF,
        false,
        Box::new(program_ram_device),
    );
    bus_peripheral.load_binary_data_into_segment_from_file(PROGRAM_SEGMENT, &args.program_file);

    for segment in args.segment {
        if let Some(mapped_file) = segment.mapped_file {
            let mm_ram_device = new_ram_device_file_mapped(mapped_file);
            bus_peripheral.map_segment(
                segment.label.as_str(),
                segment.offset,
                segment.length,
                segment.writeable,
                Box::new(mm_ram_device),
            );
        } else {
            let standard_ram_device = new_ram_device_standard();
            bus_peripheral.map_segment(
                segment.label.as_str(),
                segment.offset,
                segment.length,
                segment.writeable,
                Box::new(standard_ram_device),
            );
        }
    }

    // TODO: Profile and make this actually performant (currently is ,less than 1 fps in a tight loop)
    let execute = |_| {
        let merged_assertions = bus_peripheral.poll_all();
        !merged_assertions.exit_simulation
    };

    clock_peripheral.start_loop(execute);

    if let Some(register_dump_file) = &args.register_dump_file {
        // TODO: This is terrible - again
        let cpu: &CpuPeripheral = bus_peripheral
            .bus_master
            .as_any()
            .downcast_ref::<CpuPeripheral>()
            .expect("failed to downcast");

        info!(
                "Register dump file argument provided. Dumping registers to [{register_dump_file:?}]..."
            );
        if let Err(error) = dump_registers(register_dump_file, cpu) {
            error!(
                "There was an error dumping registers to [{}].\n{}",
                register_dump_file.display(),
                error
            );
        };
    }

    info!("Processor asserted simulation aborted (e.g. COP 0x14FF). This type of error exits with code zero for testing purposes.");
    exit(0);
}

// TODO: Probs need to a step debugger
