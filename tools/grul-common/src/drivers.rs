//! Plans d'installation guest agents / pilotes par hyperviseur — CdC §11.

use crate::vm::VirtKind;

#[derive(Debug, Clone)]
pub struct DriverPlan {
    pub packages: Vec<&'static str>,
    pub services: Vec<&'static str>,
    pub notes: Vec<String>,
}

impl DriverPlan {
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty() && self.services.is_empty()
    }
}

pub fn plan_label(kind: VirtKind) -> &'static str {
    match kind {
        VirtKind::Kvm | VirtKind::Qemu => "QEMU Guest Agent (VirtIO)",
        VirtKind::Vmware => "open-vm-tools",
        VirtKind::Virtualbox => "VirtualBox Guest Additions (utilisateur)",
        VirtKind::Hyperv | VirtKind::Microsoft => "Hyper-V Integration Services",
        VirtKind::Amazon | VirtKind::Google => "cloud-init + agents cloud",
        _ => "guest tools selon plateforme",
    }
}

pub fn plan_for(kind: VirtKind) -> DriverPlan {
    match kind {
        VirtKind::Kvm | VirtKind::Qemu => DriverPlan {
            packages: vec!["qemu-guest-agent"],
            services: vec!["qemu-guest-agent.service"],
            notes: vec!["VirtIO recommandé côté hyperviseur".into()],
        },
        VirtKind::Vmware => DriverPlan {
            packages: vec!["open-vm-tools"],
            services: vec!["open-vm-tools.service"],
            notes: vec![],
        },
        VirtKind::Virtualbox => DriverPlan {
            packages: vec!["virtualbox-guest-utils"],
            services: vec!["vboxadd-service.service"],
            notes: vec![
                "Installez aussi les Guest Additions ISO VirtualBox si besoin".into(),
            ],
        },
        VirtKind::Hyperv | VirtKind::Microsoft => DriverPlan {
            packages: vec!["hyperv-daemons", "hyperv-tools"],
            services: vec!["hyperv-daemons.service"],
            notes: vec!["Modules hv_utils / hv_vmbus chargés si disponibles".into()],
        },
        VirtKind::Amazon | VirtKind::Google => DriverPlan {
            packages: vec!["cloud-init"],
            services: vec![],
            notes: vec!["Images cloud — agents souvent préinstallés".into()],
        },
        VirtKind::Xen => DriverPlan {
            packages: vec!["xe-guest-utilities"],
            services: vec!["xen-guest-agent.service"],
            notes: vec![],
        },
        VirtKind::Parallels => DriverPlan {
            packages: vec!["parallels-tools"],
            services: vec![],
            notes: vec!["Paquet propriétaire — peut nécessiter l'ISO Parallels".into()],
        },
        VirtKind::Container => DriverPlan {
            packages: vec![],
            services: vec![],
            notes: vec!["Conteneur — pas de guest agent classique".into()],
        },
        VirtKind::None | VirtKind::Unknown => DriverPlan {
            packages: vec![],
            services: vec![],
            notes: vec![],
        },
    }
}
