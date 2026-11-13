//! Clock-accurate behavioural model of SIRCAS, independent of the SIRC bus.
//!
//! One call to [`Sircas::clock`] is one `CLKI` cycle. Everything that the draft specification
//! calls provisional or open is resolved here by an explicit, documented choice, so that the
//! tests and study programs have something deterministic to measure. Each choice is listed in
//! `docs/audio/STATUS.md`.

use crate::filter::{FilterMode, SvfConfig, SvfFixed};
use crate::{
    CLKI_PER_FRAME, ENVELOPE_PERIOD, NUM_CHANNELS, RAM_WORDS, SCHEDULE_CLOCKS, SLOT_CLOCKS,
};

// ---------------------------------------------------------------------------------------------
// Policy: the choices the draft leaves open, switchable so the studies can compare them.
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinePolicy {
    /// `SPI = (entry << octave) + FINE` (the draft's literal reading: raw units on the final SPI).
    AfterOctaveShift,
    /// `SPI = (entry + FINE) << octave` (detune scales with the octave).
    BeforeOctaveShift,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PitchModLaw {
    /// `delta = M << (S + 4)` raw 16.16 units (the draft).
    Additive,
    /// `delta = SPI * M * 2^S / 2^16` (depth is a constant fraction of the pitch).
    Proportional,
}

#[derive(Clone, Copy, Debug)]
pub struct Policy {
    pub fine: FinePolicy,
    pub pitch_mod: PitchModLaw,
    pub filter: SvfConfig,
    /// Skip the instrument reload when a `NOTE` uses the instrument already loaded.
    pub instrument_cache: bool,
    /// Keep every captured output frame (needed for rendering; costs memory).
    pub capture: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            fine: FinePolicy::AfterOctaveShift,
            pitch_mod: PitchModLaw::Additive,
            filter: SvfConfig::SPEC,
            instrument_cache: true,
            capture: false,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Register map
// ---------------------------------------------------------------------------------------------

pub mod reg {
    pub const CONFIG: u16 = 0x0000;
    pub const STATUS: u16 = 0x0001;
    pub const STATUSCLR: u16 = 0x0002;
    pub const SEQ_PERIOD: u16 = 0x0003;
    pub const SEQ_DONE: u16 = 0x0004;
    pub const SEQ_INT_EN: u16 = 0x0005;
    pub const INSTR_BASE: u16 = 0x0006;
    pub const MASTER_VOL: u16 = 0x0007;
    pub const FILTER_CONFIG: u16 = 0x0008;
    pub const FILTER_CUTOFF: u16 = 0x0009;
    pub const FILTER_RESONANCE: u16 = 0x000A;
    pub const FILTER_VOL_PAN: u16 = 0x000B;
    pub const CH_BASE: u16 = 0x0010;
    pub const CH_CONTROL: u16 = 0;
    pub const CH_VOLUME_PAN: u16 = 1;
    pub const CH_MOD_ROUTE: u16 = 2;
    pub const CH_INSTR: u16 = 3;
    pub const CH_FINE: u16 = 4;
    pub const CH_PORTA_RATE: u16 = 5;
    pub const CH_COMMAND: u16 = 6;
    pub const CH_SEQ_PC: u16 = 7;

    #[must_use]
    pub const fn ch(channel: usize, offset: u16) -> u16 {
        CH_BASE + ((channel as u16) << 4) + offset
    }

    pub const CONTROL_SEQ_EN: u16 = 1 << 15;
    pub const CONTROL_MOD_MODE: u16 = 1 << 14;
    pub const CONTROL_FILTER_EN: u16 = 1 << 13;
    pub const CONTROL_PORTA_EN: u16 = 1 << 12;
    pub const STATUS_VERSION: u16 = 1;
    pub const STATUS_SEQ_IRQ: u16 = 1 << 4;
    pub const STATUS_MEM_OVERRUN: u16 = 1 << 5;
}

/// Modulation destinations (`MOD_ROUTE[15:13]`).
pub mod moddest {
    pub const OFF: u16 = 0;
    pub const PITCH: u16 = 1;
    pub const VOLUME: u16 = 2;
    pub const PAN: u16 = 3;
    pub const FILTER_CUTOFF: u16 = 4;
    pub const MASTER_VOLUME: u16 = 5;
    pub const FILTER_PAN: u16 = 6;
}

/// Encode a `MOD_ROUTE` register value.
#[must_use]
pub fn mod_route(dest: u16, channel: u16, shift: i8, invert: bool) -> u16 {
    (dest << 13) | ((channel & 7) << 10) | (((shift as u16) & 0xF) << 6) | (u16::from(invert) << 5)
}

// ---------------------------------------------------------------------------------------------
// Internal state
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EnvPhase {
    #[default]
    Off,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone, Copy, Debug, Default)]
struct InstrData {
    start: u16,
    len: u16,
    loop_start: u16,
    spit: u16,
    attack: u8,
    decay: u8,
    sustain: u8,
    release: u8,
    loop_en: bool,
}

#[derive(Clone, Copy, Debug)]
enum LoadKind {
    Full,
    PitchOnly,
}

#[derive(Clone, Copy, Debug)]
struct Load {
    kind: LoadKind,
    note: u8,
    instr: u8,
    idx: u8,
    words: [u16; 7],
    spit: [u16; 2],
}

#[derive(Clone, Copy, Debug)]
enum Ctl {
    Idle,
    Fetch,
    Load(Load),
}

#[derive(Clone, Debug)]
struct Channel {
    // CPU-visible registers.
    control: u16,
    volume: u8,
    pan: u8,
    mod_route: u16,
    instr: u8,
    fine: i16,
    porta_rate: u16,
    pc: u16,
    // Sequencer.
    halted: bool,
    wait: u16,
    end_count: u8,
    budget: u8,
    ctl: Ctl,
    cmd_pending: Option<u16>,
    tick_clk: u64,
    // Loaded instrument and playback.
    loaded: InstrData,
    loaded_index: Option<u8>,
    playing: bool,
    spa: u32,
    spi_current: u32,
    spi_target: u32,
    // Envelope.
    env_level: u16,
    env_phase: EnvPhase,
    gain_dirty: bool,
    // PCM fetch pipeline.
    cache: Option<(u16, u16)>,
    pcm_job: Option<(u16, bool)>,
    cur_byte: i8,
    cur_valid: bool,
}

impl Channel {
    fn new() -> Self {
        Self {
            control: 0,
            volume: 0,
            pan: 128,
            mod_route: 0,
            instr: 0,
            fine: 0,
            porta_rate: 0,
            pc: 0x8000,
            halted: false,
            wait: 0,
            end_count: 0,
            budget: 0,
            ctl: Ctl::Idle,
            cmd_pending: None,
            tick_clk: 0,
            loaded: InstrData::default(),
            loaded_index: None,
            playing: false,
            spa: 0,
            spi_current: 0,
            spi_target: 0,
            env_level: 0,
            env_phase: EnvPhase::Off,
            gain_dirty: false,
            cache: None,
            pcm_job: None,
            cur_byte: 0,
            cur_valid: false,
        }
    }
    const fn seq_en(&self) -> bool {
        self.control & reg::CONTROL_SEQ_EN != 0
    }
    const fn mod_mode(&self) -> bool {
        self.control & reg::CONTROL_MOD_MODE != 0
    }
    const fn filter_en(&self) -> bool {
        self.control & reg::CONTROL_FILTER_EN != 0
    }
    const fn porta_en(&self) -> bool {
        self.control & reg::CONTROL_PORTA_EN != 0
    }
}

/// Modulation contributions in force for the current frame (one frame behind their sources).
#[derive(Clone, Copy, Debug, Default)]
struct ModApply {
    pitch: [i32; NUM_CHANNELS],
    volume: [i32; NUM_CHANNELS],
    pan: [i32; NUM_CHANNELS],
    cutoff: i32,
    master: i32,
    filter_pan: i32,
}

/// Counters the studies read out.
#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub frames: u64,
    pub pcm_reads: u64,
    pub ctl_reads: u64,
    pub idle_audio_slots: u64,
    pub cpu_transactions: u64,
    /// PCM fetches that had not completed at the frame boundary.
    pub pcm_misses: u64,
    /// Sequencer ticks dropped because the channel was still busy with the previous tick.
    pub dropped_ticks: u64,
    pub overruns: u64,
    pub dsp_clocks_max: u32,
    pub dsp_clocks_last: u32,
    pub dsp_overruns: u64,
    /// Longest time from a sequencer tick to a `NOTE` taking effect, in CLKI cycles.
    pub note_latency_max_clocks: u64,
    pub max_audio_slots_in_frame: u32,
    pub filter_saturations: u64,
}

/// A request from the CPU-facing port.
#[derive(Clone, Copy, Debug)]
pub struct CpuReq {
    pub write: bool,
    /// RAM word index, `0..0x8000`.
    pub word: u16,
    pub data: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ChannelSnapshot {
    pub env_level: u16,
    pub env_phase: EnvPhase,
    pub spi: u32,
    pub spi_target: u32,
    pub spa: u32,
    pub pc: u16,
    pub halted: bool,
    pub playing: bool,
    pub wait: u16,
    pub end_count: u8,
}

pub struct Sircas {
    pub policy: Policy,
    ram: Vec<u16>,
    chans: Vec<Channel>,
    clk: u64,
    frame: u64,
    rr: usize,
    // Global registers.
    audio_en: bool,
    seq_period: u16,
    seq_counter: u32,
    seq_done: u8,
    seq_int_en: u8,
    instr_base: u16,
    master_vol: u8,
    filter_cfg: u16,
    filter_cutoff: u8,
    filter_res: u8,
    filter_vol: u8,
    filter_pan: u8,
    mem_overrun: bool,
    // Filter and modulation.
    svf: SvfFixed,
    mod_apply: ModApply,
    // CPU port.
    cpu_pending: Option<CpuReq>,
    cpu_inflight: Option<(u64, u16)>,
    cpu_result: Option<u16>,
    // Output.
    audio_slots_this_frame: u32,
    out_latch: (i16, i16),
    pub last_out: (i16, i16),
    pub captured: Vec<(i16, i16)>,
    pub stats: Stats,
}

fn xorshift(state: &mut u32) -> u32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    x
}

const fn sat_i(x: i64, bits: u32) -> i64 {
    let max = (1i64 << (bits - 1)) - 1;
    let min = -(1i64 << (bits - 1));
    if x > max {
        max
    } else if x < min {
        min
    } else {
        x
    }
}

/// `value << (shift)` for positive, arithmetic shift right for negative.
const fn shl_signed(value: i64, shift: i32) -> i64 {
    if shift >= 0 {
        value << shift
    } else {
        value >> -shift
    }
}

impl Sircas {
    #[must_use]
    pub fn new(policy: Policy) -> Self {
        // External SRAM is undefined at power-up: fill with noise so tests can't rely on zeros.
        let mut seed = 0x1234_5678u32;
        let mut s = Self::blank(policy);
        s.ram = (0..RAM_WORDS).map(|_| xorshift(&mut seed) as u16).collect();
        s
    }

    /// Reset everything except RAM (draft section 15).
    pub fn reset_registers(&mut self) {
        let policy = self.policy;
        let ram = std::mem::take(&mut self.ram);
        let captured = std::mem::take(&mut self.captured);
        *self = Self {
            ram,
            captured,
            ..Self::blank(policy)
        };
    }

    fn blank(policy: Policy) -> Self {
        Self {
            policy,
            ram: Vec::new(),
            chans: (0..NUM_CHANNELS).map(|_| Channel::new()).collect(),
            clk: 0,
            frame: 0,
            rr: 0,
            audio_en: false,
            seq_period: 0,
            seq_counter: 0,
            seq_done: 0,
            seq_int_en: 0,
            instr_base: 0x8000,
            master_vol: 255,
            filter_cfg: 0,
            filter_cutoff: 0,
            filter_res: 0,
            filter_vol: 0,
            filter_pan: 128,
            mem_overrun: false,
            svf: SvfFixed::new(policy.filter),
            mod_apply: ModApply::default(),
            cpu_pending: None,
            cpu_inflight: None,
            cpu_result: None,
            audio_slots_this_frame: 0,
            out_latch: (0, 0),
            last_out: (0, 0),
            captured: Vec::new(),
            stats: Stats::default(),
        }
    }

    // -----------------------------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------------------------

    #[must_use]
    pub const fn clk(&self) -> u64 {
        self.clk
    }
    #[must_use]
    pub const fn frame(&self) -> u64 {
        self.frame
    }
    #[must_use]
    pub const fn mem_overrun(&self) -> bool {
        self.mem_overrun
    }
    /// Open-drain IRQ3 level (true = asserted).
    #[must_use]
    pub const fn irq3(&self) -> bool {
        (self.seq_done & self.seq_int_en) != 0
    }

    /// Backdoor RAM access by 16-bit word index (no timing).
    #[must_use]
    pub fn ram_word(&self, index: usize) -> u16 {
        self.ram[index & 0x7FFF]
    }
    pub fn set_ram_word(&mut self, index: usize, value: u16) {
        self.ram[index & 0x7FFF] = value;
    }
    /// Backdoor byte write using the MSB-first packing of the draft (even byte offset = high byte).
    pub fn set_ram_byte(&mut self, byte_offset: usize, value: u8) {
        let w = &mut self.ram[(byte_offset >> 1) & 0x7FFF];
        if byte_offset & 1 == 0 {
            *w = (*w & 0x00FF) | (u16::from(value) << 8);
        } else {
            *w = (*w & 0xFF00) | u16::from(value);
        }
    }
    #[must_use]
    pub fn ram_image(&self) -> &[u16] {
        &self.ram
    }

    /// Modulation contributions currently in force (for tests).
    #[must_use]
    pub fn mod_pitch(&self, c: usize) -> i32 {
        self.mod_apply.pitch[c]
    }
    #[must_use]
    pub fn mod_volume(&self, c: usize) -> i32 {
        self.mod_apply.volume[c]
    }
    #[must_use]
    pub fn mod_cutoff(&self) -> i32 {
        self.mod_apply.cutoff
    }

    #[must_use]
    pub fn snapshot(&self, c: usize) -> ChannelSnapshot {
        let ch = &self.chans[c];
        ChannelSnapshot {
            env_level: ch.env_level,
            env_phase: ch.env_phase,
            spi: ch.spi_current,
            spi_target: ch.spi_target,
            spa: ch.spa,
            pc: ch.pc,
            halted: ch.halted,
            playing: ch.playing,
            wait: ch.wait,
            end_count: ch.end_count,
        }
    }

    // -----------------------------------------------------------------------------------------
    // Register interface
    // -----------------------------------------------------------------------------------------

    #[must_use]
    pub fn reg_read(&self, word: u16) -> u16 {
        match word {
            reg::CONFIG => u16::from(self.audio_en) << 15,
            reg::STATUS => {
                reg::STATUS_VERSION
                    | if self.irq3() { reg::STATUS_SEQ_IRQ } else { 0 }
                    | if self.mem_overrun {
                        reg::STATUS_MEM_OVERRUN
                    } else {
                        0
                    }
            }
            reg::SEQ_PERIOD => self.seq_period,
            reg::SEQ_DONE => u16::from(self.seq_done),
            reg::SEQ_INT_EN => u16::from(self.seq_int_en),
            reg::INSTR_BASE => self.instr_base,
            reg::MASTER_VOL => u16::from(self.master_vol),
            reg::FILTER_CONFIG => self.filter_cfg,
            reg::FILTER_CUTOFF => u16::from(self.filter_cutoff),
            reg::FILTER_RESONANCE => u16::from(self.filter_res),
            reg::FILTER_VOL_PAN => (u16::from(self.filter_vol) << 8) | u16::from(self.filter_pan),
            0x0010..=0x006F => {
                let c = ((word - reg::CH_BASE) >> 4) as usize;
                let ch = &self.chans[c];
                match (word - reg::CH_BASE) & 0xF {
                    reg::CH_CONTROL => ch.control,
                    reg::CH_VOLUME_PAN => (u16::from(ch.volume) << 8) | u16::from(ch.pan),
                    reg::CH_MOD_ROUTE => ch.mod_route,
                    reg::CH_INSTR => u16::from(ch.instr),
                    reg::CH_FINE => ch.fine as u16,
                    reg::CH_PORTA_RATE => ch.porta_rate,
                    reg::CH_SEQ_PC => ch.pc,
                    _ => 0,
                }
            }
            _ => 0,
        }
    }

    pub fn reg_write(&mut self, word: u16, value: u16) {
        match word {
            reg::CONFIG => self.audio_en = value & 0x8000 != 0,
            reg::STATUSCLR => {
                if value & reg::STATUS_MEM_OVERRUN != 0 {
                    self.mem_overrun = false;
                }
            }
            reg::SEQ_PERIOD => {
                self.seq_period = value;
                self.seq_counter = 0;
            }
            reg::SEQ_DONE => self.seq_done &= !(value as u8 & 0x3F),
            reg::SEQ_INT_EN => self.seq_int_en = value as u8 & 0x3F,
            reg::INSTR_BASE => self.instr_base = value,
            reg::MASTER_VOL => self.master_vol = value as u8,
            reg::FILTER_CONFIG => self.filter_cfg = value & 3,
            reg::FILTER_CUTOFF => self.filter_cutoff = value as u8,
            reg::FILTER_RESONANCE => self.filter_res = value as u8,
            reg::FILTER_VOL_PAN => {
                self.filter_vol = (value >> 8) as u8;
                self.filter_pan = value as u8;
            }
            0x0010..=0x006F => {
                let c = ((word - reg::CH_BASE) >> 4) as usize;
                self.channel_write(c, (word - reg::CH_BASE) & 0xF, value);
            }
            _ => {}
        }
    }

    fn channel_write(&mut self, c: usize, offset: u16, value: u16) {
        let ch = &mut self.chans[c];
        match offset {
            reg::CH_CONTROL => ch.control = value & 0xF000,
            reg::CH_VOLUME_PAN => {
                ch.volume = (value >> 8) as u8;
                ch.pan = value as u8;
                ch.gain_dirty = true;
            }
            reg::CH_MOD_ROUTE => ch.mod_route = value & 0xFFE0,
            reg::CH_INSTR => {
                if !ch.seq_en() {
                    ch.instr = value as u8;
                }
            }
            reg::CH_FINE => {
                if !ch.seq_en() {
                    // Sign-extend from 13 bits.
                    ch.fine = ((value << 3) as i16) >> 3;
                }
            }
            reg::CH_PORTA_RATE => ch.porta_rate = value,
            reg::CH_COMMAND => {
                if !ch.seq_en() {
                    // Single-entry pending register, last write wins.
                    ch.cmd_pending = Some(value);
                }
            }
            reg::CH_SEQ_PC => {
                // Restart: abort any in-flight fetch/load, clear WAIT, END counter and DONE.
                ch.pc = value | 0x8000;
                ch.wait = 0;
                ch.end_count = 0;
                ch.halted = false;
                ch.ctl = Ctl::Idle;
                self.seq_done &= !(1 << c);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------------------------
    // CPU-facing RAM port
    // -----------------------------------------------------------------------------------------

    /// Queue a CPU RAM request. Returns false if one is already outstanding.
    pub fn cpu_request(&mut self, req: CpuReq) -> bool {
        if self.cpu_pending.is_some() || self.cpu_inflight.is_some() {
            return false;
        }
        self.cpu_pending = Some(req);
        true
    }

    /// Completed read data (or write acknowledge) if the current CPU transaction has finished.
    pub fn cpu_take_result(&mut self) -> Option<u16> {
        self.cpu_result.take()
    }

    // -----------------------------------------------------------------------------------------
    // Clocking
    // -----------------------------------------------------------------------------------------

    /// Advance one `CLKI` cycle.
    pub fn clock(&mut self) {
        let in_frame = (self.clk % u64::from(CLKI_PER_FRAME)) as u32;
        if in_frame == 0 {
            self.frame_boundary();
        }
        let phase = (self.clk % u64::from(SCHEDULE_CLOCKS)) as u32;
        if phase.is_multiple_of(SLOT_CLOCKS) {
            let slot = phase / SLOT_CLOCKS;
            if slot == 0 {
                self.audio_slot();
            } else if let Some(req) = self.cpu_pending.take() {
                // Perform the access at slot start; BACK follows when the two clocks elapse.
                let data = if req.write {
                    self.ram[req.word as usize & 0x7FFF] = req.data;
                    0
                } else {
                    self.ram[req.word as usize & 0x7FFF]
                };
                self.stats.cpu_transactions += 1;
                self.cpu_inflight = Some((self.clk + u64::from(SLOT_CLOCKS), data));
            }
        }
        self.clk += 1;
        if let Some((done, data)) = self.cpu_inflight {
            if self.clk >= done {
                self.cpu_inflight = None;
                self.cpu_result = Some(data);
            }
        }
    }

    // -----------------------------------------------------------------------------------------
    // Audio-engine memory slot
    // -----------------------------------------------------------------------------------------

    fn audio_slot(&mut self) {
        for c in 0..NUM_CHANNELS {
            self.maybe_start_direct(c);
        }
        // PCM reads have strict priority.
        for c in 0..NUM_CHANNELS {
            if let Some((word, hi)) = self.chans[c].pcm_job.take() {
                let data = self.ram[word as usize & 0x7FFF];
                let ch = &mut self.chans[c];
                ch.cache = Some((word, data));
                ch.cur_byte = (if hi { data >> 8 } else { data & 0xFF }) as u8 as i8;
                ch.cur_valid = true;
                self.stats.pcm_reads += 1;
                self.audio_slots_this_frame += 1;
                return;
            }
        }
        for i in 0..NUM_CHANNELS {
            let c = (self.rr + i) % NUM_CHANNELS;
            if !matches!(self.chans[c].ctl, Ctl::Idle) {
                self.rr = (c + 1) % NUM_CHANNELS;
                self.ctl_step(c);
                self.stats.ctl_reads += 1;
                self.audio_slots_this_frame += 1;
                return;
            }
        }
        self.stats.idle_audio_slots += 1;
    }

    fn cpu_word_to_ram(addr: u16) -> usize {
        (addr & 0x7FFF) as usize
    }

    fn maybe_start_direct(&mut self, c: usize) {
        let ch = &mut self.chans[c];
        if !matches!(ch.ctl, Ctl::Idle) {
            return;
        }
        if ch.seq_en() {
            ch.cmd_pending = None;
            return;
        }
        if let Some(cmd) = ch.cmd_pending.take() {
            ch.tick_clk = self.clk;
            if cmd & 0x8000 == 0 {
                self.stop_channel(c);
            } else {
                self.begin_note(c, (cmd & 0x7F) as u8);
            }
        }
    }

    fn stop_channel(&mut self, c: usize) {
        let ch = &mut self.chans[c];
        if ch.env_phase != EnvPhase::Off {
            ch.env_phase = EnvPhase::Release;
        }
    }

    fn begin_note(&mut self, c: usize, note: u8) {
        if note > 119 {
            return; // invalid note numbers are ignored
        }
        let cache = self.policy.instrument_cache;
        let ch = &mut self.chans[c];
        let same = ch.loaded_index == Some(ch.instr);
        let (kind, idx) = if same && cache {
            (LoadKind::PitchOnly, 7)
        } else {
            (LoadKind::Full, 0)
        };
        ch.ctl = Ctl::Load(Load {
            kind,
            note,
            instr: ch.instr,
            idx,
            words: [0; 7],
            spit: [0; 2],
        });
    }

    /// Perform one RAM read on behalf of channel `c`'s control state machine.
    fn ctl_step(&mut self, c: usize) {
        match self.chans[c].ctl {
            Ctl::Idle => {}
            Ctl::Fetch => {
                let pc = self.chans[c].pc;
                let word = self.ram[Self::cpu_word_to_ram(pc)];
                let ch = &mut self.chans[c];
                ch.pc = 0x8000 | (pc.wrapping_add(1) & 0x7FFF);
                ch.budget = ch.budget.saturating_sub(1);
                self.exec(c, word);
            }
            Ctl::Load(mut l) => {
                if l.idx < 7 {
                    let addr = self
                        .instr_base
                        .wrapping_add(u16::from(l.instr) * 8 + u16::from(l.idx));
                    l.words[l.idx as usize] = self.ram[Self::cpu_word_to_ram(addr)];
                    l.idx += 1;
                    self.chans[c].ctl = Ctl::Load(l);
                } else {
                    let spit = match l.kind {
                        LoadKind::Full => l.words[3],
                        LoadKind::PitchOnly => self.chans[c].loaded.spit,
                    };
                    let byte = u32::from(spit) + 4 * u32::from(l.note % 12);
                    let which = (l.idx - 7) as usize;
                    let word_index = ((byte >> 1) as usize + which) & 0x7FFF;
                    l.spit[which] = self.ram[word_index];
                    l.idx += 1;
                    if l.idx == 9 {
                        self.commit_note(c, &l);
                    } else {
                        self.chans[c].ctl = Ctl::Load(l);
                    }
                }
            }
        }
    }

    fn after_instruction(&mut self, c: usize) {
        let ch = &mut self.chans[c];
        ch.ctl = if ch.budget > 0 && !ch.halted && ch.wait == 0 {
            Ctl::Fetch
        } else {
            Ctl::Idle
        };
    }

    fn exec(&mut self, c: usize, word: u16) {
        let op = word >> 13;
        let operand = word & 0x1FFF;
        match op {
            0 => {
                // NOTE: continue the instruction stream after the load completes.
                self.chans[c].ctl = Ctl::Idle;
                self.begin_note(c, (operand & 0x7F) as u8);
                if matches!(self.chans[c].ctl, Ctl::Idle) {
                    self.after_instruction(c);
                }
            }
            1 => {
                self.stop_channel(c);
                self.after_instruction(c);
            }
            2 => {
                self.chans[c].wait = operand;
                self.after_instruction(c);
            }
            3 => {
                self.chans[c].instr = operand as u8;
                self.after_instruction(c);
            }
            4 => {
                self.chans[c].fine = ((operand << 3) as i16) >> 3;
                self.after_instruction(c);
            }
            5 => {
                let disp = i32::from(((operand << 3) as i16) >> 3);
                let ch = &mut self.chans[c];
                ch.pc = 0x8000 | ((i32::from(ch.pc & 0x7FFF) + disp) & 0x7FFF) as u16;
                self.after_instruction(c);
            }
            6 => {
                let n = (operand & 0xFF) as u8;
                let ch = &mut self.chans[c];
                let mut halt = n == 0;
                if n != 0 {
                    ch.end_count = ch.end_count.saturating_add(1);
                    // Comparison rule used by the prototype: halt once the shared counter has
                    // reached (>=) the operand.
                    halt = ch.end_count >= n;
                }
                if halt {
                    ch.halted = true;
                    self.seq_done |= 1 << c;
                }
                self.after_instruction(c);
            }
            _ => self.after_instruction(c), // reserved opcode: no operation
        }
    }

    fn commit_note(&mut self, c: usize, l: &Load) {
        let clk = self.clk;
        let fine_policy = self.policy.fine;
        let ch = &mut self.chans[c];

        if let LoadKind::Full = l.kind {
            let w = l.words;
            ch.loaded = InstrData {
                start: w[0],
                len: w[1],
                loop_start: w[2],
                spit: w[3],
                attack: (w[4] >> 8) as u8,
                decay: w[4] as u8,
                sustain: (w[5] >> 8) as u8,
                release: w[5] as u8,
                loop_en: w[6] & 0x8000 != 0,
            };
        }
        let same_instrument = ch.loaded_index == Some(l.instr);
        ch.loaded_index = Some(l.instr);

        let entry = (u64::from(l.spit[0]) << 16) | u64::from(l.spit[1]);
        let oct = u32::from(l.note / 12);
        let fine = i64::from(ch.fine);
        let spi = match fine_policy {
            FinePolicy::AfterOctaveShift => (entry << oct) as i64 + fine,
            FinePolicy::BeforeOctaveShift => (entry as i64 + fine) << oct,
        };
        let spi = spi.clamp(0, i64::from(u32::MAX)) as u32;

        let sounding = ch.env_phase != EnvPhase::Off && ch.playing;
        let mut retriggered = false;
        if ch.porta_en() && same_instrument && sounding {
            ch.spi_target = spi; // glide: no retrigger
        } else {
            ch.spi_current = spi;
            ch.spi_target = spi;
            ch.spa = 0;
            ch.playing = ch.loaded.len != 0;
            ch.env_level = 0;
            ch.env_phase = EnvPhase::Attack;
            ch.gain_dirty = true;
            ch.cache = None;
            retriggered = true;
        }
        let latency = clk.saturating_sub(ch.tick_clk);
        if latency > self.stats.note_latency_max_clocks {
            self.stats.note_latency_max_clocks = latency;
        }
        if retriggered {
            // The first envelope step happens at note-on, so the sample is never played into a
            // silent envelope (otherwise up to 64 frames of attack transient would be lost).
            self.envelope_update(c);
        }
        // Continue the instruction stream.
        self.chans[c].ctl = Ctl::Idle;
        if self.chans[c].seq_en() {
            self.after_instruction(c);
        }
    }

    // -----------------------------------------------------------------------------------------
    // Frame boundary: DSP, envelopes, tick, playback advance
    // -----------------------------------------------------------------------------------------

    fn frame_boundary(&mut self) {
        // Anything still outstanding at the boundary is a missed deadline.
        for c in 0..NUM_CHANNELS {
            if self.chans[c].pcm_job.take().is_some() {
                self.stats.pcm_misses += 1;
                self.mem_overrun = true;
                self.stats.overruns += 1;
            }
        }
        self.stats.max_audio_slots_in_frame = self
            .stats
            .max_audio_slots_in_frame
            .max(self.audio_slots_this_frame);
        self.audio_slots_this_frame = 0;

        if self.frame > 0 {
            self.dsp_frame();
        }
        self.frame += 1;
        self.stats.frames += 1;

        // Staggered envelope updates: channel c updates when (frame + c*64/6) % 64 == 0.
        for c in 0..NUM_CHANNELS {
            let phase = (c as u64 * u64::from(ENVELOPE_PERIOD)) / NUM_CHANNELS as u64;
            if (self.frame + phase).is_multiple_of(u64::from(ENVELOPE_PERIOD)) {
                self.envelope_update(c);
            }
        }

        // Global sequencer tick.
        if self.seq_period != 0 {
            self.seq_counter += 1;
            if self.seq_counter >= u32::from(self.seq_period) {
                self.seq_counter = 0;
                self.fire_tick();
            }
        } else {
            self.seq_counter = 0;
        }

        self.advance_playback();
    }

    fn fire_tick(&mut self) {
        let clk = self.clk;
        for c in 0..NUM_CHANNELS {
            // Portamento runs on every tick whatever the sequencer is doing.
            let ch = &mut self.chans[c];
            if ch.porta_rate != 0 && ch.spi_current != ch.spi_target {
                let rate = u32::from(ch.porta_rate);
                ch.spi_current = if ch.spi_current < ch.spi_target {
                    (ch.spi_current.saturating_add(rate)).min(ch.spi_target)
                } else {
                    ch.spi_current.saturating_sub(rate).max(ch.spi_target)
                };
            }
            if !ch.seq_en() || ch.halted {
                continue;
            }
            if !matches!(ch.ctl, Ctl::Idle) {
                // Previous tick's work has not finished: deadline missed.
                self.stats.dropped_ticks += 1;
                self.stats.overruns += 1;
                self.mem_overrun = true;
                continue;
            }
            if ch.wait > 0 {
                ch.wait -= 1;
                if ch.wait > 0 {
                    continue;
                }
            }
            ch.budget = 8;
            ch.tick_clk = clk;
            ch.ctl = Ctl::Fetch;
        }
    }

    fn envelope_update(&mut self, c: usize) {
        let ch = &mut self.chans[c];
        let ins = ch.loaded;
        let sustain12 = (u16::from(ins.sustain) << 4) | (u16::from(ins.sustain) >> 4);
        let before = ch.env_level;
        match ch.env_phase {
            EnvPhase::Off | EnvPhase::Sustain => {}
            EnvPhase::Attack => {
                if ins.attack == 0 {
                    ch.env_level = 4095;
                    ch.env_phase = EnvPhase::Decay;
                } else {
                    ch.env_level = (ch.env_level + u16::from(ins.attack)).min(4095);
                    if ch.env_level == 4095 {
                        ch.env_phase = EnvPhase::Decay;
                    }
                }
            }
            EnvPhase::Decay => {
                let step = if ins.decay == 0 {
                    4095
                } else {
                    u16::from(ins.decay)
                };
                ch.env_level = ch.env_level.saturating_sub(step).max(sustain12);
                if ch.env_level <= sustain12 {
                    ch.env_phase = EnvPhase::Sustain;
                }
            }
            EnvPhase::Release => {
                let step = if ins.release == 0 {
                    4095
                } else {
                    u16::from(ins.release)
                };
                ch.env_level = ch.env_level.saturating_sub(step);
                if ch.env_level == 0 {
                    ch.env_phase = EnvPhase::Off;
                }
            }
        }
        if ch.env_level != before {
            ch.gain_dirty = true;
        }
    }

    fn advance_playback(&mut self) {
        for c in 0..NUM_CHANNELS {
            let pitch_delta = self.mod_apply.pitch[c];
            let ch = &mut self.chans[c];
            ch.cur_valid = false;
            if !ch.playing {
                continue;
            }
            let ins = ch.loaded;
            let pos = (ch.spa >> 16) as u16;
            let addr = ins.start.wrapping_add(pos);
            let word = addr >> 1;
            let hi = addr & 1 == 0;
            match ch.cache {
                Some((w, data)) if w == word => {
                    ch.cur_byte = (if hi { data >> 8 } else { data & 0xFF }) as u8 as i8;
                    ch.cur_valid = true;
                }
                _ => ch.pcm_job = Some((word, hi)),
            }
            let spi = i64::from(ch.spi_current) + i64::from(pitch_delta);
            let spi = spi.clamp(0, i64::from(u32::MAX)) as u32;
            ch.spa = ch.spa.wrapping_add(spi);
            if (ch.spa >> 16) >= u32::from(ins.len) {
                if ins.loop_en && ins.len > ins.loop_start {
                    ch.spa = ch
                        .spa
                        .wrapping_sub(u32::from(ins.len - ins.loop_start) << 16);
                } else {
                    ch.playing = false;
                }
            }
        }
    }

    fn eff_vol(&self, c: usize) -> u8 {
        (i32::from(self.chans[c].volume) + self.mod_apply.volume[c]).clamp(0, 255) as u8
    }

    /// Compute one output frame from the PCM bytes fetched during the previous frame.
    fn dsp_frame(&mut self) {
        // Cycle accounting: a shift-add multiplier costs (coefficient bits + 2) clocks.
        let mul = |bits: u32| bits + 2;
        let mut cost = 0u32;

        let mut acc_l: i64 = 0;
        let mut acc_r: i64 = 0;
        let mut filter_in: i64 = 0;
        let mut mod_values = [0i64; NUM_CHANNELS];
        let mut gain_recomputes = 0u32;

        for c in 0..NUM_CHANNELS {
            let vol = self.eff_vol(c);
            let ch = &mut self.chans[c];
            if ch.gain_dirty || self.mod_apply.volume[c] != 0 {
                gain_recomputes += 1;
                ch.gain_dirty = false;
            }
            let gain = (i64::from(ch.env_level) * i64::from(vol)) >> 8;
            let sample = if ch.cur_valid {
                ((i64::from(ch.cur_byte) << 8) * gain) >> 12
            } else {
                0
            };
            cost += mul(12);
            if ch.mod_mode() {
                mod_values[c] = sample;
            } else if ch.filter_en() {
                filter_in += sample;
            } else {
                let pan = (i32::from(ch.pan) + self.mod_apply.pan[c]).clamp(0, 255);
                let r = (sample * i64::from(pan)) >> 8;
                cost += mul(8);
                acc_r += r;
                acc_l += sample - r;
            }
        }
        cost += gain_recomputes * mul(8);

        // Shared filter: runs every frame, muted or not.
        let frac = self.policy.filter.frac_bits;
        let cutoff = (i32::from(self.filter_cutoff) + self.mod_apply.cutoff).clamp(0, 255) as u8;
        let out = self.svf.step(filter_in << frac, cutoff, self.filter_res);
        self.stats.filter_saturations = self.svf.saturations;
        let sel = out.select(FilterMode::from_bits(self.filter_cfg)) >> frac;
        let sel = sat_i(sel, 20);
        let fvol = (sel * i64::from(self.filter_vol)) >> 8;
        let fpan = (i32::from(self.filter_pan) + self.mod_apply.filter_pan).clamp(0, 255);
        let fr = (fvol * i64::from(fpan)) >> 8;
        acc_r += fr;
        acc_l += fvol - fr;
        cost += 3 * mul(8) + mul(8) + mul(8);

        acc_l = sat_i(acc_l, 20);
        acc_r = sat_i(acc_r, 20);

        let master = (i32::from(self.master_vol) + self.mod_apply.master).clamp(0, 255);
        let out_l = sat_i((acc_l * i64::from(master)) >> 8, 16) as i16;
        let out_r = sat_i((acc_r * i64::from(master)) >> 8, 16) as i16;
        cost += 2 * mul(8) + 2;

        self.stats.dsp_clocks_last = cost;
        self.stats.dsp_clocks_max = self.stats.dsp_clocks_max.max(cost);
        if cost > CLKI_PER_FRAME {
            self.stats.dsp_overruns += 1;
        }

        // Output: the frame computed now is serialised during the next frame.
        let emitted = if self.audio_en {
            self.out_latch
        } else {
            (0, 0)
        };
        self.last_out = emitted;
        if self.policy.capture {
            self.captured.push(emitted);
        }
        self.out_latch = (out_l, out_r);

        self.compute_modulation(&mod_values);
    }

    /// Turn this frame's modulation-source values into the contributions applied to the next.
    fn compute_modulation(&mut self, values: &[i64; NUM_CHANNELS]) {
        let mut apply = ModApply::default();
        // Lowest numbered source wins each destination: iterate high to low and overwrite.
        let mut set = [[false; 8]; 8]; // [dest][channel or 0]
        for src in 0..NUM_CHANNELS {
            let ch = &self.chans[src];
            if !ch.mod_mode() {
                continue;
            }
            let route = ch.mod_route;
            let dest = route >> 13;
            let target = ((route >> 10) & 7) as usize;
            let s = i32::from((((route >> 6) & 0xF) as i16) << 12 >> 12);
            let invert = route & 0x20 != 0;
            if dest == moddest::OFF || dest > moddest::FILTER_PAN {
                continue;
            }
            let global = matches!(
                dest,
                moddest::FILTER_CUTOFF | moddest::MASTER_VOLUME | moddest::FILTER_PAN
            );
            let key_c = if global { 0 } else { target };
            if !global && key_c >= NUM_CHANNELS {
                continue;
            }
            if set[dest as usize][key_c] {
                continue;
            }
            set[dest as usize][key_c] = true;

            // Internal M is the 8-bit signed top of the scaled sample.
            let m = (values[src] >> 8).clamp(-128, 127);
            let m = if invert { -m } else { m };
            match dest {
                moddest::PITCH => {
                    let delta = match self.policy.pitch_mod {
                        PitchModLaw::Additive => shl_signed(m, s + 4),
                        PitchModLaw::Proportional => {
                            (i64::from(self.chans[key_c].spi_current) * m) >> (16 - s)
                        }
                    };
                    apply.pitch[key_c] = delta.clamp(-(1 << 30), 1 << 30) as i32;
                }
                moddest::VOLUME => apply.volume[key_c] = shl_signed(m, s) as i32,
                moddest::PAN => apply.pan[key_c] = shl_signed(m, s) as i32,
                moddest::FILTER_CUTOFF => {
                    apply.cutoff = shl_signed(m, s + 4).clamp(-1024, 1024) as i32
                }
                moddest::MASTER_VOLUME => apply.master = shl_signed(m, s) as i32,
                moddest::FILTER_PAN => apply.filter_pan = shl_signed(m, s) as i32,
                _ => {}
            }
        }
        self.mod_apply = apply;
    }
}
