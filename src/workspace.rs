use crate::config::Config;
use crate::models::{Scope, WorkspaceFile, WorkspaceInfo};
use anyhow::{bail, Context, Result};
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

pub fn validate_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        bail!("Workspace name cannot be empty.");
    }
    if name == "." || name == ".." {
        bail!("Invalid workspace name: '{}'.", name);
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        bail!("Workspace name cannot contain '/' or '..'.");
    }
    Ok(())
}

pub fn require_dirs(config: &Config) -> Result<()> {
    if !config.workspace_dir.is_dir() {
        bail!(
            "'{}' does not exist in {}.",
            config.workspace_dir.display(),
            config.root_dir.display()
        );
    }
    if !config.archive_dir.is_dir() {
        bail!(
            "'{}' does not exist in {}.",
            config.archive_dir.display(),
            config.root_dir.display()
        );
    }
    Ok(())
}

pub fn init_dirs(config: &Config) -> Result<()> {
    fs::create_dir_all(&config.workspace_dir)
        .with_context(|| format!("Failed to create {}", config.workspace_dir.display()))?;
    fs::create_dir_all(&config.archive_dir)
        .with_context(|| format!("Failed to create {}", config.archive_dir.display()))?;
    Ok(())
}

pub fn add_workspace(config: &Config, name: &str) -> Result<PathBuf> {
    require_dirs(config)?;
    validate_name(name)?;
    let target = config.workspace_dir.join(name);
    if target.exists() {
        bail!("Workspace '{}' already exists at {}.", name, target.display());
    }
    fs::create_dir(&target)
        .with_context(|| format!("Failed to create directory {}", target.display()))?;
    Ok(target)
}

pub fn archive_workspace(config: &Config, name: &str) -> Result<(PathBuf, PathBuf)> {
    require_dirs(config)?;
    validate_name(name)?;
    let src = config.workspace_dir.join(name);
    if !src.is_dir() {
        bail!(
            "Workspace '{}' does not exist in {}.",
            name,
            config.workspace_dir.display()
        );
    }

    let mut dest = config.archive_dir.join(name);
    if dest.exists() {
        let ts = Local::now().format("%Y%m%d-%H%M%S").to_string();
        dest = config.archive_dir.join(format!("{}-{}", name, ts));
    }

    move_dir(&src, &dest)?;
    Ok((src, dest))
}

pub fn strip_timestamp_suffix(name: &str) -> &str {
    // Pattern: -YYYYMMDD-HHMMSS (16 chars: '-' + 8 digits + '-' + 6 digits)
    if name.len() > 16 {
        let suffix = &name[name.len() - 16..];
        let bytes = suffix.as_bytes();
        if bytes[0] == b'-'
            && bytes[1..9].iter().all(|b| b.is_ascii_digit())
            && bytes[9] == b'-'
            && bytes[10..16].iter().all(|b| b.is_ascii_digit())
        {
            return &name[..name.len() - 16];
        }
    }
    name
}

pub fn restore_workspace(
    config: &Config,
    archived_name: &str,
    target_name: Option<&str>,
) -> Result<(PathBuf, PathBuf)> {
    require_dirs(config)?;
    validate_name(archived_name)?;

    let src = config.archive_dir.join(archived_name);
    if !src.is_dir() {
        bail!(
            "Workspace '{}' does not exist in {}.",
            archived_name,
            config.archive_dir.display()
        );
    }

    let dest = if let Some(target) = target_name {
        validate_name(target)?;
        let d = config.workspace_dir.join(target);
        if d.exists() {
            bail!("Workspace '{}' already exists at {}.", target, d.display());
        }
        d
    } else {
        let stripped = strip_timestamp_suffix(archived_name);
        let base_dest = config.workspace_dir.join(stripped);
        if !base_dest.exists() {
            base_dest
        } else {
            let full_dest = config.workspace_dir.join(archived_name);
            if !full_dest.exists() {
                full_dest
            } else {
                bail!(
                    "Workspace '{}' already exists at {}.",
                    stripped,
                    base_dest.display()
                );
            }
        }
    };

    move_dir(&src, &dest)?;
    Ok((src, dest))
}

pub fn rename_workspace(
    config: &Config,
    scope: Scope,
    old_name: &str,
    new_name: &str,
) -> Result<PathBuf> {
    require_dirs(config)?;
    validate_name(old_name)?;
    validate_name(new_name)?;

    let dir = config.scope_dir(scope);
    let src = dir.join(old_name);
    if !src.is_dir() {
        bail!("Workspace '{}' does not exist in {}.", old_name, dir.display());
    }

    let dest = dir.join(new_name);
    if dest.exists() {
        bail!("Workspace '{}' already exists at {}.", new_name, dest.display());
    }

    move_dir(&src, &dest)?;
    Ok(dest)
}

pub fn delete_workspace(config: &Config, scope: Scope, name: &str) -> Result<()> {
    require_dirs(config)?;
    validate_name(name)?;

    let dir = config.scope_dir(scope);
    let target = dir.join(name);
    if !target.is_dir() {
        bail!("Workspace '{}' does not exist in {}.", name, dir.display());
    }

    fs::remove_dir_all(&target)
        .with_context(|| format!("Failed to delete directory {}", target.display()))?;
    Ok(())
}

pub fn list_workspaces(config: &Config, scope: Scope) -> Result<Vec<WorkspaceInfo>> {
    let dir = config.scope_dir(scope);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut list = Vec::new();
    let entries = fs::read_dir(&dir)
        .with_context(|| format!("Failed to read directory {}", dir.display()))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            let path = entry.path();
            let name = match entry.file_name().into_string() {
                Ok(n) => n,
                Err(_) => continue,
            };

            // Skip hidden directories like .git
            if name.starts_with('.') {
                continue;
            }

            let (file_count, total_bytes) = compute_dir_stats(&path);
            let modified_at = entry.metadata().ok().and_then(|m| m.modified().ok());

            list.push(WorkspaceInfo {
                name,
                path,
                scope,
                file_count,
                total_bytes,
                modified_at,
            });
        }
    }

    list.sort_by_key(|a| a.name.to_lowercase());
    Ok(list)
}

fn compute_dir_stats(dir: &Path) -> (usize, u64) {
    let mut count = 0;
    let mut total_bytes = 0;

    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&current) {
            for entry in entries.flatten() {
                let p = entry.path();
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.starts_with('.') {
                    continue;
                }

                if let Ok(meta) = entry.metadata() {
                    if meta.is_dir() {
                        stack.push(p);
                    } else if meta.is_file() {
                        count += 1;
                        total_bytes += meta.len();
                    }
                }
            }
        }
    }

    (count, total_bytes)
}

pub fn list_workspace_files(ws_path: &Path) -> Result<Vec<WorkspaceFile>> {
    if !ws_path.is_dir() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let mut stack = vec![ws_path.to_path_buf()];

    while let Some(current) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&current) {
            for entry in entries.flatten() {
                let p = entry.path();
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.starts_with('.') {
                    continue;
                }

                if let Ok(meta) = entry.metadata() {
                    let is_dir = meta.is_dir();
                    let rel_path = match p.strip_prefix(ws_path) {
                        Ok(rel) => rel.to_string_lossy().to_string(),
                        Err(_) => p.to_string_lossy().to_string(),
                    };

                    files.push(WorkspaceFile {
                        rel_path,
                        full_path: p.clone(),
                        is_dir,
                        size_bytes: if is_dir { 0 } else { meta.len() },
                        modified_at: meta.modified().ok(),
                    });

                    if is_dir {
                        stack.push(p);
                    }
                }
            }
        }
    }

    // Sort: prompt.md / README.md near top, then alphabetical
    files.sort_by(|a, b| {
        let score_a = file_sort_priority(&a.rel_path, a.is_dir);
        let score_b = file_sort_priority(&b.rel_path, b.is_dir);
        if score_a != score_b {
            score_a.cmp(&score_b)
        } else {
            a.rel_path.to_lowercase().cmp(&b.rel_path.to_lowercase())
        }
    });

    Ok(files)
}

fn file_sort_priority(rel_path: &str, is_dir: bool) -> usize {
    let lower = rel_path.to_lowercase();
    if lower == "prompt.md" {
        0
    } else if lower == "readme.md" {
        1
    } else if is_dir {
        2
    } else {
        3
    }
}

pub fn read_file_preview(path: &Path, max_bytes: usize) -> Result<String> {
    if !path.is_file() {
        return Ok("Directory or not a regular file.".to_string());
    }

    let data = fs::read(path)?;
    if data.is_empty() {
        return Ok("(empty file)".to_string());
    }

    // Check for null bytes in initial chunk to detect binary
    let check_len = data.len().min(1024);
    if data[..check_len].contains(&0) {
        return Ok(format!("[Binary file - {} bytes]", data.len()));
    }

    let slice = if data.len() > max_bytes {
        &data[..max_bytes]
    } else {
        &data[..]
    };

    let text = String::from_utf8_lossy(slice).to_string();
    if data.len() > max_bytes {
        Ok(format!("{}\n\n... [truncated - showing first {} of {} bytes]", text, max_bytes, data.len()))
    } else {
        Ok(text)
    }
}

pub fn create_workspace_file(ws_path: &Path, rel_path: &str, content: &str) -> Result<PathBuf> {
    if rel_path.trim().is_empty() {
        bail!("File name cannot be empty.");
    }
    if rel_path.contains("..") {
        bail!("File path cannot contain '..'.");
    }

    let target = ws_path.join(rel_path);
    if target.exists() {
        bail!("File '{}' already exists.", rel_path);
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&target, content)
        .with_context(|| format!("Failed to write file {}", target.display()))?;
    Ok(target)
}

pub fn delete_workspace_file(ws_path: &Path, rel_path: &str) -> Result<()> {
    let target = ws_path.join(rel_path);
    if !target.exists() {
        bail!("File '{}' does not exist.", rel_path);
    }

    if target.is_dir() {
        fs::remove_dir_all(&target)
            .with_context(|| format!("Failed to delete directory {}", target.display()))?;
    } else {
        fs::remove_file(&target)
            .with_context(|| format!("Failed to delete file {}", target.display()))?;
    }
    Ok(())
}

pub fn rename_workspace_file(ws_path: &Path, old_rel: &str, new_rel: &str) -> Result<PathBuf> {
    if new_rel.trim().is_empty() {
        bail!("New file name cannot be empty.");
    }
    if new_rel.contains("..") {
        bail!("New file path cannot contain '..'.");
    }

    let src = ws_path.join(old_rel);
    let dst = ws_path.join(new_rel);

    if !src.exists() {
        bail!("File '{}' does not exist.", old_rel);
    }
    if dst.exists() {
        bail!("File '{}' already exists.", new_rel);
    }

    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::rename(&src, &dst)
        .with_context(|| format!("Failed to rename {} to {}", src.display(), dst.display()))?;
    Ok(dst)
}

fn move_dir(src: &Path, dest: &Path) -> Result<()> {
    if let Err(_e) = fs::rename(src, dest) {
        // Fallback for cross-device moves if necessary
        copy_dir_all(src, dest)?;
        fs::remove_dir_all(src)?;
    }
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_validate_name() {
        assert!(validate_name("").is_err());
        assert!(validate_name("   ").is_err());
        assert!(validate_name(".").is_err());
        assert!(validate_name("..").is_err());
        assert!(validate_name("foo/bar").is_err());
        assert!(validate_name("foo\\bar").is_err());
        assert!(validate_name("foo..bar").is_err());
        assert!(validate_name("my-workspace").is_ok());
        assert!(validate_name("q4_task_123").is_ok());
    }

    #[test]
    fn test_strip_timestamp_suffix() {
        assert_eq!(strip_timestamp_suffix("my-task"), "my-task");
        assert_eq!(
            strip_timestamp_suffix("my-task-20261120-091500"),
            "my-task"
        );
        assert_eq!(
            strip_timestamp_suffix("task-with-dashes-20260101-120000"),
            "task-with-dashes"
        );
        // Not a timestamp suffix:
        assert_eq!(
            strip_timestamp_suffix("task-2026112-091500"),
            "task-2026112-091500"
        );
    }

    #[test]
    fn test_lifecycle_add_archive_restore_delete() {
        let tmp = tempdir().unwrap();
        let config = Config::new(tmp.path());

        // Missing dirs
        assert!(require_dirs(&config).is_err());

        // Init dirs
        init_dirs(&config).unwrap();
        assert!(require_dirs(&config).is_ok());

        // Add
        let ws_path = add_workspace(&config, "proj-a").unwrap();
        assert!(ws_path.is_dir());
        assert!(add_workspace(&config, "proj-a").is_err()); // duplicate

        // Add a file
        create_workspace_file(&ws_path, "prompt.md", "Hello world").unwrap();

        // List active
        let active = list_workspaces(&config, Scope::Active).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "proj-a");
        assert_eq!(active[0].file_count, 1);

        // Archive
        let (_, arch_path) = archive_workspace(&config, "proj-a").unwrap();
        assert!(!ws_path.exists());
        assert!(arch_path.is_dir());

        // Restore
        let (_, restored_path) = restore_workspace(&config, "proj-a", None).unwrap();
        assert!(restored_path.is_dir());
        assert_eq!(restored_path, ws_path);

        // Delete
        delete_workspace(&config, Scope::Active, "proj-a").unwrap();
        assert!(!ws_path.exists());
    }

    #[test]
    fn test_archive_collision_and_restore_stripping() {
        let tmp = tempdir().unwrap();
        let config = Config::new(tmp.path());
        init_dirs(&config).unwrap();

        // Create in workspace
        add_workspace(&config, "test-ws").unwrap();
        // Create duplicate in archive
        fs::create_dir(config.archive_dir.join("test-ws")).unwrap();

        // Archiving should generate timestamp suffix
        let (_, arch_path) = archive_workspace(&config, "test-ws").unwrap();
        let arch_name = arch_path.file_name().unwrap().to_str().unwrap();
        assert!(arch_name.starts_with("test-ws-20"));

        // Now restoring without target should strip timestamp suffix and restore to test-ws!
        let (_, restored_path) = restore_workspace(&config, arch_name, None).unwrap();
        assert_eq!(
            restored_path.file_name().unwrap().to_str().unwrap(),
            "test-ws"
        );
    }
}
