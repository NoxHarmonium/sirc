#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    // I don't like this rule
    clippy::module_name_repetitions,
    // Not sure what this is, will have to revisit
    clippy::must_use_candidate,
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

#[cfg(test)]
#[ctor::ctor]
fn init() {
    stderrlog::new()
        .module(module_path!())
        // TODO: Find a way to populate the `stderrlog` module list automatically
        // category=refactor
        // Is there a way to get this from the dependency list?
        // - See also in sirc_vm main function
        .modules(vec![
            "device_debug",
            "device_ram",
            "device_terminal",
            "device_video",
            "peripheral_bus",
            "peripheral_cpu",
        ])
        .quiet(false)
        .verbosity(log::Level::Info)
        .timestamp(stderrlog::Timestamp::Millisecond)
        .show_module_names(true)
        .init()
        .unwrap();
}

extern crate quickcheck;
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod exceptions;
mod instructions;
