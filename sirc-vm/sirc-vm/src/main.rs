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
// #![deny(warnings)]

use std::{cell::RefCell, path::PathBuf, process::exit, thread};

use clap::{CommandFactory, Parser};
use log::{error, info, Level};

use sirc_vm::debug_adapter::debug_map::read_debug_map;
use sirc_vm::debug_adapter::server::{create_server_channels, start_server};
use sirc_vm::{run_vm, run_vm_debug, Vm};

use device_debug::new_debug_device;
use device_ram::{
    new_ram_device_file_mapped, new_ram_device_standard, new_ram_device_with_latency,
};
use device_terminal::new_terminal_device;
use peripheral_bus::new_bus_peripheral;
use peripheral_cpu::{
    coprocessors::dma::DmaCoprocessor, new_cpu_peripheral_with_config, CpuConfig,
};

#[cfg(feature = "video")]
use device_video::new_video_device;

static PROGRAM_SEGMENT: &str = "PROGRAM";
static TERMINAL_SEGMENT: &str = "TERMINAL";
static DEBUG_SEGMENT: &str = "DEBUG";
static VIDEO_SEGMENT: &str = "VIDEO";

fn segment_arg_parser(s: &str) -> Result<SegmentArg, String> {
    let segment_args: Vec<_> = s.split(':').collect();
    if segment_args.len() < 3 || segment_args.len() > 4 {
        return Err(format!(
            "Incorrect format for segment args [${s}] . Should in the format <label>:<offset>:<length>:<optional_mapped_file>:<writable>.",
        ));
    }
    match segment_args.as_slice() {
        [label, offset_str, length_str] => {
            // TODO: Clean up `segment_arg_parser`
            // category=Refactor
            // - Surely there is a cleaner idomatic way to do these guards
            // - Also see the match arm below
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

// These are built-in hardware implementations, selected once at VM startup.
#[derive(Clone, Debug, PartialEq, Eq)]
enum OptionalHardware {
    Dma,
    None,
}

impl std::str::FromStr for OptionalHardware {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "dma" => Ok(Self::Dma),
            "none" => Ok(Self::None),
            "math" => Err("math hardware is not implemented".into()),
            _ => Err(format!(
                "unknown optional coprocessor '{value}'; expected dma or none"
            )),
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long, value_parser, value_name = "FILE")]
    program_file: PathBuf,

    #[clap(short, long, value_parser = segment_arg_parser)]
    segment: Vec<SegmentArg>,

    #[clap(short, long, value_parser, value_name = "FILE")]
    register_dump_file: Option<PathBuf>,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[cfg(feature = "video")]
    #[clap(short, long)]
    enable_video: bool,

    #[clap(short, long)]
    debug: bool,

    /// Comma-separated optional hardware to install (dma or none); defaults to none.
    #[arg(long, value_delimiter = ',')]
    optional_coprocessors: Vec<OptionalHardware>,
}

impl Args {
    fn cpu_config(&self) -> Result<CpuConfig, String> {
        if self.optional_coprocessors.contains(&OptionalHardware::None)
            && self.optional_coprocessors.len() != 1
        {
            return Err("none must be used alone in --optional-coprocessors".into());
        }
        let mut config = CpuConfig::default();
        for hardware in &self.optional_coprocessors {
            if *hardware == OptionalHardware::Dma {
                config
                    .optional_coprocessors
                    .push(Box::new(DmaCoprocessor::default()));
            }
        }
        Ok(config)
    }
}

fn main() {
    const {
        assert!(
            !cfg!(target_endian = "big"),
            "This simulator is only compatible with little endian CPUs. \
        Which is ironic because the device it is simulating is big endian.\
        It might work, but it hasn't been tested and there is a lot of bit swizzling."
        );
    };

    let args = Args::parse();

    stderrlog::new()
        .module(module_path!())
        .modules(vec![
            "device_debug",
            "device_ram",
            "device_terminal",
            "device_video",
            "peripheral_bus",
            "peripheral_cpu",
        ])
        .quiet(args.verbose.is_silent())
        .verbosity(args.verbose.log_level().unwrap_or(Level::Error))
        .timestamp(stderrlog::Timestamp::Millisecond)
        .show_module_names(true)
        .init()
        .unwrap();

    let dump_file = args.register_dump_file.clone();
    let vm = setup_vm(&args);

    if args.debug {
        let channels = create_server_channels();

        let program_debug_info = read_debug_map(args.program_file).unwrap();
        let debugger_join_handle = thread::spawn(move || {
            let result = start_server(channels.debugger, &program_debug_info);
            if let Err(error) = result {
                error!("Error occurred in debug server: {error:?}");
            }
        });

        run_vm_debug(&vm, dump_file, channels.vm);
        info!("Waiting on debugger thread...");
        debugger_join_handle.join().unwrap();
    } else {
        run_vm(&vm, dump_file);
    }

    info!("Simulation termination requested. Exiting with code zero; example checks determine success from their output.");
    exit(0);
}

// TODO: Consider making a generic VM setup function that isn't coupled to command line args
// category=Refactoring
// Maybe make a public version of this that isn't coupled to command line argument parsing
#[must_use]
fn setup_vm(args: &Args) -> Vm {
    let master_clock_freq = 21_477_272;

    let mut cpu_peripheral = args
        .cpu_config()
        .and_then(new_cpu_peripheral_with_config)
        .unwrap_or_else(|message| {
            Args::command()
                .error(clap::error::ErrorKind::InvalidValue, message)
                .exit()
        });
    // Jump to reset vector
    cpu_peripheral.reset();

    let mut bus_peripheral = new_bus_peripheral(Box::new(cpu_peripheral));
    let program_ram_device = new_ram_device_with_latency(2);
    let terminal_device = new_terminal_device(master_clock_freq);
    let debug_device = new_debug_device();

    bus_peripheral.map_segment(
        PROGRAM_SEGMENT,
        0x0,
        0xFFFF,
        false,
        Box::new(program_ram_device),
    );
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

    #[cfg(feature = "video")]
    let vsync_frequency = if args.enable_video {
        let video_device = new_video_device(
            // TODO: Check mix of u32 and usize for the clock and video device
            // category=Refactoring
            master_clock_freq as usize,
        );
        let vsync_frequency = video_device.vsync_frequency;
        bus_peripheral.map_segment(
            VIDEO_SEGMENT,
            0x000C_0000,
            0xFFFF,
            true,
            Box::new(video_device),
        );
        vsync_frequency
    } else {
        // If there isn't a video device, there is no need to limit the VM to a specific FPS
        // but to keep things simple for now, lets default to 60 FPS
        60f64
    };

    #[cfg(not(feature = "video"))]
    let vsync_frequency = 60f64;

    bus_peripheral.load_binary_data_into_segment_from_file(PROGRAM_SEGMENT, &args.program_file);

    for segment in args.segment.clone() {
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

    Vm {
        bus_peripheral: RefCell::new(bus_peripheral),
        vsync_frequency,
    }
}

#[cfg(test)]
mod hardware_tests {
    use super::*;

    fn parse(selection: &[&str]) -> Args {
        Args::try_parse_from(
            ["sirc-vm", "--program-file", "unused.bin"]
                .into_iter()
                .chain(selection.iter().copied()),
        )
        .unwrap()
    }

    #[test]
    fn hardware_is_opt_in_and_none_is_explicitly_supported() {
        for selection in [&[][..], &["--optional-coprocessors", "none"][..]] {
            assert!(parse(selection)
                .cpu_config()
                .unwrap()
                .optional_coprocessors
                .is_empty());
        }
        let config = parse(&["--optional-coprocessors", "dma"])
            .cpu_config()
            .unwrap();
        assert_eq!(config.optional_coprocessors.len(), 1);
        assert_eq!(config.optional_coprocessors[0].id(), 2);
        assert!(new_cpu_peripheral_with_config(config).is_ok());
    }

    #[test]
    fn invalid_hardware_combinations_fail_instead_of_silently_changing_the_machine() {
        assert!(parse(&["--optional-coprocessors", "none,dma"])
            .cpu_config()
            .is_err());
        let config = parse(&["--optional-coprocessors", "dma,dma"])
            .cpu_config()
            .unwrap();
        assert!(new_cpu_peripheral_with_config(config).is_err());
        for selection in ["math", "unknown"] {
            let error = Args::try_parse_from([
                "sirc-vm",
                "--program-file",
                "unused.bin",
                "--optional-coprocessors",
                selection,
            ])
            .unwrap_err();
            assert!(error.to_string().contains(if selection == "math" {
                "not implemented"
            } else {
                "unknown optional coprocessor"
            }));
        }
    }
}
