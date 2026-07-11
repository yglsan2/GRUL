//! Logique de création, liste, rollback et purge des snapshots.

use crate::btrfs;
use crate::config::{GrulSnapConfig, SnapMode};
use crate::state::{SnapshotRecord, SnapshotState};
use chrono::Utc;
use std::path::{Path, PathBuf};

pub struct CreateOptions {
    pub reason: String,
    pub dry_run: bool,
    pub force: bool,
}

pub struct CreateResult {
    pub id: String,
    pub path: PathBuf,
    pub message: String,
}

pub fn print_status(config: &GrulSnapConfig) -> Result<(), String> {
    let btrfs_ok = btrfs::detect_btrfs_root().is_some();
    let fs = btrfs::root_fs_type().unwrap_or_else(|| "inconnu".into());

    println!("GRUL Snap — état");
    println!("================");
    println!("Mode        : {}", config.mode().label());
    println!("Backend     : {}", config.snap.backend);
    println!("FS racine   : {fs}");
    println!(
        "Btrfs       : {}",
        if btrfs_ok { "disponible ✓" } else { "non disponible ✗" }
    );
    println!("Répertoire  : {}", config.snap.snapshot_dir);
    println!("Max snaps   : {}", config.snap.max_snapshots);
    println!(
        "fail_open   : {} (n'bloque pas grul-update si échec)",
        config.snap.fail_open
    );
    println!();

    let state = SnapshotState::load();
    if state.snapshots.is_empty() {
        println!("Aucun snapshot enregistré.");
    } else {
        println!("{} snapshot(s) enregistré(s) :", state.snapshots.len());
        for s in &state.snapshots {
            println!(
                "  • {} — {} ({})",
                s.id, s.created_at, s.reason
            );
        }
    }

    println!();
    if config.mode() == SnapMode::Disabled {
        println!("Snapshots désactivés. Activer : sudo grul-snap enable");
    } else if !btrfs_ok && config.mode() == SnapMode::Enabled {
        println!("⚠ Mode activé mais Btrfs absent — les snapshots échoueront.");
        println!("  Installez sur Btrfs ou passez en mode auto/disabled.");
    } else if btrfs_ok && config.mode() == SnapMode::Auto {
        println!("Mode auto : snapshots pris avant upgrades Current (si grul-snap installé).");
    }

    Ok(())
}

pub fn create_snapshot(config: &GrulSnapConfig, opts: &CreateOptions) -> Result<CreateResult, String> {
    let btrfs_root = btrfs::detect_btrfs_root();

    if !opts.force && !config.should_attempt(btrfs_root.is_some()) {
        return Err(format!(
            "snapshots en mode « {} » — rien à faire (utilisez --force pour forcer)",
            config.mode().label()
        ));
    }

    let Some(root) = btrfs_root else {
        return Err("Btrfs requis pour les snapshots — partition / non Btrfs".into());
    };

    if config.snap.backend != "btrfs" {
        return Err(format!(
            "backend « {} » non implémenté (Phase 3 : ZFS)",
            config.snap.backend
        ));
    }

    let id = format!(
        "grul-{}",
        Utc::now().format("%Y%m%d-%H%M%S")
    );
    let dest = PathBuf::from(&config.snap.snapshot_dir).join(&id);

    if opts.dry_run {
        return Ok(CreateResult {
            id,
            path: dest,
            message: format!(
                "[dry-run] btrfs subvolume snapshot -r / {dest:?}"
            ),
        });
    }

    require_root()?;
    btrfs::create_readonly_snapshot(&root.mount_point, &dest)?;

    let subvol_id = btrfs::subvolume_id(&dest).ok();
    let record = SnapshotRecord {
        id: id.clone(),
        path: dest.clone(),
        created_at: Utc::now().to_rfc3339(),
        reason: opts.reason.clone(),
        subvolume_id: subvol_id,
    };

    let mut state = SnapshotState::load();
    state.push(record);
    let removed = trim_old_snapshots(config, &mut state, false)?;
    state.save()?;
    for msg in removed {
        eprintln!("  {msg}");
    }

    Ok(CreateResult {
        id,
        path: dest,
        message: format!("Snapshot créé : {dest:?}"),
    })
}

pub fn list_snapshots() -> Result<(), String> {
    let state = SnapshotState::load();
    if state.snapshots.is_empty() {
        println!("Aucun snapshot GRUL.");
        return Ok(());
    }

    println!("Snapshots GRUL :\n");
    for (i, s) in state.snapshots.iter().enumerate() {
        let id_hint = s
            .subvolume_id
            .map(|n| format!(" (subvol #{n})"))
            .unwrap_or_default();
        println!(
            "  [{i}] {} — {}{}",
            s.id, s.created_at, id_hint
        );
        println!("      raison : {}", s.reason);
        println!("      chemin : {:?}", s.path);
    }

    Ok(())
}

pub fn rollback(snapshot_id: Option<String>, dry_run: bool) -> Result<(), String> {
    let state = SnapshotState::load();
    let record = if let Some(id) = snapshot_id {
        state
            .snapshots
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| format!("snapshot introuvable: {id}"))?
            .clone()
    } else {
        state
            .last()
            .ok_or("aucun snapshot disponible")?
            .clone()
    };

    if !record.path.exists() {
        return Err(format!("chemin snapshot absent: {:?}", record.path));
    }

    let subvol_id = record
        .subvolume_id
        .or_else(|| btrfs::subvolume_id(&record.path).ok())
        .ok_or("ID subvolume introuvable")?;

    if dry_run {
        println!(
            "[dry-run] btrfs subvolume set-default {subvol_id} /"
        );
        println!("Puis redémarrage requis.");
        return Ok(());
    }

    require_root()?;
    btrfs::set_default_subvolume(subvol_id, Path::new("/"))?;

    println!("✓ Subvolume par défaut → {} (#{subvol_id})", record.id);
    println!();
    println!("⚠ Redémarrage requis pour revenir à cet état :");
    println!("  sudo reboot");
    Ok(())
}

pub fn prune(config: &GrulSnapConfig, dry_run: bool) -> Result<Vec<String>, String> {
    let mut state = SnapshotState::load();
    if !dry_run {
        require_root()?;
    }
    let actions = trim_old_snapshots(config, &mut state, dry_run)?;
    if !dry_run && !actions.is_empty() {
        state.save()?;
    }
    Ok(actions)
}

fn trim_old_snapshots(
    config: &GrulSnapConfig,
    state: &mut SnapshotState,
    dry_run: bool,
) -> Result<Vec<String>, String> {
    let mut actions = Vec::new();
    while state.snapshots.len() > config.snap.max_snapshots {
        let oldest = state.snapshots.remove(0);
        if dry_run {
            actions.push(format!("supprimerait : {}", oldest.id));
        } else if oldest.path.exists() {
            btrfs::delete_snapshot(&oldest.path)?;
            actions.push(format!("Supprimé : {}", oldest.id));
        } else {
            actions.push(format!("Retiré (absent) : {}", oldest.id));
        }
    }
    Ok(actions)
}

pub fn set_mode(config: &mut GrulSnapConfig, mode: SnapMode) -> Result<(), String> {
    config.snap.mode = match mode {
        SnapMode::Disabled => "disabled",
        SnapMode::Auto => "auto",
        SnapMode::Enabled => "enabled",
    }
    .into();
    config.save()
}

pub fn is_available() -> bool {
    btrfs::detect_btrfs_root().is_some()
}

pub fn should_snapshot_for_update(config: &GrulSnapConfig, has_current_updates: bool) -> bool {
    if config.mode() == SnapMode::Disabled {
        return false;
    }
    match config.snap.trigger.as_str() {
        "any-upgrade" => config.should_attempt(is_available()),
        _ => has_current_updates && config.should_attempt(is_available()),
    }
}

fn require_root() -> Result<(), String> {
    #[cfg(unix)]
    {
        let uid = unsafe { libc::geteuid() };
        if uid != 0 {
            return Err("root requis — relancez avec sudo".into());
        }
    }
    #[cfg(not(unix))]
    {
        return Err("grul-snap nécessite Linux".into());
    }
    Ok(())
}
