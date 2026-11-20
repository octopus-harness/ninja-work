use crate::models::Scope;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub root_dir: PathBuf,
    pub workspace_dir: PathBuf,
    pub archive_dir: PathBuf,
}

impl Config {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let root_dir = root.as_ref().to_path_buf();
        let workspace_dir = root_dir.join("workspace");
        let archive_dir = root_dir.join("archive");
        Self {
            root_dir,
            workspace_dir,
            archive_dir,
        }
    }

    pub fn from_env_or_cwd(override_root: Option<&Path>) -> Self {
        if let Some(r) = override_root {
            return Self::new(r);
        }

        if let Ok(env_root) = env::var("WORK_ROOT") {
            if !env_root.trim().is_empty() {
                return Self::new(PathBuf::from(env_root.trim()));
            }
        }

        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::new(cwd)
    }

    pub fn scope_dir(&self, scope: Scope) -> PathBuf {
        match scope {
            Scope::Active => self.workspace_dir.clone(),
            Scope::Archive => self.archive_dir.clone(),
        }
    }
}
