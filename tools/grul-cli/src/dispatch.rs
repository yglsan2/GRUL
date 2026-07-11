//! Exécution des sous-commandes — délègue aux outils GRUL existants.

use crate::help_text;
use crate::info;
use crate::{Commands, DoctorAction, VmAction};
use std::process::{Command, ExitStatus};

pub fn run_command(cmd: Commands) -> Result<(), String> {
    match cmd {
        Commands::Doctor { action } => match action {
            None => exec("grul-doctor", &[]),
            Some(DoctorAction::Quick) => exec("grul-doctor", &["quick"]),
            Some(DoctorAction::VmSetup { yes }) => {
                let mut args = vec!["vm-setup"];
                if yes {
                    args.push("--yes");
                }
                exec("grul-doctor", &args)
            }
        },

        Commands::Update { yes, dry_run } | Commands::Upgrade { yes, dry_run } => {
            run_full_update(yes, dry_run)
        }

        Commands::Optimize { yes, dry_run } => {
            if dry_run {
                println!("[dry-run] grul-tune apply --auto");
                return Ok(());
            }
            let mut args = vec!["apply", "--auto"];
            if yes {
                args.push("--yes");
            }
            exec_or_sudo("grul-tune", &args)
        }

        Commands::Status => {
            exec("grul-update", &["status"])?;
            println!();
            exec("grul-doctor", &["quick"])
        }

        Commands::Info => {
            info::print_info();
            Ok(())
        }

        Commands::Rollback { dry_run } => {
            let mut args = vec!["rollback", "--last"];
            if dry_run {
                args.push("--dry-run");
            }
            exec_or_sudo("grul-snap", &args)
        }

        Commands::Vm { action } => match action {
            VmAction::Detect => exec("grul-detect", &[]),
            VmAction::Optimize { yes } => {
                let mut args = vec!["vm-setup"];
                if yes {
                    args.push("--yes");
                }
                exec_or_sudo("grul-doctor", &args)
            }
            VmAction::Compact => stub("vm compact", "v0.5", "grul vm compact — réduction image disque"),
            VmAction::Clone => stub("vm clone", "v0.5", "préparation template golden"),
            VmAction::Export => stub("vm export", "v0.5", "export qcow2/raw"),
            VmAction::Clean { dry_run } => {
                if dry_run {
                    println!("[dry-run] grul clean — apt clean, journal vacuum");
                }
                stub_impl("vm clean", &["apt-get", "clean"], dry_run)
            }
        },

        Commands::Repair { dry_run } => {
            if dry_run {
                println!("[dry-run] grul repair — dpkg --configure -a, apt -f install");
                return Ok(());
            }
            stub_impl(
                "repair",
                &[],
                false,
            )?;
            exec_or_sudo("dpkg", &["--configure", "-a"])?;
            exec_or_sudo("apt-get", &["install", "-f", "-y"])
        }

        Commands::Benchmark => stub("benchmark", "v0.4", "grul benchmark — CPU, RAM, SSD, boot time"),

        Commands::Backup { dry_run } => {
            let mut args = vec!["create", "--reason", "manual-backup"];
            if dry_run {
                args.push("--dry-run");
            }
            exec_or_sudo("grul-snap", &args)
        }

        Commands::Restore { last: _, dry_run } => {
            let mut args = vec!["rollback", "--last"];
            if dry_run {
                args.push("--dry-run");
            }
            exec_or_sudo("grul-snap", &args)
        }

        Commands::Drivers => {
            let vm = grul_common::vm::detect_vm();
            if vm.is_virtual {
                println!("Hyperviseur : {}", vm.kind.label());
                println!("Guest agent QEMU : {}", if vm.qemu_guest_agent { "actif" } else { "inactif" });
                println!();
                println!("Optimiser : sudo grul vm optimize");
            } else {
                println!("Bare metal — pilotes via apt/Debian.");
                println!("GPU : installez mesa-vulkan-drivers ou pilote propriétaire si besoin.");
            }
            Ok(())
        }

        Commands::Security => exec("grul-update", &["upgrade", "--security-only", "--dry-run"]),

        Commands::Logs => {
            exec("journalctl", &["-p", "err", "-b", "--no-pager", "-n", "30"])
        }

        Commands::Services => {
            exec("grul-tune", &["status"])?;
            println!();
            exec("systemctl", &["--failed", "--no-pager"])
        }

        Commands::Packages => exec("grul-update", &["status"]),

        Commands::Clean { dry_run } => {
            if dry_run {
                println!("[dry-run] apt-get autoremove -y && apt-get clean");
                return Ok(());
            }
            exec_or_sudo("apt-get", &["autoremove", "-y"])?;
            exec_or_sudo("apt-get", &["clean"])
        }

        Commands::Uninstall => exec("grul-doctor", &["uninstall-guide"]),

        Commands::Help { topic } => {
            help_text::print_help(topic.as_deref());
            Ok(())
        }
    }
}

fn run_full_update(yes: bool, dry_run: bool) -> Result<(), String> {
    if dry_run {
        println!("[dry-run] grul update");
        println!("  → grul-update refresh");
        println!("  → grul-update upgrade");
        println!("  → apt-get autoremove && apt-get clean");
        return Ok(());
    }

    exec_or_sudo("grul-update", &["refresh"])?;

    let mut args = vec!["upgrade"];
    if yes {
        args.push("-y");
    }
    exec_or_sudo("grul-update", &args)?;

    let _ = exec_or_sudo("apt-get", &["autoremove", "-y"]);
    let _ = exec_or_sudo("apt-get", &["clean"]);

    println!();
    println!("✓ Mise à jour GRUL terminée.");
    Ok(())
}

fn exec(cmd: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(cmd).args(args).status();
    map_status(cmd, status)
}

fn exec_or_sudo(cmd: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(cmd).args(args).status();
    if map_status(cmd, status.clone()).is_err() {
        // Retry with sudo if permission denied pattern
        let mut sudo_args = vec![cmd];
        sudo_args.extend(args.iter().copied());
        let status = Command::new("sudo").args(&sudo_args).status();
        return map_status(&format!("sudo {cmd}"), status);
    }
    Ok(())
}

fn map_status(cmd: &str, status: Result<ExitStatus, std::io::Error>) -> Result<(), String> {
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("{cmd} a échoué (code {:?})", s.code())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(format!("{cmd} introuvable — installez grul-core ou scripts/install-tools.sh"))
        }
        Err(e) => Err(format!("{cmd}: {e}")),
    }
}

fn stub(name: &str, version: &str, desc: &str) -> Result<(), String> {
    println!("{name} — prévu {version}");
    println!("  {desc}");
    println!("  Voir docs/CAHIER-DES-CHARGES.md et docs/ROADMAP.md");
    Ok(())
}

fn stub_impl(name: &str, extra: &[&str], dry_run: bool) -> Result<(), String> {
    if dry_run {
        println!("[dry-run] {name}");
        return Ok(());
    }
    for cmd in extra {
        let parts: Vec<_> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        exec_or_sudo(parts[0], &parts[1..])?;
    }
    Ok(())
}
