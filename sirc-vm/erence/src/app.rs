//! ERENCE application state and key handling. Nothing here touches the terminal, so the whole
//! editor can be driven from tests.

use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sirc_umf::compile::{self, RenderOpts};
use sirc_umf::core::moddest;
use sirc_umf::song::{
    Cell, Instrument, ModRoute, NoteCell, Song, Source, WaveKind, CHANNELS, FINE_MAX, FINE_MIN,
    MAX_INSTRUMENTS, MAX_NOTE, MAX_ROWS,
};
use sirc_umf::OUTPUT_RATE_HZ;

use crate::audio::Audio;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Pattern,
    Instruments,
    Song,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Note,
    Inst,
    Fine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromptKind {
    Save,
    ExportAsm,
    ExportWav,
    ImportSample,
    InstName,
    Title,
    Open,
}

impl PromptKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Save => "Save as",
            Self::ExportAsm => "Export assembly to",
            Self::ExportWav => "Render WAV to",
            Self::ImportSample => "Import WAV file",
            Self::InstName => "Instrument name",
            Self::Title => "Song title",
            Self::Open => "Open",
        }
    }
}

pub struct Prompt {
    pub kind: PromptKind,
    pub buf: String,
}

/// What is playing, for the position marker.
#[derive(Clone, Copy, Debug)]
pub struct PlayInfo {
    /// Rows before the first row that was rendered.
    pub row_offset: usize,
    pub rows: usize,
    pub loop_row: Option<usize>,
    pub row_frames: u64,
    pub first_pass: u64,
}

pub struct App {
    pub song: Song,
    pub path: Option<PathBuf>,
    pub dirty: bool,
    pub tab: Tab,
    pub cur_row: usize,
    pub cur_ch: usize,
    pub field: Field,
    pub octave: u8,
    pub cur_inst: u8,
    pub inst_sel: usize,
    pub inst_field: usize,
    pub song_field: usize,
    pub mute: [bool; CHANNELS],
    pub solo: Option<usize>,
    pub prompt: Option<Prompt>,
    pub status: String,
    pub status_err: bool,
    pub quit: bool,
    quit_armed: bool,
    audio: Box<dyn Audio>,
    play: Option<PlayInfo>,
}

/// Semitone offset above C of the octave for a piano-layout key.
fn piano_key(c: char) -> Option<i32> {
    Some(match c {
        'z' => 0,
        's' => 1,
        'x' => 2,
        'd' => 3,
        'c' => 4,
        'v' => 5,
        'g' => 6,
        'b' => 7,
        'h' => 8,
        'n' => 9,
        'j' => 10,
        'm' => 11,
        'q' => 12,
        '2' => 13,
        'w' => 14,
        '3' => 15,
        'e' => 16,
        'r' => 17,
        '5' => 18,
        't' => 19,
        '6' => 20,
        'y' => 21,
        '7' => 22,
        'u' => 23,
        'i' => 24,
        '9' => 25,
        'o' => 26,
        '0' => 27,
        'p' => 28,
        _ => return None,
    })
}

fn clamp_add(v: i64, delta: i64, lo: i64, hi: i64) -> i64 {
    (v + delta).clamp(lo, hi)
}

impl App {
    #[must_use]
    pub fn new(song: Song, path: Option<PathBuf>, audio: Box<dyn Audio>) -> Self {
        Self {
            song,
            path,
            dirty: false,
            tab: Tab::Pattern,
            cur_row: 0,
            cur_ch: 0,
            field: Field::Note,
            octave: 4,
            cur_inst: 0,
            inst_sel: 0,
            inst_field: 0,
            song_field: 0,
            mute: [false; CHANNELS],
            solo: None,
            prompt: None,
            status: "F1 pattern  F2 instruments  F3 song  Space play  Ctrl-S save  Ctrl-Q quit"
                .into(),
            status_err: false,
            quit: false,
            quit_armed: false,
            audio,
            play: None,
        }
    }

    fn info(&mut self, m: impl Into<String>) {
        self.status = m.into();
        self.status_err = false;
    }

    fn fail(&mut self, m: impl Into<String>) {
        self.status = m.into();
        self.status_err = true;
    }

    fn touch(&mut self) {
        self.dirty = true;
        self.quit_armed = false;
    }

    #[must_use]
    pub fn effective_mute(&self) -> [bool; CHANNELS] {
        let mut m = self.mute;
        if let Some(s) = self.solo {
            for (c, v) in m.iter_mut().enumerate() {
                *v = c != s;
            }
        }
        m
    }

    /// Row being played, from the audio position.
    #[must_use]
    pub fn play_row(&self) -> Option<usize> {
        let info = self.play?;
        let pos = self.audio.position()? as u64;
        let row = if pos < info.first_pass || info.loop_row.is_none() {
            pos / info.row_frames
        } else {
            let l = info.loop_row? as u64;
            let n = (info.rows as u64).saturating_sub(l).max(1);
            l + ((pos - info.first_pass) / info.row_frames) % n
        };
        Some(info.row_offset + row as usize)
    }

    #[must_use]
    pub fn playing(&self) -> bool {
        self.audio.position().is_some()
    }

    // -----------------------------------------------------------------------------------------
    // Playback and files
    // -----------------------------------------------------------------------------------------

    /// The instrument and fine value in effect at `row` for a channel, from earlier cells.
    fn state_at(&self, row: usize, ch: usize) -> (u8, i16) {
        let (mut inst, mut fine) = (0u8, 0i16);
        for r in &self.song.rows[..row] {
            if let Some(i) = r[ch].inst {
                inst = i;
            }
            if let Some(f) = r[ch].fine {
                fine = f;
            }
        }
        (inst, fine)
    }

    fn start_playback(&mut self, from_row: usize) {
        let mut song = self.song.clone();
        if from_row > 0 {
            let carried: Vec<(u8, i16)> =
                (0..CHANNELS).map(|c| self.state_at(from_row, c)).collect();
            song.rows = song.rows[from_row..].to_vec();
            for (c, (inst, fine)) in carried.into_iter().enumerate() {
                let cell = &mut song.rows[0][c];
                cell.inst = cell.inst.or(Some(inst));
                cell.fine = cell.fine.or(Some(fine));
            }
            song.loop_row = song.loop_row.and_then(|l| l.checked_sub(from_row));
        }
        let opts = RenderOpts {
            mute: self.effective_mute(),
            ..RenderOpts::default()
        };
        let started = std::time::Instant::now();
        match compile::render(&song, &opts, true) {
            Ok(r) => {
                let secs = r.frames.len() as f64 / OUTPUT_RATE_HZ;
                let took = started.elapsed().as_secs_f64();
                let row_frames = u64::from(song.speed) * u64::from(song.tick);
                self.play = Some(PlayInfo {
                    row_offset: from_row,
                    rows: song.rows.len(),
                    loop_row: song.loop_row,
                    row_frames,
                    first_pass: song.pass_frames(),
                });
                let overrun = r.overrun;
                self.audio.play(r.frames, r.wrap);
                if overrun {
                    self.fail("Playing, but the sequencer missed a tick: lengthen the tick");
                } else {
                    self.info(format!("Playing {secs:.1} s (rendered in {took:.2} s)"));
                }
            }
            Err(e) => self.fail(e),
        }
    }

    fn stop_playback(&mut self) {
        self.audio.stop();
        self.play = None;
    }

    /// Play one note on the selected instrument, on channel 0's settings, for auditioning.
    fn audition(&mut self, inst: usize, note: u8) {
        let mut song = Song {
            instruments: self.song.instruments.clone(),
            rows: vec![[Cell::default(); CHANNELS]; 24],
            loop_row: None,
            filter: self.song.filter,
            master_vol: self.song.master_vol,
            tick: self.song.tick,
            speed: 2,
            ..Song::default()
        };
        song.channels[0] = sirc_umf::song::ChannelCfg {
            modulator: None,
            ..self.song.channels[self.cur_ch.min(CHANNELS - 1)]
        };
        song.channels[0].porta = 0;
        song.rows[0][0] = Cell {
            note: NoteCell::On(note),
            inst: Some(inst as u8),
            fine: None,
        };
        song.rows[10][0].note = NoteCell::Off;
        let opts = RenderOpts {
            tail_seconds: 0.5,
            ..RenderOpts::default()
        };
        match compile::render(&song, &opts, false) {
            Ok(r) => {
                self.play = None;
                self.audio.play(r.frames, None);
            }
            Err(e) => self.fail(e),
        }
    }

    fn base_dir(&self) -> Option<PathBuf> {
        self.path
            .as_ref()
            .and_then(|p| p.parent().map(Path::to_path_buf))
    }

    fn default_name(&self, ext: &str) -> String {
        match &self.path {
            Some(p) => p.with_extension(ext).display().to_string(),
            None => format!("song.{ext}"),
        }
    }

    fn open_prompt(&mut self, kind: PromptKind, initial: String) {
        self.prompt = Some(Prompt { kind, buf: initial });
    }

    fn save_to(&mut self, path: &Path) {
        match std::fs::write(path, self.song.to_text()) {
            Ok(()) => {
                self.path = Some(path.to_path_buf());
                self.dirty = false;
                self.info(format!("Saved {}", path.display()));
            }
            Err(e) => self.fail(format!("{}: {e}", path.display())),
        }
    }

    fn export_asm(&mut self, path: &Path) {
        match compile::compile(&self.song, [false; CHANNELS]) {
            Ok(img) => {
                let text = compile::export_asm(&self.song, &img);
                match std::fs::write(path, text) {
                    Ok(()) => self.info(format!(
                        "Exported {} ({})",
                        path.display(),
                        compile::describe_usage(&img.usage)
                    )),
                    Err(e) => self.fail(format!("{}: {e}", path.display())),
                }
            }
            Err(e) => self.fail(e),
        }
    }

    fn export_wav(&mut self, path: &Path) {
        let opts = RenderOpts::default();
        match compile::render(&self.song, &opts, false) {
            Ok(r) => match sirc_umf::testkit::write_wav(path, &r.frames, OUTPUT_RATE_HZ as u32) {
                Ok(()) => self.info(format!(
                    "Rendered {:.1} s to {}",
                    r.frames.len() as f64 / OUTPUT_RATE_HZ,
                    path.display()
                )),
                Err(e) => self.fail(format!("{}: {e}", path.display())),
            },
            Err(e) => self.fail(e),
        }
    }

    fn open_song(&mut self, path: &Path) {
        match std::fs::read_to_string(path) {
            Ok(text) => match Song::from_text(&text, path.parent()) {
                Ok((song, warnings)) => {
                    self.song = song;
                    self.path = Some(path.to_path_buf());
                    self.dirty = false;
                    self.cur_row = 0;
                    self.inst_sel = 0;
                    if warnings.is_empty() {
                        self.info(format!("Opened {}", path.display()));
                    } else {
                        self.fail(warnings.join("; "));
                    }
                }
                Err(e) => self.fail(format!("{}: {e}", path.display())),
            },
            Err(e) => self.fail(format!("{}: {e}", path.display())),
        }
    }

    fn finish_prompt(&mut self) {
        let Some(p) = self.prompt.take() else { return };
        let text = p.buf.trim().to_string();
        if text.is_empty() {
            return;
        }
        match p.kind {
            PromptKind::Save => self.save_to(Path::new(&text)),
            PromptKind::ExportAsm => self.export_asm(Path::new(&text)),
            PromptKind::ExportWav => self.export_wav(Path::new(&text)),
            PromptKind::Open => self.open_song(Path::new(&text)),
            PromptKind::Title => {
                self.song.title = text;
                self.touch();
            }
            PromptKind::InstName => {
                if let Some(i) = self.song.instruments.get_mut(self.inst_sel) {
                    i.name = text;
                    self.touch();
                }
            }
            PromptKind::ImportSample => self.import_sample(&text),
        }
    }

    fn import_sample(&mut self, path: &str) {
        let name = Path::new(path)
            .file_stem()
            .map_or_else(|| "Sample".to_string(), |s| s.to_string_lossy().to_string());
        // Keep paths relative to the song file when possible.
        let stored = self
            .base_dir()
            .and_then(|b| Path::new(path).strip_prefix(b).ok().map(Path::to_path_buf))
            .map_or_else(|| path.to_string(), |p| p.display().to_string());
        // A new instrument, or replace the sample of the selected one.
        let existing = self
            .song
            .instruments
            .get(self.inst_sel)
            .is_some_and(|i| matches!(i.source, Source::Sample { .. }));
        let base = self.base_dir();
        let load_from = if Path::new(path).is_absolute() || base.is_none() {
            None
        } else {
            base.as_deref()
        };
        // Relative paths typed by the user resolve against the working directory first.
        let attempt = Instrument::load_sample(&name, path, load_from, 39, 4);
        match attempt {
            Ok(mut inst) => {
                if let Source::Sample { path: p, .. } = &mut inst.source {
                    *p = stored;
                }
                if existing {
                    if let Some(slot) = self.song.instruments.get_mut(self.inst_sel) {
                        let name = slot.name.clone();
                        *slot = Instrument { name, ..inst };
                    }
                } else if self.song.instruments.len() < MAX_INSTRUMENTS {
                    self.song.instruments.push(inst);
                    self.inst_sel = self.song.instruments.len() - 1;
                } else {
                    self.fail("too many instruments");
                    return;
                }
                self.touch();
                let (len, rate) = match &self.song.instruments[self.inst_sel].source {
                    Source::Sample { data, rate, .. } => (data.len(), *rate),
                    Source::Wave { .. } => (0, 0.0),
                };
                self.info(format!(
                    "Imported {len} bytes at {rate:.0} Hz; set the root note and decimation"
                ));
            }
            Err(e) => self.fail(e),
        }
    }

    // -----------------------------------------------------------------------------------------
    // Keys
    // -----------------------------------------------------------------------------------------

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.prompt.is_some() {
            self.prompt_key(key);
            return;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match (key.code, ctrl) {
            (KeyCode::Char('q'), true) => {
                if self.dirty && !self.quit_armed {
                    self.quit_armed = true;
                    self.fail("Unsaved changes: press Ctrl-Q again to quit");
                } else {
                    self.quit = true;
                }
                return;
            }
            (KeyCode::Char('s'), true) => {
                let default = self
                    .path
                    .as_ref()
                    .map_or_else(|| "song.erence".to_string(), |p| p.display().to_string());
                match self.path.clone() {
                    Some(p) => self.save_to(&p),
                    None => self.open_prompt(PromptKind::Save, default),
                }
                return;
            }
            (KeyCode::Char('o'), true) => {
                self.open_prompt(PromptKind::Open, String::new());
                return;
            }
            (KeyCode::Char('e'), true) => {
                let n = self.default_name("sasm");
                self.open_prompt(PromptKind::ExportAsm, n);
                return;
            }
            (KeyCode::Char('w'), true) => {
                let n = self.default_name("wav");
                self.open_prompt(PromptKind::ExportWav, n);
                return;
            }
            (KeyCode::F(1), _) => {
                self.tab = Tab::Pattern;
                return;
            }
            (KeyCode::F(2), _) => {
                self.tab = Tab::Instruments;
                return;
            }
            (KeyCode::F(3), _) => {
                self.tab = Tab::Song;
                return;
            }
            (KeyCode::F(5), _) => {
                self.start_playback(0);
                return;
            }
            (KeyCode::F(6), _) => {
                self.start_playback(self.cur_row);
                return;
            }
            (KeyCode::F(7) | KeyCode::Esc, _) => {
                self.stop_playback();
                return;
            }
            _ => {}
        }
        match self.tab {
            Tab::Pattern => self.pattern_key(key),
            Tab::Instruments => self.instrument_key(key),
            Tab::Song => self.song_key(key),
        }
    }

    fn prompt_key(&mut self, key: KeyEvent) {
        let Some(p) = &mut self.prompt else { return };
        match key.code {
            KeyCode::Esc => self.prompt = None,
            KeyCode::Enter => self.finish_prompt(),
            KeyCode::Backspace => {
                p.buf.pop();
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => p.buf.push(c),
            _ => {}
        }
    }

    fn move_row(&mut self, delta: i64) {
        let max = self.song.rows.len() as i64 - 1;
        self.cur_row = clamp_add(self.cur_row as i64, delta, 0, max) as usize;
    }

    fn pattern_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.move_row(-1),
            KeyCode::Down => self.move_row(1),
            KeyCode::PageUp => self.move_row(-16),
            KeyCode::PageDown => self.move_row(16),
            KeyCode::Home => self.cur_row = 0,
            KeyCode::End => self.cur_row = self.song.rows.len() - 1,
            KeyCode::Left => {
                self.field = match self.field {
                    Field::Note => {
                        if self.cur_ch > 0 {
                            self.cur_ch -= 1;
                            Field::Fine
                        } else {
                            Field::Note
                        }
                    }
                    Field::Inst => Field::Note,
                    Field::Fine => Field::Inst,
                };
            }
            KeyCode::Right => {
                self.field = match self.field {
                    Field::Note => Field::Inst,
                    Field::Inst => Field::Fine,
                    Field::Fine => {
                        if self.cur_ch + 1 < CHANNELS {
                            self.cur_ch += 1;
                            Field::Note
                        } else {
                            Field::Fine
                        }
                    }
                };
            }
            KeyCode::Tab => {
                self.cur_ch = (self.cur_ch + 1) % CHANNELS;
                self.field = Field::Note;
            }
            KeyCode::BackTab => {
                self.cur_ch = (self.cur_ch + CHANNELS - 1) % CHANNELS;
                self.field = Field::Note;
            }
            KeyCode::Delete | KeyCode::Backspace => self.clear_field(),
            KeyCode::Insert => {
                if self.song.rows.len() < MAX_ROWS {
                    self.song
                        .rows
                        .insert(self.cur_row, [Cell::default(); CHANNELS]);
                    if let Some(l) = &mut self.song.loop_row {
                        if *l >= self.cur_row {
                            *l += 1;
                        }
                    }
                    self.touch();
                }
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.song.rows.len() > 1 {
                    self.song.rows.remove(self.cur_row);
                    self.cur_row = self.cur_row.min(self.song.rows.len() - 1);
                    self.song.loop_row =
                        self.song.loop_row.map(|l| l.min(self.song.rows.len() - 1));
                    self.touch();
                }
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Append sixteen rows.
                let room = MAX_ROWS - self.song.rows.len();
                self.song.rows.extend(std::iter::repeat_n(
                    [Cell::default(); CHANNELS],
                    16.min(room),
                ));
                self.touch();
            }
            KeyCode::Char('\\') => {
                self.mute[self.cur_ch] = !self.mute[self.cur_ch];
                let m = self.mute[self.cur_ch];
                self.info(format!(
                    "Channel {} {}",
                    self.cur_ch + 1,
                    if m { "muted" } else { "unmuted" }
                ));
            }
            KeyCode::Char('/') => {
                self.solo = if self.solo == Some(self.cur_ch) {
                    None
                } else {
                    Some(self.cur_ch)
                };
                let s = self.solo;
                self.info(match s {
                    Some(c) => format!("Solo channel {}", c + 1),
                    None => "Solo off".to_string(),
                });
            }
            KeyCode::Char('l') => {
                self.song.loop_row = if self.song.loop_row == Some(self.cur_row) {
                    None
                } else {
                    Some(self.cur_row)
                };
                self.touch();
            }
            KeyCode::Char('[') => self.octave = self.octave.saturating_sub(1).max(1),
            KeyCode::Char(']') => self.octave = (self.octave + 1).min(9),
            KeyCode::Char(';') => {
                self.cur_inst = self.cur_inst.saturating_sub(1);
            }
            KeyCode::Char('\'') => {
                let max = self.song.instruments.len().saturating_sub(1) as u8;
                self.cur_inst = (self.cur_inst + 1).min(max);
            }
            KeyCode::Char(' ') => {
                if self.playing() {
                    self.stop_playback();
                } else {
                    self.start_playback(0);
                }
            }
            KeyCode::Char(c) => self.enter_char(c),
            _ => {}
        }
    }

    fn cell_mut(&mut self) -> &mut Cell {
        &mut self.song.rows[self.cur_row][self.cur_ch]
    }

    fn clear_field(&mut self) {
        let field = self.field;
        let c = self.cell_mut();
        match field {
            Field::Note => c.note = NoteCell::None,
            Field::Inst => c.inst = None,
            Field::Fine => c.fine = None,
        }
        self.touch();
    }

    fn enter_char(&mut self, ch: char) {
        match self.field {
            Field::Note => self.enter_note(ch),
            Field::Inst => {
                if let Some(d) = ch.to_digit(16) {
                    let old = self.cell_mut().inst.unwrap_or(0);
                    let max = self.song.instruments.len().saturating_sub(1) as u8;
                    let new = ((old & 0x0F) << 4) | d as u8;
                    let new = if new > max { d as u8 } else { new };
                    self.cell_mut().inst = Some(new.min(max));
                    self.touch();
                } else if ch == '.' {
                    self.clear_field();
                }
            }
            Field::Fine => {
                if let Some(d) = ch.to_digit(10) {
                    let old = i32::from(self.cell_mut().fine.unwrap_or(0));
                    let neg = old < 0;
                    let mag = (old.abs() * 10 + d as i32).min(4096);
                    let v = if neg { -mag } else { mag };
                    self.cell_mut().fine = Some((v as i16).clamp(FINE_MIN, FINE_MAX));
                    self.touch();
                } else if ch == '-' || ch == '+' {
                    let old = self.cell_mut().fine.unwrap_or(0);
                    let v = if ch == '-' { -old.abs() } else { old.abs() };
                    self.cell_mut().fine = Some(v.clamp(FINE_MIN, FINE_MAX));
                    self.touch();
                } else if ch == '.' {
                    self.clear_field();
                }
            }
        }
    }

    fn enter_note(&mut self, ch: char) {
        if ch == '1' || ch == '`' {
            self.cell_mut().note = NoteCell::Off;
            self.touch();
            self.move_row(1);
            return;
        }
        if ch == '.' {
            self.clear_field();
            return;
        }
        let Some(semi) = piano_key(ch) else { return };
        let note = i32::from(self.octave) * 12 - 9 + semi;
        if !(0..=i32::from(MAX_NOTE)).contains(&note) {
            return;
        }
        let note = note as u8;
        // Select the current instrument for this note if the column is not already using it.
        let (inst_now, _) = self.state_at(self.cur_row, self.cur_ch);
        let want = self.cur_inst;
        let c = self.cell_mut();
        c.note = NoteCell::On(note);
        if c.inst.is_none() && inst_now != want {
            c.inst = Some(want);
        }
        let inst = c.inst.unwrap_or(inst_now);
        self.touch();
        self.audition(usize::from(inst), note);
        self.move_row(1);
    }

    // -----------------------------------------------------------------------------------------
    // Instruments tab
    // -----------------------------------------------------------------------------------------

    /// Field list for the selected instrument: (label, value).
    #[must_use]
    pub fn instrument_fields(&self) -> Vec<(&'static str, String)> {
        let Some(i) = self.song.instruments.get(self.inst_sel) else {
            return Vec::new();
        };
        let mut f = vec![("Name", i.name.clone())];
        match &i.source {
            Source::Wave {
                kind,
                len,
                amp,
                duty,
            } => {
                f.push(("Source", "waveform".into()));
                f.push(("Wave", kind.name().into()));
                f.push(("Length", format!("{len} bytes")));
                f.push(("Amplitude", amp.to_string()));
                if *kind == WaveKind::Pulse {
                    f.push(("Duty", format!("{duty} %")));
                }
            }
            Source::Sample {
                path,
                root,
                decimate,
                data,
                rate,
            } => {
                f.push(("Source", "sample".into()));
                f.push(("File", path.clone()));
                f.push(("Data", format!("{} bytes at {rate:.0} Hz", data.len())));
                f.push(("Root note", sirc_umf::song::note_name(*root)));
                f.push(("Decimate", format!("1/{decimate}")));
                f.push(("Loop", if i.loop_en { "on" } else { "off" }.into()));
                f.push(("Loop start", i.loop_start.to_string()));
            }
        }
        f.push(("Pitch div", i.div.to_string()));
        f.push(("Attack", i.adsr[0].to_string()));
        f.push(("Decay", i.adsr[1].to_string()));
        f.push(("Sustain", i.adsr[2].to_string()));
        f.push(("Release", i.adsr[3].to_string()));
        f
    }

    fn instrument_key(&mut self, key: KeyEvent) {
        let n = self.song.instruments.len();
        let fields = self.instrument_fields().len();
        match key.code {
            KeyCode::PageUp => self.inst_sel = self.inst_sel.saturating_sub(1),
            KeyCode::PageDown => self.inst_sel = (self.inst_sel + 1).min(n - 1),
            KeyCode::Up => self.inst_field = self.inst_field.saturating_sub(1),
            KeyCode::Down => self.inst_field = (self.inst_field + 1).min(fields - 1),
            KeyCode::Left => self.adjust_instrument(-1, key.modifiers),
            KeyCode::Right => self.adjust_instrument(1, key.modifiers),
            KeyCode::Enter => {
                let label = self.instrument_fields()[self.inst_field].0;
                match label {
                    "Name" => {
                        let name = self.song.instruments[self.inst_sel].name.clone();
                        self.open_prompt(PromptKind::InstName, name);
                    }
                    "File" => self.open_prompt(PromptKind::ImportSample, String::new()),
                    _ => {}
                }
            }
            KeyCode::Char('n') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                if n < MAX_INSTRUMENTS {
                    self.song
                        .instruments
                        .push(Instrument::wave(&format!("Instrument {n}"), WaveKind::Sine));
                    self.inst_sel = n;
                    self.inst_field = 0;
                    self.touch();
                }
            }
            KeyCode::Char('c') => {
                if n < MAX_INSTRUMENTS {
                    let mut copy = self.song.instruments[self.inst_sel].clone();
                    copy.name.push_str(" copy");
                    self.song.instruments.push(copy);
                    self.inst_sel = n;
                    self.touch();
                }
            }
            KeyCode::Char('i') => self.open_prompt(PromptKind::ImportSample, String::new()),
            KeyCode::Char('x') | KeyCode::Delete => self.delete_instrument(),
            KeyCode::Char(c) => {
                if let Some(semi) = piano_key(c) {
                    let note = i32::from(self.octave) * 12 - 9 + semi;
                    if (0..=i32::from(MAX_NOTE)).contains(&note) {
                        self.audition(self.inst_sel, note as u8);
                    }
                } else if c == '[' {
                    self.octave = self.octave.saturating_sub(1).max(1);
                } else if c == ']' {
                    self.octave = (self.octave + 1).min(9);
                }
            }
            _ => {}
        }
        let fields = self.instrument_fields().len();
        self.inst_field = self.inst_field.min(fields.saturating_sub(1));
    }

    fn delete_instrument(&mut self) {
        let n = self.song.instruments.len();
        if n <= 1 {
            self.fail("A song needs at least one instrument");
            return;
        }
        let gone = self.inst_sel as u8;
        self.song.instruments.remove(self.inst_sel);
        for row in &mut self.song.rows {
            for cell in row.iter_mut() {
                cell.inst = match cell.inst {
                    Some(i) if i == gone => None,
                    Some(i) if i > gone => Some(i - 1),
                    other => other,
                };
            }
        }
        self.inst_sel = self.inst_sel.min(n - 2);
        self.cur_inst = self.cur_inst.min((n - 2) as u8);
        self.touch();
        self.info("Instrument deleted; notes that used it now use the previous instrument");
    }

    fn adjust_instrument(&mut self, dir: i64, mods: KeyModifiers) {
        let big = if mods.contains(KeyModifiers::SHIFT) {
            10
        } else {
            1
        };
        let delta = dir * big;
        let label = match self.instrument_fields().get(self.inst_field) {
            Some((l, _)) => *l,
            None => return,
        };
        let Some(inst) = self.song.instruments.get_mut(self.inst_sel) else {
            return;
        };
        let byte = |v: u8| clamp_add(i64::from(v), delta, 0, 255) as u8;
        match label {
            "Source" => {
                if matches!(inst.source, Source::Sample { .. }) {
                    *inst = Instrument {
                        name: inst.name.clone(),
                        ..Instrument::wave("", WaveKind::Sine)
                    };
                } else {
                    self.open_prompt(PromptKind::ImportSample, String::new());
                    return;
                }
            }
            "Wave" => {
                if let Source::Wave { kind, .. } = &mut inst.source {
                    *kind = if dir > 0 { kind.next() } else { kind.prev() };
                }
            }
            "Length" => {
                if let Source::Wave { len, .. } = &mut inst.source {
                    // Powers of two from 8 to 8192, plus the in-between steps with Shift.
                    *len = if mods.contains(KeyModifiers::SHIFT) {
                        clamp_add(i64::from(*len), dir * 8, 8, 8192) as u16
                    } else if dir > 0 {
                        (*len).saturating_mul(2).min(8192)
                    } else {
                        (*len / 2).max(8)
                    };
                }
            }
            "Amplitude" => {
                if let Source::Wave { amp, .. } = &mut inst.source {
                    *amp = clamp_add(i64::from(*amp), delta, 1, 127) as u8;
                }
            }
            "Duty" => {
                if let Source::Wave { duty, .. } = &mut inst.source {
                    *duty = clamp_add(i64::from(*duty), delta, 1, 99) as u8;
                }
            }
            "Root note" => {
                if let Source::Sample { root, .. } = &mut inst.source {
                    *root = clamp_add(i64::from(*root), dir * if big > 1 { 12 } else { 1 }, 0, 119)
                        as u8;
                }
            }
            "Decimate" => {
                if let Source::Sample { decimate, .. } = &mut inst.source {
                    *decimate = clamp_add(i64::from(*decimate), dir, 1, 32) as u8;
                }
                let base = self.base_dir();
                if let Some(inst) = self.song.instruments.get_mut(self.inst_sel) {
                    if let Err(e) = inst.reload_sample(base.as_deref()) {
                        self.fail(e);
                    }
                }
            }
            "Pitch div" => inst.div = clamp_add(i64::from(inst.div), delta, 1, 64) as u8,
            "Loop" => inst.loop_en = !inst.loop_en,
            "Loop start" => {
                let max = match &inst.source {
                    Source::Sample { data, .. } => data.len().saturating_sub(1) as i64,
                    Source::Wave { .. } => 0,
                };
                inst.loop_start = clamp_add(i64::from(inst.loop_start), delta * 16, 0, max) as u16;
            }
            "Attack" => inst.adsr[0] = byte(inst.adsr[0]),
            "Decay" => inst.adsr[1] = byte(inst.adsr[1]),
            "Sustain" => inst.adsr[2] = byte(inst.adsr[2]),
            "Release" => inst.adsr[3] = byte(inst.adsr[3]),
            _ => return,
        }
        self.touch();
    }

    // -----------------------------------------------------------------------------------------
    // Song tab
    // -----------------------------------------------------------------------------------------

    /// Field list for the song tab: (group, label, value).
    #[must_use]
    pub fn song_fields(&self) -> Vec<(String, String, String)> {
        let s = &self.song;
        let mut f: Vec<(String, String, String)> = vec![
            ("Song".into(), "Title".into(), s.title.clone()),
            (
                "Song".into(),
                "Tick".into(),
                format!("{} samples ({:.2} ms)", s.tick, s.tick_ms()),
            ),
            (
                "Song".into(),
                "Speed".into(),
                format!("{} ticks per row ({:.0} BPM)", s.speed, s.bpm()),
            ),
            (
                "Song".into(),
                "Master volume".into(),
                s.master_vol.to_string(),
            ),
            (
                "Filter".into(),
                "Mode".into(),
                ["low-pass", "band-pass", "high-pass", "low-pass (3)"]
                    [usize::from(s.filter.mode & 3)]
                .into(),
            ),
            (
                "Filter".into(),
                "Cutoff".into(),
                format!(
                    "{} ({:.0} Hz)",
                    s.filter.cutoff,
                    sirc_umf::filter::cutoff_hz(s.filter.cutoff)
                ),
            ),
            (
                "Filter".into(),
                "Resonance".into(),
                s.filter.resonance.to_string(),
            ),
            (
                "Filter".into(),
                "Volume".into(),
                s.filter.volume.to_string(),
            ),
            ("Filter".into(), "Pan".into(), s.filter.pan.to_string()),
        ];
        for (c, ch) in s.channels.iter().enumerate() {
            let g = format!("Channel {}", c + 1);
            f.push((g.clone(), "Volume".into(), ch.volume.to_string()));
            f.push((g.clone(), "Pan".into(), ch.pan.to_string()));
            f.push((
                g.clone(),
                "Through filter".into(),
                if ch.filter { "yes" } else { "no" }.into(),
            ));
            f.push((
                g.clone(),
                "Portamento".into(),
                if ch.porta == 0 {
                    "off".into()
                } else {
                    format!("{} units / 2 ms", ch.porta)
                },
            ));
            match ch.modulator {
                None => f.push((
                    g.clone(),
                    "Modulates".into(),
                    "nothing (audio voice)".into(),
                )),
                Some(m) => {
                    f.push((g.clone(), "Modulates".into(), dest_name(m.dest).into()));
                    f.push((g.clone(), "Target voice".into(), (m.target + 1).to_string()));
                    f.push((g.clone(), "Depth shift".into(), m.shift.to_string()));
                    f.push((
                        g.clone(),
                        "Invert".into(),
                        if m.invert { "yes" } else { "no" }.into(),
                    ));
                }
            }
        }
        f
    }

    fn song_key(&mut self, key: KeyEvent) {
        let n = self.song_fields().len();
        match key.code {
            KeyCode::Up => self.song_field = self.song_field.saturating_sub(1),
            KeyCode::Down => self.song_field = (self.song_field + 1).min(n - 1),
            KeyCode::PageUp => self.song_field = self.song_field.saturating_sub(5),
            KeyCode::PageDown => self.song_field = (self.song_field + 5).min(n - 1),
            KeyCode::Left => self.adjust_song(-1, key.modifiers),
            KeyCode::Right => self.adjust_song(1, key.modifiers),
            KeyCode::Enter if self.song_fields()[self.song_field].1 == "Title" => {
                let t = self.song.title.clone();
                self.open_prompt(PromptKind::Title, t);
            }
            _ => {}
        }
        let n = self.song_fields().len();
        self.song_field = self.song_field.min(n - 1);
    }

    fn adjust_song(&mut self, dir: i64, mods: KeyModifiers) {
        let big = if mods.contains(KeyModifiers::SHIFT) {
            10
        } else {
            1
        };
        let delta = dir * big;
        let fields = self.song_fields();
        let (group, label, _) = fields[self.song_field].clone();
        let s = &mut self.song;
        let byte = |v: u8| clamp_add(i64::from(v), delta, 0, 255) as u8;
        if let Some(c) = group
            .strip_prefix("Channel ")
            .and_then(|c| c.parse::<usize>().ok())
        {
            let ch = &mut s.channels[c - 1];
            match label.as_str() {
                "Volume" => ch.volume = byte(ch.volume),
                "Pan" => ch.pan = byte(ch.pan),
                "Through filter" => ch.filter = !ch.filter,
                "Portamento" => {
                    ch.porta = clamp_add(i64::from(ch.porta), delta * 32, 0, 65535) as u16;
                }
                "Modulates" => {
                    let cur = ch.modulator.map_or(0, |m| m.dest);
                    let next = (i64::from(cur) + dir).clamp(0, i64::from(moddest::FILTER_PAN));
                    ch.modulator = if next == 0 {
                        None
                    } else {
                        let old = ch.modulator;
                        Some(ModRoute {
                            dest: next as u8,
                            target: old.map_or(0, |m| m.target),
                            shift: old.map_or(-2, |m| m.shift),
                            invert: old.is_some_and(|m| m.invert),
                        })
                    };
                }
                "Target voice" => {
                    if let Some(m) = &mut ch.modulator {
                        m.target = clamp_add(i64::from(m.target), dir, 0, 5) as u8;
                    }
                }
                "Depth shift" => {
                    if let Some(m) = &mut ch.modulator {
                        m.shift = clamp_add(i64::from(m.shift), dir, -8, 7) as i8;
                    }
                }
                "Invert" => {
                    if let Some(m) = &mut ch.modulator {
                        m.invert = !m.invert;
                    }
                }
                _ => {}
            }
        } else {
            match label.as_str() {
                "Tick" => {
                    s.tick = clamp_add(i64::from(s.tick), delta * 8, 16, 8000) as u16;
                }
                "Speed" => s.speed = clamp_add(i64::from(s.speed), dir, 1, 64) as u16,
                "Master volume" => s.master_vol = byte(s.master_vol),
                "Mode" => s.filter.mode = clamp_add(i64::from(s.filter.mode), dir, 0, 2) as u8,
                "Cutoff" => s.filter.cutoff = byte(s.filter.cutoff),
                "Resonance" => s.filter.resonance = byte(s.filter.resonance),
                "Volume" => s.filter.volume = byte(s.filter.volume),
                "Pan" => s.filter.pan = byte(s.filter.pan),
                _ => return,
            }
        }
        self.touch();
    }
}

#[must_use]
pub fn dest_name(dest: u8) -> &'static str {
    match u16::from(dest) {
        moddest::PITCH => "pitch",
        moddest::VOLUME => "volume",
        moddest::PAN => "pan",
        moddest::FILTER_CUTOFF => "filter cutoff",
        moddest::MASTER_VOLUME => "master volume",
        moddest::FILTER_PAN => "filter pan",
        _ => "nothing",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::NullAudio;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn key(c: KeyCode) -> KeyEvent {
        KeyEvent {
            code: c,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn ctrl(c: char) -> KeyEvent {
        KeyEvent {
            modifiers: KeyModifiers::CONTROL,
            ..key(KeyCode::Char(c))
        }
    }

    fn app() -> App {
        App::new(Song::default(), None, Box::new(NullAudio::default()))
    }

    fn type_keys(a: &mut App, s: &str) {
        for c in s.chars() {
            a.handle_key(key(KeyCode::Char(c)));
        }
    }

    #[test]
    fn piano_keys_enter_notes_and_advance() {
        let mut a = app();
        type_keys(&mut a, "z"); // C-4
        assert_eq!(a.song.rows[0][0].note, NoteCell::On(39));
        assert_eq!(a.cur_row, 1);
        type_keys(&mut a, "q"); // C-5
        assert_eq!(a.song.rows[1][0].note, NoteCell::On(51));
        assert!(a.dirty);
    }

    #[test]
    fn octave_keys_shift_notes() {
        let mut a = app();
        type_keys(&mut a, "]z");
        assert_eq!(a.song.rows[0][0].note, NoteCell::On(51));
        type_keys(&mut a, "[[z");
        assert_eq!(a.song.rows[1][0].note, NoteCell::On(27));
    }

    #[test]
    fn note_off_and_clear() {
        let mut a = app();
        type_keys(&mut a, "1");
        assert_eq!(a.song.rows[0][0].note, NoteCell::Off);
        a.handle_key(key(KeyCode::Up));
        a.handle_key(key(KeyCode::Delete));
        assert_eq!(a.song.rows[0][0].note, NoteCell::None);
    }

    #[test]
    fn current_instrument_is_written_when_it_changes() {
        let mut a = app();
        a.cur_inst = 2;
        type_keys(&mut a, "z");
        assert_eq!(a.song.rows[0][0].inst, Some(2));
        // The same instrument again needs no new instrument cell.
        type_keys(&mut a, "x");
        assert_eq!(a.song.rows[1][0].inst, None);
    }

    #[test]
    fn instrument_and_fine_fields_take_typed_values() {
        let mut a = app();
        a.handle_key(key(KeyCode::Right));
        type_keys(&mut a, "2");
        assert_eq!(a.song.rows[0][0].inst, Some(2));
        a.handle_key(key(KeyCode::Right));
        type_keys(&mut a, "125-");
        assert_eq!(a.song.rows[0][0].fine, Some(-125));
        type_keys(&mut a, "9999");
        assert_eq!(a.song.rows[0][0].fine, Some(-4096));
    }

    #[test]
    fn movement_wraps_between_channels() {
        let mut a = app();
        for _ in 0..3 {
            a.handle_key(key(KeyCode::Right));
        }
        assert_eq!((a.cur_ch, a.field), (1, Field::Note));
        a.handle_key(key(KeyCode::Left));
        assert_eq!((a.cur_ch, a.field), (0, Field::Fine));
        a.handle_key(key(KeyCode::Tab));
        assert_eq!(a.cur_ch, 1);
    }

    #[test]
    fn insert_and_delete_rows_keep_the_loop_row() {
        let mut a = app();
        a.song.loop_row = Some(4);
        a.cur_row = 2;
        a.handle_key(key(KeyCode::Insert));
        assert_eq!(a.song.rows.len(), 33);
        assert_eq!(a.song.loop_row, Some(5));
        a.handle_key(ctrl('d'));
        assert_eq!(a.song.rows.len(), 32);
    }

    #[test]
    fn loop_row_toggles() {
        let mut a = app();
        a.cur_row = 3;
        type_keys(&mut a, "l");
        assert_eq!(a.song.loop_row, Some(3));
        type_keys(&mut a, "l");
        assert_eq!(a.song.loop_row, None);
    }

    #[test]
    fn mute_and_solo() {
        let mut a = app();
        type_keys(&mut a, "\\");
        assert!(a.effective_mute()[0]);
        type_keys(&mut a, "\\/");
        a.cur_ch = 2;
        type_keys(&mut a, "/");
        let m = a.effective_mute();
        assert_eq!(m, [true, true, false, true, true, true]);
    }

    #[test]
    fn quitting_with_unsaved_changes_needs_confirmation() {
        let mut a = app();
        type_keys(&mut a, "z");
        a.handle_key(ctrl('q'));
        assert!(!a.quit);
        a.handle_key(ctrl('q'));
        assert!(a.quit);
    }

    #[test]
    fn tabs_and_instrument_editing() {
        let mut a = app();
        a.handle_key(key(KeyCode::F(2)));
        assert_eq!(a.tab, Tab::Instruments);
        let before = a.song.instruments.len();
        type_keys(&mut a, "n");
        assert_eq!(a.song.instruments.len(), before + 1);
        assert_eq!(a.inst_sel, before);
        // Wave field: cycle to the next waveform.
        a.inst_field = 2;
        a.handle_key(key(KeyCode::Right));
        match &a.song.instruments[a.inst_sel].source {
            Source::Wave { kind, .. } => assert_eq!(*kind, WaveKind::Saw),
            Source::Sample { .. } => panic!("expected a waveform"),
        }
    }

    #[test]
    fn deleting_an_instrument_renumbers_notes() {
        let mut a = app();
        a.song.rows[0][0] = Cell {
            note: NoteCell::On(10),
            inst: Some(2),
            fine: None,
        };
        a.song.rows[1][0] = Cell {
            note: NoteCell::On(10),
            inst: Some(1),
            fine: None,
        };
        a.tab = Tab::Instruments;
        a.inst_sel = 1;
        type_keys(&mut a, "x");
        assert_eq!(a.song.instruments.len(), 2);
        assert_eq!(a.song.rows[0][0].inst, Some(1));
        assert_eq!(a.song.rows[1][0].inst, None);
    }

    #[test]
    fn song_tab_adjusts_values_and_modulator() {
        let mut a = app();
        a.tab = Tab::Song;
        let speed = a.song.speed;
        a.song_field = 2;
        a.handle_key(key(KeyCode::Right));
        assert_eq!(a.song.speed, speed + 1);
        // Channel 6's "Modulates" field: nothing -> pitch.
        let idx = a
            .song_fields()
            .iter()
            .position(|(g, l, _)| g == "Channel 6" && l == "Modulates")
            .unwrap();
        a.song_field = idx;
        a.handle_key(key(KeyCode::Right));
        assert_eq!(
            a.song.channels[5].modulator.map(|m| m.dest),
            Some(moddest::PITCH as u8)
        );
        assert!(a.song_fields().iter().any(|(_, l, _)| l == "Depth shift"));
    }

    #[test]
    fn prompts_collect_text_and_cancel() {
        let mut a = app();
        a.tab = Tab::Song;
        a.song_field = 0;
        a.handle_key(key(KeyCode::Enter));
        assert!(a.prompt.is_some());
        a.handle_key(key(KeyCode::Backspace));
        type_keys(&mut a, "X");
        a.handle_key(key(KeyCode::Enter));
        assert!(a.song.title.ends_with('X'));
        a.handle_key(key(KeyCode::Enter));
        a.handle_key(key(KeyCode::Esc));
        assert!(a.prompt.is_none());
    }

    #[test]
    fn playback_renders_and_hands_audio_to_the_backend() {
        let mut a = app();
        type_keys(&mut a, "zzz");
        a.song.speed = 2;
        a.handle_key(key(KeyCode::F(5)));
        assert!(a.status.starts_with("Playing"), "{}", a.status);
    }
}
