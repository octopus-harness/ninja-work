pub mod files;
pub mod modals;
pub mod preview;
pub mod workspaces;

use crate::app::{App, Pane};
use crate::format::format_bytes;
use crate::models::Scope;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    render_header(f, app, chunks[0]);

    // 3-Pane Body
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(28),
            Constraint::Percentage(32),
            Constraint::Percentage(40),
        ])
        .split(chunks[1]);

    workspaces::render(f, app, body_chunks[0]);
    files::render(f, app, body_chunks[1]);
    preview::render(f, app, body_chunks[2]);

    render_footer(f, app, chunks[2]);

    // Modals overlay
    modals::render(f, app);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let root_badge = format!(" [Root: {}] ", app.config.root_dir.display());

    let (active_count, archive_count, total_bytes) = match app.scope {
        Scope::Active => {
            let active = app.all_workspaces.len();
            let bytes: u64 = app.all_workspaces.iter().map(|w| w.total_bytes).sum();
            (active.to_string(), "?".to_string(), bytes)
        }
        Scope::Archive => {
            let archive = app.all_workspaces.len();
            let bytes: u64 = app.all_workspaces.iter().map(|w| w.total_bytes).sum();
            ("?".to_string(), archive.to_string(), bytes)
        }
    };

    let stats_text = format!(
        "Active: {} · Archived: {} · Size: {}",
        active_count,
        archive_count,
        format_bytes(total_bytes)
    );

    let mut spans = vec![
        Span::styled(
            " WORK ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            root_badge,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" · {}", stats_text), Style::default().fg(Color::Gray)),
    ];

    if let Some(query) = &app.search_query {
        spans.push(Span::styled(
            format!(" · [Filter: \"{}\"] (Esc to clear)", query),
            Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let p = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(p, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let status_span = if let Some((msg, is_error)) = &app.status {
        let color = if *is_error { Color::Red } else { Color::Green };
        Span::styled(
            format!(" ● {} ", msg),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )
    } else {
        let focus_hint = match app.focused_pane {
            Pane::Workspaces => "Pane 1 (Workspaces): j/k: nav | Enter: files | a: new | A: archive/restore | r: rename | d: del | y: yank",
            Pane::Files => "Pane 2 (Files): j/k: nav | Enter: preview | n: new | r: rename | d: del | e: $EDITOR | y: yank",
            Pane::Preview => "Pane 3 (Preview): j/k/d/u: scroll | v: raw/styled | e: $EDITOR | y: yank path | Y: yank text",
        };
        Span::styled(format!(" {} ", focus_hint), Style::default().fg(Color::DarkGray))
    };

    let key_hints_str = "1/2/3: panes | Tab/s: scope | /: filter | ?: help | q: quit ";
    let key_hints = Span::styled(key_hints_str, Style::default().fg(Color::DarkGray));

    let total_width = area.width as usize;
    let left_len = status_span.content.len();
    let right_len = key_hints.content.len();
    let pad = total_width.saturating_sub(left_len + right_len);

    let line = Line::from(vec![
        status_span,
        Span::raw(" ".repeat(pad)),
        key_hints,
    ]);

    f.render_widget(Paragraph::new(line), area);
}
