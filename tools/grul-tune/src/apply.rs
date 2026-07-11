//! Application et annulation des profils GRUL sur Debian.

use grul_common::{load_applied_state, load_profile, remove_applied_state, save_applied_state, AppliedState, GrulProfile};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SYSCTL_PREFIX: &str = "99-grul-";

pub struct ApplyOptions {
    pub dry_run: bool,
    pub yes: bool,
}

pub struct ApplyResult {
    pub profile_id: String,
    pub sysctl_file: PathBuf,
    pub masked_services: Vec<String>,
    pub fstrim_enabled: bool,
    pub actions: Vec<String>,
}

pub fn apply_profile(profile: &GrulProfile, opts: &ApplyOptions) -> Result<ApplyResult, String> {
    let profile_id = profile.profile.id.clone();
    let sysctl_file = PathBuf::from(format!("/etc/sysctl.d/{SYSCTL_PREFIX}{profile_id}.conf"));

    let mut actions = Vec::new();
    let mut masked = Vec::new();

    // --- sysctl ---
    let sysctl_body = build_sysctl_file(profile);
    if opts.dry_run {
        actions.push(format!("[dry-run] écrirait {sysctl_file:?}"));
        actions.push(format!("[dry-run] contenu sysctl:\n{sysctl_body}"));
    } else {
        require_root()?;
        fs::write(&sysctl_file, &sysctl_body)
            .map_err(|e| format!("écriture {sysctl_file:?}: {e}"))?;
        actions.push(format!("Écrit {sysctl_file:?}"));
        run_and_log("sysctl", &["--system"], &mut actions)?;
    }

    // --- services mask ---
    for unit in &profile.services.mask {
        if opts.dry_run {
            actions.push(format!("[dry-run] systemctl mask {unit}"));
        } else {
            run_and_log("systemctl", &["mask", unit], &mut actions)?;
        }
        masked.push(unit.clone());
    }

    // --- fstrim ---
    let fstrim = profile.filesystem.enable_fstrim;
    if fstrim {
        if opts.dry_run {
            actions.push("[dry-run] systemctl enable --now fstrim.timer".into());
        } else {
            run_and_log(
                "systemctl",
                &["enable", "--now", "fstrim.timer"],
                &mut actions,
            )?;
        }
    }

    // --- fstab noatime (recommandation seulement — modification fstab trop risquée en auto) ---
    if profile.filesystem.ssd_noatime {
        actions.push(
            "Note: noatime sur / — ajouter manuellement dans /etc/fstab ou via grul-config (Phase 1)"
                .into(),
        );
    }

    if !opts.dry_run {
        let state = AppliedState {
            profile_id: profile_id.clone(),
            sysctl_file: sysctl_file.clone(),
            masked_services: masked.clone(),
            fstrim_enabled: fstrim,
            applied_at: chrono::Utc::now().to_rfc3339(),
        };
        save_applied_state(&state)?;
        actions.push(format!("État enregistré dans {:?}", grul_common::state_path()));
    }

    Ok(ApplyResult {
        profile_id,
        sysctl_file,
        masked_services: masked,
        fstrim_enabled: fstrim,
        actions,
    })
}

pub fn reset_profile(opts: &ApplyOptions) -> Result<Vec<String>, String> {
    let mut actions = Vec::new();
    let state = load_applied_state();

    if let Some(ref st) = state {
        if opts.dry_run {
            actions.push(format!("[dry-run] supprimer {:?}", st.sysctl_file));
        } else {
            require_root()?;
            if st.sysctl_file.exists() {
                fs::remove_file(&st.sysctl_file).map_err(|e| e.to_string())?;
                actions.push(format!("Supprimé {:?}", st.sysctl_file));
            }
            run_and_log("sysctl", &["--system"], &mut actions)?;
        }

        for unit in &st.masked_services {
            if opts.dry_run {
                actions.push(format!("[dry-run] systemctl unmask {unit}"));
            } else {
                let _ = run_and_log("systemctl", &["unmask", unit], &mut actions);
            }
        }

        if st.fstrim_enabled && !opts.dry_run {
            let _ = run_and_log("systemctl", &["disable", "--now", "fstrim.timer"], &mut actions);
        }

        if !opts.dry_run {
            remove_applied_state()?;
            actions.push("État GRUL supprimé".into());
        }
    } else {
        // Fallback : nettoyer les fichiers sysctl grul connus
        let pattern = format!("/etc/sysctl.d/{SYSCTL_PREFIX}");
        if opts.dry_run {
            actions.push(format!("[dry-run] supprimer {pattern}*.conf"));
        } else if Path::new("/etc/sysctl.d").is_dir() {
            require_root()?;
            for entry in fs::read_dir("/etc/sysctl.d").map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with(SYSCTL_PREFIX) {
                    fs::remove_file(entry.path()).map_err(|e| e.to_string())?;
                    actions.push(format!("Supprimé /etc/sysctl.d/{name}"));
                }
            }
            run_and_log("sysctl", &["--system"], &mut actions)?;
        } else {
            actions.push("Aucun profil GRUL appliqué trouvé".into());
        }
    }

    Ok(actions)
}

pub fn show_status() -> Result<String, String> {
    let mut out = String::from("GRUL Tune — état\n================\n");

    if let Some(state) = load_applied_state() {
        out.push_str(&format!("Profil actif : {}\n", state.profile_id));
        out.push_str(&format!("Appliqué le   : {}\n", state.applied_at));
        out.push_str(&format!("Sysctl        : {:?}\n", state.sysctl_file));
        if !state.masked_services.is_empty() {
            out.push_str(&format!("Services masqués : {:?}\n", state.masked_services));
        }
        out.push_str(&format!("fstrim.timer  : {}\n", state.fstrim_enabled));
    } else {
        out.push_str("Aucun profil GRUL enregistré.\n");
    }

    Ok(out)
}

fn build_sysctl_file(profile: &GrulProfile) -> String {
    let mut lines = vec![
        format!("# GRUL profile: {}", profile.profile.id),
        format!("# {}", profile.profile.description),
        String::new(),
    ];
    for (key, val) in profile.sysctl_lines() {
        lines.push(format!("{key} = {val}"));
    }
    lines.join("\n") + "\n"
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
        return Err("grul-tune apply/reset nécessite Linux avec droits root".into());
    }
    Ok(())
}

fn run_and_log(cmd: &str, args: &[&str], actions: &mut Vec<String>) -> Result<(), String> {
    let output = Command::new(cmd).args(args).output();
    match output {
        Ok(out) => log_output(cmd, args, &out, actions),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            actions.push(format!("Avertissement: {cmd} introuvable — étape ignorée"));
            Ok(())
        }
        Err(e) => Err(format!("exécution {cmd}: {e}")),
    }
}

fn log_output(cmd: &str, args: &[&str], out: &Output, actions: &mut Vec<String>) -> Result<(), String> {
    let status = out.status;
    let stderr = String::from_utf8_lossy(&out.stderr);
    actions.push(format!("Exécuté: {cmd} {}", args.join(" ")));
    if !status.success() && !stderr.is_empty() {
        actions.push(format!("  stderr: {}", stderr.trim()));
    }
    if !status.success() {
        return Err(format!("{cmd} a échoué (code {:?})", status.code()));
    }
    Ok(())
}
