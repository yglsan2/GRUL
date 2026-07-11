//! Appel optionnel à grul-snap depuis grul-update.

use std::process::Command;

pub fn grul_snap_installed() -> bool {
    Command::new("grul-snap")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or_else(|_| {
            // --version may not exist; try status
            Command::new("which")
                .arg("grul-snap")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
}

pub fn maybe_snapshot_before_upgrade(
    snapshot_before_current: bool,
    has_current_updates: bool,
    dry_run: bool,
) -> Vec<String> {
    let mut lines = Vec::new();

    if !snapshot_before_current || dry_run {
        return lines;
    }

    if !has_current_updates {
        return lines;
    }

    if !grul_snap_installed() {
        lines.push(
            "Info : grul-snap non installé — upgrade sans snapshot (installez grul-snap si vous voulez des rollbacks)"
                .into(),
        );
        return lines;
    }

    let mut cmd = Command::new("grul-snap");
    cmd.args(["create", "--reason", "pre-update"]);
    if dry_run {
        cmd.arg("--dry-run");
    }

    match cmd.output() {
        Ok(out) if out.status.success() => {
            let msg = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if msg.is_empty() {
                lines.push("Snapshot pré-upgrade créé (grul-snap)".into());
            } else {
                lines.push(msg);
            }
        }
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
            lines.push(format!(
                "Avertissement snapshot (fail_open) : {err}"
            ));
        }
        Err(e) => {
            lines.push(format!(
                "Avertissement snapshot (fail_open) : grul-snap introuvable ({e})"
            ));
        }
    }

    lines
}
