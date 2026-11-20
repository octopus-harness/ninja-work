use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    Active,
    Archive,
}

impl Scope {
    pub fn label(&self) -> &'static str {
        match self {
            Scope::Active => "Active",
            Scope::Archive => "Archive",
        }
    }

    pub fn folder_name(&self) -> &'static str {
        match self {
            Scope::Active => "workspace",
            Scope::Archive => "archive",
        }
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub name: String,
    pub path: PathBuf,
    pub scope: Scope,
    pub file_count: usize,
    pub total_bytes: u64,
    pub modified_at: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceFile {
    pub rel_path: String,
    pub full_path: PathBuf,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub modified_at: Option<SystemTime>,
}
