//! ERENCE: Event Recording, Editing, Notation and Composition Environment.
//!
//! A tracker-style editor for SIRCAS sequences, part of SIRCUMF (the SIRC Unified Music
//! Framework). It previews through the SIRCAS reference model and exports SIRC assembly.

mod app;
mod audio;
mod ui;

use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use sirc_umf::compile::{self, RenderOpts};
use sirc_umf::song::Song;
use sirc_umf::OUTPUT_RATE_HZ;

use crate::app::App;
use crate::audio::{Audio, CpalAudio, NullAudio};

const USAGE: &str = "\
ERENCE - SIRCAS tracker

usage:
  erence [song.erence]                  edit a song (starts a new one if the file does not exist)
  erence --export out.sasm song.erence  write SIRC assembly and exit
  erence --render out.wav song.erence   render the song to a WAV file and exit
  erence --new song.erence              write a new empty song and exit

In the editor: F1 pattern, F2 instruments, F3 song, Space plays, Ctrl-S saves, Ctrl-E exports
assembly, Ctrl-W renders a WAV, Ctrl-O opens, Ctrl-Q quits.";

fn load(path: &Path) -> Result<Song, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let (song, warnings) =
        Song::from_text(&text, path.parent()).map_err(|e| format!("{}: {e}", path.display()))?;
    for w in warnings {
        eprintln!("warning: {w}");
    }
    Ok(song)
}

fn run_cli(args: &[String]) -> Result<bool, String> {
    let mut it = args.iter();
    match it.next().map(String::as_str) {
        Some("--help" | "-h") => {
            println!("{USAGE}");
            Ok(true)
        }
        Some("--export") => {
            let (out, song) = (it.next(), it.next());
            let (Some(out), Some(song)) = (out, song) else {
                return Err("usage: erence --export out.sasm song.erence".into());
            };
            let s = load(Path::new(song))?;
            let image = compile::compile(&s, [false; 6])?;
            std::fs::write(out, compile::export_asm(&s, &image))
                .map_err(|e| format!("{out}: {e}"))?;
            println!("{out}: {}", compile::describe_usage(&image.usage));
            Ok(true)
        }
        Some("--render") => {
            let (out, song) = (it.next(), it.next());
            let (Some(out), Some(song)) = (out, song) else {
                return Err("usage: erence --render out.wav song.erence".into());
            };
            let s = load(Path::new(song))?;
            let r = compile::render(&s, &RenderOpts::default(), false)?;
            sirc_umf::testkit::write_wav(Path::new(out), &r.frames, OUTPUT_RATE_HZ as u32)
                .map_err(|e| format!("{out}: {e}"))?;
            println!(
                "{out}: {:.1} s{}",
                r.frames.len() as f64 / OUTPUT_RATE_HZ,
                if r.overrun {
                    " (sequencer missed a tick)"
                } else {
                    ""
                }
            );
            Ok(true)
        }
        Some("--new") => {
            let Some(path) = it.next() else {
                return Err("usage: erence --new song.erence".into());
            };
            std::fs::write(path, Song::default().to_text()).map_err(|e| format!("{path}: {e}"))?;
            println!("wrote {path}");
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run_cli(&args) {
        Ok(true) => return,
        Ok(false) => {}
        Err(e) => {
            eprintln!("erence: {e}");
            std::process::exit(1);
        }
    }
    let path: Option<PathBuf> = args.first().map(PathBuf::from);
    let (song, path) = match &path {
        Some(p) if p.exists() => match load(p) {
            Ok(s) => (s, path.clone()),
            Err(e) => {
                eprintln!("erence: {e}");
                std::process::exit(1);
            }
        },
        _ => (Song::default(), path.clone()),
    };
    let (audio, audio_note): (Box<dyn Audio>, Option<String>) = match CpalAudio::new() {
        Ok(a) => (Box::new(a), None),
        Err(e) => (Box::new(NullAudio::default()), Some(e)),
    };
    let mut app = App::new(song, path, audio);
    if let Some(note) = audio_note {
        app.status = format!("{note}: playback is off, WAV render and export still work");
        app.status_err = true;
    }
    if let Err(e) = run(&mut app) {
        eprintln!("erence: {e}");
        std::process::exit(1);
    }
}

fn run(app: &mut App) -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = io::stdout().execute(LeaveAlternateScreen);
        hook(info);
    }));
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let result = event_loop(&mut terminal, app);
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    while !app.quit {
        terminal.draw(|f| ui::draw(f, app))?;
        // Redraw often enough to follow the play position.
        if event::poll(Duration::from_millis(40))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Release {
                    app.handle_key(key);
                }
            }
        }
    }
    Ok(())
}
