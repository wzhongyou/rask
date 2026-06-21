use serde::{Deserialize, Serialize};
use crate::{Message, error::Result, paths::history_path};

#[derive(Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub model: String,
    pub messages: Vec<Message>,
    pub timestamp: u64,
    pub date: String,
}

impl HistoryEntry {
    pub fn new(model: String, messages: Vec<Message>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let ts = now.as_secs();
        let id = format!("{:x}", ts ^ (now.subsec_nanos() as u64));
        let date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self { id, model, messages, timestamp: ts, date }
    }
}

pub fn save(entry: &HistoryEntry) -> Result<()> {
    let path = history_path().ok_or_else(|| crate::error::RaskError::Config("cannot find home dir".into()))?;
    std::fs::create_dir_all(path.parent().unwrap())?;
    let line = serde_json::to_string(entry).unwrap();
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(&path)?;
    use std::io::Write;
    writeln!(file, "{line}")?;
    Ok(())
}

pub fn load_raw() -> Result<Vec<String>> {
    let path = history_path().ok_or_else(|| crate::error::RaskError::Config("cannot find home dir".into()))?;
    if !path.exists() {
        return Ok(vec![]);
    }
    Ok(std::fs::read_to_string(&path)?.lines().map(String::from).collect())
}
