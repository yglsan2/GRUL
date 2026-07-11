//! Rapport de santé GRUL.

use grul_common::vm::detect_vm;
use grul_common::{load_applied_state, load_profile};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Ok,
    Warn,
    Fail,
    Info,
}

pub struct Check {
    pub status: CheckStatus,
    pub label: String,
    pub detail: String,
}

pub struct DoctorReport {
    pub checks: Vec<Check>,
    pub score_ok: usize,
    pub score_total: usize,
}

pub fn run_full_report() -> DoctorReport {
    let mut checks = Vec::new();

    checks.push(check_grul_tools());
    checks.push(check_vm_context());
    checks.push(check_profile());
    checks.push(check_disk_space());
    checks.push(check_pending_updates());
    checks.push(check_security_timer());
    checks.push(check_guest_agent());

    let score_ok = checks
        .iter()
        .filter(|c| matches!(c.status, CheckStatus::Ok | CheckStatus::Info))
        .count();
    let score_total = checks.len();

    DoctorReport {
        checks,
        score_ok,
        score_total,
    }
}

pub fn run_quick_report() -> DoctorReport {
    let mut checks = vec![
        check_grul_tools(),
        check_vm_context(),
        check_profile(),
        check_pending_updates(),
    ];
    let score_ok = checks
        .iter()
        .filter(|c| matches!(c.status, CheckStatus::Ok | CheckStatus::Info))
        .count();
    let score_total = checks.len();
    DoctorReport {
        checks,
        score_ok,
        score_total,
    }
}

pub fn print_report(report: &DoctorReport) {
    let score = compute_score(report);
    println!("GRUL Doctor");
    println!("===========");
    println!("Score : {score} / 100\n");

    for c in &report.checks {
        let icon = match c.status {
            CheckStatus::Ok => "✓",
            CheckStatus::Warn => "⚠",
            CheckStatus::Fail => "✗",
            CheckStatus::Info => "ℹ",
        };
        println!("{icon} {} — {}", c.label, c.detail);
    }

    let recs = recommendations(report);
    if !recs.is_empty() {
        println!("\nOptimisations recommandées :");
        for (i, r) in recs.iter().take(3).enumerate() {
            println!("  {}. {r}", i + 1);
        }
    }
}

fn compute_score(report: &DoctorReport) -> u32 {
    let mut score = 100u32;
    for c in &report.checks {
        match c.status {
            CheckStatus::Fail => score = score.saturating_sub(15),
            CheckStatus::Warn => score = score.saturating_sub(8),
            _ => {}
        }
    }
    score.max(0)
}

fn recommendations(report: &DoctorReport) -> Vec<String> {
    let mut recs = Vec::new();
    for c in &report.checks {
        match c.label.as_str() {
            "Profil GRUL" if c.status == CheckStatus::Warn => {
                recs.push("sudo grul optimize — appliquer le profil adapté".into());
            }
            "Mises à jour" if c.status == CheckStatus::Warn => {
                recs.push("sudo grul update -y — installer les mises à jour".into());
            }
            "Guest agent" if c.status == CheckStatus::Warn => {
                recs.push("sudo grul vm optimize — guest agent + tuning VM".into());
            }
            "Espace disque /" if matches!(c.status, CheckStatus::Warn | CheckStatus::Fail) => {
                recs.push("sudo grul clean — libérer de l'espace (apt autoremove + clean)".into());
            }
            _ => {}
        }
    }
    if recs.is_empty() {
        recs.push("Système en bon état — grul status pour le suivi".into());
    }
    recs
}

fn check_grul_tools() -> Check {
    let tools = ["grul-detect", "grul-tune", "grul-update"];
    let missing: Vec<_> = tools
        .iter()
        .filter(|t| !command_exists(t))
        .map(|s| *s)
        .collect();

    if missing.is_empty() {
        Check {
            status: CheckStatus::Ok,
            label: "Outils GRUL".into(),
            detail: "grul-detect, grul-tune, grul-update installés".into(),
        }
    } else {
        Check {
            status: CheckStatus::Fail,
            label: "Outils GRUL".into(),
            detail: format!("manquants : {}", missing.join(", ")),
        }
    }
}

fn check_vm_context() -> Check {
    let vm = detect_vm();
    if vm.is_virtual {
        Check {
            status: CheckStatus::Info,
            label: "Machine virtuelle".into(),
            detail: format!("{} — {}", vm.kind.label(), vm.hypervisor),
        }
    } else {
        Check {
            status: CheckStatus::Info,
            label: "Machine virtuelle".into(),
            detail: "bare metal".into(),
        }
    }
}

fn check_profile() -> Check {
    match load_applied_state() {
        Some(state) => {
            let desc = load_profile(&state.profile_id)
                .map(|p| p.profile.description)
                .unwrap_or_else(|_| state.profile_id.clone());
            Check {
                status: CheckStatus::Ok,
                label: "Profil GRUL".into(),
                detail: format!("{} — {}", state.profile_id, desc),
            }
        }
        None => Check {
            status: CheckStatus::Warn,
            label: "Profil GRUL".into(),
            detail: "aucun profil appliqué — lancez : sudo grul-tune apply --auto".into(),
        },
    }
}

fn check_disk_space() -> Check {
    let output = Command::new("df")
        .args(["-h", "/"])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let line = String::from_utf8_lossy(&o.stdout)
                .lines()
                .nth(1)
                .unwrap_or("")
                .to_string();
            let pct = line.split_whitespace().nth(4).unwrap_or("?").to_string();
            let used_pct: u32 = pct.trim_end_matches('%').parse().unwrap_or(0);
            let status = if used_pct >= 90 {
                CheckStatus::Fail
            } else if used_pct >= 80 {
                CheckStatus::Warn
            } else {
                CheckStatus::Ok
            };
            Check {
                status,
                label: "Espace disque /".into(),
                detail: line,
            }
        }
        _ => Check {
            status: CheckStatus::Warn,
            label: "Espace disque /".into(),
            detail: "impossible de lire df".into(),
        },
    }
}

fn check_pending_updates() -> Check {
    if !command_exists("apt") {
        return Check {
            status: CheckStatus::Info,
            label: "Mises à jour".into(),
            detail: "apt non disponible".into(),
        };
    }

    let output = Command::new("apt")
        .args(["list", "--upgradable"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let count = String::from_utf8_lossy(&o.stdout)
                .lines()
                .skip(1)
                .filter(|l| !l.is_empty() && !l.starts_with("Listing"))
                .count();
            if count == 0 {
                Check {
                    status: CheckStatus::Ok,
                    label: "Mises à jour".into(),
                    detail: "système à jour".into(),
                }
            } else {
                Check {
                    status: CheckStatus::Warn,
                    label: "Mises à jour".into(),
                    detail: format!("{count} paquet(s) — sudo grul-update upgrade"),
                }
            }
        }
        _ => Check {
            status: CheckStatus::Info,
            label: "Mises à jour".into(),
            detail: "exécutez grul-update status".into(),
        },
    }
}

fn check_security_timer() -> Check {
    if !command_exists("systemctl") {
        return Check {
            status: CheckStatus::Info,
            label: "Sécurité auto".into(),
            detail: "systemd absent".into(),
        };
    }
    let enabled = Command::new("systemctl")
        .args(["is-enabled", "grul-update-security.timer"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if enabled {
        Check {
            status: CheckStatus::Ok,
            label: "Sécurité auto".into(),
            detail: "grul-update-security.timer actif".into(),
        }
    } else {
        Check {
            status: CheckStatus::Info,
            label: "Sécurité auto".into(),
            detail: "optionnel — systemctl enable --now grul-update-security.timer".into(),
        }
    }
}

fn check_guest_agent() -> Check {
    let vm = detect_vm();
    if !vm.is_virtual {
        return Check {
            status: CheckStatus::Info,
            label: "Guest agent".into(),
            detail: "non applicable (bare metal)".into(),
        };
    }

    if vm.qemu_guest_agent {
        Check {
            status: CheckStatus::Ok,
            label: "Guest agent".into(),
            detail: "qemu-guest-agent actif".into(),
        }
    } else if matches!(
        vm.kind,
        grul_common::vm::VirtKind::Kvm | grul_common::vm::VirtKind::Qemu
    ) {
        Check {
            status: CheckStatus::Warn,
            label: "Guest agent".into(),
            detail: "installez qemu-guest-agent (grul-doctor vm-setup)".into(),
        }
    } else {
        Check {
            status: CheckStatus::Info,
            label: "Guest agent".into(),
            detail: "non requis pour ce type de VM".into(),
        }
    }
}

fn command_exists(cmd: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {cmd}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn is_root() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

pub fn path_exists(p: &str) -> bool {
    Path::new(p).exists()
}

pub fn read_file_lossy(path: &str) -> Option<String> {
    fs::read_to_string(path).ok()
}
