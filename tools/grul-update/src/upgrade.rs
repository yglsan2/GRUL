//! Flux d'upgrade GRUL — proche d'Ubuntu : refresh → résumé → confirmation → apt.

use crate::apt::{collect_upgrade_summary, run_refresh, run_upgrade, AptOptions};
use crate::config::{write_channel_sources, GrulChannelConfig, PackageChannel};
use std::io::{self, Write};

pub struct UpgradeFlowOptions {
    pub dry_run: bool,
    pub yes: bool,
    pub security_only: bool,
    pub full_upgrade: bool,
    pub include_edge: bool,
    pub skip_refresh: bool,
}

pub fn run_upgrade_flow(
    config: &GrulChannelConfig,
    opts: &UpgradeFlowOptions,
) -> Result<(), String> {
    print_banner(config);

    if !opts.skip_refresh {
        println!("Étape 1/3 — Actualisation des index APT…");
        for line in run_refresh(opts.dry_run)? {
            println!("  {line}");
        }
        println!();
    }

    if !opts.dry_run {
        for line in write_channel_sources(config, false)? {
            println!("  {line}");
        }
    }

    println!("Étape 2/3 — Analyse des mises à jour…");
    let summary = collect_upgrade_summary(config)?;

    if summary.total() == 0 {
        println!("✓ Rien à mettre à jour. Votre système GRUL est à jour.\n");
        return Ok(());
    }

    println!(
        "{} paquet(s) disponible(s) — {} mise(s) de sécurité Debian",
        summary.total(),
        summary.security_count
    );
    for ch in [
        PackageChannel::Core,
        PackageChannel::Current,
        PackageChannel::Edge,
    ] {
        let n = summary.by_channel.get(&ch).copied().unwrap_or(0);
        if n > 0 {
            println!("  • {} : {n}", ch.label());
        }
    }
    println!();

    if !opts.yes && !opts.dry_run {
        if !confirm_upgrade(&summary, opts)? {
            println!("Mise à jour annulée.");
            return Ok(());
        }
    }

    if config.update.snapshot_before_current && !opts.dry_run && !opts.security_only {
        let current_count = summary
            .by_channel
            .get(&PackageChannel::Current)
            .copied()
            .unwrap_or(0);
        if current_count > 0 {
            println!("Étape 2b — Snapshot optionnel (grul-snap)…");
            for line in snap_hook::maybe_snapshot_before_upgrade(
                config.update.snapshot_before_current,
                current_count > 0,
                opts.dry_run,
            ) {
                println!("  {line}");
            }
            println!();
        }
    }

    println!("Étape 3/3 — Application des mises à jour…");
    let apt_opts = AptOptions {
        dry_run: opts.dry_run,
        yes: opts.yes,
        security_only: opts.security_only,
        full_upgrade: opts.full_upgrade,
        include_edge: opts.include_edge,
    };

    for line in run_upgrade(config, &apt_opts, &summary)? {
        println!("  {line}");
    }

    if !opts.dry_run {
        println!("\n✓ Mise à jour GRUL terminée.");
        if config.channel.edge_enabled {
            println!("Canal Edge actif — vérifiez les outils dev avec : grul-update status");
        }
    }

    Ok(())
}

fn print_banner(config: &GrulChannelConfig) {
    println!();
    println!("╔══════════════════════════════════════╗");
    println!("║         GRUL System Update           ║");
    println!("╚══════════════════════════════════════╝");
    println!("Canaux : {}", config.active_channels_label());
    println!();
}

fn confirm_upgrade(
    summary: &crate::apt::UpgradeSummary,
    opts: &UpgradeFlowOptions,
) -> Result<bool, String> {
    let label = if opts.security_only {
        "mises à jour de sécurité"
    } else if opts.full_upgrade {
        "mise à niveau complète (full-upgrade)"
    } else {
        "mises à jour"
    };

    print!(
        "Installer {} paquet(s) ({}) ? [O/n] ",
        summary.total(),
        label
    );
    io::stdout().flush().map_err(|e| e.to_string())?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    let answer = input.trim().to_lowercase();
    Ok(answer.is_empty() || answer == "o" || answer == "oui" || answer == "y" || answer == "yes")
}
