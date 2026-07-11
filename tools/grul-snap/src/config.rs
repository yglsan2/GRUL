//! Configuration grul-snap — /etc/grul/snap.toml

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapMode {
    Disabled,
    Auto,
    Enabled,
}

impl SnapMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "enabled" => Self::Enabled,
            "disabled" => Self::Disabled,
            _ => Self::Auto,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Disabled => "désactivé",
            Self::Auto => "auto (Btrfs détecté)",
            Self::Enabled => "activé",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapSection {
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default = "default_backend")]
    pub backend: String,
    #[serde(default = "default_max")]
    pub max_snapshots: usize,
    #[serde(default = "default_true")]
    pub fail_open: bool,
    #[serde(default = "default_trigger")]
    pub trigger: String,
    #[serde(default = "default_snapshot_dir")]
    pub snapshot_dir: String,
}

fn default_mode() -> String {
    "auto".into()
}

fn default_backend() -> String {
    "btrfs".into()
}

fn default_max() -> usize {
    5
}

fn default_true() -> bool {
    true
}

fn default_trigger() -> String {
    "current-only".into()
}

fn default_snapshot_dir() -> String {
    "/.snapshots/grul".into()
}

impl Default for SnapSection {
    fn default() -> Self {
        Self {
            mode: default_mode(),
            backend: default_backend(),
            max_snapshots: default_max(),
            fail_open: default_true(),
            trigger: default_trigger(),
            snapshot_dir: default_snapshot_dir(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GrulSnapConfig {
    #[serde(default)]
    pub snap: SnapSection,
}

impl GrulSnapConfig {
    pub fn load() -> Self {
        let path = snap_config_path();
        if path.is_file() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = toml::from_str(&content) {
                    return cfg;
                }
            }
        }

        if let Ok(cwd) = std::env::current_dir() {
            for ancestor in cwd.ancestors() {
                let dev = ancestor.join("configs/grul-snap.toml");
                if dev.is_file() {
                    if let Ok(content) = fs::read_to_string(&dev) {
                        if let Ok(cfg) = toml::from_str(&content) {
                            return cfg;
                        }
                    }
                }
            }
        }

        Self::default()
    }

    pub fn mode(&self) -> SnapMode {
        SnapMode::from_str(&self.snap.mode)
    }

    pub fn save(&self) -> Result<(), String> {
        let path = snap_config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let body = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(path, body).map_err(|e| e.to_string())
    }

    pub fn should_attempt(&self, btrfs_available: bool) -> bool {
        match self.mode() {
            SnapMode::Disabled => false,
            SnapMode::Enabled => true,
            SnapMode::Auto => btrfs_available,
        }
    }
}

pub fn snap_config_path() -> PathBuf {
    if let Ok(p) = std::env::var("GRUL_SNAP_CONFIG") {
        return PathBuf::from(p);
    }
    PathBuf::from("/etc/grul/snap.toml")
}

pub fn state_path() -> PathBuf {
    PathBuf::from("/var/lib/grul/snapshots.json")
}
