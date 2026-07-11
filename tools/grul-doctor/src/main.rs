mod report;
mod vm_setup;

use clap::{Parser, Subcommand};
use report::{print_report, run_full_report, run_quick_report};
use vm_setup::{run_vm_setup, VmSetupOptions};

#[derive(Parser)]
#[command(
    name = "grul-doctor",
    version,
    about = "Diagnostics GRUL — santé, setup VM, guide désinstallation"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long, global = true)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Rapport complet (défaut)
    Full,

    /// Vérification rapide
    Quick,

    /// Configure une VM en une commande (profil, guest agent, sécurité auto)
    VmSetup {
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Affiche comment désinstaller GRUL proprement
    UninstallGuide,
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
        None | Some(Commands::Full) => {
            print_report(&run_full_report());
            print_footer();
        }
        Some(Commands::Quick) => {
            print_report(&run_quick_report());
        }
        Some(Commands::VmSetup { yes: _ }) => {
            let opts = VmSetupOptions {
                dry_run: cli.dry_run,
                yes: true,
            };
            println!("GRUL VM Setup");
            println!("=============\n");
            for line in run_vm_setup(&opts)? {
                println!("  • {line}");
            }
        }
        Some(Commands::UninstallGuide) => {
            print_uninstall_guide();
        }
    }

    Ok(())
}

fn print_footer() {
    let vm = grul_common::vm::detect_vm();
    println!();
    if vm.is_virtual {
        println!("Commandes VM utiles :");
        println!("  sudo grul-doctor vm-setup     # configuration optimale");
        println!("  sudo grul-update upgrade -y   # mises à jour");
        println!("  sudo bash scripts/uninstall-grul.sh  # désinstallation");
    } else {
        println!("Prochaines étapes :");
        println!("  sudo grul-tune apply --auto");
        println!("  grul-update status");
    }
}

fn print_uninstall_guide() {
    println!("GRUL — guide de désinstallation");
    println!("================================");
    println!();
    println!("GRUL ne modifie pas Debian en profondeur. Désinstallation en 3 étapes :");
    println!();
    println!("  1. Annuler le tuning :");
    println!("       sudo grul-tune reset");
    println!();
    println!("  2. Désinstaller les paquets (si installés via .deb) :");
    println!("       sudo apt remove grul-core grul-detect grul-tune grul-update grul-doctor grul-snap");
    println!();
    println!("  3. Script complet (binaires + config) :");
    println!("       sudo bash /usr/share/grul/scripts/uninstall-grul.sh");
    println!("     ou depuis le repo :");
    println!("       sudo bash scripts/uninstall-grul.sh");
    println!();
    println!("Le script conserve une sauvegarde dans /var/lib/grul/backup/ avant suppression.");
    println!("Debian reste intact — seule la couche GRUL est retirée.");
}
