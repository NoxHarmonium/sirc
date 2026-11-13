//! Drawing the ERENCE screens.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use sirc_umf::compile::{compile, describe_usage};
use sirc_umf::song::{Cell, NoteCell, CHANNELS};

use crate::app::{App, Field, Tab};

const GUTTER: usize = 6;
const CELL_W: usize = 12;
const SEP_W: usize = 1;

const BG_CURSOR: Color = Color::Rgb(60, 70, 110);
const BG_BEAT: Color = Color::Rgb(34, 36, 46);
const BG_PLAY: Color = Color::Rgb(90, 60, 30);
const FG_DIM: Color = Color::DarkGray;
const FG_NOTE: Color = Color::Rgb(240, 240, 240);
const FG_INST: Color = Color::Rgb(120, 200, 255);
const FG_FINE: Color = Color::Rgb(255, 190, 110);
const FG_OFF: Color = Color::Rgb(255, 120, 120);

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    let [title, body, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(2),
    ])
    .areas(area);
    draw_title(f, title, app);
    match app.tab {
        Tab::Pattern => draw_pattern(f, body, app),
        Tab::Instruments => draw_instruments(f, body, app),
        Tab::Song => draw_song(f, body, app),
    }
    draw_status(f, status, app);
    if let Some(p) = &app.prompt {
        let w = area.width.saturating_sub(8).clamp(20, 80);
        let r = Rect::new(
            area.x + (area.width.saturating_sub(w)) / 2,
            area.y + area.height / 2 - 1.min(area.height / 2),
            w,
            3,
        );
        f.render_widget(Clear, r);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", p.kind.label()));
        let text = format!("{}_", p.buf);
        f.render_widget(Paragraph::new(text).block(block), r);
    }
}

fn draw_title(f: &mut Frame, area: Rect, app: &App) {
    let tab = |t: Tab, label: &str| {
        let on = app.tab == t;
        Span::styled(
            format!(" {label} "),
            if on {
                Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
            } else {
                Style::default().fg(FG_DIM)
            },
        )
    };
    let file = app
        .path
        .as_ref()
        .map_or("(unsaved)".to_string(), |p| p.display().to_string());
    let line = Line::from(vec![
        Span::styled(
            " ERENCE ",
            Style::default().bg(Color::Blue).fg(Color::White),
        ),
        Span::raw(" "),
        tab(Tab::Pattern, "F1 Pattern"),
        tab(Tab::Instruments, "F2 Instruments"),
        tab(Tab::Song, "F3 Song"),
        Span::raw(format!(
            "  {}{}  {}",
            app.song.title,
            if app.dirty { " *" } else { "" },
            file
        )),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn draw_status(f: &mut Frame, area: Rect, app: &App) {
    let style = if app.status_err {
        Style::default().fg(Color::LightRed)
    } else {
        Style::default().fg(Color::Gray)
    };
    let help = match app.tab {
        Tab::Pattern => {
            "keys: z-m/q-p notes  1 off  Del clear  [ ] octave  ; ' instrument  l loop  \\ mute  / solo  Space play  F6 from here"
        }
        Tab::Instruments => {
            "keys: PgUp/PgDn select  Up/Down field  Left/Right change  n new  c copy  x delete  i import WAV  z-m audition"
        }
        Tab::Song => "keys: Up/Down field  Left/Right change (Shift x10)  Enter edit text",
    };
    let usage = compile(&app.song, [false; CHANNELS])
        .map_or_else(|e| format!("error: {e}"), |i| describe_usage(&i.usage));
    let lines = vec![
        Line::from(Span::styled(app.status.clone(), style)),
        Line::from(Span::styled(
            format!("{help}  |  memory {usage}"),
            Style::default().fg(FG_DIM),
        )),
    ];
    f.render_widget(Paragraph::new(lines), area);
}

/// Spans for one cell, with the field under the cursor highlighted.
fn cell_spans(cell: &Cell, cursor: Option<Field>, base: Style) -> Vec<Span<'static>> {
    let note = match cell.note {
        NoteCell::None => ("...".to_string(), FG_DIM),
        NoteCell::Off => ("===".to_string(), FG_OFF),
        NoteCell::On(n) => (sirc_umf::song::note_name(n), FG_NOTE),
    };
    let inst = cell.inst.map_or(("..".to_string(), FG_DIM), |i| {
        (format!("{i:02X}"), FG_INST)
    });
    let fine = cell.fine.map_or((".....".to_string(), FG_DIM), |v| {
        (format!("{v:+05}"), FG_FINE)
    });
    let part = |text: String, fg: Color, field: Field| {
        let mut st = base.fg(fg);
        if cursor == Some(field) {
            st = st.bg(Color::Rgb(220, 220, 90)).fg(Color::Black);
        }
        Span::styled(text, st)
    };
    vec![
        part(note.0, note.1, Field::Note),
        Span::styled(" ", base),
        part(inst.0, inst.1, Field::Inst),
        Span::styled(" ", base),
        part(fine.0, fine.1, Field::Fine),
    ]
}

fn draw_pattern(f: &mut Frame, area: Rect, app: &App) {
    let width = usize::from(area.width);
    let visible_ch = ((width.saturating_sub(GUTTER)) / (CELL_W + SEP_W)).clamp(1, CHANNELS);
    let first_ch = app
        .cur_ch
        .saturating_sub(visible_ch - 1)
        .min(CHANNELS - visible_ch);
    let first_ch = if app.cur_ch < first_ch {
        app.cur_ch
    } else {
        first_ch
    };

    let height = usize::from(area.height);
    let rows_visible = height.saturating_sub(2).max(1);
    let top = app
        .cur_row
        .saturating_sub(rows_visible / 2)
        .min(app.song.rows.len().saturating_sub(rows_visible));

    let mut lines: Vec<Line> = Vec::new();
    // Header: channel numbers and mute marks.
    let mut head = vec![Span::raw(" ".repeat(GUTTER))];
    let mute = app.effective_mute();
    for c in first_ch..first_ch + visible_ch {
        let name = format!(
            "CH{}{}{}",
            c + 1,
            if app.song.channels[c].modulator.is_some() {
                " mod"
            } else {
                ""
            },
            if mute[c] { " (muted)" } else { "" }
        );
        let style = if c == app.cur_ch {
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(FG_DIM)
        };
        head.push(Span::styled(
            format!("{name:<w$}", w = CELL_W + SEP_W),
            style,
        ));
    }
    lines.push(Line::from(head));
    lines.push(Line::from(Span::styled(
        "-".repeat(width.min(GUTTER + visible_ch * (CELL_W + SEP_W))),
        Style::default().fg(FG_DIM),
    )));

    let playing_row = app.play_row();
    for r in top..(top + rows_visible).min(app.song.rows.len()) {
        let is_cursor = r == app.cur_row;
        let is_play = playing_row == Some(r);
        let base = if is_play {
            Style::default().bg(BG_PLAY)
        } else if is_cursor {
            Style::default().bg(BG_CURSOR)
        } else if r % 4 == 0 {
            Style::default().bg(BG_BEAT)
        } else {
            Style::default()
        };
        let marker = if app.song.loop_row == Some(r) {
            'L'
        } else {
            ' '
        };
        let mut spans = vec![Span::styled(
            format!("{marker}{r:03X} "),
            base.fg(if r % 4 == 0 { Color::Gray } else { FG_DIM }),
        )];
        for c in first_ch..first_ch + visible_ch {
            let cursor_field = (is_cursor && c == app.cur_ch).then_some(app.field);
            spans.extend(cell_spans(&app.song.rows[r][c], cursor_field, base));
            spans.push(Span::styled(" ", base));
        }
        lines.push(Line::from(spans));
    }
    f.render_widget(Paragraph::new(lines), area);
}

fn draw_instruments(f: &mut Frame, area: Rect, app: &App) {
    let [list, form] =
        Layout::horizontal([Constraint::Length(28), Constraint::Min(20)]).areas(area);

    let mut items: Vec<Line> = Vec::new();
    for (i, inst) in app.song.instruments.iter().enumerate() {
        let sel = i == app.inst_sel;
        let style = if sel {
            Style::default().bg(BG_CURSOR).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let cur = if i == usize::from(app.cur_inst) {
            '*'
        } else {
            ' '
        };
        items.push(Line::from(Span::styled(
            format!("{cur}{i:02X} {}", inst.name),
            style,
        )));
    }
    f.render_widget(
        Paragraph::new(items).block(
            Block::default()
                .borders(Borders::RIGHT)
                .title(" Instruments "),
        ),
        list,
    );

    let mut lines: Vec<Line> = Vec::new();
    for (i, (label, value)) in app.instrument_fields().iter().enumerate() {
        let sel = i == app.inst_field;
        let style = if sel {
            Style::default().bg(BG_CURSOR)
        } else {
            Style::default()
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {label:<12}"),
                Style::default().fg(FG_DIM).patch(style),
            ),
            Span::styled(value.clone(), style),
        ]));
    }
    lines.push(Line::raw(""));
    if let Some(inst) = app.song.instruments.get(app.inst_sel) {
        lines.extend(waveform_lines(
            &inst.pcm(),
            usize::from(form.width.saturating_sub(4)),
        ));
    }
    f.render_widget(Paragraph::new(lines), form);
}

/// A small text plot of the first part of a waveform.
fn waveform_lines(pcm: &[i8], width: usize) -> Vec<Line<'static>> {
    const H: usize = 9;
    if pcm.is_empty() || width < 8 {
        return Vec::new();
    }
    let n = pcm.len();
    let mut grid = vec![vec![' '; width]; H];
    for (x, col) in (0..width).enumerate() {
        let lo = x * n / width;
        let hi = (((x + 1) * n / width).max(lo + 1)).min(n);
        let (mut min, mut max) = (127i32, -128i32);
        for v in &pcm[lo..hi] {
            min = min.min(i32::from(*v));
            max = max.max(i32::from(*v));
        }
        let y = |v: i32| ((127 - v) * (H as i32 - 1) / 255) as usize;
        for cells in &mut grid[y(max)..=y(min)] {
            cells[col] = '█';
        }
    }
    let mid = H / 2;
    for (x, ch) in grid[mid].iter_mut().enumerate() {
        if *ch == ' ' && x % 2 == 0 {
            *ch = '·';
        }
    }
    grid.into_iter()
        .map(|r| {
            Line::from(Span::styled(
                format!("  {}", r.into_iter().collect::<String>()),
                Style::default().fg(FG_INST),
            ))
        })
        .collect()
}

fn draw_song(f: &mut Frame, area: Rect, app: &App) {
    let fields = app.song_fields();
    let mut lines: Vec<Line> = Vec::new();
    let mut last_group = String::new();
    let mut selected_line = 0;
    for (i, (group, label, value)) in fields.iter().enumerate() {
        if *group != last_group {
            if !last_group.is_empty() {
                lines.push(Line::raw(""));
            }
            lines.push(Line::from(Span::styled(
                format!(" {group}"),
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )));
            last_group = group.clone();
        }
        let sel = i == app.song_field;
        if sel {
            selected_line = lines.len();
        }
        let style = if sel {
            Style::default().bg(BG_CURSOR)
        } else {
            Style::default()
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("   {label:<16}"),
                Style::default().fg(FG_DIM).patch(style),
            ),
            Span::styled(value.clone(), style),
        ]));
    }
    let height = usize::from(area.height);
    let scroll = selected_line.saturating_sub(height / 2);
    let shown: Vec<Line> = lines.into_iter().skip(scroll).take(height).collect();
    f.render_widget(Paragraph::new(shown), area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::NullAudio;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    use sirc_umf::song::Song;

    fn screen(app: &App, w: u16, h: u16) -> String {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|f| draw(f, app)).unwrap();
        let buf = term.backend().buffer().clone();
        (0..h)
            .map(|y| {
                (0..w)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn app() -> App {
        let mut s = Song::default();
        s.rows[0][0] = Cell {
            note: NoteCell::On(39),
            inst: Some(1),
            fine: Some(-5),
        };
        App::new(s, None, Box::new(NullAudio::default()))
    }

    #[test]
    fn pattern_screen_shows_cells() {
        let a = app();
        let s = screen(&a, 100, 24);
        assert!(s.contains("C-4 01 -0005"), "{s}");
        assert!(s.contains("CH1"));
        assert!(s.contains("F1 Pattern"));
    }

    #[test]
    fn narrow_terminals_scroll_channels() {
        let mut a = app();
        a.cur_ch = 5;
        let s = screen(&a, 60, 20);
        assert!(s.contains("CH6"), "{s}");
        assert!(!s.contains("CH1 "), "{s}");
    }

    #[test]
    fn other_tabs_draw() {
        let mut a = app();
        a.tab = Tab::Instruments;
        let s = screen(&a, 100, 30);
        assert!(s.contains("Sine"), "{s}");
        assert!(s.contains("Attack"));
        a.tab = Tab::Song;
        let s = screen(&a, 100, 30);
        assert!(s.contains("Master volume"), "{s}");
    }

    #[test]
    fn prompt_is_drawn() {
        let mut a = app();
        a.prompt = Some(crate::app::Prompt {
            kind: crate::app::PromptKind::Save,
            buf: "tune.erence".into(),
        });
        let s = screen(&a, 100, 24);
        assert!(s.contains("Save as"), "{s}");
        assert!(s.contains("tune.erence_"));
    }

    #[test]
    fn tiny_terminal_does_not_panic() {
        let a = app();
        let _ = screen(&a, 20, 5);
        let _ = screen(&a, 1, 1);
    }
}
