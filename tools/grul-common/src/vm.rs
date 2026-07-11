//! Détection machine virtuelle — QEMU/KVM, VMware, VirtualBox, Hyper-V, cloud.

use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VirtKind {
    None,
    Kvm,
    Qemu,
    Vmware,
    Virtualbox,
    Hyperv,
    Xen,
    Parallels,
    Amazon,
    Google,
    Microsoft,
    Container,
    Unknown,
}

impl VirtKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "bare metal",
            Self::Kvm => "KVM",
            Self::Qemu => "QEMU",
            Self::Vmware => "VMware",
            Self::Virtualbox => "VirtualBox",
            Self::Hyperv => "Hyper-V",
            Self::Xen => "Xen",
            Self::Parallels => "Parallels",
            Self::Amazon => "AWS EC2",
            Self::Google => "Google Compute",
            Self::Microsoft => "Azure",
            Self::Container => "conteneur",
            Self::Unknown => "virtuel (inconnu)",
        }
    }

    pub fn is_virtual(self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmReport {
    pub is_virtual: bool,
    pub kind: VirtKind,
    pub hypervisor: String,
    pub product_name: String,
    pub system_vendor: String,
    pub cloud_init: bool,
    pub qemu_guest_agent: bool,
}

pub fn detect_vm() -> VmReport {
    let (kind, hypervisor) = detect_virt_kind();
    let product_name = read_dmi("product_name");
    let system_vendor = read_dmi("sys_vendor");
    let cloud_init = path_exists("/run/cloud-init");
    let qemu_guest_agent = command_exists("qemu-guest-agent")
        && systemd_unit_active("qemu-guest-agent.service");

    VmReport {
        is_virtual: kind.is_virtual(),
        kind,
        hypervisor,
        product_name,
        system_vendor,
        cloud_init,
        qemu_guest_agent,
    }
}

fn detect_virt_kind() -> (VirtKind, String) {
    if let Some(v) = systemd_detect_virt() {
        return v;
    }

    let vendor = read_dmi("sys_vendor").to_lowercase();
    let product = read_dmi("product_name").to_lowercase();
    let bios = read_dmi("bios_vendor").to_lowercase();

    if vendor.contains("amazon ec2") || product.contains("amazon ec2") {
        return (VirtKind::Amazon, "Amazon EC2".into());
    }
    if vendor.contains("google") || product.contains("google compute") {
        return (VirtKind::Google, "Google Compute Engine".into());
    }
    if vendor.contains("microsoft corporation") && product.contains("virtual machine") {
        return (VirtKind::Microsoft, "Microsoft Azure VM".into());
    }
    if product.contains("vmware") || vendor.contains("vmware") {
        return (VirtKind::Vmware, "VMware".into());
    }
    if product.contains("virtualbox") || vendor.contains("innotek") {
        return (VirtKind::Virtualbox, "VirtualBox".into());
    }
    if vendor.contains("qemu") || product.contains("standard pc") && bios.contains("seabios") {
        return (VirtKind::Qemu, "QEMU".into());
    }
    if read_cpuinfo_hypervisor() {
        return (VirtKind::Unknown, "hypervisor CPU flag".into());
    }

    (VirtKind::None, "none".into())
}

fn systemd_detect_virt() -> Option<(VirtKind, String)> {
    let output = Command::new("systemd-detect-virt").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
    if raw.is_empty() || raw == "none" {
        return None;
    }

    let kind = match raw.as_str() {
        "kvm" => VirtKind::Kvm,
        "qemu" => VirtKind::Qemu,
        "vmware" => VirtKind::Vmware,
        "oracle" | "virtualbox" => VirtKind::Virtualbox,
        "microsoft" | "hyperv" => VirtKind::Hyperv,
        "xen" => VirtKind::Xen,
        "parallels" => VirtKind::Parallels,
        "amazon" => VirtKind::Amazon,
        "google" => VirtKind::Google,
        "container" | "lxc" | "openvz" | "podman" | "docker" => VirtKind::Container,
        _ => VirtKind::Unknown,
    };

    Some((kind, raw))
}

fn read_cpuinfo_hypervisor() -> bool {
    fs::read_to_string("/proc/cpuinfo")
        .map(|s| {
            s.lines()
                .any(|l| l.starts_with("flags") && l.contains(" hypervisor "))
        })
        .unwrap_or(false)
}

fn read_dmi(field: &str) -> String {
    let path = format!("/sys/class/dmi/id/{field}");
    fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn path_exists(p: &str) -> bool {
    fs::metadata(p).is_ok()
}

fn command_exists(cmd: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {cmd}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn systemd_unit_active(unit: &str) -> bool {
    Command::new("systemctl")
        .args(["is-active", "--quiet", unit])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
