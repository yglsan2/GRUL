//! Réparation système — CdC §16 (v0.4).

use std::process::Command;

pub struct RepairOptions {
    pub dry_run: bool,
    pub vacuum_journal: bool,
}

pub fn run_repair(opts: &RepairOptions) -> Result<Vec<String>, String> {
    let mut actions = Vec::new();

    if opts.dry_run {
        actions.push("[dry-run] grul repair".into());
        actions.push("[dry-run] dpkg --configure -a".into());
        actions.push("[dry-run] apt-get install -f -y".into());
        actions.push("[dry-run] apt-get check".into());
        actions.push("[dry-run] systemctl reset-failed".into());
        if opts.vacuum_journal {
            actions.push("[dry-run] journalctl --vacuum-time=7d".into());
        }
        return Ok(actions);
    }

    require_root()?;

    step("dpkg --configure -a", &["dpkg", "--configure", "-a"], &mut actions)?;
    step(
        "apt-get install -f -y",
        &["apt-get", "install", "-f", "-y"],
        &mut actions,
    )?;
    step("apt-get check", &["apt-get", "check"], &mut actions)?;

    let _ = Command::new("systemctl")
        .args(["reset-failed"])
        .output();
    actions.push("systemctl reset-failed".into());

    if opts.vacuum_journal {
        let _ = Command::new("journalctl")
            .args(["--vacuum-time=7d"])
            .output();
        actions.push("journalctl --vacuum-time=7d".into());
    }

    actions.push("Réparation GRUL terminée — relancez grul doctor".into());
    Ok(actions)
}

fn step(label: &str, cmd: &[&str], actions: &mut Vec<String>) -> Result<(), String> {
    let output = Command::new(cmd[0])
        .args(&cmd[1..])
        .env("DEBIAN_FRONTEND", "noninteractive")
        .output()
        .map_err(|e| format!("{label}: {e}"))?;
    actions.push(label.into());
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{label} a échoué: {}", err.trim()));
    }
    Ok(())
}

fn require_root() -> Result<(), String> {
    #[cfg(unix)]
    {
        if unsafe { libc::geteuid() } != 0 {
            return Err("root requis — sudo grul repair".into());
        }
    }
    Ok(())
}
