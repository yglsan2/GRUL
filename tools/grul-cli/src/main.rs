//! GRUL — point d'entrée unique (marque GRUL, une commande : `grul`).

mod dispatch;
mod help_text;
mod info;

use clap::{Parser, Subcommand};
use dispatch::run_command;

#[derive(Parser)]
#[command(
    name = "grul",
    version,
    about = "GRUL — la meilleure Debian pour développer et administrer",
    long_about = "Une seule commande pour installer, optimiser, mettre à jour et diagnostiquer.\n\
                  Compatible Debian totale — bash, apt et systemd restent au cœur du système."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Diagnostic système, score /100, recommandations
    Doctor {
        #[command(subcommand)]
        action: Option<DoctorAction>,
    },

    /// Mise à jour complète (refresh + upgrade + nettoyage)
    Update {
        #[arg(long, short = 'y')]
        yes: bool,
        #[arg(long)]
        dry_run: bool,
    },

    /// Alias de update — mise à niveau système
    Upgrade {
        #[arg(long, short = 'y')]
        yes: bool,
        #[arg(long)]
        dry_run: bool,
    },

    /// Optimisation automatique selon le matériel
    Optimize {
        #[arg(long, short = 'y')]
        yes: bool,
        #[arg(long)]
        dry_run: bool,
    },

    /// État global GRUL + mises à jour
    Status,

    /// Informations système et version GRUL
    Info,

    /// Rollback vers le dernier snapshot (Btrfs, si grul-snap actif)
    Rollback {
        #[arg(long)]
        dry_run: bool,
    },

    /// Outils machines virtuelles
    Vm {
        #[command(subcommand)]
        action: VmAction,
    },

    /// Réparation système (apt, grub, paquets cassés…)
    Repair {
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        vacuum_journal: bool,
    },

    /// Mesures performance (CPU, RAM, disque, réseau)
    Benchmark,

    /// Snapshots et sauvegarde
    Backup {
        #[arg(long)]
        dry_run: bool,
    },

    /// Restauration depuis snapshot
    Restore {
        #[arg(long)]
        last: bool,
        #[arg(long)]
        dry_run: bool,
    },

    /// Pilotes et guest agents
    Drivers {
        #[command(subcommand)]
        action: Option<DriversAction>,
    },

    /// État sécurité et mises à jour critiques
    Security,

    /// Journaux utiles au diagnostic
    Logs,

    /// Services systemd (état, profil GRUL)
    Services,

    /// Paquets et canaux GRUL
    Packages,

    /// Nettoyage (cache apt, journaux…)
    Clean {
        #[arg(long)]
        dry_run: bool,
    },

    /// Désinstallation propre de la couche GRUL
    Uninstall,

    /// Installation interactive GRUL sur Debian (v0.2)
    Install,

    /// Aide détaillée par commande
    Help {
        #[arg(value_name = "COMMANDE")]
        topic: Option<String>,
    },
}

#[derive(Subcommand)]
enum DriversAction {
    /// Installe les guest agents selon l'hyperviseur détecté
    Install {
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum DoctorAction {
    /// Check rapide
    Quick,
    /// Configuration optimale VM
    VmSetup {
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum VmAction {
    /// Détecte l'hyperviseur et le profil recommandé
    Detect,
    /// Optimise la VM (profil, guest agent, sécurité auto)
    Optimize {
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Réduit l'empreinte disque (v0.5)
    Compact,
    /// Prépare un clone/template (v0.5)
    Clone,
    /// Export image (v0.5)
    Export,
    /// Nettoyage spécifique VM
    Clean {
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run_command(cli.command) {
        eprintln!("Erreur: {e}");
        eprintln!("Aide : grul help");
        std::process::exit(1);
    }
}
