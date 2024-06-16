use std::time::{Duration, Instant};

use log::{debug, info};

pub fn start_loop(vsync_frequency: f64, mut closure: impl FnMut() -> (bool, u32)) {
    let mut frame: f64 = 1f64;
    #[allow(clippy::cast_precision_loss)]
    let seconds_per_frame = Duration::from_secs(1).div_f64(vsync_frequency);

    let mut interval = spin_sleep_util::interval(seconds_per_frame);
    let mut reporter = spin_sleep_util::RateReporter::new(Duration::from_secs(5));

    let start_instant = Instant::now();
    loop {
        let (abort, _) = closure();

        if abort {
            let elapsed = start_instant.elapsed().as_secs_f64();
            let expected_frame = elapsed / seconds_per_frame.as_secs_f64();

            let run_rate = frame / expected_frame;
            info!("Exiting main loop. Actual Duration: {elapsed}s Expected frame: {expected_frame} Actual Frame: {frame} Seconds per frame: {} Run rate: {run_rate}",seconds_per_frame.as_secs_f64());
            break;
        }

        if let Some(fps) = reporter.increment_and_report() {
            debug!("Frame: [{frame}] FPS: [{fps}]");
        }

        frame += 1f64;
        interval.tick();
    }
}
