//! Backend Btrfs pour grul-snap.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct BtrfsRoot {
    pub mount_point: PathBuf,
    pub subvolume_path: PathBuf,
}

pub fn root_fs_type() -> Option<String> {
    let output = Command::new("findmnt")
        .args(["-no", "FSTYPE", "/"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let fs = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if fs.is_empty() {
        None
    } else {
        Some(fs)
    }
}

pub fn detect_btrfs_root() -> Option<BtrfsRoot> {
    if root_fs_type().as_deref() != Some("btrfs") {
        return None;
    }

    let output = Command::new("btrfs")
        .args(["subvolume", "show", "/"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let subvolume_path = text
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("Subvolume:")
                .map(|v| PathBuf::from(v.trim()))
        })
        .unwrap_or_else(|| PathBuf::from("/"));

    Some(BtrfsRoot {
        mount_point: PathBuf::from("/"),
        subvolume_path,
    })
}

pub fn ensure_directory(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(path).map_err(|e| format!("création {path:?}: {e}"))
}

pub fn create_readonly_snapshot(source: &Path, dest: &Path) -> Result<(), String> {
    if dest.exists() {
        return Err(format!("le snapshot existe déjà: {dest:?}"));
    }

    if let Some(parent) = dest.parent() {
        ensure_directory(parent)?;
    }

    let output = Command::new("btrfs")
        .args([
            "subvolume",
            "snapshot",
            "-r",
            source.to_str().ok_or("chemin source invalide")?,
            dest.to_str().ok_or("chemin dest invalide")?,
        ])
        .output()
        .map_err(|e| format!("btrfs snapshot: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "btrfs snapshot a échoué: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(())
}

pub fn subvolume_id(path: &Path) -> Result<u64, String> {
    let output = Command::new("btrfs")
        .args(["subvolume", "show", path.to_str().ok_or("chemin invalide")?])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!(
            "btrfs subvolume show: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if let Some(id) = line.trim().strip_prefix("Subvolume ID:") {
            return id
                .trim()
                .parse::<u64>()
                .map_err(|e| format!("ID invalide: {e}"));
        }
    }

    Err("Subvolume ID introuvable".into())
}

pub fn set_default_subvolume(id: u64, mount: &Path) -> Result<(), String> {
    let output = Command::new("btrfs")
        .args([
            "subvolume",
            "set-default",
            &id.to_string(),
            mount.to_str().ok_or("mount invalide")?,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!(
            "btrfs subvolume set-default: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(())
}

pub fn delete_snapshot(path: &Path) -> Result<(), String> {
    let output = Command::new("btrfs")
        .args([
            "subvolume",
            "delete",
            path.to_str().ok_or("chemin invalide")?,
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!(
            "btrfs subvolume delete: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    Ok(())
}

pub fn list_subvolumes_in(dir: &Path) -> Result<Vec<PathBuf>, String> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut snaps = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            snaps.push(path);
        }
    }
    snaps.sort();
    Ok(snaps)
}
