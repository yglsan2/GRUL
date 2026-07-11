mod btrfs;
mod config;
mod snap;
mod state;

use clap::{Parser, Subcommand};
use config::{GrulSnapConfig, SnapMode};
use snap::{CreateOptions};

#[derive(Parser)]
#[command(
    name = "grul-snap",
    version,
    about = "Snapshots Btrfs optionnels — complément à grul-update, jamais obligatoire",
    long_about = "Installez grul-snap seulement si vous voulez des rollbacks.\n\
                  Sinon, grul-core + grul-update suffisent pour le quotidien."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// État du backend et des snapshots
    Status,

    /// Créer un snapshot (ex. avant mise à jour)
    Create {
        #[arg(long, default_value = "manual")]
        reason: String,

        #[arg(long)]
        dry_run: bool,

        #[arg(long, help = "Ignorer le mode disabled/auto")]
        force: bool,
    },

    /// Lister les snapshots
    List,

    /// Revenir à un snapshot (redémarrage requis)
    Rollback {
        #[arg(long, help = "ID du snapshot (ex. grul-20250620-214100)")]
        id: Option<String>,

        #[arg(long, help = "Dernier snapshot")]
        last: bool,

        #[arg(long)]
        dry_run: bool,
    },

    /// Purger les snapshots au-delà de max_snapshots
    Prune {
        #[arg(long)]
        dry_run: bool,
    },

    /// Activer les snapshots (mode enabled)
    Enable,

    /// Désactiver les snapshots
    Disable,

    /// Mode auto : snapshots seulement si Btrfs détecté
    Auto,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Erreur: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let mut config = GrulSnapConfig::load();

    match cli.command {
        Commands::Status => snap::print_status(&config),

        Commands::Create {
            reason,
            dry_run,
            force,
        } => {
            let result = snap::create_snapshot(
                &config,
                &CreateOptions {
                    reason,
                    dry_run,
                    force,
                },
            )?;
            println!("{}", result.message);
            Ok(())
        }

        Commands::List => snap::list_snapshots(),

        Commands::Rollback { id, last: _, dry_run } => snap::rollback(id, dry_run),

        Commands::Prune { dry_run } => {
            for line in snap::prune(&config, dry_run)? {
                println!("{line}");
            }
            Ok(())
        }

        Commands::Enable => {
            snap::set_mode(&mut config, SnapMode::Enabled)?;
            println!("Snapshots activés (mode enabled).");
            println!("Test : sudo grul-snap create --dry-run");
            Ok(())
        }

        Commands::Disable => {
            snap::set_mode(&mut config, SnapMode::Disabled)?;
            println!("Snapshots désactivés — grul-update fonctionne sans eux.");
            Ok(())
        }

        Commands::Auto => {
            snap::set_mode(&mut config, SnapMode::Auto)?;
            println!("Mode auto : snapshots si Btrfs détecté.");
            Ok(())
        }
    }
}
