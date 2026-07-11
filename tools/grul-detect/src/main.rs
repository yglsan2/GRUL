//! grul-detect — analyse matérielle, VM et recommandation de profil GRUL.

use grul_common::vm::{detect_vm, VirtKind};
use grul_common::ProfileId;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareReport {
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub ram_mb: u64,
    pub disk_type: DiskType,
    pub gpu_hint: String,
    pub vm: grul_common::vm::VmReport,
    pub recommended_profile: ProfileId,
    pub rationale: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DiskType {
    Ssd,
    Hdd,
    Nvme,
    Unknown,
}

pub fn detect() -> HardwareReport {
    let cpu_model = read_cpu_model();
    let cpu_cores = read_cpu_cores();
    let ram_mb = read_ram_mb();
    let disk_type = detect_root_disk_type();
    let gpu_hint = detect_gpu_hint();
    let vm = detect_vm();
    let (recommended_profile, rationale) =
        recommend_profile(cpu_cores, ram_mb, disk_type, &gpu_hint, &vm);

    HardwareReport {
        cpu_model,
        cpu_cores,
        ram_mb,
        disk_type,
        gpu_hint,
        vm,
        recommended_profile,
        rationale,
    }
}

fn read_cpu_model() -> String {
    read_linux_first_match("/proc/cpuinfo", "model name")
        .or_else(|| read_linux_first_match("/proc/cpuinfo", "Model"))
        .unwrap_or_else(|| "Unknown CPU".into())
}

fn read_cpu_cores() -> u32 {
    fs::read_to_string("/proc/cpuinfo")
        .map(|s| s.lines().filter(|l| l.starts_with("processor")).count() as u32)
        .unwrap_or(1)
        .max(1)
}

fn read_ram_mb() -> u64 {
    fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("MemTotal:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|k| k.parse::<u64>().ok())
        })
        .map(|kb| kb / 1024)
        .unwrap_or(4096)
}

fn read_linux_first_match(path: &str, key: &str) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    content
        .lines()
        .find(|l| l.starts_with(key))
        .map(|l| {
            l.split_once(':')
                .map(|(_, v)| v.trim().to_string())
                .unwrap_or_default()
        })
        .filter(|s| !s.is_empty())
}

fn detect_root_disk_type() -> DiskType {
    let root_dev = fs::read_to_string("/proc/mounts").ok().and_then(|s| {
        s.lines()
            .find(|l| {
                let parts: Vec<_> = l.split_whitespace().collect();
                parts.get(1).map(|m| *m == "/").unwrap_or(false)
            })
            .and_then(|l| l.split_whitespace().next().map(String::from))
    });

    let Some(dev) = root_dev else {
        return DiskType::Unknown;
    };

    let block = dev
        .trim_start_matches("/dev/")
        .split('p')
        .next()
        .unwrap_or("")
        .to_string();

    if block.is_empty() {
        return DiskType::Unknown;
    }
    if block.starts_with("nvme") {
        return DiskType::Nvme;
    }

    let rotational_path = format!("/sys/block/{block}/queue/rotational");
    match fs::read_to_string(&rotational_path) {
        Ok(v) if v.trim() == "0" => DiskType::Ssd,
        Ok(v) if v.trim() == "1" => DiskType::Hdd,
        _ => DiskType::Unknown,
    }
}

fn detect_gpu_hint() -> String {
    if let Ok(vendor) = fs::read_to_string("/sys/class/drm/card0/device/vendor") {
        let v = vendor.trim();
        return match v {
            "0x10de" => "NVIDIA".into(),
            "0x1002" => "AMD".into(),
            "0x8086" => "Intel".into(),
            _ => format!("PCI vendor {v}"),
        };
    }
    "Unknown GPU".into()
}

pub fn recommend_profile(
    cores: u32,
    ram_mb: u64,
    disk: DiskType,
    gpu: &str,
    vm: &grul_common::vm::VmReport,
) -> (ProfileId, Vec<String>) {
    let mut rationale = Vec::new();

    if vm.is_virtual {
        if vm.kind == VirtKind::Container {
            rationale.push(format!(
                "Environnement conteneur ({}) — server-minimal",
                vm.kind.label()
            ));
            return (ProfileId::ServerMinimal, rationale);
        }
        rationale.push(format!(
            "Machine virtuelle {} ({}) — profil vm-minimal",
            vm.kind.label(),
            vm.hypervisor
        ));
        if vm.cloud_init {
            rationale.push("cloud-init détecté — adapté aux images cloud/VM".into());
        }
        return (ProfileId::VmMinimal, rationale);
    }

    if ram_mb >= 32_768 && cores >= 8 {
        rationale.push(format!("{ram_mb} Mo RAM et {cores} cœurs — charge dev/gaming possible"));
        if gpu == "NVIDIA" || gpu == "AMD" {
            rationale.push(format!("GPU {gpu} — profil gaming-latency"));
            return (ProfileId::GamingLatency, rationale);
        }
        rationale.push("Pas de GPU dédié clair — dev-performance".into());
        return (ProfileId::DevPerformance, rationale);
    }

    if ram_mb <= 2048 && cores <= 2 {
        rationale.push("Ressources limitées — server-minimal".into());
        return (ProfileId::ServerMinimal, rationale);
    }

    match disk {
        DiskType::Nvme | DiskType::Ssd => {
            rationale.push("Stockage flash — desktop-balanced optimisé I/O".into());
        }
        DiskType::Hdd => {
            rationale.push("HDD — swappiness conservatrice".into());
        }
        DiskType::Unknown => {
            rationale.push("Disque inconnu — profil par défaut".into());
        }
    }

    (ProfileId::DesktopBalanced, rationale)
}

fn main() {
    let json = std::env::args().any(|a| a == "--json");
    let report = detect();

    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        print_human(&report);
    }
}

fn print_human(r: &HardwareReport) {
    println!("GRUL Hardware Report");
    println!("====================");
    println!("CPU     : {} ({} cœurs)", r.cpu_model, r.cpu_cores);
    println!("RAM     : {} Mo", r.ram_mb);
    println!("Disque  : {:?}", r.disk_type);
    println!("GPU     : {}", r.gpu_hint);
    if r.vm.is_virtual {
        println!(
            "VM      : {} ({})",
            r.vm.kind.label(),
            r.vm.hypervisor
        );
        println!("Vendor  : {} / {}", r.vm.system_vendor, r.vm.product_name);
    } else {
        println!("VM      : non (bare metal)");
    }
    println!();
    println!("Profil recommandé : {}", r.recommended_profile.as_str());
    println!();
    println!("Justification :");
    for line in &r.rationale {
        println!("  • {line}");
    }
    println!();
    println!(
        "Appliquer : sudo grul-tune apply --profile {}",
        r.recommended_profile.as_str()
    );
    if r.vm.is_virtual {
        println!("VM setup  : sudo grul-doctor vm-setup");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grul_common::vm::{VmReport, VirtKind};

    #[test]
    fn profile_vm_on_kvm() {
        let vm = VmReport {
            is_virtual: true,
            kind: VirtKind::Kvm,
            hypervisor: "kvm".into(),
            product_name: "Standard PC".into(),
            system_vendor: "QEMU".into(),
            cloud_init: false,
            qemu_guest_agent: false,
        };
        let (p, _) = recommend_profile(4, 8192, DiskType::Ssd, "Unknown GPU", &vm);
        assert_eq!(p, ProfileId::VmMinimal);
    }

    #[test]
    fn profile_dev_with_ram() {
        let vm = VmReport {
            is_virtual: false,
            kind: VirtKind::None,
            hypervisor: "none".into(),
            product_name: String::new(),
            system_vendor: String::new(),
            cloud_init: false,
            qemu_guest_agent: false,
        };
        let (p, _) = recommend_profile(16, 65536, DiskType::Nvme, "Intel", &vm);
        assert_eq!(p, ProfileId::DevPerformance);
    }
}
