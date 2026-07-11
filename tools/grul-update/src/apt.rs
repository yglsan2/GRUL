//! Interaction avec apt — refresh, liste des mises à jour, upgrade.

use crate::config::{GrulChannelConfig, PackageChannel};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

#[derive(Debug, Clone)]
pub struct UpgradablePackage {
    pub name: String,
    pub new_version: String,
    pub old_version: String,
    pub channel: PackageChannel,
    pub is_security: bool,
}

#[derive(Debug, Clone, Default)]
pub struct UpgradeSummary {
    pub packages: Vec<UpgradablePackage>,
    pub security_count: usize,
    pub by_channel: HashMap<PackageChannel, usize>,
}

impl UpgradeSummary {
    pub fn total(&self) -> usize {
        self.packages.len()
    }
}

pub struct AptOptions {
    pub dry_run: bool,
    pub yes: bool,
    pub security_only: bool,
    pub full_upgrade: bool,
    pub include_edge: bool,
}

pub fn is_root() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

pub fn require_root_for_apply(dry_run: bool) -> Result<(), String> {
    if dry_run {
        return Ok(());
    }
    if !is_root() {
        return Err("root requis — relancez avec sudo".into());
    }
    Ok(())
}

pub fn apt_available() -> bool {
    Command::new("apt-get")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn run_refresh(dry_run: bool) -> Result<Vec<String>, String> {
    if dry_run {
        return Ok(vec!["[dry-run] apt-get update".into()]);
    }
    require_root_for_apply(false)?;
    let out = run_apt_get(&["update", "-q", "-y"])?;
    Ok(vec![format!(
        "Index APT actualisés (code {})",
        out.status.code().unwrap_or(-1)
    )])
}

pub fn collect_upgrade_summary(config: &GrulChannelConfig) -> Result<UpgradeSummary, String> {
    if !apt_available() {
        return Ok(UpgradeSummary::default());
    }

    let security_set = load_security_package_names();
    let raw = run_apt_list_upgradable()?;
    let mut summary = UpgradeSummary::default();

    for (name, new_ver, old_ver) in raw {
        let channel = classify_package_channel(config, &name);
        let is_security = security_set.contains(&name);
        if is_security {
            summary.security_count += 1;
        }
        *summary.by_channel.entry(channel).or_insert(0) += 1;
        summary.packages.push(UpgradablePackage {
            name,
            new_version: new_ver,
            old_version: old_ver,
            channel,
            is_security,
        });
    }

    Ok(summary)
}

pub fn run_upgrade(
    config: &GrulChannelConfig,
    opts: &AptOptions,
    summary: &UpgradeSummary,
) -> Result<Vec<String>, String> {
    let mut actions = Vec::new();

    if summary.total() == 0 && !opts.dry_run {
        actions.push("Système déjà à jour.".into());
        return Ok(actions);
    }

    let mut packages: Vec<String> = summary
        .packages
        .iter()
        .filter(|p| package_allowed(config, p, opts))
        .map(|p| p.name.clone())
        .collect();

    if opts.security_only {
        packages.retain(|name| {
            summary
                .packages
                .iter()
                .any(|p| p.name == *name && p.is_security)
        });
        if packages.is_empty() {
            actions.push("Aucune mise à jour de sécurité en attente.".into());
            return Ok(actions);
        }
    }

    if opts.dry_run {
        let cmd = if opts.full_upgrade {
            "apt-get full-upgrade -s"
        } else if opts.security_only {
            "apt-get install --only-upgrade (sécurité)"
        } else {
            "apt-get upgrade -s"
        };
        actions.push(format!("[dry-run] {cmd}"));
        for pkg in packages.iter().take(20) {
            actions.push(format!("  → {pkg}"));
        }
        if packages.len() > 20 {
            actions.push(format!("  … et {} autres paquets", packages.len() - 20));
        }
        return Ok(actions);
    }

    require_root_for_apply(false)?;

    if opts.security_only {
        if packages.is_empty() {
            actions.push("Aucune mise à jour de sécurité.".into());
            return Ok(actions);
        }
        let mut args = vec!["install", "-y", "--only-upgrade"];
        args.extend(packages.iter().map(String::as_str));
        let out = run_apt_get(&args)?;
        log_apt_result(&mut actions, "install --only-upgrade (sécurité)", &out);
        return Ok(actions);
    }

    let args = if opts.full_upgrade {
        vec!["full-upgrade", "-y"]
    } else {
        vec!["upgrade", "-y"]
    };
    let out = run_apt_get(&args)?;
    log_apt_result(
        &mut actions,
        if opts.full_upgrade {
            "full-upgrade"
        } else {
            "upgrade"
        },
        &out,
    );

    Ok(actions)
}

fn package_allowed(
    config: &GrulChannelConfig,
    pkg: &UpgradablePackage,
    opts: &AptOptions,
) -> bool {
    match pkg.channel {
        PackageChannel::Edge => opts.include_edge && config.channel.edge_enabled,
        PackageChannel::Current => config.channel.current_enabled,
        PackageChannel::Core => config.channel.core_enabled,
    }
}

fn classify_package_channel(config: &GrulChannelConfig, name: &str) -> PackageChannel {
    if let Some(ch) = config.packages.get(name) {
        match ch.as_str() {
            "edge" => return PackageChannel::Edge,
            "current" => return PackageChannel::Current,
            "core" => return PackageChannel::Core,
            _ => {}
        }
    }

    if name.starts_with("grul-") || name.contains("grul") {
        return PackageChannel::Core;
    }

    let policy = apt_cache_policy_origin(name);
    if policy.contains("grul-edge") || policy.contains("grul_edge") {
        return PackageChannel::Edge;
    }
    if policy.contains("grul-current") || policy.contains("grul_current") {
        return PackageChannel::Current;
    }
    if policy.contains("grul-core") || policy.contains("grul_core") {
        return PackageChannel::Core;
    }

    config.package_channel(name)
}

fn apt_cache_policy_origin(package: &str) -> String {
    let output = Command::new("apt-cache")
        .args(["policy", package])
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => String::new(),
    }
}

fn run_apt_list_upgradable() -> Result<Vec<(String, String, String)>, String> {
    let output = Command::new("apt")
        .args(["list", "--upgradable"])
        .output()
        .map_err(|e| format!("apt list: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "apt list --upgradable a échoué: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut out = Vec::new();

    for line in text.lines().skip(1) {
        if line.trim().is_empty() || line.starts_with("Listing...") {
            continue;
        }
        // format: pkg/version suite [upgradable from: old/version]
        let Some((left, rest)) = line.split_once('/') else {
            continue;
        };
        let name = left.trim().to_string();
        let new_version = rest
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string();
        let old_version = if let Some(idx) = line.find("[upgradable from:") {
            line[idx..]
                .trim_start_matches("[upgradable from:")
                .trim_end_matches(']')
                .split('/')
                .next()
                .unwrap_or("?")
                .trim()
                .to_string()
        } else {
            "?".into()
        };
        out.push((name, new_version, old_version));
    }

    Ok(out)
}

fn load_security_package_names() -> HashSet<String> {
    let mut names = HashSet::new();
    let lists_dir = Path::new("/var/lib/apt/lists");
    if !lists_dir.is_dir() {
        return names;
    }

    let Ok(entries) = fs::read_dir(lists_dir) else {
        return names;
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if !file_name.contains("debian-security") && !file_name.contains("_security_") {
            continue;
        }
        if !file_name.ends_with("_Packages") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(entry.path()) {
            for line in content.lines() {
                if let Some(pkg) = line.strip_prefix("Package: ") {
                    names.insert(pkg.trim().to_string());
                }
            }
        }
    }

    names
}

fn run_apt_get(args: &[&str]) -> Result<Output, String> {
    let output = Command::new("apt-get")
        .args(args)
        .env("DEBIAN_FRONTEND", "noninteractive")
        .output()
        .map_err(|e| format!("apt-get {}: {e}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "apt-get {} a échoué ({})\n{stdout}{stderr}",
            args.join(" "),
            output.status.code().unwrap_or(-1)
        ));
    }

    Ok(output)
}

fn log_apt_result(actions: &mut Vec<String>, label: &str, out: &Output) {
    actions.push(format!(
        "apt-get {label} terminé (code {})",
        out.status.code().unwrap_or(-1)
    ));
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines().filter(|l| !l.is_empty()).take(8) {
        actions.push(format!("  {line}"));
    }
}
