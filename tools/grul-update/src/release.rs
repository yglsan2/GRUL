//! Vérification de version GRUL — équivalent léger de do-release-upgrade.

use crate::config::read_release_version;
use std::fs;
use std::path::PathBuf;

const RELEASE_AVAILABLE_PATH: &str = "/var/lib/grul/release-available";

pub fn run_release_check() -> Result<(), String> {
    let current = read_release_version().unwrap_or_else(|| "inconnue".into());

    println!("GRUL — vérification de version");
    println!("==============================");
    println!("Version installée : {current}");

    if let Some(pending) = read_pending_release() {
        println!();
        println!("⚠ Nouvelle version GRUL disponible : {pending}");
        println!();
        println!("Pour mettre à niveau (comme Ubuntu do-release-upgrade) :");
        println!("  sudo grul-update release-upgrade");
        println!();
        println!("Cette opération mettra à jour les métapaquets grul-* et");
        println!("appliquera les migrations documentées pour votre canal.");
    } else {
        println!();
        println!("✓ Aucune nouvelle version GRUL signalée localement.");
        println!();
        println!("Vous êtes sur la dernière version connue de votre canal.");
        println!("Les mises à jour courantes passent par : sudo grul-update upgrade");
    }

    Ok(())
}

pub fn run_release_upgrade(dry_run: bool, yes: bool) -> Result<(), String> {
    let pending = read_pending_release().ok_or_else(|| {
        "Aucune version GRUL en attente. Lancez d'abord : grul-update release-check".into()
    })?;

    let current = read_release_version().unwrap_or_else(|| "?".into());
    println!("GRUL Release Upgrade");
    println!("====================");
    println!("{current}  →  {pending}");
    println!();

    if !yes && !dry_run {
        print!("Continuer la mise à niveau GRUL ? [O/n] ");
        use std::io::{self, Write};
        io::stdout().flush().map_err(|e| e.to_string())?;
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
        let answer = input.trim().to_lowercase();
        if !(answer.is_empty() || answer == "o" || answer == "oui" || answer == "y") {
            println!("Annulé.");
            return Ok(());
        }
    }

    if dry_run {
        println!("[dry-run] apt-get install -y grul-core grul-desktop");
        println!("[dry-run] migration scripts dans /usr/lib/grul/migrations/{pending}/");
        return Ok(());
    }

    crate::apt::require_root_for_apply(false)?;

    let out = std::process::Command::new("apt-get")
        .args(["install", "-y", "grul-core"])
        .env("DEBIAN_FRONTEND", "noninteractive")
        .output()
        .map_err(|e| e.to_string())?;

    if !out.status.success() {
        return Err(format!(
            "Échec apt-get install grul-core: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    // Mettre à jour le fichier release si le paquet ne l'a pas fait
    let release_path = crate::config::release_path();
    if release_path.parent().is_some() {
        let _ = fs::create_dir_all(release_path.parent().unwrap());
    }
    fs::write(&release_path, format!("{pending}\n")).map_err(|e| e.to_string())?;

    if PathBuf::from(RELEASE_AVAILABLE_PATH).exists() {
        let _ = fs::remove_file(RELEASE_AVAILABLE_PATH);
    }

    println!("✓ GRUL mis à niveau vers {pending}");
    println!("Relancez : sudo grul-update upgrade");
    Ok(())
}

fn read_pending_release() -> Option<String> {
    fs::read_to_string(RELEASE_AVAILABLE_PATH)
        .ok()
        .map(|s| s.lines().next().unwrap_or("").trim().to_string())
        .filter(|s| !s.is_empty())
}
