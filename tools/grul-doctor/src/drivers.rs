//! Installation des guest agents / pilotes VM — CdC §11.

use grul_common::drivers::{plan_for, plan_label};
use grul_common::vm::{detect_vm, VirtKind};
use std::process::Command;

pub struct DriverInstallOptions {
    pub dry_run: bool,
}

pub fn run_install(opts: &DriverInstallOptions) -> Result<Vec<String>, String> {
    let vm = detect_vm();
    if !vm.is_virtual {
        return Err("grul drivers : machine bare metal — rien à installer".into());
    }

    let plan = plan_for(vm.kind);
    let mut actions = vec![format!(
        "Hyperviseur : {} — {}",
        vm.kind.label(),
        plan_label(vm.kind)
    )];

    for note in &plan.notes {
        actions.push(format!("Note : {note}"));
    }

    if plan.is_empty() {
        actions.push("Aucun paquet guest automatique pour cet hyperviseur.".into());
        return Ok(actions);
    }

    if opts.dry_run {
        for pkg in &plan.packages {
            actions.push(format!("[dry-run] apt-get install -y {pkg}"));
        }
        for svc in &plan.services {
            actions.push(format!("[dry-run] systemctl enable --now {svc}"));
        }
        return Ok(actions);
    }

    require_root()?;

    if !plan.packages.is_empty() {
        let mut args = vec!["install", "-y"];
        args.extend(plan.packages.iter().copied());
        run_apt(&args, &mut actions)?;
    }

    for svc in &plan.services {
        let _ = enable_service(svc, &mut actions);
    }

    if matches!(vm.kind, VirtKind::Hyperv | VirtKind::Microsoft) {
        let _ = Command::new("modprobe").arg("hv_utils").status();
        let _ = Command::new("modprobe").arg("hv_vmbus").status();
        actions.push("Modules Hyper-V chargés si disponibles".into());
    }

    actions.push("Guest tools installés.".into());
    Ok(actions)
}

fn require_root() -> Result<(), String> {
    #[cfg(unix)]
    {
        if unsafe { libc::geteuid() } != 0 {
            return Err("root requis — sudo grul drivers install".into());
        }
    }
    #[cfg(not(unix))]
    {
        return Err("Linux requis".into());
    }
    Ok(())
}

fn run_apt(args: &[&str], actions: &mut Vec<String>) -> Result<(), String> {
    let output = Command::new("apt-get")
        .args(args)
        .env("DEBIAN_FRONTEND", "noninteractive")
        .output()
        .map_err(|e| e.to_string())?;
    actions.push(format!("apt-get {}", args.join(" ")));
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().into());
    }
    Ok(())
}

fn enable_service(unit: &str, actions: &mut Vec<String>) -> Result<(), String> {
    let output = Command::new("systemctl")
        .args(["enable", "--now", unit])
        .output()
        .map_err(|e| e.to_string())?;
    actions.push(format!("systemctl enable --now {unit}"));
    if !output.status.success() {
        actions.push(format!("  (avertissement : {unit} non activé)"));
    }
    Ok(())
}
