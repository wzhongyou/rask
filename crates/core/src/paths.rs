use std::path::PathBuf;

/// ~/.rask/
pub fn rask_home() -> Option<PathBuf> {
    dirs::home_dir().map(|d| d.join(".rask"))
}

pub fn config_path() -> Option<PathBuf> {
    rask_home().map(|d| d.join("config.toml"))
}

pub fn history_path() -> Option<PathBuf> {
    rask_home().map(|d| d.join("history.jsonl"))
}

pub fn sessions_dir() -> Option<PathBuf> {
    rask_home().map(|d| d.join("sessions"))
}
