//! The ERENCE song model: a tracker-style table of rows and six channel columns, plus the
//! instruments, channel settings and filter that go with it, and a plain-text file format.
//!
//! A row is a fixed number of sequencer ticks (`speed`). Each channel column holds events
//! (note on, note off, instrument and fine-pitch changes). The compiler (`crate::compile`) turns
//! each column into one SIRCAS instruction stream.

use std::fmt::Write as _;
use std::path::Path;

use crate::testkit;
use crate::wav;

pub const CHANNELS: usize = 6;
pub const MAX_INSTRUMENTS: usize = 256;
pub const MAX_NOTE: u8 = 119;
pub const MAX_ROWS: usize = 4096;
/// Fine pitch is a signed 13-bit value.
pub const FINE_MIN: i16 = -4096;
pub const FINE_MAX: i16 = 4095;

const NOTE_NAMES: [&str; 12] = [
    "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-",
];

/// Display name of a SIRCAS note number (note 0 is A0, 27.5 Hz). Example: note 39 is `C-4`.
#[must_use]
pub fn note_name(note: u8) -> String {
    let midi = u32::from(note) + 21;
    let octave = std::char::from_digit(midi / 12 - 1, 36)
        .unwrap_or('?')
        .to_ascii_uppercase();
    format!("{}{octave}", NOTE_NAMES[(midi % 12) as usize])
}

/// Parse a note name such as `C-4` or `F#2`.
#[must_use]
pub fn parse_note(s: &str) -> Option<u8> {
    let b = s.as_bytes();
    if b.len() != 3 {
        return None;
    }
    let name = s.get(0..2)?;
    let idx = NOTE_NAMES.iter().position(|n| *n == name)? as i32;
    let oct = (b[2] as char).to_digit(36)? as i32;
    let midi = (oct + 1) * 12 + idx;
    u8::try_from(midi - 21)
        .ok()
        .filter(|n| *n <= MAX_NOTE && midi >= 21)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NoteCell {
    #[default]
    None,
    On(u8),
    Off,
}

/// One cell of the pattern table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Cell {
    pub note: NoteCell,
    /// Instrument to select. It stays selected for later notes in this column.
    pub inst: Option<u8>,
    /// Fine pitch (raw `FINE` units). It stays set for later notes in this column.
    pub fine: Option<i16>,
}

impl Cell {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Fixed-width text form: `C-4 01 +0000`, with dots for an empty field and `===` for a stop.
    #[must_use]
    pub fn to_text(&self) -> String {
        let note = match self.note {
            NoteCell::None => "...".to_string(),
            NoteCell::Off => "===".to_string(),
            NoteCell::On(n) => note_name(n),
        };
        let inst = self
            .inst
            .map_or_else(|| "..".to_string(), |i| format!("{i:02X}"));
        let fine = self
            .fine
            .map_or_else(|| ".....".to_string(), |f| format!("{f:+05}"));
        format!("{note} {inst} {fine}")
    }

    /// # Errors
    /// Returns a message if a field is malformed.
    pub fn from_text(s: &str) -> Result<Self, String> {
        let mut it = s.split_whitespace();
        let (Some(n), Some(i), Some(f)) = (it.next(), it.next(), it.next()) else {
            return Err(format!("cell needs three fields: {s:?}"));
        };
        let note = match n {
            "..." => NoteCell::None,
            "===" => NoteCell::Off,
            _ => NoteCell::On(parse_note(n).ok_or_else(|| format!("bad note {n:?}"))?),
        };
        let inst = if i == ".." {
            None
        } else {
            Some(u8::from_str_radix(i, 16).map_err(|_| format!("bad instrument {i:?}"))?)
        };
        let fine = if f == "....." {
            None
        } else {
            let v: i16 = f.parse().map_err(|_| format!("bad fine value {f:?}"))?;
            if !(FINE_MIN..=FINE_MAX).contains(&v) {
                return Err(format!("fine value {v} out of range"));
            }
            Some(v)
        };
        Ok(Self { note, inst, fine })
    }
}

pub type Row = [Cell; CHANNELS];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveKind {
    Sine,
    Saw,
    Square,
    Triangle,
    Pulse,
    Noise,
}

impl WaveKind {
    pub const ALL: [Self; 6] = [
        Self::Sine,
        Self::Saw,
        Self::Square,
        Self::Triangle,
        Self::Pulse,
        Self::Noise,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sine => "sine",
            Self::Saw => "saw",
            Self::Square => "square",
            Self::Triangle => "triangle",
            Self::Pulse => "pulse",
            Self::Noise => "noise",
        }
    }

    #[must_use]
    pub fn from_name(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.name() == s)
    }

    #[must_use]
    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|k| *k == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    #[must_use]
    pub fn prev(self) -> Self {
        let i = Self::ALL.iter().position(|k| *k == self).unwrap_or(0);
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    /// A generated single-cycle waveform, always looped.
    Wave {
        kind: WaveKind,
        len: u16,
        amp: u8,
        /// Pulse duty in percent.
        duty: u8,
    },
    /// An imported sample. `data` is already 8-bit and decimated; `rate` is its effective rate.
    Sample {
        path: String,
        /// Note at which the sample plays at its recorded pitch.
        root: u8,
        decimate: u8,
        data: Vec<i8>,
        rate: f64,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Instrument {
    pub name: String,
    pub source: Source,
    pub loop_en: bool,
    pub loop_start: u16,
    /// Attack, decay, sustain level, release.
    pub adsr: [u8; 4],
    /// Divides the playback pitch: 5 makes note A0 sound at 5.5 Hz instead of 27.5 Hz, for
    /// low-frequency modulation sources. 1 is normal.
    pub div: u8,
}

impl Instrument {
    #[must_use]
    pub fn wave(name: &str, kind: WaveKind) -> Self {
        Self {
            name: name.to_string(),
            source: Source::Wave {
                kind,
                len: 64,
                amp: 100,
                duty: 50,
            },
            loop_en: true,
            loop_start: 0,
            adsr: [0, 4, 200, 8],
            div: 1,
        }
    }

    /// The bytes written to audio memory.
    #[must_use]
    pub fn pcm(&self) -> Vec<i8> {
        match &self.source {
            Source::Wave {
                kind,
                len,
                amp,
                duty,
            } => {
                let (n, a) = (usize::from(*len), f64::from(*amp));
                match kind {
                    WaveKind::Sine => testkit::sine(n, a),
                    WaveKind::Saw => {
                        // Start at the zero crossing so a retrigger does not begin with a step.
                        let mut v = testkit::saw(n, a);
                        v.rotate_left(n / 2);
                        v
                    }
                    WaveKind::Square => testkit::square(n, a),
                    WaveKind::Triangle => testkit::triangle_from_zero(n, a),
                    WaveKind::Pulse => testkit::pulse(n, f64::from(*duty) / 100.0, a),
                    WaveKind::Noise => testkit::noise(n, 0x1234_5678, a),
                }
            }
            Source::Sample { data, .. } => data.clone(),
        }
    }

    /// Bytes per cycle at the instrument's root pitch, which fixes its pitch table.
    #[must_use]
    pub fn cycle_bytes(&self) -> f64 {
        match &self.source {
            Source::Wave { len, .. } => f64::from(*len) / f64::from(self.div.max(1)),
            Source::Sample { root, rate, .. } => {
                rate / testkit::note_freq(u32::from(*root)) / f64::from(self.div.max(1))
            }
        }
    }

    #[must_use]
    pub fn loops(&self) -> bool {
        matches!(self.source, Source::Wave { .. }) || self.loop_en
    }

    /// Load an imported sample from `path` (relative paths resolve against `base`).
    ///
    /// # Errors
    /// Returns a message if the file cannot be read.
    pub fn load_sample(
        name: &str,
        path: &str,
        base: Option<&Path>,
        root: u8,
        decimate: u8,
    ) -> Result<Self, String> {
        let mut inst = Self {
            name: name.to_string(),
            source: Source::Sample {
                path: path.to_string(),
                root,
                decimate,
                data: Vec::new(),
                rate: 8000.0,
            },
            loop_en: false,
            loop_start: 0,
            adsr: [0, 0, 255, 16],
            div: 1,
        };
        inst.reload_sample(base)?;
        Ok(inst)
    }

    /// Re-read the sample file and re-apply decimation.
    ///
    /// # Errors
    /// Returns a message if this is not a sample or the file cannot be read.
    pub fn reload_sample(&mut self, base: Option<&Path>) -> Result<(), String> {
        let Source::Sample {
            path,
            decimate,
            data,
            rate,
            ..
        } = &mut self.source
        else {
            return Err("not a sample instrument".into());
        };
        let p = Path::new(path.as_str());
        let full = match base {
            Some(b) if p.is_relative() => b.join(p),
            _ => p.to_path_buf(),
        };
        let w = wav::read_wav(&full)?;
        let (d, r) = wav::to_pcm8(&w, u32::from(*decimate));
        *data = d;
        *rate = r;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModRoute {
    /// A `moddest` code.
    pub dest: u8,
    /// Voice (or unused) the destination applies to.
    pub target: u8,
    pub shift: i8,
    pub invert: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChannelCfg {
    pub volume: u8,
    pub pan: u8,
    /// Route this voice through the filter.
    pub filter: bool,
    /// Portamento rate in raw units per 2 ms, 0 for off.
    pub porta: u16,
    /// `Some` makes the voice a modulation source instead of an audio voice.
    pub modulator: Option<ModRoute>,
}

impl Default for ChannelCfg {
    fn default() -> Self {
        Self {
            volume: 160,
            pan: 128,
            filter: false,
            porta: 0,
            modulator: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilterCfg {
    /// 0 low-pass, 1 band-pass, 2 high-pass.
    pub mode: u8,
    pub cutoff: u8,
    pub resonance: u8,
    pub volume: u8,
    pub pan: u8,
}

impl Default for FilterCfg {
    fn default() -> Self {
        Self {
            mode: 0,
            cutoff: 120,
            resonance: 64,
            volume: 160,
            pan: 128,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Song {
    pub title: String,
    /// `SEQ_PERIOD`: output samples per sequencer tick.
    pub tick: u16,
    /// Ticks per row.
    pub speed: u16,
    pub master_vol: u8,
    pub filter: FilterCfg,
    pub channels: [ChannelCfg; CHANNELS],
    pub instruments: Vec<Instrument>,
    pub rows: Vec<Row>,
    /// Row to jump back to at the end, or `None` to stop.
    pub loop_row: Option<usize>,
}

impl Default for Song {
    fn default() -> Self {
        Self {
            title: "Untitled".into(),
            tick: 400,
            speed: 6,
            master_vol: 224,
            filter: FilterCfg::default(),
            channels: [ChannelCfg::default(); CHANNELS],
            instruments: vec![
                Instrument::wave("Sine", WaveKind::Sine),
                Instrument::wave("Saw", WaveKind::Saw),
                Instrument::wave("Square", WaveKind::Square),
            ],
            rows: vec![[Cell::default(); CHANNELS]; 32],
            loop_row: Some(0),
        }
    }
}

impl Song {
    /// Milliseconds per tick.
    #[must_use]
    pub fn tick_ms(&self) -> f64 {
        f64::from(self.tick) * 1000.0 / crate::OUTPUT_RATE_HZ
    }

    /// Beats per minute if four rows make a beat.
    #[must_use]
    pub fn bpm(&self) -> f64 {
        60_000.0 / (self.tick_ms() * f64::from(self.speed) * 4.0)
    }

    /// Output samples in one pass through the table.
    #[must_use]
    pub fn pass_frames(&self) -> u64 {
        self.rows.len() as u64 * u64::from(self.speed) * u64::from(self.tick)
    }

    // -----------------------------------------------------------------------------------------
    // Text format
    // -----------------------------------------------------------------------------------------

    /// Serialise to the `.erence` text format.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "ERENCE 1");
        let _ = writeln!(s, "title {}", self.title);
        let _ = writeln!(s, "tick {}", self.tick);
        let _ = writeln!(s, "speed {}", self.speed);
        let _ = writeln!(s, "master {}", self.master_vol);
        let f = &self.filter;
        let _ = writeln!(
            s,
            "filter mode={} cutoff={} res={} vol={} pan={}",
            f.mode, f.cutoff, f.resonance, f.volume, f.pan
        );
        match self.loop_row {
            Some(r) => {
                let _ = writeln!(s, "loop {r}");
            }
            None => {
                let _ = writeln!(s, "loop none");
            }
        }
        for (i, c) in self.channels.iter().enumerate() {
            let _ = write!(
                s,
                "channel {i} vol={} pan={} filter={} porta={}",
                c.volume,
                c.pan,
                u8::from(c.filter),
                c.porta
            );
            if let Some(m) = c.modulator {
                let _ = write!(
                    s,
                    " mod={},{},{},{}",
                    m.dest,
                    m.target,
                    m.shift,
                    u8::from(m.invert)
                );
            }
            let _ = writeln!(s);
        }
        for (i, inst) in self.instruments.iter().enumerate() {
            let _ = writeln!(s, "instname {i} {}", inst.name);
            let adsr = inst.adsr;
            let common = format!(
                "loop={} loopstart={} div={} adsr={},{},{},{}",
                u8::from(inst.loop_en),
                inst.loop_start,
                inst.div,
                adsr[0],
                adsr[1],
                adsr[2],
                adsr[3]
            );
            match &inst.source {
                Source::Wave {
                    kind,
                    len,
                    amp,
                    duty,
                } => {
                    let _ = writeln!(
                        s,
                        "inst {i} kind=wave wave={} len={len} amp={amp} duty={duty} {common}",
                        kind.name()
                    );
                }
                Source::Sample {
                    path,
                    root,
                    decimate,
                    ..
                } => {
                    let _ = writeln!(
                        s,
                        "inst {i} kind=sample root={root} decimate={decimate} {common} path={path}"
                    );
                }
            }
        }
        let _ = writeln!(s, "rows {}", self.rows.len());
        for row in &self.rows {
            let line: Vec<String> = row.iter().map(Cell::to_text).collect();
            let _ = writeln!(s, "{}", line.join(" | "));
        }
        s
    }

    /// Parse the `.erence` text format. Imported samples are read relative to `base`; samples
    /// that cannot be read are left empty and reported in the returned warnings.
    ///
    /// # Errors
    /// Returns a message with the line number for a malformed file.
    pub fn from_text(text: &str, base: Option<&Path>) -> Result<(Self, Vec<String>), String> {
        let mut lines = text.lines().enumerate();
        match lines.next() {
            Some((_, l)) if l.trim() == "ERENCE 1" => {}
            _ => return Err("not an ERENCE 1 file".into()),
        }
        let mut song = Self {
            instruments: Vec::new(),
            rows: Vec::new(),
            ..Self::default()
        };
        let mut warnings = Vec::new();
        let mut names: Vec<(usize, String)> = Vec::new();
        let mut expect_rows: Option<usize> = None;
        for (n, line) in lines {
            let err = |m: String| format!("line {}: {m}", n + 1);
            if let Some(count) = expect_rows {
                if line.trim().is_empty() && song.rows.len() == count {
                    continue;
                }
                let cells: Vec<&str> = line.split('|').collect();
                if cells.len() != CHANNELS {
                    return Err(err(format!("row needs {CHANNELS} cells")));
                }
                let mut row = [Cell::default(); CHANNELS];
                for (c, t) in cells.iter().enumerate() {
                    row[c] = Cell::from_text(t).map_err(&err)?;
                }
                song.rows.push(row);
                continue;
            }
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
            match key {
                "title" => song.title = rest.to_string(),
                "tick" => song.tick = rest.parse().map_err(|_| err("bad tick".into()))?,
                "speed" => song.speed = rest.parse().map_err(|_| err("bad speed".into()))?,
                "master" => song.master_vol = rest.parse().map_err(|_| err("bad master".into()))?,
                "loop" => {
                    song.loop_row = if rest == "none" {
                        None
                    } else {
                        Some(rest.parse().map_err(|_| err("bad loop".into()))?)
                    }
                }
                "filter" => {
                    let kv = KeyValues::new(rest);
                    song.filter = FilterCfg {
                        mode: kv.num("mode").map_err(&err)?,
                        cutoff: kv.num("cutoff").map_err(&err)?,
                        resonance: kv.num("res").map_err(&err)?,
                        volume: kv.num("vol").map_err(&err)?,
                        pan: kv.num("pan").map_err(&err)?,
                    };
                }
                "channel" => {
                    let (idx, rest) = rest.split_once(' ').unwrap_or((rest, ""));
                    let idx: usize = idx.parse().map_err(|_| err("bad channel".into()))?;
                    if idx >= CHANNELS {
                        return Err(err("channel out of range".into()));
                    }
                    let kv = KeyValues::new(rest);
                    let modulator = match kv.get("mod") {
                        None => None,
                        Some(v) => {
                            let p: Vec<&str> = v.split(',').collect();
                            if p.len() != 4 {
                                return Err(err("mod needs dest,target,shift,invert".into()));
                            }
                            let bad = || err("bad mod".into());
                            Some(ModRoute {
                                dest: p[0].parse().map_err(|_| bad())?,
                                target: p[1].parse().map_err(|_| bad())?,
                                shift: p[2].parse().map_err(|_| bad())?,
                                invert: p[3] == "1",
                            })
                        }
                    };
                    song.channels[idx] = ChannelCfg {
                        volume: kv.num("vol").map_err(&err)?,
                        pan: kv.num("pan").map_err(&err)?,
                        filter: kv.num::<u8>("filter").map_err(&err)? != 0,
                        porta: kv.num("porta").map_err(&err)?,
                        modulator,
                    };
                }
                "instname" => {
                    let (idx, name) = rest.split_once(' ').unwrap_or((rest, ""));
                    let idx: usize = idx.parse().map_err(|_| err("bad instname".into()))?;
                    names.push((idx, name.to_string()));
                }
                "inst" => {
                    let (idx, rest) = rest.split_once(' ').unwrap_or((rest, ""));
                    let idx: usize = idx.parse().map_err(|_| err("bad inst".into()))?;
                    if idx != song.instruments.len() || idx >= MAX_INSTRUMENTS {
                        return Err(err("instruments must be numbered in order".into()));
                    }
                    let kv = KeyValues::new(rest);
                    let adsr: Vec<u8> = kv
                        .get("adsr")
                        .ok_or_else(|| err("missing adsr".into()))?
                        .split(',')
                        .map(|v| v.parse().map_err(|_| err("bad adsr".into())))
                        .collect::<Result<_, _>>()?;
                    if adsr.len() != 4 {
                        return Err(err("adsr needs four values".into()));
                    }
                    let adsr = [adsr[0], adsr[1], adsr[2], adsr[3]];
                    let loop_en = kv.num::<u8>("loop").map_err(&err)? != 0;
                    let loop_start = kv.num("loopstart").map_err(&err)?;
                    let source = match kv.get("kind") {
                        Some("wave") => Source::Wave {
                            kind: kv
                                .get("wave")
                                .and_then(WaveKind::from_name)
                                .ok_or_else(|| err("bad wave".into()))?,
                            len: kv.num("len").map_err(&err)?,
                            amp: kv.num("amp").map_err(&err)?,
                            duty: kv.num("duty").map_err(&err)?,
                        },
                        Some("sample") => {
                            let path = rest
                                .split_once("path=")
                                .map(|(_, p)| p.to_string())
                                .ok_or_else(|| err("missing path".into()))?;
                            Source::Sample {
                                path,
                                root: kv.num("root").map_err(&err)?,
                                decimate: kv.num("decimate").map_err(&err)?,
                                data: Vec::new(),
                                rate: 8000.0,
                            }
                        }
                        _ => return Err(err("kind must be wave or sample".into())),
                    };
                    let mut inst = Instrument {
                        name: format!("Instrument {idx}"),
                        source,
                        loop_en,
                        loop_start,
                        adsr,
                        div: kv
                            .get("div")
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(1)
                            .max(1),
                    };
                    if matches!(inst.source, Source::Sample { .. }) {
                        if let Err(e) = inst.reload_sample(base) {
                            warnings.push(format!("instrument {idx}: {e}"));
                        }
                    }
                    song.instruments.push(inst);
                }
                "rows" => {
                    let count: usize = rest.parse().map_err(|_| err("bad row count".into()))?;
                    if count == 0 || count > MAX_ROWS {
                        return Err(err("row count out of range".into()));
                    }
                    expect_rows = Some(count);
                }
                _ => return Err(err(format!("unknown directive {key:?}"))),
            }
        }
        match expect_rows {
            Some(count) if song.rows.len() == count => {}
            Some(count) => {
                return Err(format!("expected {count} rows, found {}", song.rows.len()));
            }
            None => return Err("no rows section".into()),
        }
        for (i, name) in names {
            if let Some(inst) = song.instruments.get_mut(i) {
                inst.name = name;
            }
        }
        Ok((song, warnings))
    }
}

/// `key=value` pairs separated by spaces.
struct KeyValues<'a> {
    pairs: Vec<(&'a str, &'a str)>,
}

impl<'a> KeyValues<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            pairs: s
                .split_whitespace()
                .filter_map(|p| p.split_once('='))
                .collect(),
        }
    }

    fn get(&self, key: &str) -> Option<&'a str> {
        self.pairs.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }

    fn num<T: std::str::FromStr>(&self, key: &str) -> Result<T, String> {
        self.get(key)
            .ok_or_else(|| format!("missing {key}"))?
            .parse()
            .map_err(|_| format!("bad {key}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_names_round_trip() {
        assert_eq!(note_name(0), "A-0");
        assert_eq!(note_name(39), "C-4");
        assert_eq!(parse_note("C-4"), Some(39));
        assert_eq!(parse_note("A-0"), Some(0));
        assert_eq!(parse_note("G#9"), Some(107));
        assert_eq!(parse_note("H-4"), None);
        for n in 0..=MAX_NOTE {
            assert_eq!(parse_note(&note_name(n)), Some(n), "note {n}");
        }
    }

    #[test]
    fn cell_text_round_trips() {
        let cells = [
            Cell::default(),
            Cell {
                note: NoteCell::On(39),
                inst: Some(0x1F),
                fine: Some(-12),
            },
            Cell {
                note: NoteCell::Off,
                inst: None,
                fine: Some(4095),
            },
        ];
        for c in cells {
            assert_eq!(Cell::from_text(&c.to_text()).unwrap(), c);
        }
        assert_eq!(Cell::default().to_text(), "... .. .....");
    }

    #[test]
    fn song_text_round_trips() {
        let mut s = Song {
            title: "A tune".into(),
            ..Song::default()
        };
        s.rows[0][0] = Cell {
            note: NoteCell::On(39),
            inst: Some(1),
            fine: None,
        };
        s.rows[4][2].note = NoteCell::Off;
        s.loop_row = Some(2);
        s.channels[1].porta = 640;
        s.channels[5].modulator = Some(ModRoute {
            dest: 1,
            target: 1,
            shift: -2,
            invert: false,
        });
        let text = s.to_text();
        let (back, warnings) = Song::from_text(&text, None).unwrap();
        assert!(warnings.is_empty());
        assert_eq!(back, s);
    }

    #[test]
    fn rejects_bad_files() {
        assert!(Song::from_text("nope", None).is_err());
        assert!(Song::from_text("ERENCE 1\nrows 2\n", None).is_err());
    }
}
