//! Audio output. The model renders ahead of time into a buffer; the player resamples it from the
//! chip's output rate to the sound card's rate and plays it, optionally repeating a range.

use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use sirc_umf::OUTPUT_RATE_HZ;

/// What the application needs from an audio back end.
pub trait Audio {
    /// Start playing `frames` (stereo, at the chip's rate). With `wrap`, jump from `wrap.1` back
    /// to `wrap.0` forever.
    fn play(&mut self, frames: Vec<(i16, i16)>, wrap: Option<(usize, usize)>);
    fn stop(&mut self);
    /// Current position in chip frames, or `None` when nothing is playing.
    fn position(&self) -> Option<usize>;
}

/// No sound; used when there is no output device, and by tests.
#[derive(Default)]
pub struct NullAudio {
    pub last_len: Option<usize>,
    pub playing: bool,
}

impl Audio for NullAudio {
    fn play(&mut self, frames: Vec<(i16, i16)>, _wrap: Option<(usize, usize)>) {
        self.last_len = Some(frames.len());
        self.playing = true;
    }
    fn stop(&mut self) {
        self.playing = false;
    }
    fn position(&self) -> Option<usize> {
        None
    }
}

#[derive(Default)]
struct Shared {
    frames: Vec<(i16, i16)>,
    wrap: Option<(usize, usize)>,
    /// Position in chip frames (fractional).
    pos: f64,
    playing: bool,
}

pub struct CpalAudio {
    shared: Arc<Mutex<Shared>>,
    _stream: cpal::Stream,
}

impl CpalAudio {
    /// Open the default output device.
    ///
    /// # Errors
    /// Returns a message if there is no usable output device.
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("no audio output device")?;
        let config = device
            .default_output_config()
            .map_err(|e| format!("audio config: {e}"))?;
        let shared = Arc::new(Mutex::new(Shared::default()));
        let channels = usize::from(config.channels());
        let step = OUTPUT_RATE_HZ / f64::from(config.sample_rate().0);
        let stream_config: cpal::StreamConfig = config.clone().into();
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                build::<f32>(&device, &stream_config, &shared, channels, step)
            }
            cpal::SampleFormat::I16 => {
                build::<i16>(&device, &stream_config, &shared, channels, step)
            }
            cpal::SampleFormat::U16 => {
                build::<u16>(&device, &stream_config, &shared, channels, step)
            }
            other => return Err(format!("unsupported sample format {other:?}")),
        }
        .map_err(|e| format!("audio stream: {e}"))?;
        stream.play().map_err(|e| format!("audio start: {e}"))?;
        Ok(Self {
            shared,
            _stream: stream,
        })
    }
}

fn build<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    shared: &Arc<Mutex<Shared>>,
    channels: usize,
    step: f64,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: SizedSample + FromSample<f32>,
{
    let shared = Arc::clone(shared);
    device.build_output_stream(
        config,
        move |data: &mut [T], _| {
            let Ok(mut st) = shared.lock() else {
                data.fill(T::from_sample(0.0));
                return;
            };
            for frame in data.chunks_mut(channels) {
                let (l, r) = next_sample(&mut st, step);
                for (i, out) in frame.iter_mut().enumerate() {
                    *out = T::from_sample(match i {
                        0 => l,
                        1 => r,
                        _ => 0.0,
                    });
                }
            }
        },
        |e| eprintln!("audio error: {e}"),
        None,
    )
}

/// Next output sample, linearly interpolated between chip frames.
fn next_sample(st: &mut Shared, step: f64) -> (f32, f32) {
    if !st.playing || st.frames.is_empty() {
        return (0.0, 0.0);
    }
    let i = st.pos as usize;
    if i + 1 >= st.frames.len() {
        st.playing = false;
        return (0.0, 0.0);
    }
    let frac = (st.pos - i as f64) as f32;
    let (a, b) = (st.frames[i], st.frames[i + 1]);
    let mix = |x: i16, y: i16| (f32::from(x) + (f32::from(y) - f32::from(x)) * frac) / 32768.0;
    let out = (mix(a.0, b.0), mix(a.1, b.1));
    st.pos += step;
    if let Some((start, end)) = st.wrap {
        if st.pos >= end as f64 {
            st.pos = start as f64 + (st.pos - end as f64);
        }
    }
    out
}

impl Audio for CpalAudio {
    fn play(&mut self, frames: Vec<(i16, i16)>, wrap: Option<(usize, usize)>) {
        if let Ok(mut st) = self.shared.lock() {
            st.frames = frames;
            st.wrap = wrap;
            st.pos = 0.0;
            st.playing = true;
        }
    }

    fn stop(&mut self) {
        if let Ok(mut st) = self.shared.lock() {
            st.playing = false;
        }
    }

    fn position(&self) -> Option<usize> {
        let st = self.shared.lock().ok()?;
        st.playing.then_some(st.pos as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_back_to_the_loop_start() {
        let frames: Vec<(i16, i16)> = (0..100).map(|i| (i as i16 * 100, 0)).collect();
        let mut st = Shared {
            frames,
            wrap: Some((20, 40)),
            pos: 0.0,
            playing: true,
        };
        for _ in 0..45 {
            next_sample(&mut st, 1.0);
        }
        assert!(st.playing);
        assert!((20.0..40.0).contains(&st.pos), "pos {}", st.pos);
    }

    #[test]
    fn stops_at_the_end_without_a_wrap() {
        let mut st = Shared {
            frames: vec![(0, 0); 10],
            wrap: None,
            pos: 0.0,
            playing: true,
        };
        for _ in 0..20 {
            next_sample(&mut st, 1.0);
        }
        assert!(!st.playing);
    }
}
