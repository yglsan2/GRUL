//! État des snapshots GRUL — /var/lib/grul/snapshots.json

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::config::state_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotRecord {
    pub id: String,
    pub path: PathBuf,
    pub created_at: String,
    pub reason: String,
    pub subvolume_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SnapshotState {
    #[serde(default)]
    pub snapshots: Vec<SnapshotRecord>,
}

impl SnapshotState {
    pub fn load() -> Self {
        let path = state_path();
        if !path.is_file() {
            return Self::default();
        }
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = state_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }

    pub fn push(&mut self, record: SnapshotRecord) {
        self.snapshots.push(record);
    }

    pub fn last(&self) -> Option<&SnapshotRecord> {
        self.snapshots.last()
    }

    pub fn remove_by_id(&mut self, id: &str) -> Option<SnapshotRecord> {
        if let Some(pos) = self.snapshots.iter().position(|s| s.id == id) {
            Some(self.snapshots.remove(pos))
        } else {
            None
        }
    }
}
