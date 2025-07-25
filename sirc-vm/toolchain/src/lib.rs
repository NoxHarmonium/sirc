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
// TODO: mismatched_lifetime_syntaxes is a new lint that is in nightly rust and causes tons of errors
// need to fix it properly when I have time
#![allow(unknown_lints)] // The lint is unknown to stable rust, so suppress the warning
#![allow(mismatched_lifetime_syntaxes)]
#![deny(unknown_lints)]
#![deny(warnings)]

pub mod data;
pub mod parsers;
pub mod printers;
pub mod types;
