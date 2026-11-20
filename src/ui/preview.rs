use crate::app::{App, Pane};
use crate::format::{format_bytes, format_relative_time, word_and_line_count};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == Pane::Preview;
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

    let view_mode_str = if app.preview_raw { "Raw" } else { "Styled" };
    let block_title = format!(" [3] Preview [{}] ", view_mode_str);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(block_title, title_style));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Header metadata (2 lines), separator (1 line), content body (remaining)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);

    if let Some(file) = app.selected_file() {
        let (words, lines) = word_and_line_count(&app.preview_content);
        let time_str = file
            .modified_at
            .map(format_relative_time)
            .unwrap_or_else(|| "unknown".into());

        let line1 = Line::from(vec![
            Span::styled(" File: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &file.rel_path,
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("Size: {}", format_bytes(file.size_bytes)),
                Style::default().fg(Color::Gray),
            ),
            Span::raw(" · "),
            Span::styled(
                format!("{} lines, {} words", lines, words),
                Style::default().fg(Color::Gray),
            ),
            Span::raw(" · "),
            Span::styled(
                format!("modified {}", time_str),
                Style::default().fg(Color::DarkGray),
            ),
        ]);

        let line2 = Line::from(vec![
            Span::styled(" Actions: ", Style::default().fg(Color::DarkGray)),
            Span::styled("[e] ", Style::default().fg(Color::Yellow)),
            Span::raw("edit in $EDITOR  "),
            Span::styled("[v] ", Style::default().fg(Color::Yellow)),
            Span::raw("toggle raw/styled  "),
            Span::styled("[y] ", Style::default().fg(Color::Yellow)),
            Span::raw("copy path  "),
            Span::styled("[Y] ", Style::default().fg(Color::Yellow)),
            Span::raw("copy content"),
        ]);

        f.render_widget(Paragraph::new(vec![line1, line2]), chunks[0]);
    } else if let Some(ws) = app.selected_workspace() {
        let line1 = Line::from(vec![
            Span::styled(" Workspace: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &ws.name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                format!("Scope: {}", ws.scope),
                Style::default().fg(Color::Gray),
            ),
            Span::raw(" · "),
            Span::styled(
                format!("Total size: {}", format_bytes(ws.total_bytes)),
                Style::default().fg(Color::Gray),
            ),
        ]);
        let line2 = Line::from(Span::styled(
            " Select a file in Pane 2 to view content preview.",
            Style::default().fg(Color::DarkGray),
        ));
        f.render_widget(Paragraph::new(vec![line1, line2]), chunks[0]);
    } else {
        let line = Line::from(Span::styled(
            " No workspace or file selected.",
            Style::default().fg(Color::DarkGray),
        ));
        f.render_widget(Paragraph::new(line), chunks[0]);
    }

    // Separator
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(chunks[1].width as usize),
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(sep, chunks[1]);

    // Body content
    if app.preview_content.is_empty() {
        let p = Paragraph::new("(empty)").style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, chunks[2]);
        return;
    }

    let rendered_lines: Vec<Line> = if app.preview_raw {
        app.preview_content
            .lines()
            .map(|l| Line::from(Span::raw(l)))
            .collect()
    } else {
        render_markdown_lines(&app.preview_content)
    };

    let p = Paragraph::new(rendered_lines)
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0));
    f.render_widget(p, chunks[2]);
}

fn render_markdown_lines(content: &str) -> Vec<Line<'_>> {
    let mut in_code_block = false;
    let mut lines = Vec::new();

    for raw_line in content.lines() {
        let trimmed = raw_line.trim_start();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            lines.push(Line::from(Span::styled(
                raw_line,
                Style::default().fg(Color::DarkGray),
            )));
            continue;
        }

        if in_code_block {
            lines.push(Line::from(Span::styled(
                raw_line,
                Style::default().fg(Color::LightGreen),
            )));
            continue;
        }

        if trimmed.starts_with("# ") {
            lines.push(Line::from(Span::styled(
                raw_line,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
        } else if trimmed.starts_with("## ") {
            lines.push(Line::from(Span::styled(
                raw_line,
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(Modifier::BOLD),
            )));
        } else if trimmed.starts_with("### ") {
            lines.push(Line::from(Span::styled(
                raw_line,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
        } else if trimmed.starts_with("- [x]") || trimmed.starts_with("* [x]") {
            lines.push(Line::from(vec![
                Span::styled("  ✔ ", Style::default().fg(Color::Green)),
                Span::styled(
                    trimmed[5..].trim_start(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        } else if trimmed.starts_with("- [ ]") || trimmed.starts_with("* [ ]") {
            lines.push(Line::from(vec![
                Span::styled("  ◻ ", Style::default().fg(Color::Yellow)),
                Span::raw(trimmed[5..].trim_start()),
            ]));
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            lines.push(Line::from(vec![
                Span::styled("  • ", Style::default().fg(Color::Cyan)),
                Span::raw(trimmed[2..].trim_start()),
            ]));
        } else if trimmed.starts_with('>') {
            lines.push(Line::from(Span::styled(
                raw_line,
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::ITALIC),
            )));
        } else {
            lines.push(Line::from(Span::raw(raw_line)));
        }
    }

    lines
}
