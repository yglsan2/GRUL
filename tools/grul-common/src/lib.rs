//! Types et chargement de profils partagés entre outils GRUL.

pub mod drivers;
pub mod vm;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileId {
    DesktopBalanced,
    DevPerformance,
    ServerMinimal,
    GamingLatency,
    VmMinimal,
}

impl ProfileId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DesktopBalanced => "desktop-balanced",
            Self::DevPerformance => "dev-performance",
            Self::ServerMinimal => "server-minimal",
            Self::GamingLatency => "gaming-latency",
            Self::VmMinimal => "vm-minimal",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "desktop-balanced" => Some(Self::DesktopBalanced),
            "dev-performance" => Some(Self::DevPerformance),
            "server-minimal" => Some(Self::ServerMinimal),
            "gaming-latency" => Some(Self::GamingLatency),
            "vm-minimal" => Some(Self::VmMinimal),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileMeta {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServicesSection {
    #[serde(default)]
    pub mask: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilesystemSection {
    #[serde(default)]
    pub ssd_noatime: bool,
    #[serde(default)]
    pub enable_fstrim: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageSuggest {
    #[serde(default)]
    pub suggest: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackagesSection {
    #[serde(default)]
    pub current: PackageSuggest,
    #[serde(default)]
    pub edge: PackageSuggest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrulProfile {
    pub profile: ProfileMeta,
    #[serde(default)]
    pub sysctl: HashMap<String, toml::Value>,
    #[serde(default)]
    pub services: ServicesSection,
    #[serde(default)]
    pub filesystem: FilesystemSection,
    #[serde(default)]
    pub packages: PackagesSection,
    #[serde(default)]
    pub grul: HashMap<String, toml::Value>,
}

impl GrulProfile {
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("lecture {path:?}: {e}"))?;
        toml::from_str(&content).map_err(|e| format!("TOML invalide: {e}"))
    }

    pub fn sysctl_lines(&self) -> Vec<(String, String)> {
        self.sysctl
            .iter()
            .map(|(k, v)| (k.clone(), format_toml_value(v)))
            .collect()
    }
}

fn format_toml_value(v: &toml::Value) -> String {
    match v {
        toml::Value::String(s) => s.clone(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        other => other.to_string(),
    }
}

/// Répertoire des profils : /etc/grul/profiles ou GRUL_PROFILES_DIR ou ./configs/profiles (dev).
pub fn profiles_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("GRUL_PROFILES_DIR") {
        return PathBuf::from(dir);
    }
    let etc = PathBuf::from("/etc/grul/profiles");
    if etc.is_dir() {
        return etc;
    }
    // Dev : depuis la racine du repo
    PathBuf::from("configs/profiles")
}

pub fn load_profile(id: &str) -> Result<GrulProfile, String> {
    let path = profiles_dir().join(format!("{id}.toml"));
    if !path.is_file() {
        return Err(format!("profil introuvable: {path:?}"));
    }
    GrulProfile::load_from_file(&path)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedState {
    pub profile_id: String,
    pub sysctl_file: PathBuf,
    #[serde(default)]
    pub masked_services: Vec<String>,
    pub fstrim_enabled: bool,
    pub applied_at: String,
}

pub fn state_path() -> PathBuf {
    PathBuf::from("/var/lib/grul/applied.json")
}

pub fn load_applied_state() -> Option<AppliedState> {
    let path = state_path();
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_applied_state(state: &AppliedState) -> Result<(), String> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn remove_applied_state() -> Result<(), String> {
    let path = state_path();
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
