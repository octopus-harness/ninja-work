use crate::config::Config;
use crate::models::{Scope, WorkspaceFile, WorkspaceInfo};
use crate::workspace;
use anyhow::Result;
use crossterm::event::KeyCode;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Workspaces,
    Files,
    Preview,
}

#[derive(Debug, Clone)]
pub enum Modal {
    AddWorkspace {
        input: String,
    },
    RenameWorkspace {
        scope: Scope,
        old_name: String,
        input: String,
    },
    ConfirmArchive {
        name: String,
    },
    ConfirmRestore {
        name: String,
        target_input: String,
    },
    ConfirmDeleteWorkspace {
        scope: Scope,
        name: String,
    },
    NewFile {
        input: String,
    },
    RenameFile {
        old_rel: String,
        input: String,
    },
    ConfirmDeleteFile {
        rel_path: String,
    },
    Search {
        input: String,
    },
    InitDirs {
        add_to_gitignore: bool,
        git_detected: bool,
    },
    Help,
}

pub struct App {
    pub config: Config,
    pub focused_pane: Pane,
    pub scope: Scope,
    pub all_workspaces: Vec<WorkspaceInfo>,
    pub filtered_workspaces: Vec<WorkspaceInfo>,
    pub workspace_cursor: usize,
    pub files: Vec<WorkspaceFile>,
    pub file_cursor: usize,
    pub preview_content: String,
    pub preview_scroll: u16,
    pub preview_raw: bool,
    pub search_query: Option<String>,
    pub modal: Option<Modal>,
    pub status: Option<(String, bool)>, // (message, is_error)
    pub should_quit: bool,
    pub pending_editor: Option<PathBuf>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let mut app = Self {
            config,
            focused_pane: Pane::Workspaces,
            scope: Scope::Active,
            all_workspaces: Vec::new(),
            filtered_workspaces: Vec::new(),
            workspace_cursor: 0,
            files: Vec::new(),
            file_cursor: 0,
            preview_content: String::new(),
            preview_scroll: 0,
            preview_raw: false,
            search_query: None,
            modal: None,
            status: None,
            should_quit: false,
            pending_editor: None,
        };

        if workspace::require_dirs(&app.config).is_err() {
            let git_detected = workspace::is_git_repo(&app.config.root_dir);
            app.modal = Some(Modal::InitDirs {
                add_to_gitignore: git_detected,
                git_detected,
            });
        } else {
            app.reload_workspaces();
        }

        app
    }

    pub fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status = Some((msg.into(), is_error));
    }

    pub fn clear_status(&mut self) {
        self.status = None;
    }

    pub fn copy_to_clipboard(&mut self, text: &str, success_msg: &str) {
        match arboard::Clipboard::new() {
            Ok(mut cb) => {
                if let Err(e) = cb.set_text(text) {
                    self.set_status(format!("Failed to copy to clipboard: {}", e), true);
                } else {
                    self.set_status(success_msg, false);
                }
            }
            Err(e) => {
                self.set_status(format!("Clipboard unavailable: {}", e), true);
            }
        }
    }

    pub fn reload_workspaces(&mut self) {
        match workspace::list_workspaces(&self.config, self.scope) {
            Ok(ws) => {
                self.all_workspaces = ws;
                self.apply_filter();
            }
            Err(e) => {
                self.set_status(format!("Failed to list workspaces: {}", e), true);
                self.all_workspaces.clear();
                self.filtered_workspaces.clear();
                self.files.clear();
                self.preview_content.clear();
            }
        }
    }

    pub fn apply_filter(&mut self) {
        if let Some(query) = &self.search_query {
            let q = query.to_lowercase();
            self.filtered_workspaces = self
                .all_workspaces
                .iter()
                .filter(|w| w.name.to_lowercase().contains(&q))
                .cloned()
                .collect();
        } else {
            self.filtered_workspaces = self.all_workspaces.clone();
        }

        if self.filtered_workspaces.is_empty() {
            self.workspace_cursor = 0;
            self.files.clear();
            self.file_cursor = 0;
            self.preview_content.clear();
        } else {
            if self.workspace_cursor >= self.filtered_workspaces.len() {
                self.workspace_cursor = self.filtered_workspaces.len().saturating_sub(1);
            }
            self.reload_files();
        }
    }

    pub fn selected_workspace(&self) -> Option<&WorkspaceInfo> {
        self.filtered_workspaces.get(self.workspace_cursor)
    }

    pub fn selected_file(&self) -> Option<&WorkspaceFile> {
        self.files.get(self.file_cursor)
    }

    pub fn reload_files(&mut self) {
        if let Some(ws) = self.selected_workspace() {
            match workspace::list_workspace_files(&ws.path) {
                Ok(files) => {
                    self.files = files;
                    if self.file_cursor >= self.files.len() {
                        self.file_cursor = self.files.len().saturating_sub(1);
                    }
                    self.reload_preview();
                }
                Err(e) => {
                    self.set_status(format!("Failed to list files: {}", e), true);
                    self.files.clear();
                    self.file_cursor = 0;
                    self.preview_content.clear();
                }
            }
        } else {
            self.files.clear();
            self.file_cursor = 0;
            self.preview_content.clear();
        }
    }

    pub fn reload_preview(&mut self) {
        self.preview_scroll = 0;
        if let Some(file) = self.selected_file() {
            if file.is_dir {
                self.preview_content = format!("Directory: {}\n", file.rel_path);
            } else {
                match workspace::read_file_preview(&file.full_path, 64 * 1024) {
                    Ok(text) => self.preview_content = text,
                    Err(e) => self.preview_content = format!("Error reading preview: {}", e),
                }
            }
        } else if let Some(ws) = self.selected_workspace() {
            self.preview_content = format!(
                "Workspace: {}\nPath: {}\nScope: {}\nFiles: {}\nSize: {}\n\n(No file selected)",
                ws.name,
                ws.path.display(),
                ws.scope,
                ws.file_count,
                crate::format::format_bytes(ws.total_bytes)
            );
        } else {
            self.preview_content = "No workspace selected.".to_string();
        }
    }

    pub fn toggle_scope(&mut self) {
        self.scope = match self.scope {
            Scope::Active => Scope::Archive,
            Scope::Archive => Scope::Active,
        };
        self.workspace_cursor = 0;
        self.reload_workspaces();
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        // Clear transient status on any new action
        if self.status.is_some() {
            self.clear_status();
        }

        // Handle active modal first
        if self.modal.is_some() {
            self.handle_modal_key(key);
            return;
        }

        // Global shortcuts
        match key {
            KeyCode::Char('q') => {
                if self.search_query.is_some() {
                    self.search_query = None;
                    self.apply_filter();
                } else {
                    self.should_quit = true;
                }
                return;
            }
            KeyCode::Esc => {
                if self.search_query.is_some() {
                    self.search_query = None;
                    self.apply_filter();
                }
                return;
            }
            KeyCode::Char('?') => {
                self.modal = Some(Modal::Help);
                return;
            }
            KeyCode::Char('/') => {
                self.modal = Some(Modal::Search {
                    input: self.search_query.clone().unwrap_or_default(),
                });
                return;
            }
            KeyCode::Char('1') => {
                self.focused_pane = Pane::Workspaces;
                return;
            }
            KeyCode::Char('2') => {
                self.focused_pane = Pane::Files;
                return;
            }
            KeyCode::Char('3') => {
                self.focused_pane = Pane::Preview;
                return;
            }
            KeyCode::Tab => {
                // In workspaces pane, Tab toggles scope; in other panes, cycles panes
                if self.focused_pane == Pane::Workspaces {
                    self.toggle_scope();
                } else {
                    self.focused_pane = match self.focused_pane {
                        Pane::Workspaces => Pane::Files,
                        Pane::Files => Pane::Preview,
                        Pane::Preview => Pane::Workspaces,
                    };
                }
                return;
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.toggle_scope();
                return;
            }
            KeyCode::Char('R') => {
                self.reload_workspaces();
                self.set_status("Refreshed.", false);
                return;
            }
            _ => {}
        }

        // Pane-specific shortcuts
        match self.focused_pane {
            Pane::Workspaces => self.handle_workspaces_key(key),
            Pane::Files => self.handle_files_key(key),
            Pane::Preview => self.handle_preview_key(key),
        }
    }

    fn handle_workspaces_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.filtered_workspaces.is_empty()
                    && self.workspace_cursor + 1 < self.filtered_workspaces.len()
                {
                    self.workspace_cursor += 1;
                    self.reload_files();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.workspace_cursor > 0 {
                    self.workspace_cursor -= 1;
                    self.reload_files();
                }
            }
            KeyCode::Char('g') | KeyCode::Home => {
                if !self.filtered_workspaces.is_empty() {
                    self.workspace_cursor = 0;
                    self.reload_files();
                }
            }
            KeyCode::Char('G') | KeyCode::End => {
                if !self.filtered_workspaces.is_empty() {
                    self.workspace_cursor = self.filtered_workspaces.len().saturating_sub(1);
                    self.reload_files();
                }
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                if !self.files.is_empty() {
                    self.focused_pane = Pane::Files;
                }
            }
            KeyCode::Char('a') | KeyCode::Char('n') => {
                self.modal = Some(Modal::AddWorkspace {
                    input: String::new(),
                });
            }
            KeyCode::Char('A') => {
                if let Some(ws) = self.selected_workspace() {
                    let name = ws.name.clone();
                    match self.scope {
                        Scope::Active => {
                            self.modal = Some(Modal::ConfirmArchive { name });
                        }
                        Scope::Archive => {
                            let default_target =
                                workspace::strip_timestamp_suffix(&name).to_string();
                            self.modal = Some(Modal::ConfirmRestore {
                                name,
                                target_input: default_target,
                            });
                        }
                    }
                }
            }
            KeyCode::Char('r') | KeyCode::Char('m') => {
                if let Some(ws) = self.selected_workspace() {
                    self.modal = Some(Modal::RenameWorkspace {
                        scope: self.scope,
                        old_name: ws.name.clone(),
                        input: ws.name.clone(),
                    });
                }
            }
            KeyCode::Char('d') | KeyCode::Char('x') => {
                if let Some(ws) = self.selected_workspace() {
                    self.modal = Some(Modal::ConfirmDeleteWorkspace {
                        scope: self.scope,
                        name: ws.name.clone(),
                    });
                }
            }
            KeyCode::Char('y') => {
                if let Some(ws) = self.selected_workspace() {
                    let p = ws.path.display().to_string();
                    self.copy_to_clipboard(&p, "Copied workspace path to clipboard.");
                }
            }
            _ => {}
        }
    }

    fn handle_files_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.files.is_empty() && self.file_cursor + 1 < self.files.len() {
                    self.file_cursor += 1;
                    self.reload_preview();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.file_cursor > 0 {
                    self.file_cursor -= 1;
                    self.reload_preview();
                }
            }
            KeyCode::Char('g') | KeyCode::Home => {
                if !self.files.is_empty() {
                    self.file_cursor = 0;
                    self.reload_preview();
                }
            }
            KeyCode::Char('G') | KeyCode::End => {
                if !self.files.is_empty() {
                    self.file_cursor = self.files.len().saturating_sub(1);
                    self.reload_preview();
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.focused_pane = Pane::Workspaces;
            }
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                self.focused_pane = Pane::Preview;
            }
            KeyCode::Char('n') | KeyCode::Char('a') => {
                if self.selected_workspace().is_some() {
                    self.modal = Some(Modal::NewFile {
                        input: String::new(),
                    });
                }
            }
            KeyCode::Char('r') | KeyCode::Char('m') => {
                if let Some(file) = self.selected_file() {
                    self.modal = Some(Modal::RenameFile {
                        old_rel: file.rel_path.clone(),
                        input: file.rel_path.clone(),
                    });
                }
            }
            KeyCode::Char('d') | KeyCode::Char('x') => {
                if let Some(file) = self.selected_file() {
                    self.modal = Some(Modal::ConfirmDeleteFile {
                        rel_path: file.rel_path.clone(),
                    });
                }
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                if let Some(file) = self.selected_file() {
                    if !file.is_dir {
                        self.pending_editor = Some(file.full_path.clone());
                    }
                }
            }
            KeyCode::Char('y') => {
                if let Some(file) = self.selected_file() {
                    let p = file.full_path.display().to_string();
                    self.copy_to_clipboard(&p, "Copied file path to clipboard.");
                }
            }
            _ => {}
        }
    }

    fn handle_preview_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('j') | KeyCode::Down => {
                self.preview_scroll = self.preview_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.preview_scroll = self.preview_scroll.saturating_sub(1);
            }
            KeyCode::Char('d') | KeyCode::PageDown => {
                self.preview_scroll = self.preview_scroll.saturating_add(15);
            }
            KeyCode::Char('u') | KeyCode::PageUp => {
                self.preview_scroll = self.preview_scroll.saturating_sub(15);
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.preview_scroll = 0;
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.preview_scroll = 1000;
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.focused_pane = Pane::Files;
            }
            KeyCode::Char('v') => {
                self.preview_raw = !self.preview_raw;
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                if let Some(file) = self.selected_file() {
                    if !file.is_dir {
                        self.pending_editor = Some(file.full_path.clone());
                    }
                }
            }
            KeyCode::Char('y') => {
                if let Some(file) = self.selected_file() {
                    let p = file.full_path.display().to_string();
                    self.copy_to_clipboard(&p, "Copied file path to clipboard.");
                }
            }
            KeyCode::Char('Y') if !self.preview_content.is_empty() => {
                self.copy_to_clipboard(
                    &self.preview_content.clone(),
                    "Copied preview content to clipboard.",
                );
            }
            _ => {}
        }
    }

    fn handle_modal_key(&mut self, key: KeyCode) {
        let modal = match self.modal.take() {
            Some(m) => m,
            None => return,
        };

        match modal {
            Modal::InitDirs {
                add_to_gitignore,
                git_detected,
            } => match key {
                KeyCode::Char(' ') if git_detected => {
                    self.modal = Some(Modal::InitDirs {
                        add_to_gitignore: !add_to_gitignore,
                        git_detected,
                    });
                }
                KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('i') => {
                    match workspace::init_dirs(&self.config) {
                        Ok(()) => {
                            let mut msg =
                                "Initialized workspace and archive directories.".to_string();
                            if add_to_gitignore {
                                match workspace::add_to_gitignore(
                                    &self.config.root_dir,
                                    &["workspace/", "archive/"],
                                ) {
                                    Ok(added) => {
                                        if !added.is_empty() {
                                            msg.push_str(" Added to .gitignore.");
                                        }
                                    }
                                    Err(e) => {
                                        self.set_status(
                                            format!(
                                                "Initialized dirs, but failed to update .gitignore: {}",
                                                e
                                            ),
                                            true,
                                        );
                                        self.reload_workspaces();
                                        return;
                                    }
                                }
                            }
                            self.set_status(msg, false);
                            self.reload_workspaces();
                        }
                        Err(e) => {
                            self.set_status(format!("Failed to initialize: {}", e), true);
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                _ => {
                    self.modal = Some(Modal::InitDirs {
                        add_to_gitignore,
                        git_detected,
                    });
                }
            },
            Modal::Help => match key {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter | KeyCode::Char('?') => {}
                _ => {
                    self.modal = Some(Modal::Help);
                }
            },
            Modal::Search { mut input } => match key {
                KeyCode::Esc => {
                    // Cancel search
                }
                KeyCode::Enter => {
                    if input.trim().is_empty() {
                        self.search_query = None;
                    } else {
                        self.search_query = Some(input);
                    }
                    self.apply_filter();
                }
                KeyCode::Backspace => {
                    input.pop();
                    self.modal = Some(Modal::Search { input });
                }
                KeyCode::Char(c) => {
                    input.push(c);
                    self.modal = Some(Modal::Search { input });
                }
                _ => {
                    self.modal = Some(Modal::Search { input });
                }
            },
            Modal::AddWorkspace { mut input } => match key {
                KeyCode::Esc => {}
                KeyCode::Enter => {
                    let trimmed = input.trim();
                    match workspace::add_workspace(&self.config, trimmed) {
                        Ok(p) => {
                            self.set_status(format!("Created {}", p.display()), false);
                            self.scope = Scope::Active;
                            self.reload_workspaces();
                            // Select new workspace
                            if let Some(idx) = self
                                .filtered_workspaces
                                .iter()
                                .position(|w| w.name == trimmed)
                            {
                                self.workspace_cursor = idx;
                                self.reload_files();
                            }
                        }
                        Err(e) => {
                            self.set_status(format!("Error: {}", e), true);
                        }
                    }
                }
                KeyCode::Backspace => {
                    input.pop();
                    self.modal = Some(Modal::AddWorkspace { input });
                }
                KeyCode::Char(c) => {
                    input.push(c);
                    self.modal = Some(Modal::AddWorkspace { input });
                }
                _ => {
                    self.modal = Some(Modal::AddWorkspace { input });
                }
            },
            Modal::RenameWorkspace {
                scope,
                old_name,
                mut input,
            } => match key {
                KeyCode::Esc => {}
                KeyCode::Enter => {
                    let trimmed = input.trim();
                    if trimmed != old_name {
                        match workspace::rename_workspace(&self.config, scope, &old_name, trimmed) {
                            Ok(dst) => {
                                self.set_status(format!("Renamed to {}", dst.display()), false);
                                self.reload_workspaces();
                                if let Some(idx) = self
                                    .filtered_workspaces
                                    .iter()
                                    .position(|w| w.name == trimmed)
                                {
                                    self.workspace_cursor = idx;
                                    self.reload_files();
                                }
                            }
                            Err(e) => {
                                self.set_status(format!("Error: {}", e), true);
                            }
                        }
                    }
                }
                KeyCode::Backspace => {
                    input.pop();
                    self.modal = Some(Modal::RenameWorkspace {
                        scope,
                        old_name,
                        input,
                    });
                }
                KeyCode::Char(c) => {
                    input.push(c);
                    self.modal = Some(Modal::RenameWorkspace {
                        scope,
                        old_name,
                        input,
                    });
                }
                _ => {
                    self.modal = Some(Modal::RenameWorkspace {
                        scope,
                        old_name,
                        input,
                    });
                }
            },
            Modal::ConfirmArchive { name } => match key {
                KeyCode::Enter | KeyCode::Char('y') => {
                    match workspace::archive_workspace(&self.config, &name) {
                        Ok((src, dest)) => {
                            self.set_status(
                                format!("Archived {} -> {}", src.display(), dest.display()),
                                false,
                            );
                            self.reload_workspaces();
                        }
                        Err(e) => {
                            self.set_status(format!("Error: {}", e), true);
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Char('n') => {}
                _ => {
                    self.modal = Some(Modal::ConfirmArchive { name });
                }
            },
            Modal::ConfirmRestore {
                name,
                mut target_input,
            } => match key {
                KeyCode::Enter => {
                    let trimmed = target_input.trim();
                    let target = if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed)
                    };
                    match workspace::restore_workspace(&self.config, &name, target) {
                        Ok((src, dest)) => {
                            self.set_status(
                                format!("Restored {} -> {}", src.display(), dest.display()),
                                false,
                            );
                            self.scope = Scope::Active;
                            self.reload_workspaces();
                            if let Some(target_str) = target {
                                if let Some(idx) = self
                                    .filtered_workspaces
                                    .iter()
                                    .position(|w| w.name == target_str)
                                {
                                    self.workspace_cursor = idx;
                                    self.reload_files();
                                }
                            }
                        }
                        Err(e) => {
                            self.set_status(format!("Error: {}", e), true);
                        }
                    }
                }
                KeyCode::Esc => {}
                KeyCode::Backspace => {
                    target_input.pop();
                    self.modal = Some(Modal::ConfirmRestore { name, target_input });
                }
                KeyCode::Char(c) => {
                    target_input.push(c);
                    self.modal = Some(Modal::ConfirmRestore { name, target_input });
                }
                _ => {
                    self.modal = Some(Modal::ConfirmRestore { name, target_input });
                }
            },
            Modal::ConfirmDeleteWorkspace { scope, name } => match key {
                KeyCode::Enter | KeyCode::Char('y') => {
                    match workspace::delete_workspace(&self.config, scope, &name) {
                        Ok(()) => {
                            self.set_status(format!("Deleted workspace '{}'.", name), false);
                            self.reload_workspaces();
                        }
                        Err(e) => {
                            self.set_status(format!("Error: {}", e), true);
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Char('n') => {}
                _ => {
                    self.modal = Some(Modal::ConfirmDeleteWorkspace { scope, name });
                }
            },
            Modal::NewFile { mut input } => match key {
                KeyCode::Esc => {}
                KeyCode::Enter => {
                    let trimmed = input.trim();
                    if let Some(ws) = self.selected_workspace() {
                        match workspace::create_workspace_file(&ws.path, trimmed, "") {
                            Ok(path) => {
                                self.set_status(format!("Created {}", path.display()), false);
                                self.reload_files();
                                if let Some(idx) =
                                    self.files.iter().position(|f| f.rel_path == trimmed)
                                {
                                    self.file_cursor = idx;
                                    self.reload_preview();
                                }
                            }
                            Err(e) => {
                                self.set_status(format!("Error: {}", e), true);
                            }
                        }
                    }
                }
                KeyCode::Backspace => {
                    input.pop();
                    self.modal = Some(Modal::NewFile { input });
                }
                KeyCode::Char(c) => {
                    input.push(c);
                    self.modal = Some(Modal::NewFile { input });
                }
                _ => {
                    self.modal = Some(Modal::NewFile { input });
                }
            },
            Modal::RenameFile {
                old_rel,
                mut input,
            } => match key {
                KeyCode::Esc => {}
                KeyCode::Enter => {
                    let trimmed = input.trim();
                    if let Some(ws) = self.selected_workspace() {
                        match workspace::rename_workspace_file(&ws.path, &old_rel, trimmed) {
                            Ok(dst) => {
                                self.set_status(format!("Renamed to {}", dst.display()), false);
                                self.reload_files();
                                if let Some(idx) =
                                    self.files.iter().position(|f| f.rel_path == trimmed)
                                {
                                    self.file_cursor = idx;
                                    self.reload_preview();
                                }
                            }
                            Err(e) => {
                                self.set_status(format!("Error: {}", e), true);
                            }
                        }
                    }
                }
                KeyCode::Backspace => {
                    input.pop();
                    self.modal = Some(Modal::RenameFile { old_rel, input });
                }
                KeyCode::Char(c) => {
                    input.push(c);
                    self.modal = Some(Modal::RenameFile { old_rel, input });
                }
                _ => {
                    self.modal = Some(Modal::RenameFile { old_rel, input });
                }
            },
            Modal::ConfirmDeleteFile { rel_path } => match key {
                KeyCode::Enter | KeyCode::Char('y') => {
                    if let Some(ws) = self.selected_workspace() {
                        match workspace::delete_workspace_file(&ws.path, &rel_path) {
                            Ok(()) => {
                                self.set_status(format!("Deleted '{}'.", rel_path), false);
                                self.reload_files();
                            }
                            Err(e) => {
                                self.set_status(format!("Error: {}", e), true);
                            }
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Char('n') => {}
                _ => {
                    self.modal = Some(Modal::ConfirmDeleteFile { rel_path });
                }
            },
        }
    }

    pub fn complete_external_editor(&mut self) -> Result<()> {
        self.pending_editor = None;
        self.reload_files();
        Ok(())
    }
}
