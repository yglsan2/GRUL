//! Configuration automatique de base pour machines virtuelles.

use crate::report::is_root;
use grul_common::vm::{detect_vm, VirtKind};
use std::process::Command;

pub struct VmSetupOptions {
    pub dry_run: bool,
    pub yes: bool,
}

pub fn run_vm_setup(opts: &VmSetupOptions) -> Result<Vec<String>, String> {
    let vm = detect_vm();
    if !vm.is_virtual {
        return Err("vm-setup est réservé aux machines virtuelles — utilisez grul-tune apply --auto".into());
    }

    if !opts.dry_run && !is_root() {
        return Err("root requis — sudo grul-doctor vm-setup".into());
    }

    let mut actions = Vec::new();

    actions.push(format!(
        "VM détectée : {} ({})",
        vm.kind.label(),
        vm.hypervisor
    ));

        if opts.dry_run {
            actions.push("[dry-run] appliquer grul-channel-vm.toml → /etc/grul/channel.toml".into());
        } else if let Some(src) = find_channel_vm_template() {
            std::fs::copy(&src, "/etc/grul/channel.toml")
                .map_err(|e| format!("copie canal VM: {e}"))?;
            actions.push(format!("Canal VM appliqué depuis {src:?}"));
        } else {
            actions.push("Canal VM : Core + sécurité auto (template introuvable, canal actuel conservé)".into());
        }

    // Profil vm-minimal
    if opts.dry_run {
        actions.push("[dry-run] grul-tune apply --profile vm-minimal".into());
    } else {
        run_cmd("grul-tune", &["apply", "--profile", "vm-minimal", "--yes"], &mut actions)?;
    }

    // Guest agents / pilotes hyperviseur
    if opts.dry_run {
        actions.push("[dry-run] grul-doctor drivers install".into());
    } else {
        let driver_lines = crate::drivers::run_install(&crate::drivers::DriverInstallOptions {
            dry_run: false,
        })?;
        actions.extend(driver_lines);
    }

    // cloud-init usually preinstalled on cloud images
    if !vm.cloud_init && matches!(vm.kind, VirtKind::Amazon | VirtKind::Google | VirtKind::Microsoft) {
        if opts.dry_run {
            actions.push("[dry-run] apt-get install -y cloud-init".into());
        } else {
            let _ = run_cmd("apt-get", &["install", "-y", "cloud-init"], &mut actions);
        }
    }

    // Timer sécurité
    if opts.dry_run {
        actions.push("[dry-run] systemctl enable --now grul-update-security.timer".into());
    } else {
        let _ = run_cmd(
            "systemctl",
            &["enable", "--now", "grul-update-security.timer"],
            &mut actions,
        );
        actions.push("Timer sécurité GRUL activé".into());
    }

    // Refresh + status
    if !opts.dry_run {
        let _ = run_cmd("grul-update", &["refresh"], &mut actions);
    } else {
        actions.push("[dry-run] grul-update refresh".into());
    }

    actions.push("VM prête — grul-doctor pour vérifier".into());
    Ok(actions)
}

fn run_cmd(cmd: &str, args: &[&str], actions: &mut Vec<String>) -> Result<(), String> {
    let output = Command::new(cmd)
        .args(args)
        .env("DEBIAN_FRONTEND", "noninteractive")
        .output()
        .map_err(|e| format!("{cmd}: {e}"))?;

    actions.push(format!("Exécuté: {cmd} {}", args.join(" ")));
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{cmd} a échoué: {}", stderr.trim()));
    }
    Ok(())
}

fn find_channel_vm_template() -> Option<std::path::PathBuf> {
    const CANDIDATES: &[&str] = &[
        "/usr/share/grul/grul-channel-vm.toml",
        "configs/grul-channel-vm.toml",
    ];
    for c in CANDIDATES {
        let p = std::path::Path::new(c);
        if p.is_file() {
            return Some(p.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            let p = ancestor.join("configs/grul-channel-vm.toml");
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}
