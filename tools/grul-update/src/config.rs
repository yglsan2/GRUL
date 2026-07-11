//! Configuration canaux GRUL — /etc/grul/channel.toml

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageChannel {
    Core,
    Current,
    Edge,
}

impl PackageChannel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Core => "Core (Debian Stable)",
            Self::Current => "GRUL Current",
            Self::Edge => "GRUL Edge",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSection {
    #[serde(default = "default_true")]
    pub core_enabled: bool,
    #[serde(default = "default_true")]
    pub current_enabled: bool,
    #[serde(default)]
    pub edge_enabled: bool,
}

fn default_true() -> bool {
    true
}

impl Default for ChannelSection {
    fn default() -> Self {
        Self {
            core_enabled: true,
            current_enabled: true,
            edge_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcesSection {
    #[serde(default = "default_repo_uri")]
    pub uri: String,
    #[serde(default = "default_suite_core")]
    pub core_suite: String,
    #[serde(default = "default_suite_current")]
    pub current_suite: String,
    #[serde(default = "default_suite_edge")]
    pub edge_suite: String,
    #[serde(default = "default_components")]
    pub components: String,
}

fn default_repo_uri() -> String {
    "https://repo.grul.org/debian".into()
}

fn default_suite_core() -> String {
    "grul-core".into()
}

fn default_suite_current() -> String {
    "grul-current".into()
}

fn default_suite_edge() -> String {
    "grul-edge".into()
}

fn default_components() -> String {
    "main".into()
}

impl Default for SourcesSection {
    fn default() -> Self {
        Self {
            uri: default_repo_uri(),
            core_suite: default_suite_core(),
            current_suite: default_suite_current(),
            edge_suite: default_suite_edge(),
            components: default_components(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSection {
    #[serde(default = "default_true")]
    pub snapshot_before_current: bool,
    #[serde(default = "default_true")]
    pub auto_security: bool,
    #[serde(default)]
    pub show_changelog: bool,
}

impl Default for UpdateSection {
    fn default() -> Self {
        Self {
            snapshot_before_current: true,
            auto_security: true,
            show_changelog: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrulChannelConfig {
    #[serde(default)]
    pub channel: ChannelSection,
    #[serde(default)]
    pub sources: SourcesSection,
    #[serde(default)]
    pub packages: HashMap<String, String>,
    #[serde(default)]
    pub update: UpdateSection,
}

impl Default for GrulChannelConfig {
    fn default() -> Self {
        Self {
            channel: ChannelSection::default(),
            sources: SourcesSection::default(),
            packages: HashMap::new(),
            update: UpdateSection::default(),
        }
    }
}

impl GrulChannelConfig {
    pub fn load() -> Result<Self, String> {
        let path = channel_config_path();
        if path.is_file() {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("lecture {path:?}: {e}"))?;
            toml::from_str(&content).map_err(|e| format!("TOML invalide ({path:?}): {e}"))
        } else {
            Ok(Self::default())
        }
    }

    pub fn package_channel(&self, name: &str) -> PackageChannel {
        if let Some(ch) = self.packages.get(name) {
            match ch.as_str() {
                "edge" => return PackageChannel::Edge,
                "current" => return PackageChannel::Current,
                "core" => return PackageChannel::Core,
                _ => {}
            }
        }
        PackageChannel::Core
    }

    pub fn active_channels_label(&self) -> String {
        let mut parts = Vec::new();
        if self.channel.core_enabled {
            parts.push("Core ✓");
        } else {
            parts.push("Core ✗");
        }
        if self.channel.current_enabled {
            parts.push("Current ✓");
        } else {
            parts.push("Current ✗");
        }
        if self.channel.edge_enabled {
            parts.push("Edge ✓");
        } else {
            parts.push("Edge ✗");
        }
        parts.join("  ")
    }
}

pub fn channel_config_path() -> PathBuf {
    PathBuf::from("/etc/grul/channel.toml")
}

pub fn release_path() -> PathBuf {
    PathBuf::from("/etc/grul/release")
}

pub fn apt_sources_dir() -> PathBuf {
    PathBuf::from("/etc/apt/sources.list.d")
}

pub fn source_list_path(suite: &str) -> PathBuf {
    apt_sources_dir().join(format!("{suite}.list"))
}

pub fn write_channel_sources(config: &GrulChannelConfig, dry_run: bool) -> Result<Vec<String>, String> {
    let mut actions = Vec::new();
    let keyring = "/usr/share/keyrings/grul-archive-keyring.gpg";

    let entries = [
        (
            &config.sources.core_suite,
            config.channel.core_enabled,
            "GRUL Core",
        ),
        (
            &config.sources.current_suite,
            config.channel.current_enabled,
            "GRUL Current",
        ),
        (
            &config.sources.edge_suite,
            config.channel.edge_enabled,
            "GRUL Edge",
        ),
    ];

    for (suite, enabled, label) in entries {
        let path = source_list_path(suite);
        let body = format!(
            "# {label} — géré par grul-update\n\
             deb [signed-by={keyring}] {uri} {suite} {components}\n",
            label = label,
            keyring = keyring,
            uri = config.sources.uri,
            suite = suite,
            components = config.sources.components,
        );

        if enabled {
            if dry_run {
                actions.push(format!("[dry-run] activerait {path:?}"));
            } else {
                ensure_apt_dir()?;
                fs::write(&path, &body).map_err(|e| format!("écriture {path:?}: {e}"))?;
                actions.push(format!("Activé {label} → {path:?}"));
            }
        } else if path.exists() {
            if dry_run {
                actions.push(format!("[dry-run] supprimerait {path:?}"));
            } else {
                fs::remove_file(&path).map_err(|e| format!("suppression {path:?}: {e}"))?;
                actions.push(format!("Désactivé {label} — {path:?} retiré"));
            }
        }
    }

    Ok(actions)
}

pub fn save_channel_config(config: &GrulChannelConfig, dry_run: bool) -> Result<(), String> {
    let path = channel_config_path();
    let body = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    if dry_run {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, body).map_err(|e| e.to_string())
}

fn ensure_apt_dir() -> Result<(), String> {
    fs::create_dir_all(apt_sources_dir()).map_err(|e| e.to_string())
}

pub fn read_release_version() -> Option<String> {
    fs::read_to_string(release_path())
        .ok()
        .map(|s| s.lines().next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn dev_channel_config_path(repo_root: &Path) -> PathBuf {
    repo_root.join("configs/grul-channel.toml")
}
