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
    clippy::missing_const_for_fn
)]

use std::time::Duration;

use spin_sleep::LoopHelper;

pub struct ClockPeripheral {
    pub master_clock_freq: u32, //hz
    pub cpu_divider: u32,
    pub hsync_divider: u32,
}

impl ClockPeripheral {
    pub fn start_loop(&self, mut closure: impl FnMut(Duration, u32)) {
        let clocks_per_hsync = self.master_clock_freq / self.hsync_divider;
        // 312.5 lines per frame
        let vsync_frequency = 50;

        let mut loop_helper = LoopHelper::builder()
            .report_interval_s(5f64)
            .build_with_target_rate(vsync_frequency);

        loop {
            let delta = loop_helper.loop_start(); // or .loop_start_s() for f64 seconds

            // TODO: Instruction quota
            // TODO TODO: Test with something that takes time (bubble sort a whole segment?) (https://stackoverflow.com/a/47366256/1153203)
            closure(delta, clocks_per_hsync);

            if let Some(fps) = loop_helper.report_rate() {
                println!("HSync Per Second: {fps}");
            }

            // render_fps(current_fps);

            loop_helper.loop_sleep(); // sleeps to achieve a 250 FPS rate
        }
    }
}

// SNES Reference: https://wiki.superfamicom.org/timing
// Target FGPA Peripherals Reference: https://github.com/emeb/ulx3s_6502/blob/master/trellis/ulx3s_v20.lpf

// Master clock of target FGPA is 25 Mhz
// CPU divider = 6 = 4.16666667 Mhz (Similar to the SNES)

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
