use chrono::{DateTime, Local};
use std::time::SystemTime;

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_systime(systime: SystemTime) -> String {
    let dt: DateTime<Local> = systime.into();
    dt.format("%Y-%m-%d %H:%M").to_string()
}

pub fn format_relative_time(systime: SystemTime) -> String {
    let now = SystemTime::now();
    let duration = match now.duration_since(systime) {
        Ok(d) => d,
        Err(_) => return "just now".to_string(),
    };

    let secs = duration.as_secs();
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        let mins = secs / 60;
        format!("{}m ago", mins)
    } else if secs < 86400 {
        let hours = secs / 3600;
        format!("{}h ago", hours)
    } else {
        let days = secs / 86400;
        if days < 30 {
            format!("{}d ago", days)
        } else {
            let dt: DateTime<Local> = systime.into();
            dt.format("%b %d").to_string()
        }
    }
}

pub fn word_and_line_count(text: &str) -> (usize, usize) {
    if text.is_empty() {
        return (0, 0);
    }
    let lines = text.lines().count();
    let words = text.split_whitespace().count();
    (words, lines)
}
