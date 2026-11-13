//! Minimal WAV reader for sample import: PCM 8, 16, 24 and 32 bit, float 32 bit, any channel count
//! (mixed down to mono).

/// A decoded WAV file: mono samples in -1.0..=1.0 and the sample rate.
#[derive(Clone, Debug)]
pub struct Wav {
    pub rate: f64,
    pub samples: Vec<f32>,
}

fn u16_le(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}

fn u32_le(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// Decode a WAV file image.
///
/// # Errors
/// Returns a message if the data is not a supported WAV file.
pub fn parse_wav(bytes: &[u8]) -> Result<Wav, String> {
    if bytes.get(0..4) != Some(b"RIFF") || bytes.get(8..12) != Some(b"WAVE") {
        return Err("not a RIFF/WAVE file".into());
    }
    let mut pos = 12;
    let mut fmt: Option<(u16, u16, u32, u16)> = None; // tag, channels, rate, bits
    let mut data: Option<&[u8]> = None;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let len = u32_le(bytes, pos + 4).ok_or("truncated chunk")? as usize;
        let body = pos + 8;
        let end = (body + len).min(bytes.len());
        match id {
            b"fmt " => {
                let tag = u16_le(bytes, body).ok_or("short fmt chunk")?;
                let channels = u16_le(bytes, body + 2).ok_or("short fmt chunk")?;
                let rate = u32_le(bytes, body + 4).ok_or("short fmt chunk")?;
                let bits = u16_le(bytes, body + 14).ok_or("short fmt chunk")?;
                // WAVE_FORMAT_EXTENSIBLE carries the real tag in the sub-format.
                let tag = if tag == 0xFFFE {
                    u16_le(bytes, body + 24).unwrap_or(tag)
                } else {
                    tag
                };
                fmt = Some((tag, channels, rate, bits));
            }
            b"data" => data = Some(&bytes[body..end]),
            _ => {}
        }
        pos = body + len + (len & 1);
    }
    let (tag, channels, rate, bits) = fmt.ok_or("no fmt chunk")?;
    let data = data.ok_or("no data chunk")?;
    if channels == 0 {
        return Err("zero channels".into());
    }
    let bytes_per = usize::from(bits / 8);
    if bytes_per == 0 {
        return Err("unsupported bit depth".into());
    }
    let decode = |chunk: &[u8]| -> Result<f32, String> {
        Ok(match (tag, bits) {
            (1, 8) => (f32::from(chunk[0]) - 128.0) / 128.0,
            (1, 16) => f32::from(i16::from_le_bytes([chunk[0], chunk[1]])) / 32768.0,
            (1, 24) => {
                let v = i32::from_le_bytes([0, chunk[0], chunk[1], chunk[2]]) >> 8;
                v as f32 / 8_388_608.0
            }
            (1, 32) => {
                i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f32
                    / 2_147_483_648.0
            }
            (3, 32) => f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
            _ => return Err(format!("unsupported WAV format {tag} / {bits} bit")),
        })
    };
    let frame = bytes_per * usize::from(channels);
    let mut samples = Vec::with_capacity(data.len() / frame);
    for f in data.chunks_exact(frame) {
        let mut acc = 0.0;
        for c in f.chunks_exact(bytes_per) {
            acc += decode(c)?;
        }
        samples.push(acc / f32::from(channels));
    }
    Ok(Wav {
        rate: f64::from(rate),
        samples,
    })
}

/// Read and decode a WAV file.
///
/// # Errors
/// Returns a message for I/O errors and unsupported files.
pub fn read_wav(path: &std::path::Path) -> Result<Wav, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_wav(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}

/// Convert decoded audio to the 8-bit signed bytes SIRCAS plays: average every `decimate`
/// source samples (a crude low-pass), then scale so the peak is 127. Returns the bytes and the
/// effective sample rate.
#[must_use]
pub fn to_pcm8(wav: &Wav, decimate: u32) -> (Vec<i8>, f64) {
    let d = decimate.max(1) as usize;
    let mixed: Vec<f32> = wav
        .samples
        .chunks(d)
        .map(|c| c.iter().sum::<f32>() / c.len() as f32)
        .collect();
    let peak = mixed.iter().fold(0f32, |m, v| m.max(v.abs()));
    let gain = if peak > 0.0 { 127.0 / peak } else { 0.0 };
    let data = mixed
        .iter()
        .map(|v| (v * gain).round().clamp(-128.0, 127.0) as i8)
        .collect();
    (data, wav.rate / d as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav16(rate: u32, channels: u16, samples: &[i16]) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend(b"RIFF");
        v.extend((36 + samples.len() as u32 * 2).to_le_bytes());
        v.extend(b"WAVEfmt ");
        v.extend(16u32.to_le_bytes());
        v.extend(1u16.to_le_bytes());
        v.extend(channels.to_le_bytes());
        v.extend(rate.to_le_bytes());
        v.extend((rate * u32::from(channels) * 2).to_le_bytes());
        v.extend((channels * 2).to_le_bytes());
        v.extend(16u16.to_le_bytes());
        v.extend(b"data");
        v.extend((samples.len() as u32 * 2).to_le_bytes());
        for s in samples {
            v.extend(s.to_le_bytes());
        }
        v
    }

    #[test]
    fn reads_16_bit_mono() {
        let w = parse_wav(&wav16(8000, 1, &[0, 16384, -16384])).unwrap();
        assert_eq!(w.rate, 8000.0);
        assert_eq!(w.samples, vec![0.0, 0.5, -0.5]);
    }

    #[test]
    fn stereo_is_mixed_down() {
        let w = parse_wav(&wav16(8000, 2, &[16384, -16384, 8192, 8192])).unwrap();
        assert_eq!(w.samples, vec![0.0, 0.25]);
    }

    #[test]
    fn pcm8_normalises_and_decimates() {
        let w = Wav {
            rate: 8000.0,
            samples: vec![0.25, 0.25, -0.5, -0.5],
        };
        let (d, rate) = to_pcm8(&w, 2);
        assert_eq!(rate, 4000.0);
        assert_eq!(d, vec![64, -127]);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_wav(b"hello").is_err());
    }
}
