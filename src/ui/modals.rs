use crate::app::{App, Modal};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let modal = match &app.modal {
        Some(m) => m,
        None => return,
    };

    match modal {
        Modal::InitDirs {
            add_to_gitignore,
            git_detected,
        } => render_init_dirs(f, app, *add_to_gitignore, *git_detected),
        Modal::Help => render_help(f),
        Modal::Search { input } => render_search(f, input),
        Modal::AddWorkspace { input } => render_add_workspace(f, input),
        Modal::RenameWorkspace {
            scope: _,
            old_name,
            input,
        } => render_rename_workspace(f, old_name, input),
        Modal::ConfirmArchive { name } => render_confirm_archive(f, name),
        Modal::ConfirmRestore { name, target_input } => {
            render_confirm_restore(f, name, target_input)
        }
        Modal::ConfirmDeleteWorkspace { scope, name } => {
            render_confirm_delete_workspace(f, *scope, name)
        }
        Modal::NewFile { input } => render_new_file(f, input),
        Modal::RenameFile { old_rel, input } => render_rename_file(f, old_rel, input),
        Modal::ConfirmDeleteFile { rel_path } => render_confirm_delete_file(f, rel_path),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_init_dirs(
    f: &mut Frame,
    app: &App,
    add_to_gitignore: bool,
    git_detected: bool,
) {
    let area = centered_rect(65, 35, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Initialize Workspace Directories ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = vec![
        Line::from(Span::styled(
            "The required workspace directories were not found:",
            Style::default().fg(Color::LightYellow),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw(" • "),
            Span::styled(
                app.config.workspace_dir.display().to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw(" • "),
            Span::styled(
                app.config.archive_dir.display().to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ]),
    ];

    if git_detected {
        lines.push(Line::from(""));
        let (checkbox, box_style) = if add_to_gitignore {
            (
                "[X] ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            ("[ ] ", Style::default().fg(Color::DarkGray))
        };
        lines.push(Line::from(vec![
            Span::styled(checkbox, box_style),
            Span::styled(
                "Add workspace/ and archive/ to .gitignore ",
                Style::default().fg(Color::White),
            ),
            Span::styled(
                "(Press [Space] to toggle)",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    lines.push(Line::from(""));
    let mut footer_spans = vec![
        Span::styled("Press ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            "[Enter]",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " to create directories",
            Style::default().fg(Color::DarkGray),
        ),
    ];

    if git_detected {
        footer_spans.push(Span::styled(", ", Style::default().fg(Color::DarkGray)));
        footer_spans.push(Span::styled(
            "[Space]",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
        footer_spans.push(Span::styled(
            " to toggle .gitignore",
            Style::default().fg(Color::DarkGray),
        ));
    }

    footer_spans.push(Span::styled(", or ", Style::default().fg(Color::DarkGray)));
    footer_spans.push(Span::styled(
        "[Esc / q]",
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD),
    ));
    footer_spans.push(Span::styled(" to exit.", Style::default().fg(Color::DarkGray)));

    lines.push(Line::from(footer_spans));

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_search(f: &mut Frame, input: &str) {
    let area = centered_rect(50, 15, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Filter Workspaces (/) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(vec![
            Span::styled("Filter: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                input,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Apply filter   [Esc] Clear & close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_add_workspace(f: &mut Frame, input: &str) {
    let area = centered_rect(55, 18, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" New Workspace ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(Span::styled(
            "Enter name for new workspace under ./workspace:",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                input,
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Create   [Esc] Cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_rename_workspace(f: &mut Frame, old_name: &str, input: &str) {
    let area = centered_rect(55, 18, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Rename Workspace ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(vec![
            Span::styled("Renaming: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                old_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("New name: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                input,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Rename   [Esc] Cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_confirm_archive(f: &mut Frame, name: &str) {
    let area = centered_rect(55, 18, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Archive Workspace ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(vec![
            Span::styled("Move workspace ", Style::default().fg(Color::White)),
            Span::styled(
                name,
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to ./archive?", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "If an archive with this name exists, a timestamp suffix will be added.",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "[Enter / y]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Archive   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "[Esc / n]",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_confirm_restore(f: &mut Frame, name: &str, target_input: &str) {
    let area = centered_rect(55, 20, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Restore Workspace ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(vec![
            Span::styled("Restoring: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                name,
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Target name: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                target_input,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Timestamp suffixes are stripped automatically by default.",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Restore to ./workspace   [Esc] Cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_confirm_delete_workspace(f: &mut Frame, scope: crate::models::Scope, name: &str) {
    let area = centered_rect(55, 18, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" ⚠️ Delete Workspace ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(vec![
            Span::styled("Are you sure you want to delete ", Style::default().fg(Color::White)),
            Span::styled(
                name,
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" in {}?", scope), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "This will delete all files inside and CANNOT be undone!",
            Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "[Enter / y]",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Delete   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "[Esc / n]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_new_file(f: &mut Frame, input: &str) {
    let area = centered_rect(55, 18, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" New File in Workspace ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Green));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(Span::styled(
            "Enter relative file name (e.g. prompt.md, notes.txt):",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("File: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                input,
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Create   [Esc] Cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_rename_file(f: &mut Frame, old_rel: &str, input: &str) {
    let area = centered_rect(55, 18, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Rename File ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(vec![
            Span::styled("Old path: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                old_rel,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("New path: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                input,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter] Rename   [Esc] Cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_confirm_delete_file(f: &mut Frame, rel_path: &str) {
    let area = centered_rect(55, 18, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Delete File ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(vec![
            Span::styled("Delete file ", Style::default().fg(Color::White)),
            Span::styled(
                rel_path,
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("?", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "[Enter / y]",
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Delete   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "[Esc / n]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(70, 75, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Keyboard Shortcuts & Guide ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![
        Line::from(Span::styled(
            "Global Navigation",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  1 / 2 / 3      Switch focus between Panes (Workspaces, Files, Preview)"),
        Line::from("  Tab / s        Toggle Scope (Active <-> Archive)"),
        Line::from("  /              Filter / Search workspaces by name"),
        Line::from("  R              Refresh workspace and file listings"),
        Line::from("  ?              Open this Help dialog"),
        Line::from("  q / Esc        Clear filter, close dialog, or quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Pane 1: Workspaces",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  j / k / ↑ / ↓  Navigate workspaces"),
        Line::from("  Enter / l / →  Open selected workspace (focus Files pane)"),
        Line::from("  a / n          Add new workspace"),
        Line::from("  A              Archive (if Active) or Restore (if Archive)"),
        Line::from("  r / m          Rename workspace"),
        Line::from("  d / x          Delete workspace (with confirmation)"),
        Line::from("  y              Yank workspace path to clipboard"),
        Line::from(""),
        Line::from(Span::styled(
            "Pane 2: Files",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  j / k / ↑ / ↓  Navigate files"),
        Line::from("  h / ←          Return to Workspaces pane"),
        Line::from("  Enter / l / →  Focus Preview pane"),
        Line::from("  n / a          Create new file"),
        Line::from("  r / m          Rename file"),
        Line::from("  d / x          Delete file"),
        Line::from("  e / E          Edit file in $EDITOR"),
        Line::from("  y              Yank file path to clipboard"),
        Line::from(""),
        Line::from(Span::styled(
            "Pane 3: Preview",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  j / k / d / u  Scroll preview down/up"),
        Line::from("  v              Toggle Raw vs Styled Markdown preview"),
        Line::from("  e / E          Edit file in $EDITOR"),
        Line::from("  y / Y          Yank file path (y) or content (Y)"),
        Line::from(""),
        Line::from(Span::styled(
            "Press [Esc], [Enter], or [?] to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}
