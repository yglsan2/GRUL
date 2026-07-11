mod apt;
mod config;
mod release;
mod snap_hook;
mod status;
mod upgrade;

use clap::{Parser, Subcommand};
use config::{save_channel_config, write_channel_sources, GrulChannelConfig};
use std::path::PathBuf;
use upgrade::{run_upgrade_flow, UpgradeFlowOptions};

#[derive(Parser)]
#[command(
    name = "grul-update",
    about = "Mises à jour GRUL — canaux Core / Current / Edge",
    long_about = "Sans sous-commande : affiche le statut puis propose l'upgrade interactif.\n\
                  En pratique : apt update + apt upgrade, avec les canaux GRUL."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Inclure le canal Edge (outils dev)
    #[arg(long, global = true)]
    edge: bool,

    /// Sans confirmation
    #[arg(long, short = 'y', global = true)]
    yes: bool,

    /// Simulation sans modification
    #[arg(long, global = true)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// État des mises à jour disponibles
    Status,

    /// Actualiser les index APT uniquement (apt update)
    Refresh,

    /// Mettre à jour Core + Current (+ Edge si --edge)
    Upgrade {
        /// apt full-upgrade au lieu de upgrade
        #[arg(long)]
        full: bool,

        /// Uniquement les correctifs Debian Security
        #[arg(long)]
        security_only: bool,

        /// Ne pas relancer apt update avant l'upgrade
        #[arg(long)]
        skip_refresh: bool,
    },

    /// Gérer le canal Edge (opt-in)
    Edge {
        #[command(subcommand)]
        action: EdgeAction,
    },

    /// Vérifier si une nouvelle version GRUL est disponible
    ReleaseCheck,

    /// Mettre à niveau vers une nouvelle version GRUL (do-release-upgrade)
    ReleaseUpgrade,
}

#[derive(Subcommand)]
enum EdgeAction {
    /// Activer le dépôt Edge
    Enable,
    /// Désactiver le dépôt Edge
    Disable,
    /// Afficher l'état du canal Edge
    Status,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Erreur: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let mut config = load_config()?;

    match cli.command {
        None => {
            status::print_status(&config)?;
            let summary = apt::collect_upgrade_summary(&config)?;
            if summary.total() > 0 {
                if apt::is_root() && !cli.dry_run {
                    println!();
                    let opts = UpgradeFlowOptions {
                        dry_run: cli.dry_run,
                        yes: cli.yes,
                        security_only: false,
                        full_upgrade: false,
                        include_edge: cli.edge,
                        skip_refresh: false,
                    };
                    run_upgrade_flow(&config, &opts)?;
                } else if summary.total() > 0 {
                    println!();
                    println!("Pour installer les mises à jour : sudo grul-update upgrade");
                }
            }
        }

        Some(Commands::Status) => {
            status::print_status(&config)?;
        }

        Some(Commands::Refresh) => {
            if !cli.dry_run {
                apt::require_root_for_apply(false)?;
            }
            for line in write_channel_sources(&config, cli.dry_run)? {
                println!("{line}");
            }
            for line in apt::run_refresh(cli.dry_run)? {
                println!("{line}");
            }
        }

        Some(Commands::Upgrade {
            full,
            security_only,
            skip_refresh,
        }) => {
            let opts = UpgradeFlowOptions {
                dry_run: cli.dry_run,
                yes: cli.yes,
                security_only,
                full_upgrade: full,
                include_edge: cli.edge || config.channel.edge_enabled,
                skip_refresh,
            };
            run_upgrade_flow(&config, &opts)?;
        }

        Some(Commands::Edge { action }) => {
            handle_edge(&mut config, action, cli.dry_run)?;
        }

        Some(Commands::ReleaseCheck) => {
            release::run_release_check()?;
        }

        Some(Commands::ReleaseUpgrade) => {
            release::run_release_upgrade(cli.dry_run, cli.yes)?;
        }
    }

    Ok(())
}

fn load_config() -> Result<GrulChannelConfig, String> {
    if let Ok(path) = std::env::var("GRUL_CHANNEL_CONFIG") {
        if PathBuf::from(&path).is_file() {
            let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            return toml::from_str(&content).map_err(|e| e.to_string());
        }
    }

    if config::channel_config_path().is_file() {
        return GrulChannelConfig::load();
    }

    // Dev : remonter depuis le cwd pour trouver configs/grul-channel.toml
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            let dev = ancestor.join("configs/grul-channel.toml");
            if dev.is_file() {
                let content = std::fs::read_to_string(&dev).map_err(|e| e.to_string())?;
                return toml::from_str(&content).map_err(|e| e.to_string());
            }
        }
    }

    Ok(GrulChannelConfig::default())
}

fn handle_edge(
    config: &mut GrulChannelConfig,
    action: EdgeAction,
    dry_run: bool,
) -> Result<(), String> {
    match action {
        EdgeAction::Enable => {
            config.channel.edge_enabled = true;
            println!("Canal Edge : activation");
            if !dry_run {
                apt::require_root_for_apply(false)?;
                save_channel_config(config, false)?;
            }
            for line in write_channel_sources(config, dry_run)? {
                println!("  {line}");
            }
            println!();
            println!("Edge activé. Mettez à jour avec : sudo grul-update upgrade --edge");
            print_edge_packages(config);
        }
        EdgeAction::Disable => {
            config.channel.edge_enabled = false;
            println!("Canal Edge : désactivation");
            if !dry_run {
                apt::require_root_for_apply(false)?;
                save_channel_config(config, false)?;
            }
            for line in write_channel_sources(config, dry_run)? {
                println!("  {line}");
            }
            println!("Les paquets Edge déjà installés restent en place.");
        }
        EdgeAction::Status => {
            println!(
                "Canal Edge : {}",
                if config.channel.edge_enabled {
                    "activé ✓"
                } else {
                    "désactivé ✗"
                }
            );
            print_edge_packages(config);
        }
    }
    Ok(())
}

fn print_edge_packages(config: &GrulChannelConfig) {
    let edge: Vec<_> = config
        .packages
        .iter()
        .filter(|(_, ch)| ch.as_str() == "edge")
        .map(|(name, _)| name.as_str())
        .collect();
    if edge.is_empty() {
        println!("Paquets Edge configurés : (aucun override — voir docs/REPOSITORIES.md)");
    } else {
        println!("Paquets Edge configurés :");
        for p in edge {
            println!("  • {p}");
        }
    }
}
