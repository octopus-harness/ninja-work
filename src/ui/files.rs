use crate::app::{App, Pane};
use crate::format::{format_bytes, format_relative_time};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == Pane::Files;
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

    let count_text = if app.files.is_empty() {
        "".to_string()
    } else {
        format!(" ({})", app.files.len())
    };
    let block_title = format!(" [2] Files{} ", count_text);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(block_title, title_style));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Current workspace header (1 line), separator (1 line), file list (remaining)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    let ws_header = if let Some(ws) = app.selected_workspace() {
        Line::from(vec![
            Span::styled(" Workspace: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &ws.name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ('n': new file) ", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(Span::styled(
            " No workspace selected",
            Style::default().fg(Color::DarkGray),
        ))
    };
    f.render_widget(Paragraph::new(ws_header), chunks[0]);

    // Separator
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(chunks[1].width as usize),
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(sep, chunks[1]);

    if app.files.is_empty() {
        let msg = if app.selected_workspace().is_some() {
            "Workspace is empty.\nPress 'n' to create a file."
        } else {
            "No files."
        };
        let p = Paragraph::new(msg).style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, chunks[2]);
        return;
    }

    let items: Vec<ListItem> = app
        .files
        .iter()
        .enumerate()
        .map(|(idx, file)| {
            let is_selected = idx == app.file_cursor;
            let icon = if file.is_dir {
                "📁 "
            } else if file.rel_path.ends_with(".md") {
                "📝 "
            } else if file.rel_path.ends_with(".rs") || file.rel_path.ends_with(".sh") {
                "⚙️  "
            } else if file.rel_path.ends_with(".json") || file.rel_path.ends_with(".toml") {
                "🔧 "
            } else {
                "📄 "
            };

            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if is_selected { "▶ " } else { "  " };

            let size_text = if file.is_dir {
                "<dir>".to_string()
            } else {
                format_bytes(file.size_bytes)
            };

            let time_str = file
                .modified_at
                .map(format_relative_time)
                .unwrap_or_default();

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::raw(icon),
                Span::styled(&file.rel_path, name_style),
                Span::raw("  "),
                Span::styled(format!("[{}]", size_text), Style::default().fg(Color::DarkGray)),
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
