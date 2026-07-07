use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::NaiveDate;
use chrono::Datelike;

pub fn vault_path(vault_dir: &Path, db_key: &str, page_id: &str, date: Option<NaiveDate>) -> PathBuf {
    let db_dir = vault_dir.join(db_key);
    if let Some(date) = date {
        let year = format!("{:04}", date.year());
        let month = format!("{:02}", date.month());
        let filename = format!("{}.md", page_id);
        db_dir.join(year).join(month).join(filename)
    } else {
        db_dir.join(format!("{}.md", page_id))
    }
}

pub fn read_index(vault_dir: &Path) -> Result<HashMap<String, IndexEntry>, String> {
    let path = vault_dir.join(".vault.index.json");
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("Read index: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Parse index: {e}"))
}

pub fn write_index(vault_dir: &Path, index: &HashMap<String, IndexEntry>) -> Result<(), String> {
    let path = vault_dir.join(".vault.index.json");
    std::fs::create_dir_all(vault_dir).map_err(|e| format!("Create vault dir: {e}"))?;
    let raw = serde_json::to_string_pretty(index).map_err(|e| format!("Serialize index: {e}"))?;
    std::fs::write(&path, &raw).map_err(|e| format!("Write index: {e}"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub page_id: String,
    pub last_edited_time: String,
    pub file_path: String,
    pub db_key: String,
    pub title: String,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LastPullTimes {
    #[serde(default)]
    pub per_db: HashMap<String, String>,
    #[serde(default)]
    pub global: Option<String>,
}

pub fn read_last_pull_times(vault_dir: &Path) -> Result<LastPullTimes, String> {
    let path = vault_dir.join(".vault.last_pull");
    if !path.exists() {
        return Ok(LastPullTimes::default());
    }
    let raw = std::fs::read_to_string(&path).map_err(|e| format!("Read last_pull: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("Parse last_pull: {e}"))
}

pub fn write_last_pull_times(vault_dir: &Path, times: &LastPullTimes) -> Result<(), String> {
    let path = vault_dir.join(".vault.last_pull");
    let raw = serde_json::to_string_pretty(times).map_err(|e| format!("Serialize last_pull: {e}"))?;
    std::fs::write(&path, &raw).map_err(|e| format!("Write last_pull: {e}"))
}

pub fn utc_now_iso() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}