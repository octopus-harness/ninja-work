use crate::app::{App, Pane};
use crate::format::{format_bytes, format_relative_time};
use crate::models::Scope;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == Pane::Workspaces;
    let border_color = if is_focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    let title_style = if is_focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block_title = format!(" [1] Workspaces ({}) ", app.scope);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(block_title, title_style));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split inner area into: Scope tab selector (1 line), separator (1 line), list (remaining)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    // Scope tabs
    let active_style = if app.scope == Scope::Active {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let archive_style = if app.scope == Scope::Archive {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let scope_line = Line::from(vec![
        Span::raw(" Scope: "),
        Span::styled(" [ Active ] ", active_style),
        Span::raw(" "),
        Span::styled(" [ Archive ] ", archive_style),
        Span::styled(" (Tab/s) ", Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(scope_line), chunks[0]);

    // Separator
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(chunks[1].width as usize),
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(sep, chunks[1]);

    // Workspaces list
    if app.filtered_workspaces.is_empty() {
        let msg = if app.search_query.is_some() {
            "No workspaces match search filter."
        } else {
            match app.scope {
                Scope::Active => "No active workspaces. Press 'a' to create one.",
                Scope::Archive => "No archived workspaces.",
            }
        };
        let p = Paragraph::new(Line::from(Span::styled(
            msg,
            Style::default().fg(Color::DarkGray),
        )));
        f.render_widget(p, chunks[2]);
        return;
    }

    let items: Vec<ListItem> = app
        .filtered_workspaces
        .iter()
        .enumerate()
        .map(|(idx, ws)| {
            let is_selected = idx == app.workspace_cursor;
            let icon = match ws.scope {
                Scope::Active => "📁 ",
                Scope::Archive => "📦 ",
            };

            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if is_selected { "▶ " } else { "  " };

            let stats_text = format!(
                " ({} · {})",
                if ws.file_count == 1 {
                    "1 file".to_string()
                } else {
                    format!("{} files", ws.file_count)
                },
                format_bytes(ws.total_bytes)
            );

            let time_str = ws
                .modified_at
                .map(format_relative_time)
                .unwrap_or_default();

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::raw(icon),
                Span::styled(&ws.name, name_style),
                Span::styled(stats_text, Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::styled(time_str, Style::default().fg(Color::DarkGray)),
            ]);

            let item_style = if is_selected && is_focused {
                Style::default().bg(Color::Rgb(35, 45, 60))
            } else if is_selected {
                Style::default().bg(Color::Rgb(25, 25, 30))
            } else {
                Style::default()
            };

            ListItem::new(line).style(item_style)
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, chunks[2]);
}
