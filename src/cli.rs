use crate::config::Config;
use crate::format::{format_bytes, format_systime};
use crate::models::Scope;
use crate::workspace;
use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "work",
    about = "A fast CLI and TUI workspace manager for Octopus AI agent harnesses",
    version
)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        help = "Custom root directory (defaults to current directory or WORK_ROOT)"
    )]
    pub root: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Launch the interactive 3-pane TUI dashboard
    Tui,

    /// Initialize ./workspace and ./archive directories
    Init,

    /// Create a workspace under ./workspace
    Add {
        #[arg(help = "Name of the workspace to create")]
        name: Option<String>,
    },

    /// Move a workspace from ./workspace to ./archive
    Archive {
        #[arg(help = "Name of the workspace to archive")]
        name: Option<String>,
    },

    /// Move a workspace from ./archive back to ./workspace
    Restore {
        #[arg(help = "Name of the archived workspace to restore")]
        name: Option<String>,

        #[arg(
            long,
            help = "Target name in ./workspace (defaults to stripping timestamp suffix)"
        )]
        target: Option<String>,
    },

    /// Rename a workspace
    Rename {
        #[arg(help = "Current workspace name")]
        old_name: String,

        #[arg(help = "New workspace name")]
        new_name: String,

        #[arg(long, help = "Rename in archive instead of active workspace")]
        archived: bool,
    },

    /// Delete a workspace
    #[command(alias = "rm")]
    Delete {
        #[arg(help = "Name of the workspace to delete")]
        name: Option<String>,

        #[arg(long, help = "Delete from archive instead of active workspace")]
        archived: bool,

        #[arg(short = 'y', long = "yes", help = "Skip confirmation prompt")]
        yes: bool,
    },

    /// List active and/or archived workspaces
    #[command(alias = "ls")]
    List {
        #[arg(long, help = "List archived workspaces")]
        archived: bool,

        #[arg(long, help = "List both active and archived workspaces")]
        all: bool,

        #[arg(long, help = "Output in JSON format")]
        json: bool,
    },

    /// Print the filesystem path to a workspace
    Path {
        #[arg(help = "Name of the workspace")]
        name: Option<String>,

        #[arg(long, help = "Look in archive instead of active workspace")]
        archived: bool,
    },
}

pub fn run_cli(cli: Cli, config: Config) -> Result<()> {
    match cli.command {
        Some(Commands::Tui) | None => {
            // Handled in main.rs to launch TUI
            unreachable!("Tui command should be handled by caller");
        }
        Some(Commands::Init) => {
            workspace::init_dirs(&config)?;
            println!("Initialized workspace directories:");
            println!("  Created {}", config.workspace_dir.display());
            println!("  Created {}", config.archive_dir.display());

            if workspace::is_git_repo(&config.root_dir) && is_interactive_terminal() {
                if prompt_confirm("Add workspace/ and archive/ to .gitignore? [Y/n]: ", true)? {
                    let added = workspace::add_to_gitignore(&config.root_dir, &["workspace/", "archive/"])?;
                    if !added.is_empty() {
                        println!("  Added to .gitignore: {}", added.join(", "));
                    } else {
                        println!("  Patterns already present in .gitignore");
                    }
                }
            }
            Ok(())
        }
        Some(Commands::Add { name }) => {
            let ws_name = match name {
                Some(n) if !n.trim().is_empty() => n,
                _ => prompt_input("Workspace name: ")?,
            };

            let target = workspace::add_workspace(&config, &ws_name)?;
            println!("Created {}", target.display());
            Ok(())
        }
        Some(Commands::Archive { name }) => {
            let ws_name = match name {
                Some(n) if !n.trim().is_empty() => n,
                _ => {
                    let items = workspace::list_workspaces(&config, Scope::Active)?;
                    if items.is_empty() {
                        bail!("No workspaces found in {}.", config.workspace_dir.display());
                    }
                    let names: Vec<String> = items.into_iter().map(|w| w.name).collect();
                    prompt_select("Archive workspace:", &names)?
                }
            };

            let (src, dest) = workspace::archive_workspace(&config, &ws_name)?;
            println!("Archived {} -> {}", src.display(), dest.display());
            Ok(())
        }
        Some(Commands::Restore { name, target }) => {
            let ws_name = match name {
                Some(n) if !n.trim().is_empty() => n,
                _ => {
                    let items = workspace::list_workspaces(&config, Scope::Archive)?;
                    if items.is_empty() {
                        bail!(
                            "No archived workspaces found in {}.",
                            config.archive_dir.display()
                        );
                    }
                    let names: Vec<String> = items.into_iter().map(|w| w.name).collect();
                    prompt_select("Restore workspace:", &names)?
                }
            };

            let (src, dest) =
                workspace::restore_workspace(&config, &ws_name, target.as_deref())?;
            println!("Restored {} -> {}", src.display(), dest.display());
            Ok(())
        }
        Some(Commands::Rename {
            old_name,
            new_name,
            archived,
        }) => {
            let scope = if archived {
                Scope::Archive
            } else {
                Scope::Active
            };
            let dst = workspace::rename_workspace(&config, scope, &old_name, &new_name)?;
            println!("Renamed {} -> {}", old_name, dst.display());
            Ok(())
        }
        Some(Commands::Delete {
            name,
            archived,
            yes,
        }) => {
            let scope = if archived {
                Scope::Archive
            } else {
                Scope::Active
            };
            let ws_name = match name {
                Some(n) if !n.trim().is_empty() => n,
                _ => {
                    let items = workspace::list_workspaces(&config, scope)?;
                    if items.is_empty() {
                        bail!("No workspaces found in {}.", config.scope_dir(scope).display());
                    }
                    let names: Vec<String> = items.into_iter().map(|w| w.name).collect();
                    prompt_select(&format!("Delete workspace in {}:", scope), &names)?
                }
            };

            if !yes {
                let prompt = format!(
                    "Are you sure you want to permanently delete workspace '{}' in {}? [y/N]: ",
                    ws_name, scope
                );
                let answer = prompt_input(&prompt)?;
                if !answer.eq_ignore_ascii_case("y") && !answer.eq_ignore_ascii_case("yes") {
                    println!("Aborted.");
                    return Ok(());
                }
            }

            workspace::delete_workspace(&config, scope, &ws_name)?;
            println!("Deleted workspace '{}'.", ws_name);
            Ok(())
        }
        Some(Commands::List {
            archived,
            all,
            json,
        }) => {
            workspace::require_dirs(&config)?;

            let scopes = if all {
                vec![Scope::Active, Scope::Archive]
            } else if archived {
                vec![Scope::Archive]
            } else {
                vec![Scope::Active]
            };

            let mut all_results = Vec::new();
            for scope in scopes {
                let ws = workspace::list_workspaces(&config, scope)?;
                all_results.extend(ws);
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&all_results)?);
                return Ok(());
            }

            if all_results.is_empty() {
                println!("No workspaces found.");
                return Ok(());
            }

            println!(
                "{:<10} {:<30} {:<10} {:<12} {:<16}",
                "SCOPE", "WORKSPACE", "FILES", "SIZE", "MODIFIED"
            );
            println!("{}", "─".repeat(80));

            for w in all_results {
                let modified = w
                    .modified_at
                    .map(format_systime)
                    .unwrap_or_else(|| "-".to_string());
                println!(
                    "{:<10} {:<30} {:<10} {:<12} {:<16}",
                    w.scope.label(),
                    w.name,
                    w.file_count,
                    format_bytes(w.total_bytes),
                    modified
                );
            }

            Ok(())
        }
        Some(Commands::Path { name, archived }) => {
            workspace::require_dirs(&config)?;
            let scope = if archived {
                Scope::Archive
            } else {
                Scope::Active
            };
            let dir = config.scope_dir(scope);

            let ws_name = match name {
                Some(n) if !n.trim().is_empty() => n,
                _ => {
                    let items = workspace::list_workspaces(&config, scope)?;
                    if items.is_empty() {
                        bail!("No workspaces found in {}.", dir.display());
                    }
                    let names: Vec<String> = items.into_iter().map(|w| w.name).collect();
                    prompt_select(&format!("Select workspace in {}:", scope), &names)?
                }
            };

            let p = dir.join(&ws_name);
            if !p.is_dir() {
                bail!("Workspace '{}' does not exist at {}.", ws_name, p.display());
            }
            println!("{}", p.display());
            Ok(())
        }
    }
}

fn prompt_input(prompt: &str) -> Result<String> {
    eprint!("{}", prompt);
    io::stderr().flush()?;
    let mut line = String::new();
    let stdin = io::stdin();
    stdin.lock().read_line(&mut line)?;
    let trimmed = line.trim().to_string();
    if trimmed.is_empty() {
        bail!("Input cannot be empty.");
    }
    Ok(trimmed)
}

fn prompt_select(prompt: &str, options: &[String]) -> Result<String> {
    if options.is_empty() {
        bail!("No options available to select.");
    }

    if options.len() == 1 {
        return Ok(options[0].clone());
    }

    eprintln!("{}", prompt);
    for (i, opt) in options.iter().enumerate() {
        eprintln!("  {}) {}", i + 1, opt);
    }

    loop {
        eprint!("Enter choice (1-{}): ", options.len());
        io::stderr().flush()?;
        let mut line = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut line)?;
        let trimmed = line.trim();
        if let Ok(num) = trimmed.parse::<usize>() {
            if num >= 1 && num <= options.len() {
                return Ok(options[num - 1].clone());
            }
        }
        eprintln!("Invalid selection. Please choose a number between 1 and {}.", options.len());
    }
}

fn is_interactive_terminal() -> bool {
    if !io::stdin().is_terminal() {
        return false;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(s) = exe.to_str() {
            if s.contains("_tests") || s.contains("/deps/") {
                return false;
            }
        }
    }
    true
}

fn prompt_confirm_from<R: BufRead>(mut reader: R, prompt: &str, default_yes: bool) -> Result<bool> {
    eprint!("{}", prompt);
    io::stderr().flush()?;
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Ok(default_yes);
    }
    let trimmed = line.trim().to_lowercase();
    if trimmed.is_empty() {
        Ok(default_yes)
    } else {
        Ok(trimmed == "y" || trimmed == "yes")
    }
}

fn prompt_confirm(prompt: &str, default_yes: bool) -> Result<bool> {
    prompt_confirm_from(io::stdin().lock(), prompt, default_yes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_confirm_from() {
        assert!(prompt_confirm_from("y\n".as_bytes(), "prompt", false).unwrap());
        assert!(prompt_confirm_from("Y\n".as_bytes(), "prompt", false).unwrap());
        assert!(prompt_confirm_from("yes\n".as_bytes(), "prompt", false).unwrap());
        assert!(prompt_confirm_from("\n".as_bytes(), "prompt", true).unwrap());
        assert!(!prompt_confirm_from("\n".as_bytes(), "prompt", false).unwrap());
        assert!(!prompt_confirm_from("n\n".as_bytes(), "prompt", true).unwrap());
        assert!(!prompt_confirm_from("no\n".as_bytes(), "prompt", true).unwrap());
    }
}
