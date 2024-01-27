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

use std::time::{Duration, Instant};

use log::debug;

pub struct ClockPeripheral {
    pub master_clock_freq: u32, //hz
    pub vsync_frequency: u32,   //hz
}

impl ClockPeripheral {
    #[allow(clippy::cast_precision_loss)]
    pub fn start_loop(&self, mut closure: impl FnMut(u32) -> bool) {
        let vsync_frequency = 50;
        let clocks_per_vsync = self.master_clock_freq / self.vsync_frequency;
        let mut frame: u64 = 0;
        let start_instant = Instant::now();
        let seconds_per_frame = Duration::from_secs(1) / vsync_frequency;

        let mut interval = spin_sleep_util::interval(seconds_per_frame);
        let mut reporter = spin_sleep_util::RateReporter::new(Duration::from_secs(5));

        // 312.5 lines per frame
        loop {
            frame += 1;

            // TODO TODO: Test with something that takes time (bubble sort a whole segment?) (https://stackoverflow.com/a/47366256/1153203)
            for _ in 0..clocks_per_vsync {
                if !closure(clocks_per_vsync) {
                    let elapsed = start_instant.elapsed().as_secs_f64();
                    let expected_frame = elapsed / seconds_per_frame.as_secs_f64();

                    // TODO: Need to get this run_rate actually up to 1.0
                    let run_rate = frame as f64 / expected_frame;
                    debug!("Exiting main loop. Actual Duration: {elapsed}s Expected frame: {expected_frame} Actual Frame: {frame} Seconds per frame: {} Run rate: {run_rate}",seconds_per_frame.as_secs_f64());
                    return;
                }
            }

            if let Some(fps) = reporter.increment_and_report() {
                debug!("Frame: [{frame}] FPS: [{fps}]");
            }

            interval.tick();
        }
    }
}

// SNES Reference: https://wiki.superfamicom.org/timing
// Target FGPA Peripherals Reference: https://github.com/emeb/ulx3s_6502/blob/master/trellis/ulx3s_v20.lpf

// Master clock of target FGPA is 25 Mhz
// CPU divider = 6 cycles per instruction = 4.16666667 Mhz (Similar to the SNES)

// Component video rather than VGA because I want to be like a retro console
// It will probably be a lot more complicated
// See it is possible!: https://jeelabs.org/2016/11/composite-video-from-fpga/
// Encoding yc: https://github.com/MikeS11/MiSTerFPGA_YC_Encoder/blob/main/Template_MiSTer/sys/yc_out.sv
// https://hackaday.io/project/19483-tjipp8-a-chip-8-game-console/log/52072-pal-video-timings-cycle-counting
// http://martin.hinner.info/vga/pal.html

// (NTSC/PAL Ref Freqency * 2 ^ 40) / Core CLK_VIDEO
// (4.43361875 * 2 ^ 40) / 25000000 = 194992.61475003

// Start with PAL (because I live in Australia, also 50 goes into 25 easily)
// Frame rate = 50 hz
// 25Mhz / 50 hz = 500000 cycles per frame
// 25Mhz / 50 hz / 288 scan lines =

// 312 total lines (including vblank)
// 266 visible scanlines (rendered by video logic)
// 64us per line = 19.968ms per frame or 50.08Hz

// Hsync = 15625 hz (master clock / 1600)
// Vsync (interlaced) = 50hz (312.5 lines a frame)
// Vsync (not interlaced) = 50.08hz (312 lines a frame)
