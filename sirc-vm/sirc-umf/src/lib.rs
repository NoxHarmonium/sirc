//! SIRCAS (SIRC Audio System) behavioural reference model, shared by the VM audio device and the
//! SIRCUMF tools (ERENCE).
//!
//! The model is clocked at `CLKI` resolution so that memory slot scheduling, deadlines and DSP cycle
//! budgets can be measured. It is not RTL. See `docs/audio/STATUS.md`.

pub mod compile;
pub mod core;
pub mod filter;
pub mod i2s;
pub mod song;
pub mod testkit;
pub mod wav;

pub use crate::core::{CpuReq, Policy, Sircas};

pub const NUM_CHANNELS: usize = 6;
/// CLKI cycles per output sample.
pub const CLKI_PER_FRAME: u32 = 336;
pub const SCHEDULE_CLOCKS: u32 = 8;
pub const SLOT_CLOCKS: u32 = 2;
pub const AUDIO_SLOTS_PER_FRAME: u32 = CLKI_PER_FRAME / SCHEDULE_CLOCKS;
pub const ENVELOPE_PERIOD: u32 = 64;
pub const RAM_WORDS: usize = 0x8000;
pub const RAM_WINDOW: u16 = 0x8000;
pub const CLKI_HZ: f64 = 10_738_636.0;
pub const OUTPUT_RATE_HZ: f64 = 21_477_272.0 / 672.0;
