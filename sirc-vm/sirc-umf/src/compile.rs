//! Compile a [`Song`] to a SIRCAS audio memory image and register setup, load it into the model
//! for preview, and export it as SIRC assembly.

use std::fmt::Write as _;

use crate::core::{mod_route, reg, Policy, Sircas};
use crate::song::{
    Cell, NoteCell, Song, Source, CHANNELS, FINE_MAX, FINE_MIN, MAX_INSTRUMENTS, MAX_NOTE,
};
use crate::testkit::{isa, spit_entries};
use crate::{CLKI_PER_FRAME, OUTPUT_RATE_HZ, RAM_WINDOW, RAM_WORDS};

/// Longest `WAIT` the sequencer takes.
const MAX_WAIT: u32 = 8191;

/// A compiled song: the audio memory contents and the register writes that start it.
#[derive(Clone, Debug)]
pub struct Image {
    /// Audio memory words from word 0, trimmed to the used length.
    pub words: Vec<u16>,
    /// Window address of the instrument table.
    pub instr_base: u16,
    /// Window address of each channel's sequence, or `None` for a channel with no events.
    pub seq_pc: [Option<u16>; CHANNELS],
    /// Register writes in order, as (offset, value). The last ones start the sequencers.
    pub regs: Vec<(u16, u16)>,
    pub usage: Usage,
}

/// Where the audio memory went, in bytes.
#[derive(Clone, Copy, Debug, Default)]
pub struct Usage {
    pub instruments: usize,
    pub pitch_tables: usize,
    pub samples: usize,
    pub sequences: [usize; CHANNELS],
}

impl Usage {
    #[must_use]
    pub fn total(&self) -> usize {
        self.instruments + self.pitch_tables + self.samples + self.sequences.iter().sum::<usize>()
    }
}

/// Bytes of audio memory.
pub const MEMORY_BYTES: usize = RAM_WORDS * 2;

struct Mem {
    words: Vec<u16>,
    next: usize, // byte offset
}

impl Mem {
    fn align(&mut self, to: usize) {
        self.next = self.next.div_ceil(to) * to;
    }

    fn ensure(&mut self, end: usize) -> Result<(), String> {
        if end > MEMORY_BYTES {
            return Err(format!(
                "audio memory full: {end} bytes needed, {MEMORY_BYTES} available"
            ));
        }
        if self.words.len() < end.div_ceil(2) {
            self.words.resize(end.div_ceil(2), 0);
        }
        Ok(())
    }

    fn put_bytes(&mut self, data: &[i8]) -> Result<usize, String> {
        self.align(2);
        let at = self.next;
        self.ensure(at + data.len())?;
        for (i, b) in data.iter().enumerate() {
            let o = at + i;
            let w = &mut self.words[o / 2];
            if o.is_multiple_of(2) {
                *w = (*w & 0x00FF) | (u16::from(*b as u8) << 8);
            } else {
                *w = (*w & 0xFF00) | u16::from(*b as u8);
            }
        }
        self.next += data.len();
        Ok(at)
    }

    fn put_words(&mut self, data: &[u16]) -> Result<usize, String> {
        self.align(2);
        let at = self.next;
        self.ensure(at + data.len() * 2)?;
        self.words[at / 2..at / 2 + data.len()].copy_from_slice(data);
        self.next += data.len() * 2;
        Ok(at)
    }
}

/// What the compiler tracks about a channel while it walks the rows.
#[derive(Clone, Copy)]
struct Tracked {
    /// Value the `INSTR` / `FINE` registers are known to hold, or `None` if unknown.
    inst: Option<u8>,
    fine: Option<i16>,
}

fn flush_wait(prog: &mut Vec<u16>, pending: &mut u32) {
    while *pending > 0 {
        let n = (*pending).min(MAX_WAIT);
        prog.push(isa::wait(n as u16));
        *pending -= n;
    }
}

/// Compile one channel column into sequencer instructions.
fn compile_channel(song: &Song, c: usize) -> Result<Vec<u16>, String> {
    let mut prog: Vec<u16> = Vec::new();
    let mut pending: u32 = 0;
    let mut want_inst: u8 = 0;
    let mut want_fine: i16 = 0;
    let mut state = Tracked {
        inst: None,
        fine: None,
    };
    let mut loop_label: Option<usize> = None;
    for (r, row) in song.rows.iter().enumerate() {
        if song.loop_row == Some(r) {
            // The pass through the loop must not depend on what ran before it.
            flush_wait(&mut prog, &mut pending);
            loop_label = Some(prog.len());
            state = Tracked {
                inst: None,
                fine: None,
            };
        }
        let cell: Cell = row[c];
        if let Some(i) = cell.inst {
            if usize::from(i) >= song.instruments.len() {
                return Err(format!(
                    "row {r}, channel {}: instrument {i:02X} does not exist",
                    c + 1
                ));
            }
            want_inst = i;
        }
        if let Some(f) = cell.fine {
            want_fine = f.clamp(FINE_MIN, FINE_MAX);
        }
        match cell.note {
            NoteCell::None => {}
            NoteCell::Off => {
                flush_wait(&mut prog, &mut pending);
                prog.push(isa::stop());
            }
            NoteCell::On(n) => {
                if n > MAX_NOTE {
                    return Err(format!("row {r}, channel {}: note {n} too high", c + 1));
                }
                flush_wait(&mut prog, &mut pending);
                if state.inst != Some(want_inst) {
                    prog.push(isa::inst(want_inst));
                    state.inst = Some(want_inst);
                }
                if state.fine != Some(want_fine) {
                    prog.push(isa::fine(want_fine));
                    state.fine = Some(want_fine);
                }
                prog.push(isa::note(n));
            }
        }
        pending += u32::from(song.speed);
    }
    flush_wait(&mut prog, &mut pending);
    match loop_label {
        Some(label) => {
            let disp = label as i64 - (prog.len() as i64 + 1);
            if !(-4096..=4095).contains(&disp) {
                return Err(format!("channel {}: loop jump too long", c + 1));
            }
            prog.push(isa::goto(disp as i16));
        }
        None => prog.push(isa::end(0)),
    }
    Ok(prog)
}

fn has_events(song: &Song, c: usize) -> bool {
    song.rows.iter().any(|r| r[c].note != NoteCell::None)
}

/// Compile a song. `mute` silences channels for preview (their volume is written as 0).
///
/// # Errors
/// Returns a message if the song does not fit the device or refers to missing instruments.
pub fn compile(song: &Song, mute: [bool; CHANNELS]) -> Result<Image, String> {
    if song.instruments.len() > MAX_INSTRUMENTS {
        return Err("too many instruments".into());
    }
    if song.tick == 0 || song.speed == 0 {
        return Err("tick and speed must be at least 1".into());
    }
    let mut usage = Usage::default();
    let mut mem = Mem {
        words: Vec::new(),
        next: 0,
    };

    // Instrument table first, filled in as the data lands.
    let table_bytes = song.instruments.len().max(1) * 16;
    mem.ensure(table_bytes)?;
    mem.next = table_bytes;
    usage.instruments = table_bytes;

    let mut tables: Vec<([u32; 12], usize)> = Vec::new();
    for (i, inst) in song.instruments.iter().enumerate() {
        let pcm = inst.pcm();
        if pcm.is_empty() {
            return Err(format!("instrument {i:02X} has no sample data"));
        }
        if pcm.len() > 0xFFFF {
            return Err(format!("instrument {i:02X} is longer than 65535 bytes"));
        }
        let entries = spit_entries(inst.cycle_bytes());
        let spit = if let Some((_, at)) = tables.iter().find(|(e, _)| *e == entries) {
            *at
        } else {
            mem.align(4);
            let at = mem.next;
            let mut w = Vec::with_capacity(24);
            for e in entries {
                w.push((e >> 16) as u16);
                w.push(e as u16);
            }
            mem.put_words(&w)?;
            usage.pitch_tables += 48;
            tables.push((entries, at));
            at
        };
        let start = mem.put_bytes(&pcm)?;
        usage.samples += pcm.len() + (pcm.len() & 1);
        let loop_start = if inst.loop_start as usize >= pcm.len() {
            0
        } else {
            inst.loop_start
        };
        let base = i * 8;
        let rec = [
            start as u16,
            pcm.len() as u16,
            loop_start,
            spit as u16,
            (u16::from(inst.adsr[0]) << 8) | u16::from(inst.adsr[1]),
            (u16::from(inst.adsr[2]) << 8) | u16::from(inst.adsr[3]),
            if inst.loops() { 0x8000 } else { 0 },
            0,
        ];
        mem.words[base..base + 8].copy_from_slice(&rec);
        // A pitch table or sample may not start beyond a 16-bit byte offset.
        if start > 0xFFFF || spit > 0xFFFF {
            return Err("audio memory layout exceeds 64 KB".into());
        }
    }

    // Sequences.
    let mut seq_pc = [None; CHANNELS];
    for c in 0..CHANNELS {
        if !has_events(song, c) {
            continue;
        }
        let prog = compile_channel(song, c)?;
        let at = mem.put_words(&prog)?;
        usage.sequences[c] = prog.len() * 2;
        seq_pc[c] = Some(RAM_WINDOW + (at / 2) as u16);
    }

    // Registers.
    let mut regs: Vec<(u16, u16)> = Vec::new();
    regs.push((reg::INSTR_BASE, RAM_WINDOW));
    regs.push((reg::MASTER_VOL, u16::from(song.master_vol)));
    let f = &song.filter;
    regs.push((reg::FILTER_CONFIG, u16::from(f.mode)));
    regs.push((reg::FILTER_CUTOFF, u16::from(f.cutoff)));
    regs.push((reg::FILTER_RESONANCE, u16::from(f.resonance)));
    regs.push((
        reg::FILTER_VOL_PAN,
        (u16::from(f.volume) << 8) | u16::from(f.pan),
    ));
    for (c, cfg) in song.channels.iter().enumerate() {
        let mut control = 0u16;
        if seq_pc[c].is_some() {
            control |= reg::CONTROL_SEQ_EN;
        }
        if cfg.modulator.is_some() {
            control |= reg::CONTROL_MOD_MODE;
        }
        if cfg.filter {
            control |= reg::CONTROL_FILTER_EN;
        }
        if cfg.porta > 0 {
            control |= reg::CONTROL_PORTA_EN;
        }
        regs.push((reg::ch(c, reg::CH_CONTROL), control));
        let volume = if mute[c] { 0 } else { cfg.volume };
        regs.push((
            reg::ch(c, reg::CH_VOLUME_PAN),
            (u16::from(volume) << 8) | u16::from(cfg.pan),
        ));
        if let Some(m) = cfg.modulator {
            regs.push((
                reg::ch(c, reg::CH_MOD_ROUTE),
                mod_route(u16::from(m.dest), u16::from(m.target), m.shift, m.invert),
            ));
        }
        if cfg.porta > 0 {
            regs.push((reg::ch(c, reg::CH_PORTA_RATE), cfg.porta));
        }
    }
    regs.push((reg::CONFIG, 0x8000));
    regs.push((reg::SEQ_PERIOD, song.tick));
    for (c, pc) in seq_pc.iter().enumerate() {
        if let Some(pc) = pc {
            regs.push((reg::ch(c, reg::CH_SEQ_PC), *pc));
        }
    }

    Ok(Image {
        words: mem.words,
        instr_base: RAM_WINDOW,
        seq_pc,
        regs,
        usage,
    })
}

impl Image {
    /// Write the image and register setup into a model.
    pub fn load_into(&self, s: &mut Sircas) {
        for (i, w) in self.words.iter().enumerate() {
            s.set_ram_word(i, *w);
        }
        for (r, v) in &self.regs {
            s.reg_write(*r, *v);
        }
    }
}

/// Options for rendering a song to audio.
#[derive(Clone, Copy, Debug)]
pub struct RenderOpts {
    /// Passes through a looping song.
    pub loops: u32,
    /// Seconds of tail after the last pass, for releases to ring out.
    pub tail_seconds: f64,
    pub mute: [bool; CHANNELS],
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self {
            loops: 2,
            tail_seconds: 1.5,
            mute: [false; CHANNELS],
        }
    }
}

/// Result of a render: stereo frames at [`OUTPUT_RATE_HZ`] and the overrun flag.
pub struct Rendered {
    pub frames: Vec<(i16, i16)>,
    pub overrun: bool,
    /// For a looping song, the frame range `[start, end)` that repeats, if `loop_forever` was
    /// set: play the frames, then jump from `end` back to `start`.
    pub wrap: Option<(usize, usize)>,
}

/// Frames in the part of a looping song that repeats (rows from the loop row to the end).
#[must_use]
pub fn loop_pass_frames(song: &Song) -> Option<u64> {
    let l = song.loop_row?;
    Some(song.rows.len().saturating_sub(l) as u64 * u64::from(song.speed) * u64::from(song.tick))
}

/// Render a song through the reference model.
///
/// A looping song plays its first pass, then `opts.loops - 1` more passes of the looped part.
/// With `loop_forever` the tail is dropped and the result carries the range to repeat; the
/// repeated range is the last pass, so the model is in its steady state at its start.
///
/// # Errors
/// Returns the compile error if the song cannot be compiled.
pub fn render(song: &Song, opts: &RenderOpts, loop_forever: bool) -> Result<Rendered, String> {
    let image = compile(song, opts.mute)?;
    let mut s = Sircas::new(Policy {
        capture: true,
        ..Policy::default()
    });
    image.load_into(&mut s);
    let first = song.pass_frames();
    let (mut total, mut wrap) = (first, None);
    if let Some(lp) = loop_pass_frames(song) {
        let passes = u64::from(if loop_forever {
            opts.loops.max(3)
        } else {
            opts.loops.max(1)
        });
        total += (passes - 1) * lp;
        if loop_forever {
            wrap = Some(((total - lp) as usize, total as usize));
        }
    }
    if wrap.is_none() {
        total += (opts.tail_seconds * OUTPUT_RATE_HZ) as u64;
    }
    for _ in 0..total * u64::from(CLKI_PER_FRAME) {
        s.clock();
    }
    let overrun = s.reg_read(reg::STATUS) & reg::STATUS_MEM_OVERRUN != 0;
    Ok(Rendered {
        frames: std::mem::take(&mut s.captured),
        overrun,
        wrap,
    })
}

// ---------------------------------------------------------------------------------------------
// Assembly export
// ---------------------------------------------------------------------------------------------

/// Export an image as SIRC assembly data that a program links and copies into the device.
///
/// The output has two blobs of 16-bit words, written with `.DQ` so each slot holds two
/// consecutive words:
///
/// * `:erence_image`: a word count `n`, then the `n` words of audio memory to copy to the
///   device window from offset `0x8000`.
/// * `:erence_regs`: a count `n`, then `n` pairs of (register offset, value) to write to the
///   device in order.
#[must_use]
pub fn export_asm(song: &Song, image: &Image) -> String {
    let mut s = String::new();
    let _ = writeln!(s, ";; Warning: Exported by ERENCE. Don't edit manually.");
    let _ = writeln!(s, ";; Song: {}", song.title);
    let _ = writeln!(
        s,
        ";; {} ticks of {} samples ({:.2} ms), {} rows of {} ticks, {} audio memory bytes used",
        song.rows.len() as u32 * u32::from(song.speed),
        song.tick,
        song.tick_ms(),
        song.rows.len(),
        song.speed,
        image.usage.total()
    );
    let _ = writeln!(
        s,
        ";; Both tables are 16-bit words packed two to a .DQ slot, so they are copied as\n;; consecutive words. Audio memory is {OUTPUT_RATE_HZ:.3} Hz output, window at offset 0x8000."
    );
    let _ = writeln!(s, ";\n;; Audio memory image");
    let _ = writeln!(s, ":erence_image");
    dq_table(&mut s, image.words.len() as u16, &image.words);
    let flat: Vec<u16> = image.regs.iter().flat_map(|(r, v)| [*r, *v]).collect();
    let _ = writeln!(s, ";\n;; Register setup: (offset, value) pairs, in order");
    let _ = writeln!(s, ":erence_regs");
    dq_table(&mut s, image.regs.len() as u16, &flat);
    s
}

fn dq_table(s: &mut String, count: u16, words: &[u16]) {
    let mut all = Vec::with_capacity(words.len() + 1);
    all.push(count);
    all.extend_from_slice(words);
    if all.len() % 2 == 1 {
        all.push(0);
    }
    for pair in all.chunks(2) {
        let _ = writeln!(s, ".DQ #0x{:04X}{:04X}", pair[0], pair[1]);
    }
}

/// Description of a song's memory use, for display.
#[must_use]
pub fn describe_usage(u: &Usage) -> String {
    format!(
        "{} / {} bytes (instr {}, pitch {}, samples {}, seq {})",
        u.total(),
        MEMORY_BYTES,
        u.instruments,
        u.pitch_tables,
        u.samples,
        u.sequences.iter().sum::<usize>()
    )
}

/// Helper for sample-source instruments: bytes used.
#[must_use]
pub fn sample_bytes(song: &Song) -> usize {
    song.instruments
        .iter()
        .map(|i| match &i.source {
            Source::Sample { data, .. } => data.len(),
            Source::Wave { len, .. } => usize::from(*len),
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::song::{Cell, Instrument, NoteCell, WaveKind};

    fn note(n: u8) -> Cell {
        Cell {
            note: NoteCell::On(n),
            ..Cell::default()
        }
    }

    fn small_song() -> Song {
        let mut s = Song::default();
        s.rows.truncate(8);
        s.instruments = vec![Instrument::wave("Sine", WaveKind::Sine)];
        s
    }

    #[test]
    fn empty_channels_get_no_sequence() {
        let mut s = small_song();
        s.rows[0][0] = note(39);
        let img = compile(&s, [false; CHANNELS]).unwrap();
        assert!(img.seq_pc[0].is_some());
        assert!(img.seq_pc[1..].iter().all(Option::is_none));
    }

    #[test]
    fn column_compiles_to_instructions_with_merged_waits() {
        let mut s = small_song();
        s.speed = 6;
        s.loop_row = None;
        s.rows[0][0] = note(39);
        s.rows[3][0].note = NoteCell::Off;
        let prog = compile_channel(&s, 0).unwrap();
        assert_eq!(
            prog,
            vec![
                isa::inst(0),
                isa::fine(0),
                isa::note(39),
                isa::wait(18), // rows 0..2
                isa::stop(),
                isa::wait(30), // rows 3..7
                isa::end(0),
            ]
        );
    }

    #[test]
    fn long_waits_are_split() {
        let mut s = small_song();
        s.rows = vec![[Cell::default(); CHANNELS]; 4];
        s.rows[0][0] = note(10);
        s.speed = 5000;
        s.loop_row = None;
        let prog = compile_channel(&s, 0).unwrap();
        assert_eq!(
            prog[3..],
            [
                isa::wait(8191),
                isa::wait(8191),
                isa::wait(3618),
                isa::end(0)
            ]
        );
    }

    #[test]
    fn looping_jumps_back_and_restates_instrument() {
        let mut s = small_song();
        s.loop_row = Some(2);
        s.rows[0][0] = note(10);
        s.rows[2][0] = note(20);
        let prog = compile_channel(&s, 0).unwrap();
        // inst fine note | wait | inst fine note wait goto
        let goto = *prog.last().unwrap();
        assert_eq!(goto >> 13, 5);
        // the loop label is the explicit INST before the second note
        let label = prog.iter().rposition(|w| *w == isa::inst(0)).unwrap();
        let disp = (goto & 0x1FFF) as i16;
        let disp = (disp << 3) >> 3;
        assert_eq!(label as i64, prog.len() as i64 + i64::from(disp));
    }

    #[test]
    fn unknown_instrument_is_an_error() {
        let mut s = small_song();
        s.rows[0][0] = Cell {
            note: NoteCell::On(10),
            inst: Some(9),
            fine: None,
        };
        assert!(compile(&s, [false; CHANNELS]).is_err());
    }

    #[test]
    fn identical_pitch_tables_are_shared() {
        let mut s = small_song();
        s.instruments.push(Instrument::wave("Saw", WaveKind::Saw));
        s.rows[0][0] = note(10);
        let img = compile(&s, [false; CHANNELS]).unwrap();
        assert_eq!(img.usage.pitch_tables, 48);
    }

    #[test]
    fn full_memory_is_reported() {
        let mut s = small_song();
        if let Source::Wave { len, .. } = &mut s.instruments[0].source {
            *len = 40000;
        }
        let mut second = s.instruments[0].clone();
        second.name = "b".into();
        s.instruments.push(second);
        s.rows[0][0] = note(10);
        assert!(compile(&s, [false; CHANNELS]).unwrap_err().contains("full"));
    }

    #[test]
    fn rendered_song_makes_the_expected_pitch() {
        let mut s = small_song();
        s.master_vol = 255;
        s.channels[0].volume = 255;
        s.loop_row = None;
        s.rows[0][0] = note(39 + 12); // C5, 523.25 Hz
        s.speed = 20;
        let r = render(&s, &RenderOpts::default(), false).unwrap();
        assert!(!r.overrun);
        // Count zero crossings between 50 ms and 450 ms (the first tick delays the note).
        let (a, b) = (
            (OUTPUT_RATE_HZ * 0.05) as usize,
            (OUTPUT_RATE_HZ * 0.45) as usize,
        );
        let left: Vec<i16> = r.frames[a..b].iter().map(|f| f.0).collect();
        assert!(left.iter().any(|v| v.abs() > 1000), "silent");
        let crossings = left.windows(2).filter(|w| w[0] <= 0 && w[1] > 0).count();
        let hz = crossings as f64 / 0.4;
        assert!((hz - 523.25).abs() < 8.0, "pitch {hz}");
    }

    #[test]
    fn export_has_both_tables_and_round_trips_words() {
        let mut s = small_song();
        s.rows[0][0] = note(39);
        let img = compile(&s, [false; CHANNELS]).unwrap();
        let asm = export_asm(&s, &img);
        assert!(asm.contains(":erence_image"));
        assert!(asm.contains(":erence_regs"));
        // Parse the .DQ slots back and compare.
        let mut words = Vec::new();
        let mut tables = Vec::new();
        let mut cur: Option<Vec<u16>> = None;
        for line in asm.lines() {
            if line.starts_with(':') {
                if let Some(c) = cur.take() {
                    tables.push(c);
                }
                cur = Some(Vec::new());
            } else if let Some(v) = line.strip_prefix(".DQ #0x") {
                let v = u32::from_str_radix(v, 16).unwrap();
                cur.as_mut().unwrap().extend([(v >> 16) as u16, v as u16]);
            }
        }
        tables.push(cur.unwrap());
        let image_table = &tables[0];
        assert_eq!(usize::from(image_table[0]), img.words.len());
        words.extend_from_slice(&image_table[1..=img.words.len()]);
        assert_eq!(words, img.words);
        let regs_table = &tables[1];
        assert_eq!(usize::from(regs_table[0]), img.regs.len());
        assert_eq!(regs_table[1], img.regs[0].0);
        assert_eq!(regs_table[2], img.regs[0].1);
    }
}
