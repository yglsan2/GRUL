//! Affichage du statut des mises à jour — style « Ubuntu Software Updater ».

use crate::apt::{collect_upgrade_summary, UpgradeSummary};
use crate::config::{read_release_version, GrulChannelConfig, PackageChannel};
use grul_common::load_applied_state;

pub fn print_status(config: &GrulChannelConfig) -> Result<(), String> {
    let summary = collect_upgrade_summary(config)?;

    println!("GRUL Update — état du système");
    println!("==============================");
    print_release_line();
    println!("Canaux : {}", config.active_channels_label());
    println!();

    if let Some(state) = load_applied_state() {
        println!("Profil actif : {}", state.profile_id);
        println!();
    }

    print_summary_counts(&summary);
    print_channel_breakdown(&summary);

    if summary.total() > 0 {
        println!();
        println!("Exemples de mises à jour :");
        for pkg in summary.packages.iter().take(8) {
            let sec = if pkg.is_security { " [sécurité]" } else { "" };
            println!(
                "  • {} {} → {} ({}){}",
                pkg.name,
                pkg.old_version,
                pkg.new_version,
                pkg.channel.label(),
                sec
            );
        }
        if summary.total() > 8 {
            println!("  … et {} autres paquets", summary.total() - 8);
        }
    }

    print_profile_hints();
    print_next_steps(summary.total());

    Ok(())
}

fn print_release_line() {
    match read_release_version() {
        Some(v) => println!("Version GRUL : {v}"),
        None => println!("Version GRUL : (non définie — /etc/grul/release)"),
    }
}

fn print_summary_counts(summary: &UpgradeSummary) {
    if summary.total() == 0 {
        println!("✓ Système à jour — aucune mise à jour en attente.");
        return;
    }

    println!(
        "{} mise(s) à jour disponible(s) ({} sécurité Debian)",
        summary.total(),
        summary.security_count
    );
}

fn print_channel_breakdown(summary: &UpgradeSummary) {
    if summary.total() == 0 {
        return;
    }

    println!();
    println!("Répartition par canal :");
    for ch in [
        PackageChannel::Core,
        PackageChannel::Current,
        PackageChannel::Edge,
    ] {
        let count = summary.by_channel.get(&ch).copied().unwrap_or(0);
        println!("  • {:<24} : {count}", ch.label());
    }
}

fn print_profile_hints() {
    if let Some(state) = load_applied_state() {
        if let Ok(profile) = grul_common::load_profile(&state.profile_id) {
            let mut hints: Vec<&str> = profile
                .packages
                .current
                .suggest
                .iter()
                .map(String::as_str)
                .collect();
            hints.extend(
                profile
                    .packages
                    .edge
                    .suggest
                    .iter()
                    .map(String::as_str),
            );
            if !hints.is_empty() {
                println!();
                println!("Paquets suggérés par le profil « {} » :", state.profile_id);
                for h in hints {
                    println!("  • {h}");
                }
            }
        }
    }
}

fn print_next_steps(total: usize) {
    println!();
    if total > 0 {
        println!("Appliquer les mises à jour :");
        println!("  sudo grul-update upgrade");
        println!("  sudo grul-update upgrade --edge    # inclure le canal Edge");
        println!("  sudo grul-update upgrade -y         # sans confirmation");
    } else {
        println!("Vérifier une nouvelle version GRUL :");
        println!("  grul-update release-check");
    }
}
