mod apply;

use apply::{apply_profile, reset_profile, show_status, ApplyOptions};
use clap::{Parser, Subcommand};
use grul_common::{load_profile, ProfileId};
use std::process::Command;

#[derive(Parser)]
#[command(name = "grul-tune", about = "Applique ou annule les profils d'optimisation GRUL")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Applique un profil (sysctl, services, fstrim)
    Apply {
        /// ID du profil (ex. desktop-balanced)
        #[arg(long)]
        profile: Option<String>,

        /// Utilise grul-detect pour choisir le profil
        #[arg(long, conflicts_with = "profile")]
        auto: bool,

        /// Affiche les actions sans les exécuter
        #[arg(long)]
        dry_run: bool,

        /// Sans confirmation interactive
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Annule les réglages GRUL
    Reset {
        #[arg(long)]
        dry_run: bool,
    },

    /// Affiche le profil actuellement appliqué
    Status,

    /// Liste les profils disponibles
    List,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Erreur: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Apply {
            profile,
            auto,
            dry_run,
            yes,
        } => {
            let profile_id = resolve_profile_id(profile, auto)?;
            let grul_profile = load_profile(&profile_id)?;

            println!("Profil : {} — {}", grul_profile.profile.id, grul_profile.profile.description);
            if dry_run {
                println!("Mode dry-run (aucune modification)\n");
            }

            let opts = ApplyOptions { dry_run, yes };
            let result = apply_profile(&grul_profile, &opts)?;

            println!("Actions :");
            for action in &result.actions {
                println!("  • {action}");
            }

            if !dry_run {
                println!("\nProfil {} appliqué.", result.profile_id);
                print_package_hints(&grul_profile);
            }
        }

        Commands::Reset { dry_run } => {
            let opts = ApplyOptions {
                dry_run,
                yes: true,
            };
            let actions = reset_profile(&opts)?;
            for action in actions {
                println!("  • {action}");
            }
        }

        Commands::Status => {
            print!("{}", show_status()?);
        }

        Commands::List => {
            list_profiles()?;
        }
    }

    Ok(())
}

fn resolve_profile_id(profile: Option<String>, auto: bool) -> Result<String, String> {
    if let Some(id) = profile {
        if ProfileId::from_str(&id).is_none() {
            return Err(format!("profil inconnu: {id}"));
        }
        return Ok(id);
    }
    if auto {
        return detect_profile_via_binary();
    }
    Err("spécifiez --profile <id> ou --auto".into())
}

fn detect_profile_via_binary() -> Result<String, String> {
    let output = Command::new("grul-detect")
        .arg("--json")
        .output()
        .map_err(|e| format!("grul-detect introuvable — installez grul-detect ou utilisez --profile: {e}"))?;

    if !output.status.success() {
        return Err("grul-detect a échoué".into());
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    json.get("recommended_profile")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| "recommended_profile absent dans la sortie JSON".into())
}

fn list_profiles() -> Result<(), String> {
    let dir = grul_common::profiles_dir();
    println!("Profils dans {dir:?} :\n");
    for id in [
        "desktop-balanced",
        "dev-performance",
        "server-minimal",
        "gaming-latency",
        "vm-minimal",
    ] {
        match load_profile(id) {
            Ok(p) => println!("  • {} — {}", p.profile.id, p.profile.description),
            Err(_) => println!("  • {id} — (fichier manquant)"),
        }
    }
    Ok(())
}

fn print_package_hints(profile: &grul_common::GrulProfile) {
    if !profile.packages.current.suggest.is_empty() {
        println!("\nPaquets suggérés (canal Current) :");
        for pkg in &profile.packages.current.suggest {
            println!("  • {pkg}");
        }
    }
    if !profile.packages.edge.suggest.is_empty() {
        println!("\nPaquets suggérés (canal Edge, opt-in) :");
        for pkg in &profile.packages.edge.suggest {
            println!("  • {pkg}");
        }
    }
    println!("\nProchaine étape : grul-update status");
}
