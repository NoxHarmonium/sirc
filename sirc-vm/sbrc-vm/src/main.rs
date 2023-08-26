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

use std::{path::PathBuf, process::exit};

use peripheral_clock::ClockPeripheral;
use peripheral_cpu::new_cpu_peripheral;
use peripheral_mem::new_memory_peripheral;

static PROGRAM_SEGMENT: &str = "PROGRAM";

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
}

fn main() {
    let args = Args::parse();

    let clock_peripheral = ClockPeripheral {
        master_clock_freq: 25_000_000,
        cpu_divider: 6,
        hsync_divider: 1600,
    };
    let mut memory_peripheral = new_memory_peripheral();

    memory_peripheral.map_segment(PROGRAM_SEGMENT, 0x0, 0xFFFF, false);
    memory_peripheral.load_binary_data_into_segment_from_file(PROGRAM_SEGMENT, &args.program_file);

    for segment in args.segment {
        match segment.mapped_file {
            Some(mapped_file) => {
                memory_peripheral.map_segment_to_file(
                    segment.label.as_str(),
                    segment.offset,
                    segment.length,
                    segment.writeable,
                    &mapped_file,
                );
            }
            None => {
                memory_peripheral.map_segment(
                    segment.label.as_str(),
                    segment.offset,
                    segment.length,
                    segment.writeable,
                );
            }
        }
    }

    let mut cpu_peripheral = new_cpu_peripheral(&memory_peripheral, PROGRAM_SEGMENT);

    // Jump to reset vector
    cpu_peripheral.reset();

    let execute = |_delta, clock_quota| match cpu_peripheral.run_cpu(clock_quota) {
        Ok(actual_clocks_executed) => {
            println!("actual_clocks_executed: {actual_clocks_executed}");
        }
        Err(error) => match error {
            peripheral_cpu::Error::ProcessorHalted(_) => {
                println!("Processor halted error caught. This type of error will exit with code zero for testing purposes.");
                exit(0);
            }
            peripheral_cpu::Error::InvalidInstruction(_) => panic!("CPU Error: {error:08x?}"),
        },
    };

    clock_peripheral.start_loop(execute);
}

// TODO: Infinite loop but at least it assembles?
// Probs need to a step debugger
